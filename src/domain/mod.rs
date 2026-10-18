//! Domain module re-exports and common types.

pub mod document;
pub mod geom;
pub mod markup;
pub mod project;
pub mod style;
pub mod units;

pub use document::{Document, Page};
pub use geom::{angle_degrees, path_length, polygon_area, Pt, Rect};
pub use markup::{
    Comment, Markup, MarkupGeometry, MarkupKind, Measurement, ReviewStatus, ToolGroup,
};
pub use project::{History, Project, ProjectSnapshot, Settings, Tool, ToolSet};
pub use style::{palette, Color, DashPattern, FillMode, MarkupStyle};
pub use units::{MeasureKind, MeasureUnit, Scale};
