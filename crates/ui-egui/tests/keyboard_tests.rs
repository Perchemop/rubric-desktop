//! Unit tests for the Rubric keyboard navigation module.
//!
//! These tests verify that keyboard shortcut resolution, event mapping,
//! and focus ring properties work correctly headlessly (no display required).
//! All tests are run by CI via `cargo test -p ui-egui --lib`.
//!
//! > **Note:** The `keyboard.rs` module is pure Rust — no egui RenderContext
//!   or OS input is needed. Tests can run on any CI runner.

use egui::Modifiers as EguiModifiers;
use ui_egui::keyboard::{
    DialogAction, FocusRingProperties, GanttAction, GanttJump, GlobalShortcut, InspectorAction,
    KeyboardError, ResourcesAction, TableAction, ViewEvent, event_label, resolve_global_shortcut,
    shortcut_label,
};

#[test]
fn test_resolve_f1_help() {
    let result = resolve_global_shortcut(egui::Key::F1, EguiModifiers::default());
    assert_eq!(result, Some(GlobalShortcut::F1Help));
}

#[test]
fn test_resolve_ctrl_k() {
    let mut mods = EguiModifiers::default();
    mods.ctrl = true;
    let result = resolve_global_shortcut(egui::Key::K, mods);
    assert_eq!(result, Some(GlobalShortcut::CtrlKCommandPalette));
}

#[test]
fn test_resolve_ctrl_view_switch_1() {
    let mut mods = EguiModifiers::default();
    mods.ctrl = true;
    let result = resolve_global_shortcut(egui::Key::Num1, mods);
    assert_eq!(result, Some(GlobalShortcut::CtrlViewSwitch(1)));
}

#[test]
fn test_resolve_ctrl_view_switch_9() {
    let mut mods = EguiModifiers::default();
    mods.ctrl = true;
    let result = resolve_global_shortcut(egui::Key::Num9, mods);
    assert_eq!(result, Some(GlobalShortcut::CtrlViewSwitch(9)));
}

#[test]
fn test_resolve_alt_density_1() {
    let mut mods = EguiModifiers::default();
    mods.alt = true;
    let result = resolve_global_shortcut(egui::Key::Num1, mods);
    assert_eq!(result, Some(GlobalShortcut::AltDensity(1)));
}

#[test]
fn test_resolve_alt_density_3() {
    let mut mods = EguiModifiers::default();
    mods.alt = true;
    let result = resolve_global_shortcut(egui::Key::Num3, mods);
    assert_eq!(result, Some(GlobalShortcut::AltDensity(3)));
}

#[test]
fn test_resolve_alt_shift_d_cycle() {
    let mut mods = EguiModifiers::default();
    mods.alt = true;
    mods.shift = true;
    let result = resolve_global_shortcut(egui::Key::D, mods);
    assert_eq!(result, Some(GlobalShortcut::AltShiftDCycle));
}

#[test]
fn test_resolve_no_shortcut() {
    // Pressing 'A' with no modifiers should not match any global shortcut.
    let result = resolve_global_shortcut(egui::Key::A, EguiModifiers::default());
    assert_eq!(result, None);
}

#[test]
fn test_event_label_table_navigation() {
    let event = ViewEvent::Table {
        action: TableAction::Navigate { dx: 1, dy: -1 },
    };
    assert_eq!(event_label(&event), "Navigate (1, -1)");
}

#[test]
fn test_event_label_table_select() {
    let event = ViewEvent::Table {
        action: TableAction::Select,
    };
    assert_eq!(event_label(&event), "Select");
}

#[test]
fn test_event_label_table_select_all() {
    let event = ViewEvent::Table {
        action: TableAction::SelectAll,
    };
    assert_eq!(event_label(&event), "Select All");
}

#[test]
fn test_event_label_table_delete() {
    let event = ViewEvent::Table {
        action: TableAction::Delete,
    };
    assert_eq!(event_label(&event), "Delete");
}

#[test]
fn test_event_label_table_copy_paste_cut() {
    let event = ViewEvent::Table {
        action: TableAction::Copy,
    };
    assert_eq!(event_label(&event), "Copy");

    let event = ViewEvent::Table {
        action: TableAction::Paste,
    };
    assert_eq!(event_label(&event), "Paste");

    let event = ViewEvent::Table {
        action: TableAction::Cut,
    };
    assert_eq!(event_label(&event), "Cut");
}

#[test]
fn test_event_label_gantt_pan() {
    let event = ViewEvent::Gantt {
        action: GanttAction::Pan { dx: 0.0, dy: -8.0 },
    };
    assert_eq!(event_label(&event), "Gantt: Pan (0, -8)");
}

#[test]
fn test_event_label_gantt_zoom() {
    let event = ViewEvent::Gantt {
        action: GanttAction::Zoom { factor: 1.25 },
    };
    // `{:.1}` rounds half to even, so 1.25 formats as "1.2".
    assert_eq!(event_label(&event), "Gantt: Zoom 1.2x");
}

#[test]
fn test_event_label_gantt_jump_start() {
    let event = ViewEvent::Gantt {
        action: GanttAction::Jump {
            position: GanttJump::Start,
        },
    };
    assert_eq!(event_label(&event), "Gantt: Jump to start");
}

#[test]
fn test_event_label_gantt_jump_end() {
    let event = ViewEvent::Gantt {
        action: GanttAction::Jump {
            position: GanttJump::End,
        },
    };
    assert_eq!(event_label(&event), "Gantt: Jump to end");
}

#[test]
fn test_event_label_gantt_select() {
    let event = ViewEvent::Gantt {
        action: GanttAction::Select {
            id: "task-42".into(),
        },
    };
    assert_eq!(event_label(&event), "Gantt: Select");
}

