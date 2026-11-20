//! Quantities panel: bottom-panel tab strip with takeoff condition schedule.
//!
//! Displays a table of conditions with their aggregated quantities,
//! supporting assignment tracking and selection of contributing markups.
//!
//! This is a thin view over the domain `quantity_rows()` function. It never
//! re-implements aggregation logic; it only translates the UI controls into
//! domain values and renders the resulting rows.

use eframe::egui::{self, RichText, Ui};
use uuid::Uuid;

use crate::domain::takeoff::{quantity_rows, ConditionKind, QuantityRow};
use crate::domain::MarkupKind;
use crate::domain::Project;
use crate::ui::app::UiState;
use crate::ui::markups_list::{self, MarkupListAction};

/// Draw the Quantities panel with tab strip (Markups List | Quantities).
///
/// Returns an action when a row is clicked and whether the project is dirty.
pub(crate) fn draw_quantities_panel(
    ui: &mut Ui,
    project: &Project,
    state: &mut UiState,
) -> (Option<QuantityAction>, bool) {
    let mut action = None;
    let mut dirty = false;

    if !state.show_quantities || state.focus_mode {
        return (None, dirty);
    }

    egui::Panel::bottom("quantities")
        .default_size(200.0)
        .min_size(140.0)
        .max_size(400.0)
        .resizable(true)
        .show(ui, |ui| {
            // Tab strip: Markups List | Quantities (Inspector-style segmented control)
            ui.horizontal(|ui| {
                ui.add_space(4.0);

                // Markups List tab
                if state.quantities_tab_selected {
                    if ui.button("Markups List").clicked() {
                        state.quantities_tab_selected = false;
                    }
                } else {
                    ui.label(RichText::new("Markups List").strong());
                }

                ui.separator();

                // Quantities tab
                if !state.quantities_tab_selected {
                    if ui.button("Quantities").clicked() {
                        state.quantities_tab_selected = true;
                    }
                } else {
                    ui.label(RichText::new("Quantities").strong());
                }

                // Right-aligned buttons (only show Export CSV when Quantities tab is selected)
                if state.quantities_tab_selected {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Export CSV…").clicked() {
                            export_quantities_csv(project, state);
                        }
                    });
                }
            });

            ui.separator();

            if state.quantities_tab_selected {
                // Quantities table
                let rows = quantity_rows(project);
                draw_quantities_table(ui, project, state, &rows, &mut action);
            } else {
                // Markups List panel
                draw_markups_list_panel(ui, project, state, &mut action, &mut dirty);
            }
        });

    (action, dirty)
}

/// An action requested by the quantities panel.
#[derive(Debug, Clone)]
pub(crate) enum QuantityAction {
    /// Select all markups assigned to a condition and jump to its first contributor's page.
    SelectCondition { condition_id: Uuid, first_page: usize },
    /// Select all unassigned measurements.
    SelectUnassigned { count: usize },
    /// Open the condition dialog to create a new condition.
    NewCondition,
    /// Open the condition dialog to edit an existing condition.
    EditCondition { condition_id: Uuid },
    /// Delegate to MarkupListAction from the markups list.
    MarkupList(MarkupListAction),
}

/// Draw the Quantities table.
fn draw_quantities_table(
    ui: &mut Ui,
    project: &Project,
    state: &mut UiState,
    rows: &[QuantityRow],
    action: &mut Option<QuantityAction>,
) {
    if rows.is_empty() {
        // Empty state: no conditions yet (R-13)
        ui.vertical(|ui| {
            ui.add_space(12.0);
            ui.heading("No takeoff conditions yet.");
            ui.label("Conditions name what you measure — create one to start the schedule.");
            ui.add_space(8.0);
            if ui.button("New condition…").clicked() {
                *action = Some(QuantityAction::NewCondition);
            }
            ui.add_space(12.0);
        });
        return;
    }

    // Table header
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(RichText::new("Condition").strong());
        ui.label(RichText::new("Kind").strong());
        ui.label(RichText::new("Items").strong());
        ui.label(RichText::new("Quantity").strong());
        ui.label(RichText::new("Unit").strong());
    });

    ui.separator();

    // Data rows in a scroll area
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for row in rows {
                if let Some(a) = draw_quantity_row(ui, project, state, row) {
                    *action = Some(a);
                }
            }
        });
}

/// Draw the Markups List panel (delegates to markups_list module).
fn draw_markups_list_panel(
    ui: &mut Ui,
    project: &Project,
    state: &mut UiState,
    action: &mut Option<QuantityAction>,
    dirty: &mut bool,
) {
    // We need a mutable project for the markups list (it allows status changes)
    // Cast away const for the panel - the domain methods will handle mutability
    let mut project_mut = unsafe { std::mem::transmute::<&Project, &mut Project>(project) };
    let (list_action, status_changed) = markups_list::draw_markups_list(ui, project_mut, state);
    if status_changed {
        *dirty = true;
    }
    if let Some(ma) = list_action {
        *action = Some(QuantityAction::MarkupList(ma));
    }
}

