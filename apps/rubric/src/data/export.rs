//! Export helpers for summary reports (spec F-304) and for flattening markups
//! back into PDF files (gap #1: a path from markups to a real PDF).
//!
//! Writing files is a data-layer concern; the domain [`Report`] stays pure.

use std::path::Path;

use crate::data::error::DataError;
use crate::domain::bates::BatesConfig;
use crate::domain::markup::Markup;
use crate::domain::report::Report;

/// Write `report` as plain UTF-8 text using [`Report::to_text`].
pub fn write_report_text(path: &Path, report: &Report) -> Result<(), DataError> {
    std::fs::write(path, report.to_text().as_bytes())?;
    Ok(())
}

/// Write `report` as pretty-printed JSON with a trailing newline.
pub fn write_report_json(path: &Path, report: &Report) -> Result<(), DataError> {
    let mut content = serde_json::to_string_pretty(report)?;
    content.push('\n');
    std::fs::write(path, content.as_bytes())?;
    Ok(())
}

/// Flatten one page of a PDF, with its markups, into a new single-page PDF.
///
/// The page named by `page_index` (0-based, in document order) is copied from
/// `source_pdf` into a new PDF: lopdf loads the source, every other page is
/// unlinked from the page tree, unreferenced objects are pruned and object
/// numbers compacted, and the result is saved to `output_path`. Only markups
/// whose [`Markup::page`] equals `page_index` are rendered, as an appended
/// overlay content stream; a page with no markups passes through unchanged.
///
/// The caller decides which markups to pass, so hidden layers stay the
/// caller's choice — the UI passes the page's paint order (what the canvas
/// shows). The source PDF is opened read-only and never modified.
///
/// # Errors
///
/// Returns [`DataError::Pdf`] when the source cannot be opened or parsed,
/// when `page_index` is out of range, when the overlay cannot be appended,
/// or when `output_path` cannot be written.
pub fn export_page_to_pdf(
    source_pdf: &Path,
    page_index: usize,
    output_path: &Path,
    markups: &[Markup],
    bates: &BatesConfig,
) -> Result<(), DataError> {
    let mut doc = lopdf::Document::load(source_pdf)
        .map_err(|e| DataError::Pdf(format!("could not open PDF {}: {e}", source_pdf.display())))?;

    let pages = doc.get_pages();
    let page_count = pages.len();
    // Domain pages are 0-based; lopdf page numbers from get_pages() are 1-based.
    let page_num = u32::try_from(page_index)
        .ok()
        .and_then(|n| n.checked_add(1))
        .ok_or_else(|| {
            DataError::Pdf(format!(
                "page index {page_index} is out of range for {} ({page_count} page(s))",
                source_pdf.display()
            ))
        })?;
    let page_id = *pages.get(&page_num).ok_or_else(|| {
        DataError::Pdf(format!(
            "page index {page_index} is out of range for {} ({page_count} page(s))",
            source_pdf.display()
        ))
    })?;

    // Overlay first, while the captured page object id is still valid: the
    // deletion step below renumbers objects, which would stale any id held
    // across it.
    let page_markups: Vec<&Markup> = markups.iter().filter(|m| m.page == page_index).collect();
    if !page_markups.is_empty() || bates.enabled {
        let font_id = super::pdf::add_base_font_object(&mut doc);
        super::pdf::overlay_page_with_markups(
            &mut doc,
            page_id,
            font_id,
            page_num,
            &page_markups,
            Some(bates),
        )?;
    }

    // Reduce the loaded document to the requested page: `delete_pages`
    // unlinks the other pages from the page tree, `prune_objects` drops their
    // now unreachable content, and `renumber_objects` compacts object numbers
    // so the saved xref table has no gaps.
    let other_pages: Vec<u32> = pages.keys().copied().filter(|&n| n != page_num).collect();
    if !other_pages.is_empty() {
        doc.delete_pages(&other_pages);
        doc.prune_objects();
        doc.renumber_objects();
    }

    doc.save(output_path).map_err(|e| {
        DataError::Pdf(format!("could not save PDF {}: {e}", output_path.display()))
    })?;

    Ok(())
}

