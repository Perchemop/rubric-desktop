//! Status bar panel orchestration.
//!
//! Extracted from `ui/app.rs` to reduce the god-file size. UI redesign T4
//! consolidated the app status message here (severity-coloured) so the top
//! toolbar carries only the tool chip, zoom and page navigation.

use egui::{Align, Layout, Ui};

use crate::ui::app::{RubricApp, StatusKind};

pub(crate) fn draw_status_bar(app: &mut RubricApp, ui: &mut Ui) {
    let doc_title = app.active_doc().map(|d| d.title.clone());
    let page_count = app.active_doc().map(|d| d.page_count());

    ui.horizontal(|ui| {
        // Severity-coloured app status (UI audit P1-1, relocated from the top
        // bar in UI redesign T4): errors are visually distinct from
        // success/info and long messages are shown on hover.
        let status_color = match app.ui.status_kind {
            StatusKind::Info => ui.visuals().text_color(),
            StatusKind::Success => egui::Color32::from_rgb(76, 175, 80),
            StatusKind::Warning => egui::Color32::from_rgb(255, 152, 0),
            StatusKind::Error => egui::Color32::from_rgb(229, 57, 53),
        };
        ui.add(
            egui::Label::new(egui::RichText::new(app.ui.status.clone()).color(status_color)),
        )
        .on_hover_text(app.ui.status.clone());

        ui.separator();
        ui.small(app.ui.hint.clone());
        if let Some(p) = app.last_page_pos {
            ui.small(format!("Cursor: {:.1},{:.1} pt", p.x, p.y));
        }
        if let (Some(pages), Some(title)) = (page_count, doc_title) {
            let plural = if pages == 1 { "" } else { "s" };
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Version button styled as clickable (UI audit follow-up):
                // a framed button rather than frameless plain text, so the
                // release-notes affordance is discoverable.
                let version_btn = egui::Button::new(
                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).small(),
                );
                if ui
                    .add(version_btn)
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