/// Draw a single quantity row.
///
/// Returns an action if the row was clicked/double-clicked/Enter pressed.
fn draw_quantity_row(
    ui: &mut Ui,
    project: &Project,
    state: &mut UiState,
    row: &QuantityRow,
) -> Option<QuantityAction> {
    let is_unassigned = row.condition.id.is_nil();
    let selected = state.selected.contains(&row.condition.id);

    // Use a horizontal layout with interactive sense for the whole row
    let response = ui.horizontal(|ui| {
        // Colour swatch (24pt target with padding)
        let colour = row.condition.colour;
        let (rect, resp) = ui
            .allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::click());
        ui.painter().rect_filled(
            rect,
            2.0,
            egui::Color32::from_rgba_unmultiplied(colour.r, colour.g, colour.b, colour.a),
        );
        ui.painter().rect_stroke(
            rect,
            2.0,
            egui::Stroke::new(1.0, egui::Color32::from_black_alpha(120)),
            egui::StrokeKind::Middle,
        );

        // Selection indicator
        if selected {
            ui.painter().rect_filled(
                rect,
                2.0,
                egui::Color32::from_black_alpha(80),
            );
        }

        // Hover tooltip with hex colour (AccessKit label requirement)
        resp.on_hover_text(format!("#{:02X}{:02X}{:02X}", colour.r, colour.g, colour.b));

        let mut action = None;

        // Condition name (clickable for selection, double-click/Enter for edit)
        let name_resp = ui.selectable_label(selected, &row.condition.name);
        let name_clicked = name_resp.clicked();
        let name_double_clicked = name_resp.double_clicked();

        // Handle click / double-click / Enter for non-Unassigned rows
        if !is_unassigned {
            if name_double_clicked {
                action = Some(QuantityAction::EditCondition {
                    condition_id: row.condition.id,
                });
            } else if name_clicked {
                action = build_action(project, state, row);
            }
            // Handle Enter key when name label is focused
            if name_resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                action = Some(QuantityAction::EditCondition {
                    condition_id: row.condition.id,
                });
            }
        } else if name_clicked {
            // Unassigned row: single click selects
            action = build_action(project, state, row);
        }

        // Kind label
        let kind_label = match row.condition.kind {
            ConditionKind::Linear => "Linear",
            ConditionKind::Area => "Area",
            ConditionKind::Perimeter => "Perimeter",
            ConditionKind::WallSurface => "Wall surface",
            ConditionKind::Count => "Count",
        };
        if ui.label(kind_label).clicked() {
            if action.is_none() && !is_unassigned {
                action = build_action(project, state, row);
            }
        }

        // Item count
        let items_text = if row.item_count == 1 {
            "1"
        } else {
            &format!("{}", row.item_count)
        };
        if ui.label(items_text).clicked() {
            if action.is_none() && !is_unassigned {
                action = build_action(project, state, row);
            }
        }

        // Quantity with unit suffix
        let total_str = format_quantity(row);
        let mut qty_label = RichText::new(&total_str);
        if is_unassigned {
            qty_label = qty_label.weak();
        }
        if row.mixed_units {
            qty_label = qty_label.color(ui.visuals().warn_fg_color);
        }
        if ui.label(qty_label).clicked() {
            if action.is_none() && !is_unassigned {
                action = build_action(project, state, row);
            }
        }

        // Unit column
        let unit_name = row.unit.name();
        if ui.label(unit_name).clicked() {
            if action.is_none() && !is_unassigned {
                action = build_action(project, state, row);
            }
        }

        action
    });
    response.inner
}

/// Build an action for row click handling.
fn build_action(project: &Project, _state: &mut UiState, row: &QuantityRow) -> Option<QuantityAction> {
    let is_unassigned = row.condition.id.is_nil();

    if is_unassigned {
        // For Unassigned row, select all unassigned measurements
        Some(QuantityAction::SelectUnassigned { count: row.item_count })
    } else {
        // Find the first page with a markup assigned to this condition
        let first_page = find_first_page_for_condition(project, row.condition.id);
        Some(QuantityAction::SelectCondition {
            condition_id: row.condition.id,
            first_page,
        })
    }
}

/// Find the first page that has a markup assigned to the given condition.
fn find_first_page_for_condition(project: &Project, condition_id: Uuid) -> usize {
    for doc in &project.documents {
        for markup in &doc.markups {
            if markup.condition_id == Some(condition_id) {
                return markup.page;
            }
        }
    }
    0
}

/// Format a quantity row for display.
/// Adds the appropriate unit suffix based on the condition kind.
fn format_quantity(row: &QuantityRow) -> String {
    let total = row.total;

    // Format the number with appropriate precision
    let formatted = format!("{:.3}", total);

    // Add the unit suffix based on kind
    let suffix = row.condition.kind.unit_suffix();

    format!("{}{}", formatted, suffix)
}

