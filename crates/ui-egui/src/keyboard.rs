//! Keyboard navigation and shortcut handling for the Rubric UI.
//!
//! This module defines the complete keyboard shortcut map, view-specific
//! navigation events, and focus management for the egui-based UI.
//! All types are serde-serialisable and testable headlessly.
//!
//! Coordinate system: PDF points (1 pt = 1/72 in), y-down, origin top-left.
//! Shortcuts are defined independently of rendering so they can be rebound
//! or visualised without UI coupling.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Global keyboard shortcuts applicable in any view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalShortcut {
    /// F1 — show help
    F1Help,
    /// Ctrl+K — open command palette
    CtrlKCommandPalette,
    /// Ctrl+1..9 — switch to view at index (1-based)
    CtrlViewSwitch(u8),
    /// Alt+1/2/3 — density cycle (compact/standard/wide)
    AltDensity(u8),
    /// Alt+Shift+D — cycle density direction
    AltShiftDCycle,
}

/// View-specific navigation events, re-emitted by the UI layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ViewEvent {
    /// Table navigation: arrow keys move focus, Enter edits, Space selects.
    Table { action: TableAction },
    /// Gantt chart navigation: pan/zoom/select.
    Gantt { action: GanttAction },
    /// Resource view navigation.
    Resources { action: ResourcesAction },
    /// Inspector field navigation.
    Inspector { action: InspectorAction },
    /// Dialog navigation events.
    Dialog { action: DialogAction },
    /// Command palette fuzzy-search results.
    CommandPalette { query: String, selected: Option<usize> },
}

/// Table cell/actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TableAction {
    /// Move focus up/down/left/right.
    Navigate { dx: i16, dy: i16 },
    /// Edit the current cell.
    Edit,
    /// Toggle selection of the current cell.
    Select,
    /// Select all cells in the table.
    SelectAll,
    /// Delete the selected cell/row.
    Delete,
    /// Copy the selected content to clipboard.
    Copy,
    /// Paste from clipboard to the current cell.
    Paste,
    /// Cut the selected content to clipboard.
    Cut,
}

/// Gantt chart actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GanttAction {
    /// Pan the view by (dx, dy) points.
    Pan { dx: f64, dy: f64 },
    /// Zoom in/out by a factor.
    Zoom { factor: f64 },
    /// Jump to the start or end of the timeline.
    Jump { position: GanttJump },
    /// Select a task via Shift+drag or Space.
    Select { id: String },
}

/// Gantt jump position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GanttJump {
    /// Jump to the beginning of the timeline.
    Start,
    /// Jump to the end of the timeline.
    End,
}

/// Resource view actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ResourcesAction {
    /// Move focus up/down.
    Navigate { dy: i16 },
    /// Edit the assignment for the selected resource.
    EditAssignment,
    /// Remove the selected resource.
    Remove,
}

/// Inspector widget actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InspectorAction {
    /// Move focus to the next/previous field.
    Navigate { forward: bool },
    /// Open/close a dropdown list.
    ToggleDropdown,
    /// Confirm the selected dropdown value.
    Confirm,
    /// Cancel the dropdown selection.
    Cancel,
}

/// Dialog navigation actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DialogAction {
    /// Move focus to the next focusable element within the dialog.
    FocusNext,
    /// Move focus to the previous focusable element within the dialog.
    FocusPrev,
    /// Close the dialog.
    Close,
    /// Activate the default action (e.g. OK button).
    Default,
}

/// Focus ring visual properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusRingProperties {
    /// Thickness in pixels.
    pub thickness: f32,
    /// Offset from the element edge in pixels.
    pub offset: f32,
    /// Colour (ACCENT, ensuring ≥3:1 contrast).
    pub colour: egui::Color32,
}

impl FocusRingProperties {
    /// Default focus ring: 2px thickness, 2px offset, ACCENT colour.
    pub const fn default_accent() -> Self {
        Self {
            thickness: 2.0,
            offset: 2.0,
            colour: egui::Color32::from_rgb(228, 87, 46), // Rubric ACCENT
        }
    }
}

