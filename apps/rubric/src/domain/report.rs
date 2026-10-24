//! Summary report generation (spec F-304).
//!
//! A [`Report`] is a derived snapshot of every markup in a [`Project`],
//! grouped by status, author or subject. Reports are pure domain data: they
//! serialise to JSON for the export stage and render to stable plain text via
//! [`Report::to_text`]. This module performs no I/O; writing files belongs to
//! the data/UI layers.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Write as _;

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::markup::{Markup, MarkupGeometry, MarkupKind, ReviewStatus};
use super::project::Project;
use super::units::MeasureUnit;

/// Group value used for empty author and subject strings.
///
/// A whitespace-only value counts as empty. Sections keyed by this value are
/// always sorted last.
pub const NONE_LABEL: &str = "(none)";

/// The field a [`Report`] groups its entries by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportGroupKey {
    /// Group by [`ReviewStatus`], using the status label as the key value.
    Status,
    /// Group by markup author, with empty authors falling into
    /// [`NONE_LABEL`].
    Author,
    /// Group by markup subject, with empty subjects falling into
    /// [`NONE_LABEL`].
    Subject,
}

impl ReportGroupKey {
    /// All grouping keys in display order (grouping picker).
    pub const ALL: [ReportGroupKey; 3] = [
        ReportGroupKey::Status,
        ReportGroupKey::Author,
        ReportGroupKey::Subject,
    ];

    /// Human-readable label for the grouping key, e.g. `"Status"`.
    pub fn as_str(self) -> &'static str {
        match self {
            ReportGroupKey::Status => "Status",
            ReportGroupKey::Author => "Author",
            ReportGroupKey::Subject => "Subject",
        }
    }

    /// The section key value this markup belongs to under this grouping.
    fn value_for(self, markup: &Markup) -> String {
        match self {
            ReportGroupKey::Status => markup.status.label().to_string(),
            ReportGroupKey::Author => normalise_group_value(&markup.author),
            ReportGroupKey::Subject => normalise_group_value(&markup.subject),
        }
    }
}

/// Trim a group value, mapping empty (or whitespace-only) values to
/// [`NONE_LABEL`].
fn normalise_group_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        NONE_LABEL.to_string()
    } else {
        trimmed.to_string()
    }
}

/// One reported markup: a flat, export-friendly projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportEntry {
    /// Stable identifier of the source markup.
    pub markup_id: Uuid,
    /// Title of the document containing the markup.
    pub document_title: String,
    /// Page number as shown to users (1-based; the domain stores 0-based).
    pub page: usize,
    /// Tool/kind of the markup; use [`MarkupKind::label`] for its display
    /// label.
    pub kind: MarkupKind,
    /// Review status copied from the markup.
    pub status: ReviewStatus,
    /// Markup author (may be empty).
    pub author: String,
    /// Markup subject (may be empty).
    pub subject: String,
    /// Optional extra text: the text of a text box/callout, the first comment
    /// of a sticky note, or the label of a measurement.
    pub detail: Option<String>,
}

/// One group of [`ReportEntry`] rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportSection {
    /// The field this group is keyed by.
    pub key: ReportGroupKey,
    /// The group value, e.g. `"Approved"`, an author name, or [`NONE_LABEL`].
    pub value: String,
    /// Entries in this group, sorted by document title, then page, then
    /// markup id.
    pub entries: Vec<ReportEntry>,
}

/// Markup counts per [`ReviewStatus`].
///
/// Every status is always present, including zeroes, so text and JSON output
/// have a stable shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StatusCounts {
    /// Markups with [`ReviewStatus::Open`].
    pub open: usize,
    /// Markups with [`ReviewStatus::InReview`].
    pub in_review: usize,
    /// Markups with [`ReviewStatus::Approved`].
    pub approved: usize,
    /// Markups with [`ReviewStatus::Rejected`].
    pub rejected: usize,
}

impl StatusCounts {
    /// Count for `status`.
    pub fn count(&self, status: ReviewStatus) -> usize {
        match status {
            ReviewStatus::Open => self.open,
            ReviewStatus::InReview => self.in_review,
            ReviewStatus::Approved => self.approved,
            ReviewStatus::Rejected => self.rejected,
        }
    }

    /// Total across all statuses.
    pub fn total(&self) -> usize {
        self.open + self.in_review + self.approved + self.rejected
    }

    /// Increment the count for `status`.
    fn add(&mut self, status: ReviewStatus) {
        match status {
            ReviewStatus::Open => self.open += 1,
            ReviewStatus::InReview => self.in_review += 1,
            ReviewStatus::Approved => self.approved += 1,
            ReviewStatus::Rejected => self.rejected += 1,
        }
    }
}

