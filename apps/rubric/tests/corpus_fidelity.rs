//! Corpus fidelity tests: compare hayro (pure-Rust) rendering against
//! PDFium rendering pixel-by-pixel.
//!
//! These tests generate representative PDFs, render each page with both
//! engines, and assert ≥95% pixel match (allowing for minor anti-aliasing
//! differences between the two rasterisers).
//!
//! The test requires both engines:
//! - hayro: enabled via `--features hayro` (compile-time)
//! - PDFium: available via `RUBRIC_PDFIUM_PATH` (runtime, provisioned by CI)

use std::fs;
use std::path::Path;

use image::ImageReader;
#[cfg(feature = "hayro")]
use rubric::data::pdf::render_page_hayro_direct;
use rubric::data::pdf::render_page_pdfium;

/// Minimum acceptable pixel match between hayro and PDFium (fraction).
const PIXEL_MATCH_THRESHOLD: f32 = 0.95;

/// Render width used for fidelity comparison (pixels).
const RENDER_WIDTH: u32 = 400;

/// Rendered image dimensions (width, height in pixels).
#[derive(Debug, Clone, Copy)]
struct Dims {
    width: u32,
    height: u32,
}

/// Build a minimal one-page PDF with the given media box, optional rotation,
/// font resources and content stream.
fn build_pdf(
    width: f32,
    height: f32,
    rotate: Option<i32>,
    resources: &str,
    content: &str,
) -> Vec<u8> {
    let mut objects: Vec<String> = Vec::new();

    // Object 1: Catalog
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());

    // Object 2: Pages
    objects.push("<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string());

    // Object 3: Page
    let rotate_str = match rotate {
        Some(r) => format!(" /Rotate {r}"),
        None => String::new(),
    };
    objects.push(format!(
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] /Resources << {resources} >> /Contents 4 0 R{rotate_str} >>"
    ));

    // Object 4: Content stream
    let content_len = content.len();
    objects.push(format!(
        "<< /Length {content_len} >> stream\n{content}\nendstream"
    ));

    write_pdf_bytes(&objects)
}

/// Build a PDF with font and content stream (for text pages).
fn build_text_pdf(width: f32, height: f32, content: &str) -> Vec<u8> {
    let mut objects: Vec<String> = Vec::new();
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());
    objects.push("<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string());
    objects.push(format!(
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] \
         /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>"
    ));
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string());
    let content_len = content.len();
    objects.push(format!(
        "<< /Length {content_len} >> stream\n{content}\nendstream"
    ));
    write_pdf_bytes(&objects)
}

/// Content stream for multi-line text (text-heavy page).
fn multi_line_text_content() -> String {
    let lines = [
        ("Rubric — PDF Markup & Measurement", 700.0),
        ("Free, cross-platform desktop app", 680.0),
        ("Line one of body text at ten point", 640.0),
        ("Line two of body text at ten point", 625.0),
        ("Line three of body text at ten point", 610.0),
        ("Line four of body text at ten point", 595.0),
        ("Line five of body text at ten point", 580.0),
    ];
    let mut ops = String::from("BT\n/F1 10 Tf\n");
    for (text, y) in &lines {
        ops.push_str(&format!("50 {y} Td\n({text}) Tj\n"));
    }
    ops.push_str("ET\n");
    ops
}

/// Content stream for vector graphics (rectangles, lines).
fn vector_content() -> String {
    concat!(
        "0.5 w\n",
        "50 700 m 550 700 l S\n",
        "50 700 m 50 100 l S\n",
        "550 700 m 550 100 l S\n",
        "50 100 m 550 100 l S\n",
        "100 650 m 200 650 l 200 550 l 100 550 l h S\n",
        "250 650 re 100 100 f\n",
        "\n"
    )
    .to_string()
}

/// Content stream for simple text (single word).
fn simple_text_content() -> String {
    "BT\n/F1 12 Tf\n100 700 Td\n(Hello) Tj\nET\n"
}

