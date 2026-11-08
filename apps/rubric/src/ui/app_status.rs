//! Status bar panel orchestration.
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use egui::{Align, Layout, Ui};

use crate::ui::app::RubricApp;

pub(crate) fn draw_status_bar(app: &mut RubricApp, ui: &mut Ui) {
    let doc_title = app.active_doc().map(|d| d.title.clone());
    let page_count = app.active_doc().map(|d| d.page_count());

    ui.horizontal(|ui| {
        ui.small(app.ui.hint.clone());
        if let Some(p) = app.last_page_pos {
            ui.small(format!("Cursor: {:.1},{:.1} pt", p.x, p.y));
        }
        if let (Some(pages), Some(title)) = (page_count, doc_title) {
            let plural = if pages == 1 { "" } else { "s" };
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .small(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .on_hover_text("Click for release notes")
                    .clicked()
                {
                    app.ui.show_release_notes = true;
                }
                ui.small(format!("{pages} page{plural}"));
                ui.small(title);
            });
        }
    });
}
