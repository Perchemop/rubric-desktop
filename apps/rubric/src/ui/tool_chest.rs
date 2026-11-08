//! Tool Chest (v0.4 gap #7): the left-panel "My Tools" over the domain's
//! `StylePresetLibrary`.
//!
//! The panel turns the previously invisible reusable-style domain model into a
//! workflow:
//!
//! - **Apply** arms a preset: every markup created afterwards (until the user
//!   stops it) is drawn with that preset's look. `UiState::pending_style_preset`
//!   carries the armed preset across frames and `canvas::finish_markup`
//!   consumes it at creation time (style only; stamps are excluded).
//! - **Save current tool** captures the active tool's default style.
//! - **Save selected markup** captures a whole markup (style, kind, author,
//!   subject and review status) — the "save a markup as a tool" path.
//! - Rename/delete manage the library; the per-row "→" button restyles the
//!   selected markups through `StylePreset::apply_to`.
//!
//! The library itself is app-level session state on `RubricApp` (it survives
//! project switches and is not written to `.rubric.json` yet); only changes
//! that mutate the *project* (apply-to-selection) request an undo snapshot.

use eframe::egui::{self, Ui};
use uuid::Uuid;

use crate::domain::{Color, MarkupKind, Project, StylePreset, StylePresetLibrary};

use super::app::UiState;
use super::theme::{to_color32, TEXT_CAPTION};

/// Draw the Tool Chest panel.
///
/// Returns `true` when the panel mutated the *project* this frame (the
/// apply-to-selection path), so the caller pushes an undo snapshot. Arming,
/// saving, renaming and deleting presets only touch app-level session state
/// and never snapshot.
pub(crate) fn draw_tool_chest(
    ui: &mut Ui,
    library: &mut StylePresetLibrary,
    project: &mut Project,
    ui_state: &mut UiState,
) -> bool {
    let mut project_mutated = false;

    ui.heading("My Tools");
    ui.add_space(2.0);

    // Armed-preset banner: shows what `finish_markup` will style the next
    // created markup with, with a one-click way to stop.
    if let Some(armed) = ui_state.pending_style_preset.clone() {
        ui.horizontal(|ui| {
            draw_swatch(ui, armed.style.stroke);
            ui.label(format!("Next markup style: {}", armed.name));
            if ui.button("Stop").clicked() {
                set_armed(ui_state, &armed, true);
            }
        });
    } else {
        ui.small("No active style. Pick Apply on a saved style to arm it.");
    }
    ui.separator();

    if library.is_empty() {
        ui.label("No saved styles yet.");
        ui.label(
            egui::RichText::new("Save a style below, then Apply it any time.")
                .font(TEXT_CAPTION)
                .color(egui::Color32::from_gray(120)),
        );
    } else {
        ui.label(
            egui::RichText::new(format!(
                "{} of {} saved styles",
                library.len(),
                StylePresetLibrary::MAX_PRESETS
            ))
            .font(TEXT_CAPTION)
            .color(egui::Color32::from_gray(120)),
        );
        ui.separator();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Clone the rows up front so rename/delete can mutate the
                // library while the list is being drawn (the library is
                // capped at 64 presets, so the copy is cheap).
                let presets: Vec<StylePreset> = library.iter().cloned().collect();
                for preset in &presets {
                    project_mutated |= draw_preset_row(ui, library, project, ui_state, preset);
                }
            });
    }
    ui.separator();

    draw_save_section(ui, library, project, ui_state);

    project_mutated
}

