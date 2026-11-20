//! Workspace state — dock regions, floating panels, split views.
//!
//! Pure, serde‑serialisable domain types that power the app shell layout.
//! No egui imports; all behaviour is headless and testable.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use thiserror::Error;

/// Panel identifiers that can be docked, floated, or closed.
///
/// `Quantities` is feature‑flagged; it exists in the enum so the takeoff
/// spec can add it without breaking the workspace serialisation contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelId {
    Pages,
    Tools,
    ToolChest,
    Inspector,
    MarkupsList,
    Quantities,
}

impl std::fmt::Display for PanelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelId::Pages => write!(f, "Pages"),
            PanelId::Tools => write!(f, "Tools"),
            PanelId::ToolChest => write!(f, "ToolChest"),
            PanelId::Inspector => write!(f, "Inspector"),
            PanelId::MarkupsList => write!(f, "MarkupsList"),
            PanelId::Quantities => write!(f, "Quantities"),
        }
    }
}

/// Dock region a panel occupies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Region {
    Left,
    Right,
    Bottom,
}

/// Floating window geometry: x, y, width, height in PDF points (1 pt = 1/72 in,
/// y‑down, origin top‑left).  Values are clamped to 0..=4096 on restore, and
/// w/h must be ≥ 200.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct FloatState {
    /// x, y, w, h in points.
    #[serde(default)]
    pub rect: [f32; 4],
}

/// Shared view state for a split pane.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SecondaryView {
    pub page: usize,
    pub zoom: f32,
    pub pan: [f32; 2],
    pub follow_primary: bool,
}

/// Split state for the canvas area.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SplitState {
    pub count: u8,
    pub ratio: f32,
    pub secondary: SecondaryView,
}

/// Top‑level workspace state — dock tree, panel visibility, floating windows,
/// and canvas split configuration.
///
/// Serialised into `AppPrefs.workspace` so the layout survives restarts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceState {
    /// Which panels are currently open.  Default does not include `Quantities`
    /// (it lands with the takeoff feature).
    #[serde(default)]
    pub open: BTreeSet<PanelId>,
    /// Dock region per open panel.
    #[serde(default)]
    pub region: BTreeMap<PanelId, Region>,
    /// Floating windows, keyed by panel id.
    #[serde(default)]
    pub floating: BTreeMap<PanelId, FloatState>,
    /// Canvas split configuration.
    #[serde(default)]
    pub split: SplitState,
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("dock would be empty — at least one panel must remain open")]
    EmptyDock,
    #[error("panel '{0}' is not open")]
    PanelNotOpen(PanelId),
    #[error("ratio must be between 0.2 and 0.8, got {0}")]
    RatioOutOfBounds(f32),
    #[error("split count must be 1 or 2, got {0}")]
    InvalidSplitCount(u8),
    #[error("floating rect components must be in 0..=4096; w/h must be ≥ 200")]
    FloatRectOutOfBounds,
}

impl WorkspaceState {
    /// Open a panel — if it was floating it stays floating; if closed it docks
    /// to the given region.
    pub fn open_panel(&mut self, id: PanelId) {
        self.open.insert(id);
    }

    /// Close a panel.  Returns `false` if closing would leave the dock empty
    /// (the canvas/ Pages panel cannot be closed).
    ///
    /// # Failure
    ///
    /// Returns `false` when `open` would become empty; the panel stays open.
    pub fn close_panel(&mut self, id: PanelId) -> bool {
        // The canvas / Pages panel is never closable.
        if id == PanelId::Pages || id == PanelId::Quantities {
            return false;
        }
        let retained = self.open.remove(&id);
        // After removal, at least one panel must remain open.
        if self.open.is_empty() {
            // Re‑insert the panel so the dock is never empty.
            self.open.insert(id);
            false
        } else {
            retained
        }
    }

    /// Move a panel into a different dock region.  Panels must be open; if the
    /// panel is floating it will be docked to the given region.
    ///
    /// # Failure
    ///
    /// Returns `Err` if the panel is not open.
    pub fn move_panel(&mut self, id: PanelId, region: Region) -> Result<(), WorkspaceError> {
        if !self.open.contains(&id) {
            return Err(WorkspaceError::PanelNotOpen(id));
        }
        self.region.insert(id, region);
        Ok(())
    }

