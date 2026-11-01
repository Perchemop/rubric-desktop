//! PDF import helpers: PDFium binding, page-size probing, and page rendering.
//!
//! PDFium is loaded at run time (no native dependency at build time). Binding
//! order is:
//!
//! 1. the `RUBRIC_PDFIUM_PATH` environment variable (a directory containing the
//!    platform library, or the library file itself);
//! 2. the directory containing the running executable, and its `lib/`
//!    subdirectory (some packagers ship the library there);
//! 3. the system library path.
//!
//! Rendered pages are cached as PNG files under the app config directory so the
//! UI can ask for the same page size repeatedly without re-rasterising.
//!
//! When the `hayro` feature is enabled, hayro (pure-Rust) is used for rendering;
//! otherwise PDFium is the default renderer.

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::domain::markup::{Markup, MarkupGeometry};
use crate::domain::style::FillMode;
use directories::ProjectDirs;
use lopdf::Object;
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};

use super::DataError;

/// Environment variable pointing at the PDFium shared library.
///
/// The value may be a directory containing the platform library
/// (`libpdfium.so`, `pdfium.dll`, `libpdfium.dylib`, ...) or the full path to
/// the library file itself. When set, it is authoritative: a failure to load
/// from this path is reported instead of silently falling back to the system
/// library, so misconfiguration is visible.
pub const PDFIUM_PATH_ENV: &str = "RUBRIC_PDFIUM_PATH";

/// Ceiling for the requested page-render width, in pixels.
///
/// Both engines rasterise into pixmaps with limited dimensions: hayro's
/// `vello_cpu` pixmap is addressed with `u16` sizes and pdfium-render limits
/// bitmap dimensions to `u16` as well. A requested width beyond that would
/// either fail obscurely or silently render smaller than the cache file name
/// suggests, so oversized requests are clamped to this documented maximum by
/// [`clamp_render_width`] before anything is rendered or cached.
pub const MAX_RENDER_WIDTH_PX: u32 = 8192;

/// Clamp a requested render width to [`MAX_RENDER_WIDTH_PX`].
fn clamp_render_width(target_width_px: u32) -> u32 {
    target_width_px.min(MAX_RENDER_WIDTH_PX)
}

/// Page sizes read from a PDF, in PDF points and document order.
///
/// Each entry is `(width, height)` as reported by the active engine (PDFium,
/// or hayro when the `hayro` feature is enabled), with identical semantics in
/// both engines: the crop box intersected with the media box, with the page's
/// intrinsic rotation already applied — a `/Rotate 90|270` page reports
/// swapped width and height, matching the orientation of the rendered page
/// bitmap. The domain [`crate::domain::Page::rotation`] field separately
/// tracks the page's `/Rotate` value for the UI.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PdfSourceInfo {
    /// `(width, height)` in PDF points for every page, in document order.
    pub page_sizes: Vec<(f32, f32)>,
}

/// Bind to PDFium, loading the shared library if this is the first call.
///
/// Binding is attempted from [`PDFIUM_PATH_ENV`], then the executable's
/// directory (and its `lib/` subdirectory), then the system library path.
/// PDFium itself initialises its global bindings once per process; subsequent
/// calls reuse those bindings and still return a usable [`Pdfium`] handle.
///
/// Returns [`DataError::PdfLibrary`] when no library can be loaded, with a
/// message that names the environment variable to set.
pub fn bind_pdfium() -> Result<Pdfium, DataError> {
    if let Some(configured) = std::env::var_os(PDFIUM_PATH_ENV) {
        let configured = PathBuf::from(configured);
        let library = if configured.is_dir() {
            Pdfium::pdfium_platform_library_name_at_path(&configured)
        } else {
            configured
        };

        return match pdfium_from_bindings(Pdfium::bind_to_library(&library)) {
            Ok(pdfium) => Ok(pdfium),
            Err(error) => Err(DataError::PdfLibrary(format!(
                "could not load PDFium from {PDFIUM_PATH_ENV} ({}): {error}. \
                 Set {PDFIUM_PATH_ENV} to the directory containing {} or to the \
                 library file itself.",
                library.display(),
                Pdfium::pdfium_platform_library_name().to_string_lossy(),
            ))),
        };
    }

    let mut errors: Vec<String> = Vec::new();

    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        // Packagers may ship the library beside the executable or in a `lib/`
        // subdirectory (see docs/pdfium.md); try both, collecting every
        // failure so the final error names all attempted locations.
        for dir in [exe_dir.clone(), exe_dir.join("lib")] {
            let library = Pdfium::pdfium_platform_library_name_at_path(&dir);
            match pdfium_from_bindings(Pdfium::bind_to_library(&library)) {
                Ok(pdfium) => return Ok(pdfium),
                Err(error) => errors.push(format!("{}: {error}", library.display())),
            }
        }
    } else {
        errors.push("could not determine the executable directory".into());
    }

    match pdfium_from_bindings(Pdfium::bind_to_system_library()) {
        Ok(pdfium) => Ok(pdfium),
        Err(error) => {
            errors.push(format!("system library: {error}"));
            Err(DataError::PdfLibrary(format!(
                "could not load PDFium ({}). Set {PDFIUM_PATH_ENV} to the directory \
                 containing {} or to the library file itself.",
                errors.join("; "),
                Pdfium::pdfium_platform_library_name().to_string_lossy(),
            )))
        }
    }
}

/// Normalise a binding result, reusing PDFium's global bindings when they have
/// already been initialised by an earlier call.
///
/// `pdfium-render` initialises its library bindings exactly once per process;
/// later `bind_to_*` calls report `PdfiumLibraryBindingsAlreadyInitialized`.
/// In that case [`Pdfium::default`] takes its already-initialised branch and
/// hands out another handle to the same global bindings (it cannot panic on
/// that path because no library load is attempted).
fn pdfium_from_bindings(
    bindings: Result<Box<dyn PdfiumLibraryBindings>, PdfiumError>,
) -> Result<Pdfium, PdfiumError> {
    match bindings {
        Ok(bindings) => Ok(Pdfium::new(bindings)),
        Err(PdfiumError::PdfiumLibraryBindingsAlreadyInitialized) => Ok(Pdfium::default()),
        Err(error) => Err(error),
    }
}

