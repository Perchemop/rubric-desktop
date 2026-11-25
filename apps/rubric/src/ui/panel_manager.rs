//! Panel Manager — owns all panel (dock/float/split) state and applies operations.
//!
//! Thin UI-side coordinator over the pure domain `WorkspaceState`. All layout
//! mutations go through here so UI components never touch domain state directly.

use crate::domain::{DockLocation, PanelLayout, Region, WorkspaceState};
use crate::domain::workspace::PanelId;
use egui::Context;

/// A 4-zone split target used by the Split Wheel overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitZone {
    Left,
    Right,
    Top,
    Bottom,
}

/// Panel manager owns the live workspace layout plus transient drag state.
#[derive(Debug, Clone)]
pub struct PanelManager {
    pub workspace: WorkspaceState,
    pub panel_layout: PanelLayout,
    /// Panel currently being dragged (for split-wheel / docking).
    pub dragging: Option<PanelId>,
    /// Whether the split-wheel overlay is visible.
    pub split_wheel_visible: bool,
    /// Active split zone under the cursor, if any.
    pub active_zone: Option<SplitZone>,
    /// Auto-hide state for the Panel Access Bar.
    pub access_bar_auto_hide: bool,
    /// Access bar expanded (hover) state.
    pub access_bar_expanded: bool,
}

impl Default for PanelManager {
    fn default() -> Self {
        Self {
            workspace: WorkspaceState::default(),
            panel_layout: PanelLayout::default(),
            dragging: None,
            split_wheel_visible: false,
            active_zone: None,
            access_bar_auto_hide: false,
            access_bar_expanded: true,
        }
    }
}

impl PanelManager {
    pub fn new(workspace: WorkspaceState) -> Self {
        let panel_layout = PanelLayout::from_workspace(&workspace);
        Self {
            workspace,
            panel_layout,
            ..Self::default()
        }
    }

    /// Dock a panel into the given region (edge docking).
    pub fn dock_panel(&mut self, panel: PanelId, edge: Region) {
        let _ = self.workspace.move_panel(panel.clone(), edge);
        self.workspace.open.insert(panel.clone());
        self.workspace.region.insert(panel, edge);
        self.panel_layout = PanelLayout::from_workspace(&self.workspace);
    }

    fn sync_layout(&mut self) {
        self.panel_layout = PanelLayout::from_workspace(&self.workspace);
    }

    /// Split a target panel, placing `dragged` in the given zone.
    pub fn split_panel(&mut self, _target: PanelId, dragged: PanelId, _zone: SplitZone) {
        // Domain layer only models a single split ratio today; record the intent
        // by docking the dragged panel into a neighbouring region. Full recursive
        // split tree lands with domain-dev PanelLayout.
        self.workspace.open.insert(dragged.clone());
        let region = self
            .workspace
            .region
            .get(&_target)
            .cloned()
            .unwrap_or(Region::Right);
        self.workspace.region.insert(dragged, region);
    }

    /// Add `dragged` as a tab of `target`.
    pub fn tab_panel(&mut self, _target: PanelId, dragged: PanelId) {
        self.workspace.open.insert(dragged.clone());
    }

    /// Float a panel (overlay/windowed mode handled by caller).
    pub fn float_panel(&mut self, panel: PanelId, rect: [f32; 4]) {
        let _ = self.workspace.float_panel(panel.clone(), rect);
        self.sync_layout();
    }

    /// Toggle panel visibility.
    pub fn toggle_panel(&mut self, panel: PanelId) {
        if self.workspace.open.contains(&panel) || self.workspace.floating.contains_key(&panel) {
            // hide: remove from open + floating
            self.workspace.open.remove(&panel);
            self.workspace.floating.remove(&panel);
        } else {
            self.workspace.open.insert(panel.clone());
            self.workspace.region.entry(panel).or_insert(Region::Left);
        }
        self.sync_layout();
    }

    /// Apply a full layout (from profile).
    pub fn apply_layout(&mut self, layout: PanelLayout) {
        // Convert PanelLayout (profile) -> WorkspaceState (live domain state).
        self.workspace = layout.to_workspace();
        self.panel_layout = layout;
    }

    /// Serialize current layout to PanelLayout (profile format).
    pub fn serialize_layout(&self) -> PanelLayout {
        PanelLayout::from_workspace(&self.workspace)
    }

    /// Begin a drag.
    pub fn begin_drag(&mut self, panel: PanelId) {
        self.dragging = Some(panel);
        self.split_wheel_visible = false;
    }

    /// Update split-wheel visibility based on cursor over a docked panel.
    pub fn update_split_wheel(&mut self, over_panel: bool) {
        self.split_wheel_visible = over_panel && self.dragging.is_some();
    }

    /// Commit a drop on a zone.
    pub fn commit_drop(&mut self, target: PanelId, zone: SplitZone) {
        if let Some(dragged) = self.dragging.take() {
            self.split_panel(target, dragged, zone);
        }
        self.split_wheel_visible = false;
        self.active_zone = None;
        let _ = (target, zone);
    }

    /// Cancel current drag.
    pub fn cancel_drag(&mut self) {
        self.dragging = None;
        self.split_wheel_visible = false;
        self.active_zone = None;
    }

    pub fn is_visible(&self, panel: PanelId) -> bool {
        self.workspace.open.contains(&panel) || self.workspace.floating.contains_key(&panel)
    }

    pub fn _ctx(&self) -> Option<&Context> {
        None
    }
}
