//! Panel Docking & Splitting domain models.
//!
//! Pure, serde-serialisable types for panel layout persistence and manipulation.
//! No egui imports; all behaviour is headless and testable.
//!
//! Based on specification: `docs/spec/panel-docking-splitting.spec.md`

use serde::{Deserialize, Serialize};

/// Schema version for panel layout serialization.
/// Increment when backwards-compatible changes are made; bump major for breaking changes.
pub const PANEL_SCHEMA_VERSION: u32 = 1;

/// Dock location for a panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockLocation {
    /// Panel is docked in the left strip.
    Left,
    /// Panel is docked in the right strip.
    Right,
    /// Panel is docked in the bottom strip.
    Bottom,
    /// Panel is floating (not in any dock strip).
    Floating,
}

impl Default for DockLocation {
    fn default() -> Self {
        Self::Left
    }
}

/// Panel identifiers that can be docked, split, tabbed, or floated.
///
/// Based on the panel inventory in spec F-607 section 2.1.
/// Extensible for future plugins via the `Other(String)` variant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelId {
    /// Page Thumbnails panel.
    Thumbnails,
    /// Layers visibility/order panel.
    Layers,
    /// Tool Chest with stamp presets and custom tools.
    ToolChest,
    /// Markups List panel.
    MarkupsList,
    /// Properties panel for selected object.
    Properties,
    /// Stamps library panel.
    Stamps,
    /// Measurements list and calibration.
    Measurements,
    /// Extensible variant for plugin panels or future additions.
    #[serde(other)]
    Other(String),
}

impl PanelId {
    /// Returns the default keyboard shortcut for this panel (Ctrl+1..7).
    /// Panels not in the primary 7 return None.
    pub fn default_shortcut(&self) -> Option<u8> {
        match self {
            PanelId::Thumbnails => Some(1),
            PanelId::Layers => Some(2),
            PanelId::ToolChest => Some(3),
            PanelId::MarkupsList => Some(4),
            PanelId::Properties => Some(5),
            PanelId::Stamps => Some(6),
            PanelId::Measurements => Some(7),
            PanelId::Other(_) => None,
        }
    }

    /// Returns the display title for this panel.
    pub fn title(&self) -> String {
        match self {
            PanelId::Thumbnails => "Thumbnails".to_string(),
            PanelId::Layers => "Layers".to_string(),
            PanelId::ToolChest => "Tool Chest".to_string(),
            PanelId::MarkupsList => "Markups List".to_string(),
            PanelId::Properties => "Properties".to_string(),
            PanelId::Stamps => "Stamps".to_string(),
            PanelId::Measurements => "Measurements".to_string(),
            PanelId::Other(name) => name.clone(),
        }
    }
}

impl std::fmt::Display for PanelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title())
    }
}

/// Split orientation for recursive panel splits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitOrientation {
    /// Panels arranged horizontally (side by side).
    Horizontal,
    /// Panels arranged vertically (stacked top to bottom).
    Vertical,
}

/// Recursive split tree for panel layout.
///
/// Represents the hierarchical division of a dock strip into panels.
/// Used for profile serialization and UI rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum SplitTree {
    /// A simple leaf containing just a panel ID.
    Leaf(PanelId),
    /// A split with two children arranged along an axis.
    Split {
        /// The axis of division.
        orientation: SplitOrientation,
        /// The ratio (0.0–1.0) for split position. 0.5 = centered.
        ratio: f32,
        /// The two child trees.
        children: Box<[SplitTree; 2]>,
    },
}

impl SplitTree {
    /// Creates a leaf node for a single panel.
    pub fn leaf(panel: PanelId) -> Self {
        SplitTree::Leaf(panel)
    }

    /// Creates a split from two children. Ratio is clamped to [0.05, 0.95].
    pub fn split(orientation: SplitOrientation, left: SplitTree, right: SplitTree) -> Self {
        let ratio = 0.5;
        SplitTree::Split {
            orientation,
            ratio,
            children: Box::new([left, right]),
        }
    }