/// Flatten every page of a PDF, with its markups, into a new PDF.
///
/// Every page of `source_pdf` is kept in document order; each page that
/// carries markups receives an appended overlay content stream rendering
/// them, and pages without markups pass through unchanged. This is the
/// all-pages form of [`export_page_to_pdf`] and delegates to the shared
/// lopdf overlay writer [`crate::data::pdf::export_pdf_to`].
///
/// # Errors
///
/// Returns [`DataError::Pdf`] when the source cannot be opened or parsed,
/// when an overlay cannot be appended, or when `output_path` cannot be
/// written.
pub fn export_all_pages_to_pdf(
    source_pdf: &Path,
    output_path: &Path,
    markups: &[Markup],
    bates: &BatesConfig,
) -> Result<(), DataError> {
    crate::data::pdf::export_pdf_to(source_pdf, output_path, markups, bates)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use crate::data::pdf::page_dimensions;
    use crate::domain::bates::{BatesConfig, BatesCorner};
    use crate::domain::geom::{Pt, Rect};
    use crate::domain::markup::{MarkupGeometry, MarkupKind};
    use crate::domain::project::Project;

    #[test]
    fn writes_text_and_round_trips() {
        let project = Project::new("Export Test");
        let report = Report::from_project(&project, crate::domain::report::ReportGroupKey::Status);
        let path = std::env::temp_dir().join(format!(
            "rubric_export_text_{}_{}.txt",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        write_report_text(&path, &report).expect("write text");
        let content = std::fs::read_to_string(&path).expect("read text");
        assert!(content.starts_with("Rubric Summary Report"));
        assert_eq!(content, report.to_text());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn writes_json_and_round_trips() {
        let project = Project::new("Export Test");
        let report = Report::from_project(&project, crate::domain::report::ReportGroupKey::Status);
        let path = std::env::temp_dir().join(format!(
            "rubric_export_json_{}_{}.json",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        write_report_json(&path, &report).expect("write json");
        let content = std::fs::read_to_string(&path).expect("read json");
        let back: Report = serde_json::from_str(&content).expect("parse json");
        assert_eq!(back, report);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn impossible_path_returns_err() {
        let project = Project::new("Export Test");
        let report = Report::from_project(&project, crate::domain::report::ReportGroupKey::Status);
        // A directory path cannot be written as a file.
        let dir = std::env::temp_dir();
        assert!(write_report_text(&dir, &report).is_err());
        assert!(write_report_json(&dir, &report).is_err());
        // A path in a non-existent subdirectory.
        let missing = std::env::temp_dir()
            .join("rubric_no_such_dir_xyz")
            .join("report.txt");
        assert!(write_report_text(&missing, &report).is_err());
    }

    // --- Flatten-to-PDF tests (helpers follow the pattern of the
    // `minimal_pdf` helper in data/pdf.rs tests) ---

    /// Assemble a minimal, valid PDF with one page per given media box.
    ///
    /// Object offsets are computed while writing, so the xref table stays
    /// correct even if the object bodies change.
    fn minimal_pdf(pages: &[(f32, f32)]) -> Vec<u8> {
        let kids = (0..pages.len())
            .map(|i| format!("{} 0 R", i + 3))
            .collect::<Vec<_>>()
            .join(" ");
        let mut objects = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            format!("<< /Type /Pages /Kids [{kids}] /Count {} >>", pages.len()),
        ];
        for (width, height) in pages {
            objects.push(format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] \
                 /Resources << >> >>"
            ));
        }

        let mut bytes = Vec::new();
        // ASCII-only header keeps byte offsets stable.
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

    /// Path for a per-test temporary PDF, unique per process and thread.
    fn temp_pdf_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "rubric_export_pdf_{}_{}_{:?}.pdf",
            std::process::id(),
            tag,
            std::thread::current().id()
        ))
    }

    /// Write a temporary test PDF with one page per given media box.
    fn write_test_pdf(tag: &str, pages: &[(f32, f32)]) -> PathBuf {
        let path = temp_pdf_path(tag);
        fs::write(&path, minimal_pdf(pages)).expect("write test PDF");
        path
    }

    /// A text markup whose string lands verbatim in the overlay content
    /// stream, so tests can assert its presence or absence.
    fn text_markup(page: usize, text: &str) -> Markup {
        Markup::new(
            page,
            MarkupKind::Text,
            MarkupGeometry::Text {
                at: Pt::new(60.0, 70.0),
                wrap: 200.0,
                text: text.to_string(),
            },
            "test",
        )
    }

    #[test]
    fn export_page_to_pdf_produces_one_page_with_overlay() {
        let source = write_test_pdf("page0", &[(612.0, 792.0), (400.0, 300.0)]);
        let output = temp_pdf_path("page0_out");
        let _ = fs::remove_file(&output);

        let markups = vec![
            text_markup(0, "PAGE-ZERO-MARKER"),
            Markup::new(
                0,
                MarkupKind::Rectangle,
                MarkupGeometry::Bounds(Rect::new(50.0, 50.0, 100.0, 60.0)),
                "test",
            ),
            text_markup(1, "PAGE-ONE-MARKER"),
        ];

        export_page_to_pdf(&source, 0, &output, &markups, &BatesConfig::default())
            .expect("export page 0");

        // The output re-loads as a valid PDF with exactly one page, and that
        // page is the requested one (the second page's size would be a mixup).
        let out = lopdf::Document::load(&output).expect("re-load exported PDF");
        let pages = out.get_pages();
        assert_eq!(
            pages.len(),
            1,
            "export must contain only the requested page"
        );
        let page_id = *pages.values().next().expect("one page");
        let (width, height) = page_dimensions(&out, page_id).expect("page dimensions");
        assert!((width - 612.0).abs() < 0.01, "width was {width}");
        assert!((height - 792.0).abs() < 0.01, "height was {height}");

        // The overlay content stream is stored as plain-text ops, so both the
        // marker string and the rectangle operator must be present.
        let content_bytes = out.get_page_content(page_id);
        let content = String::from_utf8_lossy(&content_bytes);
        assert!(
            content.contains("PAGE-ZERO-MARKER"),
            "overlay text missing: {content}"
        );
        assert!(content.contains("re"), "rectangle op missing: {content}");
        // The other page's markup must not leak into a single-page export.
        assert!(!content.contains("PAGE-ONE-MARKER"));

        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&output);
    }

    #[test]
    fn export_page_to_pdf_exports_the_second_page_when_asked() {
        let source = write_test_pdf("page1", &[(612.0, 792.0), (400.0, 300.0)]);
        let output = temp_pdf_path("page1_out");
        let _ = fs::remove_file(&output);

        let markups = vec![
            text_markup(0, "PAGE-ZERO-MARKER"),
            text_markup(1, "PAGE-ONE-MARKER"),
        ];

        export_page_to_pdf(&source, 1, &output, &markups, &BatesConfig::default())
            .expect("export page 1");

        let out = lopdf::Document::load(&output).expect("re-load exported PDF");
        let pages = out.get_pages();
        assert_eq!(
            pages.len(),
            1,
            "export must contain only the requested page"
        );
        let page_id = *pages.values().next().expect("one page");
        let (width, height) = page_dimensions(&out, page_id).expect("page dimensions");
        assert!((width - 400.0).abs() < 0.01, "width was {width}");
        assert!((height - 300.0).abs() < 0.01, "height was {height}");

        let content_bytes = out.get_page_content(page_id);
        let content = String::from_utf8_lossy(&content_bytes);
        assert!(content.contains("PAGE-ONE-MARKER"), "content: {content}");
        assert!(!content.contains("PAGE-ZERO-MARKER"), "content: {content}");

        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&output);
    }

    #[test]
    fn export_page_to_pdf_passes_an_unmarked_page_through_unchanged() {
        let source = write_test_pdf("unmarked", &[(612.0, 792.0), (400.0, 300.0)]);
        let output = temp_pdf_path("unmarked_out");
        let _ = fs::remove_file(&output);

        // Page 1 carries a markup, but only page 0 is exported — and page 0
        // has none, so it must pass through with no overlay at all. Passing
        // the full list also exercises the per-page filter.
        let markups = vec![text_markup(1, "PAGE-ONE-MARKER")];

        export_page_to_pdf(&source, 0, &output, &markups, &BatesConfig::default())
            .expect("export unmarked page 0");

        let out = lopdf::Document::load(&output).expect("re-load exported PDF");
        let pages = out.get_pages();
        assert_eq!(
            pages.len(),
            1,
            "export must contain only the requested page"
        );
        let page_id = *pages.values().next().expect("one page");
        let (width, height) = page_dimensions(&out, page_id).expect("page dimensions");
        assert!((width - 612.0).abs() < 0.01 && (height - 792.0).abs() < 0.01);
        let content = out.get_page_content(page_id);
        assert!(
            content.is_empty(),
            "unmarked page must have no overlay: {}",
            String::from_utf8_lossy(&content)
        );

        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&output);
    }

    #[test]
    fn export_page_to_pdf_handles_a_single_page_source_without_deletions() {
        let source = write_test_pdf("single", &[(300.0, 400.0)]);
        let output = temp_pdf_path("single_out");
        let _ = fs::remove_file(&output);

        export_page_to_pdf(
            &source,
            0,
            &output,
            &[text_markup(0, "ONLY-MARKER")],
            &BatesConfig::default(),
        )
        .expect("export the only page");

        let out = lopdf::Document::load(&output).expect("re-load exported PDF");
        let pages = out.get_pages();
        assert_eq!(
            pages.len(),
            1,
            "export must contain only the requested page"
        );
        let page_id = *pages.values().next().expect("one page");
        let (width, height) = page_dimensions(&out, page_id).expect("page dimensions");
        assert!((width - 300.0).abs() < 0.01 && (height - 400.0).abs() < 0.01);
        let content_bytes = out.get_page_content(page_id);
        let content = String::from_utf8_lossy(&content_bytes);
        assert!(content.contains("ONLY-MARKER"), "content: {content}");

        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&output);
    }

    #[test]
    fn export_all_pages_to_pdf_keeps_every_page_with_its_overlay() {
        let source = write_test_pdf("all", &[(612.0, 792.0), (400.0, 300.0)]);
        let output = temp_pdf_path("all_out");
        let _ = fs::remove_file(&output);

        let markups = vec![
            text_markup(0, "PAGE-ZERO-MARKER"),
            text_markup(1, "PAGE-ONE-MARKER"),
        ];

        export_all_pages_to_pdf(&source, &output, &markups, &BatesConfig::default())
            .expect("export all pages");

        let out = lopdf::Document::load(&output).expect("re-load exported PDF");
        let pages = out.get_pages();
        assert_eq!(pages.len(), 2, "all pages must be kept");
        for (page_num, page_id) in &pages {
            let content_bytes = out.get_page_content(*page_id);
            let content = String::from_utf8_lossy(&content_bytes);
            let marker = if *page_num == 1 {
                "PAGE-ZERO-MARKER"
            } else {
                "PAGE-ONE-MARKER"
            };
            assert!(
                content.contains(marker),
                "page {page_num} missing {marker}: {content}"
            );
        }

        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&output);
    }

    #[test]
    fn export_all_pages_to_pdf_stamps_bates_on_every_in_range_page() {
        let source = write_test_pdf("bates", &[(612.0, 792.0), (400.0, 300.0)]);
        let output = temp_pdf_path("bates_out");
        let _ = fs::remove_file(&output);

        // No markups at all: the only overlay on each page is the Bates stamp.
        let bates = BatesConfig {
            enabled: true,
            prefix: "RUB".to_string(),
            start: 1,
            digits: 3,
            corner: BatesCorner::BottomRight,
            ..BatesConfig::default()
        };

        export_all_pages_to_pdf(&source, &output, &[], &bates).expect("export all pages");

        let out = lopdf::Document::load(&output).expect("re-load exported PDF");
        let pages = out.get_pages();
        assert_eq!(pages.len(), 2, "all pages must be kept");
        for (page_num, page_id) in &pages {
            let expected = format!("RUB{:03}", page_num);
            let content_bytes = out.get_page_content(*page_id);
            let content = String::from_utf8_lossy(&content_bytes);
            assert!(
                content.contains(expected.as_str()),
                "page {page_num} missing Bates stamp {expected}: {content}"
            );
            // A stamp needs the shared font resource on the page.
            assert!(
                content.contains("/F1"),
                "page {page_num} missing font resource: {content}"
            );
        }

        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&output);
    }

    #[test]
    fn export_all_pages_to_pdf_skips_pages_without_markups() {
        let source = write_test_pdf("all_partial", &[(612.0, 792.0), (400.0, 300.0)]);
        let output = temp_pdf_path("all_partial_out");
        let _ = fs::remove_file(&output);

        export_all_pages_to_pdf(
            &source,
            &output,
            &[text_markup(0, "PAGE-ZERO-MARKER")],
            &BatesConfig::default(),
        )
        .expect("export all pages");

        let out = lopdf::Document::load(&output).expect("re-load exported PDF");
        let pages = out.get_pages();
        assert_eq!(pages.len(), 2);
        // Page 1 keeps its overlay; page 2 (no markups) stays content-free.
        let second = *pages.get(&2).expect("second page");
        let content = out.get_page_content(second);
        assert!(
            content.is_empty(),
            "unmarked page must pass through unchanged: {}",
            String::from_utf8_lossy(&content)
        );

        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&output);
    }

    #[test]
    fn export_page_to_pdf_rejects_an_out_of_range_page() {
        let source = write_test_pdf("range", &[(612.0, 792.0)]);
        let output = temp_pdf_path("range_out");
        let _ = fs::remove_file(&output);

        let error = export_page_to_pdf(&source, 5, &output, &[], &BatesConfig::default())
            .expect_err("page 5 must not export");
        match &error {
            DataError::Pdf(message) => {
                assert!(message.contains("out of range"), "message: {message}");
            }
            other => panic!("expected DataError::Pdf, got {other:?}"),
        }
        assert!(!output.exists(), "a failed export must write no output");

        let _ = fs::remove_file(&source);
    }

    #[test]
    fn export_page_to_pdf_reports_a_missing_source() {
        let missing = temp_pdf_path("does_not_exist");
        let _ = fs::remove_file(&missing);
        let output = temp_pdf_path("missing_out");
        let _ = fs::remove_file(&output);

        let error = export_page_to_pdf(&missing, 0, &output, &[], &BatesConfig::default())
            .expect_err("missing source must fail");
        assert!(matches!(error, DataError::Pdf(_)), "got {error:?}");
        assert!(!output.exists(), "a failed export must write no output");
    }
}
