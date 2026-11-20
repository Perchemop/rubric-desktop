//! Quantities panel orchestration.
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use egui::Ui;

use crate::ui::app::RubricApp;
use crate::ui::quantities_panel;

/// Draw the Quantities panel (bottom-panel tab with takeoff schedule).
///
/// Returns true when the project needs to be marked dirty (rare for this panel).
pub(crate) fn draw_quantities_panel(
    app: &mut RubricApp,
    ui: &mut Ui,
) -> bool {
    if app.ui.show_quantities && !app.ui.focus_mode {
        let project = app.history.current_mut();
        let ui_state = &mut app.ui;
        quantities_panel::draw_quantities_panel(ui, project, ui_state)
    } else {
        false
    }
}