    /// Creates a split with a specific ratio.
    pub fn split_with_ratio(
        orientation: SplitOrientation,
        ratio: f32,
        left: SplitTree,
        right: SplitTree,
    ) -> Self {
        SplitTree::Split {
            orientation,
            ratio: ratio.clamp(0.05, 0.95),
            children: Box::new([left, right]),
        }
    }

    /// Returns the maximum depth of this split tree.
    pub fn depth(&self) -> u32 {
        match self {
            SplitTree::Leaf(_) => 0,
            SplitTree::Split { children, .. } => {
                1 + children.iter().map(|c| c.depth()).max().unwrap_or(0)
            }
        }
    }

    /// Collects all panel IDs in this tree (in display order).
    pub fn panels(&self) -> Vec<PanelId> {
        match self {
            SplitTree::Leaf(id) => vec![id.clone()],
            SplitTree::Split {
                orientation: _,
                ratio: _,
                children,
            } => {
                let mut panels = Vec::new();
                // Left then right for consistent ordering
                panels.extend(children[0].panels());
                panels.extend(children[1].panels());
                panels
            }
        }
    }
}

/// Tab group containing multiple panels that share a tab bar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabGroup {
    /// Panel IDs in tab order (leftmost tab = index 0).
    pub tabs: Vec<PanelId>,
    /// Index of the currently active tab. Invalid if >= tabs.len().
    pub active_index: usize,
}

impl TabGroup {
    /// Creates a new tab group with a single panel.
    pub fn single(panel: PanelId) -> Self {
        Self {
            tabs: vec![panel],
            active_index: 0,
        }
    }

    /// Returns true if this tab group can be closed (has more than one tab).
    pub fn can_close_tab(&self) -> bool {
        self.tabs.len() > 1
    }

    /// Closes the tab at the given index, returns the closed panel ID.
    /// Returns None if the tab group would become empty.
    pub fn close_tab(&mut self, index: usize) -> Option<PanelId> {
        if index >= self.tabs.len() || self.tabs.len() <= 1 {
            return None;
        }
        let closed = self.tabs.remove(index);
        // Adjust active_index if needed
        if self.active_index >= self.tabs.len() && self.active_index > 0 {
            self.active_index -= 1;
        }
        Some(closed)
    }

    /// Activates the tab at the given index.
    pub fn activate(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }
}

impl Default for TabGroup {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_index: 0,
        }
    }
}

/// Floating panel state with position, size, and mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloatingPanel {
    /// The panel ID shown in this floating window.
    pub panel_id: PanelId,
    /// Position in PDF points (x, y) from top-left of workspace.
    pub position: [f32; 2],
    /// Size in PDF points (width, height).
    pub size: [f32; 2],
    /// Window mode.
    pub mode: FloatingMode,
    /// Monitor this panel was last seen on (if multi-monitor setup).
    #[serde(default)]
    pub monitor: Option<u64>,
}

impl FloatingPanel {
    /// Creates a new floating panel centered on primary monitor.
    pub fn new(panel: PanelId) -> Self {
        Self {
            panel_id: panel,
            position: [100.0, 100.0],
            size: [400.0, 300.0],
            mode: FloatingMode::Windowed,
            monitor: None,
        }
    }

    /// Clamps position to screen bounds (0..=4096) and size to minimums.
    pub fn clamp(&mut self) {
        self.position[0] = self.position[0].clamp(0.0, 4096.0);
        self.position[1] = self.position[1].clamp(0.0, 4096.0);
        self.size[0] = self.size[0].max(200.0).min(4096.0);
        self.size[1] = self.size[1].max(150.0).min(4096.0);
    }
}

/// Floating window mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatingMode {
    /// Native OS window with title bar and taskbar entry.
    Windowed,
    /// Frameless, always-on-top overlay over the main window.
    Overlay,
}

impl Default for FloatingMode {
    fn default() -> Self {
        Self::Windowed
    }
}

