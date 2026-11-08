//! Inspector panel orchestration.
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use egui::Ui;

use crate::ui::app::RubricApp;
use crate::ui::menu;

pub(crate) fn draw_inspector_panel(app: &mut RubricApp, ui: &mut Ui) -> bool {
    let mut inspector_mutated = false;
    if !app.ui.focus_mode {
        let inspector_response = egui::Panel::right("inspector")
            .default_size(280.0)
            .min_size(240.0)
            .show(ui, |ui| {
                inspector_mutated = crate::ui::inspector::draw_inspector(
                    ui,
                    app.history.current_mut(),
                    &mut app.ui,
                    &mut app.comment_text,
                );
            })
            .response;
        let _ = inspector_response.context_menu(|ui| {
            menu::draw_editor_context_menu(ui, &mut app.ui);
        });
    }
    inspector_mutated
}
