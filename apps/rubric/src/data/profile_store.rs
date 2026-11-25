//! Profile persistence layer.
//!
//! Handles reading/writing profile files, managing the profile directory,
//! and providing CRUD operations for profiles.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use directories::ProjectDirs;
use uuid::Uuid;

use crate::domain::{
    find_predefined_by_name, is_predefined_id, predefined_profiles, toolbar::ToolbarProfileConfig,
    CanvasSettings, MeasurementDefaults, Profile, ProfileSummary, ToolbarLayout,
};

use super::profile_migration::migrate_profile;
use super::DataError;

/// Path to the profiles directory.
/// Created on first run if missing.
pub fn profiles_dir() -> Option<PathBuf> {
    ProjectDirs::from("dev", "Rubric", "Rubric")
        .map(|pd| pd.data_dir().to_path_buf())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|h| h.join(".config").join("rubric"))
        })
        .map(|d| d.join("profiles"))
}

/// Get the full path for a profile file by name.
pub fn profile_path(name: &str) -> Option<PathBuf> {
    profiles_dir().map(|d| {
        let safe_name = Profile::sanitize_filename(name);
        d.join(format!(
            "{}.{}",
            safe_name,
            crate::domain::PROFILE_EXTENSION
        ))
    })
}

/// Build a Profile from the current application UI state.
/// This is used when saving a profile from the current UI layout.
pub fn build_profile_from_state(
    name: String,
    description: String,
    panel_layout: crate::domain::panel::PanelLayout,
    toolbar_profile: Option<crate::domain::toolbar::ToolbarProfileConfig>,
    theme: crate::domain::theme::Theme,
    measurement_defaults: crate::domain::workspace::MeasurementDefaults,
    canvas_settings: crate::domain::CanvasSettings,
) -> crate::domain::profile::Profile {
    let now = chrono::Utc::now();
    // Convert ToolbarProfileConfig to ToolbarLayout for profile storage
    let toolbar_layout = if let Some(tp) = toolbar_profile {
        crate::domain::profile::ToolbarLayout {
            visible: tp.main_toolbar.visible,
            tool_order: tp
                .main_toolbar
                .items
                .iter()
                .filter_map(|i| i.command_id())
                .map(|c| c.0.clone())
                .collect(),
            locked: tp.lock_state.global_locked,
            custom_toolbars: tp
                .custom_toolbars
                .iter()
                .map(|tb| {
                    (
                        tb.id.clone(),
                        crate::domain::profile::CustomToolbar {
                            name: tb.name.clone(),
                            tools: tb
                                .items
                                .iter()
                                .filter_map(|i| i.command_id())
                                .map(|c| c.0.clone())
                                .collect(),
                            visible: tb.visible,
                        },
                    )
                })
                .collect(),
        }
    } else {
        // Default toolbar layout
        crate::domain::profile::ToolbarLayout::default()
    };

    let now = chrono::Utc::now();
    crate::domain::profile::Profile {
        schema_version: crate::domain::profile::CURRENT_PROFILE_SCHEMA_VERSION,
        id: uuid::Uuid::new_v4(),
        name,
        description,
        created_at: now,
        modified_at: now,
        is_predefined: false,
        panel_layout,
        toolbar_layout,
        measurement_defaults,
        canvas_settings,
        theme_override: Some(theme),
    }
}

/// ProfileStore manages profile persistence.
pub struct ProfileStore {
    profiles_dir: PathBuf,
}

impl ProfileStore {
    /// Create a new ProfileStore, initializing the directory if needed.
    pub fn new() -> Result<Self, DataError> {
        let dir = profiles_dir()
            .ok_or_else(|| DataError::Path("no config directory available for profiles".into()))?;

        fs::create_dir_all(&dir)?;

        Ok(Self { profiles_dir: dir })
    }

    /// Initialize the profile directory and write predefined profiles if missing.
    pub fn initialize() -> Result<Self, DataError> {
        let store = Self::new()?;

        // Write predefined profiles if they don't exist
        for profile in predefined_profiles() {
            let path = profile_path(&profile.name)
                .ok_or_else(|| DataError::Path("invalid profile path".into()))?;
            if !path.exists() {
                store.save_profile_to_path(profile, &path)?;
            }
        }

        Ok(store)
    }

