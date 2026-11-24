//! Top bar (toolbar + active-tool chip + status + page nav).
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use eframe::egui::{self, Align, Ui};

use crate::ui::app::{RubricApp, StatusKind};
use crate::ui::icon_button::{triangle_button, TriangleDir};
use crate::ui::toolbar;

/// Draw the top bar: document tabs, toolbar, active-tool chip, status, zoom, page nav.
pub(crate) fn draw_top_bar(app: &mut RubricApp, ui: &mut Ui) {
    // Document tab bar (Bluebeam audit gap #11): one tab per document.
    ui.horizontal(|ui| {
        let docs: Vec<(uuid::Uuid, String)> = app
            .history
            .current()
            .documents
            .iter()
            .map(|d| (d.id, d.title.clone()))
            .collect();
        let active_index = app.history.current().active_document;
        let active_id = app
            .history
            .current()
            .documents
            .get(active_index)
            .map(|d| d.id);
        let count = docs.len();
        for (id, title) in &docs {
            let selected = Some(*id) == active_id;
            let label = if title.chars().count() > 18 {
                format!("{}...", title.chars().take(15).collect::<String>())
            } else {
                title.clone()
            };
            let resp = ui.selectable_label(selected, label);
            if resp.clicked() {
                app.switch_active_document(*id);
            }
            resp.on_hover_text(title);
            if count > 1
                && ui
                    .add(egui::Button::new("x").min_size(egui::vec2(24.0, 24.0)))
                    .clicked()
            {
                app.close_document_tab(*id);
            }
        }
        if ui.button("+").clicked() {
            app.add_new_document();
        }
    });
    ui.separator();
    ui.horizontal(|ui| {
        // Tool dropdown (replaces the inline strip that cluttered the top bar).
        // All 25 tools live inside the menu, grouped by family.
        if toolbar::draw_tool_dropdown(ui, &mut app.ui.active_tool) {
            app.set_toast(crate::ui::app::StatusKind::Info, format!("Tool: {}", app.ui.active_tool.label()));
        }

        // Active-tool chip (P0-4): a persistent pill showing what tool is
        // currently active, with its shortcut key. Always visible so the
        // user never has to hunt through the left panel to know the mode.
        // Uses a neutral chip style (not the selection orange) so the
        // active tool is distinct from selected markups.
        ui.separator();
        egui::Frame::new()
            .fill(ui.visuals().widgets.active.bg_fill)
            .stroke(ui.visuals().widgets.active.bg_stroke)
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::vec2(6.0, 2.0))
            .show(ui, |ui| {
                let tool_name = app.ui.active_tool.label();
                let shortcut = app
                    .ui
                    .active_tool
                    .shortcut_key()
                    .map(|k| format!(" ({})", k.to_ascii_uppercase()))
                    .unwrap_or_default();
                let label = match &app.ui.pending_style_preset {
                    Some(armed) => format!("{} (style: {})", tool_name, armed.name),
                    None => format!("{}{}", tool_name, shortcut),
                };
                ui.label(
                    egui::RichText::new(label).color(ui.visuals().widgets.active.text_color()),
                );
            });

        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            // Status text lives in the bottom status bar (UI redesign T4);
            // the top bar keeps only zoom, page nav and the transient toast.

            ui.label(format!("{:.0}%", app.ui.zoom * 100.0));
            // Page navigation chrome.
            let count = app.active_doc().map(|d| d.page_count()).unwrap_or(0);
            if count > 0 {
                let current = app.ui.current_page.min(count - 1);
                if triangle_button(ui, TriangleDir::Right, current + 1 < count, "Next page") {
                    app.jump_to_page(current + 1);
                }
                ui.label(format!("Page {}/{}", current + 1, count));
                if triangle_button(ui, TriangleDir::Left, current > 0, "Previous page") {
                    app.jump_to_page(current.saturating_sub(1));
                }
            }

            // Toast notification (UI audit P1-1-b): transient, auto-clears after 3s.
            if let Some((kind, msg, instant)) = &app.toast {
                let age = instant.elapsed().as_secs_f32();
                let show = age < 3.0;
                let alpha = if show { 1.0 - (age / 3.0) } else { 0.0 };
                if alpha > 0.0 {
                    let toast_color = match kind {
                        StatusKind::Info => ui.visuals().text_color(),
                        StatusKind::Success => egui::Color32::from_rgb(76, 175, 80),
                        StatusKind::Warning => egui::Color32::from_rgb(255, 152, 0),
                        StatusKind::Error => egui::Color32::from_rgb(229, 57, 53),
                    };
                    ui.add(egui::Label::new(
                        egui::RichText::new(msg.clone()).color(toast_color),
                    ));
                }
            }
        });
    });
}
