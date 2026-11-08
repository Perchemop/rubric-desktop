//! Tool chest panel orchestration.
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use egui::Ui;

use crate::ui::app::RubricApp;

pub(crate) fn draw_tool_chest_panel(app: &mut RubricApp, ui: &mut Ui) -> bool {
    let mut chest_mutated = false;
    if app.ui.show_tool_chest && !app.ui.focus_mode {
        let library = &mut app.tool_chest_library;
        let project = app.history.current_mut();
        let ui_state = &mut app.ui;
        egui::Panel::left("tool_chest")
            .default_size(210.0)
            .min_size(170.0)
            .resizable(true)
            .show(ui, |ui| {
                chest_mutated = crate::ui::tool_chest::draw_tool_chest(ui, library, project, ui_state);
            });
    }
    chest_mutated
}
