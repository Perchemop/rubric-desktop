//! Apply a profile to the application UI state.

use crate::domain::profile::{PanelLayout, Profile, ToolbarConfig, Theme, MeasurementDefaults, CanvasSettings};
use crate::ui::app::RubricApp;
use anyhow::Result;

impl RubricApp {
    /// Apply a complete profile to the application state.
    pub fn apply_profile(&mut self, profile: &Profile) -> Result<()> {
        // Apply panel layout
        self.apply_panel_layout(&profile.ui_layout.panels)?;

        // Apply toolbar configuration
        self.apply_toolbar_config(&profile.ui_layout.toolbars)?;

        // Apply theme override
        if let Some(theme) = profile.ui_layout.theme_override {
            self.apply_theme(theme)?;
        }

        // Apply measurement defaults
        self.apply_measurement_defaults(&profile.ui_layout.measurement_defaults)?;

        // Apply canvas settings
        self.apply_canvas_settings(&profile.ui_layout.canvas)?;

        Ok(())
    }

    fn apply_panel_layout(&mut self, layout: &PanelLayout) -> Result<()> {
        // Apply left dock strip
        if let Some(ref mut left_panel) = self.ui.left_panel {
            left_panel.visible = layout.left.iter().any(|p| p.visible);
        }
        // Apply right dock strip
        if let Some(ref mut right_panel) = self.ui.right_panel {
            right_panel.visible = layout.right.iter().any(|p| p.visible);
        }
        // Apply bottom dock strip
        if let Some(ref mut bottom_panel) = self.ui.bottom_panel {
            bottom_panel.visible = layout.bottom.iter().any(|p| p.visible);
        }

        // TODO: Full split tree restoration, tab groups, floating panels
        // This requires the PanelManager to be fully wired up

        Ok(())
    }

    fn apply_toolbar_config(&mut self, config: &ToolbarConfig) -> Result<()> {
        // Main toolbar
        self.ui.toolbar_manager.main_toolbar.visible = config.visible;
        self.ui.toolbar_manager.main_toolbar.locked = config.locked;

        // Quick access area
        self.ui.toolbar_manager.quick_access = config.quick_access.clone();

        // Properties toolbar
        self.ui.toolbar_manager.properties_toolbar.visible = config.properties_visible;

        // Custom toolbars
        self.ui.toolbar_manager.custom_toolbars = config.custom_toolbars.clone();

        Ok(())
    }

    fn apply_theme(&mut self, theme: Theme) -> Result<()> {
        self.set_theme(theme);
        Ok(())
    }

    fn apply_measurement_defaults(&mut self, defaults: &MeasurementDefaults) -> Result<()> {
        self.ui.measurement_defaults = defaults.clone();
        Ok(())
    }

    fn apply_canvas_settings(&mut self, settings: &CanvasSettings) -> Result<()> {
        self.ui.zoom = settings.zoom;
        self.ui.page_layout = settings.page_layout;
        self.ui.show_rulers = settings.show_rulers;
        self.ui.show_guides = settings.show_guides;
        Ok(())
    }
}

/// Helper to get the active profile name from settings.
pub fn get_active_profile_name(app: &RubricApp) -> String {
    app.settings.active_profile.clone()
}

/// Helper to set the active profile name in settings.
pub fn set_active_profile_name(app: &mut RubricApp, name: String) {
    app.settings.active_profile = name;
}