#[test]
fn test_event_label_resources_navigate() {
    let event = ViewEvent::Resources {
        action: ResourcesAction::Navigate { dy: -1 },
    };
    assert_eq!(event_label(&event), "Navigate vertically -1");
}

#[test]
fn test_event_label_resources_edit() {
    let event = ViewEvent::Resources {
        action: ResourcesAction::EditAssignment,
    };
    assert_eq!(event_label(&event), "Edit assignment");
}

#[test]
fn test_event_label_resources_remove() {
    let event = ViewEvent::Resources {
        action: ResourcesAction::Remove,
    };
    assert_eq!(event_label(&event), "Remove");
}

#[test]
fn test_event_label_inspector_next() {
    let event = ViewEvent::Inspector {
        action: InspectorAction::Navigate { forward: true },
    };
    assert_eq!(event_label(&event), "Focus next field");
}

#[test]
fn test_event_label_inspector_prev() {
    let event = ViewEvent::Inspector {
        action: InspectorAction::Navigate { forward: false },
    };
    assert_eq!(event_label(&event), "Focus previous field");
}

#[test]
fn test_event_label_inspector_confirm() {
    let event = ViewEvent::Inspector {
        action: InspectorAction::Confirm,
    };
    assert_eq!(event_label(&event), "Confirm");
}

#[test]
fn test_event_label_inspector_cancel() {
    let event = ViewEvent::Inspector {
        action: InspectorAction::Cancel,
    };
    assert_eq!(event_label(&event), "Cancel");
}

#[test]
fn test_event_label_dialog_next() {
    let event = ViewEvent::Dialog {
        action: DialogAction::FocusNext,
    };
    assert_eq!(event_label(&event), "Focus next");
}

#[test]
fn test_event_label_dialog_prev() {
    let event = ViewEvent::Dialog {
        action: DialogAction::FocusPrev,
    };
    assert_eq!(event_label(&event), "Focus prev");
}

#[test]
fn test_event_label_dialog_close() {
    let event = ViewEvent::Dialog {
        action: DialogAction::Close,
    };
    assert_eq!(event_label(&event), "Close");
}

#[test]
fn test_event_label_dialog_default() {
    let event = ViewEvent::Dialog {
        action: DialogAction::Default,
    };
    assert_eq!(event_label(&event), "Default action");
}

#[test]
fn test_event_label_command_palette_with_selection() {
    let event = ViewEvent::CommandPalette {
        query: "widget".into(),
        selected: Some(2),
    };
    assert_eq!(event_label(&event), "Command palette: 'widget' selection 3");
}

#[test]
fn test_event_label_command_palette_no_selection() {
    let event = ViewEvent::CommandPalette {
        query: "widget".into(),
        selected: None,
    };
    assert_eq!(
        event_label(&event),
        "Command palette: 'widget' (no selection)"
    );
}

#[test]
fn test_shortcut_label_all() {
    assert_eq!(shortcut_label(GlobalShortcut::F1Help), "F1");
    assert_eq!(
        shortcut_label(GlobalShortcut::CtrlKCommandPalette),
        "Ctrl+K"
    );
    assert_eq!(shortcut_label(GlobalShortcut::CtrlViewSwitch(1)), "Ctrl+1");
    assert_eq!(shortcut_label(GlobalShortcut::CtrlViewSwitch(9)), "Ctrl+9");
    assert_eq!(shortcut_label(GlobalShortcut::AltDensity(1)), "Alt+1");
    assert_eq!(shortcut_label(GlobalShortcut::AltDensity(3)), "Alt+3");
    assert_eq!(
        shortcut_label(GlobalShortcut::AltShiftDCycle),
        "Alt+Shift+D"
    );
}

// `key_to_number` is a private helper; its behaviour is unit-tested inside
// `crates/ui-egui/src/keyboard.rs` (`test_key_to_number`) and is not repeated
// here because integration tests cannot reach private items.

#[test]
fn test_focus_ring_properties_default() {
    let props = FocusRingProperties::default_accent();
    assert_eq!(props.thickness, 2.0);
    assert_eq!(props.offset, 2.0);
    assert_eq!(props.colour, egui::Color32::from_rgb(228, 87, 46));
}

#[test]
fn test_focus_ring_stroke_colour_contrast() {
    let props = FocusRingProperties::default_accent();
    let fg = props.colour;
    let bg_white = egui::Color32::WHITE;
    let bg_black = egui::Color32::BLACK;

    let ratio_white = ui_egui::design_tokens::contrast_ratio(fg.into(), bg_white.into());
    let ratio_black = ui_egui::design_tokens::contrast_ratio(fg.into(), bg_black.into());

    // AA-large requires ≥3:1 for UI components.
    assert!(
        ratio_white >= 3.0,
        "ACCENT on white: ratio {:.2}:1, expected ≥3.0",
        ratio_white
    );
    assert!(
        ratio_black >= 3.0,
        "ACCENT on black: ratio {:.2}:1, expected ≥3.0",
        ratio_black
    );
}

#[test]
fn test_keyboard_error_messages() {
    let err = KeyboardError::UnknownShortcut { key: "F2".into() };
    assert!(err.to_string().contains("Unknown shortcut"));

    let err = KeyboardError::FocusRingNotConfigured;
    assert!(err.to_string().contains("Focus ring not configured"));
}

#[test]
fn test_modifiers_from_egui_roundtrip() {
    let egui_mods = EguiModifiers {
        ctrl: true,
        shift: false,
        alt: true,
        ..Default::default()
    };
    let m: ui_egui::keyboard::Modifiers = egui_mods.into();
    let back: egui::Modifiers = m.into();
    assert_eq!(back.ctrl, true);
    assert_eq!(back.shift, false);
    assert_eq!(back.alt, true);
}
