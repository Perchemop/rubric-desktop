//! Data persistence (project save/load, app data dir, PDF import, errors).

pub mod error;
pub mod export;
pub mod pdf;
pub mod prefs;
pub mod print;
pub mod recent;
pub mod render;
pub mod store;

pub use error::DataError;
pub use export::{write_report_json, write_report_text};
#[cfg(not(feature = "hayro"))]
pub use pdf::{bind_pdfium, PDFIUM_PATH_ENV};
pub use pdf::{read_pdf_info, render_cache_dir, render_page_to_cache, PdfSourceInfo};
pub use prefs::{load_prefs, prefs_config_path, save_prefs};
pub use print::{print_command, print_image};
pub use recent::{load_recent, recent_config_path, save_recent};
pub use render::rasterize_primitives;
pub(crate) use render::render_page_to_png_bytes;
pub use store::{
    autosave, autosave_due, default_project_path, load_project, normalise_project_filename,
    project_dir, save_project,
};
