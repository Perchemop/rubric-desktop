//! Domain module re-exports and common types.

pub mod document;
pub mod geom;
pub mod markup;
pub mod project;
pub mod report;
pub mod style;
pub mod theme;
pub mod units;

pub use document::{Document, Layer, LayerError, Page};
pub use geom::{angle_degrees, path_length, polygon_area, Pt, Rect};
pub use markup::{
    Comment, Markup, MarkupGeometry, MarkupKind, Measurement, ReviewStatus, StampPreset, ToolGroup,
};
pub use project::{
    History, Project, ProjectSnapshot, RecentList, Settings, Tool, ToolSet, MAX_RECENT,
};
pub use report::{Report, ReportEntry, ReportGroupKey, ReportSection, StatusCounts, NONE_LABEL};
pub use style::{palette, Color, DashPattern, FillMode, MarkupStyle};
pub use theme::{AppPrefs, Theme};
pub use units::{
    clamp_precision, format_value, format_value_precise, MeasureKind, MeasureUnit, Scale,
    DEFAULT_PRECISION, MAX_PRECISION,
};
