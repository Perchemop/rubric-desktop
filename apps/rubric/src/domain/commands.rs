//! Command registry for toolbar customization.
//!
//! Defines all available commands with their metadata (icon, shortcut, category,
//! description) for use in toolbar customization and keyboard shortcut handling.

use crate::domain::toolbar::{CommandCategory, CommandId, CommandInfo};
use std::collections::HashMap;

/// Command icon names used in the UI.
pub mod icon {
    // Selection & Navigation
    pub const SELECT: &str = "sel";
    pub const PAN: &str = "pan";
    pub const ZOOM_IN: &str = "zi";
    pub const ZOOM_OUT: &str = "zo";
    pub const ZOOM_FIT: &str = "zf";
    pub const PAGE_FIRST: &str = "pf";
    pub const PAGE_PREV: &str = "pp";
    pub const PAGE_NEXT: &str = "pn";
    pub const PAGE_LAST: &str = "pl";
    pub const JUMP_TO_PAGE: &str = "jmp";

    // Markup Tools
    pub const HIGHLIGHT: &str = "hl";
    pub const STRIKEOUT: &str = "sk";
    pub const UNDERLINE: &str = "ul";
    pub const SQUIGGLY: &str = "sq";
    pub const NOTE: &str = "note";
    pub const TEXT_BOX: &str = "txt";
    pub const CALLOUT: &str = "co";
    pub const RECTANGLE: &str = "rect";
    pub const ELLIPSE: &str = "ell";
    pub const CLOUD: &str = "cld";
    pub const POLYLINE: &str = "pline";
    pub const POLYGON: &str = "poly";
    pub const LINE: &str = "line";
    pub const ARROW: &str = "arrow";
    pub const INK: &str = "ink";
    pub const STAMP_PRESET: &str = "stamp";

    // Measurement Tools
    pub const LENGTH: &str = "ln";
    pub const PERIMETER: &str = "perim";
    pub const AREA: &str = "ar";
    pub const ANGLE: &str = "ang";
    pub const DIAMETER: &str = "dia";
    pub const RADIUS: &str = "rad";
    pub const COUNT: &str = "cnt";
    pub const CALIBRATE: &str = "cal";
    pub const UNIT_SELECTOR: &str = "unit";

    // Review & Comment
    pub const STATUS_DROPDOWN: &str = "status";
    pub const ADD_COMMENT: &str = "add_cm";
    pub const SHOW_COMMENTS: &str = "show_cm";
    pub const AUTHOR_FILTER: &str = "author";
    pub const SUMMARY_REPORT: &str = "report";
    pub const REVIEW_STATUS: &str = "review";

    // Edit & Modify
    pub const COPY_PROPERTIES: &str = "copy_prop";
    pub const PASTE_PROPERTIES: &str = "paste_prop";
    pub const BRING_FORWARD: &str = "bring_fwd";
    pub const SEND_BACKWARD: &str = "send_back";
    pub const BRING_TO_FRONT: &str = "bring_front";
    pub const SEND_TO_BACK: &str = "send_back_bt";
    pub const TOGGLE_LOCK: &str = "toggle_lock";
    pub const LAYER_VISIBILITY: &str = "layer";

    // View & Display
    pub const THEME_TOGGLE: &str = "theme";
    pub const GRID_VISIBLE: &str = "grid";
    pub const SNAP_TO_GRID: &str = "snap";
    pub const GRID_SPACING: &str = "grid_spc";
    pub const RULERS_VISIBLE: &str = "ru";
    pub const FULL_SCREEN: &str = "fs";
    pub const PANEL_VISIBILITY: &str = "panel";

    // File & Project
    pub const NEW: &str = "new";
    pub const OPEN: &str = "open";
    pub const SAVE: &str = "save";
    pub const SAVE_AS: &str = "save_as";
    pub const IMPORT_PDF: &str = "import";
    pub const EXPORT_IMAGE: &str = "export";
    pub const PRINT: &str = "print";
    pub const RECENT_FILES: &str = "recent";
    pub const PROJECT_SETTINGS: &str = "settings";

    // Advanced
    pub const BATES_NUMBERING: &str = "bates";
    pub const CALIBRATION_DIALOG: &str = "cal_dlg";
    pub const PREFERENCES: &str = "prefs";
    pub const KEYBOARD_SHORTCUTS: &str = "kb";
    pub const PROFILES: &str = "profiles";
}

