//! Domain module re-exports and common types.

pub mod action;
pub mod diff;
pub mod document;
pub mod geom;
pub mod grid;
pub mod markup;
pub mod markup_list;
pub mod project;
pub mod report;
pub mod style;
pub mod style_preset;
pub mod theme;
pub mod thumbs;
pub mod units;

pub use action::EditorAction;
pub use diff::{diff_documents, diff_projects, DiffResult, MarkupChange, MarkupSummaryRow};
pub use document::{Document, Layer, LayerError, Page};
pub use geom::{angle_degrees, path_length, polygon_area, Pt, Rect};
pub use grid::{nice_step, ruler_ticks, Grid};
pub use markup::{
    Comment, Markup, MarkupGeometry, MarkupKind, Measurement, ReviewStatus, StampPreset, ToolGroup,
};
pub use markup_list::{MarkupGroupKey, MarkupList, MarkupListFilter, MarkupListSort, MarkupRow};
pub use project::{
    auto_save_interval_label, History, Project, ProjectSnapshot, RecentList, Settings, Tool,
    ToolSet, AUTO_SAVE_INTERVALS, CURRENT_SCHEMA_VERSION, MAX_RECENT, MIN_AUTO_SAVE_INTERVAL_SECS,
};
pub use report::{Report, ReportEntry, ReportGroupKey, ReportSection, StatusCounts, NONE_LABEL};
pub use style::{palette, Color, DashPattern, FillMode, MarkupStyle};
pub use style_preset::{StylePreset, StylePresetLibrary};
pub use theme::{AppPrefs, Theme};
pub use thumbs::{thumb_layout, ThumbLayout};
pub use units::{
    clamp_precision, format_value, format_value_precise, MeasureKind, MeasureUnit, Scale,
    DEFAULT_PRECISION, MAX_PRECISION,
};
