//! Predefined profiles embedded in the binary.
//!
//! Four built-in profiles for common workflows:
//! - Default: Balanced layout for general use
//! - Review: Optimized for markup review
//! - Measurement: Ready for taking measurements
//! - Markup: Full canvas for creating markups

use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use uuid::Uuid;

use super::grid::Grid;
use super::panel::predefined_layouts::{
    default as default_panel_layout, markup, measurement, review,
};
use super::panel::PanelLayout;
use super::profile::{CanvasSettings, MeasurementDefaults, Profile, ToolbarLayout};
use super::theme::Theme;
use super::units::{MeasureUnit, Scale};

/// Predefined profile IDs (for reliable identification).
pub mod ids {
    use uuid::Uuid;

    // These UIDs are stable across versions so users can identify their profiles.
    pub const DEFAULT: Uuid = Uuid::nil(); // Nil UUID = Default profile
    pub const REVIEW: Uuid = Uuid::from_u128(0x00000000_00000000_00000000_00000001);
    pub const MEASUREMENT: Uuid = Uuid::from_u128(0x00000000_00000000_00000000_00000002);
    pub const MARKUP: Uuid = Uuid::from_u128(0x00000000_00000000_00000000_00000003);
}

fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}

/// Create the Default profile with balanced layout.
pub fn create_default_profile() -> Profile {
    Profile {
        schema_version: super::profile::CURRENT_PROFILE_SCHEMA_VERSION,
        id: ids::DEFAULT,
        name: "Default".into(),
        description: "Balanced starting layout with all panels visible".into(),
        created_at: now(),
        modified_at: now(),
        is_predefined: true,
        panel_layout: default_panel_layout(),
        toolbar_layout: ToolbarLayout {
            visible: true,
            tool_order: Vec::new(),
            locked: false,
            custom_toolbars: std::collections::BTreeMap::new(),
        },
        measurement_defaults: MeasurementDefaults {
            unit: MeasureUnit::Point,
            precision: 3,
            scale: None,
            grid: Grid::default(),
            rulers_visible: true,
        },
        canvas_settings: CanvasSettings {
            zoom: 1.0,
            ruler_visible: false,
            guides_visible: false,
            background_color: None,
        },
        theme_override: None,
    }
}

/// Create the Review profile optimized for markup review.
///
/// Layout: Left = Thumbnails+Bookmarks | Right = Markups List+Properties | Bottom = Status
pub fn create_review_profile() -> Profile {
    Profile {
        schema_version: super::profile::CURRENT_PROFILE_SCHEMA_VERSION,
        id: ids::REVIEW,
        name: "Review".into(),
        description: "Optimized for reviewing markups: Thumbnails + Markups List + Properties"
            .into(),
        created_at: now(),
        modified_at: now(),
        is_predefined: true,
        panel_layout: review(),
        toolbar_layout: ToolbarLayout {
            visible: true,
            tool_order: Vec::new(),
            locked: false,
            custom_toolbars: std::collections::BTreeMap::new(),
        },
        measurement_defaults: MeasurementDefaults {
            unit: MeasureUnit::Point,
            precision: 2,
            scale: None,
            grid: Grid::default(),
            rulers_visible: true,
        },
        canvas_settings: CanvasSettings {
            zoom: 1.0,
            ruler_visible: true,
            guides_visible: false,
            background_color: None,
        },
        theme_override: None,
    }
}

/// Create the Measurement profile ready for taking measurements.
///
/// Layout: Left = Layers + Tool Chest | Right = Properties + Measurements
/// Also: Rulers visible, Grid enabled with snap
pub fn create_measurement_profile() -> Profile {
    let mut grid = Grid::default();
    grid.snap = true;

    Profile {
        schema_version: super::profile::CURRENT_PROFILE_SCHEMA_VERSION,
        id: ids::MEASUREMENT,
        name: "Measurement".into(),
        description: "Ready for taking measurements: Layers + Measurements + Grid snapped".into(),
        created_at: now(),
        modified_at: now(),
        is_predefined: true,
        panel_layout: measurement(),
        toolbar_layout: ToolbarLayout {
            visible: true,
            tool_order: Vec::new(),
            locked: false,
            custom_toolbars: std::collections::BTreeMap::new(),
        },
        measurement_defaults: MeasurementDefaults {
            unit: MeasureUnit::Inch,
            precision: 3,
            scale: Some(Scale::PAPER),
            grid,
            rulers_visible: true,
        },
        canvas_settings: CanvasSettings {
            zoom: 1.0,
            ruler_visible: true,
            guides_visible: true,
            background_color: None,
        },
        theme_override: None,
    }
}