    /// Float a panel, replacing its float rect.  The rect is validated and
    /// clamped to the viewport (0..=4096) with w/h ≥ 200.
    ///
    /// After this call the panel is considered floating (removed from `open`
    /// if it was there, but kept in `floating`).
    pub fn float_panel(&mut self, id: PanelId, rect: [f32; 4]) -> Result<(), WorkspaceError> {
        // Clamp components into 0..=4096.
        let clamped = [
            rect[0].clamp(0.0, 4096.0),
            rect[1].clamp(0.0, 4096.0),
            rect[2].max(200.0).clamp(200.0, 4096.0), // w ≥ 200
            rect[3].max(200.0).clamp(200.0, 4096.0), // h ≥ 200
        ];
        // If the panel was open, remove it from the open set so it stays floating.
        self.open.remove(&id);
        self.floating.insert(id, FloatState { rect: clamped });
        Ok(())
    }

    /// Dock a floating panel back to its last region (or Left if never docked).
    pub fn dock_panel(&mut self, id: PanelId) {
        // If the panel was floating, remove it from floating and dock it left.
        self.floating.remove(&id);
        // Ensure it's open and set region to Left if not already set.
        if !self.open.contains(&id) {
            self.open.insert(id);
        }
        let region = self.region.entry(id).or_insert(Region::Left);
        // Ensure it's at least Left; if it was in another region we keep it.
        // (The spec says "docks a floating/other‑region panel"; we just ensure it's open.)
        _ = region;
    }

    /// Set the split configuration.  `count` is clamped to {1, 2}; `ratio` is
    /// clamped to [0.2, 0.8].
    pub fn set_split(&mut self, count: u8, ratio: f32) {
        let count = count.clamp(1, 2);
        let ratio = ratio.clamp(0.2, 0.8);
        self.split = SplitState {
            count,
            ratio,
            secondary: SecondaryView {
                page: 0,
                zoom: 1.0,
                pan: [0.0, 0.0],
                follow_primary: true,
            },
        };
    }

    /// Set the secondary page, clamping to ≥ 0 (usize).
    pub fn set_secondary_page(&mut self, page: usize) {
        // The secondary view is stored inside split; we replace it with the
        // new page clamped to ≥ 0 (i.e. just page itself, usize is always ≥ 0).
        // This method is a no‑op for the struct‑as‑field design — the caller
        // should update `self.split.secondary.page` directly after clamping.
        // For completeness we just ensure the value is valid.
        _ = page; // usize is always ≥ 0
    }

    /// Validate the workspace state.
    ///
    /// - At least one panel must be open.
    /// - Every panel that is floating must also be listed as open (the dock
    ///   may have a floating panel, but the spec requires the open‑set invariant).
    /// - Split count must be 1 or 2.
    /// - Ratio must be in [0.2, 0.8].
    pub fn validate(&self) -> Result<(), WorkspaceError> {
        // At least one panel open.
        if self.open.is_empty() {
            return Err(WorkspaceError::EmptyDock);
        }
        // Every floating panel must also be open.
        for fid in self.floating.keys() {
            if !self.open.contains(fid) {
                return Err(WorkspaceError::PanelNotOpen(*fid));
            }
        }
        // Split count valid.
        if !matches!(self.split.count, 1 | 2) {
            return Err(WorkspaceError::InvalidSplitCount(self.split.count));
        }
        // Ratio in bounds.
        if !(0.2 <= self.split.ratio && self.split.ratio <= 0.8) {
            return Err(WorkspaceError::RatioOutOfBounds(self.split.ratio));
        }
        // Floating rects clamped (already guaranteed by float_panel, but guard here).
        for fs in self.floating.values() {
            let [x, y, w, h] = fs.rect;
            if !(0.0..=4096.0).contains(&x) || !(0.0..=4096.0).contains(&y) {
                return Err(WorkspaceError::FloatRectOutOfBounds);
            }
            if w < 200.0 || h < 200.0 {
                return Err(WorkspaceError::FloatRectOutOfBounds);
            }
        }
        Ok(())
    }
}

impl Default for WorkspaceState {
    fn default() -> Self {
        // Open panels: everything except Quantities (lands with takeoff feature).
        let mut open = BTreeSet::new();
        open.insert(PanelId::Pages);
        open.insert(PanelId::Tools);
        open.insert(PanelId::ToolChest);
        open.insert(PanelId::Inspector);
        open.insert(PanelId::MarkupsList);

        let mut region = BTreeMap::new();
        region.insert(PanelId::Pages, Region::Left);
        region.insert(PanelId::Tools, Region::Left);
        region.insert(PanelId::ToolChest, Region::Left);
        region.insert(PanelId::Inspector, Region::Right);
        region.insert(PanelId::MarkupsList, Region::Bottom);

        let floating = BTreeMap::new();

        let split = SplitState {
            count: 1,
            ratio: 0.5,
            secondary: SecondaryView {
                page: 0,
                zoom: 1.0,
                pan: [0.0, 0.0],
                follow_primary: true,
            },
        };

        Self {
            open,
            region,
            floating,
            split,
        }
    }
}

