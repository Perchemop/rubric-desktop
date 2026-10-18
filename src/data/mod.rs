//! Data persistence (project save/load, app data dir, errors).

pub mod error;
pub mod store;

pub use error::DataError;
pub use store::{default_project_path, load_project, project_dir, save_project};
