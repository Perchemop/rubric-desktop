//! Quantities panel: bottom-panel tab strip with takeoff condition schedule.
//!
//! Displays a table of conditions with their aggregated quantities,
//! supporting assignment tracking and selection of contributing markups.

use std::collections::HashSet;

use eframe::egui::{self, Button, RichText, Ui, Vec2};
use uuid::Uuid;

use crate::domain::takeoff::{quantity_rows, ConditionKind, QuantityRow};
use crate::domain::units::MeasureUnit;
use crate::domain::Project;
use crate::ui::app::UiState;

/// Action returned from the quantities panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QuantitiesPanelAction {
    pub(crate) condition_id: Uuid,
    pub(crate) page: usize,
}

/// Column widths for the quantities table (in pixels).
const COLUMN_WIDTHS: [f32; 5] = [24.0, 160.0, 100.0, 60.0, 140.0];

/// Draw the Quantities panel with tab strip.
///
/// Returns (action, status_changed) where action requests a selection change.
pub(crate) fn draw_quantities_panel(
    ui: &mut Ui,
    project: &mut Project,
    state: &mut UiState,
) -> (Option<QuantitiesPanelAction>, bool) {
    let mut action = None;
    let mut status_changed = false;

    // Tab strip: Markups List | Quantities
    ui.horizontal(|ui| {
        ui.add_space(4.0);

        // Markups List tab
        if !state.quantities_tab_selected {
            if ui.add(Button::new("Markups List")).clicked() {
                state.quantities_tab_selected = false;
                state.show_quantities = false;
                state.show_markups_list = true;
            }
        } else {
            ui.label(RichText::new("Markups List").weak());
        }

        ui.separator();

        // Quantities tab (current)
        if state.quantities_tab_selected {
            ui.label(RichText::new("Quantities").strong());
        } else {
            if ui.add(Button::new("Quantities")).clicked() {
                state.quantities_tab_selected = true;
                state.show_quantities = true;
                state.show_markups_list = false;
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(Button::new("Export CSV…")).clicked() {
                export_quantities_csv(project, state);
            }
        });
    });

    ui.separator();

    let rows = quantity_rows(project);

    if rows.is_empty() {
        // Empty state: no conditions yet
        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.heading("No takeoff conditions yet.");
            ui.label("Conditions name what you measure — create one to start the schedule.");
            ui.add_space(12.0);
            if ui.add(Button::new("New condition…")).clicked() {
                state.status = "New condition action not yet implemented".to_string();
            }
            ui.add_space(16.0);
        });
        return (action, status_changed);
    }

    // Quantities table header
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.add(egui::Label::new(RichText::new("Condition").strong()).wrap(false));
        ui.add(egui::Label::new(RichText::new("Kind").strong()).wrap(false));
        ui.add(egui::Label::new(RichText::new("Items").strong()).wrap(false));
        ui.add(egui::Label::new(RichText::new("Quantity").strong()).wrap(false));
    });

    ui.separator();

    // Quantities table body
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for row in &rows {
                let row_action = draw_quantity_row(ui, project, state, row);
                if row_action.is_some() {
                    action = row_action;
                }
            }
        });

    (action, status_changed)
}

/// Draw a single quantity row.
fn draw_quantity_row(
    ui: &mut Ui,
    project: &mut Project,
    state: &mut UiState,
    row: &QuantityRow,
) -> Option<QuantitiesPanelAction> {
    let is_unassigned = row.condition.id.is_nil();
    let selected = state.selected.contains(&row.condition.id);
    let dark = ui.visuals().dark_mode;

    ui.horizontal(|ui| {
        ui.add_space(4.0);

        // Colour swatch
        let colour = row.condition.colour;
        let swatch_colour = egui::Color32::from_rgba_unmultiplied(colour.r, colour.g, colour.b, colour.a);
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::click());
        ui.painter().rect_filled(rect, 2.0, swatch_colour);
        ui.painter().rect_stroke(
            rect,
            2.0,
            egui::Stroke::new(1.0, if dark { egui::Color32::WHITE } else { egui::Color32::BLACK }.gamma_multiply(0.3)),
            egui::StrokeKind::Middle,
        );

        // AccessKit label for icon-only control
        let label = if is_unassigned { "Unassigned measurements" } else { &row.condition.name };
        ui.ctx().accesskit_root().write(|tree| {
            if let Some(node) = tree.get(resp.id) {
                let mut node = node;
                node.set_name(label);
            }
        });

        let clicked = resp.clicked();

        // Condition name
        let name_label = if is_unassigned {
            RichText::new("Unassigned").italics().weak()
        } else if selected {
            RichText::new(&row.condition.name).strong()
        } else {
            RichText::new(&row.condition.name)
        };

        if ui.add(egui::Label::new(name_label).wrap(false)).clicked() || clicked {
            return Some(select_markups(project, state, row, is_unassigned));
        }

        // Kind label
        let kind_text = if is_unassigned {
            String::new()
        } else {
            match row.condition.kind {
                ConditionKind::Linear => "Linear",
                ConditionKind::Area => "Area",
                ConditionKind::Perimeter => "Perimeter",
                ConditionKind::WallSurface => "Wall surface",
                ConditionKind::Count => "Count",
            }.to_string()
        };

        if is_unassigned {
            ui.add(egui::Label::new("").wrap(false));
        } else {
            ui.add(egui::Label::new(kind_text).wrap(false));
        }

        // Item count
        let items_text = if row.item_count == 1 {
            "1".to_string()
        } else {
            format!("{}", row.item_count)
        };
        if ui.add(egui::Label::new(&items_text).wrap(false)).clicked() || clicked {
            return Some(select_markups(project, state, row, is_unassigned));
        }

        // Quantity with unit suffix
        let total_str = format_quantity(row);
        if ui.add(egui::Label::new(RichText::new(&total_str).monospace()).wrap(false)).clicked() || clicked {
            return Some(select_markups(project, state, row, is_unassigned));
        }

        // Mixed units badge
        if row.mixed_units && !is_unassigned {
            ui.add_space(4.0);
            let tooltip_text = format!("Mixed units: contributors span multiple display units. Total shown in project default unit ({}).", project.default_unit().name());
            if ui.add(Button::new("⚠").small()).on_hover_text(tooltip_text).clicked() {
                state.status = "Mixed units — total shown in project default unit".to_string();
            }
        }

        None
    }).inner
}

