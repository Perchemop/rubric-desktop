//! Shared editor context menu (spec F-603).
//!
//! Both the canvas and the inspector attach the *same* menu: the canvas to its
//! page response and the inspector to its panel response. Building it once
//! keeps the two in lockstep.
//!
//! The menu never executes a command itself. It records the choice on
//! [`UiState`] (`pending_action`, or `active_tool` for the tool submenu) so a
//! single path in `RubricApp` performs the work and owns history snapshots and
//! status text.

use eframe::egui::{self, Ui};

use crate::domain::{EditorAction, MarkupKind, Tool};
use crate::ui::app::UiState;

/// Menu label for a tool. `Select` has no markup kind, so it is named
/// explicitly; every other tool uses its kind label.
fn tool_label(tool: Tool) -> &'static str {
    match tool {
        Tool::Select => "Select",
        other => other.to_kind().map(MarkupKind::label).unwrap_or("Tool"),
    }
}

/// Record `action` on `ui_state` when its menu entry is clicked.
fn action_button(ui: &mut Ui, ui_state: &mut UiState, action: EditorAction) {
    let enabled = action.is_enabled(ui_state.selected.len(), ui_state.style_clipboard.is_some());
    if ui
        .add_enabled(enabled, egui::Button::new(action.label()))
        .clicked()
    {
        ui_state.pending_action = Some(action);
        ui.close();
    }
}

/// Selection-dependent editing commands shared by the context menu and the
/// Edit menu (F-113–F-115).
pub(crate) fn draw_selection_commands(ui: &mut Ui, ui_state: &mut UiState) {
    for action in [EditorAction::DeleteSelection, EditorAction::ClearSelection] {
        action_button(ui, ui_state, action);
    }

    ui.separator();

    for action in [EditorAction::CopyProperties, EditorAction::PasteProperties] {
        action_button(ui, ui_state, action);
    }

    ui.separator();

    ui.menu_button("Order", |ui| {
        for action in [
            EditorAction::BringForward,
            EditorAction::SendBackward,
            EditorAction::BringToFront,
            EditorAction::SendToBack,
        ] {
            action_button(ui, ui_state, action);
        }
    });
}

/// Render the editor context menu, recording the chosen action on `ui_state`.
pub(crate) fn draw_editor_context_menu(ui: &mut Ui, ui_state: &mut UiState) {
    // Tool submenu — reuses the F-601 shortcut keys for discoverability.
    ui.menu_button("Tool", |ui| {
        for tool in Tool::ALL {
            let selected = ui_state.active_tool == tool;
            let text = match tool.shortcut_key() {
                Some(key) => format!("{}\t{}", tool_label(tool), key.to_ascii_uppercase()),
                None => tool_label(tool).to_string(),
            };
            if ui.selectable_label(selected, text).clicked() {
                ui_state.active_tool = tool;
                ui_state.status = format!("Tool: {}", tool_label(tool));
                ui.close();
            }
        }
    });

    ui.separator();

    draw_selection_commands(ui, ui_state);

    ui.separator();

    // History and view actions. Their applicability is decided by
    // `EditorAction::is_enabled`; the executor no-ops where nothing applies.
    for action in [
        EditorAction::Undo,
        EditorAction::Redo,
        EditorAction::ZoomIn,
        EditorAction::ZoomOut,
        EditorAction::FitPage,
    ] {
        action_button(ui, ui_state, action);
    }
}
