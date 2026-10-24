//! Export helpers for summary reports (spec F-304).
//!
//! Writing files is a data-layer concern; the domain [`Report`] stays pure.

use std::path::Path;

use crate::data::error::DataError;
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