/// Error type for keyboard handling.
#[derive(Debug, Error)]
pub enum KeyboardError {
    #[error("Unknown shortcut for key {key:?}")]
    UnknownShortcut { key: String },
    #[error("Focus ring not configured")]
    FocusRingNotConfigured,
}

/// Key press context, produced by the egui input pipeline.
#[derive(Debug, Clone)]
pub struct KeyPress {
    /// Physical key code (egui Key).
    pub key: egui::Key,
    /// Modifier state at the time of the press.
    pub mods: egui::Modifiers,
    /// Whether the key repeat is active (held down).
    pub repeat: bool,
}

/// Modifier mask bits matching egui::Modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Modifiers {
    /// Ctrl was held.
    pub ctrl: bool,
    /// Shift was held.
    pub shift: bool,
    /// Alt was held.
    pub alt: bool,
}

impl From<egui::Modifiers> for Modifiers {
    fn from(m: egui::Modifiers) -> Self {
        Self {
            ctrl: m.ctrl,
            shift: m.shift,
            alt: m.alt,
        }
    }
}

impl From<Modifiers> for egui::Modifiers {
    fn from(m: Modifiers) -> Self {
        egui::Modifiers {
            ctrl: m.ctrl,
            shift: m.shift,
            alt: m.alt,
            ..Default::default()
        }
    }
}

/// Convert a GlobalShortcut to a human-readable string.
pub fn shortcut_label(shortcut: GlobalShortcut) -> String {
    match shortcut {
        GlobalShortcut::F1Help => "F1".into(),
        GlobalShortcut::CtrlKCommandPalette => "Ctrl+K".into(),
        GlobalShortcut::CtrlViewSwitch(n) => format!("Ctrl+{}", n),
        GlobalShortcut::AltDensity(n) => format!("Alt+{}", n),
        GlobalShortcut::AltShiftDCycle => "Alt+Shift+D".into(),
    }
}

/// Convert a ViewEvent to a human-readable string.
pub fn event_label(event: &ViewEvent) -> String {
    match event {
        ViewEvent::Table { action } => action_label(action),
        ViewEvent::Gantt { action } => format!("Gantt: {}", action_label_gantt(action)),
        ViewEvent::Resources { action } => action_label_resources(action),
        ViewEvent::Inspector { action } => action_label_inspector(action),
        ViewEvent::Dialog { action } => action_label_dialog(action),
        ViewEvent::CommandPalette { query, selected } => {
            if let Some(s) = selected {
                format!("Command palette: '{}' selection {}", query, s + 1)
            } else {
                format!("Command palette: '{}' (no selection)", query)
            }
        }
    }
}

fn action_label(action: &TableAction) -> String {
    match action {
        TableAction::Navigate { dx, dy } => format!("Navigate ({}, {})", dx, dy),
        TableAction::Edit => "Edit".to_string(),
        TableAction::Select => "Select".to_string(),
        TableAction::SelectAll => "Select All".to_string(),
        TableAction::Delete => "Delete".to_string(),
        TableAction::Copy => "Copy".to_string(),
        TableAction::Paste => "Paste".to_string(),
        TableAction::Cut => "Cut".to_string(),
    }
}

fn action_label_gantt(action: &GanttAction) -> String {
    match action {
        GanttAction::Pan { dx, dy } => format!("Pan ({}, {})", dx, dy),
        GanttAction::Zoom { factor } => format!("Zoom {:.1}x", factor),
        GanttAction::Jump { position } => match position {
            GanttJump::Start => "Jump to start".to_string(),
            GanttJump::End => "Jump to end".to_string(),
        },
        GanttAction::Select { .. } => "Select".to_string(),
    }
}

fn action_label_resources(action: &ResourcesAction) -> String {
    match action {
        ResourcesAction::Navigate { dy } => format!("Navigate vertically {}", dy),
        ResourcesAction::EditAssignment => "Edit assignment".to_string(),
        ResourcesAction::Remove => "Remove".to_string(),
    }
}

