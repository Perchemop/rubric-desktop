//! Unified bottom panel (UI redesign T3).
//!
//! Markups List, Quantities, Layers and Thumbnails live in one tabbed bottom
//! container following the Bluebeam pattern: a single `Panel::bottom` with a
//! tab strip; only the active tab's content is drawn. The former arrangement
//! stacked two competing bottom panels (status + quantities/markups list) and
//! toggled visibility flags between them.

use egui::Ui;

use crate::ui::app::{RubricApp, UiState};
use crate::ui::menu;

/// Which tab the unified bottom panel is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum BottomTab {
    /// The markups table with filter/search (default).
    #[default]
    Markups,
    /// Takeoff condition schedule.
    Quantities,
}

impl BottomTab {
    pub(crate) const ALL: [BottomTab; 2] = [BottomTab::Markups, BottomTab::Quantities];

    pub(crate) fn label(self) -> &'static str {
        match self {
            BottomTab::Markups => "Markups List",
            BottomTab::Quantities => "Quantities",
        }
    }
}

/// Draw the unified bottom panel.
///
/// Returns `(action, mutated)` where `action` is the Quantities panel's
/// dialog-opening request and `mutated` is whether the project changed.
pub(crate) fn draw_bottom_panel(
    app: &mut RubricApp,
    ui: &mut Ui,
) -> (Option<crate::ui::quantities_panel::QuantityAction>, bool) {
    let mut action = None;
    let mutated = false;

    if app.ui.focus_mode {
        return (action, mutated);
    }

    egui::Panel::bottom("bottom_panel")
        .default_size(200.0)
        .min_size(140.0)
        .max_size(400.0)
        .resizable(true)
        .show(ui, |ui| {
            draw_tab_strip(ui, &mut app.ui, app.ui.show_quantities);

            match app.ui.bottom_tab {
                BottomTab::Markups => {
                    let project = app.history.current_mut();
                    let ui_state = &mut app.ui;
                    let (list_action, status_changed) =
                        crate::ui::markups_list::draw_markups_list(ui, project, ui_state);
                    let _ = status_changed;
                    if let Some(la) = list_action {
                        ui_state.selected.clear();
                        ui_state.selected.insert(la.id);
                        if la.page != ui_state.current_page {
                            app_jump(app, la.page);
                        }
                    }
                }
                BottomTab::Quantities => {
                    let project = app.history.current_mut();
                    let ui_state = &mut app.ui;
                    let (qty_action, _dirty) =
                        crate::ui::quantities_panel::draw_quantities_content(ui, project, ui_state);
                    action = qty_action;
                }
            }

            // Context menu on the panel body (same editor menu as canvas).
            let _ = ui.response().context_menu(|ui| {
                menu::draw_editor_context_menu(ui, &mut app.ui, app.history.current())
            });
        });

    (action, mutated)
}

/// Jump helper so the markups-list row click can change pages without
/// borrowing the whole app inside the closure.
fn app_jump(app: &mut RubricApp, page: usize) {
    app.jump_to_page(page);
}

/// The tab strip: one row of tab buttons; active tab highlighted.
fn draw_tab_strip(ui: &mut Ui, state: &mut UiState, show_quantities: bool) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        for tab in BottomTab::ALL {
            // Skip Quantities tab if not enabled in preferences
            if tab == BottomTab::Quantities && !show_quantities {
                continue;
            }
            let selected = state.bottom_tab == tab;
            let text = if selected {
                egui::RichText::new(tab.label()).strong()
            } else {
                egui::RichText::new(tab.label()).weak()
            };
            if ui.selectable_label(selected, text).clicked() {
                state.bottom_tab = tab;
            }
            ui.separator();
        }
    });
}
