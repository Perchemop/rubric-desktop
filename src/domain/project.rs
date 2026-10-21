//! Project model and workspace-level state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::document::{Document, Page};
use super::markup::MarkupKind;
use super::units::{clamp_precision, DEFAULT_PRECISION};

/// Application-level settings (preferences).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub author: String,
    pub auto_save: bool,
    pub auto_save_interval_secs: u32,
    /// UI scale factor (0.85..=1.5).
    pub ui_scale: f32,
    /// Whether to show the grid on the canvas.
    pub show_grid: bool,
    /// Whether to show measurement labels by default.
    pub show_measurements: bool,
    /// Decimal places (0..=6) used when formatting linear and area
    /// measurements. Older project files without this field default to 3.
    #[serde(default = "default_precision")]
    pub precision: u8,
}

/// Maximum number of entries retained in [`RecentList`].
pub const MAX_RECENT: usize = 10;

/// App-level list of recently opened project paths, most recent first.
///
/// Deliberately *not* part of [`Settings`] or [`Project`]: a project file is
/// shareable, so it must never embed the user's absolute local paths.
/// Recent projects live in the data layer instead (see `data::load_recent` /
/// `data::save_recent`); this type is the pure container they persist.
///
/// Serde representation is a plain JSON array of strings.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecentList(Vec<String>);

impl RecentList {
    /// Record `path` as the most recently opened project.
    ///
    /// An existing entry with the same exact string is removed and the path is
    /// moved to the front. The list is capped at [`MAX_RECENT`] entries; the
    /// oldest entries are dropped.
    pub fn push(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.0.retain(|existing| existing != &path);
        self.0.insert(0, path);
        self.0.truncate(MAX_RECENT);
    }

    /// Drop every entry with the exact string `path` (e.g. a file that no
    /// longer exists). No-op when it is not present.
    pub fn remove(&mut self, path: &str) {
        self.0.retain(|existing| existing != path);
    }

    /// Remove all entries from the list.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Iterate entries, most recent first.
    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }

    /// Entries as a slice, most recent first.
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// True when the list has no entries.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Serde default for [`Settings::precision`], also used by projects saved
/// before the preference existed.
fn default_precision() -> u8 {
    DEFAULT_PRECISION
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            author: "Reviewer".to_string(),
            auto_save: false,
            auto_save_interval_secs: 60,
            ui_scale: 1.0,
            show_grid: false,
            show_measurements: true,
            precision: default_precision(),
        }
    }
}

impl Settings {
    /// Set the measurement decimal precision, clamped to the supported
    /// `0..=6` range.
    ///
    /// Prefer this over assigning the field directly so out-of-range values
    /// from UI widgets cannot be persisted.
    pub fn set_precision(&mut self, precision: u8) {
        self.precision = clamp_precision(precision);
    }
}

/// A reusable toolbar preset grouping markup tools by family.
///
/// This is a tool-palette grouping only: it does not own markups and does not
/// control visibility or z-order. Markup visibility/locking lives in
/// [`super::document::Layer`] (one document owns many layers). The
/// `visible`/`locked` fields are retained for serialization compatibility and
/// future palette behaviour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolSet {
    pub id: Uuid,
    pub name: String,
    pub tools: Vec<MarkupKind>,
    #[serde(default)]
    pub visible: bool,
    #[serde(default)]
    pub locked: bool,
}

impl ToolSet {
    pub fn new(name: impl Into<String>, tools: Vec<MarkupKind>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            tools,
            visible: true,
            locked: false,
        }
    }

    pub fn default_review() -> Self {
        Self::new(
            "Review",
            vec![
                MarkupKind::Highlight,
                MarkupKind::Strikeout,
                MarkupKind::Underline,
                MarkupKind::Squiggly,
                MarkupKind::Text,
                MarkupKind::Callout,
                MarkupKind::Note,
                MarkupKind::Stamp,
            ],
        )
    }

    pub fn default_shapes() -> Self {
        Self::new(
            "Shapes",
            vec![
                MarkupKind::Rectangle,
                MarkupKind::Ellipse,
                MarkupKind::Cloud,
                MarkupKind::Polygon,
                MarkupKind::Polyline,
            ],
        )
    }

    pub fn default_measure() -> Self {
        Self::new(
            "Measure",
            vec![
                MarkupKind::MeasureLength,
                MarkupKind::MeasurePerimeter,
                MarkupKind::MeasureArea,
                MarkupKind::MeasureAngle,
                MarkupKind::MeasureDiameter,
                MarkupKind::MeasureRadius,
                MarkupKind::MeasureCount,
            ],
        )
    }
}

