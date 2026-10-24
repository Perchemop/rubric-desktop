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

    let selection = ui_state.selected.len();
    for action in [EditorAction::DeleteSelection, EditorAction::ClearSelection] {
        let button = egui::Button::new(action.label());
        if ui
            .add_enabled(action.is_enabled(selection), button)
            .clicked()
        {
            ui_state.pending_action = Some(action);
            ui.close();
        }
    }

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
        let button = egui::Button::new(action.label());
        if ui
            .add_enabled(action.is_enabled(selection), button)
            .clicked()
        {
            ui_state.pending_action = Some(action);
            ui.close();
        }
    }
}
