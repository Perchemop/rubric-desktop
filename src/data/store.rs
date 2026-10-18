//! Project storage on disk (JSON).

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::domain::Project;

use super::DataError;

/// Return the platform-appropriate app data directory for Rubric.
pub fn project_dir() -> Option<PathBuf> {
    ProjectDirs::from("dev", "Rubric", "Rubric")
        .map(|pd| pd.data_dir().to_path_buf())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|h| h.join(".config").join("rubric"))
        })
}

/// Default path for an untitled project save.
pub fn default_project_path(name: &str) -> Option<PathBuf> {
    let safe = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(*c, '-' | '_' | ' '))
        .collect::<String>()
        .trim()
        .replace(' ', "_");
    let safe = if safe.is_empty() {
        "untitled".into()
    } else {
        safe
    };
    project_dir().map(|d| d.join(format!("{safe}.rubric.json")))
}

/// Ensure the project directory exists.
fn ensure_dir(p: &Path) -> Result<(), DataError> {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Save a project as JSON to `path`.
pub fn save_project<P: AsRef<Path>>(project: &Project, path: P) -> Result<PathBuf, DataError> {
    let p = path.as_ref().to_path_buf();
    ensure_dir(&p)?;
    let mut f = File::create(&p)?;
    let json = serde_json::to_string_pretty(project)?;
    f.write_all(json.as_bytes())?;
    Ok(p)
}

/// Load a project from JSON at `path`.
pub fn load_project<P: AsRef<Path>>(path: P) -> Result<Project, DataError> {
    let p = path.as_ref();
    let bytes = fs::read(p)?;
    serde_json::from_slice(&bytes).map_err(|e| {
        if e.is_io() {
            DataError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        } else {
            DataError::Serde(e)
        }
    })
}

/// Autosave stub (will be wired into the UI loop later).
#[allow(dead_code)]
pub fn autosave(project: &Project) -> Result<Option<PathBuf>, DataError> {
    if let Some(mut p) = default_project_path(&project.name) {
        p.set_extension("autosave.rubric.json");
        save_project(project, &p).map(Some)
    } else {
        Ok(None)
    }
}