/// One saved-preset row: colour swatch, name (highlighted while armed) and
/// the Apply / apply-to-selection / rename / delete controls.
fn draw_preset_row(
    ui: &mut Ui,
    library: &mut StylePresetLibrary,
    project: &mut Project,
    ui_state: &mut UiState,
    preset: &StylePreset,
) -> bool {
    // A rename draft replaces its row with an inline editor.
    if ui_state
        .tool_chest_rename
        .as_ref()
        .is_some_and(|(id, _)| *id == preset.id)
    {
        return draw_rename_editor(ui, library, ui_state, preset);
    }

    let mut project_mutated = false;
    let is_armed = ui_state
        .pending_style_preset
        .as_ref()
        .is_some_and(|p| p.id == preset.id);
    let has_selection = !ui_state.selected.is_empty();

    ui.horizontal(|ui| {
        draw_swatch(ui, preset.style.stroke);
        if ui
            .selectable_label(is_armed, preset.name.as_str())
            .on_hover_text(format!("{} style", preset.kind.label()))
            .clicked()
        {
            // The name doubles as the arm toggle, mirroring the tool
            // palette's selectable labels.
            set_armed(ui_state, preset, is_armed);
        }
        let apply_label = if is_armed { "Stop" } else { "Apply" };
        if ui
            .add_sized(egui::vec2(42.0, 24.0), egui::Button::new(apply_label))
            .on_hover_text(if is_armed {
                "Stop styling new markups with this style"
            } else {
                "Style every markup created from now on with this style"
            })
            .clicked()
        {
            set_armed(ui_state, preset, is_armed);
        }

        // Restyle the current selection with the full preset (style plus the
        // author/subject/status fill rules of `apply_to`). Only offered when
        // something is selected. This is the one row action that mutates the
        // project, so it asks for an undo snapshot.
        if has_selection
            && ui
                .add_sized(
                    egui::vec2(84.0, 24.0),
                    egui::Button::new("Apply to Selection"),
                )
                .on_hover_text("Apply this style to every selected markup")
                .clicked()
        {
            project_mutated = apply_preset_to_selection(project, ui_state, preset);
        // P1-6: disable Apply to Selection when panel is too narrow.
        if has_selection && ui.state().rect.width() < 180.0 {
            ui.add_enabled(false, egui::Button::new("Apply to Selection"))
                .on_hover_text("Apply this style to every selected markup");
        }

        if ui
            .add_sized(egui::vec2(56.0, 24.0), egui::Button::new("Rename"))
            .on_hover_text("Rename this style")
            .clicked()
        {
            ui_state.tool_chest_rename = Some((preset.id, preset.name.clone()));
        }
        if ui
            .add_sized(egui::vec2(56.0, 24.0), egui::Button::new("Delete"))
            .on_hover_text("Delete this style")
            .clicked()
        {
            // P2: Set delete confirmation flag instead of immediate delete
            ui_state.tool_chest_delete_confirm = Some(preset.id);}
        {
            library.remove(preset.id);
            if is_armed {
                ui_state.pending_style_preset = None;
            }
            if ui_state
                .tool_chest_rename
                .as_ref()
                .is_some_and(|(id, _)| *id == preset.id)
            {
                ui_state.tool_chest_rename = None;
            }
            ui_state.status = format!("Style '{}' deleted", preset.name);
        }
    });
    project_mutated
}

/// Inline rename editor shown in place of a preset row while its rename draft
/// is open. Never mutates the project (always returns `false`).
fn draw_rename_editor(
    ui: &mut Ui,
    library: &mut StylePresetLibrary,
    ui_state: &mut UiState,
    preset: &StylePreset,
) -> bool {
    let mut close = false;
    ui.horizontal(|ui| {
        let Some((_, draft)) = ui_state.tool_chest_rename.as_mut() else {
            return;
        };
        ui.add(
            egui::TextEdit::singleline(draft)
                .desired_width(110.0)
                .id(egui::Id::new(("tool_chest_rename", preset.id))),
        );
        if ui.button("Save").clicked() {
            let new_name = draft.trim().to_string();
            if new_name.is_empty() {
                ui_state.status = "Style name cannot be empty".to_string();
            } else if library.rename(preset.id, new_name.clone()) {
                // Keep the armed copy in sync so the banner and the row
                // agree on the new name.
                if let Some(armed) = ui_state
                    .pending_style_preset
                    .as_mut()
                    .filter(|armed| armed.id == preset.id)
                {
                    armed.name = new_name.clone();
                }
                ui_state.status = format!("Style renamed to '{new_name}'");
                close = true;
            } else {
                ui_state.status = "Style no longer exists".to_string();
                close = true;
            }
        }
        if ui.button("Cancel").clicked() {
            close = true;
        }
    });
    if close {
        ui_state.tool_chest_rename = None;
    }
    false
}

/// "Save Style" section: a shared name draft with two capture paths — the
/// active tool's default style, and (when exactly one markup is selected) a
/// full capture of that markup via `StylePreset::from_markup`. Both save
/// through `save_named`, so re-using a name updates one entry.
fn draw_save_section(
    ui: &mut Ui,
    library: &mut StylePresetLibrary,
    project: &mut Project,
    ui_state: &mut UiState,
) {
    ui.heading("Save Style");
    ui.add(
        egui::TextEdit::singleline(&mut ui_state.tool_chest_save_name)
            .hint_text("Style name")
            .desired_width(ui.available_width())
            .id(egui::Id::new("tool_chest_save_name")),
    );
    let draft_name = ui_state.tool_chest_save_name.trim().to_string();
    let name_ok = !draft_name.is_empty();
    let kind = ui_state.active_tool.to_kind();

    ui.horizontal(|ui| {
        // Always rendered, disabled with an explanatory hint when the active
        // tool has no style to capture (Tool::Select).
        if ui
            .add_enabled(
                name_ok && kind.is_some(),
                egui::Button::new("Save Current Style"),
            )
            .on_hover_text("Capture the active tool's default style as a named style")
            .on_disabled_hover_text("Pick a drawing tool and enter a name")
            .clicked()
        {
            save_current_tool(library, project, ui_state, &draft_name, kind);
        }
        let single_id = single_selected_id(ui_state);
        let markup_enabled = name_ok && single_id.is_some();
        if ui
            .add_enabled(markup_enabled, egui::Button::new("Save Markup as Style"))
            .on_hover_text("Capture the selected markup's style and metadata as a named style")
            .on_disabled_hover_text("Select exactly one markup and enter a name")
            .clicked()
        {
            save_selected_markup(library, project, ui_state, &draft_name, single_id);
        }
    });
    if kind.is_none() && single_selected_id(ui_state).is_none() {
        ui.small("Pick a drawing tool (or select a markup) to capture its style.");
    }
}

/// Capture the active tool's default style as a named preset (the "Save
/// current tool" button). `kind` is `None` only when the button was disabled
/// (Tool::Select), in which case this is a no-op.
fn save_current_tool(
    library: &mut StylePresetLibrary,
    project: &mut Project,
    ui_state: &mut UiState,
    name: &str,
    kind: Option<MarkupKind>,
) {
    let Some(kind) = kind else {
        return;
    };
    let author = project.settings.author.clone();
    let style = kind.default_style();
    library.save_named(StylePreset::from_style(name, kind, style, author));
    ui_state.tool_chest_save_name.clear();
    ui_state.status = format!("Style '{name}' saved");
}

/// Capture the single selected markup as a named preset via
/// `StylePreset::from_markup` (the "Save selected markup" button). `id` is
/// `None` only when the button was disabled (no single selection).
fn save_selected_markup(
    library: &mut StylePresetLibrary,
    project: &mut Project,
    ui_state: &mut UiState,
    name: &str,
    id: Option<Uuid>,
) {
    let Some(id) = id else {
        return;
    };
    let captured = project
        .active()
        .and_then(|doc| doc.find_markup(id))
        .map(|m| StylePreset::from_markup(m, name));
    if let Some(preset) = captured {
        library.save_named(preset);
        ui_state.tool_chest_save_name.clear();
        ui_state.status = format!("Style '{name}' saved");
    }
}

/// Arm (or stop) a Tool Chest style preset. While armed, `canvas::finish_markup`
/// styles every non-stamp markup created from now on with the preset's look.
fn set_armed(ui_state: &mut UiState, preset: &StylePreset, stop: bool) {
    if stop {
        ui_state.pending_style_preset = None;
        ui_state.status = format!("Style '{}' stopped", preset.name);
    } else {
        ui_state.pending_style_preset = Some(preset.clone());
        ui_state.status = format!("Style '{}' active — next markup uses it", preset.name);
    }
}

/// Apply `preset` to every selected editable markup, following the documented
/// `StylePreset::apply_to` rules (restyle, fill empty author/subject, set
/// status, skip locked targets). Returns `true` when at least one markup was
/// touched, so the caller pushes an undo snapshot.
fn apply_preset_to_selection(
    project: &mut Project,
    ui_state: &mut UiState,
    preset: &StylePreset,
) -> bool {
    let ids: Vec<Uuid> = ui_state.selected.iter().copied().collect();
    let mut changed = 0usize;
    if let Some(doc) = project.active_mut() {
        for id in &ids {
            let editable = doc
                .find_markup(*id)
                .is_some_and(|m| doc.is_markup_editable(m));
            if !editable {
                continue;
            }
            if let Some(markup) = doc.find_markup_mut(*id) {
                preset.apply_to(markup);
                changed += 1;
            }
        }
        if changed > 0 {
            doc.touch();
        }
    }
    ui_state.status = if changed == 0 {
        "No editable markups selected".to_string()
    } else {
        format!("'{}' applied to {changed} markup(s)", preset.name)
    };
    changed > 0
}

/// The selected markup id when exactly one markup is selected.
fn single_selected_id(ui_state: &UiState) -> Option<Uuid> {
    let mut ids = ui_state.selected.iter();
    let id = *ids.next()?;
    ids.next().is_none().then_some(id)
}

/// Small filled colour square previewing a preset's stroke colour.
fn draw_swatch(ui: &mut Ui, color: Color) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 3.0, to_color32(color));
}

// P2: Tool Chest delete confirmation dialog
// Shows a confirmation window when Delete is clicked on a style preset
if ui_state.tool_chest_delete_confirm.is_some() {
    let (delete_id, delete_name) = ui_state.tool_chest_delete_confirm.take().unwrap();
    let delete_confirmed = egui::Window::new("Delete Confirmation")
        .open(&mut true)
        .resizable(false)
        .fixed_size(egui::vec2(300.0, 150.0))
        .show(ui.ctx(), |ui| {
            ui.label(format!("Delete style '{}'?", delete_name));
            ui.separator();
            if ui.button("Confirm").clicked() {
                library.remove(delete_id);
                ui_state.status = format!("Style '{}' deleted", delete_name);
            }
            if ui.button("Cancel").clicked() {}
        });
    // The window is immediately dropped each frame since open defaults to true;
    // the confirmation action happens inside the .show() closure.
}}

