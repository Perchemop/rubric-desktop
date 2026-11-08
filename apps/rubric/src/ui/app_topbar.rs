//! Top bar (toolbar + active-tool chip + status + page nav).
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use eframe::egui::{self, Align, Ui};
use crate::ui::icon_button::{self, draw_prev_icon, draw_next_icon};

use crate::ui::app::{RubricApp, StatusKind};
use crate::ui::toolbar;
use crate::domain::{PageStep, page_nav_target};

/// Draw the top bar: toolbar, active-tool chip, status, zoom, page nav.
pub(crate) fn draw_top_bar(app: &mut RubricApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        toolbar::draw_toolbar(ui, &mut app.ui.active_tool, &mut app.ui.status);

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
            // Severity-coloured status (UI audit P1-1): errors are visually
            // distinct from success/info and long messages are shown on hover.
            let status_color = match app.ui.status_kind {
                StatusKind::Info => ui.visuals().text_color(),
                StatusKind::Success => egui::Color32::from_rgb(76, 175, 80),
                StatusKind::Warning => egui::Color32::from_rgb(255, 152, 0),
                StatusKind::Error => egui::Color32::from_rgb(229, 57, 53),
            };
            ui.add(egui::Label::new(
                egui::RichText::new(app.ui.status.clone()).color(status_color),
            ))
            .on_hover_text(app.ui.status.clone());

            ui.label(format!("{:.0}%", app.ui.zoom * 100.0));
            // Page navigation chrome.
            // The outer layout is right-to-left, so we emit widgets in reverse
            // visual order to achieve the expected left-to-right reading order:
            // Previous | Page x/y | Next.
            let count = app.active_doc().map(|d| d.page_count()).unwrap_or(0);
            if count > 0 {
                let current = app.ui.current_page.min(count - 1);
                // Next button emitted first → lands rightmost in right-to-left layout.
                let next_target = page_nav_target(current, count, PageStep::Next);
                let can_next = next_target.is_some();
                if ui
                    .add_enabled(can_next, icon_button(ui, draw_next_icon, "Next page", true))
                    .on_hover_text("Next page")
                    .clicked()
                {
                    if let Some(target) = next_target {
                        app.jump_to_page(target);
                    }
                }
                ui.label(format!("Page {}/{}", current + 1, count));
                // Previous button emitted last → lands leftmost in right-to-left layout.
                let prev_target = page_nav_target(current, count, PageStep::Previous);
                let can_prev = prev_target.is_some();
                if ui
                    .add_enabled(can_prev, icon_button(ui, draw_prev_icon, "Previous page", true))
                    .on_hover_text("Previous page")
                    .clicked()
                {
                    if let Some(target) = prev_target {
                        app.jump_to_page(target);
                    }
                }
            }
        });
    });
}