fn action_label_inspector(action: &InspectorAction) -> String {
    match action {
        InspectorAction::Navigate { forward } => {
            if *forward { "Focus next field" } else { "Focus previous field" }.to_string()
        }
        InspectorAction::ToggleDropdown => "Toggle dropdown".to_string(),
        InspectorAction::Confirm => "Confirm".to_string(),
        InspectorAction::Cancel => "Cancel".to_string(),
    }
}

fn action_label_dialog(action: &DialogAction) -> String {
    match action {
        DialogAction::FocusNext => "Focus next".to_string(),
        DialogAction::FocusPrev => "Focus prev".to_string(),
        DialogAction::Close => "Close".to_string(),
        DialogAction::Default => "Default action".to_string(),
    }
}

/// Map a raw egui key + modifiers to a GlobalShortcut, if one matches.
///
/// This is a pure function — no I/O, no UI state — so it can be tested
/// headlessly and used in CI tests without a display.
pub fn resolve_global_shortcut(key: egui::Key, mods: egui::Modifiers) -> Option<GlobalShortcut> {
    let m = Modifiers::from(mods);

    // F1 is standalone; check regardless of modifiers.
    if key == egui::Key::F1 {
        return Some(GlobalShortcut::F1Help);
    }

    // Ctrl+K for command palette.
    if key == egui::Key::K && m.ctrl {
        return Some(GlobalShortcut::CtrlKCommandPalette);
    }

    // Ctrl+1..9 for view switching.
    if m.ctrl {
        if let Some(num_key) = key_to_number(key) {
            if (1..=9).contains(&num_key) {
                return Some(GlobalShortcut::CtrlViewSwitch(num_key));
            }
        }
    }

    // Alt+1/2/3 for density.
    if m.alt {
        if let Some(num_key) = key_to_number(key) {
            if (1..=3).contains(&num_key) {
                return Some(GlobalShortcut::AltDensity(num_key));
            }
        }
    }

    // Alt+Shift+D for density cycle.
    if m.alt && m.shift && key == egui::Key::D {
        return Some(GlobalShortcut::AltShiftDCycle);
    }

    None
}

/// Map a raw egui key + modifiers to a ViewEvent, if one matches.
pub fn resolve_view_event(key: egui::Key, mods: egui::Modifiers) -> Option<ViewEvent> {
    let m = Modifiers::from(mods);

    // Table navigation with shift: arrow keys move focus.
    // egui::Key::ArrowUp/Down/Left/Right
    match key {
        egui::Key::ArrowUp if m.shift => Some(ViewEvent::Table {
            action: TableAction::Navigate { dx: 0, dy: -1 },
        }),
        egui::Key::ArrowDown if m.shift => Some(ViewEvent::Table {
            action: TableAction::Navigate { dx: 0, dy: 1 },
        }),
        egui::Key::ArrowLeft if m.shift => Some(ViewEvent::Table {
            action: TableAction::Navigate { dx: -1, dy: 0 },
        }),
        egui::Key::ArrowRight if m.shift => Some(ViewEvent::Table {
            action: TableAction::Navigate { dx: 1, dy: 0 },
        }),

        // Table: Enter=Edit, Space=Select.
        egui::Key::Enter => Some(ViewEvent::Table {
            action: TableAction::Edit,
        }),
        egui::Key::Space => Some(ViewEvent::Table {
            action: TableAction::Select,
        }),

        // Ctrl+A = Select All (in table).
        egui::Key::A if m.ctrl => Some(ViewEvent::Table {
            action: TableAction::SelectAll,
        }),

        // Delete in table.
        egui::Key::Delete => Some(ViewEvent::Table {
            action: TableAction::Delete,
        }),

        // Ctrl+C/V/X clipboard actions.
        egui::Key::C if m.ctrl => Some(ViewEvent::Table {
            action: TableAction::Copy,
        }),
        egui::Key::V if m.ctrl => Some(ViewEvent::Table {
            action: TableAction::Paste,
        }),
        egui::Key::X if m.ctrl => Some(ViewEvent::Table {
            action: TableAction::Cut,
        }),

        // Gantt: Arrow keys for pan (no shift = pan).
        egui::Key::ArrowUp => Some(ViewEvent::Gantt {
            action: GanttAction::Pan { dx: 0.0, dy: -8.0 },
        }),
        egui::Key::ArrowDown => Some(ViewEvent::Gantt {
            action: GanttAction::Pan { dx: 0.0, dy: 8.0 },
        }),
        egui::Key::ArrowLeft => Some(ViewEvent::Gantt {
            action: GanttAction::Pan { dx: -8.0, dy: 0.0 },
        }),
        egui::Key::ArrowRight => Some(ViewEvent::Gantt {
            action: GanttAction::Pan { dx: 8.0, dy: 0.0 },
        }),

        // Gantt: Zoom +/-
        // egui 0.37: Key::Equals (unshifted) and with shift for +
        egui::Key::Equals => {
            if m.shift {
                Some(ViewEvent::Gantt {
                    action: GanttAction::Zoom { factor: 1.25 },
                })
            } else {
                Some(ViewEvent::Gantt {
                    action: GanttAction::Zoom { factor: 0.80 },
                })
            }
        }

        // Gantt: Home/End.
        egui::Key::Home => {
            Some(ViewEvent::Gantt {
                action: GanttAction::Jump { position: GanttJump::Start },
            })
        }
        egui::Key::End => {
            Some(ViewEvent::Gantt {
                action: GanttAction::Jump { position: GanttJump::End },
            })
        }

        // Inspector: Tab/Shift+Tab navigate fields.
        egui::Key::Tab if !m.shift => Some(ViewEvent::Inspector {
            action: InspectorAction::Navigate { forward: true },
        }),
        egui::Key::Tab if m.shift => Some(ViewEvent::Inspector {
            action: InspectorAction::Navigate { forward: false },
        }),

        // Inspector: Escape cancel.
        egui::Key::Escape => Some(ViewEvent::Inspector {
            action: InspectorAction::Cancel,
        }),

        _ => None,
    }
}