/// A complete summary report for a project.
///
/// Build reports from a [`Project`] with [`Report::from_project`] (current
/// time) or [`Report::from_project_at`] (fixed time, useful for deterministic
/// tests). The report is serialisable so a later stage can export it as JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// Project name at the time the report was generated.
    pub project_title: String,
    /// Generation timestamp in UTC.
    pub generated_at: DateTime<Utc>,
    /// Number of markups included in the report.
    pub total_markups: usize,
    /// Markup counts per review status.
    pub status_counts: StatusCounts,
    /// Grouped entries, sorted by value with [`NONE_LABEL`] last.
    pub sections: Vec<ReportSection>,
}

impl Report {
    /// Build a report from `project`, grouping markups by `key`, stamped with
    /// the current UTC time.
    pub fn from_project(project: &Project, key: ReportGroupKey) -> Self {
        Self::from_project_at(project, key, Utc::now())
    }

    /// Build a report from `project` with an explicit generation timestamp.
    ///
    /// Every document in the project contributes its markups. Sections are
    /// sorted by group value using a plain (locale-independent) string
    /// comparison, with [`NONE_LABEL`] forced last. Entries within a section
    /// are sorted by document title, then 1-based page number, then markup id.
    ///
    /// Measurements are labelled using the markup's page scale unit and the
    /// project's precision preference, falling back to PDF points when the
    /// page is missing.
    pub fn from_project_at(
        project: &Project,
        key: ReportGroupKey,
        generated_at: DateTime<Utc>,
    ) -> Self {
        let mut total_markups = 0;
        let mut status_counts = StatusCounts::default();
        let mut groups: BTreeMap<String, Vec<ReportEntry>> = BTreeMap::new();

        for document in &project.documents {
            for markup in &document.markups {
                let unit = document
                    .pages
                    .get(markup.page)
                    .map(|page| page.scale.unit)
                    .unwrap_or(MeasureUnit::Point);
                let entry = ReportEntry {
                    markup_id: markup.id,
                    document_title: document.title.clone(),
                    page: markup.page.saturating_add(1),
                    kind: markup.kind,
                    status: markup.status,
                    author: markup.author.clone(),
                    subject: markup.subject.clone(),
                    detail: entry_detail(markup, unit, project.settings.precision),
                };
                groups.entry(key.value_for(markup)).or_default().push(entry);
                total_markups += 1;
                status_counts.add(markup.status);
            }
        }

        let mut sections: Vec<ReportSection> = groups
            .into_iter()
            .map(|(value, mut entries)| {
                entries.sort_by(|a, b| {
                    a.document_title
                        .cmp(&b.document_title)
                        .then_with(|| a.page.cmp(&b.page))
                        .then_with(|| a.markup_id.cmp(&b.markup_id))
                });
                ReportSection {
                    key,
                    value,
                    entries,
                }
            })
            .collect();

        sections.sort_by(
            |a, b| match (a.value == NONE_LABEL, b.value == NONE_LABEL) {
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                _ => a.value.cmp(&b.value),
            },
        );

        Self {
            project_title: project.name.clone(),
            generated_at,
            total_markups,
            status_counts,
            sections,
        }
    }

    /// Render the report as stable, human-readable plain text.
    ///
    /// The timestamp is rendered as RFC 3339 in UTC (`2026-01-02T03:04:05Z`)
    /// so output never depends on the host locale or timezone. Each section
    /// starts with `== <value> (<count>) ==`; each entry is one aligned line
    /// with page, kind, subject, author, status and optional detail.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "Rubric Summary Report");
        let _ = writeln!(out, "Project: {}", one_line(&self.project_title));
        let _ = writeln!(
            out,
            "Generated: {}",
            self.generated_at.to_rfc3339_opts(SecondsFormat::Secs, true)
        );
        let _ = writeln!(out);
        let _ = writeln!(out, "Total markups: {}", self.total_markups);
        for status in ReviewStatus::ALL {
            let _ = writeln!(
                out,
                "  {}: {}",
                status.label(),
                self.status_counts.count(status)
            );
        }

        for section in &self.sections {
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "== {} ({}) ==",
                one_line(&section.value),
                section.entries.len()
            );
            if section.entries.is_empty() {
                continue;
            }

            let page_width = section
                .entries
                .iter()
                .map(|entry| entry.page.to_string().len())
                .max()
                .unwrap_or(1);
            let kind_width = section
                .entries
                .iter()
                .map(|entry| entry.kind.label().chars().count())
                .max()
                .unwrap_or(0);
            let subject_width = section
                .entries
                .iter()
                .map(|entry| one_line(&entry.subject).chars().count())
                .max()
                .unwrap_or(0);
            let author_width = section
                .entries
                .iter()
                .map(|entry| one_line(&entry.author).chars().count())
                .max()
                .unwrap_or(0);
            let status_width = section
                .entries
                .iter()
                .map(|entry| entry.status.label().chars().count())
                .max()
                .unwrap_or(0);

