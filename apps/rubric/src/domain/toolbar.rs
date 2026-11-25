//! Toolbar customization domain types.
//!
//! Pure, serde-serialisable types for toolbar configuration, command registry,
//! and lock state. Used for profile integration and UI persistence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unique identifier for a command in the command registry.
///
/// String-based identifiers allow easy serialization and matching against
/// toolbar items while keeping the domain agnostic to specific UI types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandId(pub String);

impl CommandId {
    /// Create a new command identifier from a string.
    #[inline]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for CommandId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for CommandId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for CommandId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for CommandId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Command categories (7 total as per spec).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandCategory {
    /// Select, Pan, Zoom In/Out/Fit, Page First/Prev/Next/Last, Jump to Page
    SelectionNavigation,
    /// Highlight, Strikeout, Underline, Squiggly, Note, Text Box, Callout,
    /// Rectangle, Ellipse, Cloud, Polyline, Polygon, Line, Arrow, Ink
    MarkupTools,
    /// Length, Perimeter, Area, Angle, Diameter/Radius, Count, Calibrate Scale, Unit Selector
    MeasurementTools,
    /// Status Dropdown, Add Comment, Show Comments, Author Filter, Summary Report
    ReviewComment,
    /// Copy Properties, Paste Properties, Bring Forward, Send Backward, Bring to Front,
    /// Send to Back, Toggle Lock, Layer Visibility
    EditModify,
    /// Theme Toggle, Show/Hide Grid, Snap to Grid, Grid Spacing, Show/Hide Rulers,
    /// Full Screen, Panel Visibility
    ViewDisplay,
    /// New, Open, Save, Save As, Import PDF, Export Image, Print, Recent Files, Project Settings
    FileProject,
    /// Bates Numbering Config, Calibration Dialog, Preferences, Keyboard Shortcuts, Profiles Management
    Advanced,
}

impl CommandCategory {
    /// All categories in display order.
    pub const ALL: [CommandCategory; 8] = [
        CommandCategory::SelectionNavigation,
        CommandCategory::MarkupTools,
        CommandCategory::MeasurementTools,
        CommandCategory::ReviewComment,
        CommandCategory::EditModify,
        CommandCategory::ViewDisplay,
        CommandCategory::FileProject,
        CommandCategory::Advanced,
    ];

    /// Human-readable label for the category.
    pub fn label(self) -> &'static str {
        match self {
            CommandCategory::SelectionNavigation => "Selection & Navigation",
            CommandCategory::MarkupTools => "Markup Tools",
            CommandCategory::MeasurementTools => "Measurement Tools",
            CommandCategory::ReviewComment => "Review & Comment",
            CommandCategory::EditModify => "Edit & Modify",
            CommandCategory::ViewDisplay => "View & Display",
            CommandCategory::FileProject => "File & Project",
            CommandCategory::Advanced => "Advanced",
        }
    }
}

/// Information about a single command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandInfo {
    /// Unique identifier for this command.
    pub id: String,
    /// Human-readable name shown in UI.
    pub name: String,
    /// Icon name or shorthand (UI-specific).
    pub icon: String,
    /// Keyboard shortcut (e.g., "Ctrl+S", "V", "1").
    pub shortcut: String,
    /// Category this command belongs to.
    pub category: CommandCategory,
    /// Brief description of what the command does.
    pub description: String,
}

impl CommandInfo {
    /// Create a new command info entry.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        icon: impl Into<String>,
        shortcut: impl Into<String>,
        category: CommandCategory,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: icon.into(),
            shortcut: shortcut.into(),
            category,
            description: description.into(),
        }
    }
}

/// A single entry in a toolbar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolbarItem {
    /// A command from the command registry, identified by its string id.
    Tool(CommandId),
    /// A visual separator.
    Divider,
    /// Flexible spacer.
    Spacer,
}

impl ToolbarItem {
    /// Create a toolbar item from a command ID.
    pub fn tool(id: impl Into<String>) -> Self {
        ToolbarItem::Tool(CommandId::new(id))
    }

    /// Get the command ID if this item is a Tool variant.
    pub fn command_id(&self) -> Option<&CommandId> {
        match self {
            ToolbarItem::Tool(id) => Some(id),
            ToolbarItem::Divider | ToolbarItem::Spacer => None,
        }
    }
}

/// Visibility + configuration for one toolbar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolbarConfig {
    /// Stable id (e.g., "main", "quick_access", "properties", or custom name).
    pub id: String,
    /// Display name shown in the customization dialog.
    pub name: String,
    /// Whether this toolbar is predefined (not deletable/renamable).
    pub predefined: bool,
    /// Whether the toolbar is currently visible.
    pub visible: bool,
    /// Ordered items in the toolbar.
    pub items: Vec<ToolbarItem>,
    /// Lock state for this toolbar (overrides global lock when Some).
    pub lock_state: Option<bool>,
}

