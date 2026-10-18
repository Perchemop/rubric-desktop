//! Domain module re-exports and common types.

pub mod geom;
pub mod units;
pub mod style;
pub mod markup;
pub mod document;
pub mod project;

pub use geom::{angle_degrees, path_length, polygon_area, Pt, Rect};
pub use units::{MeasureKind, MeasureUnit, Scale};
pub use style::{Color, DashPattern, FillMode, MarkupStyle, palette};
pub use markup::{Comment, Markup, MarkupGeometry, MarkupKind, Measurement, ReviewStatus, ToolGroup};
pub use document::{Document, Page};
pub use project::{History, Project, ProjectSnapshot, Settings, Tool, ToolSet};
