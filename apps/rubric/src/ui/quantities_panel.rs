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
use crate::domain::Project;
use crate::ui::app::{StatusKind, UiState};

/// An action requested by the quantities panel.
///
/// Row-click selection is performed inside the panel (R-17) via
/// [`select_markups`]; only dialog-opening requests bubble up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QuantityAction {
    /// Open the condition dialog to create a new condition.
    NewCondition,
    /// Open the condition dialog to edit an existing condition.
    EditCondition { condition_id: uuid::Uuid },
}

/// Draw the Quantities tab content inside the unified bottom panel
/// (UI redesign T3). The panel wrapper and tab strip live in
/// `app_bottom_panel.rs`; this draws only the toolbar + table.
///
/// Returns (action, dirty) where action requests opening the condition dialog.
pub(crate) fn draw_quantities_content(
    ui: &mut Ui,
    project: &Project,
    state: &mut UiState,
) -> (Option<QuantityAction>, bool) {
    let mut action = None;
    let dirty = false;

    // Toolbar row
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Export CSV…").clicked() {
                export_quantities_csv(project, state);
            }
        });
    });

    ui.separator();

    let rows = quantity_rows(project);

    if rows.is_empty() {
        // Empty state: no conditions yet (R-13)
        ui.vertical(|ui| {
            ui.add_space(12.0);
            ui.heading("No takeoff conditions yet.");
            ui.label("Conditions name what you measure — create one to start the schedule.");
            ui.add_space(8.0);
            if ui.button("New condition…").clicked() {
                action = Some(QuantityAction::NewCondition);
            }
            ui.add_space(12.0);
        });
        return (action, dirty);
    }

    // Quantities table header
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
            for row in &rows {
                if let Some(a) = draw_quantity_row(ui, project, state, row) {
                    action = Some(a);
                }
            }
        });

    (action, dirty)
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

    ui.horizontal(|ui| {
        ui.add_space(4.0);

        // Colour swatch
        let colour = row.condition.colour;
        let swatch_colour = egui::Color32::from_rgba_unmultiplied(colour.r, colour.g, colour.b, colour.a);
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::click());
        ui.painter().rect_filled(rect, 2.0, swatch_colour);
        ui.painter().rect_stroke(
            rect,
            2.0,
            ui.visuals().widgets.inactive.bg_stroke,
            egui::StrokeKind::Middle,
        );

        // Selection indicator
        if selected {
            ui.painter().rect_filled(
                rect,
                2.0,
                ui.visuals().selection.bg_fill.gamma_multiply(0.35),
            );
        }

        let clicked = resp.clicked();
        let double_clicked = resp.double_clicked();

        let mut action = None;

        // Condition name
        let name_resp = ui.selectable_label(selected, &row.condition.name);
        let name_clicked = name_resp.clicked();
        let name_double_clicked = name_resp.double_clicked();

        // Handle click / double-click / Enter for non-Unassigned rows
        if !is_unassigned {
            if name_double_clicked || double_clicked {
                action = Some(QuantityAction::EditCondition {
                    condition_id: row.condition.id,
                });
            } else if name_clicked || clicked {
                select_markups(project, row.condition.id, is_unassigned, state);
            }
            // Handle Enter key when name label is focused
            if name_resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                action = Some(QuantityAction::EditCondition {
                    condition_id: row.condition.id,
                });
            }
        } else if name_clicked || clicked {
            // Unassigned row: single click selects
            select_markups(project, row.condition.id, is_unassigned, state);
        }

        // Kind label
        let kind_text = match row.condition.kind {
            ConditionKind::Linear => "Linear",
            ConditionKind::Area => "Area",
            ConditionKind::Perimeter => "Perimeter",
            ConditionKind::WallSurface => "Wall surface",
            ConditionKind::Count => "Count",
        };
        if is_unassigned {
            ui.add(egui::Label::new(" "));
        } else if ui.label(kind_text).clicked() && action.is_none() {
            select_markups(project, row.condition.id, is_unassigned, state);
        }

        // Item count
        let items_text = if row.item_count == 1 {
            "1"
        } else {
            &format!("{}", row.item_count)
        };
        if ui.label(items_text).clicked() && action.is_none() {
            select_markups(project, row.condition.id, is_unassigned, state);
        }

        // Quantity with unit suffix
        let total_str = format_quantity(row);
        if ui.label(total_str).clicked() && action.is_none() {
            select_markups(project, row.condition.id, is_unassigned, state);
        }

        // Mixed units badge
        if row.mixed_units && !is_unassigned {
            ui.add_space(4.0);
            let tooltip = format!(
                "Mixed units: contributors span multiple display units. Total shown in project default unit ({}).",
                row.unit.name()
            );
            if ui
                .add(egui::Button::new("⚠").small())
                .on_hover_text(tooltip)
                .clicked()
            {
                state.set_status(StatusKind::Info, "Mixed units — total shown in project default unit");
            }
        }

        // Unit column
        let unit_name = row.unit.name();
        if ui.label(unit_name).clicked() && action.is_none() {
            select_markups(project, row.condition.id, is_unassigned, state);
        }

        action
    }).inner
}