/// A workspace project containing one or more documents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    /// Absolute or relative path of the PDF backing this project.
    ///
    /// `None` means the project has no backing PDF (a blank, PDF-less project).
    /// Older `.rubric.json` files saved before PDF import existed have no such
    /// field and deserialize to `None` thanks to `#[serde(default)]`.
    #[serde(default)]
    pub source_pdf: Option<String>,
    pub documents: Vec<Document>,
    #[serde(default)]
    pub active_document: usize,
    #[serde(default)]
    pub tool_sets: Vec<ToolSet>,
    pub settings: Settings,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        let mut p = Self {
            id: Uuid::new_v4(),
            name: name.into(),
            source_pdf: None,
            documents: vec![Document::empty("Untitled Document")],
            active_document: 0,
            tool_sets: vec![
                ToolSet::default_review(),
                ToolSet::default_shapes(),
                ToolSet::default_measure(),
            ],
            settings: Settings::default(),
            created: now,
            modified: now,
        };
        if p.documents.is_empty() {
            p.documents.push(Document::empty("Untitled Document"));
            p.active_document = 0;
        }
        p
    }

    /// Build a project backed by a PDF whose page sizes are already known.
    ///
    /// `page_sizes` holds one `(width, height)` pair per PDF page, in document
    /// order and in PDF points. Each pair becomes a [`Page`] with a zero-based
    /// index, no rotation, and the paper-space scale used by [`Project::new`]
    /// (one point = one point). Pages smaller than one point are clamped by
    /// [`Page::with_size`].
    ///
    /// The single document is titled `name`, and both
    /// [`Project::source_pdf`] and [`Document::source_path`] record
    /// `source_pdf`.
    ///
    /// An empty `page_sizes` slice is degenerate (a zero-page document cannot
    /// be edited), so defensively it produces a single A4 page instead of
    /// panicking. Callers normally pass the page sizes returned by
    /// [`crate::data::read_pdf_info`]; a PDF that reports no pages yields this
    /// fallback rather than an empty document.
    pub fn from_pdf(
        name: impl Into<String>,
        source_pdf: impl Into<String>,
        page_sizes: &[(f32, f32)],
    ) -> Self {
        let name = name.into();
        let source_pdf = source_pdf.into();
        let pages: Vec<Page> = if page_sizes.is_empty() {
            vec![Page::a4(0)]
        } else {
            page_sizes
                .iter()
                .enumerate()
                .map(|(index, (width, height))| Page::letter(index).with_size(*width, *height))
                .collect()
        };

        let mut document = Document::new(name.clone(), pages);
        document.source_path = Some(source_pdf.clone());

        let mut project = Self::new(name);
        project.source_pdf = Some(source_pdf);
        project.documents = vec![document];
        project.active_document = 0;
        project
    }

    pub fn add_document(&mut self, d: Document) {
        self.documents.push(d);
        self.active_document = self.documents.len().saturating_sub(1);
        self.touch();
    }

    pub fn remove_document(&mut self, index: usize) {
        if index < self.documents.len() && self.documents.len() > 1 {
            self.documents.remove(index);
            if self.active_document >= self.documents.len() {
                self.active_document = self.documents.len() - 1;
            }
            self.touch();
        }
    }

    pub fn active(&self) -> Option<&Document> {
        self.documents.get(self.active_document)
    }

    pub fn active_mut(&mut self) -> Option<&mut Document> {
        self.documents.get_mut(self.active_document)
    }

    pub fn touch(&mut self) {
        self.modified = Utc::now();
    }

    pub fn has_documents(&self) -> bool {
        !self.documents.is_empty()
    }

    pub fn rename(&mut self, name: impl Into<String>) {
        self.name = name.into();
        self.touch();
    }
}