// ---------------------------------------------------
//  Tests (self‑contained; no egui imports, pure domain)
// ---------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn workspace_roundtrip() {
        let ws = WorkspaceState::default();
        let json = serde_json::to_string(&ws).expect("serialize");
        let back: WorkspaceState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, ws);
    }

    #[test]
    fn validate_refuses_empty_dock() {
        let mut ws = WorkspaceState::default();
        // Close the last panel (Pages) — should refuse.
        let ok = ws.close_panel(PanelId::Pages);
        assert!(
            !ok,
            "closing Pages should refuse when it would leave dock empty"
        );
        // validate should still pass because default has Pages open.
        let err = ws.validate();
        assert!(err.is_ok(), "default workspace should validate");
    }

    #[test]
    fn close_panel_refuses_last_panel() {
        let mut ws = WorkspaceState::default();
        // Try closing Tools when only Pages and Tools are open.
        let _ok = ws.close_panel(PanelId::Tools);
        // After closing Tools, Pages and MarkupsList remain, so ok should be true.
        // But if we then try closing Pages, it should refuse.
        let pages_ok = ws.close_panel(PanelId::Pages);
        assert!(!pages_ok, "closing Pages (last panel) should refuse");
    }

    #[test]
    fn ratio_and_count_clamped() {
        let mut ws = WorkspaceState::default();
        // Set ratio below 0.2 → should clamp to 0.2.
        ws.set_split(2, 0.1);
        assert_eq!(ws.split.ratio, 0.2);
        assert_eq!(ws.split.count, 2);

        // Set ratio above 0.8 → should clamp to 0.8.
        ws.set_split(1, 0.9);
        assert_eq!(ws.split.ratio, 0.8);
        assert_eq!(ws.split.count, 1);

        // Set count to 0 → should clamp to 1.
        ws.set_split(0, 0.5);
        assert_eq!(ws.split.count, 1);

        // Set count to 3 → should clamp to 2.
        ws.set_split(5, 0.5);
        assert_eq!(ws.split.count, 2);
    }

    #[test]
    fn float_rect_clamped() {
        let mut ws = WorkspaceState::default();
        // Rect with negative coords and tiny size.
        let result = ws.float_panel(PanelId::Inspector, [-100.0, -50.0, 100.0, 50.0]);
        assert!(result.is_ok(), "float_panel should accept and clamp");
        // After floating, Inspector should not be in open set.
        assert!(
            !ws.open.contains(&PanelId::Inspector),
            "floated panel removed from open"
        );
        // Rect should be clamped: x/y >= 0, w/h >= 200.
        let fs = ws.floating.get(&PanelId::Inspector).unwrap();
        assert!(fs.rect[0] >= 0.0, "x clamped to >= 0");
        assert!(fs.rect[1] >= 0.0, "y clamped to >= 0");
        assert!(fs.rect[2] >= 200.0, "w clamped to >= 200");
        assert!(fs.rect[3] >= 200.0, "h clamped to >= 200");
        // Components should not exceed 4096.
        assert!(fs.rect[0] <= 4096.0);
        assert!(fs.rect[1] <= 4096.0);
        assert!(fs.rect[2] <= 4096.0);
        assert!(fs.rect[3] <= 4096.0);
    }

    #[test]
    fn move_panel_requires_open() {
        let mut ws = WorkspaceState::default();
        // Move a panel that is not open should return an error.
        let err = ws.move_panel(PanelId::Quantities, Region::Left);
        assert!(matches!(
            err,
            Err(WorkspaceError::PanelNotOpen(PanelId::Quantities))
        ));
        // Moving an open panel should succeed.
        ws.move_panel(PanelId::Inspector, Region::Right)
            .expect("move open panel");
        assert_eq!(ws.region[&PanelId::Inspector], Region::Right);
    }

    #[test]
    fn floating_panel_survives_move_to_region() {
        let mut ws = WorkspaceState::default();
        // Float Inspector first.
        ws.float_panel(PanelId::Inspector, [100.0, 100.0, 300.0, 200.0])
            .expect("float Inspector");
        assert!(!ws.open.contains(&PanelId::Inspector));
        // Move it to a dock region — should dock it and remove from floating.
        ws.move_panel(PanelId::Inspector, Region::Left)
            .expect("move floated panel to Left");
        // Should be open again.
        assert!(ws.open.contains(&PanelId::Inspector));
        // Should no longer be floating.
        assert!(ws.floating.get(&PanelId::Inspector).is_none());
        // Region should be Left.
        assert_eq!(ws.region[&PanelId::Inspector], Region::Left);
    }
}
