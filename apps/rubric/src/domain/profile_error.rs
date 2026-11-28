//! Profile-specific error types.
//!
//! Uses `thiserror` for ergonomic error handling. These errors are used
//! by both the domain layer (validation) and data layer (IO, migration).

use thiserror::Error;

/// Errors that can occur when working with profiles.
#[derive(Debug, Error)]
pub enum ProfileError {
    /// I/O error while reading or writing the profile file.
    #[error("Profile I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization error.
    #[error("Profile serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Profile validation failed.
    #[error("Profile validation error: {0}")]
    Validation(String),

    /// Profile not found.
    #[error("Profile not found: {0}")]
    NotFound(String),

    /// Profile name already exists.
    #[error("Profile already exists: {0}")]
    DuplicateName(String),

    /// Profile is read-only (predefined).
    #[error("Cannot modify predefined profile: {0}")]
    Predefined(String),

    /// Schema version is newer than supported.
    #[error(
        "Profile requires newer Rubric version (schema {found} > supported {supported})"
    )]
    UnsupportedSchema { found: u32, supported: u32 },

    /// Migration failed.
    #[error("Profile migration failed: {0}")]
    Migration(String),

    /// Invalid profile file path.
    #[error("Invalid profile path: {0}")]
    InvalidPath(String),

    /// Maximum profile count reached.
    #[error("Maximum {max} profiles reached ({current} already exist). Delete unused profiles to create more.")]
    ProfileLimit { max: usize, current: usize },
}

impl ProfileError {
    /// Create a validation error with a custom message.
    pub fn validation(msg: impl Into<String>) -> Self {
        ProfileError::Validation(msg.into())
    }

    /// Check if this error is recoverable (can continue with a different operation).
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ProfileError::Validation(_)
                | ProfileError::DuplicateName(_)
                | ProfileError::Predefined(_)
                | ProfileError::NotFound(_)
                | ProfileError::ProfileLimit { .. }
        )
    }
}