impl ToolbarConfig {
    /// Create a new toolbar configuration.
    pub fn new(id: impl Into<String>, name: impl Into<String>, predefined: bool) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            predefined,
            visible: true,
            items: Vec::new(),
            lock_state: None,
        }
    }

    /// Create the main toolbar (predefined).
    pub fn main() -> Self {
        Self::new("main", "Main Toolbar", true)
    }

    /// Create the quick access area (predefined).
    pub fn quick_access() -> Self {
        Self::new("quick_access", "Quick Access Area", true)
    }

    /// Create the properties toolbar (predefined).
    pub fn properties() -> Self {
        Self::new("properties", "Properties Toolbar", true)
    }
}

/// Configuration for the Quick Access Area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickAccessConfig {
    /// Commands in the quick access area (5-12 items).
    pub commands: Vec<CommandId>,
}

impl QuickAccessConfig {
    /// Create a new quick access config with the given commands.
    pub fn new(commands: Vec<CommandId>) -> Self {
        Self { commands }
    }

    /// Maximum number of tools allowed (flexible in spec, but 12 is typical max).
    pub const MAX_TOOLS: usize = 12;

    /// Minimum number of tools (spec says 5-12, but this is a soft minimum).
    pub const MIN_TOOLS: usize = 0;

    /// Check if this config is valid (command count within limits).
    pub fn is_valid(&self) -> bool {
        self.commands.len() <= Self::MAX_TOOLS
    }
}

/// Global lock state for all toolbars.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ToolbarLockState {
    /// Whether toolbars are globally locked.
    pub global_locked: bool,
    /// Per-custom-toolbar lock overrides: key = toolbar id, value = locked.
    /// When `true`, the toolbar is locked even if global lock is off.
    /// When `false`, the toolbar is unlocked even if global lock is on.
    pub custom_overrides: HashMap<String, bool>,
}

impl ToolbarLockState {
    /// Create a new lock state (unlocked by default).
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a specific toolbar is locked.
    pub fn is_locked(&self, toolbar_id: &str) -> bool {
        if !self.global_locked {
            return false;
        }
        *self.custom_overrides.get(toolbar_id).unwrap_or(&true)
    }

    /// Set global lock state.
    pub fn set_global_locked(&mut self, locked: bool) {
        self.global_locked = locked;
    }

    /// Set per-toolbar override.
    pub fn set_override(&mut self, toolbar_id: String, locked: bool) {
        self.custom_overrides.insert(toolbar_id, locked);
    }
}

/// Profile-level toolbar configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ToolbarProfileConfig {
    /// Main toolbar configuration.
    pub main_toolbar: ToolbarConfig,
    /// Quick access configuration.
    pub quick_access: QuickAccessConfig,
    /// Properties toolbar configuration.
    pub properties_toolbar: ToolbarConfig,
    /// Custom toolbars.
    pub custom_toolbars: Vec<ToolbarConfig>,
    /// Lock state (persisted per profile).
    pub lock_state: ToolbarLockState,
}

impl ToolbarProfileConfig {
    /// Create a new profile toolbar config with defaults.
    pub fn new() -> Self {
        Self {
            main_toolbar: ToolbarConfig::main(),
            quick_access: QuickAccessConfig::new(Vec::new()),
            properties_toolbar: ToolbarConfig::properties(),
            custom_toolbars: Vec::new(),
            lock_state: ToolbarLockState::new(),
        }
    }
}

impl Default for ToolbarConfig {
    fn default() -> Self {
        Self::new("main", "Main Toolbar", true)
    }
}

impl Default for QuickAccessConfig {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_id_new_and_from_str() {
        let id = CommandId::new("select");
        assert_eq!(id.0, "select");

        let id2: CommandId = "pan".into();
        assert_eq!(id2.0, "pan");
    }

    #[test]
    fn command_id_display() {
        let id = CommandId::new("highlight");
        assert_eq!(format!("{}", id), "highlight");
    }

    #[test]
    fn command_id_as_ref() {
        let id = CommandId::new("length");
        assert_eq!(id.as_ref(), "length");
    }

    #[test]
    fn command_category_labels() {
        assert_eq!(
            CommandCategory::SelectionNavigation.label(),
            "Selection & Navigation"
        );
        assert_eq!(CommandCategory::MarkupTools.label(), "Markup Tools");
        assert_eq!(
            CommandCategory::MeasurementTools.label(),
            "Measurement Tools"
        );
        assert_eq!(CommandCategory::ReviewComment.label(), "Review & Comment");
        assert_eq!(CommandCategory::EditModify.label(), "Edit & Modify");
        assert_eq!(CommandCategory::ViewDisplay.label(), "View & Display");
        assert_eq!(CommandCategory::FileProject.label(), "File & Project");
        assert_eq!(CommandCategory::Advanced.label(), "Advanced");
    }

    #[test]
    fn command_category_all_count() {
        assert_eq!(CommandCategory::ALL.len(), 8);
    }

    #[test]
    fn command_info_creation() {
        let info = CommandInfo::new(
            "test_cmd",
            "Test Command",
            "test",
            "T",
            CommandCategory::MarkupTools,
            "Test description",
        );
        assert_eq!(info.id, "test_cmd");
        assert_eq!(info.name, "Test Command");
        assert_eq!(info.icon, "test");
        assert_eq!(info.shortcut, "T");
        assert_eq!(info.category, CommandCategory::MarkupTools);
        assert_eq!(info.description, "Test description");
    }