/// State for the Panel Access Bar (left-edge icon strip).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelAccessBarState {
    /// Whether the bar is in auto-hide mode (collapsed until hovered).
    #[serde(default)]
    pub auto_hide: bool,
    /// List of panel IDs whose icons are visible in the bar.
    #[serde(default)]
    pub visible_icons: Vec<PanelId>,
}

impl Default for PanelAccessBarState {
    fn default() -> Self {
        Self {
            auto_hide: false,
            visible_icons: Vec::new(),
        }
    }
}

/// Complete panel layout state for profile serialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelLayout {
    /// Panels docked in the left strip (in tab order).
    #[serde(default)]
    pub left: Vec<DockedPanel>,
    /// Panels docked in the right strip (in tab order).
    #[serde(default)]
    pub right: Vec<DockedPanel>,
    /// Panels docked in the bottom strip (in tab order).
    #[serde(default)]
    pub bottom: Vec<DockedPanel>,
    /// Width of the left dock strip in PDF points.
    #[serde(default = "default_strip_size_left")]
    pub strip_size_left: f32,
    /// Width of the right dock strip in PDF points.
    #[serde(default = "default_strip_size_right")]
    pub strip_size_right: f32,
    /// Height of the bottom dock strip in PDF points.
    #[serde(default = "default_strip_size_bottom")]
    pub strip_size_bottom: f32,
    /// Panel Access Bar state.
    #[serde(default)]
    pub access_bar: PanelAccessBarState,
    /// Schema version for migration purposes.
    #[serde(default = "PANEL_SCHEMA_VERSION")]
    pub schema_version: u32,
}

// Default strip sizes
fn default_strip_size_left() -> f32 {
    200.0
}
fn default_strip_size_right() -> f32 {
    260.0
}
fn default_strip_size_bottom() -> f32 {
    200.0
}

impl Default for PanelLayout {
    fn default() -> Self {
        Self {
            left: Vec::new(),
            right: Vec::new(),
            bottom: Vec::new(),
            strip_size_left: default_strip_size_left(),
            strip_size_right: default_strip_size_right(),
            strip_size_bottom: default_strip_size_bottom(),
            access_bar: PanelAccessBarState::default(),
            schema_version: PANEL_SCHEMA_VERSION,
        }
    }
}

/// A panel entry in a dock strip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockedPanel {
    /// Unique panel identifier.
    pub id: PanelId,
    /// Whether this panel is visible.
    #[serde(default)]
    pub visible: bool,
    /// Current dock location.
    #[serde(default)]
    pub dock_location: DockLocation,
    /// Split tree if this panel has splits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub split_tree: Option<SplitTree>,
    /// Order of tabs within this panel's group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tab_order: Vec<PanelId>,
    /// Floating state (position/size/mode) if panel is floating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floating: Option<FloatingPanel>,
}

impl DockedPanel {
    /// Creates a new docked panel entry.
    pub fn new(id: PanelId) -> Self {
        let id_clone = id.clone();
        Self {
            id,
            visible: true,
            dock_location: DockLocation::Left,
            split_tree: None,
            tab_order: vec![id_clone],
            floating: None,
        }
    }

    /// Returns true if this panel is floating.
    pub fn is_floating(&self) -> bool {
        self.dock_location == DockLocation::Floating || self.floating.is_some()
    }

    /// Returns true if this panel has an active split.
    pub fn has_split(&self) -> bool {
        self.split_tree.is_some()
    }
}

/// Predefined profile layouts as documented in spec section 2.9.
pub mod predefined_layouts {
    use super::*;