/// Format a quantity row for display.
fn format_quantity(row: &QuantityRow) -> String {
    let suffix = match row.condition.kind {
        ConditionKind::Area | ConditionKind::WallSurface => "²",
        _ => "",
    };

    // Use tabular numbers via monospace font
    format!("{:.3}{}", row.total, suffix)
}

/// Select all markups assigned to a condition (or unassigned) and jump to the first contributor's page.
fn select_markups(
    project: &mut Project,
    state: &mut UiState,
    row: &QuantityRow,
    is_unassigned: bool,
) -> QuantitiesPanelAction {
    let mut found_page = None;
    let mut selected_ids = Vec::new();

    if is_unassigned {
        for doc in &mut project.documents {
            for markup in &mut doc.markups {
                if markup.condition_id.is_none() && markup.kind.is_measure() {
                    selected_ids.push(markup.id);
                    if found_page.is_none() {
                        found_page = Some(markup.page);
                    }
                }
            }
        }
    } else {
        for doc in &mut project.documents {
            for markup in &mut doc.markups {
                if markup.condition_id == Some(row.condition.id) {
                    selected_ids.push(markup.id);
                    if found_page.is_none() {
                        found_page = Some(markup.page);
                    }
                }
            }
        }
    }

    state.selected.clear();
    for id in selected_ids {
        state.selected.insert(id);
    }

    let count = selected_ids.len();
    let page = found_page.unwrap_or(state.current_page);

    if count > 0 {
        if page != state.current_page {
            state.status = format!(
                "Selected {} markup{} on page {}",
                count,
                if count == 1 { "" } else { "s" },
                page + 1
            );
        } else {
            state.status = format!(
                "Selected {} markup{}",
                count,
                if count == 1 { "" } else { "s" }
            );
        }
    }

    QuantitiesPanelAction { condition_id: row.condition.id, page }
}

/// Export quantities to CSV.
fn export_quantities_csv(project: &mut Project, state: &mut UiState) {
    let rows = quantity_rows(project);

    // Gate: only export when there are conditions (not just Unassigned)
    if rows.iter().all(|r| r.condition.id.is_nil()) {
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
                .unwrap_or_default()
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

        // Quote fields that may contain commas, quotes, or newlines
        let name = if condition.name.contains(',') || condition.name.contains('"') || condition.name.contains('\n') {
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
    use uuid::Uuid;

    fn make_row(kind: ConditionKind, total: f64, unit: MeasureUnit) -> QuantityRow {
        QuantityRow {
            condition: crate::domain::takeoff::TakeoffCondition {
                id: Uuid::new_v4(),
                name: "Test".to_string(),
                kind,
                height: None,
                colour: Color::default(),
                notes: None,
                created: Utc::now(),
                modified: Utc::now(),
            },
            total,
            unit,
            item_count: 1,
            mixed_units: false,
        }
    }

    #[test]
    fn format_quantity_linear_no_suffix() {
        let row = make_row(ConditionKind::Linear, 123.456, MeasureUnit::Meter);
        assert_eq!(format_quantity(&row), "123.456");
    }

    #[test]
    fn format_quantity_area_squared_suffix() {
        let row = make_row(ConditionKind::Area, 45.678, MeasureUnit::Meter);
        assert_eq!(format_quantity(&row), "45.678²");
    }

    #[test]
    fn format_quantity_wall_surface_squared_suffix() {
        let row = make_row(ConditionKind::WallSurface, 32.4, MeasureUnit::Meter);
        assert_eq!(format_quantity(&row), "32.400²");
    }

    #[test]
    fn format_quantity_perimeter_no_suffix() {
        let row = make_row(ConditionKind::Perimeter, 12.0, MeasureUnit::Meter);
        assert_eq!(format_quantity(&row), "12.000");
    }

    #[test]
    fn format_quantity_count_no_suffix() {
        let row = make_row(ConditionKind::Count, 4.0, MeasureUnit::Each);
        assert_eq!(format_quantity(&row), "4.000");
    }
}