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

    /// The file was written by a schema version this build cannot read.
    ///
    /// Refusing up front beats loading a newer file as if it were the current
    /// schema and silently dropping fields.
    #[error(
        "project schema version {found} is not supported by this build (supported: 1..={supported})"
    )]
    UnsupportedSchema { found: u32, supported: u32 },

    // Profile-related errors
    #[error("Profile error: {0}")]
    ProfileError(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Profile '{0}' already exists")]
    ProfileDuplicate(String),

    #[error("Profile '{0}' is read-only")]
    ProfilePredefined(String),

    #[error("Maximum {max} profiles reached ({current} already exist)")]
    ProfileLimit { max: usize, current: usize },
}