    /// Default profile: Left thumbnails+layers, Right properties+stamps+measurements, Bottom markups list.
    pub fn default() -> PanelLayout {
        PanelLayout {
            left: vec![
                DockedPanel {
                    id: PanelId::Thumbnails,
                    visible: true,
                    dock_location: DockLocation::Left,
                    split_tree: None,
                    tab_order: vec![PanelId::Thumbnails, PanelId::Layers],
                    floating: None,
                },
                DockedPanel::new(PanelId::Layers),
            ],
            right: vec![
                DockedPanel {
                    id: PanelId::Properties,
                    visible: true,
                    dock_location: DockLocation::Right,
                    split_tree: None,
                    tab_order: vec![PanelId::Properties, PanelId::Stamps, PanelId::Measurements],
                    floating: None,
                },
                DockedPanel::new(PanelId::Stamps),
                DockedPanel::new(PanelId::Measurements),
            ],
            bottom: vec![DockedPanel {
                id: PanelId::MarkupsList,
                visible: true,
                dock_location: DockLocation::Bottom,
                split_tree: None,
                tab_order: vec![PanelId::MarkupsList],
                floating: None,
            }],
            strip_size_left: 200.0,
            strip_size_right: 260.0,
            strip_size_bottom: 200.0,
            access_bar: PanelAccessBarState {
                auto_hide: false,
                visible_icons: vec![
                    PanelId::Thumbnails,
                    PanelId::Layers,
                    PanelId::ToolChest,
                    PanelId::MarkupsList,
                    PanelId::Properties,
                    PanelId::Stamps,
                    PanelId::Measurements,
                ],
            },
            schema_version: PANEL_SCHEMA_VERSION,
        }
    }

    /// Review profile: Left thumbnails+bookmarks, Right properties+markups list.
    pub fn review() -> PanelLayout {
        // Note: spec mentions Bookmarks tab but that's handled via tab_order
        PanelLayout {
            left: vec![DockedPanel {
                id: PanelId::Thumbnails,
                visible: true,
                dock_location: DockLocation::Left,
                split_tree: None,
                tab_order: vec![PanelId::Thumbnails, PanelId::Other("bookmarks".to_string())],
                floating: None,
            }],
            right: vec![
                DockedPanel {
                    id: PanelId::MarkupsList,
                    visible: true,
                    dock_location: DockLocation::Right,
                    split_tree: None,
                    tab_order: vec![PanelId::MarkupsList, PanelId::Properties],
                    floating: None,
                },
                DockedPanel::new(PanelId::Properties),
            ],
            bottom: Vec::new(),
            strip_size_left: 200.0,
            strip_size_right: 280.0,
            strip_size_bottom: 0.0,
            access_bar: PanelAccessBarState {
                auto_hide: false,
                visible_icons: vec![
                    PanelId::Thumbnails,
                    PanelId::MarkupsList,
                    PanelId::Properties,
                ],
            },
            schema_version: PANEL_SCHEMA_VERSION,
        }
    }

    /// Measurement profile: Left layers+tool chest, Right properties+measurements.
    pub fn measurement() -> PanelLayout {
        PanelLayout {
            left: vec![
                DockedPanel {
                    id: PanelId::Layers,
                    visible: true,
                    dock_location: DockLocation::Left,
                    split_tree: None,
                    tab_order: vec![PanelId::Layers, PanelId::ToolChest],
                    floating: None,
                },
                DockedPanel::new(PanelId::ToolChest),
            ],
            right: vec![
                DockedPanel {
                    id: PanelId::Properties,
                    visible: true,
                    dock_location: DockLocation::Right,
                    split_tree: None,
                    tab_order: vec![PanelId::Properties, PanelId::Measurements],
                    floating: None,
                },
                DockedPanel::new(PanelId::Measurements),
            ],
            bottom: Vec::new(),
            strip_size_left: 180.0,
            strip_size_right: 260.0,
            strip_size_bottom: 0.0,
            access_bar: PanelAccessBarState {
                auto_hide: false,
                visible_icons: vec![
                    PanelId::Layers,
                    PanelId::ToolChest,
                    PanelId::Properties,
                    PanelId::Measurements,
                ],
            },
            schema_version: PANEL_SCHEMA_VERSION,
        }
    }