            for entry in &section.entries {
                let page = entry.page.to_string();
                let kind = entry.kind.label();
                let subject = one_line(&entry.subject);
                let author = one_line(&entry.author);
                let status = entry.status.label();
                let detail = entry.detail.as_deref().map(one_line).unwrap_or_default();
                let line = format!(
                    "  {}  {}  {}  {}  {}  {}",
                    pad_left(&page, page_width),
                    pad_right(kind, kind_width),
                    pad_right(&subject, subject_width),
                    pad_right(&author, author_width),
                    pad_right(status, status_width),
                    detail
                );
                let _ = writeln!(out, "{}", line.trim_end());
            }
        }

        out
    }
}

/// Collapse all whitespace runs into single spaces and trim the ends, so a
/// multi-line comment or text box still fits on one report line.
fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Left-align `value` in a column `width` characters wide (never truncates).
fn pad_right(value: &str, width: usize) -> String {
    let len = value.chars().count();
    if len >= width {
        value.to_string()
    } else {
        let mut padded = value.to_string();
        padded.push_str(&" ".repeat(width - len));
        padded
    }
}

/// Right-align `value` in a column `width` characters wide (never truncates).
fn pad_left(value: &str, width: usize) -> String {
    let len = value.chars().count();
    if len >= width {
        value.to_string()
    } else {
        let mut padded = " ".repeat(width - len);
        padded.push_str(value);
        padded
    }
}

