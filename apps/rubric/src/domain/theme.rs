//! Theme preference (light/dark).
//!
//! The theme is an **app-level** preference (like the recent-files list), not
//! project data: it lives in the platform config directory and is never
//! written into a shareable `.rubric.json` project file.

use serde::{Deserialize, Serialize};

use crate::domain::style::DefaultStyles;
use crate::domain::style_preset::StylePresetLibrary;
use crate::domain::MeasureUnit;

/// Colour theme used by the application shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Light surfaces with dark text.
    #[default]
    Light,
    /// Dark surfaces with light text.
    Dark,
}

impl Theme {
    /// Every theme, in menu order.
    pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];

    /// Human-readable label for menus.
    pub fn label(self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }

    /// Whether this is the dark theme.
    pub fn is_dark(self) -> bool {
        matches!(self, Theme::Dark)
    }
}

/// [`MeasureUnit`] has no inherent default in `units.rs`; the app default is
/// the paper-space unit (`Point`), matching [`Scale::PAPER`].
impl Default for MeasureUnit {
    fn default() -> Self {
        MeasureUnit::Point
    }
}

/// Application-level preferences stored outside project files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppPrefs {
    /// Selected colour theme. Missing values fall back to
    /// [`Theme::Light`] so older preference files keep loading.
    #[serde(default)]
    pub theme: Theme,
    /// Show the Markups List panel. Missing values fall back to `true`.
    #[serde(default = "default_true")]
    pub show_markups_list: bool,
    /// Show the Tool Chest panel. Missing values fall back to `true`.
    #[serde(default = "default_true")]
    pub show_tool_chest: bool,
    /// Show the Pages panel. Missing values fall back to `true`.
    #[serde(default = "default_true")]
    pub show_pages: bool,
    /// Canvas zoom level. Missing values fall back to `1.0`.
    #[serde(default = "default_one")]
    pub zoom: f32,
    /// Current page index. Missing values fall back to `0`.
    #[serde(default)]
    pub current_page: usize,
    /// Tool Chest style preset library, persisted across sessions.
    #[serde(default)]
    pub tool_chest_library: StylePresetLibrary,
    /// UI scale multiplier. Missing values fall back to `1.0`.
    #[serde(default = "default_one")]
    pub ui_scale: f32,
    /// Focus/reading mode: hides all side panels. Missing values fall back to `false`.
    #[serde(default)]
    pub focus_mode: bool,
    /// Show minimap overlay in canvas corner (proposal P2-16). Missing values fall back to `false`.
    #[serde(default)]
    pub show_minimap: bool,
    /// High-contrast mode (a11y A3 / proposal P2-15): pure black-on-white or
    /// white-on-black text and surfaces with stronger widget borders, for users
    /// who need maximum contrast. Missing values fall back to `false`.
    #[serde(default)]
    pub high_contrast: bool,
    /// Per-kind default style overrides ("Set As Default"). Missing values fall
    /// back to empty overrides, so new markups use their kind's coded default.
    #[serde(default)]
    pub default_styles: DefaultStyles,
    /// Display unit used for new measurements (gap #8: app default).
    #[serde(default)]
    pub default_measure_unit: MeasureUnit,
    /// Maximum number of entries kept in the Tool Chest "Recent" list
    /// (gap #3: Recent Tools). Session-temporary list; the cap is persisted.
    #[serde(default = "default_max_recents")]
    pub tool_chest_max_recents: usize,
    /// Persisted workspace state — dock regions, floating panels, split view.
    /// `#[serde(default)]` ensures legacy preference files that omit this key
    /// load unchanged; the default layout matches `WorkspaceState::default()`.
    #[serde(default)]
    pub workspace: WorkspaceState,
}

fn default_max_recents() -> usize {
    8
}

fn default_true() -> bool {
    true
}

fn default_one() -> f32 {
    1.0
}

impl Default for AppPrefs {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            show_markups_list: true,
            show_tool_chest: true,
            show_pages: true,
            zoom: 1.0,
            current_page: 0,
            tool_chest_library: StylePresetLibrary::new(),
            ui_scale: 1.0,
            focus_mode: false,
            show_minimap: false,
            high_contrast: false,
            default_styles: DefaultStyles::default(),
            default_measure_unit: MeasureUnit::default(),
            tool_chest_max_recents: default_max_recents(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_defaults_to_light() {
        assert_eq!(Theme::default(), Theme::Light);
        assert!(!Theme::default().is_dark());
        assert!(Theme::Dark.is_dark());
    }

    #[test]
    fn theme_all_lists_both_variants() {
        assert_eq!(Theme::ALL, [Theme::Light, Theme::Dark]);
        assert_eq!(Theme::ALL[0].label(), "Light");
        assert_eq!(Theme::ALL[1].label(), "Dark");
    }

    #[test]
    fn theme_serde_roundtrips() {
        for theme in Theme::ALL {
            let json = serde_json::to_string(&theme).expect("serialize");
            assert!(json.contains(&theme.label().to_lowercase()));
            let back: Theme = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, theme);
        }
    }

    #[test]
    fn prefs_without_theme_field_default_to_light() {
        let prefs: AppPrefs = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(prefs.theme, Theme::Light);
        assert_eq!(prefs, AppPrefs::default());
    }

    #[test]
    fn prefs_roundtrip() {
        let prefs = AppPrefs {
            theme: Theme::Dark,
            show_markups_list: true,
            show_tool_chest: true,
            show_pages: true,
            zoom: 1.5,
            current_page: 2,
            tool_chest_library: StylePresetLibrary::new(),
            ui_scale: 1.0,
            focus_mode: true,
            show_minimap: false,
            high_contrast: true,
            default_styles: DefaultStyles::default(),
            default_measure_unit: MeasureUnit::Inch,
            tool_chest_max_recents: 12,
        };
        let json = serde_json::to_string_pretty(&prefs).expect("serialize");
        let back: AppPrefs = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, prefs);
    }

    #[test]
    fn prefs_without_workspace_defaults_to_default_workspace() {
        // Legacy prefs that omit the `workspace` key should fall back to
        // `WorkspaceState::default()` via `#[serde(default)]`.
        let json = r#"{}#";
        let prefs: AppPrefs = serde_json::from_str(json).expect("deserialize");
        assert_eq!(prefs.workspace, WorkspaceState::default());
    }

    #[test]
    fn prefs_roundtrip_with_workspace() {
        let prefs = AppPrefs {
            theme: Theme::Light,
            show_markups_list: true,
            show_tool_chest: true,
            show_pages: true,
            zoom: 1.0,
            current_page: 0,
            tool_chest_library: StylePresetLibrary::new(),
            ui_scale: 1.0,
            focus_mode: false,
            show_minimap: false,
            high_contrast: false,
            default_styles: DefaultStyles::default(),
            default_measure_unit: MeasureUnit::Point,
            tool_chest_max_recents: 8,
            workspace: WorkspaceState::default(),
        };
        let json = serde_json::to_string_pretty(&prefs).expect("serialize");
        let back: AppPrefs = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, prefs);
    }
}