/// Read the page sizes of the PDF at `path`, in PDF points and document order.
///
/// When the `hayro` feature is enabled, uses the pure-Rust hayro-syntax parser
/// for page-size probing. Otherwise falls back to PDFium.
///
/// Passwords are not supported yet; encrypted PDFs are reported as an explicit
/// single-line error. The file is opened read-only and never modified.
pub fn read_pdf_info(path: impl AsRef<Path>) -> Result<PdfSourceInfo, DataError> {
    let path = path.as_ref();

    #[cfg(feature = "hayro")]
    {
        // When hayro is enabled, use hayro's parser exclusively.
        hayro_pages::probe_pages_hayro(path).map(|page_sizes| PdfSourceInfo { page_sizes })
    }

    #[cfg(not(feature = "hayro"))]
    {
        // When hayro is not enabled, fall back to PDFium
        let pdfium = bind_pdfium()?;
        let document = load_document(&pdfium, path)?;

        let page_sizes = document
            .pages()
            .page_sizes()
            .map_err(|error| {
                DataError::Pdf(format!(
                    "could not read page sizes from {}: {error}",
                    path.display()
                ))
            })?
            .into_iter()
            .map(|rect| (rect.width().value, rect.height().value))
            .collect();

        Ok(PdfSourceInfo { page_sizes })
    }
}

// --- Hayro support (page-size probing and rendering) ---
// Pure-Rust PDF implementation using hayro (v0.8.0).
// This module is compiled when the `hayro` feature is enabled.
//
// hayro renders a page to a `vello_cpu` pixmap of premultiplied RGBA8, which we
// convert to an `image::RgbaImage` and save as a PNG. API reference:
// <https://docs.rs/hayro/0.8.0/hayro/>.

#[cfg(feature = "hayro")]
mod hayro_pages {
    use super::*;

    use hayro::hayro_interpret::InterpreterSettings;
    use hayro::vello_cpu::color::palette::css::WHITE;
    use hayro::{render, PixmapSettings, RenderCache, RenderSettings};
    use hayro_syntax::{LoadPdfError, Pdf};

    /// Single-line, honest message for a hayro-syntax document-open failure.
    ///
    /// `LoadPdfError` implements `Debug` but not `Display`; the decryption case
    /// is mapped to an explicit encrypted-PDF message and everything else falls
    /// back to a compact single-line `Debug` format so status lines stay
    /// readable.
    fn open_error_message(error: LoadPdfError, path: &Path) -> String {
        match error {
            LoadPdfError::Decryption(_) => format!(
                "could not open PDF {}: the PDF is encrypted or password-protected; \
                 Rubric cannot open encrypted PDFs",
                path.display()
            ),
            LoadPdfError::Invalid => format!(
                "could not parse PDF {} with hayro-syntax: {error:?}",
                path.display()
            ),
        }
    }

    /// Probe page sizes using hayro-syntax (pure-Rust PDF parser).
    ///
    /// Returns the `(width, height)` in PDF points for every page, in document
    /// order, using `Page::render_dimensions()`: the crop box intersected with
    /// the media box, with the page's intrinsic rotation applied (`/Rotate
    /// 90|270` swaps width and height). This is the same semantics as PDFium's
    /// `pages().page_sizes()`, so both engines report identical geometry for
    /// the same document.
    pub fn probe_pages_hayro(path: &Path) -> Result<Vec<(f32, f32)>, DataError> {
        let bytes = fs::read(path)
            .map_err(|e| DataError::Pdf(format!("could not read PDF {}: {e}", path.display())))?;

        let pdf = Pdf::new(bytes).map_err(|e| DataError::Pdf(open_error_message(e, path)))?;

        // `render_dimensions()` is what hayro's own render path uses, so the
        // probed geometry always matches the rendered bitmap.
        let page_sizes = pdf
            .pages()
            .iter()
            .map(|page| page.render_dimensions())
            .collect();

        Ok(page_sizes)
    }

    /// Render a PDF page using hayro (pure Rust) and save as PNG.
    ///
    /// `target_width_px` is the desired output width in pixels; the height is
    /// derived from the page's own aspect ratio so the scale stays uniform. The
    /// page is composited over an opaque white background so the exported PNG
    /// has no transparency (matching the PDFium renderer).
    ///
    /// Returns the path to the cached PNG file.
    pub fn render_page_hayro(
        pdf_path: &Path,
        page_index: usize,
        target_width_px: u32,
        cache_path: &Path,
    ) -> Result<PathBuf, DataError> {
        let bytes = fs::read(pdf_path).map_err(|e| {
            DataError::Pdf(format!("could not read PDF {}: {e}", pdf_path.display()))
        })?;

        let pdf = Pdf::new(bytes).map_err(|e| DataError::Pdf(open_error_message(e, pdf_path)))?;

        let pages = pdf.pages();
        let page_count = pages.len();
        if page_index >= page_count {
            return Err(DataError::Pdf(format!(
                "page index {page_index} is out of range for {} ({} page(s))",
                pdf_path.display(),
                page_count
            )));
        }

        let page = &pages[page_index];

        // Render dimensions are in PDF points (72 pt = 1 in). They already
        // account for the media/crop box and the page's intrinsic rotation, so a
        // uniform scale factor is all that is needed to hit the target width.
        let (src_width, _src_height) = page.render_dimensions();
        let scale = if src_width > 0.0 {
            target_width_px as f32 / src_width
        } else {
            1.0
        };

        // The render cache is reusable across pages; one render per call here.
        let cache = RenderCache::new();
        let interpreter_settings = InterpreterSettings::default();
        let render_settings = RenderSettings::default();
        let pixmap_settings = PixmapSettings {
            x_scale: scale,
            y_scale: scale,
            bg_color: WHITE,
        };

        let pixmap = render(
            page,
            &cache,
            &interpreter_settings,
            &render_settings,
            &pixmap_settings,
        );

        let width = u32::from(pixmap.width());
        let height = u32::from(pixmap.height());

        // `data_as_u8_slice` yields premultiplied RGBA8 in row-major order. The
        // pixmap is composited over an opaque white background, so every pixel
        // is fully opaque and premultiplied values equal straight-alpha values.
        let rgba = pixmap.data_as_u8_slice().to_vec();
        let img = image::RgbaImage::from_raw(width, height, rgba).ok_or_else(|| {
            DataError::Pdf(
                "could not build an image from the hayro pixmap: \
                 dimensions do not match the pixel buffer length"
                    .into(),
            )
        })?;

        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        save_png_atomically(cache_path, |path| {
            img.save_with_format(path, image::ImageFormat::Png)
        })?;

        Ok(cache_path.to_path_buf())
    }
}

