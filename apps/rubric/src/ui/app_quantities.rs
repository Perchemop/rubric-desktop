//! Quantities panel orchestration.
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use egui::Ui;

use crate::ui::app::RubricApp;
use crate::ui::quantities_panel::{draw_quantities_panel, QuantitiesPanelAction};

/// Draw the Quantities panel (bottom-panel tab with takeoff schedule).
///
/// Returns true when the project needs to be marked dirty (rare for this panel).
pub(crate) fn draw_quantities_panel(
    app: &mut RubricApp,
    ui: &mut Ui,
) -> (Option<QuantitiesPanelAction>, bool) {
    let mut action = None;
    let mut status_changed = false;

    if app.ui.show_quantities && !app.ui.focus_mode {
        let project = app.history.current_mut();
        let ui_state = &mut app.ui;
        egui::Panel::bottom("quantities")
            .default_size(200.0)
            .min_size(140.0)
            .max_size(400.0)
            .resizable(true)
            .show(ui, |ui| {
                let (a, sc) = draw_quantities_panel(ui, project, ui_state);
                action = a;
                status_changed = sc;
            });
    }
    (action, status_changed)
}