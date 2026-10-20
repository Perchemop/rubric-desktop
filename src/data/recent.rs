//! App-level recent-projects persistence (config dir, JSON).
//!
//! Recents are application state, not project state: they are stored in a
//! small JSON file in the platform config directory — never inside a
//! `.rubric.json` project file, which is shareable and must not embed the
//! user's absolute local paths.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::domain::RecentList;

use super::DataError;

/// Path of the recent-projects config file, or `None` when no config
/// directory can be located. Mirrors [`super::store::project_dir`]'s
/// `ProjectDirs` pattern, using the config dir instead of the data dir.
pub fn recent_config_path() -> Option<PathBuf> {
    ProjectDirs::from("dev", "Rubric", "Rubric")
        .map(|pd| pd.config_dir().to_path_buf())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|h| h.join(".config").join("rubric"))
        })
        .map(|d| d.join("recent.json"))
}

/// Load the recent-projects list. Any error (no config dir, missing file,
/// unreadable or corrupt JSON) yields an empty list — recents must never
/// block startup or the menu.
pub fn load_recent() -> RecentList {
    match recent_config_path() {
        Some(p) => load_recent_from(p),
        None => RecentList::default(),
    }
}

/// Persist the recent-projects list to the app config file. Returns the
/// data-layer error when the config dir cannot be located or written.
pub fn save_recent(recent: &RecentList) -> Result<(), DataError> {
    match recent_config_path() {
        Some(p) => save_recent_to(p, recent),
        None => Err(DataError::Path(
            "no config directory available for recents".into(),
        )),
    }
}

/// Load recents from an explicit path: missing file or any parse failure
/// yields [`RecentList::default()`].
pub fn load_recent_from<P: AsRef<Path>>(path: P) -> RecentList {
    let Ok(bytes) = fs::read(path.as_ref()) else {
        return RecentList::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Save recents to an explicit path as pretty JSON, creating the parent dir.
/// Writes the same way [`super::store::save_project`] writes projects; the
/// payload is tiny, so a plain write is sufficient.
pub fn save_recent_to<P: AsRef<Path>>(path: P, recent: &RecentList) -> Result<(), DataError> {
    let p = path.as_ref();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::create(p)?;
    let json = serde_json::to_string_pretty(recent)?;
    f.write_all(json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_recent_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "rubric_recent_test_{}_{}_{:?}",
            std::process::id(),
            tag,
            std::thread::current().id()
        ))
    }

    #[test]
    fn load_recent_missing_file_yields_default() {
        assert_eq!(
            load_recent_from("/nonexistent/rubric/recent.json"),
            RecentList::default()
        );
    }

    #[test]
    fn load_recent_corrupt_file_yields_default() {
        let path = temp_recent_path("corrupt");
        std::fs::write(&path, b"not json").expect("write");
        assert_eq!(load_recent_from(&path), RecentList::default());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn recent_roundtrips_through_disk() {
        let path = temp_recent_path("roundtrip");
        let mut recent = RecentList::default();
        recent.push("first.rubric.json");
        recent.push("second.rubric.json");
        save_recent_to(&path, &recent).expect("save");
        let loaded = load_recent_from(&path);
        assert_eq!(loaded, recent);
        assert_eq!(
            loaded.as_slice(),
            &["second.rubric.json", "first.rubric.json"]
        );
        let _ = std::fs::remove_file(&path);
    }
}
