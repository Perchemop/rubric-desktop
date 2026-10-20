//! Data persistence (project save/load, app data dir, errors).

pub mod error;
pub mod recent;
pub mod store;

pub use error::DataError;
pub use recent::{load_recent, recent_config_path, save_recent};
pub use store::{
    default_project_path, load_project, normalise_project_filename, project_dir, save_project,
};
