//! Canvas (central panel) orchestration.
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use egui::Ui;

use crate::ui::app::RubricApp;

pub(crate) fn draw_canvas_panel(app: &mut RubricApp, ui: &mut Ui) {
    let prior_status = app.ui.status.clone();
    let show_minimap = app.ui.show_minimap;
    let mut need_snapshot = false;
    egui::CentralPanel::default().show(ui, |ui| {
        app.ui.canvas_size = ui.available_size();
        need_snapshot = crate::ui::canvas::draw_canvas(
            ui,
            app.history.current_mut(),
            &mut app.ui,
            &mut app.draw,
            &mut app.last_page_pos,
            &mut app.page_textures,
            &mut app.failed_page_textures,
            &mut app.bg_error,
            false, // is_export: show chrome in live editor
            show_minimap,
        );
    });
    if need_snapshot {
        if app.ui.status != prior_status {
            app.push_snapshot_quiet();
        } else {
            app.push_snapshot("drawing");
        }
    }
}
