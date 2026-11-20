//! Quantities panel orchestration.
//!
//! Extracted from `ui/app.rs` to reduce the god-file size.

use egui::Ui;

use crate::domain::takeoff::ConditionKind;
use crate::domain::style::Color;
use crate::ui::app::RubricApp;
use crate::ui::quantities_panel::{draw_quantities_panel as draw_qty_panel, QuantityAction};

/// Draw the Quantities panel (bottom-panel tab with takeoff schedule).
///
/// Returns true when the project needs to be marked dirty (rare for this panel).
pub(crate) fn draw_quantities_panel(app: &mut RubricApp, ui: &mut Ui) -> bool {
    let mut dirty = false;

    if app.ui.show_quantities && !app.ui.focus_mode {
        let project = app.history.current_mut();
        let ui_state = &mut app.ui;
        let (action, panel_dirty) = draw_qty_panel(ui, project, ui_state);
        dirty = panel_dirty;

        // Handle actions that open the condition dialog
        if let Some(act) = action {
            match act {
                QuantityAction::NewCondition => {
                    // Generate a unique default name
                    let conditions = &project.takeoff.conditions;
                    let mut n = 1;
                    let name = loop {
                        let name = format!("Condition {}", n);
                        let normalized = name.to_lowercase();
                        if !conditions.iter().any(|c| c.name.to_lowercase() == normalized) {
                            break name;
                        }
                        n += 1;
                    };
                    
                    // Get default colour for Linear kind
                    let colour = Color::rgb(0, 128, 255);
                    
                    app.ui.takeoff_dialog = Some(crate::ui::app::TakeoffDialogState {
                        editing_id: None,
                        name,
                        kind: ConditionKind::Linear,
                        height: String::new(),
                        height_error: false,
                        colour,
                        notes: String::new(),
                        original_name: String::new(),
                        has_assignments: false,
                        assigned_count: 0,
                        focus_name: true,
                        show_delete_confirm: false,
                    });
                }
                QuantityAction::EditCondition { condition_id } => {
                    if let Some(cond) = project.takeoff.conditions.iter().find(|c| c.id == condition_id) {
                        // Count assigned markups
                        let mut assigned_count = 0;
                        for doc in &project.documents {
                            for m in &doc.markups {
                                if m.condition_id == Some(condition_id) {
                                    assigned_count += 1;
                                }
                            }
                        }
                        
                        app.ui.takeoff_dialog = Some(crate::ui::app::TakeoffDialogState {
                            editing_id: Some(condition_id),
                            name: cond.name.clone(),
                            kind: cond.kind,
                            height: cond.height.map(|h| h.to_string()).unwrap_or_default(),
                            height_error: false,
                            colour: cond.colour,
                            notes: cond.notes.clone().unwrap_or_default(),
                            original_name: cond.name.clone(),
                            has_assignments: assigned_count > 0,
                            assigned_count,
                            focus_name: true,
                            show_delete_confirm: false,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    dirty
}