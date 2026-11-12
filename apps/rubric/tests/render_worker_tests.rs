//! Integration tests for the async thumbnail render worker.
//!
//! `worker_renders_one_fixture_page_and_returns_non_empty_pixels` renders
//! one page of a tiny fixture PDF through the real worker thread and
//! asserts non-empty RGBA pixels come back via `try_drain` (headless: no
//! display, no GPU). It is runtime-gated on `RUBRIC_PDFIUM_PATH` with the
//! same skip pattern as `tests/corpus_fidelity.rs`, so CI jobs that do not
//! provision the PDFium shared library (the hayro job) skip it instead of
//! failing; the file compiles unchanged under default features and under
//! `--features hayro`.
//!
//! `worker_reports_failure_payload_for_missing_pdf` needs no PDF engine at
//! all: a missing PDF fails in `render_page_to_cache` before any engine is
//! touched, so it exercises the worker thread, the repaint path and the
//! drain semantics in every CI job.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui;
use rubric::data::render_worker::{RenderWorker, ThumbPixels};
use rubric::domain::thumb_render::{RenderJob, RenderKey};

/// How long to wait for the worker before giving up.
const RESULT_TIMEOUT: Duration = Duration::from_secs(20);

/// How often to poll the result channel.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Poll `try_drain` until a result for `key` arrives or the timeout
/// expires. The worker renders on its own thread, so results are never
/// synchronous; the channel buffers them in the meantime.
fn wait_for_result(worker: &RenderWorker, key: &RenderKey) -> Option<ThumbPixels> {
    let deadline = Instant::now() + RESULT_TIMEOUT;
    while Instant::now() < deadline {
        for pixels in worker.try_drain() {
            if pixels.key == *key {
                return Some(pixels);
            }
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    None
}

/// True when the PDFium shared library is provisioned; prints a skip
/// message otherwise. Mirrors `require_pdfium` in `tests/corpus_fidelity.rs`.
fn pdfium_available() -> bool {
    if std::env::var_os("RUBRIC_PDFIUM_PATH").is_none() {
        eprintln!("skipping: RUBRIC_PDFIUM_PATH not set");
        false
    } else {
        true
    }
}

/// Assemble a minimal one-page PDF that draws one line of text (US Letter,
/// base-14 Helvetica). Same hand-rolled xref discipline as the fixtures in
/// `src/data/pdf.rs` and `tests/corpus_fidelity.rs`: object offsets are
/// computed while writing, so the table stays correct.
fn fixture_pdf_bytes() -> Vec<u8> {
    let content = "BT /F1 24 Tf 72 700 Td (Rubric render worker fixture) Tj ET";
    let content_len = content.len();
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
             /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>"
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        format!("<< /Length {content_len} >>\nstream\n{content}\nendstream"),
    ];
    pdf_bytes_from_objects(&objects)
}

/// Serialise numbered objects into a valid PDF byte stream (correct xref
/// offsets computed as the objects are written).
fn pdf_bytes_from_objects(objects: &[String]) -> Vec<u8> {
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

/// Write the fixture PDF to a unique temp file and return its path.
fn write_fixture_pdf() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "rubric_render_worker_{}_{:?}.pdf",
        std::process::id(),
        std::thread::current().id()
    ));
    fs::write(&path, fixture_pdf_bytes()).expect("write fixture PDF");
    path
}

/// Best-effort cleanup: remove the fixture PDF and the cached PNG, then the
/// now-empty cache-key directory (`render_cache/<key>/`) when the PNG
/// removal left it empty.
fn cleanup(pdf_path: &Path, cache_png: &Path) {
    let _ = fs::remove_file(pdf_path);
    if fs::remove_file(cache_png).is_ok() {
        if let Some(cache_key_dir) = cache_png.parent() {
            let _ = fs::remove_dir(cache_key_dir);
        }
    }
}

#[test]
fn worker_renders_one_fixture_page_and_returns_non_empty_pixels() {
    if !pdfium_available() {
        return;
    }

    let pdf_path = write_fixture_pdf();
    let key = RenderKey {
        pdf_id: pdf_path.display().to_string(),
        page: 0,
        width_px: 192,
    };
    let job = RenderJob {
        key: key.clone(),
        priority: 0,
    };

    let ctx = egui::Context::default();
    let worker = RenderWorker::spawn(&ctx);
    worker.submit(job);

    // Capture the result before any cleanup so the timeout path still
    // removes the fixture instead of leaking it to a panic.
    let result = wait_for_result(&worker, &key);
    match result {
        Some(pixels) => {
            assert!(
        pixels.outcome.is_ok(),
        "fixture render failed: {:?}",
        pixels.outcome
    );
            assert!(pixels.width > 0, "rendered width must be positive");
            assert!(pixels.height > 0, "rendered height must be positive");
            assert_eq!(
                pixels.rgba.len(),
                (pixels.width * pixels.height * 4) as usize,
                "RGBA payload must be width*height*4 straight-alpha bytes"
            );
            assert!(
        pixels.rgba.iter().any(|byte| *byte != 0),
        "page must have non-zero pixels"
    );
            assert!(!pixels.cache_path.is_empty());
            cleanup(&pdf_path, Path::new(&pixels.cache_path));
        }
        None => {
            // No result within the timeout: remove the fixture PDF before
            // panicking. Any cached PNG is keyed under the temp path and is
            // harmless (the OS temp-dir hygiene drops it).
            let _ = fs::remove_file(&pdf_path);
            panic!("worker result within timeout");
        }
    }
}

#[test]
fn worker_reports_failure_payload_for_missing_pdf() {
    let key = RenderKey {
        pdf_id: "/nonexistent/rubric-render-worker/missing.pdf".into(),
        page: 0,
        width_px: 192,
    };
    let job = RenderJob {
        key: key.clone(),
        priority: 0,
    };

    let ctx = egui::Context::default();
    let worker = RenderWorker::spawn(&ctx);
    worker.submit(job);

    let pixels = wait_for_result(&worker, &key).expect("worker failure within timeout");

    assert!(
        pixels.outcome.is_err(),
        "a missing PDF must fail the render"
    );
    let message = pixels.outcome.unwrap_err();
    assert!(!message.is_empty(), "failure payload must carry a message");
    assert_eq!(pixels.key, key);
    assert_eq!(pixels.width, 0, "failure payload echoes no dimensions");
    assert_eq!(pixels.height, 0);
    assert!(pixels.rgba.is_empty());
    assert!(pixels.cache_path.is_empty());
}