    #[test]
    fn toolbar_item_tool_creation() {
        let item = ToolbarItem::tool("zoom_in");
        assert!(matches!(item, ToolbarItem::Tool(ref id) if id.0 == "zoom_in"));

        let item2 = ToolbarItem::Divider;
        assert!(matches!(item2, ToolbarItem::Divider));

        let item3 = ToolbarItem::Spacer;
        assert!(matches!(item3, ToolbarItem::Spacer));
    }

    #[test]
    fn toolbar_config_creation() {
        let config = ToolbarConfig::main();
        assert_eq!(config.id, "main");
        assert_eq!(config.name, "Main Toolbar");
        assert!(config.predefined);
        assert!(config.visible);
        assert!(config.items.is_empty());
        assert!(config.lock_state.is_none());

        let custom = ToolbarConfig::new("custom_toolbar", "My Tools", false);
        assert_eq!(custom.id, "custom_toolbar");
        assert!(!custom.predefined);
    }

    #[test]
    fn toolbar_config_default() {
        let config: ToolbarConfig = Default::default();
        assert_eq!(config.id, "main");
        assert_eq!(config.name, "Main Toolbar");
        assert!(config.predefined);
    }

    #[test]
        #[test]
    fn quick_access_config_validation() {
        let valid = QuickAccessConfig::new(
            vec![
                "select",
                "pan",
                "zoom_in",
                "zoom_out",
                "zoom_fit",
                "page_first",
                "page_prev",
                "page_next",
                "page_last",
                "jump_to_page",
                "highlight",
                "strikeout",
            ]
            .into_iter()
            .map(CommandId::new)
            .collect(),
        );
        assert!(valid.is_valid());

        let invalid = QuickAccessConfig::new(
            vec![
                "select",
                "pan",
                "zoom_in",
                "zoom_out",
                "zoom_fit",
                "page_first",
                "page_prev",
                "page_next",
                "page_last",
                "jump_to_page",
                "highlight",
                "strikeout",
                "underline",
            ]
            .into_iter()
            .map(CommandId::new)
            .collect(),
        );
        assert!(!invalid.is_valid());
    }    }

    #[test]
    fn toolbar_lock_state_behavior() {
        let mut lock_state = ToolbarLockState::new();

        // Initially unlocked
        assert!(!lock_state.is_locked("main"));

        // After global lock
        lock_state.set_global_locked(true);
        assert!(lock_state.is_locked("main"));

        // Override unlocks specific toolbar
        lock_state.set_override("custom1".to_string(), false);
        assert!(!lock_state.is_locked("custom1"));

        // Override keeps it locked
        lock_state.set_override("custom2".to_string(), true);
        assert!(lock_state.is_locked("custom2"));
    }

    #[test]
    fn toolbar_profile_config_new() {
        let config = ToolbarProfileConfig::new();
        assert_eq!(config.main_toolbar.id, "main");
        assert_eq!(config.quick_access.commands.len(), 0);
        assert_eq!(config.properties_toolbar.id, "properties");
        assert!(config.custom_toolbars.is_empty());
    }

    #[test]
    fn toolbar_profile_config_default() {
        let config: ToolbarProfileConfig = Default::default();
        assert!(config.main_toolbar.predefined);
    }

    #[test]
    fn toolbar_config_serde_roundtrip() {
        let mut config = ToolbarConfig::new("test_toolbar", "Test", false);
        config.visible = true;
        config.items.push(ToolbarItem::tool("highlight"));
        config.items.push(ToolbarItem::Divider);
        config.items.push(ToolbarItem::Spacer);

        let json = serde_json::to_string(&config).expect("serialize");
        let restored: ToolbarConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, config);
    }

    #[test]
    fn toolbar_item_serde_roundtrip() {
        for item in [
            ToolbarItem::Tool(CommandId::new("select")),
            ToolbarItem::Divider,
            ToolbarItem::Spacer,
        ] {
            let json = serde_json::to_string(&item).expect("serialize");
            let restored: ToolbarItem = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(restored, item);
        }
    }

    #[test]
    fn quick_access_config_serde_roundtrip() {
        let config = QuickAccessConfig::new(vec![
            CommandId::new("highlight"),
            CommandId::new("length"),
            CommandId::new("area"),
        ]);

        let json = serde_json::to_string(&config).expect("serialize");
        let restored: QuickAccessConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, config);
    }

    #[test]
    fn toolbar_lock_state_serde_roundtrip() {
        let mut state = ToolbarLockState::new();
        state.global_locked = true;
        state.custom_overrides.insert("custom1".to_string(), false);

        let json = serde_json::to_string(&state).expect("serialize");
        let restored: ToolbarLockState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, state);
    }

    #[test]
    fn toolbar_profile_config_serde_roundtrip() {
        let config = ToolbarProfileConfig::new();
        let json = serde_json::to_string(&config).expect("serialize");
        let restored: ToolbarProfileConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, config);
    }
}