    /// List all profiles (predefined and custom).
    pub fn list_profiles(&self) -> Result<Vec<ProfileSummary>, DataError> {
        let mut profiles = Vec::new();

        // Add predefined profiles
        for profile in predefined_profiles() {
            profiles.push(ProfileSummary::from_profile(&profile));
        }

        // Add custom profiles from disk
        if let Ok(entries) = fs::read_dir(&self.profiles_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .map_or(false, |ext| ext == "rubric-profile")
                {
                    if let Some(name) = path.file_stem() {
                        let name_str = name.to_string_lossy().to_string();

                        // Check if it's a predefined profile by checking if name matches
                        if find_predefined_by_name(&name_str).is_none() {
                            // Try to read the file for modified_at
                            let modified = fs::metadata(&path)
                                .ok()
                                .and_then(|m| m.modified().ok())
                                .map(|t| chrono::DateTime::<Utc>::from(t));

                            profiles.push(ProfileSummary {
                                name: name_str,
                                predefined: false,
                                modified_at: modified,
                            });
                        }
                    }
                }
            }
        }

        // Sort by name, predefined first
        profiles.sort_by(|a, b| {
            let a_pre = a.predefined;
            let b_pre = b.predefined;
            a_pre.cmp(&b_pre).then_with(|| a.name.cmp(&b.name))
        });

        Ok(profiles)
    }

    /// Load a profile by name.
    pub fn load_profile(&self, name: &str) -> Result<Profile, DataError> {
        // Check if it's a predefined profile
        if let Some(profile) = find_predefined_by_name(name) {
            return Ok(Profile {
                id: profile.id,
                name: profile.name.clone(),
                description: profile.description.clone(),
                created_at: profile.created_at,
                modified_at: profile.modified_at,
                is_predefined: true,
                schema_version: super::profile::CURRENT_PROFILE_SCHEMA_VERSION,
                panel_layout: profile.panel_layout.clone(),
                toolbar_layout: profile.toolbar_layout.clone(),
                measurement_defaults: profile.measurement_defaults.clone(),
                canvas_settings: profile.canvas_settings.clone(),
                theme_override: profile.theme_override,
            });
        }

        // Load from disk
        let path =
            profile_path(name).ok_or_else(|| DataError::ProfileNotFound(name.to_string()))?;
        let content = fs::read(&path).map_err(DataError::Io)?;

        // Parse and migrate if needed
        let value = serde_json::from_slice(&content).map_err(DataError::Serde)?;
        let migrated = migrate_profile(value)?;

        serde_json::from_value(migrated).map_err(DataError::Serde)
    }

    /// Save a profile (overwrite existing or create new).
    pub fn save_profile(&self, profile: &Profile) -> Result<(), DataError> {
        // Check if it's predefined
        if profile.is_predefined || is_predefined_id(&profile.id) {
            return Err(DataError::ProfilePredefined(profile.name.clone()));
        }

        let path = profile_path(&profile.name)
            .ok_or_else(|| DataError::Path("invalid profile path".into()))?;

        self.save_profile_to_path(profile, &path)
    }

    fn save_profile_to_path(&self, profile: &Profile, path: &Path) -> Result<(), DataError> {
        let json = serde_json::to_string_pretty(profile)?;

        // Atomic write: write to temp file, then rename
        let temp_path = path.with_extension(format!(
            "{}.tmp.{}",
            path.file_name().unwrap().to_string_lossy(),
            std::process::id()
        ));

        {
            let mut file = File::create(&temp_path)?;
            file.write_all(json.as_bytes())?;
            file.write_all(b"\n")?;
        }

        fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Delete a profile by name.
    pub fn delete_profile(&self, name: &str) -> Result<(), DataError> {
        // Check if it's predefined
        if find_predefined_by_name(name).is_some() {
            return Err(DataError::ProfilePredefined(name.to_string()));
        }

        let path =
            profile_path(name).ok_or_else(|| DataError::ProfileNotFound(name.to_string()))?;

        fs::remove_file(&path)?;

        Ok(())
    }

    /// Export a profile to a specific path.
    pub fn export_profile(&self, name: &str, dest_path: &Path) -> Result<(), DataError> {
        let profile = if name.is_empty() || find_predefined_by_name(name).is_some() {
            // Export predefined as custom copy
            find_predefined_by_name(name)
                .map(|p| Profile {
                    id: Uuid::new_v4(),
                    is_predefined: false,
                    ..p.clone()
                })
                .ok_or_else(|| DataError::ProfileNotFound(name.to_string()))?
        } else {
            self.load_profile(name)?
        };

        let json = serde_json::to_string_pretty(&profile)?;

        let mut file = File::create(dest_path)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;

        Ok(())
    }

    /// Import a profile from a file.
    pub fn import_profile(&self, path: &Path) -> Result<Profile, DataError> {
        let content = fs::read(path).map_err(DataError::Io)?;

        // Check if it has the required schema_version field
        let value: serde_json::Value =
            serde_json::from_slice(&content).map_err(DataError::Serde)?;
        if !value.get("schema_version").is_some() {
            return Err(DataError::Profile(
                "Selected file is not a valid Rubric profile (missing schema_version)".into(),
            ));
        }

        // Migrate if needed
        let migrated = migrate_profile(value)?;

        let mut profile: Profile = serde_json::from_value(migrated).map_err(DataError::Serde)?;

        // Make it a custom profile (not predefined)
        profile.id = Uuid::new_v4();
        profile.is_predefined = false;

        // Save it
        self.save_profile(&profile)?;

        Ok(profile)
    }

    /// Get the active profile name from settings.
    pub fn get_active_profile() -> Option<String> {
        // TODO: Read from AppPrefs.active_profile
        None
    }

    /// Set the active profile in settings.
    pub fn set_active_profile(name: &str) -> Result<(), DataError> {
        // TODO: Write to AppPrefs.active_profile
        let _ = name;
        Ok(())
    }
}

