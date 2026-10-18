//! Data persistence (project save/load, app data dir, errors).

pub mod error;
pub mod store;

pub use error::DataError;
pub use store::{load_project, save_project, project_dir, default_project_path};
