//! Assistant review panel: sidebar panel with enabling state and keyboard shortcut.

use egui::Ui;

/// Draw the assistant review panel if enabled.
///
/// The panel appears as a collapsible side panel on the right side of the
/// canvas, showing review-related information and controls. It can be toggled
/// on/off via the View menu or the keyboard shortcut (Ctrl+Shift+R).
pub fn draw_review_panel(ui: &mut Ui, enabled: bool, open: &mut bool) {
    if !enabled {
        return;
    }

    // Panel window flags: movable, resizable, floating, no title bar
    // (we use the View menu/shortcut to show/hide it)
    egui::Window::new("Assistant Review")
        .collapsible(false)
        .resizable(false)
        .fixed_width(300.0)
        .anchor(egui::Pos2::new(1.0, 0.0), egui::Align2::TOPRIGHT)
        .scope(ui.id());

    ui.horizontal(|ui| {
        ui.label("Review Panel");
        if ui.button("Close").clicked() {
            *open = false;
        }
    });

    ui.add_space(8.0);
    ui.label("No active reviews");
    ui.add_space(8.0);
    ui.label("Use Ctrl+Shift+R to toggle this panel");
}