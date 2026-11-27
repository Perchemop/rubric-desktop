//! Toolbar runtime: builds egui toolbars from a [`ToolbarConfig`] and renders
//! the Main Toolbar, Quick Access Area, Properties Toolbar and custom toolbars.
//!
//! Domain persistence (profile integration) is a hook only at this stage: the
//! runtime calls `on_change`-style callbacks via the `ToolbarManager::dirty`
//! flag so the app shell can debounce-save to the profile.

use eframe::egui;
use eframe::egui::Ui;
use std::collections::HashMap;
/// A single entry in a toolbar.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ToolbarItem {
    /// A command from the command registry, identified by its string id.
    Tool(String),
    /// A visual separator.
    Divider,
    /// Flexible spacer.
    Spacer,
}

/// Visibility + lock state for one toolbar.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolbarConfig {
    /// Stable id (e.g. "main", "quick_access", "properties", or custom name).
    pub id: String,
    /// Display name shown in the customization dialog.
    pub name: String,
    /// Whether this toolbar is predefined (not deletable/renamable).
    pub predefined: bool,
    /// Docking edge for custom toolbars.
    pub dock: ToolbarDock,
    /// Whether the toolbar is currently visible.
    pub visible: bool,
    /// Ordered items.
    pub items: Vec<ToolbarItem>,
    /// Per-toolbar lock override (only meaningful for custom toolbars).
    pub locked_override: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum ToolbarDock {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
    Float,
}

/// Global lock state shared across all toolbars.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct ToolbarLockState {
    pub global_locked: bool,
    /// Per-custom-toolbar overrides: true = unlocked even when global lock on.
    pub custom_overrides: HashMap<String, bool>,
}

/// Runtime manager owning toolbar configs + lock state. Builds egui UI.
#[derive(Debug, Clone, Default)]
pub struct ToolbarManager {
    pub toolbars: Vec<ToolbarConfig>,
    pub lock: ToolbarLockState,
    /// Set when config changed; app shell debounces save (500ms).
    pub dirty: bool,
    /// Last change instant for debounce.
    pub last_change: Option<std::time::Instant>,
}

impl ToolbarManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_locked(&self, toolbar_id: &str) -> bool {
        if !self.lock.global_locked {
            return false;
        }
        // Custom override may unlock.
        if let Some(ov) = self.lock.custom_overrides.get(toolbar_id) {
            return !*ov;
        }
        true
    }

    pub fn set_dirty(&mut self) {
        self.dirty = true;
        self.last_change = Some(std::time::Instant::now());
    }

    /// Debounced persistence hook: returns true when 500ms elapsed since last change.
    pub fn should_persist(&self) -> bool {
        if !self.dirty {
            return false;
        }
        self.last_change
            .map(|t| t.elapsed().as_millis() >= 500)
            .unwrap_or(false)
    }

    pub fn mark_persisted(&mut self) {
        self.dirty = false;
    }

    /// Render the main toolbar (top, horizontal). Returns true if a command was clicked.
    pub fn render_main_toolbar(&self, ui: &mut Ui, _active_command: &mut Option<String>) -> bool {
        let mut clicked = false;
        let cfg = self.toolbars.iter().find(|t| t.id == "main");
        let Some(cfg) = cfg else { return false };
        if !cfg.visible {
            return false;
        }
        let locked = self.is_locked(&cfg.id);
        ui.horizontal(|ui| {
            if !locked {
                ui.label("......");
            }
            for item in &cfg.items {
                match item {
                    ToolbarItem::Tool(id) => {
                        if ui.button(id).clicked() {
                            *_active_command = Some(id.clone());
                            clicked = true;
                        }
                    }
                    ToolbarItem::Divider => {
                        ui.separator();
                    }
                    ToolbarItem::Spacer => {
                        ui.add_space(8.0);
                    }
                }
            }
        });
        clicked
    }

    /// Render the quick access area (top-right, flexible).
    pub fn render_quick_access(&self, ui: &mut Ui, _active_command: &mut Option<String>) -> bool {
        let mut clicked = false;
        let cfg = self.toolbars.iter().find(|t| t.id == "quick_access");
        let Some(cfg) = cfg else { return false };
        if !cfg.visible {
            return false;
        }
        ui.horizontal(|ui| {
            for item in &cfg.items {
                match item {
                    ToolbarItem::Tool(id) => {
                        if ui.small_button(id).clicked() {
                            *_active_command = Some(id.clone());
                            clicked = true;
                        }
                    }
                    ToolbarItem::Divider => {
                        ui.separator();
                    }
                    ToolbarItem::Spacer => {
                        ui.add_space(4.0);
                    }
                }
            }
        });
        clicked
    }

    /// Render properties toolbar (below main, auto-show/hide controlled by caller).
    pub fn render_properties_toolbar(&self, ui: &mut Ui) {
        let cfg = self.toolbars.iter().find(|t| t.id == "properties");
        let Some(cfg) = cfg else { return };
        if !cfg.visible {
            return;
        }
        ui.horizontal(|ui| {
            for item in &cfg.items {
                match item {
                    ToolbarItem::Tool(id) => {
                        ui.label(id);
                    }
                    ToolbarItem::Divider => {
                        ui.separator();
                    }
                    ToolbarItem::Spacer => {
                        ui.add_space(4.0);
                    }
                }
            }
        });
    }

    /// Render custom toolbars (dockable). Caller positions them.
    pub fn render_custom_toolbar(
        &self,
        ui: &mut Ui,
        id: &str,
        _active_command: &mut Option<String>,
    ) -> bool {
        let mut clicked = false;
        let cfg = self.toolbars.iter().find(|t| t.id == id);
        let Some(cfg) = cfg else { return false };
        if !cfg.visible {
            return false;
        }
        let locked = self.is_locked(id);
        ui.horizontal(|ui| {
            if !locked {
                ui.label("......");
            }
            for item in &cfg.items {
                match item {
                    ToolbarItem::Tool(cmd) => {
                        if ui.button(cmd).clicked() {
                            *_active_command = Some(cmd.clone());
                            clicked = true;
                        }
                    }
                    ToolbarItem::Divider => {
                        ui.separator();
                    }
                    ToolbarItem::Spacer => {
                        ui.add_space(8.0);
                    }
                }
            }
        });
        clicked
    }
}
