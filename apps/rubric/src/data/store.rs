//! Project storage on disk (JSON).

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use directories::ProjectDirs;

use crate::domain::{Project, CURRENT_SCHEMA_VERSION, MIN_AUTO_SAVE_INTERVAL_SECS};

use super::DataError;

/// Name appended to the project stem for its autosave file.
const AUTOSAVE_SUFFIX: &str = ".autosave.rubric.json";

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

/// Normalise a user-provided project file name so it always ends with the
/// canonical `.rubric.json` extension.
///
/// - `a.rubric.json` (any casing) is returned unchanged.
/// - a trailing `.json` (any casing) is *replaced* by `.rubric.json` rather
///   than stacked (`a.json` → `a.rubric.json`).
/// - anything else gets `.rubric.json` appended.
pub fn normalise_project_filename(name: String) -> String {
    // ASCII lowering preserves byte offsets and lengths, so suffix checks
    // against the lowered copy line up with `name` itself.
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".rubric.json") {
        name
    } else if lower.ends_with(".json") {
        // The matched suffix is ASCII regardless of original casing, so
        // slicing at `len - 5` is always on a char boundary.
        let stem = &name[..name.len() - ".json".len()];
        format!("{stem}.rubric.json")
    } else {
        format!("{name}.rubric.json")
    }
}

/// Ensure the project directory exists.
fn ensure_dir(p: &Path) -> Result<(), DataError> {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Save a project as JSON to `path`.
///
/// The write is atomic: the JSON is serialised first, written to a sibling
/// temporary file, and then renamed over the target. A crash or an error
/// mid-write therefore never leaves a truncated `.rubric.json` in place of the
/// previous good one (audit M3).
pub fn save_project<P: AsRef<Path>>(project: &Project, path: P) -> Result<PathBuf, DataError> {
    let p = path.as_ref().to_path_buf();
    ensure_dir(&p)?;
    let json = serde_json::to_string_pretty(project)?;

    let tmp = temp_sibling(&p);
    {
        let mut f = File::create(&tmp)?;
        f.write_all(json.as_bytes())?;
        f.sync_all()?;
    }
    if let Err(e) = fs::rename(&tmp, &p) {
        // Never leave the temporary file behind on a failed rename.
        let _ = fs::remove_file(&tmp);
        return Err(DataError::Io(e));
    }
    Ok(p)
}

/// A unique sibling path used by [`save_project`]'s atomic rename. It lives in
/// the target's directory so the rename stays on one filesystem (and is thus
/// atomic).
fn temp_sibling(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_string());
    path.with_file_name(format!(".{name}.{}.tmp", uuid::Uuid::new_v4()))
}

/// Load a project from JSON at `path`.
///
/// The on-disk `schema_version` is checked before the project is handed back:
/// a file this build cannot read is refused with
/// [`DataError::UnsupportedSchema`] rather than loaded as the current schema.
pub fn load_project<P: AsRef<Path>>(path: P) -> Result<Project, DataError> {
    let p = path.as_ref();
    let bytes = fs::read(p)?;
    let project: Project = serde_json::from_slice(&bytes).map_err(|e| {
        if e.is_io() {
            DataError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        } else {
            DataError::Serde(e)
        }
    })?;
    check_schema_version(project.schema_version)?;
    Ok(project)
}

/// Whether this build can read a project stamped with `version`.
///
/// Version 0 (explicitly written) and anything newer than
/// [`CURRENT_SCHEMA_VERSION`] are refused. This is the single seam a future
/// migration step hooks into.
pub fn check_schema_version(version: u32) -> Result<(), DataError> {
    if (1..=CURRENT_SCHEMA_VERSION).contains(&version) {
        Ok(())
    } else {
        Err(DataError::UnsupportedSchema {
            found: version,
            supported: CURRENT_SCHEMA_VERSION,
        })
    }
}

