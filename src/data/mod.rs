//! Data persistence (project save/load, app data dir, PDF import, errors).

pub mod error;
pub mod pdf;
pub mod recent;
pub mod store;

pub use error::DataError;
pub use pdf::{
    bind_pdfium, read_pdf_info, render_cache_dir, render_page_to_cache, PdfSourceInfo,
    PDFIUM_PATH_ENV,
};
pub use recent::{load_recent, recent_config_path, save_recent};
pub use store::{
    default_project_path, load_project, normalise_project_filename, project_dir, save_project,
};