/// Extract the numeric part from a number key (1..9).
fn key_to_number(key: egui::Key) -> Option<u8> {
    // egui 0.36 uses Num0..Num9 for digit keys (from main row or numpad).
    match key {
        egui::Key::Num1 => Some(1),
        egui::Key::Num2 => Some(2),
        egui::Key::Num3 => Some(3),
        egui::Key::Num4 => Some(4),
        egui::Key::Num5 => Some(5),
        egui::Key::Num6 => Some(6),
        egui::Key::Num7 => Some(7),
        egui::Key::Num8 => Some(8),
        egui::Key::Num9 => Some(9),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Modifiers;

    #[test]
    fn test_resolve_f1_help() {
        let result = resolve_global_shortcut(egui::Key::F1, Modifiers::default());
        assert_eq!(result, Some(GlobalShortcut::F1Help));
    }

    #[test]
    fn test_resolve_ctrl_k() {
        let mut mods = Modifiers::default();
        mods.ctrl = true;
        let result = resolve_global_shortcut(egui::Key::K, mods);
        assert_eq!(result, Some(GlobalShortcut::CtrlKCommandPalette));
    }

    #[test]
    fn test_resolve_ctrl_view_switch() {
        let mut mods = Modifiers::default();
        mods.ctrl = true;
        let result = resolve_global_shortcut(egui::Key::Digit(3), mods);
        assert_eq!(result, Some(GlobalShortcut::CtrlViewSwitch(3)));
    }

    #[test]
    fn test_resolve_alt_density() {
        let mut mods = Modifiers::default();
        mods.alt = true;
        let result = resolve_global_shortcut(egui::Key::Digit(2), mods);
        assert_eq!(result, Some(GlobalShortcut::AltDensity(2)));
    }

    #[test]
    fn test_resolve_alt_shift_d_cycle() {
        let mut mods = Modifiers::default();
        mods.alt = true;
        mods.shift = true;
        let result = resolve_global_shortcut(egui::Key::D, mods);
        assert_eq!(result, Some(GlobalShortcut::AltShiftDCycle));
    }

    #[test]
    fn test_modifiers_from_egui() {
        let egui_mods = Modifiers {
            ctrl: true,
            shift: false,
            alt: true,
            ..Default::default()
        };
        let m: Modifiers = egui_mods.into();
        assert_eq!(m.ctrl, true);
        assert_eq!(m.shift, false);
        assert_eq!(m.alt, true);
    }

    #[test]
    fn test_modifiers_to_and_from_egui_roundtrip() {
        let original = Modifiers { ctrl: true, shift: false, alt: true };
        let back: egui::Modifiers = original.into();
        let roundtrip: Modifiers = back.into();
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn test_shortcut_label() {
        assert_eq!(shortcut_label(GlobalShortcut::F1Help), "F1");
        assert_eq!(shortcut_label(GlobalShortcut::CtrlKCommandPalette), "Ctrl+K");
        assert_eq!(shortcut_label(GlobalShortcut::CtrlViewSwitch(1)), "Ctrl+1");
        assert_eq!(shortcut_label(GlobalShortcut::AltDensity(3)), "Alt+3");
        assert_eq!(shortcut_label(GlobalShortcut::AltShiftDCycle), "Alt+Shift+D");
    }

    #[test]
    fn test_event_label_table() {
        let event = ViewEvent::Table {
            action: TableAction::Navigate { dx: 1, dy: 0 },
        };
        assert_eq!(event_label(&event), "Navigate (1, 0)");

        let event = ViewEvent::Table {
            action: TableAction::SelectAll,
        };
        assert_eq!(event_label(&event), "Select All");
    }

    #[test]
    fn test_event_label_gantt() {
        let event = ViewEvent::Gantt {
            action: GanttAction::Pan { dx: 0.0, dy: -8.0 },
        };
        assert_eq!(event_label(&event), "Gantt: Pan (0, -8)");

        let event = ViewEvent::Gantt {
            action: GanttAction::Zoom { factor: 1.25 },
        };
        assert_eq!(event_label(&event), "Gantt: Zoom 1.3x");

        let event = ViewEvent::Gantt {
            action: GanttAction::Jump { position: GanttJump::Start },
        };
        assert_eq!(event_label(&event), "Gantt: Jump to start");
    }

    #[test]
    fn test_event_label_resources() {
        let event = ViewEvent::Resources {
            action: ResourcesAction::Navigate { dy: 1 },
        };
        assert_eq!(event_label(&event), "Navigate vertically 1");

        let event = ViewEvent::Resources {
            action: ResourcesAction::Remove,
        };
        assert_eq!(event_label(&event), "Remove");
    }

    #[test]
    fn test_event_label_inspector() {
        let event = ViewEvent::Inspector {
            action: InspectorAction::Navigate { forward: true },
        };
        assert_eq!(event_label(&event), "Focus next field");

        let event = ViewEvent::Inspector {
            action: InspectorAction::Confirm,
        };
        assert_eq!(event_label(&event), "Confirm");
    }

    #[test]
    fn test_event_label_dialog() {
        let event = ViewEvent::Dialog {
            action: DialogAction::FocusNext,
        };
        assert_eq!(event_label(&event), "Focus next");

        let event = ViewEvent::Dialog {
            action: DialogAction::Close,
        };
        assert_eq!(event_label(&event), "Close");
    }

    #[test]
    fn test_key_to_number() {
        assert_eq!(key_to_number(egui::Key::Digit(1)), Some(1));
        assert_eq!(key_to_number(egui::Key::Digit(5)), Some(5));
        assert_eq!(key_to_number(egui::Key::Digit(9)), Some(9));
        assert_eq!(key_to_number(egui::Key::Escape), None);
    }

    #[test]
    fn test_focus_ring_properties_default() {
        let props = FocusRingProperties::default_accent();
        assert_eq!(props.thickness, 2.0);
        assert_eq!(props.offset, 2.0);
        assert_eq!(props.colour, egui::Color32::from_rgb(228, 87, 46));
    }

    #[test]
    fn test_keyboard_error_variants() {
        let err = KeyboardError::UnknownShortcut { key: "F2".into() };
        assert!(err.to_string().contains("UnknownShortcut"));

        let err = KeyboardError::FocusRingNotConfigured;
        assert!(err.to_string().contains("Focus ring not configured"));
    }
}