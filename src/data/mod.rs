//! Data persistence (project save/load, app data dir, PDF import, errors).

pub mod error;
pub mod export;
pub mod pdf;
pub mod print;
pub mod recent;
pub mod render;
pub mod store;

pub use error::DataError;
pub use export::{write_report_json, write_report_text};
pub use pdf::{
    bind_pdfium, read_pdf_info, render_cache_dir, render_page_to_cache, PdfSourceInfo,
    PDFIUM_PATH_ENV,
};
pub use print::{print_command, print_image};
pub use recent::{load_recent, recent_config_path, save_recent};
pub use render::{rasterize_primitives, render_page_to_png_bytes};
pub use store::{
    default_project_path, load_project, normalise_project_filename, project_dir, save_project,
};