/// The current drawing tool selected in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    #[default]
    Select,
    // Review
    Highlight,
    Strikeout,
    Underline,
    Squiggly,
    Text,
    Callout,
    Note,
    Stamp,
    // Shapes/paths
    Rectangle,
    Ellipse,
    Cloud,
    Polygon,
    Polyline,
    Line,
    Arrow,
    Ink,
    // Measure
    MeasureLength,
    MeasurePerimeter,
    MeasureArea,
    MeasureAngle,
    MeasureDiameter,
    MeasureRadius,
    MeasureCount,
}

impl Tool {
    pub fn from_kind(k: MarkupKind) -> Self {
        match k {
            MarkupKind::Highlight => Tool::Highlight,
            MarkupKind::Strikeout => Tool::Strikeout,
            MarkupKind::Underline => Tool::Underline,
            MarkupKind::Squiggly => Tool::Squiggly,
            MarkupKind::Text => Tool::Text,
            MarkupKind::Callout => Tool::Callout,
            MarkupKind::Note => Tool::Note,
            MarkupKind::Stamp => Tool::Stamp,
            MarkupKind::Rectangle => Tool::Rectangle,
            MarkupKind::Ellipse => Tool::Ellipse,
            MarkupKind::Cloud => Tool::Cloud,
            MarkupKind::Polygon => Tool::Polygon,
            MarkupKind::Polyline => Tool::Polyline,
            MarkupKind::Line => Tool::Line,
            MarkupKind::Arrow => Tool::Arrow,
            MarkupKind::Ink => Tool::Ink,
            MarkupKind::MeasureLength => Tool::MeasureLength,
            MarkupKind::MeasurePerimeter => Tool::MeasurePerimeter,
            MarkupKind::MeasureArea => Tool::MeasureArea,
            MarkupKind::MeasureAngle => Tool::MeasureAngle,
            MarkupKind::MeasureDiameter => Tool::MeasureDiameter,
            MarkupKind::MeasureRadius => Tool::MeasureRadius,
            MarkupKind::MeasureCount => Tool::MeasureCount,
        }
    }

    pub fn to_kind(self) -> Option<MarkupKind> {
        Some(match self {
            Tool::Select => return None,
            Tool::Highlight => MarkupKind::Highlight,
            Tool::Strikeout => MarkupKind::Strikeout,
            Tool::Underline => MarkupKind::Underline,
            Tool::Squiggly => MarkupKind::Squiggly,
            Tool::Text => MarkupKind::Text,
            Tool::Callout => MarkupKind::Callout,
            Tool::Note => MarkupKind::Note,
            Tool::Stamp => MarkupKind::Stamp,
            Tool::Rectangle => MarkupKind::Rectangle,
            Tool::Ellipse => MarkupKind::Ellipse,
            Tool::Cloud => MarkupKind::Cloud,
            Tool::Polygon => MarkupKind::Polygon,
            Tool::Polyline => MarkupKind::Polyline,
            Tool::Line => MarkupKind::Line,
            Tool::Arrow => MarkupKind::Arrow,
            Tool::Ink => MarkupKind::Ink,
            Tool::MeasureLength => MarkupKind::MeasureLength,
            Tool::MeasurePerimeter => MarkupKind::MeasurePerimeter,
            Tool::MeasureArea => MarkupKind::MeasureArea,
            Tool::MeasureAngle => MarkupKind::MeasureAngle,
            Tool::MeasureDiameter => MarkupKind::MeasureDiameter,
            Tool::MeasureRadius => MarkupKind::MeasureRadius,
            Tool::MeasureCount => MarkupKind::MeasureCount,
        })
    }

    pub fn is_draw_tool(self) -> bool {
        self != Tool::Select
    }

    pub fn is_measure_tool(self) -> bool {
        matches!(
            self,
            Tool::MeasureLength
                | Tool::MeasurePerimeter
                | Tool::MeasureArea
                | Tool::MeasureAngle
                | Tool::MeasureDiameter
                | Tool::MeasureRadius
                | Tool::MeasureCount
        )
    }
}

/// Historical snapshot of a project for undo/redo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub project: Project,
}

/// A simple, deterministic command stack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct History {
    snapshots: Vec<ProjectSnapshot>,
    cursor: usize,
}

impl History {
    pub fn new(initial: Project) -> Self {
        Self {
            snapshots: vec![ProjectSnapshot { project: initial }],
            cursor: 0,
        }
    }