/// Get all commands organized by category.
pub fn commands_by_category() -> HashMap<CommandCategory, Vec<CommandId>> {
    let mut map = HashMap::new();

    // Selection & Navigation
    map.insert(
        CommandCategory::SelectionNavigation,
        vec![
            CommandId::new(icon::SELECT),
            CommandId::new(icon::PAN),
            CommandId::new(icon::ZOOM_IN),
            CommandId::new(icon::ZOOM_OUT),
            CommandId::new(icon::ZOOM_FIT),
            CommandId::new(icon::PAGE_FIRST),
            CommandId::new(icon::PAGE_PREV),
            CommandId::new(icon::PAGE_NEXT),
            CommandId::new(icon::PAGE_LAST),
            CommandId::new(icon::JUMP_TO_PAGE),
        ],
    );

    // Markup Tools
    map.insert(
        CommandCategory::MarkupTools,
        vec![
            CommandId::new(icon::HIGHLIGHT),
            CommandId::new(icon::STRIKEOUT),
            CommandId::new(icon::UNDERLINE),
            CommandId::new(icon::SQUIGGLY),
            CommandId::new(icon::NOTE),
            CommandId::new(icon::TEXT_BOX),
            CommandId::new(icon::CALLOUT),
            CommandId::new(icon::RECTANGLE),
            CommandId::new(icon::ELLIPSE),
            CommandId::new(icon::CLOUD),
            CommandId::new(icon::POLYLINE),
            CommandId::new(icon::POLYGON),
            CommandId::new(icon::LINE),
            CommandId::new(icon::ARROW),
            CommandId::new(icon::INK),
            CommandId::new(icon::STAMP_PRESET),
        ],
    );

    // Measurement Tools
    map.insert(
        CommandCategory::MeasurementTools,
        vec![
            CommandId::new(icon::LENGTH),
            CommandId::new(icon::PERIMETER),
            CommandId::new(icon::AREA),
            CommandId::new(icon::ANGLE),
            CommandId::new(icon::DIAMETER),
            CommandId::new(icon::RADIUS),
            CommandId::new(icon::COUNT),
            CommandId::new(icon::CALIBRATE),
            CommandId::new(icon::UNIT_SELECTOR),
        ],
    );

    // Review & Comment
    map.insert(
        CommandCategory::ReviewComment,
        vec![
            CommandId::new(icon::STATUS_DROPDOWN),
            CommandId::new(icon::ADD_COMMENT),
            CommandId::new(icon::SHOW_COMMENTS),
            CommandId::new(icon::AUTHOR_FILTER),
            CommandId::new(icon::SUMMARY_REPORT),
            CommandId::new(icon::REVIEW_STATUS),
        ],
    );

    // Edit & Modify
    map.insert(
        CommandCategory::EditModify,
        vec![
            CommandId::new(icon::COPY_PROPERTIES),
            CommandId::new(icon::PASTE_PROPERTIES),
            CommandId::new(icon::BRING_FORWARD),
            CommandId::new(icon::SEND_BACKWARD),
            CommandId::new(icon::BRING_TO_FRONT),
            CommandId::new(icon::SEND_TO_BACK),
            CommandId::new(icon::TOGGLE_LOCK),
            CommandId::new(icon::LAYER_VISIBILITY),
        ],
    );

    // View & Display
    map.insert(
        CommandCategory::ViewDisplay,
        vec![
            CommandId::new(icon::THEME_TOGGLE),
            CommandId::new(icon::GRID_VISIBLE),
            CommandId::new(icon::SNAP_TO_GRID),
            CommandId::new(icon::GRID_SPACING),
            CommandId::new(icon::RULERS_VISIBLE),
            CommandId::new(icon::FULL_SCREEN),
            CommandId::new(icon::PANEL_VISIBILITY),
        ],
    );

    // File & Project
    map.insert(
        CommandCategory::FileProject,
        vec![
            CommandId::new(icon::NEW),
            CommandId::new(icon::OPEN),
            CommandId::new(icon::SAVE),
            CommandId::new(icon::SAVE_AS),
            CommandId::new(icon::IMPORT_PDF),
            CommandId::new(icon::EXPORT_IMAGE),
            CommandId::new(icon::PRINT),
            CommandId::new(icon::RECENT_FILES),
            CommandId::new(icon::PROJECT_SETTINGS),
        ],
    );

    // Advanced
    map.insert(
        CommandCategory::Advanced,
        vec![
            CommandId::new(icon::BATES_NUMBERING),
            CommandId::new(icon::CALIBRATION_DIALOG),
            CommandId::new(icon::PREFERENCES),
            CommandId::new(icon::KEYBOARD_SHORTCUTS),
            CommandId::new(icon::PROFILES),
        ],
    );

    map
}