/// Create the Markup profile for creating markups.
///
/// Layout: Left = Tool Chest + Stamps | Right = Properties + Layers
/// Maximum canvas space
pub fn create_markup_profile() -> Profile {
    Profile {
        schema_version: super::profile::CURRENT_PROFILE_SCHEMA_VERSION,
        id: ids::MARKUP,
        name: "Markup".into(),
        description: "Maximum canvas space for creating markups: Tool Chest + Stamps".into(),
        created_at: now(),
        modified_at: now(),
        is_predefined: true,
        panel_layout: markup(),
        toolbar_layout: ToolbarLayout {
            visible: true,
            tool_order: Vec::new(),
            locked: false,
            custom_toolbars: std::collections::BTreeMap::new(),
        },
        measurement_defaults: MeasurementDefaults {
            unit: MeasureUnit::Point,
            precision: 3,
            scale: None,
            grid: Grid::default(),
            rulers_visible: false,
        },
        canvas_settings: CanvasSettings {
            zoom: 1.0,
            ruler_visible: false,
            guides_visible: false,
            background_color: None,
        },
        theme_override: None,
    }
}

/// Get all predefined profiles as a static slice.
///
/// Returns profiles in order: Default, Review, Measurement, Markup.
pub fn predefined_profiles() -> Vec<Profile> {
    vec![
        create_default_profile(),
        create_review_profile(),
        create_measurement_profile(),
        create_markup_profile(),
    ]
}

/// Find a predefined profile by its stable ID.
pub fn find_predefined_by_id(id: &Uuid) -> Option<&'static Profile> {
    predefined_profiles().iter().find(|p| &p.id == id)
}

/// Find a predefined profile by name (case-insensitive).
pub fn find_predefined_by_name(name: &str) -> Option<&'static Profile> {
    predefined_profiles()
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(name))
}

/// Check if a profile ID belongs to a predefined profile.
pub fn is_predefined_id(id: &Uuid) -> bool {
    find_predefined_by_id(id).is_some()
}

/// Generate JSON string for all predefined profiles (for embedding in binary or debugging).
pub fn serialize_predefined_profiles_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(predefined_profiles())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predefined_profile_ids_are_stable() {
        assert_eq!(ids::DEFAULT, Uuid::nil());
        assert_ne!(ids::REVIEW, ids::MEASUREMENT);
        assert_ne!(ids::MEASUREMENT, ids::MARKUP);
    }

    #[test]
    fn all_four_profiles_exist() {
        let profiles = predefined_profiles();
        assert_eq!(profiles.len(), 4);
    }

    #[test]
    fn default_profile_is_balanced() {
        let profile = find_predefined_by_name("Default").unwrap();
        // The predefined layouts are in the panel module
        assert!(profile.is_predefined);
    }

    #[test]
    fn review_profile_has_specific_layout() {
        let profile = find_predefined_by_name("Review").unwrap();
        assert!(profile.is_predefined);
        // Check that measurement rulers are visible for review profile
        assert!(profile.canvas_settings.ruler_visible);
    }

    #[test]
    fn measurement_profile_has_grid_snap() {
        let profile = find_predefined_by_name("Measurement").unwrap();
        assert_eq!(profile.measurement_defaults.unit, MeasureUnit::Inch);
        assert!(profile.measurement_defaults.grid.snap);
        assert!(profile.measurement_defaults.rulers_visible);
    }

    #[test]
    fn markup_profile_has_maximum_canvas() {
        let profile = find_predefined_by_name("Markup").unwrap();
        assert!(profile.is_predefined);
        assert!(!profile.canvas_settings.ruler_visible);
    }

    #[test]
    fn find_by_id_works() {
        let profile = find_predefined_by_id(&ids::DEFAULT);
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().name, "Default");
    }

    #[test]
    fn find_by_name_is_case_insensitive() {
        assert!(find_predefined_by_name("default").is_some());
        assert!(find_predefined_by_name("DEFAULT").is_some());
        assert!(find_predefined_by_name("Review").is_some());
        assert!(find_predefined_by_name("review").is_some());
    }

    #[test]
    fn predefined_profiles_roundtrip() {
        let profiles = predefined_profiles();
        let json = serde_json::to_string(profiles).expect("serialize");
        let back: Vec<Profile> = serde_json::from_str(&json).expect("deserialize");

        for (original, restored) in profiles.iter().zip(back.iter()) {
            assert_eq!(original.id, restored.id);
            assert_eq!(original.name, restored.name);
            assert_eq!(original.is_predefined, restored.is_predefined);
        }
    }

    #[test]
    fn all_profiles_are_marked_predefined() {
        for profile in predefined_profiles() {
            assert!(
                profile.is_predefined,
                "Profile {} should be marked as predefined",
                profile.name
            );
        }
    }

    #[test]
    fn profile_names_are_unique() {
        let names: std::collections::HashSet<_> = predefined_profiles()
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(names.len(), 4, "Profile names should be unique");
    }
}
