//! Markups list panel orchestration.
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use egui::Ui;

use crate::ui::app::RubricApp;
use crate::ui::markups_list::{self, MarkupListAction};

pub(crate) fn draw_markups_list_panel(
    app: &mut RubricApp,
    ui: &mut Ui,
) -> (Option<MarkupListAction>, bool) {
    let mut list_action = None;
    let mut status_changed = false;
    if app.ui.show_markups_list && !app.ui.focus_mode {
        let project = app.history.current_mut();
        let ui_state = &mut app.ui;
        egui::Panel::bottom("markups_list")
            .default_size(190.0)
            .min_size(110.0)
            .max_size(420.0)
            .resizable(true)
            .show(ui, |ui| {
                let (action, sc) = markups_list::draw_markups_list(ui, project, ui_state);
                list_action = action;
                status_changed = sc;
            });
    }
    (list_action, status_changed)
}