/// Render one PDF page to a cached PNG using the PDFium engine.
///
/// Returns the path to the cached PNG file. The page is rasterised to
/// `target_width_px` pixels wide with the aspect ratio preserved. Widths above
/// [`MAX_RENDER_WIDTH_PX`] are clamped. The PNG is written atomically to
/// `cache_path`.
pub fn render_page_pdfium(
    pdf_path: &Path,
    page_index: usize,
    target_width_px: u32,
    cache_path: &Path,
) -> Result<PathBuf, DataError> {
    let pdfium = bind_pdfium()?;
    let document = load_document(&pdfium, pdf_path)?;
    let pages = document.pages();
    let page_count = pages.len();

    if page_index >= page_count as usize {
        return Err(DataError::Pdf(format!(
            "page index {page_index} is out of range for {} ({} page(s))",
            pdf_path.display(),
            page_count
        )));
    }

    let page = pages.get(page_index as PdfPageIndex).map_err(|error| {
        DataError::Pdf(format!(
            "could not load page {page_index} of {}: {error}",
            pdf_path.display()
        ))
    })?;

    let target_width_px = i32::try_from(target_width_px).unwrap_or(i32::MAX);
    let bitmap = page
        .render_with_config(&PdfRenderConfig::new().set_target_width(target_width_px))
        .map_err(|error| {
            DataError::Pdf(format!(
                "could not render page {page_index} of {}: {error}",
                pdf_path.display()
            ))
        })?;

    let image = bitmap.as_image().map_err(|error| {
        DataError::Pdf(format!(
            "could not convert page {page_index} of {} to an image: {error}",
            pdf_path.display()
        ))
    })?;

    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    save_png_atomically(cache_path, |path| {
        image.save_with_format(path, image::ImageFormat::Png)
    })?;

    Ok(cache_path.to_path_buf())
}

/// Render one PDF page to a cached PNG and return the cache file path.
///
/// The page is rasterised to `target_width_px` pixels wide, with the aspect
/// ratio preserved. Widths above [`MAX_RENDER_WIDTH_PX`] are clamped to that
/// ceiling before the cache path is computed, so the file name always matches
/// the width actually rendered. The cache location is
/// `<config dir>/render_cache/<key>/page_<index>_<width>.png`, where `<key>` is a
/// non-cryptographic hash of the canonical PDF path, its modification time and
/// its length. The key changes when the PDF changes, so stale renders are not
/// served. When the file already exists it is returned without rendering.
///
/// A zero width is rejected, as is an out-of-range `page_index`. The cache
/// directory is created on demand, and the PNG is written atomically (a crash
/// or two concurrent instances can never leave a truncated PNG in the cache).
///
/// When the `hayro` feature is enabled, uses the pure-Rust hayro renderer.
/// Otherwise falls back to PDFium (requires PDFium native library at runtime).
pub fn render_page_to_cache(
    pdf_path: impl AsRef<Path>,
    page_index: usize,
    target_width_px: u32,
) -> Result<PathBuf, DataError> {
    if target_width_px == 0 {
        return Err(DataError::Pdf(
            "render width must be greater than zero".into(),
        ));
    }
    let target_width_px = clamp_render_width(target_width_px);

    let pdf_path = pdf_path.as_ref();
    let cache_path = render_cache_page_path(pdf_path, page_index, target_width_px)?;
    if cache_path.exists() {
        return Ok(cache_path);
    }

    // When the hayro feature is enabled, render with the pure-Rust engine.
    #[cfg(feature = "hayro")]
    {
        hayro_pages::render_page_hayro(pdf_path, page_index, target_width_px, &cache_path)
    }

    // PDFium fallback (when hayro feature is not enabled).
    #[cfg(not(feature = "hayro"))]
    {
        render_page_pdfium(pdf_path, page_index, target_width_px, &cache_path)
    }
}

/// Render one PDF page to a cached PNG using the hayro engine.
///
/// Only available when the `hayro` feature is enabled. Returns the path to the
/// cached PNG file.
#[cfg(feature = "hayro")]
pub fn render_page_hayro_direct(
    pdf_path: &Path,
    page_index: usize,
    target_width_px: u32,
    cache_path: &Path,
) -> Result<PathBuf, DataError> {
    hayro_pages::render_page_hayro(pdf_path, page_index, target_width_px, cache_path)
}

