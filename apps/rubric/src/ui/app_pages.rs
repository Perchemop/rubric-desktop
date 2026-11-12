//! Pages sidebar panel orchestration.
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use egui::Ui;

use crate::ui::app::RubricApp;

pub(crate) fn draw_pages_panel(app: &mut RubricApp, ui: &mut Ui) {
    let page_count = app.active_doc().map(|d| d.page_count()).unwrap_or(0);
    if page_count == 0 || !app.ui.show_pages || app.ui.focus_mode {
        return;
    }
    let source_pdf = app.history.current().source_pdf.clone();

    // Lazily spawn the shared render worker on first use (pages sidebar or
    // canvas); ownership is hoisted to `RubricApp` (Slice 5).
    RubricApp::ensure_render_worker(&mut app.render_worker, ui.ctx());

    egui::Panel::left("pages")
        .default_size(168.0)
        .min_size(120.0)
        .max_size(260.0)
        .resizable(true)
        .show(ui, |ui| {
            if let Some(index) = crate::ui::pages::draw_pages(
                ui,
                page_count,
                app.ui.current_page.min(page_count - 1),
                source_pdf.as_deref().map(std::path::Path::new),
                &mut app.thumb_cache,
                app.render_worker.as_ref().unwrap(),
            ) {
                app.jump_to_page(index);
            }
        });
}
