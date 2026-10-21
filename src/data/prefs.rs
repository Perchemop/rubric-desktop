//! App-level preference persistence (config dir, JSON).
//!
//! Preferences such as the colour theme belong to the application, not to a
//! project: they are stored in a small JSON file inside the platform config
//! directory — never inside a `.rubric.json` project file, which is
//! shareable and must not embed machine-local choices.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::domain::AppPrefs;

use super::DataError;

/// Path of the preferences config file, or `None` when no config directory
/// can be located. Mirrors [`super::recent::recent_config_path`]'s
/// `ProjectDirs` pattern, using the config dir.
pub fn prefs_config_path() -> Option<PathBuf> {
    ProjectDirs::from("dev", "Rubric", "Rubric")
        .map(|pd| pd.config_dir().to_path_buf())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|h| h.join(".config").join("rubric"))
        })
        .map(|d| d.join("prefs.json"))
}

/// Load the application preferences. Any error (no config dir, missing file,
/// unreadable or corrupt JSON) yields the defaults — preferences must never
/// block startup.
pub fn load_prefs() -> AppPrefs {
    match prefs_config_path() {
        Some(p) => load_prefs_from(p),
        None => AppPrefs::default(),
    }
}

/// Persist the application preferences to the app config file.
pub fn save_prefs(prefs: &AppPrefs) -> Result<(), DataError> {
    match prefs_config_path() {
        Some(p) => save_prefs_to(p, prefs),
        None => Err(DataError::Path(
            "no config directory available for preferences".into(),
        )),
    }
}

/// Load preferences from an explicit path: missing file or any parse failure
/// yields [`AppPrefs::default`].
pub fn load_prefs_from<P: AsRef<Path>>(path: P) -> AppPrefs {
    let Ok(bytes) = fs::read(path.as_ref()) else {
        return AppPrefs::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Save preferences to an explicit path as pretty JSON, creating the parent
/// directory when needed.
pub fn save_prefs_to<P: AsRef<Path>>(path: P, prefs: &AppPrefs) -> Result<(), DataError> {
    let p = path.as_ref();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::create(p)?;
    let json = serde_json::to_string_pretty(prefs)?;
    f.write_all(json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Theme;

    fn temp_prefs_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "rubric_prefs_test_{}_{}_{:?}",
            std::process::id(),
            tag,
            std::thread::current().id()
        ))
    }

    #[test]
    fn load_prefs_missing_file_yields_default() {
        assert_eq!(
            load_prefs_from("/nonexistent/rubric/prefs.json"),
            AppPrefs::default()
        );
    }

    #[test]
    fn load_prefs_corrupt_file_yields_default() {
        let path = temp_prefs_path("corrupt");
        std::fs::write(&path, b"not json").expect("write");
        assert_eq!(load_prefs_from(&path), AppPrefs::default());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn prefs_roundtrip_through_disk() {
        let path = temp_prefs_path("roundtrip");
        let prefs = AppPrefs { theme: Theme::Dark };
        save_prefs_to(&path, &prefs).expect("save");
        assert_eq!(load_prefs_from(&path), prefs);
        let raw = std::fs::read_to_string(&path).expect("read");
        assert!(raw.contains("dark"), "theme must be persisted: {raw}");
        let _ = std::fs::remove_file(&path);
    }
}