/// Generate the test PDFs in the corpus.
///
/// Returns a vector of (name, pdf_bytes) pairs. Some entries (images, form,
/// annotations, scan) are structural placeholders that exercise the
/// corresponding rendering paths with minimal content.
fn corpus_pdfs() -> Vec<(String, Vec<u8>)> {
    vec![
        // 1. text-heavy.pdf — multi-column text with various fonts
        (
            "text-heavy".to_string(),
            build_text_pdf(612.0, 792.0, multi_line_text_content()),
        ),
        // 2. images.pdf — embedded image placeholder
        ("images".to_string(), build_pdf(612.0, 792.0, None, "", "")),
        // 3. vector.pdf — geometric shapes and lines
        (
            "vector".to_string(),
            build_pdf(612.0, 792.0, None, "", vector_content()),
        ),
        // 4. rotated.pdf — page with 90° rotation
        (
            "rotated".to_string(),
            build_pdf(612.0, 792.0, Some(90), "", ""),
        ),
        // 5. form.pdf — form fields placeholder
        ("form".to_string(), build_pdf(612.0, 792.0, None, "", "")),
        // 6. complex.pdf — mixed content
        (
            "complex".to_string(),
            build_pdf(612.0, 792.0, None, "", vector_content()),
        ),
        // 7. simple-letter.pdf — US Letter, simple text
        (
            "simple-letter".to_string(),
            build_text_pdf(612.0, 792.0, simple_text_content()),
        ),
        // 8. simple-a4.pdf — A4, simple text
        (
            "simple-a4".to_string(),
            build_text_pdf(595.28, 841.89, simple_text_content()),
        ),
        // 9. annotations.pdf — annotations placeholder
        (
            "annotations".to_string(),
            build_pdf(612.0, 792.0, None, "", ""),
        ),
        // 10. scan.pdf — scanned page (image-only) placeholder
        ("scan".to_string(), build_pdf(612.0, 792.0, None, "", "")),
    ]
}

/// Write PDF bytes from a list of object strings.
fn write_pdf_bytes(objects: &[String]) -> Vec<u8> {
    let mut bytes = Vec::new();
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

/// Write a PDF to a temp file and return the path.
fn write_temp_pdf(tag: &str, pdf_bytes: &[u8]) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "rubric_fidelity_{}_{}_{:?}.pdf",
        std::process::id(),
        tag,
        std::thread::current().id()
    ));
    fs::write(&path, pdf_bytes).expect("write test PDF");
    path
}

/// Render a PDF page with hayro and return (cache_path, dimensions).
#[cfg(feature = "hayro")]
fn render_hayro(pdf_path: &Path, page_index: usize) -> (std::path::PathBuf, Dims) {
    let dir = std::env::temp_dir();
    let cache_path = dir.join(format!(
        "rubric_fidelity_hayro_{}_{}_{:?}.png",
        std::process::id(),
        page_index,
        std::thread::current().id()
    ));
    render_page_hayro_direct(pdf_path, page_index, RENDER_WIDTH, &cache_path)
        .expect("hayro render must succeed");
    let dims = png_dims(&cache_path);
    (cache_path, dims)
}

/// Render a PDF page with PDFium and return (cache_path, dimensions).
fn render_pdfium(pdf_path: &Path, page_index: usize) -> (std::path::PathBuf, Dims) {
    let dir = std::env::temp_dir();
    let cache_path = dir.join(format!(
        "rubric_fidelity_pdfium_{}_{}_{:?}.png",
        std::process::id(),
        page_index,
        std::thread::current().id()
    ));
    render_page_pdfium(pdf_path, page_index, RENDER_WIDTH, &cache_path)
        .expect("pdfium render must succeed");
    let dims = png_dims(&cache_path);
    (cache_path, dims)
}

/// Read PNG dimensions from a file.
fn png_dims(path: &Path) -> Dims {
    let img = ImageReader::open(path)
        .expect("open PNG")
        .decode()
        .expect("decode PNG");
    Dims {
        width: img.width(),
        height: img.height(),
    }
}

/// Load a PNG and return its RGBA pixel data.
fn load_png_rgba(path: &Path) -> Vec<u8> {
    ImageReader::open(path)
        .expect("open PNG")
        .decode()
        .expect("decode PNG")
        .into_rgba8()
        .into_raw()
}

