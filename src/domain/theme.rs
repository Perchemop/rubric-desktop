//! Theme preference (light/dark).
//!
//! The theme is an **app-level** preference (like the recent-files list), not
//! project data: it lives in the platform config directory and is never
//! written into a shareable `.rubric.json` project file.

use serde::{Deserialize, Serialize};

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

/// Application-level preferences stored outside project files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppPrefs {
    /// Selected colour theme. Missing values fall back to
    /// [`Theme::Light`] so older preference files keep loading.
    #[serde(default)]
    pub theme: Theme,
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
            assert!(json.contains(theme.label().to_lowercase()));
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
        let prefs = AppPrefs { theme: Theme::Dark };
        let json = serde_json::to_string_pretty(&prefs).expect("serialize");
        let back: AppPrefs = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, prefs);
    }
}