/// File name for a project's autosave: the project stem plus
/// `.autosave.rubric.json`, keeping the canonical extension so the result is a
/// normal `.rubric.json` file that File → Open can read.
fn autosave_file_name(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let lower = name.to_ascii_lowercase();
    let stem = if lower.ends_with(".rubric.json") {
        &name[..name.len() - ".rubric.json".len()]
    } else if lower.ends_with(".json") {
        &name[..name.len() - ".json".len()]
    } else {
        name.as_str()
    };
    let file = format!("{stem}{AUTOSAVE_SUFFIX}");
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(file),
        _ => PathBuf::from(file),
    }
}

/// Where an autosave for `project` should be written.
///
/// A session that knows its project path autosaves beside it so the recovery
/// file is easy to find; a pathless project (untitled, never saved) falls back
/// to the app data directory. `None` means no writable location is known.
pub fn autosave_path(project: &Project, current_path: Option<&Path>) -> Option<PathBuf> {
    let base = match current_path {
        Some(p) => p.to_path_buf(),
        None => default_project_path(&project.name)?,
    };
    Some(autosave_file_name(&base))
}

/// Write an autosave of `project`, returning the path written.
///
/// `Ok(None)` means no autosave location could be resolved (no project path and
/// no app data directory); callers should not treat that as dirty forever.
pub fn autosave(
    project: &Project,
    current_path: Option<&Path>,
) -> Result<Option<PathBuf>, DataError> {
    match autosave_path(project, current_path) {
        Some(p) => save_project(project, &p).map(Some),
        None => Ok(None),
    }
}