/// Extra detail text for a markup, if any.
///
/// Measurement labels take priority (they are derived data). Text boxes and
/// callouts use their geometry text; sticky notes use their first comment.
/// Text-like markups with no geometry text fall back to their first comment.
/// The result is whitespace-normalised to a single line.
fn entry_detail(markup: &Markup, unit: MeasureUnit, precision: u8) -> Option<String> {
    if let Some(measurement) = markup
        .measurement
        .as_ref()
        .filter(|_| markup.kind.is_measure())
    {
        return Some(measurement.label_with(unit, precision));
    }

    let geometry_text = match &markup.geometry {
        MarkupGeometry::Text { text, .. } | MarkupGeometry::Callout { text, .. } => text.as_str(),
        _ => "",
    };
    let text = if !geometry_text.trim().is_empty() {
        geometry_text
    } else if matches!(
        markup.kind,
        MarkupKind::Text | MarkupKind::Callout | MarkupKind::Note
    ) {
        markup
            .comments
            .first()
            .map(|comment| comment.text.as_str())
            .unwrap_or("")
    } else {
        ""
    };

    let text = one_line(text);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::document::{Document, Page};
    use crate::domain::geom::{Pt, Rect};
    use crate::domain::markup::Comment;

    fn fixed_time() -> DateTime<Utc> {
        "2026-01-02T03:04:05Z"
            .parse()
            .expect("fixed timestamp parses")
    }

    /// Build a plain markup with a deterministic id and explicit review
    /// metadata.
    fn sample_markup(
        id: u128,
        page: usize,
        kind: MarkupKind,
        author: &str,
        subject: &str,
        status: ReviewStatus,
    ) -> Markup {
        let mut markup = Markup::new(
            page,
            kind,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 10.0, 10.0)),
            author,
        );
        markup.id = Uuid::from_u128(id);
        markup.subject = subject.to_string();
        markup.status = status;
        markup
    }

    fn add(project: &mut Project, markup: Markup) {
        project.documents[0].add_markup(markup);
    }

    fn section_values(report: &Report) -> Vec<&str> {
        report
            .sections
            .iter()
            .map(|section| section.value.as_str())
            .collect()
    }

    fn entry(report: &Report, id: u128) -> &ReportEntry {
        report
            .sections
            .iter()
            .flat_map(|section| section.entries.iter())
            .find(|entry| entry.markup_id == Uuid::from_u128(id))
            .expect("entry present")
    }

    #[test]
    fn group_key_labels_and_serde_are_snake_case() {
        assert_eq!(ReportGroupKey::ALL.len(), 3);
        assert_eq!(ReportGroupKey::Status.as_str(), "Status");
        assert_eq!(ReportGroupKey::Author.as_str(), "Author");
        assert_eq!(ReportGroupKey::Subject.as_str(), "Subject");

        let cases = [
            (ReportGroupKey::Status, "\"status\""),
            (ReportGroupKey::Author, "\"author\""),
            (ReportGroupKey::Subject, "\"subject\""),
        ];
        for (key, json) in cases {
            assert_eq!(serde_json::to_string(&key).expect("serialize"), json);
            let back: ReportGroupKey = serde_json::from_str(json).expect("deserialize");
            assert_eq!(back, key);
        }
    }

    #[test]
    fn groups_by_status_with_sorted_sections_and_counts() {
        let mut project = Project::new("Plan");
        add(
            &mut project,
            sample_markup(
                1,
                0,
                MarkupKind::Highlight,
                "Alice",
                "General",
                ReviewStatus::Approved,
            ),
        );
        add(
            &mut project,
            sample_markup(
                2,
                0,
                MarkupKind::Highlight,
                "Alice",
                "General",
                ReviewStatus::Open,
            ),
        );
        add(
            &mut project,
            sample_markup(
                3,
                0,
                MarkupKind::Highlight,
                "Bob",
                "General",
                ReviewStatus::Open,
            ),
        );
        add(
            &mut project,
            sample_markup(
                4,
                0,
                MarkupKind::Highlight,
                "Bob",
                "General",
                ReviewStatus::Rejected,
            ),
        );

        let report = Report::from_project_at(&project, ReportGroupKey::Status, fixed_time());

        assert_eq!(report.project_title, "Plan");
        assert_eq!(report.total_markups, 4);
        assert_eq!(report.status_counts.open, 2);
        assert_eq!(report.status_counts.in_review, 0);
        assert_eq!(report.status_counts.approved, 1);
        assert_eq!(report.status_counts.rejected, 1);
        assert_eq!(report.status_counts.total(), 4);

        // Sections are ordered by key value, not by status picker order.
        assert_eq!(section_values(&report), ["Approved", "Open", "Rejected"]);
        let counts: Vec<usize> = report
            .sections
            .iter()
            .map(|section| section.entries.len())
            .collect();
        assert_eq!(counts, [1, 2, 1]);
        assert!(report
            .sections
            .iter()
            .all(|section| section.key == ReportGroupKey::Status));
        assert_eq!(report.sections[2].entries[0].status, ReviewStatus::Rejected);
    }

    #[test]
    fn groups_by_author_and_places_none_last() {
        let mut project = Project::new("Plan");
        add(
            &mut project,
            sample_markup(
                1,
                0,
                MarkupKind::Highlight,
                "Alice",
                "General",
                ReviewStatus::Open,
            ),
        );
        add(
            &mut project,
            sample_markup(
                2,
                0,
                MarkupKind::Highlight,
                "",
                "General",
                ReviewStatus::Open,
            ),
        );
        add(
            &mut project,
            sample_markup(
                3,
                0,
                MarkupKind::Highlight,
                "   ",
                "General",
                ReviewStatus::Open,
            ),
        );
        add(
            &mut project,
            sample_markup(
                4,
                0,
                MarkupKind::Highlight,
                "Bob",
                "General",
                ReviewStatus::Open,
            ),
        );

        let report = Report::from_project_at(&project, ReportGroupKey::Author, fixed_time());

        assert_eq!(section_values(&report), ["Alice", "Bob", NONE_LABEL]);
        assert_eq!(report.sections[0].entries.len(), 1);
        assert_eq!(report.sections[1].entries.len(), 1);
        assert_eq!(
            report.sections[2].entries.len(),
            2,
            "empty and whitespace-only authors share the (none) bucket"
        );
        assert!(report
            .sections
            .iter()
            .all(|section| section.key == ReportGroupKey::Author));
    }

    #[test]
    fn groups_by_subject_and_places_none_last() {
        let mut project = Project::new("Plan");
        add(
            &mut project,
            sample_markup(
                1,
                0,
                MarkupKind::Highlight,
                "Alice",
                "Electrical",
                ReviewStatus::Open,
            ),
        );
        add(
            &mut project,
            sample_markup(2, 0, MarkupKind::Highlight, "Alice", "", ReviewStatus::Open),
        );
        add(
            &mut project,
            sample_markup(
                3,
                0,
                MarkupKind::Highlight,
                "Alice",
                "Plumbing",
                ReviewStatus::Open,
            ),
        );

        let report = Report::from_project_at(&project, ReportGroupKey::Subject, fixed_time());

        assert_eq!(
            section_values(&report),
            ["Electrical", "Plumbing", NONE_LABEL]
        );
        assert_eq!(report.sections[2].entries.len(), 1);
        assert_eq!(
            report.sections[2].entries[0].subject, "",
            "the entry keeps the raw subject; only the section key is normalised"
        );
        assert!(report
            .sections
            .iter()
            .all(|section| section.key == ReportGroupKey::Subject));
    }

    #[test]
    fn entries_sort_by_document_title_page_and_markup_id() {
        let mut doc_a = Document::new("A", vec![Page::a4(0), Page::a4(1)]);
        doc_a.add_markup(sample_markup(
            9,
            1,
            MarkupKind::Rectangle,
            "Alice",
            "General",
            ReviewStatus::Open,
        ));
        doc_a.add_markup(sample_markup(
            8,
            0,
            MarkupKind::Rectangle,
            "Alice",
            "General",
            ReviewStatus::Open,
        ));
        doc_a.add_markup(sample_markup(
            7,
            0,
            MarkupKind::Rectangle,
            "Alice",
            "General",
            ReviewStatus::Open,
        ));

        let mut doc_b = Document::new("B", vec![Page::a4(0)]);
        doc_b.add_markup(sample_markup(
            1,
            0,
            MarkupKind::Rectangle,
            "Alice",
            "General",
            ReviewStatus::Open,
        ));

        let mut project = Project::new("Multi-doc");
        // Deliberately list B first: sorting must not follow document order.
        project.documents = vec![doc_b, doc_a];

        let report = Report::from_project_at(&project, ReportGroupKey::Author, fixed_time());
        assert_eq!(report.sections.len(), 1);
        let ids: Vec<Uuid> = report.sections[0]
            .entries
            .iter()
            .map(|entry| entry.markup_id)
            .collect();
        assert_eq!(
            ids,
            [
                Uuid::from_u128(7),
                Uuid::from_u128(8),
                Uuid::from_u128(9),
                Uuid::from_u128(1),
            ]
        );

        let pages_and_titles: Vec<(usize, &str)> = report.sections[0]
            .entries
            .iter()
            .map(|entry| (entry.page, entry.document_title.as_str()))
            .collect();
        assert_eq!(
            pages_and_titles,
            [(1, "A"), (1, "A"), (2, "A"), (1, "B")],
            "pages are 1-based"
        );
    }

    #[test]
    fn entries_carry_page_kind_and_detail() {
        let mut project = Project::new("Details");

        let mut text = Markup::new(
            0,
            MarkupKind::Text,
            MarkupGeometry::Text {
                at: Pt::new(10.0, 10.0),
                wrap: 80.0,
                text: "  Fix\n this ".to_string(),
            },
            "Alice",
        );
        text.id = Uuid::from_u128(1);
        text.status = ReviewStatus::Approved;
        add(&mut project, text);

        let mut note = Markup::new(
            0,
            MarkupKind::Note,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 40.0, 24.0)),
            "Alice",
        );
        note.id = Uuid::from_u128(2);
        note.comments.push(Comment::new("Alice", "Check\nnote"));
        add(&mut project, note);

        let mut measure = Markup::new(
            0,
            MarkupKind::MeasureLength,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(72.0, 0.0),
            },
            "Alice",
        );
        measure.id = Uuid::from_u128(3);
        add(&mut project, measure);

        let plain = sample_markup(
            4,
            0,
            MarkupKind::Rectangle,
            "Alice",
            "General",
            ReviewStatus::Open,
        );
        add(&mut project, plain);

        let report = Report::from_project_at(&project, ReportGroupKey::Author, fixed_time());
        assert_eq!(report.sections.len(), 1);

        let text_entry = entry(&report, 1);
        assert_eq!(text_entry.page, 1, "page 0 is reported as page 1");
        assert_eq!(text_entry.kind, MarkupKind::Text);
        assert_eq!(text_entry.detail.as_deref(), Some("Fix this"));

        let note_entry = entry(&report, 2);
        assert_eq!(note_entry.kind, MarkupKind::Note);
        assert_eq!(note_entry.detail.as_deref(), Some("Check note"));

        let measure_entry = entry(&report, 3);
        assert_eq!(measure_entry.kind, MarkupKind::MeasureLength);
        assert_eq!(
            measure_entry.detail.as_deref(),
            Some("72.000 pt"),
            "measurements use the page scale unit (paper scale = points)"
        );

        assert_eq!(
            entry(&report, 4).detail,
            None,
            "non-text markups without a measurement have no detail"
        );
    }

    #[test]
    fn empty_project_report_has_zero_counts_and_header_only_text() {
        let project = Project::new("Empty");
        let report = Report::from_project_at(&project, ReportGroupKey::Status, fixed_time());

        assert_eq!(report.total_markups, 0);
        assert_eq!(report.status_counts, StatusCounts::default());
        assert_eq!(report.status_counts.total(), 0);
        assert!(report.sections.is_empty());

        let text = report.to_text();
        assert!(!text.is_empty());
        assert!(text.starts_with("Rubric Summary Report\n"));
        assert!(text.contains("Project: Empty"));
        assert!(text.contains("Generated: 2026-01-02T03:04:05Z"));
        assert!(text.contains("Total markups: 0"));
        assert!(text.contains("  Open: 0"));
        assert!(text.contains("  In review: 0"));
        assert!(text.contains("  Approved: 0"));
        assert!(text.contains("  Rejected: 0"));
        assert!(!text.contains("=="), "no empty sections are rendered");
        assert!(text.ends_with('\n'));
    }

    #[test]
    fn from_project_uses_the_project_title_and_current_time() {
        let project = Project::new("Now");
        let report = Report::from_project(&project, ReportGroupKey::Status);

        assert_eq!(report.project_title, "Now");
        assert_eq!(report.total_markups, 0);
        assert!(report.sections.is_empty());
    }

    #[test]
    fn report_serde_roundtrip() {
        let mut project = Project::new("Serde Plan");
        add(
            &mut project,
            sample_markup(
                1,
                0,
                MarkupKind::Highlight,
                "Alice",
                "Electrical",
                ReviewStatus::InReview,
            ),
        );
        let report = Report::from_project_at(&project, ReportGroupKey::Subject, fixed_time());

        let json = serde_json::to_string(&report).expect("serialize");
        let back: Report = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, report);
        assert_eq!(back.status_counts.in_review, 1);
        assert_eq!(back.sections.len(), 1);
        assert_eq!(back.sections[0].value, "Electrical");
    }

    #[test]
    fn to_text_is_deterministic_and_aligned() {
        let mut project = Project::new("Text Plan");
        project.documents[0].add_page(Page::a4(1));

        let mut text = Markup::new(
            0,
            MarkupKind::Text,
            MarkupGeometry::Text {
                at: Pt::new(10.0, 10.0),
                wrap: 80.0,
                text: "Fix this".to_string(),
            },
            "Alice",
        );
        text.id = Uuid::from_u128(1);
        text.subject = "General".to_string();
        text.status = ReviewStatus::Approved;
        add(&mut project, text);

        let highlight = sample_markup(
            2,
            1,
            MarkupKind::Highlight,
            "Bob",
            "General",
            ReviewStatus::Open,
        );
        add(&mut project, highlight);

        let at = fixed_time();
        let first = Report::from_project_at(&project, ReportGroupKey::Author, at);
        let second = Report::from_project_at(&project, ReportGroupKey::Author, at);
        let first_text = first.to_text();

        assert_eq!(first, second);
        assert_eq!(first_text, first.to_text());
        assert_eq!(first_text, second.to_text());
        assert!(first_text.contains("Generated: 2026-01-02T03:04:05Z"));
        assert!(first_text.contains("Total markups: 2"));
        assert!(first_text.contains("  Approved: 1"));
        assert!(first_text.contains("  Open: 1"));
        assert!(first_text.contains("== Alice (1) =="));
        assert!(first_text.contains("== Bob (1) =="));
        assert!(
            first_text
                .lines()
                .any(|line| line == "  1  Text Box  General  Alice  Approved  Fix this"),
            "aligned entry line missing:\n{first_text}"
        );
        assert!(
            first_text
                .lines()
                .any(|line| line == "  2  Highlight  General  Bob  Open"),
            "detail-less entry line should have no trailing padding:\n{first_text}"
        );
    }

    #[test]
    fn whitespace_only_subject_groups_under_none_keeping_raw_value() {
        let mut project = Project::new("Subjects");
        add(
            &mut project,
            sample_markup(
                1,
                0,
                MarkupKind::Highlight,
                "Alice",
                "  \t ",
                ReviewStatus::Open,
            ),
        );
        add(
            &mut project,
            sample_markup(
                2,
                0,
                MarkupKind::Highlight,
                "Alice",
                "Piping",
                ReviewStatus::Open,
            ),
        );
        add(
            &mut project,
            sample_markup(
                3,
                0,
                MarkupKind::Highlight,
                "Alice",
                "\n ",
                ReviewStatus::Open,
            ),
        );

        let report = Report::from_project_at(&project, ReportGroupKey::Subject, fixed_time());

        assert_eq!(section_values(&report), ["Piping", NONE_LABEL]);
        assert_eq!(
            report.sections[1].entries.len(),
            2,
            "tab/newline-only subjects join the empty ones under (none)"
        );
        let raw: Vec<&str> = report.sections[1]
            .entries
            .iter()
            .map(|entry| entry.subject.as_str())
            .collect();
        assert_eq!(
            raw,
            ["  \t ", "\n "],
            "entries keep the raw subject; only the section key is normalised"
        );
        assert!(report
            .sections
            .iter()
            .all(|section| section.key == ReportGroupKey::Subject));
    }

    #[test]
    fn multi_document_report_keeps_titles_and_merges_equal_titled_docs() {
        let mut doc_one = Document::new("Plan", vec![Page::a4(0)]);
        doc_one.add_markup(sample_markup(
            1,
            0,
            MarkupKind::Highlight,
            "Alice",
            "General",
            ReviewStatus::Open,
        ));

        let mut doc_two = Document::new("Plan", vec![Page::a4(0), Page::a4(1)]);
        doc_two.add_markup(sample_markup(
            2,
            1,
            MarkupKind::Highlight,
            "Alice",
            "General",
            ReviewStatus::Open,
        ));

        let mut project = Project::new("Same Title");
        project.documents = vec![doc_one, doc_two];

        let report = Report::from_project_at(&project, ReportGroupKey::Author, fixed_time());

        assert_eq!(report.total_markups, 2);
        assert_eq!(report.status_counts.total(), 2);
        assert_eq!(
            report.sections.len(),
            1,
            "documents sharing a title still form a single author group"
        );
        let rows: Vec<(u128, &str, usize)> = report.sections[0]
            .entries
            .iter()
            .map(|entry| {
                (
                    entry.markup_id.as_u128(),
                    entry.document_title.as_str(),
                    entry.page,
                )
            })
            .collect();
        assert_eq!(
            rows,
            [(1, "Plan", 1), (2, "Plan", 2)],
            "both docs' markups survive with their own title and 1-based page, \
             sorted by page"
        );
    }

    #[test]
    fn detail_selection_falls_back_to_comments_and_honours_precision() {
        let mut project = Project::new("Detail Gaps");

        // Text box with empty geometry text falls back to the first comment.
        let mut empty_text = Markup::new(
            0,
            MarkupKind::Text,
            MarkupGeometry::Text {
                at: Pt::new(10.0, 10.0),
                wrap: 80.0,
                text: String::new(),
            },
            "Alice",
        );
        empty_text.id = Uuid::from_u128(1);
        empty_text
            .comments
            .push(Comment::new("Alice", "Fallback comment"));
        add(&mut project, empty_text);

        // Whitespace-only geometry text counts as empty too.
        let mut blank_text = Markup::new(
            0,
            MarkupKind::Text,
            MarkupGeometry::Text {
                at: Pt::new(10.0, 30.0),
                wrap: 80.0,
                text: "   \n\t ".to_string(),
            },
            "Alice",
        );
        blank_text.id = Uuid::from_u128(2);
        blank_text
            .comments
            .push(Comment::new("Alice", "Whitespace fallback"));
        add(&mut project, blank_text);

        // Non-empty geometry text wins over comments.
        let mut preferred = Markup::new(
            0,
            MarkupKind::Text,
            MarkupGeometry::Text {
                at: Pt::new(10.0, 50.0),
                wrap: 80.0,
                text: "Primary text".to_string(),
            },
            "Alice",
        );
        preferred.id = Uuid::from_u128(3);
        preferred
            .comments
            .push(Comment::new("Alice", "Secondary comment"));
        add(&mut project, preferred);

        // A callout reports its geometry text.
        let mut callout = Markup::new(
            0,
            MarkupKind::Callout,
            MarkupGeometry::Callout {
                rect: Rect::new(0.0, 0.0, 100.0, 40.0),
                tail: Pt::new(50.0, 60.0),
                text: "Callout text".to_string(),
            },
            "Alice",
        );
        callout.id = Uuid::from_u128(4);
        add(&mut project, callout);

        // ...and falls back to its first comment when its text is empty.
        let mut blank_callout = Markup::new(
            0,
            MarkupKind::Callout,
            MarkupGeometry::Callout {
                rect: Rect::new(0.0, 80.0, 100.0, 40.0),
                tail: Pt::new(50.0, 140.0),
                text: String::new(),
            },
            "Alice",
        );
        blank_callout.id = Uuid::from_u128(5);
        blank_callout
            .comments
            .push(Comment::new("Bob", "Tail note"));
        add(&mut project, blank_callout);

        // A text box with neither text nor comments has no detail.
        let mut silent = Markup::new(
            0,
            MarkupKind::Text,
            MarkupGeometry::Text {
                at: Pt::new(10.0, 90.0),
                wrap: 80.0,
                text: String::new(),
            },
            "Alice",
        );
        silent.id = Uuid::from_u128(6);
        add(&mut project, silent);

        // A measurement label uses the project's precision preference.
        let mut measure = Markup::new(
            0,
            MarkupKind::MeasureLength,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(72.0, 0.0),
            },
            "Alice",
        );
        measure.id = Uuid::from_u128(7);
        add(&mut project, measure);

        project.settings.set_precision(1);
        let report = Report::from_project_at(&project, ReportGroupKey::Author, fixed_time());
        assert_eq!(
            entry(&report, 1).detail.as_deref(),
            Some("Fallback comment"),
            "empty text box falls back to its first comment"
        );
        assert_eq!(
            entry(&report, 2).detail.as_deref(),
            Some("Whitespace fallback"),
            "whitespace-only text counts as empty"
        );
        assert_eq!(
            entry(&report, 3).detail.as_deref(),
            Some("Primary text"),
            "geometry text wins over comments"
        );
        assert_eq!(entry(&report, 4).detail.as_deref(), Some("Callout text"));
        assert_eq!(
            entry(&report, 5).detail.as_deref(),
            Some("Tail note"),
            "callout with empty text falls back to its first comment"
        );
        assert_eq!(entry(&report, 6).detail, None);
        assert_eq!(
            entry(&report, 7).detail.as_deref(),
            Some("72.0 pt"),
            "measurement detail honours settings.precision = 1"
        );

        project.settings.set_precision(0);
        let report = Report::from_project_at(&project, ReportGroupKey::Author, fixed_time());
        assert_eq!(
            entry(&report, 7).detail.as_deref(),
            Some("72 pt"),
            "measurement detail follows the updated precision"
        );
    }

    #[test]
    fn status_counts_answer_every_variant_and_total() {
        let mut project = Project::new("Mixed");
        let statuses = [
            (1u128, ReviewStatus::Open),
            (2, ReviewStatus::InReview),
            (3, ReviewStatus::Approved),
            (4, ReviewStatus::Rejected),
            (5, ReviewStatus::Open),
        ];
        for (id, status) in statuses {
            add(
                &mut project,
                sample_markup(id, 0, MarkupKind::Highlight, "Alice", "General", status),
            );
        }

        let report = Report::from_project_at(&project, ReportGroupKey::Status, fixed_time());

        let expected = [
            (ReviewStatus::Open, 2),
            (ReviewStatus::InReview, 1),
            (ReviewStatus::Approved, 1),
            (ReviewStatus::Rejected, 1),
        ];
        for (status, count) in expected {
            assert_eq!(
                report.status_counts.count(status),
                count,
                "count for {status:?}"
            );
        }
        assert_eq!(report.status_counts.total(), 5);
        assert_eq!(report.total_markups, 5);

        let empty = StatusCounts::default();
        for status in ReviewStatus::ALL {
            assert_eq!(
                empty.count(status),
                0,
                "{status:?} starts at zero so output shape is stable"
            );
        }
        assert_eq!(empty.total(), 0);
    }

    #[test]
    fn to_text_covers_every_section_header_and_has_no_trailing_whitespace() {
        let mut project = Project::new("Text Gaps");
        project.documents[0].add_page(Page::a4(1));
        add(
            &mut project,
            sample_markup(
                1,
                0,
                MarkupKind::Highlight,
                "Alice",
                "General",
                ReviewStatus::Approved,
            ),
        );
        add(
            &mut project,
            sample_markup(
                2,
                1,
                MarkupKind::Highlight,
                "Bob",
                "General",
                ReviewStatus::Open,
            ),
        );
        add(
            &mut project,
            sample_markup(
                3,
                0,
                MarkupKind::Highlight,
                "   ",
                "General",
                ReviewStatus::Open,
            ),
        );

        let report = Report::from_project_at(&project, ReportGroupKey::Author, fixed_time());
        assert_eq!(
            report.generated_at,
            fixed_time(),
            "generated_at is exactly the supplied timestamp"
        );
        assert_eq!(section_values(&report), ["Alice", "Bob", NONE_LABEL]);

        let text = report.to_text();

        // Every section value gets its `== value (count) ==` header.
        for section in &report.sections {
            let header = format!("== {} ({}) ==", section.value, section.entries.len());
            assert!(
                text.contains(&header),
                "missing section header {header:?} in:\n{text}"
            );
        }

        // No line ever ends in whitespace (stable for golden-file diffs).
        for line in text.lines() {
            assert_eq!(
                line,
                line.trim_end(),
                "trailing whitespace on line {line:?}"
            );
        }

        // RFC 3339 in UTC with the 'Z' marker, seconds precision.
        let generated = text
            .lines()
            .find(|line| line.starts_with("Generated: "))
            .expect("generated line present");
        assert_eq!(
            generated, "Generated: 2026-01-02T03:04:05Z",
            "RFC3339 timestamp must carry the 'Z' UTC marker"
        );
    }
}