    pub fn current(&self) -> &Project {
        &self.snapshots[self.cursor].project
    }

    pub fn current_mut(&mut self) -> &mut Project {
        &mut self.snapshots[self.cursor].project
    }

    /// Push a new snapshot after the current one (truncating the redo branch).
    pub fn push(&mut self, project: Project) {
        // Truncate any redo history.
        self.snapshots.truncate(self.cursor + 1);
        self.snapshots.push(ProjectSnapshot { project });
        self.cursor = self.snapshots.len() - 1;
        // Keep history bounded (cheap snapshots for now; first slice small).
        if self.snapshots.len() > 128 {
            self.snapshots.remove(0);
            self.cursor = self.snapshots.len() - 1;
        }
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.snapshots.len()
    }

    pub fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.cursor -= 1;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.can_redo() {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    /// Replace current snapshot without creating a new branch entry (useful on
    /// live edits where we don't want every drag frame in history).
    pub fn overwrite_current(&mut self, project: Project) {
        self.snapshots[self.cursor] = ProjectSnapshot { project };
    }

    pub fn clear(&mut self, initial: Project) {
        *self = History::new(initial);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_precision_is_three() {
        assert_eq!(Settings::default().precision, 3);
    }

    #[test]
    fn settings_precision_setter_clamps_to_supported_range() {
        let mut settings = Settings::default();
        settings.set_precision(99);
        assert_eq!(settings.precision, 6);
        settings.set_precision(0);
        assert_eq!(settings.precision, 0);
        settings.set_precision(4);
        assert_eq!(settings.precision, 4);
    }

    #[test]
    fn settings_without_precision_field_loads_default() {
        // A project saved before the preference existed has no `precision`.
        let json = r#"{
            "author": "Reviewer",
            "auto_save": false,
            "auto_save_interval_secs": 60,
            "ui_scale": 1.0,
            "show_grid": false,
            "show_measurements": true
        }"#;
        let settings: Settings = serde_json::from_str(json).expect("deserialize");
        assert_eq!(settings.precision, 3);
    }

    #[test]
    fn settings_precision_serde_roundtrip() {
        let mut settings = Settings::default();
        settings.set_precision(5);
        let json = serde_json::to_string(&settings).expect("serialize");
        let back: Settings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, settings);
        assert_eq!(back.precision, 5);
    }

    #[test]
    fn recent_list_default_is_empty() {
        let recent = RecentList::default();
        assert!(recent.is_empty());
        assert!(recent.as_slice().is_empty());
    }

    #[test]
    fn recent_list_push_dedupes_and_moves_to_front() {
        let mut recent = RecentList::default();
        recent.push("a.rubric.json");
        recent.push("b.rubric.json");
        recent.push("a.rubric.json");
        assert_eq!(recent.as_slice(), &["a.rubric.json", "b.rubric.json"]);
    }

    #[test]
    fn recent_list_remove_drops_only_matching_entry() {
        let mut recent = RecentList::default();
        recent.push("a.rubric.json");
        recent.push("b.rubric.json");
        recent.remove("missing.rubric.json"); // no-op
        assert_eq!(recent.as_slice(), &["b.rubric.json", "a.rubric.json"]);
        recent.remove("a.rubric.json");
        assert_eq!(recent.as_slice(), &["b.rubric.json"]);
    }

    #[test]
    fn recent_list_push_caps_at_max_recent_dropping_oldest() {
        let mut recent = RecentList::default();
        for i in 0..12 {
            recent.push(format!("path_{i}.rubric.json"));
        }
        assert_eq!(recent.as_slice().len(), MAX_RECENT);
        assert_eq!(recent.as_slice()[0], "path_11.rubric.json");
        assert_eq!(recent.as_slice()[MAX_RECENT - 1], "path_2.rubric.json");
        assert!(recent.iter().all(|p| p != "path_0.rubric.json"));
        assert!(recent.iter().all(|p| p != "path_1.rubric.json"));
    }

    #[test]
    fn recent_list_clear_empties_list() {
        let mut recent = RecentList::default();
        recent.push("a.rubric.json");
        recent.push("b.rubric.json");
        recent.clear();
        assert!(recent.is_empty());
    }