/// Export a PDF with markups layered over the original page backgrounds.
///
/// Opens `source_pdf` with lopdf, appends an overlay content stream to each
/// page that renders the project's markups, and writes the result to
/// `output_path`. The original PDF content is preserved underneath the overlay.
///
/// `markups` is the full list of project markups; only those whose `page`
/// falls within the exported PDF are rendered.
///
/// Only available when lopdf is a dependency (v0.3-05).
pub fn export_pdf_to(
    source_pdf: &Path,
    output_path: &Path,
    markups: &[Markup],
) -> Result<(), DataError> {
    let mut doc = lopdf::Document::load(source_pdf)
        .map_err(|e| DataError::Pdf(format!("could not open PDF {}: {e}", source_pdf.display())))?;

    // Group markups by page index (domain pages are 0-based, PDF page numbers
    // from get_pages() are 1-based).
    let mut markups_by_page: BTreeMap<usize, Vec<&Markup>> = BTreeMap::new();
    for m in markups {
        markups_by_page.entry(m.page).or_default().push(m);
    }

    // Register a base-14 Helvetica font object once, for text markups.
    let font_dict = lopdf::Dictionary::from_iter([
        ("Type".into(), lopdf::Object::Name(b"/Font".to_vec())),
        ("Subtype".into(), lopdf::Object::Name(b"/Type1".to_vec())),
        ("BaseFont".into(), lopdf::Object::Name(b"/Helvetica".to_vec())),
        ("Encoding".into(), lopdf::Object::Name(b"/WinAnsiEncoding".to_vec())),
    ]);
    let font_id = doc.add_object(lopdf::Object::Dictionary(font_dict));

    for (page_num, page_id) in doc.get_pages() {
        let page_index = (page_num - 1) as usize;
        let page_markups = markups_by_page
            .get(&page_index)
            .cloned()
            .unwrap_or_default();
        if page_markups.is_empty() {
            continue;
        }

        // Ensure the page has a Resources dictionary mapping /F1 to the font.
        let has_text = page_markups
            .iter()
            .any(|m| matches!(m.geometry, MarkupGeometry::Text { .. }));
        if has_text {
            add_font_resource_to_page(&mut doc, page_id, font_id)?;
        }

        let (width, height) = page_dimensions(&doc, page_id)?;
        let content = render_page_overlay(&page_markups, width, height);
        doc.add_to_page_content(page_id, content).map_err(|e| {
            DataError::Pdf(format!("could not add overlay to page {page_num}: {e}"))
        })?;
    }

    doc.save(output_path).map_err(|e| {
        DataError::Pdf(format!("could not save PDF {}: {e}", output_path.display()))
    })?;

    Ok(())
}

/// Add a /Font resource mapping /F1 to `font_id` on `page_id`.
fn add_font_resource_to_page(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    font_id: lopdf::ObjectId,
) -> Result<(), DataError> {
    use lopdf::Object;

    // Clone existing Resources (if any) before mutating the page dict,
    // because doc.add_object borrows doc mutably.
    let existing_resources: Option<lopdf::Dictionary> = {
        let dict = doc
            .get_dictionary(page_id)
            .map_err(|e| DataError::Pdf(format!("could not get page dict: {e}")))?;
        match dict.get(b"Resources") {
            Some(Object::Dictionary(d)) => Some(d.clone()),
            Some(Object::Reference(id)) => doc
                .get_dictionary(*id)
                .map(|d| d.clone())
                .map_err(|e| DataError::Pdf(format!("could not get resources: {e}")))?,
            other => {
                return Err(DataError::Pdf(format!(
                    "unexpected Resources type: {other:?}"
                )));
            }
        }
    };

    let mut resources_dict = existing_resources.unwrap_or_default();

    // Set /Font sub-dictionary referencing the shared font_id.
    let mut font_dict = lopdf::Dictionary::new();
    font_dict.set("/F1", Object::Reference(font_id));
    resources_dict.set("Font", Object::Dictionary(font_dict));

    // Set or replace the Resources entry on the page.
    let resources_id = doc.add_object(Object::Dictionary(resources_dict));
    let dict = doc
        .get_dictionary_mut(page_id)
        .map_err(|e| DataError::Pdf(format!("could not get page dict: {e}")))?;
    dict.set("Resources", Object::Reference(resources_id));

    Ok(())
}

/// Read a page's MediaBox and return its (width, height) in PDF points.
fn page_dimensions(
    doc: &lopdf::Document,
    page_id: lopdf::ObjectId,
) -> Result<(f32, f32), DataError> {
    let dict = doc
        .get_dictionary(page_id)
        .map_err(|e| DataError::Pdf(format!("could not read page dictionary: {e}")))?;
    let media_box = dict
        .get(b"MediaBox")
        .map_err(|e| DataError::Pdf(format!("page has no MediaBox: {e}")))?;
    let arr = media_box
        .as_array()
        .map_err(|e| DataError::Pdf(format!("MediaBox is not an array: {e}")))?;
    // MediaBox = [llx, lly, urx, ury]
    if arr.len() < 4 {
        return Err(DataError::Pdf("MediaBox has fewer than 4 elements".into()));
    }
    let width = array_num(&arr[2])? - array_num(&arr[0])?;
    let height = array_num(&arr[3])? - array_num(&arr[1])?;
    Ok((width as f32, height as f32))
}

/// Convert a MediaBox array element to f64.
fn array_num(obj: &lopdf::Object) -> Result<f64, DataError> {
    match obj {
        lopdf::Object::Integer(n) => Ok(*n as f64),
        lopdf::Object::Real(f) => Ok(*f as f64),
        _ => Err(DataError::Pdf("MediaBox element is not a number".into())),
    }
}

/// Render all markups on one page into a single PDF content stream.
///
/// Domain coordinates use y-down from the top-left; PDF content coordinates
/// use y-up from the bottom-left. All y values are flipped via
/// `pdf_y = page_height - domain_y`.
fn render_page_overlay(markups: &[&Markup], width: f32, height: f32) -> lopdf::content::Content {
    use lopdf::content::Operation;

    let mut ops: Vec<Operation> = vec![Operation::new("q", vec![])]; // save graphics state

    for m in markups {
        ops.extend(render_markup(m, width, height));
    }

    ops.push(Operation::new("Q", vec![])); // restore graphics state

    lopdf::content::Content { operations: ops }
}

/// Convert a domain y (down from top) to a PDF y (up from bottom).
fn pdf_y(y: f32, height: f32) -> f32 {
    height - y
}