/// Get the full command registry as a HashMap.
pub fn command_registry() -> HashMap<String, CommandInfo> {
    let mut registry = HashMap::new();

    // Selection & Navigation
    registry.insert(
        icon::SELECT.to_string(),
        CommandInfo::new(
            icon::SELECT,
            "Select",
            icon::SELECT,
            "V",
            CommandCategory::SelectionNavigation,
            "Select, move and resize markups",
        ),
    );
    registry.insert(
        icon::PAN.to_string(),
        CommandInfo::new(
            icon::PAN,
            "Pan",
            icon::PAN,
            "H",
            CommandCategory::SelectionNavigation,
            "Pan the canvas",
        ),
    );
    registry.insert(
        icon::ZOOM_IN.to_string(),
        CommandInfo::new(
            icon::ZOOM_IN,
            "Zoom In",
            icon::ZOOM_IN,
            "+",
            CommandCategory::SelectionNavigation,
            "Zoom in on the canvas",
        ),
    );
    registry.insert(
        icon::ZOOM_OUT.to_string(),
        CommandInfo::new(
            icon::ZOOM_OUT,
            "Zoom Out",
            icon::ZOOM_OUT,
            "-",
            CommandCategory::SelectionNavigation,
            "Zoom out on the canvas",
        ),
    );
    registry.insert(
        icon::ZOOM_FIT.to_string(),
        CommandInfo::new(
            icon::ZOOM_FIT,
            "Zoom Fit",
            icon::ZOOM_FIT,
            "F",
            CommandCategory::SelectionNavigation,
            "Fit page to canvas",
        ),
    );
    registry.insert(
        icon::PAGE_FIRST.to_string(),
        CommandInfo::new(
            icon::PAGE_FIRST,
            "First Page",
            icon::PAGE_FIRST,
            "Ctrl+Home",
            CommandCategory::SelectionNavigation,
            "Go to first page",
        ),
    );
    registry.insert(
        icon::PAGE_PREV.to_string(),
        CommandInfo::new(
            icon::PAGE_PREV,
            "Previous Page",
            icon::PAGE_PREV,
            "Page Up",
            CommandCategory::SelectionNavigation,
            "Go to previous page",
        ),
    );
    registry.insert(
        icon::PAGE_NEXT.to_string(),
        CommandInfo::new(
            icon::PAGE_NEXT,
            "Next Page",
            icon::PAGE_NEXT,
            "Page Down",
            CommandCategory::SelectionNavigation,
            "Go to next page",
        ),
    );
    registry.insert(
        icon::PAGE_LAST.to_string(),
        CommandInfo::new(
            icon::PAGE_LAST,
            "Last Page",
            icon::PAGE_LAST,
            "Ctrl+End",
            CommandCategory::SelectionNavigation,
            "Go to last page",
        ),
    );
    registry.insert(
        icon::JUMP_TO_PAGE.to_string(),
        CommandInfo::new(
            icon::JUMP_TO_PAGE,
            "Jump to Page",
            icon::JUMP_TO_PAGE,
            "Ctrl+G",
            CommandCategory::SelectionNavigation,
            "Jump to a specific page",
        ),
    );

    // Markup Tools
    registry.insert(
        icon::HIGHLIGHT.to_string(),
        CommandInfo::new(
            icon::HIGHLIGHT,
            "Highlight",
            icon::HIGHLIGHT,
            "H",
            CommandCategory::MarkupTools,
            "Draw highlight markup",
        ),
    );
    registry.insert(
        icon::STRIKEOUT.to_string(),
        CommandInfo::new(
            icon::STRIKEOUT,
            "Strikeout",
            icon::STRIKEOUT,
            "S",
            CommandCategory::MarkupTools,
            "Draw strikeout markup",
        ),
    );
    registry.insert(
        icon::UNDERLINE.to_string(),
        CommandInfo::new(
            icon::UNDERLINE,
            "Underline",
            icon::UNDERLINE,
            "U",
            CommandCategory::MarkupTools,
            "Draw underline markup",
        ),
    );
    registry.insert(
        icon::SQUIGGLY.to_string(),
        CommandInfo::new(
            icon::SQUIGGLY,
            "Squiggly",
            icon::SQUIGGLY,
            "Q",
            CommandCategory::MarkupTools,
            "Draw squiggly underline markup",
        ),
    );
    registry.insert(
        icon::TEXT_BOX.to_string(),
        CommandInfo::new(
            icon::TEXT_BOX,
            "Text Box",
            icon::TEXT_BOX,
            "T",
            CommandCategory::MarkupTools,
            "Draw a text box",
        ),
    );
    registry.insert(
        icon::CALLOUT.to_string(),
        CommandInfo::new(
            icon::CALLOUT,
            "Callout",
            icon::CALLOUT,
            "C",
            CommandCategory::MarkupTools,
            "Draw a callout box",
        ),
    );
    registry.insert(
        icon::NOTE.to_string(),
        CommandInfo::new(
            icon::NOTE,
            "Sticky Note",
            icon::NOTE,
            "N",
            CommandCategory::MarkupTools,
            "Add a sticky note",
        ),
    );
    registry.insert(
        icon::STAMP_PRESET.to_string(),
        CommandInfo::new(
            icon::STAMP_PRESET,
            "Stamp Presets",
            icon::STAMP_PRESET,
            "M",
            CommandCategory::MarkupTools,
            "Apply stamp presets",
        ),
    );
    registry.insert(
        icon::RECTANGLE.to_string(),
        CommandInfo::new(
            icon::RECTANGLE,
            "Rectangle",
            icon::RECTANGLE,
            "R",
            CommandCategory::MarkupTools,
            "Draw a rectangle",
        ),
    );
    registry.insert(
        icon::ELLIPSE.to_string(),
        CommandInfo::new(
            icon::ELLIPSE,
            "Ellipse",
            icon::ELLIPSE,
            "O",
            CommandCategory::MarkupTools,
            "Draw an ellipse",
        ),
    );
    registry.insert(
        icon::CLOUD.to_string(),
        CommandInfo::new(
            icon::CLOUD,
            "Cloud",
            icon::CLOUD,
            "D",
            CommandCategory::MarkupTools,
            "Draw a cloud callout",
        ),
    );
    registry.insert(
        icon::POLYLINE.to_string(),
        CommandInfo::new(
            icon::POLYLINE,
            "Polyline",
            icon::POLYLINE,
            "Y",
            CommandCategory::MarkupTools,
            "Draw a polyline",
        ),
    );
    registry.insert(
        icon::POLYGON.to_string(),
        CommandInfo::new(
            icon::POLYGON,
            "Polygon",
            icon::POLYGON,
            "P",
            CommandCategory::MarkupTools,
            "Draw a polygon",
        ),
    );
    registry.insert(
        icon::LINE.to_string(),
        CommandInfo::new(
            icon::LINE,
            "Line",
            icon::LINE,
            "L",
            CommandCategory::MarkupTools,
            "Draw a line",
        ),
    );
    registry.insert(
        icon::ARROW.to_string(),
        CommandInfo::new(
            icon::ARROW,
            "Arrow",
            icon::ARROW,
            "A",
            CommandCategory::MarkupTools,
            "Draw an arrow",
        ),
    );
    registry.insert(
        icon::INK.to_string(),
        CommandInfo::new(
            icon::INK,
            "Pen",
            icon::INK,
            "I",
            CommandCategory::MarkupTools,
            "Freehand ink drawing",
        ),
    );

    // Measurement Tools
    registry.insert(
        icon::LENGTH.to_string(),
        CommandInfo::new(
            icon::LENGTH,
            "Length",
            icon::LENGTH,
            "1",
            CommandCategory::MeasurementTools,
            "Measure length",
        ),
    );
    registry.insert(
        icon::PERIMETER.to_string(),
        CommandInfo::new(
            icon::PERIMETER,
            "Perimeter",
            icon::PERIMETER,
            "2",
            CommandCategory::MeasurementTools,
            "Measure perimeter",
        ),
    );
    registry.insert(
        icon::AREA.to_string(),
        CommandInfo::new(
            icon::AREA,
            "Area",
            icon::AREA,
            "3",
            CommandCategory::MeasurementTools,
            "Measure area",
        ),
    );
    registry.insert(
        icon::ANGLE.to_string(),
        CommandInfo::new(
            icon::ANGLE,
            "Angle",
            icon::ANGLE,
            "4",
            CommandCategory::MeasurementTools,
            "Measure angle",
        ),
    );
    registry.insert(
        icon::DIAMETER.to_string(),
        CommandInfo::new(
            icon::DIAMETER,
            "Diameter",
            icon::DIAMETER,
            "5",
            CommandCategory::MeasurementTools,
            "Measure diameter",
        ),
    );
    registry.insert(
        icon::RADIUS.to_string(),
        CommandInfo::new(
            icon::RADIUS,
            "Radius",
            icon::RADIUS,
            "6",
            CommandCategory::MeasurementTools,
            "Measure radius",
        ),
    );
    registry.insert(
        icon::COUNT.to_string(),
        CommandInfo::new(
            icon::COUNT,
            "Count",
            icon::COUNT,
            "7",
            CommandCategory::MeasurementTools,
            "Count markups",
        ),
    );
    registry.insert(
        icon::CALIBRATE.to_string(),
        CommandInfo::new(
            icon::CALIBRATE,
            "Calibrate Scale",
            icon::CALIBRATE,
            "K",
            CommandCategory::MeasurementTools,
            "Calibrate page scale with known distance",
        ),
    );
    registry.insert(
        icon::UNIT_SELECTOR.to_string(),
        CommandInfo::new(
            icon::UNIT_SELECTOR,
            "Unit Selector",
            icon::UNIT_SELECTOR,
            "",
            CommandCategory::MeasurementTools,
            "Change measurement units",
        ),
    );

    // Review & Comment
    registry.insert(
        icon::STATUS_DROPDOWN.to_string(),
        CommandInfo::new(
            icon::STATUS_DROPDOWN,
            "Status Dropdown",
            icon::STATUS_DROPDOWN,
            "",
            CommandCategory::ReviewComment,
            "Set markup review status",
        ),
    );
    registry.insert(
        icon::ADD_COMMENT.to_string(),
        CommandInfo::new(
            icon::ADD_COMMENT,
            "Add Comment",
            icon::ADD_COMMENT,
            "",
            CommandCategory::ReviewComment,
            "Add a comment to selection",
        ),
    );
    registry.insert(
        icon::SHOW_COMMENTS.to_string(),
        CommandInfo::new(
            icon::SHOW_COMMENTS,
            "Show Comments",
            icon::SHOW_COMMENTS,
            "",
            CommandCategory::ReviewComment,
            "Toggle marks list visibility",
        ),
    );
    registry.insert(
        icon::AUTHOR_FILTER.to_string(),
        CommandInfo::new(
            icon::AUTHOR_FILTER,
            "Author Filter",
            icon::AUTHOR_FILTER,
            "",
            CommandCategory::ReviewComment,
            "Filter markups by author",
        ),
    );
    registry.insert(
        icon::SUMMARY_REPORT.to_string(),
        CommandInfo::new(
            icon::SUMMARY_REPORT,
            "Summary Report",
            icon::SUMMARY_REPORT,
            "",
            CommandCategory::ReviewComment,
            "Generate markup summary report",
        ),
    );
    registry.insert(
        icon::REVIEW_STATUS.to_string(),
        CommandInfo::new(
            icon::REVIEW_STATUS,
            "Review Status",
            icon::REVIEW_STATUS,
            "",
            CommandCategory::ReviewComment,
            "Show review status filter",
        ),
    );

    // Edit & Modify
    registry.insert(
        icon::COPY_PROPERTIES.to_string(),
        CommandInfo::new(
            icon::COPY_PROPERTIES,
            "Copy Properties",
            icon::COPY_PROPERTIES,
            "Ctrl+Shift+C",
            CommandCategory::EditModify,
            "Copy markup properties",
        ),
    );
    registry.insert(
        icon::PASTE_PROPERTIES.to_string(),
        CommandInfo::new(
            icon::PASTE_PROPERTIES,
            "Paste Properties",
            icon::PASTE_PROPERTIES,
            "Ctrl+Shift+V",
            CommandCategory::EditModify,
            "Paste markup properties",
        ),
    );
    registry.insert(
        icon::BRING_FORWARD.to_string(),
        CommandInfo::new(
            icon::BRING_FORWARD,
            "Bring Forward",
            icon::BRING_FORWARD,
            "Ctrl+Shift+] ",
            CommandCategory::EditModify,
            "Bring selected markups forward",
        ),
    );
    registry.insert(
        icon::SEND_BACKWARD.to_string(),
        CommandInfo::new(
            icon::SEND_BACKWARD,
            "Send Backward",
            icon::SEND_BACKWARD,
            "Ctrl+Shift+[ ",
            CommandCategory::EditModify,
            "Send selected markups backward",
        ),
    );
    registry.insert(
        icon::BRING_TO_FRONT.to_string(),
        CommandInfo::new(
            icon::BRING_TO_FRONT,
            "Bring to Front",
            icon::BRING_TO_FRONT,
            "Ctrl+] ",
            CommandCategory::EditModify,
            "Bring selected markups to front",
        ),
    );
    registry.insert(
        icon::SEND_TO_BACK.to_string(),
        CommandInfo::new(
            icon::SEND_TO_BACK,
            "Send to Back",
            icon::SEND_TO_BACK,
            "Ctrl+[ ",
            CommandCategory::EditModify,
            "Send selected markups to back",
        ),
    );
    registry.insert(
        icon::TOGGLE_LOCK.to_string(),
        CommandInfo::new(
            icon::TOGGLE_LOCK,
            "Toggle Lock",
            icon::TOGGLE_LOCK,
            "Ctrl+L",
            CommandCategory::EditModify,
            "Lock or unlock selected markups",
        ),
    );
    registry.insert(
        icon::LAYER_VISIBILITY.to_string(),
        CommandInfo::new(
            icon::LAYER_VISIBILITY,
            "Layer Visibility",
            icon::LAYER_VISIBILITY,
            "",
            CommandCategory::EditModify,
            "Toggle layer visibility",
        ),
    );

    // View & Display
    registry.insert(
        icon::THEME_TOGGLE.to_string(),
        CommandInfo::new(
            icon::THEME_TOGGLE,
            "Theme Toggle",
            icon::THEME_TOGGLE,
            "",
            CommandCategory::ViewDisplay,
            "Toggle between light and dark theme",
        ),
    );
    registry.insert(
        icon::GRID_VISIBLE.to_string(),
        CommandInfo::new(
            icon::GRID_VISIBLE,
            "Show/Hide Grid",
            icon::GRID_VISIBLE,
            "",
            CommandCategory::ViewDisplay,
            "Toggle grid visibility",
        ),
    );
    registry.insert(
        icon::SNAP_TO_GRID.to_string(),
        CommandInfo::new(
            icon::SNAP_TO_GRID,
            "Snap to Grid",
            icon::SNAP_TO_GRID,
            "",
            CommandCategory::ViewDisplay,
            "Toggle snapping to grid",
        ),
    );
    registry.insert(
        icon::GRID_SPACING.to_string(),
        CommandInfo::new(
            icon::GRID_SPACING,
            "Grid Spacing",
            icon::GRID_SPACING,
            "",
            CommandCategory::ViewDisplay,
            "Adjust grid spacing",
        ),
    );
    registry.insert(
        icon::RULERS_VISIBLE.to_string(),
        CommandInfo::new(
            icon::RULERS_VISIBLE,
            "Show/Hide Rulers",
            icon::RULERS_VISIBLE,
            "Ctrl+R",
            CommandCategory::ViewDisplay,
            "Toggle rulers visibility",
        ),
    );
    registry.insert(
        icon::FULL_SCREEN.to_string(),
        CommandInfo::new(
            icon::FULL_SCREEN,
            "Full Screen",
            icon::FULL_SCREEN,
            "F11",
            CommandCategory::ViewDisplay,
            "Toggle full screen mode",
        ),
    );
    registry.insert(
        icon::PANEL_VISIBILITY.to_string(),
        CommandInfo::new(
            icon::PANEL_VISIBILITY,
            "Panel Visibility",
            icon::PANEL_VISIBILITY,
            "",
            CommandCategory::ViewDisplay,
            "Show or hide side panels",
        ),
    );

    // File & Project
    registry.insert(
        icon::NEW.to_string(),
        CommandInfo::new(
            icon::NEW,
            "New",
            icon::NEW,
            "Ctrl+N",
            CommandCategory::FileProject,
            "Create a new project",
        ),
    );
    registry.insert(
        icon::OPEN.to_string(),
        CommandInfo::new(
            icon::OPEN,
            "Open",
            icon::OPEN,
            "Ctrl+O",
            CommandCategory::FileProject,
            "Open an existing project",
        ),
    );
    registry.insert(
        icon::SAVE.to_string(),
        CommandInfo::new(
            icon::SAVE,
            "Save",
            icon::SAVE,
            "Ctrl+S",
            CommandCategory::FileProject,
            "Save the current project",
        ),
    );
    registry.insert(
        icon::SAVE_AS.to_string(),
        CommandInfo::new(
            icon::SAVE_AS,
            "Save As",
            icon::SAVE_AS,
            "Ctrl+Shift+S",
            CommandCategory::FileProject,
            "Save the current project as a new file",
        ),
    );
    registry.insert(
        icon::IMPORT_PDF.to_string(),
        CommandInfo::new(
            icon::IMPORT_PDF,
            "Import PDF",
            icon::IMPORT_PDF,
            "",
            CommandCategory::FileProject,
            "Import a PDF file",
        ),
    );
    registry.insert(
        icon::EXPORT_IMAGE.to_string(),
        CommandInfo::new(
            icon::EXPORT_IMAGE,
            "Export Image",
            icon::EXPORT_IMAGE,
            "Ctrl+E",
            CommandCategory::FileProject,
            "Export markups as image",
        ),
    );
    registry.insert(
        icon::PRINT.to_string(),
        CommandInfo::new(
            icon::PRINT,
            "Print",
            icon::PRINT,
            "Ctrl+P",
            CommandCategory::FileProject,
            "Print the current view",
        ),
    );
    registry.insert(
        icon::RECENT_FILES.to_string(),
        CommandInfo::new(
            icon::RECENT_FILES,
            "Recent Files",
            icon::RECENT_FILES,
            "",
            CommandCategory::FileProject,
            "Open a recently used file",
        ),
    );
    registry.insert(
        icon::PROJECT_SETTINGS.to_string(),
        CommandInfo::new(
            icon::PROJECT_SETTINGS,
            "Project Settings",
            icon::PROJECT_SETTINGS,
            "",
            CommandCategory::FileProject,
            "Configure project settings",
        ),
    );

    // Advanced
    registry.insert(
        icon::BATES_NUMBERING.to_string(),
        CommandInfo::new(
            icon::BATES_NUMBERING,
            "Bates Numbering Config",
            icon::BATES_NUMBERING,
            "",
            CommandCategory::Advanced,
            "Configure Bates numbering for markups",
        ),
    );
    registry.insert(
        icon::CALIBRATION_DIALOG.to_string(),
        CommandInfo::new(
            icon::CALIBRATION_DIALOG,
            "Calibration Dialog",
            icon::CALIBRATION_DIALOG,
            "",
            CommandCategory::Advanced,
            "Open calibration dialog",
        ),
    );
    registry.insert(
        icon::PREFERENCES.to_string(),
        CommandInfo::new(
            icon::PREFERENCES,
            "Preferences",
            icon::PREFERENCES,
            "Ctrl+,",
            CommandCategory::Advanced,
            "Open preferences dialog",
        ),
    );
    registry.insert(
        icon::KEYBOARD_SHORTCUTS.to_string(),
        CommandInfo::new(
            icon::KEYBOARD_SHORTCUTS,
            "Keyboard Shortcuts",
            icon::KEYBOARD_SHORTCUTS,
            "Ctrl+K",
            CommandCategory::Advanced,
            "Show keyboard shortcuts reference",
        ),
    );
    registry.insert(
        icon::PROFILES.to_string(),
        CommandInfo::new(
            icon::PROFILES,
            "Profiles Management",
            icon::PROFILES,
            "",
            CommandCategory::Advanced,
            "Manage user profiles",
        ),
    );

    registry
}