    #[test]
    fn recent_list_serde_roundtrip_is_json_array_of_strings() {
        let mut recent = RecentList::default();
        recent.push("first.rubric.json");
        recent.push("second.rubric.json");
        let json = serde_json::to_string(&recent).expect("serialize");
        assert_eq!(json, r#"["second.rubric.json","first.rubric.json"]"#);
        let back: RecentList = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, recent);
    }

    #[test]
    fn settings_with_legacy_recent_paths_field_still_loads() {
        // Transient project files recorded recents inside Settings; serde
        // (no `deny_unknown_fields` in this codebase) ignores the now-unknown
        // field instead of failing.
        let json = r#"{
            "author": "Reviewer",
            "auto_save": false,
            "auto_save_interval_secs": 60,
            "ui_scale": 1.0,
            "show_grid": false,
            "show_measurements": true,
            "recent_paths": ["old.rubric.json"]
        }"#;
        let settings: Settings = serde_json::from_str(json).expect("deserialize");
        assert_eq!(settings.precision, 3);
    }

    #[test]
    fn project_new_has_no_source_pdf() {
        assert_eq!(Project::new("Plan").source_pdf, None);
    }

    #[test]
    fn project_source_pdf_serde_roundtrip() {
        let mut project = Project::new("Plan");
        project.source_pdf = Some("plans/site.pdf".into());
        let json = serde_json::to_string(&project).expect("serialize");
        let back: Project = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, project);
        assert_eq!(back.source_pdf.as_deref(), Some("plans/site.pdf"));
    }

    #[test]
    fn legacy_project_without_source_pdf_loads_none() {
        let mut value = serde_json::to_value(Project::new("Legacy")).expect("project json");
        value
            .as_object_mut()
            .expect("project object")
            .remove("source_pdf");

        let loaded: Project = serde_json::from_value(value).expect("deserialize legacy");
        assert_eq!(loaded.source_pdf, None);
    }

    #[test]
    fn from_pdf_builds_one_page_per_page_size() {
        let sizes = [(612.0, 792.0), (595.28, 841.89)];
        let project = Project::from_pdf("Site Plan", "plans/site.pdf", &sizes);

        assert_eq!(project.name, "Site Plan");
        assert_eq!(project.source_pdf.as_deref(), Some("plans/site.pdf"));
        assert_eq!(project.documents.len(), 1);
        assert_eq!(project.active_document, 0);

        let document = &project.documents[0];
        assert_eq!(document.title, "Site Plan");
        assert_eq!(document.source_path.as_deref(), Some("plans/site.pdf"));
        assert_eq!(document.page_count(), 2);

        let first = document.pages[0];
        assert_eq!(first.index, 0);
        assert_eq!(first.width, 612.0);
        assert_eq!(first.height, 792.0);
        assert_eq!(first.rotation, 0);
        assert_eq!(first.scale, Page::a4(0).scale);

        let second = document.pages[1];
        assert_eq!(second.index, 1);
        assert!((second.width - 595.28).abs() < 0.001);
        assert!((second.height - 841.89).abs() < 0.001);
    }

    #[test]
    fn from_pdf_roundtrips_through_serde() {
        let project = Project::from_pdf("Plan", "/tmp/plan.pdf", &[(100.0, 200.0)]);
        let json = serde_json::to_string(&project).expect("serialize");
        let back: Project = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(back, project);
        assert_eq!(back.source_pdf.as_deref(), Some("/tmp/plan.pdf"));
        assert_eq!(back.documents[0].pages[0].width, 100.0);
        assert_eq!(back.documents[0].pages[0].height, 200.0);
        assert_eq!(back.documents[0].pages[0].index, 0);
    }

    #[test]
    fn from_pdf_with_no_pages_falls_back_to_one_a4_page_without_panicking() {
        let project = Project::from_pdf("Empty", "empty.pdf", &[]);

        assert_eq!(project.documents.len(), 1);
        assert_eq!(project.documents[0].page_count(), 1);

        let page = project.documents[0].primary_page().expect("primary page");
        let a4 = Page::a4(0);
        assert_eq!(page.index, 0);
        assert_eq!(page.width, a4.width);
        assert_eq!(page.height, a4.height);
        assert_eq!(page.rotation, 0);
    }
}