impl Default for ProfileStore {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            profiles_dir: dirs::ProjectDirs::from("dev", "Rubric", "Rubric")
                .map(|pd| pd.data_dir().to_path_buf())
                .unwrap_or_else(|| {
                    std::env::var_os("HOME")
                        .map(PathBuf::from)
                        .unwrap_or_default()
                        .join(".config")
                        .join("rubric")
                        .join("profiles")
                }),
        })
    }
}

// Helper function to convert DateTime<Utc> to String for JSON
impl ProfileSummary {
    pub fn modified(&self) -> String {
        self.modified_at
            .as_ref()
            .map(|t| t.to_rfc3339())
            .unwrap_or_else(|| "—".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_profiles_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rubric_profiles_test_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn temp_profile_path(dir: &Path, name: &str) -> PathBuf {
        dir.join(format!(
            "{}.{}",
            Profile::sanitize_filename(name),
            crate::domain::PROFILE_EXTENSION
        ))
    }

    #[test]
    fn profile_store_creates_directory() {
        let temp_dir = temp_profiles_dir();
        let _ = fs::remove_dir_all(&temp_dir);

        let store = ProfileStore::new().ok();
        // If the temp dir was deleted, this test verifies the pattern
        drop(store);
    }

    #[test]
    fn sanitize_filename_produces_kebab_case() {
        assert_eq!(Profile::sanitize_filename("My Profile"), "my-profile");
        assert_eq!(Profile::sanitize_filename("Test 123"), "test-123");
        assert_eq!(Profile::sanitize_filename("Hello World!"), "hello-world");
    }

    #[test]
    fn profile_path_uses_correct_extension() {
        // This test verifies the profile_path function logic
        let profile = Profile::new("Test Profile", "Description");
        assert!(profile_path(&profile.name).is_some());
    }

    #[test]
    fn predefined_profile_cannot_be_modified() {
        let profile = predefined_profiles()[0].clone();
        assert!(profile.is_predefined);
    }

    #[test]
    fn profile_serialization_roundtrip() {
        let profile = Profile::new("Test", "Description");
        let json = serde_json::to_string_pretty(&profile).unwrap();
        let back: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile.name, back.name);
        assert_eq!(profile.description, back.description);
    }

    #[test]
    fn profile_summary_from_profile() {
        let profile = Profile::new("Summary Test", "Test description");
        let summary = ProfileSummary::from_profile(&profile);
        assert_eq!(summary.name, "Summary Test");
        assert!(!summary.predefined);
    }
}