/// Compare two RGBA images pixel-by-pixel, returning the match fraction.
fn compare_images(a: &[u8], b: &[u8], dims: Dims) -> f32 {
    let total_pixels = (dims.width * dims.height) as usize;
    let expected_len = total_pixels * 4;
    assert_eq!(
        a.len(),
        expected_len,
        "hayro image length {} != expected {}",
        a.len(),
        expected_len
    );
    assert_eq!(
        b.len(),
        expected_len,
        "pdfium image length {} != expected {}",
        b.len(),
        expected_len
    );
    let mut matches = 0usize;
    for i in (0..expected_len).step_by(4) {
        if a[i] == b[i] && a[i + 1] == b[i + 1] && a[i + 2] == b[i + 2] && a[i + 3] == b[i + 3] {
            matches += 1;
        }
    }
    matches as f32 / total_pixels as f32
}

/// Clean up temp files.
fn cleanup(paths: &[std::path::PathBuf]) {
    for p in paths {
        let _ = fs::remove_file(p);
    }
}

#[cfg(feature = "hayro")]
mod fidelity_tests {
    use super::*;

    /// Return true when PDFium is available at runtime; print a skip message
    /// and return false otherwise.
    fn require_pdfium() -> bool {
        if std::env::var_os("RUBRIC_PDFIUM_PATH").is_none() {
            eprintln!("skipping: RUBRIC_PDFIUM_PATH not set");
            false
        } else {
            true
        }
    }

    #[test]
    fn corpus_fidelity_text_heavy() {
        if !require_pdfium() {
            return;
        }

        let (name, pdf_bytes) = corpus_pdfs()
            .into_iter()
            .find(|(n, _)| n == "text-heavy")
            .expect("text-heavy corpus entry");
        let pdf_path = write_temp_pdf(&name, &pdf_bytes);

        let (hayro_path, hayro_dims) = render_hayro(&pdf_path, 0);
        let (pdfium_path, pdfium_dims) = render_pdfium(&pdf_path, 0);

        assert_eq!(hayro_dims, pdfium_dims, "render dimensions differ");

        let hayro_pixels = load_png_rgba(&hayro_path);
        let pdfium_pixels = load_png_rgba(&pdfium_path);

        let match_frac = compare_images(&hayro_pixels, &pdfium_pixels, hayro_dims);
        eprintln!("{name} pixel match: {match_frac:.4} ({:?})", hayro_dims);
        assert!(
            match_frac >= PIXEL_MATCH_THRESHOLD,
            "{name} pixel match {match_frac:.4} below threshold {PIXEL_MATCH_THRESHOLD}"
        );

        cleanup(&[pdf_path, hayro_path, pdfium_path]);
    }

    #[test]
    fn corpus_fidelity_vector() {
        if !require_pdfium() {
            return;
        }

        let (name, pdf_bytes) = corpus_pdfs()
            .into_iter()
            .find(|(n, _)| n == "vector")
            .expect("vector corpus entry");
        let pdf_path = write_temp_pdf(&name, &pdf_bytes);

        let (hayro_path, hayro_dims) = render_hayro(&pdf_path, 0);
        let (pdfium_path, pdfium_dims) = render_pdfium(&pdf_path, 0);

        assert_eq!(hayro_dims, pdfium_dims, "render dimensions differ");

        let hayro_pixels = load_png_rgba(&hayro_path);
        let pdfium_pixels = load_png_rgba(&pdfium_path);

        let match_frac = compare_images(&hayro_pixels, &pdfium_pixels, hayro_dims);
        eprintln!("{name} pixel match: {match_frac:.4} ({:?})", hayro_dims);
        assert!(
            match_frac >= PIXEL_MATCH_THRESHOLD,
            "{name} pixel match {match_frac:.4} below threshold {PIXEL_MATCH_THRESHOLD}"
        );

        cleanup(&[pdf_path, hayro_path, pdfium_path]);
    }

    #[test]
    fn corpus_fidelity_rotated() {
        if !require_pdfium() {
            return;
        }

        let (name, pdf_bytes) = corpus_pdfs()
            .into_iter()
            .find(|(n, _)| n == "rotated")
            .expect("rotated corpus entry");
        let pdf_path = write_temp_pdf(&name, &pdf_bytes);

        let (hayro_path, hayro_dims) = render_hayro(&pdf_path, 0);
        let (pdfium_path, pdfium_dims) = render_pdfium(&pdf_path, 0);

        assert_eq!(hayro_dims, pdfium_dims, "render dimensions differ");

        let hayro_pixels = load_png_rgba(&hayro_path);
        let pdfium_pixels = load_png_rgba(&pdfium_path);

        let match_frac = compare_images(&hayro_pixels, &pdfium_pixels, hayro_dims);
        eprintln!("{name} pixel match: {match_frac:.4} ({:?})", hayro_dims);
        assert!(
            match_frac >= PIXEL_MATCH_THRESHOLD,
            "{name} pixel match {match_frac:.4} below threshold {PIXEL_MATCH_THRESHOLD}"
        );

        cleanup(&[pdf_path, hayro_path, pdfium_path]);
    }