/// Render one markup to PDF drawing operations.
fn render_markup(m: &Markup, _width: f32, height: f32) -> Vec<lopdf::content::Operation> {
    use lopdf::content::Operation;

    let style = &m.style;
    let c = style.stroke;
    let (r, g, b) = (c.r as f64 / 255.0, c.g as f64 / 255.0, c.b as f64 / 255.0);

    let mut ops: Vec<Operation> = vec![
        Operation::new("q", vec![]), // save
        Operation::new(
            "RG",
            vec![
                Object::Real(r as f32),
                Object::Real(g as f32),
                Object::Real(b as f32),
            ],
        ),
        Operation::new(
            "rg",
            vec![
                Object::Real(r as f32),
                Object::Real(g as f32),
                Object::Real(b as f32),
            ],
        ),
        Operation::new("w", vec![Object::Real(style.width)]),
    ];

    match &m.geometry {
        MarkupGeometry::Bounds(rct) => {
            let x0 = rct.x;
            let y0 = pdf_y(rct.y, height) - rct.h; // bottom-left in PDF
            ops.push(Operation::new(
                "re",
                vec![
                    Object::Real(x0),
                    Object::Real(y0),
                    Object::Real(rct.w),
                    Object::Real(rct.h),
                ],
            ));
            match style.fill_mode {
                FillMode::None => ops.push(Operation::new("S", vec![])), // stroke only
                FillMode::Solid | FillMode::Highlight => ops.push(Operation::new("f", vec![])),
            }
        }
        MarkupGeometry::Segment { a, b } => {
            ops.push(Operation::new(
                "m",
                vec![Object::Real(a.x), Object::Real(pdf_y(a.y, height))],
            ));
            ops.push(Operation::new(
                "l",
                vec![Object::Real(b.x), Object::Real(pdf_y(b.y, height))],
            ));
            ops.push(Operation::new("S", vec![]));
        }
        MarkupGeometry::Path { points, closed } => {
            if let Some(first) = points.first() {
                ops.push(Operation::new(
                    "m",
                    vec![Object::Real(first.x), Object::Real(pdf_y(first.y, height))],
                ));
                for p in points.iter().skip(1) {
                    ops.push(Operation::new(
                        "l",
                        vec![Object::Real(p.x), Object::Real(pdf_y(p.y, height))],
                    ));
                }
                if *closed {
                    ops.push(Operation::new("h", vec![])); // close path
                }
            }
            match style.fill_mode {
                FillMode::None => ops.push(Operation::new("S", vec![])),
                FillMode::Solid | FillMode::Highlight => ops.push(Operation::new("f", vec![])),
            }
        }
        MarkupGeometry::Ink { strokes } => {
            for stroke in strokes {
                if let Some(first) = stroke.first() {
                    ops.push(Operation::new(
                        "m",
                        vec![Object::Real(first.x), Object::Real(pdf_y(first.y, height))],
                    ));
                    for p in stroke.iter().skip(1) {
                        ops.push(Operation::new(
                            "l",
                            vec![Object::Real(p.x), Object::Real(pdf_y(p.y, height))],
                        ));
                    }
                    ops.push(Operation::new("S", vec![]));
                }
            }
        }
        MarkupGeometry::Text { at, wrap: _, text } => {
            // Draw text as a text object using a base-14 font (Helvetica).
            // y-flip already applied via pdf_y below; baseline sits at the
            // anchor y (domain text anchors at top-left).
            ops.push(Operation::new("BT", vec![])); // begin text
            ops.push(Operation::new(
                "Tf",
                vec![
                    Object::Name(b"/F1".to_vec()),
                    Object::Real(style.font_size as f32),
                ],
            ));
            ops.push(Operation::new(
                "Td",
                vec![
                    Object::Real(at.x),
                    Object::Real(pdf_y(at.y, height) - style.font_size),
                ],
            ));
            ops.push(Operation::new("Tj", vec![Object::string_literal(text)]));
            ops.push(Operation::new("ET", vec![])); // end text
        }
    }

    ops.push(Operation::new("Q", vec![])); // restore
    ops
}

/// Directory holding rendered page PNGs, or `None` when no base directory can
/// be located.
///
/// Mirrors [`super::store::project_dir`]'s `ProjectDirs` usage but prefers the
/// platform config directory (`~/.config/rubric` on Linux), which is where
/// app-level caches belong. When that is unavailable it falls back to the app
/// data directory, then reports `None` so callers can return an error instead
/// of writing somewhere surprising.
pub fn render_cache_dir() -> Option<PathBuf> {
    ProjectDirs::from("dev", "Rubric", "Rubric")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config").join("rubric"))
        })
        .or_else(super::store::project_dir)
        .map(|base| base.join("render_cache"))
}

/// Load a PDF from disk, mapping PDFium's failure into a data error.
///
/// PDF loading via PDFium. Used by [`render_page_pdfium`] and by
/// [`read_pdf_info`] when the `hayro` feature is not enabled.
fn load_document<'a>(pdfium: &'a Pdfium, path: &Path) -> Result<PdfDocument<'a>, DataError> {
    pdfium
        .load_pdf_from_file(path, None)
        .map_err(|error| DataError::Pdf(pdfium_open_error(&error, path)))
}

/// Single-line, honest message for a PDFium document-open failure.
///
/// `PdfiumError`'s `Display` implementation pretty-`Debug`s its payload, which
/// produces unreadable multi-line status text. Password and security failures
/// are mapped to an explicit encrypted-PDF message; everything else falls back
/// to a compact single-line `Debug` format so status lines stay readable.
fn pdfium_open_error(error: &PdfiumError, path: &Path) -> String {
    if let PdfiumError::PdfiumLibraryInternalError(internal) = error {
        if matches!(
            internal,
            PdfiumInternalError::PasswordError | PdfiumInternalError::SecurityError
        ) {
            return format!(
                "could not open PDF {}: the PDF is encrypted or password-protected; \
                 Rubric cannot open encrypted PDFs",
                path.display()
            );
        }
    }
    format!("could not open PDF {}: {error:?}", path.display())
}

/// Cache path for one page render, creating neither directories nor files.
fn render_cache_page_path(
    pdf_path: &Path,
    page_index: usize,
    target_width_px: u32,
) -> Result<PathBuf, DataError> {
    let key = pdf_cache_key(pdf_path)?;
    let root = render_cache_dir().ok_or_else(|| {
        DataError::Path("no config directory available for the render cache".into())
    })?;
    Ok(root
        .join(key)
        .join(format!("page_{page_index}_{target_width_px}.png")))
}

