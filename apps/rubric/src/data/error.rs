//! Data-layer error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Invalid path: {0}")]
    Path(String),

    #[error("Project not found: {0}")]
    NotFound(String),

    #[error("PDF error: {0}")]
    Pdf(String),

    #[error("PDFium library error: {0}")]
    PdfLibrary(String),
}