/// Whether a dirty, auto-save-enabled project is due for another autosave.
///
/// Pure so the schedule is testable without a real clock: `last` is the
/// previous attempt (`None` for a project never autosaved in this session) and
/// `now` is the caller's current instant. The interval is clamped up to
/// [`MIN_AUTO_SAVE_INTERVAL_SECS`] so a persisted zero cannot rewrite the file
/// every frame.
pub fn autosave_due(
    dirty: bool,
    auto_save: bool,
    interval_secs: u32,
    last: Option<Instant>,
    now: Instant,
) -> bool {
    if !dirty || !auto_save {
        return false;
    }
    let interval = Duration::from_secs(u64::from(interval_secs.max(MIN_AUTO_SAVE_INTERVAL_SECS)));
    match last {
        None => true,
        Some(t) => now.saturating_duration_since(t) >= interval,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Project, CURRENT_SCHEMA_VERSION};
    use std::path::PathBuf;

    fn temp_project_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "rubric_store_test_{}_{}_{:?}.rubric.json",
            std::process::id(),
            tag,
            std::thread::current().id()
        ))
    }

    /// Audit M3: `save_project` writes atomically — it must leave no temporary
    /// file behind and must still replace an existing project cleanly.
    #[test]
    fn save_project_is_atomic_and_leaves_no_temp_file() {
        let dir = std::env::temp_dir().join(format!(
            "rubric_atomic_save_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("proj.rubric.json");

        let saved = save_project(&Project::new("Atomic"), &path).expect("save");
        assert_eq!(saved, path);
        assert_eq!(load_project(&path).expect("load").name, "Atomic");

        let leftovers: Vec<String> = fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "temp files left behind: {leftovers:?}"
        );

        // Overwriting an existing project still succeeds (rename replaces).
        save_project(&Project::new("Atomic v2"), &path).expect("overwrite");
        assert_eq!(load_project(&path).expect("reload").name, "Atomic v2");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn normalise_appends_rubric_json() {
        assert_eq!(
            normalise_project_filename("my_project".into()),
            "my_project.rubric.json"
        );
    }

    #[test]
    fn normalise_replaces_plain_json_suffix_instead_of_stacking() {
        assert_eq!(normalise_project_filename("a.json".into()), "a.rubric.json");
        assert_eq!(normalise_project_filename("a.JSON".into()), "a.rubric.json");
    }

    #[test]
    fn normalise_keeps_existing_rubric_json_any_casing() {
        assert_eq!(
            normalise_project_filename("a.rubric.json".into()),
            "a.rubric.json"
        );
        assert_eq!(
            normalise_project_filename("A.RUBRIC.JSON".into()),
            "A.RUBRIC.JSON"
        );
    }

    #[test]
    fn autosave_stem_replaces_the_project_extension() {
        assert_eq!(
            autosave_file_name(std::path::Path::new("/tmp/site.rubric.json")),
            PathBuf::from("/tmp/site.autosave.rubric.json")
        );
        assert_eq!(
            autosave_file_name(std::path::Path::new("/tmp/site.JSON")),
            PathBuf::from("/tmp/site.autosave.rubric.json")
        );
        assert_eq!(
            autosave_file_name(std::path::Path::new("/tmp/plain")),
            PathBuf::from("/tmp/plain.autosave.rubric.json")
        );
    }

    #[test]
    fn autosave_path_prefers_a_sibling_of_the_open_project() {
        let project = Project::new("Site Plan");
        let path = autosave_path(
            &project,
            Some(std::path::Path::new("/tmp/site.rubric.json")),
        )
        .expect("path");
        assert_eq!(path, PathBuf::from("/tmp/site.autosave.rubric.json"));
    }

    #[test]
    fn autosave_writes_a_project_file_that_loads_back() {
        let project = Project::new("Recoverable");
        let target = temp_project_path("autosave_roundtrip");
        let written = autosave(&project, Some(&target))
            .expect("autosave")
            .expect("path");
        assert!(written.to_string_lossy().ends_with(".autosave.rubric.json"));
        let loaded = load_project(&written).expect("load autosave");
        assert_eq!(loaded, project);
        assert_eq!(loaded.schema_version, CURRENT_SCHEMA_VERSION);
        let _ = std::fs::remove_file(&written);
    }

    #[test]
    fn autosave_due_requires_dirty_and_enabled() {
        let now = Instant::now();
        assert!(autosave_due(true, true, 60, None, now));
        assert!(!autosave_due(false, true, 60, None, now));
        assert!(!autosave_due(true, false, 60, None, now));
    }

    #[test]
    fn autosave_due_respects_the_interval_and_clamps_tiny_values() {
        let start = Instant::now();
        let clamped = Duration::from_secs(u64::from(MIN_AUTO_SAVE_INTERVAL_SECS));
        assert!(!autosave_due(
            true,
            true,
            60,
            Some(start),
            start + Duration::from_secs(59)
        ));
        assert!(autosave_due(
            true,
            true,
            60,
            Some(start),
            start + Duration::from_secs(60)
        ));
        // A persisted 0 must not autosave every frame; the clamp makes it wait.
        assert!(!autosave_due(
            true,
            true,
            0,
            Some(start),
            start + Duration::from_secs(1)
        ));
        assert!(autosave_due(true, true, 0, Some(start), start + clamped));
    }

    #[test]
    fn schema_versions_outside_the_supported_range_are_refused() {
        assert!(check_schema_version(1).is_ok());
        assert!(check_schema_version(CURRENT_SCHEMA_VERSION).is_ok());
        assert!(check_schema_version(0).is_err());
        assert!(check_schema_version(CURRENT_SCHEMA_VERSION + 1).is_err());
    }

    #[test]
    fn load_project_refuses_a_newer_schema_version() {
        let path = temp_project_path("newer");
        let mut project = Project::new("Future");
        project.schema_version = CURRENT_SCHEMA_VERSION + 1;
        save_project(&project, &path).expect("save");
        match load_project(&path) {
            Err(DataError::UnsupportedSchema { found, supported }) => {
                assert_eq!(found, CURRENT_SCHEMA_VERSION + 1);
                assert_eq!(supported, CURRENT_SCHEMA_VERSION);
            }
            other => panic!("expected UnsupportedSchema, got {other:?}"),
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_project_treats_a_file_without_schema_version_as_current() {
        let path = temp_project_path("legacy");
        let mut value = serde_json::to_value(Project::new("Legacy")).expect("json");
        value
            .as_object_mut()
            .expect("object")
            .remove("schema_version");
        std::fs::write(&path, serde_json::to_vec_pretty(&value).expect("bytes")).expect("write");
        let loaded = load_project(&path).expect("load");
        assert_eq!(loaded.schema_version, CURRENT_SCHEMA_VERSION);
        let _ = std::fs::remove_file(&path);
    }
}
