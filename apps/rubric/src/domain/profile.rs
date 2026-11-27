//! Profile system: named UI layout configurations with versioned schema.
//!
//! Profiles capture the complete UI state (panel layout, toolbars, theme,
//! measurement defaults, canvas settings) so users can switch between
//! distinct workflows instantly (Review, Measurement, Markup, etc.).
//!
//! All types are pure, serde-serialisable, and testable headlessly.
//! No egui or filesystem access here.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use super::grid::Grid;
use super::theme::Theme;
use super::units::{MeasureUnit, Scale};

/// Current profile schema version.
///
/// Increment when a change makes an older build mis-read a newer profile file.
pub const CURRENT_PROFILE_SCHEMA_VERSION: u32 = 1;

/// Profile file extension.
pub const PROFILE_EXTENSION: &str = "rubric-profile";

/// Maximum number of profiles per user.
pub const MAX_PROFILES: usize = 50;

/// Warning threshold for profile count.
pub const PROFILE_WARNING_THRESHOLD: usize = 45;

/// Maximum name length.
pub const MAX_PROFILE_NAME_LENGTH: usize = 64;

/// Maximum description length.
pub const MAX_PROFILE_DESCRIPTION_LENGTH: usize = 256;

/// A complete UI profile capturing panel layout, toolbars, theme, and canvas state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    /// JSON schema version for migration tracking.
    pub schema_version: u32,

    /// Unique identifier for the profile.
    pub id: Uuid,

    /// User-visible name (max 64 chars).
    pub name: String,

    /// Optional description (max 256 chars).
    #[serde(default)]
    pub description: String,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp.
    pub modified_at: DateTime<Utc>,

    /// Whether this is a predefined profile (shipped with the app).
    #[serde(default)]
    pub is_predefined: bool,

    /// Panel visibility, positions, and docking state.
    pub panel_layout: crate::domain::panel::PanelLayout,

    /// Toolbar configuration and visibility.
    pub toolbar_layout: ToolbarLayout,

    /// Measurement defaults for new readings.
    pub measurement_defaults: MeasurementDefaults,

    /// Canvas-specific settings.
    pub canvas_settings: CanvasSettings,

    /// Theme override (None = follow app theme).
    #[serde(default)]
    pub theme_override: Option<Theme>,
}

impl Profile {
    /// Create a new profile with the given name and description.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            schema_version: CURRENT_PROFILE_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            created_at: now,
            modified_at: now,
            is_predefined: false,
            panel_layout: crate::domain::panel::PanelLayout::default(),
            toolbar_layout: ToolbarLayout::default(),
            measurement_defaults: MeasurementDefaults::default(),
            canvas_settings: CanvasSettings::default(),
            theme_override: None,
        }
    }

    /// Validate the profile.
    ///
    /// Returns an error string if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Profile name cannot be empty".into());
        }
        if self.name.chars().count() > MAX_PROFILE_NAME_LENGTH {
            return Err(format!(
                "Profile name exceeds {} characters",
                MAX_PROFILE_NAME_LENGTH
            ));
        }
        if self.description.chars().count() > MAX_PROFILE_DESCRIPTION_LENGTH {
            return Err(format!(
                "Description exceeds {} characters",
                MAX_PROFILE_DESCRIPTION_LENGTH
            ));
        }
        Ok(())
    }

    /// Sanitize a profile name for filesystem use (kebab-case).
    pub fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c
                } else if c.is_whitespace() {
                    '-'
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_lowercase()
    }

    /// Check if this is a predefined profile.
    pub fn is_predefined(&self) -> bool {
        self.is_predefined
    }
}

impl Default for Profile {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            schema_version: CURRENT_PROFILE_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            name: "Default".into(),
            description: "Default profile with balanced layout".into(),
            created_at: now,
            modified_at: now,
            is_predefined: false,
            panel_layout: crate::domain::panel::PanelLayout::default(),
            toolbar_layout: ToolbarLayout::default(),
            measurement_defaults: MeasurementDefaults::default(),
            canvas_settings: CanvasSettings::default(),
            theme_override: None,
        }
    }
}

/// Toolbar layout configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolbarLayout {
    /// Whether the main toolbar is visible.
    #[serde(default)]
    pub visible: bool,

    /// Order of tools in the toolbar (by name).
    #[serde(default)]
    pub tool_order: Vec<String>,

    /// Whether the toolbar is locked (user cannot reorder).
    #[serde(default)]
    pub locked: bool,

    /// Custom toolbar configurations.
    #[serde(default)]
    pub custom_toolbars: BTreeMap<String, CustomToolbar>,
}

impl Default for ToolbarLayout {
    fn default() -> Self {
        Self {
            visible: true,
            tool_order: Vec::new(), // Uses default from Tool enum
            locked: false,
            custom_toolbars: BTreeMap::new(),
        }
    }
}

/// A custom toolbar with specific tools.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomToolbar {
    /// Display name.
    pub name: String,

    /// Tools in this toolbar.
    pub tools: Vec<String>,

    /// Visibility state.
    pub visible: bool,
}

/// Measurement defaults for new readings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementDefaults {
    /// Default unit for new measurements.
    #[serde(default)]
    pub unit: MeasureUnit,

    /// Decimal precision (0-6).
    #[serde(default)]
    pub precision: u8,

    /// Active scale factor.
    #[serde(default)]
    pub scale: Option<Scale>,

    /// Grid settings.
    #[serde(default)]
    pub grid: Grid,

    /// Whether rulers are visible.
    #[serde(default)]
    pub rulers_visible: bool,
}