/// Format a quantity row for display.
/// Adds the appropriate unit suffix based on the condition kind.
fn format_quantity(row: &QuantityRow) -> String {
    let total = row.total;

    // Format the number with appropriate precision
    let formatted = format!("{:.3}", total);

    // Add the unit suffix based on kind
    let suffix = match row.condition.kind {
        ConditionKind::Area | ConditionKind::WallSurface => "²",
        _ => "",
    };

    format!("{}{}", formatted, suffix)
}

/// Select all markups assigned to a condition (or unassigned) and update selection state.
fn select_markups(project: &Project, condition_id: Uuid, is_unassigned: bool, state: &mut UiState) {
    let mut found_page = None;
    let mut selected_ids = Vec::new();

    for doc in &project.documents {
        for markup in &doc.markups {
            let matches = if is_unassigned {
                markup.condition_id.is_none() && markup.kind.is_measure()
            } else {
                markup.condition_id == Some(condition_id)
            };

            if matches {
                selected_ids.push(markup.id);
                if found_page.is_none() {
                    found_page = Some(markup.page);
                }
            }
        }
    }

    state.selected.clear();
    let count = selected_ids.len();
    for id in selected_ids {
        state.selected.insert(id);
    }

    if count > 0 {
        state.set_status(
            StatusKind::Info,
            format!(
                "Selected {} markup{}",
                count,
                if count == 1 { "" } else { "s" }
            ),
        );
        // R-17: jump to the first contributor's page (same contract as the
        // Markups List rows). The panel owns session UI state directly.
        if let Some(page) = found_page {
            if page != state.current_page {
                state.current_page = page;
            }
        }
    }
}

/// Export quantities to CSV.
fn export_quantities_csv(project: &Project, state: &mut UiState) {
    let rows = quantity_rows(project);

    // Gate: only export when there are real conditions (not just Unassigned)
    let has_conditions = rows.iter().any(|r| !r.condition.id.is_nil());
    if !has_conditions {
        state.set_status(StatusKind::Warning, "No takeoff conditions to export");
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
        state.set_status(StatusKind::Error, format!("Export failed: {e}"));
    } else {
        state.set_status(
            StatusKind::Success,
            format!(
                "Quantities exported to {}",
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ),
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

    // Atomic write: write to temp file, then rename
    let temp_path = path.with_extension("csv.tmp");
    std::fs::write(&temp_path, content)?;
    std::fs::rename(temp_path, path)?;

    Ok(())
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
    fn format_quantity_wall_surface_adds_squared_suffix() {
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
            unit: crate::domain::units::MeasureUnit::Point,
            item_count: 4,
            mixed_units: false,
        };

        assert_eq!(format_quantity(&row), "4.000");
    }

    #[test]
    fn write_quantities_csv_handles_comma_in_name() {
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
