//! Apply a profile to the application UI state.

use crate::domain::profile::Profile;
use crate::ui::app::RubricApp;

impl RubricApp {
    /// Apply a complete profile to the application state.
    pub fn apply_profile(&mut self, ctx: &egui::Context, profile: &Profile) -> anyhow::Result<()> {
        // Apply theme override
        if let Some(theme) = profile.theme_override {
            self.set_theme(ctx, theme);
        }

        Ok(())
    }
}

/// Helper to get the active profile name from settings.
pub fn get_active_profile_name(app: &RubricApp) -> String {
    app.active_profile.clone()
}

/// Helper to set the active profile name in settings.
pub fn set_active_profile_name(app: &mut RubricApp, name: String) {
    app.active_profile = name;
}