impl Default for MeasurementDefaults {
    fn default() -> Self {
        Self {
            unit: MeasureUnit::Point,
            precision: 3,
            scale: None, // Paper scale
            grid: Grid::default(),
            rulers_visible: true,
        }
    }
}

/// Canvas-specific settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasSettings {
    /// Current zoom factor.
    #[serde(default)]
    pub zoom: f32,

    /// Whether the ruler is visible.
    #[serde(default)]
    pub ruler_visible: bool,

    /// Whether guides are visible.
    #[serde(default)]
    pub guides_visible: bool,

    /// Background color (optional override as CSS color string).
    #[serde(default)]
    pub background_color: Option<String>,
}

impl Default for CanvasSettings {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            ruler_visible: false,
            guides_visible: false,
            background_color: None,
        }
    }
}

/// A summary of a profile for listing (name, type, modified date).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSummary {
    /// Profile name.
    pub name: String,

    /// Whether this is a predefined profile.
    #[serde(default)]
    pub predefined: bool,

    /// Last modified timestamp.
    #[serde(default)]
    pub modified_at: Option<DateTime<Utc>>,
}

impl ProfileSummary {
    /// Create a summary from a profile.
    pub fn from_profile(profile: &Profile) -> Self {
        Self {
            name: profile.name.clone(),
            predefined: profile.is_predefined,
            modified_at: Some(profile.modified_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_schema_version_is_1() {
        assert_eq!(CURRENT_PROFILE_SCHEMA_VERSION, 1);
    }

    #[test]
    fn profile_max_limits() {
        assert_eq!(MAX_PROFILES, 50);
        assert_eq!(MAX_PROFILE_NAME_LENGTH, 64);
        assert_eq!(MAX_PROFILE_DESCRIPTION_LENGTH, 256);
    }

    #[test]
    fn profile_roundtrip() {
        let profile = Profile {
            schema_version: CURRENT_PROFILE_SCHEMA_VERSION,
            id: Uuid::nil(),
            name: "Test Profile".into(),
            description: "A test profile".into(),
            created_at: DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            modified_at: DateTime::parse_from_rfc3339("2026-01-02T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            is_predefined: false,
            panel_layout: crate::domain::panel::PanelLayout::default(),
            toolbar_layout: ToolbarLayout::default(),
            measurement_defaults: MeasurementDefaults::default(),
            canvas_settings: CanvasSettings::default(),
            theme_override: Some(Theme::Dark),
        };

        let json = serde_json::to_string(&profile).expect("serialize");
        let back: Profile = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(back, profile);
    }

    #[test]
    fn profile_validator_rejects_empty_name() {
        let mut profile = Profile::default();
        profile.name = "   ".into();
        assert!(profile.validate().is_err());
    }

    #[test]
    fn profile_validator_rejects_long_name() {
        let mut profile = Profile::default();
        profile.name = "a".repeat(65);
        assert!(profile.validate().is_err());
    }

    #[test]
    fn profile_validator_accepts_valid_name() {
        let profile = Profile::default();
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn sanitize_filename_converts_to_kebab_case() {
        assert_eq!(Profile::sanitize_filename("My Profile"), "my-profile");
        assert_eq!(Profile::sanitize_filename("Test 123"), "test-123");
        assert_eq!(Profile::sanitize_filename("Hello World!"), "hello-world");
        assert_eq!(Profile::sanitize_filename("  Trim Me  "), "trim-me");
    }

    #[test]
    fn measurement_defaults_default_unit_is_point() {
        let defaults = MeasurementDefaults::default();
        assert_eq!(defaults.unit, MeasureUnit::Point);
        assert_eq!(defaults.precision, 3);
    }

    #[test]
    fn canvas_settings_default_zoom_is_one() {
        let settings = CanvasSettings::default();
        assert_eq!(settings.zoom, 1.0);
    }

    #[test]
    fn predefined_profile_flag_works() {
        let mut profile = Profile::default();
        assert!(!profile.is_predefined());
        profile.is_predefined = true;
        assert!(profile.is_predefined());
    }

    #[test]
    fn toolbar_layout_roundtrip() {
        let layout = ToolbarLayout {
            visible: true,
            tool_order: vec!["select".into(), "draw".into()],
            locked: false,
            custom_toolbars: BTreeMap::new(),
        };
        let json = serde_json::to_string(&layout).unwrap();
        let back: ToolbarLayout = serde_json::from_str(&json).unwrap();
        assert_eq!(back, layout);
    }

    #[test]
    fn measurement_defaults_roundtrip() {
        let defaults = MeasurementDefaults {
            unit: MeasureUnit::Inch,
            precision: 2,
            scale: Some(Scale::PAPER),
            grid: Grid::default(),
            rulers_visible: true,
        };
        let json = serde_json::to_string(&defaults).unwrap();
        let back: MeasurementDefaults = serde_json::from_str(&json).unwrap();
        assert_eq!(back, defaults);
    }

    #[test]
    fn canvas_settings_roundtrip() {
        let settings = CanvasSettings {
            zoom: 1.5,
            ruler_visible: true,
            guides_visible: false,
            background_color: Some("#ffffff".into()),
        };
        let json = serde_json::to_string(&settings).unwrap();
        let back: CanvasSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back, settings);
    }

    #[test]
    fn profile_summary_from_profile() {
        let profile = Profile::new("Test", "Description");
        let summary = ProfileSummary::from_profile(&profile);
        assert_eq!(summary.name, "Test");
        assert!(!summary.predefined);
        assert!(summary.modified_at.is_some());
    }
}