/// Export quantities to CSV.
fn export_quantities_csv(project: &Project, state: &mut UiState) {
    let rows = quantity_rows(project);

    // Gate: only export when there are real conditions (not just Unassigned)
    let has_conditions = rows.iter().any(|r| !r.condition.id.is_nil());
    if !has_conditions {
        state.status = "No takeoff conditions to export".to_string();
        return;
    }

    let Some(path) = rfd::FileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name("quantities.csv")
        .save_file()
    else {
        return;
    };

    if let Err(e) = write_quantities_csv(&path, &rows) {
        state.status = format!("Export failed: {e}");
    } else {
        state.status = format!(
            "Quantities exported to {}",
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );
    }
}

/// Write quantities to CSV file (RFC 4180 compliant).
fn write_quantities_csv(path: &std::path::Path, rows: &[QuantityRow]) -> std::io::Result<()> {
    let mut content = String::new();

    // Header
    content.push_str("Condition,Kind,Unit,Quantity,Items,Height,Notes\n");

    // Data rows
    for row in rows {
        if row.condition.id.is_nil() {
            continue; // Don't export Unassigned row
        }

        let condition = &row.condition;

        // Quote fields that may contain commas
        let name = if condition.name.contains(',') || condition.name.contains('"') {
            format!("\"{}\"", condition.name.replace('"', "\"\""))
        } else {
            condition.name.clone()
        };

        let notes = condition.notes.as_deref().unwrap_or("");
        let notes_quoted = if notes.contains(',') || notes.contains('"') || notes.contains('\n') {
            format!("\"{}\"", notes.replace('"', "\"\""))
        } else {
            notes.to_string()
        };

        let height = condition.height.map_or(String::new(), |h| h.to_string());

        let line = format!(
            "{},{},{},{:.6},{},{},{}\n",
            name,
            condition.kind.label(),
            row.unit.name(),
            row.total,
            row.item_count,
            height,
            notes_quoted
        );
        content.push_str(&line);
    }

    std::fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Color;
    use chrono::Utc;

    #[test]
    fn format_quantity_linear_adds_no_suffix() {
        let row = QuantityRow {
            condition: crate::domain::takeoff::TakeoffCondition {
                id: Uuid::new_v4(),
                name: "Test".to_string(),
                kind: ConditionKind::Linear,
                height: None,
                colour: Color::default(),
                notes: None,
                created: Utc::now(),
                modified: Utc::now(),
            },
            total: 123.456,
            unit: crate::domain::units::MeasureUnit::Meter,
            item_count: 5,
            mixed_units: false,
        };

        assert_eq!(format_quantity(&row), "123.456");
    }

    #[test]
    fn format_quantity_area_adds_squared_suffix() {
        let row = QuantityRow {
            condition: crate::domain::takeoff::TakeoffCondition {
                id: Uuid::new_v4(),
                name: "Carpet".to_string(),
                kind: ConditionKind::Area,
                height: None,
                colour: Color::default(),
                notes: None,
                created: Utc::now(),
                modified: Utc::now(),
            },
            total: 45.678,
            unit: crate::domain::units::MeasureUnit::Meter,
            item_count: 3,
            mixed_units: false,
        };

        assert_eq!(format_quantity(&row), "45.678²");
    }

    #[test]
    fn format_quantity_wallsurface_adds_squared_suffix() {
        let row = QuantityRow {
            condition: crate::domain::takeoff::TakeoffCondition {
                id: Uuid::new_v4(),
                name: "Walls".to_string(),
                kind: ConditionKind::WallSurface,
                height: Some(2.7),
                colour: Color::default(),
                notes: None,
                created: Utc::now(),
                modified: Utc::now(),
            },
            total: 32.4,
            unit: crate::domain::units::MeasureUnit::Meter,
            item_count: 1,
            mixed_units: false,
        };

        assert_eq!(format_quantity(&row), "32.400²");
    }

    #[test]
    fn format_quantity_count_adds_no_suffix() {
        let row = QuantityRow {
            condition: crate::domain::takeoff::TakeoffCondition {
                id: Uuid::new_v4(),
                name: "Count".to_string(),
                kind: ConditionKind::Count,
                height: None,
                colour: Color::default(),
                notes: None,
                created: Utc::now(),
                modified: Utc::now(),
            },
            total: 4.0,
            unit: crate::domain::units::MeasureUnit::Meter,
            item_count: 4,
            mixed_units: false,
        };

        assert_eq!(format_quantity(&row), "4.000");
    }

    #[test]
    fn write_quantities_csv_handles_comma_in_name() {
        use std::io::Write;

        let temp_path = std::env::temp_dir().join("test_quantities.csv");
        let rows = vec![QuantityRow {
            condition: crate::domain::takeoff::TakeoffCondition {
                id: Uuid::new_v4(),
                name: "Carpet, Red".to_string(),
                kind: ConditionKind::Area,
                height: None,
                colour: Color::default(),
                notes: None,
                created: Utc::now(),
                modified: Utc::now(),
            },
            total: 10.5,
            unit: crate::domain::units::MeasureUnit::Meter,
            item_count: 1,
            mixed_units: false,
        }];

        write_quantities_csv(&temp_path, &rows).unwrap();

        let contents = std::fs::read_to_string(&temp_path).unwrap();
        assert!(contents.contains("\"Carpet, Red\""));
        std::fs::remove_file(&temp_path).ok();
    }
}