    /// Markup profile: Left tool chest+stamps, Right properties+layers.
    pub fn markup() -> PanelLayout {
        PanelLayout {
            left: vec![
                DockedPanel {
                    id: PanelId::ToolChest,
                    visible: true,
                    dock_location: DockLocation::Left,
                    split_tree: None,
                    tab_order: vec![PanelId::ToolChest, PanelId::Stamps],
                    floating: None,
                },
                DockedPanel::new(PanelId::Stamps),
            ],
            right: vec![
                DockedPanel {
                    id: PanelId::Properties,
                    visible: true,
                    dock_location: DockLocation::Right,
                    split_tree: None,
                    tab_order: vec![PanelId::Properties, PanelId::Layers],
                    floating: None,
                },
                DockedPanel::new(PanelId::Layers),
            ],
            bottom: Vec::new(),
            strip_size_left: 200.0,
            strip_size_right: 260.0,
            strip_size_bottom: 0.0,
            access_bar: PanelAccessBarState {
                auto_hide: false,
                visible_icons: vec![
                    PanelId::ToolChest,
                    PanelId::Stamps,
                    PanelId::Properties,
                    PanelId::Layers,
                ],
            },
            schema_version: PANEL_SCHEMA_VERSION,
        }
    }

    /// Minimal profile: all panels hidden.
    pub fn minimal() -> PanelLayout {
        PanelLayout {
            left: Vec::new(),
            right: Vec::new(),
            bottom: Vec::new(),
            strip_size_left: 0.0,
            strip_size_right: 0.0,
            strip_size_bottom: 0.0,
            access_bar: PanelAccessBarState {
                auto_hide: true,
                visible_icons: Vec::new(),
            },
            schema_version: PANEL_SCHEMA_VERSION,
        }
    }