/// Deterministic, non-cryptographic cache key for a PDF file.
///
/// Hashes the canonical path, file length and modification time with
/// [`DefaultHasher`]. This is adequate for cache invalidation (a changed or
/// replaced PDF yields a new directory) but is not a security primitive and is
/// not guaranteed to be stable across Rust releases; a changed hash only costs
/// a cache miss.
fn pdf_cache_key(path: &Path) -> Result<String, DataError> {
    let canonical = fs::canonicalize(path)?;
    let metadata = fs::metadata(&canonical)?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    let mut hasher = DefaultHasher::new();
    canonical.hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    modified.hash(&mut hasher);

    Ok(format!("{:016x}", hasher.finish()))
}

/// A unique sibling path used by [`save_png_atomically`]'s atomic rename. It
/// lives in the target's directory so the rename stays on one filesystem (and
/// is thus atomic).
fn temp_cache_sibling(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "page.png".to_string());
    target.with_file_name(format!(".{name}.{}.tmp", uuid::Uuid::new_v4()))
}

/// Save a rendered page PNG to `cache_path` atomically.
///
/// `save` receives the path to write the PNG to; the helper points it at a
/// unique temporary sibling in the same directory and then renames it over the
/// target, mirroring [`super::store::save_project`]. Writing directly to the
/// target would let a crash mid-write — or two concurrent instances — leave a
/// truncated PNG that the `exists()` cache check would then serve forever. The
/// temporary file is removed on every error path.
fn save_png_atomically<F>(cache_path: &Path, save: F) -> Result<(), DataError>
where
    F: FnOnce(&Path) -> Result<(), image::ImageError>,
{
    let tmp = temp_cache_sibling(cache_path);

    // Every error path after the temp file is created must clean it up.
    let result = (|| -> Result<(), DataError> {
        save(&tmp).map_err(|e| {
            DataError::Pdf(format!(
                "could not write rendered page to {}: {e}",
                tmp.display()
            ))
        })?;
        fs::rename(&tmp, cache_path)?;
        Ok(())
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check if PDFium is available for testing (PDFium runtime library).
    ///
    /// Tests skip gracefully when CI has not provisioned the shared library
    /// (`RUBRIC_PDFIUM_PATH` unset locally). No test mutates the environment;
    /// the guard relies on the CI-provided value.
    #[cfg(not(feature = "hayro"))]
    fn test_pdfium() -> Option<Pdfium> {
        if std::env::var_os(PDFIUM_PATH_ENV).is_none() {
            eprintln!("skipping: {PDFIUM_PATH_ENV} not set");
            return None;
        }
        Some(bind_pdfium().expect("PDFium must bind when RUBRIC_PDFIUM_PATH is set"))
    }

    /// Check if hayro is available for testing (always available when feature enabled).
    #[cfg(feature = "hayro")]
    fn test_hayro() -> bool {
        // hayro is always available when the feature is enabled
        true
    }

    /// Path for a per-test temporary PDF, unique per process and thread.
    fn temp_pdf_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "rubric_pdf_test_{}_{}_{:?}.pdf",
            std::process::id(),
            tag,
            std::thread::current().id()
        ))
    }

    /// Assemble a minimal, valid one-page PDF with the given media box.
    ///
    /// Object offsets are computed while writing, so the xref table stays
    /// correct even if the object bodies change.
    fn minimal_pdf(page_width: f32, page_height: f32) -> Vec<u8> {
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {page_width} {page_height}] \
                 /Resources << >> >>"
            ),
        ];

        let mut bytes = Vec::new();
        // ASCII-only header keeps byte offsets stable when the test decodes the
        // document as text to validate the xref table.
        bytes.extend_from_slice(b"%PDF-1.4\n");

        let mut offsets = Vec::with_capacity(objects.len());
        for (index, body) in objects.iter().enumerate() {
            offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, body).as_bytes());
        }

        let xref_offset = bytes.len();
        bytes.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
        bytes.extend_from_slice(b"0000000000 65535 f \n");
        for offset in offsets {
            bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        bytes.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );

        bytes
    }

    /// Write a temporary one-page PDF and return its path.
    fn write_minimal_pdf(tag: &str, width: f32, height: f32) -> PathBuf {
        let path = temp_pdf_path(tag);
        fs::write(&path, minimal_pdf(width, height)).expect("write test PDF");
        path
    }

    /// Remove a rendered page and its cache-key directory.
    fn cleanup_render(path: &Path) {
        let _ = fs::remove_file(path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir(parent);
        }
    }

    #[test]
    fn render_with_zero_width_is_rejected_without_a_library() {
        let error = render_page_to_cache("/nonexistent/plan.pdf", 0, 0)
            .expect_err("zero width must be rejected");
        assert!(matches!(error, DataError::Pdf(_)));
    }

    #[test]
    fn pdf_source_info_serde_roundtrip() {
        let info = PdfSourceInfo {
            page_sizes: vec![(612.0, 792.0), (595.28, 841.89)],
        };
        let json = serde_json::to_string(&info).expect("serialize");
        let back: PdfSourceInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, info);
    }

    #[test]
    fn minimal_pdf_helper_has_correct_xref_offsets() {
        let bytes = minimal_pdf(612.0, 792.0);
        let text = String::from_utf8_lossy(&bytes);

        let start = text.rfind("startxref\n").expect("startxref marker");
        let xref_offset: usize = text[start + "startxref\n".len()..]
            .lines()
            .next()
            .expect("startxref offset line")
            .parse()
            .expect("numeric startxref offset");
        assert_eq!(&bytes[xref_offset..xref_offset + 4], b"xref");

        let entries: Vec<&str> = text[xref_offset..]
            .lines()
            .filter(|line| line.ends_with(" n "))
            .collect();
        assert_eq!(entries.len(), 3);

        for (index, entry) in entries.iter().enumerate() {
            let offset: usize = entry[..10].parse().expect("numeric xref entry offset");
            let expected = format!("{} 0 obj", index + 1);
            assert!(
                text[offset..].starts_with(&expected),
                "xref entry {index} points at the wrong object"
            );
        }
    }

    /// Test that read_pdf_info works with PDFium (or falls back to hayro for page-size probing).
    /// When hayro feature is enabled, this test always runs with hayro.
    #[test]
    fn read_pdf_info_reads_us_letter_page() {
        #[cfg(not(feature = "hayro"))]
        {
            let Some(_pdfium) = test_pdfium() else {
                return;
            };
        }

        #[cfg(feature = "hayro")]
        let _ = test_hayro();

        let path = write_minimal_pdf("info", 612.0, 792.0);

        // When hayro is enabled, use hayro's probe; otherwise use PDFium
        let info = read_pdf_info(&path).expect("read PDF info");
        assert_eq!(info.page_sizes.len(), 1);
        let (width, height) = info.page_sizes[0];
        assert!((width - 612.0).abs() < 0.5, "width was {width}");
        assert!((height - 792.0).abs() < 0.5, "height was {height}");

        let _ = fs::remove_file(&path);
    }

    /// Test PNG rendering - uses PDFium when hayro is disabled, hayro when enabled.
    #[test]
    fn render_page_to_cache_writes_png_and_reuses_cache() {
        #[cfg(not(feature = "hayro"))]
        {
            let Some(_pdfium) = test_pdfium() else {
                return;
            };
        }

        #[cfg(feature = "hayro")]
        let _ = test_hayro();

        let path = write_minimal_pdf("render", 612.0, 792.0);

        let first = render_page_to_cache(&path, 0, 400).expect("render page");
        assert!(first.is_file(), "{} should exist", first.display());
        let bytes = fs::read(&first).expect("read PNG");
        assert!(bytes.len() > 8, "PNG should not be empty");
        assert_eq!(&bytes[..4], b"\x89PNG");

        // Replacing the cached file proves the second call returns early
        // instead of re-rendering over it.
        fs::write(&first, b"sentinel").expect("write sentinel");
        let second = render_page_to_cache(&path, 0, 400).expect("cached page");
        assert_eq!(second, first);
        assert_eq!(
            fs::read(&second).expect("read sentinel").as_slice(),
            b"sentinel"
        );

        cleanup_render(&first);
        let _ = fs::remove_file(&path);
    }

    /// Test out-of-range page rejection.
    #[test]
    fn render_page_to_cache_rejects_out_of_range_page() {
        #[cfg(not(feature = "hayro"))]
        {
            let Some(_pdfium) = test_pdfium() else {
                return;
            };
        }

        #[cfg(feature = "hayro")]
        let _ = test_hayro();

        let path = write_minimal_pdf("out_of_range", 612.0, 792.0);

        let error = render_page_to_cache(&path, 5, 200).expect_err("page 5 must not render");
        assert!(matches!(error, DataError::Pdf(_)));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn cache_key_changes_when_file_is_replaced_with_longer_content() {
        let path = temp_pdf_path("key_length");
        fs::write(&path, minimal_pdf(612.0, 792.0)).expect("write original");
        let before = pdf_cache_key(&path).expect("key for original");

        // Same path, longer file: length and modification time both differ,
        // so the key must change or a stale render would be served.
        fs::write(&path, minimal_pdf(1024.0, 2048.0)).expect("overwrite");
        let after = pdf_cache_key(&path).expect("key for replacement");
        assert_ne!(before, after);

        // An untouched file keeps its key, so the cache stays reusable.
        assert_eq!(pdf_cache_key(&path).expect("key again"), after);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn cache_key_changes_when_only_mtime_changes() {
        let path = temp_pdf_path("key_mtime");
        fs::write(&path, b"0123456789abcdef").expect("write");
        let before = pdf_cache_key(&path).expect("key");

        // Identical bytes, bumped modification time: only the mtime component
        // of the key differs.
        let file = fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .expect("open for mtime bump");
        let bumped = std::time::SystemTime::now() + std::time::Duration::from_secs(300);
        file.set_modified(bumped).expect("set modified");

        let after = pdf_cache_key(&path).expect("key after touch");
        assert_ne!(before, after, "a touched file must change the key");
        assert_eq!(
            fs::read(&path).expect("bytes unchanged").as_slice(),
            b"0123456789abcdef"
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn cache_key_for_missing_file_is_an_error() {
        let path = temp_pdf_path("key_missing");
        let _ = fs::remove_file(&path);
        assert!(pdf_cache_key(&path).is_err());
    }

    #[test]
    fn render_cache_paths_do_not_collide_across_widths_or_pages() {
        let path = write_minimal_pdf("cache_paths", 612.0, 792.0);

        let page0_w400 = render_cache_page_path(&path, 0, 400).expect("path");
        // Deterministic: identical inputs map to the same cache file.
        assert_eq!(
            render_cache_page_path(&path, 0, 400).expect("path"),
            page0_w400
        );

        let page0_w800 = render_cache_page_path(&path, 0, 800).expect("path");
        let page1_w400 = render_cache_page_path(&path, 1, 400).expect("path");

        assert_ne!(page0_w400, page0_w800, "widths must not collide");
        assert_ne!(page0_w400, page1_w400, "pages must not collide");
        assert_ne!(page0_w800, page1_w400);

        // One key directory per PDF state; the file name carries page/width.
        assert_eq!(page0_w400.parent(), page0_w800.parent());
        assert_eq!(page0_w400.parent(), page1_w400.parent());
        assert!(page0_w400.ends_with("page_0_400.png"));
        assert!(page0_w800.ends_with("page_0_800.png"));
        assert!(page1_w400.ends_with("page_1_400.png"));

        let _ = fs::remove_file(&path);
    }

    /// Test that re-rendering after PDF replacement works.
    #[test]
    fn render_page_to_cache_re_renders_after_pdf_is_replaced() {
        #[cfg(not(feature = "hayro"))]
        {
            let Some(_pdfium) = test_pdfium() else {
                return;
            };
        }

        #[cfg(feature = "hayro")]
        let _ = test_hayro();

        let path = write_minimal_pdf("invalidate", 612.0, 792.0);

        let stale = render_page_to_cache(&path, 0, 400).expect("render original PDF");
        assert!(stale.is_file(), "{} should exist", stale.display());
        // Sentinel: if the second call returned `stale`, it would be serving
        // bytes from the old file instead of re-rendering the replacement.
        fs::write(&stale, b"stale-sentinel").expect("write sentinel");

        // Replace the PDF at the same path with longer, different content
        // (file length and modification time both change).
        fs::write(&path, minimal_pdf(1024.0, 2048.0)).expect("overwrite PDF");

        let fresh = render_page_to_cache(&path, 0, 400).expect("render replacement");
        assert_ne!(fresh, stale, "replaced PDF must get a new cache key");
        assert!(fresh.is_file(), "{} should exist", fresh.display());
        let bytes = fs::read(&fresh).expect("read fresh PNG");
        assert_eq!(&bytes[..4], b"\x89PNG");

        // The old entry stays untouched under its old key: it is neither
        // overwritten nor returned for the new file.
        assert_eq!(
            fs::read(&stale).expect("stale sentinel").as_slice(),
            b"stale-sentinel"
        );

        cleanup_render(&fresh);
        cleanup_render(&stale);
        let _ = fs::remove_file(&path);
    }

    /// Test that different widths produce different cache files.
    #[test]
    fn render_page_to_cache_keeps_a_separate_file_per_width() {
        #[cfg(not(feature = "hayro"))]
        {
            let Some(_pdfium) = test_pdfium() else {
                return;
            };
        }

        #[cfg(feature = "hayro")]
        let _ = test_hayro();

        let path = write_minimal_pdf("width_api", 612.0, 792.0);

        let narrow = render_page_to_cache(&path, 0, 300).expect("render at 300 px");
        let wide = render_page_to_cache(&path, 0, 600).expect("render at 600 px");
        assert_ne!(narrow, wide, "widths must use different cache files");
        assert!(narrow.is_file(), "{} should exist", narrow.display());
        assert!(wide.is_file(), "{} should exist", wide.display());
        assert_ne!(
            fs::read(&narrow).expect("read narrow PNG"),
            fs::read(&wide).expect("read wide PNG"),
            "renders at different widths must differ"
        );

        // Re-requesting a width reuses its own cache file, not the other's.
        assert_eq!(
            render_page_to_cache(&path, 0, 300).expect("cached 300 px"),
            narrow
        );

        cleanup_render(&narrow);
        cleanup_render(&wide);
        let _ = fs::remove_file(&path);
    }

    /// Test that missing files are rejected.
    #[test]
    fn read_pdf_info_rejects_missing_file() {
        #[cfg(not(feature = "hayro"))]
        {
            let Some(_pdfium) = test_pdfium() else {
                return;
            };
        }

        #[cfg(feature = "hayro")]
        let _ = test_hayro();

        let path = temp_pdf_path("missing");
        let _ = fs::remove_file(&path);

        let error = read_pdf_info(&path).expect_err("missing file must fail");
        assert!(matches!(error, DataError::Pdf(_)));
    }

    /// Test that corrupt PDFs are rejected.
    #[test]
    fn read_pdf_info_rejects_corrupt_pdf() {
        #[cfg(not(feature = "hayro"))]
        {
            let Some(_pdfium) = test_pdfium() else {
                return;
            };
        }

        #[cfg(feature = "hayro")]
        let _ = test_hayro();

        let path = temp_pdf_path("corrupt");
        fs::write(&path, b"this is not a PDF").expect("write corrupt file");

        let error = read_pdf_info(&path).expect_err("corrupt file must fail");
        assert!(matches!(error, DataError::Pdf(_)));

        let _ = fs::remove_file(&path);
    }

    /// Test hayro-specific: render page using hayro engine.
    #[test]
    #[cfg(feature = "hayro")]
    fn hayro_render_page_to_cache_works() {
        let _ = test_hayro();

        let path = write_minimal_pdf("hayro_render", 612.0, 792.0);

        let rendered = render_page_to_cache(&path, 0, 400).expect("render page with hayro");
        assert!(rendered.is_file(), "{:?} should exist", rendered);
        let bytes = fs::read(&rendered).expect("read PNG");
        assert!(bytes.len() > 8, "PNG should not be empty");
        assert_eq!(&bytes[..4], b"\x89PNG", "should be a valid PNG");

        cleanup_render(&rendered);
        let _ = fs::remove_file(&path);
    }

    /// Oversized render widths are clamped to the documented ceiling: hayro's
    /// pixmap and pdfium-render's bitmaps are both `u16`-limited, so a runaway
    /// width must never be passed to an engine (or into a cache file name).
    #[test]
    fn render_width_is_clamped_to_the_documented_ceiling() {
        assert_eq!(clamp_render_width(400), 400);
        assert_eq!(clamp_render_width(MAX_RENDER_WIDTH_PX), MAX_RENDER_WIDTH_PX);
        assert_eq!(clamp_render_width(u32::MAX), MAX_RENDER_WIDTH_PX);
        assert!(clamp_render_width(u32::MAX) <= u16::MAX as u32);
    }

    /// The atomic cache write must remove its temporary file on every error
    /// path. Here the final rename fails (the target is occupied by a
    /// non-empty directory) long after the temp file was written.
    #[test]
    fn png_cache_write_cleans_up_its_temp_file_when_the_rename_fails() {
        let dir = std::env::temp_dir().join(format!(
            "rubric_png_atomic_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create test dir");

        let target = dir.join("page_0_400.png");
        fs::create_dir_all(target.join("inner")).expect("occupy target with a directory");

        let error = save_png_atomically(&target, |_path| Ok(()))
            .expect_err("rename over a directory must fail");
        assert!(matches!(error, DataError::Io(_)), "got {error:?}");

        let leftovers: Vec<String> = fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "temp files left behind: {leftovers:?}"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// A successful atomic cache write must leave the PNG at the target and no
    /// temporary sibling behind.
    #[test]
    fn png_cache_write_renames_its_temp_file_into_place() {
        let dir = std::env::temp_dir().join(format!(
            "rubric_png_atomic_ok_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create test dir");

        let target = dir.join("page_1_300.png");
        save_png_atomically(&target, |path| {
            fs::write(path, b"png-bytes").map_err(image::ImageError::IoError)
        })
        .expect("atomic save");

        assert_eq!(fs::read(&target).expect("read target"), b"png-bytes");
        let leftovers: Vec<String> = fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "temp files left behind: {leftovers:?}"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