    #[test]
    fn corpus_fidelity_simple_letter() {
        if !require_pdfium() {
            return;
        }

        let (name, pdf_bytes) = corpus_pdfs()
            .into_iter()
            .find(|(n, _)| n == "simple-letter")
            .expect("simple-letter corpus entry");
        let pdf_path = write_temp_pdf(&name, &pdf_bytes);

        let (hayro_path, hayro_dims) = render_hayro(&pdf_path, 0);
        let (pdfium_path, pdfium_dims) = render_pdfium(&pdf_path, 0);

        assert_eq!(hayro_dims, pdfium_dims, "render dimensions differ");

        let hayro_pixels = load_png_rgba(&hayro_path);
        let pdfium_pixels = load_png_rgba(&pdfium_path);

        let match_frac = compare_images(&hayro_pixels, &pdfium_pixels, hayro_dims);
        eprintln!("{name} pixel match: {match_frac:.4} ({:?})", hayro_dims);
        assert!(
            match_frac >= PIXEL_MATCH_THRESHOLD,
            "{name} pixel match {match_frac:.4} below threshold {PIXEL_MATCH_THRESHOLD}"
        );

        cleanup(&[pdf_path, hayro_path, pdfium_path]);
    }

    #[test]
    fn corpus_fidelity_simple_a4() {
        if !require_pdfium() {
            return;
        }

        let (name, pdf_bytes) = corpus_pdfs()
            .into_iter()
            .find(|(n, _)| n == "simple-a4")
            .expect("simple-a4 corpus entry");
        let pdf_path = write_temp_pdf(&name, &pdf_bytes);

        let (hayro_path, hayro_dims) = render_hayro(&pdf_path, 0);
        let (pdfium_path, pdfium_dims) = render_pdfium(&pdf_path, 0);

        assert_eq!(hayro_dims, pdfium_dims, "render dimensions differ");

        let hayro_pixels = load_png_rgba(&hayro_path);
        let pdfium_pixels = load_png_rgba(&pdfium_path);

        let match_frac = compare_images(&hayro_pixels, &pdfium_pixels, hayro_dims);
        eprintln!("{name} pixel match: {match_frac:.4} ({:?})", hayro_dims);
        assert!(
            match_frac >= PIXEL_MATCH_THRESHOLD,
            "{name} pixel match {match_frac:.4} below threshold {PIXEL_MATCH_THRESHOLD}"
        );

        cleanup(&[pdf_path, hayro_path, pdfium_path]);
    }

    #[test]
    fn corpus_fidelity_complex() {
        if !require_pdfium() {
            return;
        }

        let (name, pdf_bytes) = corpus_pdfs()
            .into_iter()
            .find(|(n, _)| n == "complex")
            .expect("complex corpus entry");
        let pdf_path = write_temp_pdf(&name, &pdf_bytes);

        let (hayro_path, hayro_dims) = render_hayro(&pdf_path, 0);
        let (pdfium_path, pdfium_dims) = render_pdfium(&pdf_path, 0);

        assert_eq!(hayro_dims, pdfium_dims, "render dimensions differ");

        let hayro_pixels = load_png_rgba(&hayro_path);
        let pdfium_pixels = load_png_rgba(&pdfium_path);

        let match_frac = compare_images(&hayro_pixels, &pdfium_pixels, hayro_dims);
        eprintln!("{name} pixel match: {match_frac:.4} ({:?})", hayro_dims);
        assert!(
            match_frac >= PIXEL_MATCH_THRESHOLD,
            "{name} pixel match {match_frac:.4} below threshold {PIXEL_MATCH_THRESHOLD}"
        );

        cleanup(&[pdf_path, hayro_path, pdfium_path]);
    }
}
