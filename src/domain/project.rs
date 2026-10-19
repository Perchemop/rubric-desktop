//! Project model and workspace-level state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::document::Document;
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
}
