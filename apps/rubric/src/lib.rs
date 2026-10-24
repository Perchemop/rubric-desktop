//! Library entry point.

pub mod data;
pub mod domain;
pub mod ui;

pub use data::{load_project, save_project};
pub use domain::{Project, ProjectSnapshot};
pub use ui::RubricApp;