    /// Presentation profile: clean primary, markups floating on secondary monitor.
    pub fn presentation() -> PanelLayout {
        PanelLayout {
            left: Vec::new(),
            right: Vec::new(),
            bottom: Vec::new(),
            strip_size_left: 0.0,
            strip_size_right: 0.0,
            strip_size_bottom: 0.0,
            access_bar: PanelAccessBarState {
                auto_hide: true,
                visible_icons: Vec::new(),
            },
            schema_version: PANEL_SCHEMA_VERSION,
            // Note: floating panels are stored in the Profile struct, not PanelLayout
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_id_shortcut_mapping() {
        assert_eq!(PanelId::Thumbnails.default_shortcut(), Some(1));
        assert_eq!(PanelId::Layers.default_shortcut(), Some(2));
        assert_eq!(PanelId::ToolChest.default_shortcut(), Some(3));
        assert_eq!(PanelId::MarkupsList.default_shortcut(), Some(4));
        assert_eq!(PanelId::Properties.default_shortcut(), Some(5));
        assert_eq!(PanelId::Stamps.default_shortcut(), Some(6));
        assert_eq!(PanelId::Measurements.default_shortcut(), Some(7));
        assert_eq!(
            PanelId::Other("custom".to_string()).default_shortcut(),
            None
        );
    }

    #[test]
    fn split_tree_leaf() {
        let leaf = SplitTree::leaf(PanelId::Thumbnails);
        assert!(matches!(leaf, SplitTree::Leaf(PanelId::Thumbnails)));
        assert_eq!(leaf.depth(), 0);
        assert_eq!(leaf.panels(), vec![PanelId::Thumbnails]);
    }

    #[test]
    fn split_tree_split() {
        let left = SplitTree::leaf(PanelId::Thumbnails);
        let right = SplitTree::leaf(PanelId::Layers);
        let split = SplitTree::split(SplitOrientation::Horizontal, left, right);

        assert!(matches!(
            split,
            SplitTree::Split {
                orientation: SplitOrientation::Horizontal,
                ..
            }
        ));
        assert_eq!(split.depth(), 1);
        assert_eq!(split.panels(), vec![PanelId::Thumbnails, PanelId::Layers]);
    }

    #[test]
    fn split_tree_with_ratio() {
        let split = SplitTree::split_with_ratio(
            SplitOrientation::Vertical,
            0.3,
            SplitTree::leaf(PanelId::Layers),
            SplitTree::leaf(PanelId::Properties),
        );

        if let SplitTree::Split { ratio, .. } = split {
            assert!((ratio - 0.3).abs() < 1e-6);
        } else {
            panic!("Expected split tree");
        }
    }

    #[test]
    fn split_tree_ratio_clamped() {
        let split = SplitTree::split_with_ratio(
            SplitOrientation::Horizontal,
            1.5, // Should be clamped to 0.95
            SplitTree::leaf(PanelId::Thumbnails),
            SplitTree::leaf(PanelId::Layers),
        );

        if let SplitTree::Split { ratio, .. } = split {
            assert!((ratio - 0.95).abs() < 1e-6);
        } else {
            panic!("Expected split tree");
        }
    }

    #[test]
    fn split_tree_nested_depth() {
        // Create a nested split: (Thumbnails | Layers) vertically split from (Properties | Measurements)
        let left = SplitTree::split(
            SplitOrientation::Vertical,
            SplitTree::leaf(PanelId::Thumbnails),
            SplitTree::leaf(PanelId::Layers),
        );
        let right = SplitTree::split(
            SplitOrientation::Vertical,
            SplitTree::leaf(PanelId::Properties),
            SplitTree::leaf(PanelId::Measurements),
        );
        let nested = SplitTree::split(SplitOrientation::Horizontal, left, right);

        assert_eq!(nested.depth(), 2);
    }

    #[test]
    fn tab_group_single() {
        let group = TabGroup::single(PanelId::Thumbnails);
        assert_eq!(group.tabs.len(), 1);
        assert_eq!(group.active_index, 0);
        assert!(!group.can_close_tab());
    }

    #[test]
    fn tab_group_close_tab() {
        let mut group = TabGroup {
            tabs: vec![PanelId::Thumbnails, PanelId::Layers, PanelId::Properties],
            active_index: 1,
        };

        let closed = group.close_tab(1);
        assert_eq!(closed, Some(PanelId::Layers));
        assert_eq!(group.tabs, vec![PanelId::Thumbnails, PanelId::Properties]);
        assert_eq!(group.active_index, 1); // Should stay at Layers' position (now Properties)
    }

    #[test]
    fn tab_group_close_last_tab_fails() {
        let mut group = TabGroup::single(PanelId::Thumbnails);
        let closed = group.close_tab(0);
        assert!(closed.is_none());
    }

    #[test]
    fn floating_panel_clamp() {
        let mut fp = FloatingPanel::new(PanelId::Measurements);
        fp.position = [-100.0, -50.0];
        fp.size = [100.0, 50.0];
        fp.clamp();

        assert!(fp.position[0] >= 0.0);
        assert!(fp.position[1] >= 0.0);
        assert!(fp.size[0] >= 200.0);
        assert!(fp.size[1] >= 150.0);
    }

    #[test]
    fn panel_layout_default() {
        let layout = PanelLayout::default();
        assert!(layout.left.is_empty());
        assert!(layout.right.is_empty());
        assert!(layout.bottom.is_empty());
        assert_eq!(layout.strip_size_left, 200.0);
        assert_eq!(layout.strip_size_right, 260.0);
        assert_eq!(layout.strip_size_bottom, 200.0);
    }

    #[test]
    fn docked_panel_floating_check() {
        let docked = DockedPanel::new(PanelId::Properties);
        assert!(!docked.is_floating());

        let mut floating = DockedPanel::new(PanelId::MarkupsList);
        floating.dock_location = DockLocation::Floating;
        assert!(floating.is_floating());
    }

    #[test]
    fn serde_roundtrip_panel_layout() {
        let original = PanelLayout::default();
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: PanelLayout = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, original);
    }

    #[test]
    fn serde_roundtrip_split_tree() {
        let split = SplitTree::split(
            SplitOrientation::Horizontal,
            SplitTree::leaf(PanelId::Thumbnails),
            SplitTree::leaf(PanelId::Layers),
        );
        let json = serde_json::to_string(&split).expect("serialize");
        let restored: SplitTree = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, split);
    }

    #[test]
    fn serde_roundtrip_docked_panel() {
        let panel = DockedPanel::new(PanelId::Properties);
        let json = serde_json::to_string(&panel).expect("serialize");
        let restored: DockedPanel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, panel);
    }
}