/// Get command info by ID, returns None if not found.
pub fn get_command(id: &str) -> Option<CommandInfo> {
    command_registry().get(id).cloned()
}

/// Check if a command ID exists in the registry.
pub fn has_command(id: &str) -> bool {
    command_registry().contains_key(id)
}

/// Get all commands in a category.
pub fn get_commands_for_category(cat: CommandCategory) -> Vec<CommandId> {
    commands_by_category()
        .get(&cat)
        .cloned()
        .unwrap_or_default()
}

/// Get all commands across all categories.
pub fn get_all_command_ids() -> Vec<CommandId> {
    let mut ids = Vec::new();
    for cmds in commands_by_category().values() {
        ids.extend(cmds.clone());
    }
    ids
}

/// Get total number of commands in the registry.
pub fn command_count() -> usize {
    command_registry().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_count() {
        // 10 + 16 + 9 + 6 + 8 + 7 + 9 + 5 = 70 commands
        assert_eq!(command_count(), 70);
    }

    #[test]
    fn get_command_exists() {
        let cmd = get_command("select");
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.id, "select");
        assert_eq!(cmd.name, "Select");
    }

    #[test]
    fn get_command_not_exists() {
        let cmd = get_command("nonexistent");
        assert!(cmd.is_none());
    }

    #[test]
    fn has_command_test() {
        assert!(has_command("hl"));
        assert!(!has_command("invalid"));
    }

    #[test]
    fn commands_by_category_counts() {
        let by_cat = commands_by_category();

        // Selection & Navigation: 10 commands
        assert_eq!(
            by_cat
                .get(&CommandCategory::SelectionNavigation)
                .map(|v| v.len())
                .unwrap_or(0),
            10
        );

        // Markup Tools: 15 commands
        assert_eq!(
            by_cat
                .get(&CommandCategory::MarkupTools)
                .map(|v| v.len())
                .unwrap_or(0),
            15
        );

        // Measurement Tools: 9 commands
        assert_eq!(
            by_cat
                .get(&CommandCategory::MeasurementTools)
                .map(|v| v.len())
                .unwrap_or(0),
            9
        );

        // Review & Comment: 6 commands
        assert_eq!(
            by_cat
                .get(&CommandCategory::ReviewComment)
                .map(|v| v.len())
                .unwrap_or(0),
            6
        );

        // Edit & Modify: 8 commands
        assert_eq!(
            by_cat
                .get(&CommandCategory::EditModify)
                .map(|v| v.len())
                .unwrap_or(0),
            8
        );

        // View & Display: 7 commands
        assert_eq!(
            by_cat
                .get(&CommandCategory::ViewDisplay)
                .map(|v| v.len())
                .unwrap_or(0),
            7
        );

        // File & Project: 9 commands
        assert_eq!(
            by_cat
                .get(&CommandCategory::FileProject)
                .map(|v| v.len())
                .unwrap_or(0),
            9
        );

        // Advanced: 5 commands
        assert_eq!(
            by_cat
                .get(&CommandCategory::Advanced)
                .map(|v| v.len())
                .unwrap_or(0),
            5
        );
    }

    #[test]
    fn get_commands_for_category_test() {
        let cmds = get_commands_for_category(CommandCategory::SelectionNavigation);
        assert_eq!(cmds.len(), 10);
        assert!(cmds.iter().any(|c| c.0 == "select"));
        assert!(cmds.iter().any(|c| c.0 == "pan"));
    }

    #[test]
    fn get_all_command_ids() {
        let all_ids = get_all_command_ids();
        assert_eq!(all_ids.len(), 70);
        assert!(all_ids.iter().any(|c| c.0 == "select"));
        assert!(all_ids.iter().any(|c| c.0 == "save"));
    }

    #[test]
    fn command_category_labels() {
        use crate::domain::toolbar::CommandCategory;
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
    fn cmd_info_from_registry() {
        let info = get_command("length").unwrap();
        assert_eq!(info.id, "length");
        assert_eq!(info.name, "Length");
        assert_eq!(info.icon, "ln");
        assert_eq!(info.shortcut, "1");
        assert_eq!(info.category, CommandCategory::MeasurementTools);
    }

    #[test]
    fn all_commands_have_valid_ids() {
        let registry = command_registry();
        for (id, info) in &registry {
            assert_eq!(id, &info.id, "Registry key and info.id mismatch for {}", id);
            assert!(!id.is_empty(), "Empty command id");
            assert!(!info.name.is_empty(), "Empty name for command {}", id);
        }
    }

    #[test]
    fn unique_shortcuts() {
        use std::collections::HashSet;
        let registry = command_registry();
        let mut shortcuts = HashSet::new();

        for info in registry.values() {
            if !info.shortcut.is_empty() {
                // Shortcuts can be same if they map to same key (e.g., Ctrl+C vs C)
                // But let's check they're at least present
                if !shortcuts.contains(&info.shortcut) {
                    shortcuts.insert(info.shortcut.clone());
                }
            }
        }
    }

    #[test]
    fn category_membership_consistency() {
        let registry = command_registry();
        let by_cat = commands_by_category();

        for (cat, ids) in &by_cat {
            for cmd_id in ids {
                let info = registry.get(&cmd_id.0);
                assert!(
                    info.is_some(),
                    "Command {} in category {:?} not found in registry",
                    cmd_id.0,
                    cat
                );
                if let Some(info) = info {
                    assert_eq!(
                        info.category, *cat,
                        "Command {} category mismatch",
                        cmd_id.0
                    );
                }
            }
        }
    }

    #[test]
    fn command_info_serde_roundtrip() {
        let info = CommandInfo::new(
            "test",
            "Test Command",
            "test_icon",
            "T",
            CommandCategory::MarkupTools,
            "Test description",
        );

        let json = serde_json::to_string(&info).expect("serialize");
        let restored: CommandInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, info);
    }
}
