//! Quantities panel orchestration.
//!
//! The Quantities content now draws inside the unified bottom panel
//! (`ui/app_bottom_panel.rs`, UI redesign T3). This module owns the
//! dialog-opening action handling (`NewCondition` / `EditCondition`,
//! mirroring the Inspector flow): the panel only *requests*; this handler
//! performs, so snapshot/status stay on a single path.

use crate::domain::takeoff::ConditionKind;
use crate::domain::style::Color;
use crate::ui::app::RubricApp;
use crate::ui::quantities_panel::QuantityAction;

/// Handle a Quantities panel action that opens the condition dialog (T8
/// state lives in the app shell; the panel only requests it).
pub(crate) fn handle_quantity_action(app: &mut RubricApp, act: QuantityAction) {
    let project = app.history.current_mut();
    let ui_state = &mut app.ui;
    match act {
        QuantityAction::NewCondition => {
            // Generate a unique default name
            let conditions = &project.takeoff.conditions;
            let mut n = 1;
            let name = loop {
                let name = format!("Condition {}", n);
                let normalized = name.to_lowercase();
                if !conditions
                    .iter()
                    .any(|c| c.name.to_lowercase() == normalized)
                {
                    break name;
                }
                n += 1;
            };

            // Get default colour for Linear kind
            let colour = Color::rgb(0, 128, 255);

            ui_state.takeoff_dialog = Some(crate::ui::app::TakeoffDialogState {
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
            if let Some(cond) = project
                .takeoff
                .conditions
                .iter()
                .find(|c| c.id == condition_id)
            {
                // Count assigned markups
                let mut assigned_count = 0;
                for doc in &project.documents {
                    for m in &doc.markups {
                        if m.condition_id == Some(condition_id) {
                            assigned_count += 1;
                        }
                    }
                }

                ui_state.takeoff_dialog = Some(crate::ui::app::TakeoffDialogState {
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
    }
}
