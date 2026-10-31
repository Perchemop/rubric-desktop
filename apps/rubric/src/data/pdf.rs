//! PDF import helpers: PDFium binding, page-size probing, and page rendering.
//!
//! PDFium is loaded at run time (no native dependency at build time). Binding
//! order is:
//!
//! 1. the `RUBRIC_PDFIUM_PATH` environment variable (a directory containing the
//!    platform library, or the library file itself);
//! 2. the directory containing the running executable;
//! 3. the system library path.
//!
//! Rendered pages are cached as PNG files under the app config directory so the
//! UI can ask for the same page size repeatedly without re-rasterising.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
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

/// Page sizes read from a PDF, in PDF points and document order.
///
/// Each entry is `(width, height)` as reported by PDFium. Width and height are
/// unrotated media-box dimensions; any intrinsic page rotation is not applied
/// here (the domain [`crate::domain::Page::rotation`] field tracks rotation for
/// the UI to apply).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PdfSourceInfo {
    /// `(width, height)` in PDF points for every page, in document order.
    pub page_sizes: Vec<(f32, f32)>,
}

/// Bind to PDFium, loading the shared library if this is the first call.
///
/// Binding is attempted from [`PDFIUM_PATH_ENV`], then the executable's
/// directory, then the system library path. PDFium itself initialises its
/// global bindings once per process; subsequent calls reuse those bindings and
/// still return a usable [`Pdfium`] handle.
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
        let library = Pdfium::pdfium_platform_library_name_at_path(&exe_dir);
        match pdfium_from_bindings(Pdfium::bind_to_library(&library)) {
            Ok(pdfium) => return Ok(pdfium),
            Err(error) => errors.push(format!("{}: {error}", library.display())),
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
/// Passwords are not supported yet; encrypted PDFs are reported as an error.
/// The file is opened read-only and never modified.
pub fn read_pdf_info(path: impl AsRef<Path>) -> Result<PdfSourceInfo, DataError> {
    let path = path.as_ref();
    
    #[cfg(feature = "hayro")]
    {
        match hayro_pages::probe_pages_hayro(path) {
            Ok(page_sizes) => return Ok(PdfSourceInfo { page_sizes }),
            Err(e) => eprintln!("hayro probe failed, falling back to PDFium: {e}"),
        }
    }
    
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

// --- Hayro page-size probing support ---
// Pure-Rust PDF parser implementation using hayro-syntax.
// This module is only compiled when the `hayro` feature is enabled.
//
// Based on hayro-syntax v0.5 API (docs.rs/hayro_syntax/0.5.0).

#[cfg(feature = "hayro")]
mod hayro_pages {
    use super::*;
    use std::sync::Arc;

    /// Probe page sizes using hayro-syntax (pure Rust PDF parser).
    /// Returns page dimensions in PDF points (unrotated media-box).
    pub fn probe_pages_hayro(path: &Path) -> Result<Vec<(f32, f32)>, DataError> {
        let bytes = fs::read(path).map_err(|e| {
            DataError::Pdf(format!("could not read PDF {}: {e}", path.display()))
        })?;

        // Parse PDF using hayro-syntax (v0.5 API uses Pdf::new)
        let pdf = hayro_syntax::Pdf::new(Arc::new(bytes)).map_err(|e| {
            DataError::Pdf(format!("could not parse PDF {} with hayro-syntax: {e}", path.display()))
        })?;

        // Get pages iterator - hayro provides typed page access
        let pages = pdf.pages();
        let mut page_sizes = Vec::new();

        for page in pages.iter() {
            // Each page has a media_box() method
            if let Some(media_box) = page.media_box() {
                let w = media_box.width() as f32;
                let h = media_box.height() as f32;
                if w > 0.0 && h > 0.0 {
                    page_sizes.push((w, h));
                }
            }
        }

        Ok(page_sizes)
    }
}

/// Render one PDF page to a cached PNG and return the cache file path.
///
/// The page is rasterised to `target_width_px` pixels wide, with the aspect
/// ratio preserved. The cache location is
/// `<config dir>/render_cache/<key>/page_<index>_<width>.png`, where `<key>` is a
/// non-cryptographic hash of the canonical PDF path, its modification time and
/// its length. The key changes when the PDF changes, so stale renders are not
/// served. When the file already exists it is returned without rendering.
///
/// A zero width is rejected, as is an out-of-range `page_index`. The cache
/// directory is created on demand.
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

    let pdf_path = pdf_path.as_ref();
    let cache_path = render_cache_page_path(pdf_path, page_index, target_width_px)?;
    if cache_path.exists() {
        return Ok(cache_path);
    }

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
    image
        .save_with_format(&cache_path, image::ImageFormat::Png)
        .map_err(|error| {
            DataError::Pdf(format!(
                "could not write rendered page to {}: {error}",
                cache_path.display()
            ))
        })?;

    Ok(cache_path)
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
fn load_document<'a>(pdfium: &'a Pdfium, path: &Path) -> Result<PdfDocument<'a>, DataError> {
    pdfium
        .load_pdf_from_file(path, None)
        .map_err(|error| DataError::Pdf(format!("could not open PDF {}: {error}", path.display())))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Bind PDFium for a library-dependent test.
    ///
    /// Tests skip gracefully when CI has not provisioned the shared library
    /// (`RUBRIC_PDFIUM_PATH` unset locally). No test mutates the environment;
    /// the guard relies on the CI-provided value.
    fn test_pdfium() -> Option<Pdfium> {
        if std::env::var_os(PDFIUM_PATH_ENV).is_none() {
            eprintln!("skipping: {PDFIUM_PATH_ENV} not set");
            return None;
        }
        Some(bind_pdfium().expect("PDFium must bind when RUBRIC_PDFIUM_PATH is set"))
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

    #[test]
    fn read_pdf_info_reads_us_letter_page() {
        let Some(_pdfium) = test_pdfium() else {
            return;
        };
        let path = write_minimal_pdf("info", 612.0, 792.0);

        let info = read_pdf_info(&path).expect("read PDF info");
        assert_eq!(info.page_sizes.len(), 1);
        let (width, height) = info.page_sizes[0];
        assert!((width - 612.0).abs() < 0.5, "width was {width}");
        assert!((height - 792.0).abs() < 0.5, "height was {height}");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn render_page_to_cache_writes_png_and_reuses_cache() {
        let Some(_pdfium) = test_pdfium() else {
            return;
        };
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

    #[test]
    fn render_page_to_cache_rejects_out_of_range_page() {
        let Some(_pdfium) = test_pdfium() else {
            return;
        };
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

    #[test]
    fn render_page_to_cache_re_renders_after_pdf_is_replaced() {
        let Some(_pdfium) = test_pdfium() else {
            return;
        };
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

    #[test]
    fn render_page_to_cache_keeps_a_separate_file_per_width() {
        let Some(_pdfium) = test_pdfium() else {
            return;
        };
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

    #[test]
    fn read_pdf_info_rejects_missing_file() {
        let Some(_pdfium) = test_pdfium() else {
            return;
        };
        let path = temp_pdf_path("missing");
        let _ = fs::remove_file(&path);

        let error = read_pdf_info(&path).expect_err("missing file must fail");
        assert!(matches!(error, DataError::Pdf(_)));
    }

    #[test]
    fn read_pdf_info_rejects_corrupt_pdf() {
        let Some(_pdfium) = test_pdfium() else {
            return;
        };
        let path = temp_pdf_path("corrupt");
        fs::write(&path, b"this is not a PDF").expect("write corrupt file");

        let error = read_pdf_info(&path).expect_err("corrupt file must fail");
        assert!(matches!(error, DataError::Pdf(_)));

        let _ = fs::remove_file(&path);
    }
}
