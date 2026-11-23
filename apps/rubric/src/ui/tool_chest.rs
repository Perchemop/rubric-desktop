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
use std::cell::RefCell;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Color, Markup, MarkupKind, Project, StylePreset, StylePresetLibrary};

use super::app::UiState;
use super::theme::{to_color32, TEXT_CAPTION};

// Session-only Drawing/Style mode per preset id. `true` = Drawing mode
// (re-place an exact duplicate), `false` = Style mode (re-style only). Stored
// here because `UiState` (app.rs) cannot be edited from this slice; a future
// `tool_chest_modes: HashMap<Uuid, bool>` field on `UiState` would be cleaner
// and is reported as the preferred home.
thread_local! {
    static DRAWING_MODE: RefCell<HashMap<Uuid, bool>> = RefCell::new(HashMap::new());
}

fn drawing_mode_for(preset: &StylePreset) -> bool {
    // Presets without captured geometry can only ever be Style mode.
    if !preset.can_draw() {
        return false;
    }
    DRAWING_MODE.with(|m| *m.borrow().get(&preset.id).unwrap_or(&true))
}

fn set_drawing_mode(id: Uuid, drawing: bool) {
    DRAWING_MODE.with(|m| {
        m.borrow_mut().insert(id, drawing);
    });
}

thread_local! {
    static RECENT_EXPANDED: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
}

thread_local! {
    static SET_FILTER: std::cell::RefCell<Option<Uuid>> = const { std::cell::RefCell::new(None) };
}

fn set_filter() -> Option<Uuid> {
    SET_FILTER.with(|c| *c.borrow())
}

fn set_set_filter(v: Option<Uuid>) {
    SET_FILTER.with(|c| *c.borrow_mut() = v);
}

thread_local! {
    static RENAME_SET: std::cell::RefCell<Option<(Uuid, String)>> =
        const { std::cell::RefCell::new(None) };
}

fn persist_library(library: &StylePresetLibrary) {
    let mut prefs = crate::data::load_prefs();
    prefs.tool_chest_library = library.clone();
    let _ = crate::data::save_prefs(&prefs);
}

fn recent_expanded() -> bool {
    RECENT_EXPANDED.with(|c| *c.borrow())
}

fn set_recent_expanded(value: bool) {
    RECENT_EXPANDED.with(|c| *c.borrow_mut() = value);
}

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
            let mode = if armed.geometry.is_some() {
                "Drawing"
            } else {
                "Style"
            };
            ui.label(format!("Next markup: {} ({})", armed.name, mode));
            if ui.button("Stop").clicked() {
                set_armed(ui_state, &armed, true);
            }
        });
    } else {
        ui.small("No active style. Pick Apply on a saved style to arm it.");
    }
    ui.separator();

    // Recent Tools (gap #3): session-temporary auto-recorded presets, shown
    // at the top of the chest. Omitted entirely when empty (no empty header).
    if !ui_state.recent_tools.is_empty() {
        ui.heading("Recent");
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            if ui
                .add_sized(egui::vec2(56.0, 24.0), egui::Button::new("Clear"))
                .on_hover_text("Clear the recent tools list")
                .clicked()
            {
                ui_state.recent_tools.clear();
                set_recent_expanded(false);
                ui_state.status = "Recent tools cleared".to_string();
            }
            ui.label("Max");
            let mut max = ui_state.tool_chest_max_recents as i32;
            let resp = ui.add(egui::DragValue::new(&mut max).range(1..=32));
            if resp.changed() {
                let clamped = max.clamp(1, 32) as usize;
                ui_state.tool_chest_max_recents = clamped;
                ui_state.recent_tools.truncate(clamped);
            }
        });
        let max = ui_state.tool_chest_max_recents.clamp(1, 32);
        let expanded = recent_expanded();
        let visible = visible_recent_count(ui_state.recent_tools.len(), max, expanded);
        let recents: Vec<StylePreset> = ui_state
            .recent_tools
            .iter()
            .take(visible)
            .cloned()
            .collect();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(160.0)
            .show(ui, |ui| {
                for preset in &recents {
                    project_mutated |= draw_recent_row(ui, project, ui_state, preset);
                }
            });
        if ui_state.recent_tools.len() > visible {
            let more = ui_state.recent_tools.len() - visible;
            if ui
                .add_sized(
                    egui::vec2(120.0, 24.0),
                    egui::Button::new(format!("⋯ {more} more")),
                )
                .clicked()
            {
                set_recent_expanded(true);
            }
        } else if expanded
            && ui_state.recent_tools.len() > 4
            && ui
                .add_sized(egui::vec2(100.0, 24.0), egui::Button::new("Show less"))
                .clicked()
        {
            set_recent_expanded(false);
        }
        ui.separator();
    }

    // Set filter dropdown + management.
    let sets = library.sets.clone();
    ui.horizontal(|ui| {
        let current = set_filter();
        let label = current
            .and_then(|id| sets.iter().find(|s| s.id == id).map(|s| s.name.clone()))
            .unwrap_or_else(|| "All presets".to_string());
        let set_filter_resp = egui::ComboBox::from_id_salt("tool_set_filter")
            .selected_text(label)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(current.is_none(), "All presets")
                    .clicked()
                {
                    set_set_filter(None);
                }
                for s in &sets {
                    if ui
                        .selectable_label(current == Some(s.id), s.name.as_str())
                        .clicked()
                    {
                        set_set_filter(Some(s.id));
                    }
                }
            });
        set_filter_resp.response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::Role::ComboBox,
                set_filter_resp.response.enabled(),
                "Tool set filter".to_owned(),
            )
        });
        if ui.small_button("New set").clicked() {
            let created = library.create_set("New set");
            let id = created.id;
            RENAME_SET.with(|c| *c.borrow_mut() = Some((id, "New set".to_string())));
            set_set_filter(Some(id));
            persist_library(library);
        }
        if let Some(id) = current {
            if ui.small_button("Rename").clicked() {
                if let Some(s) = library.sets.iter().find(|s| s.id == id) {
                    RENAME_SET.with(|c| *c.borrow_mut() = Some((id, s.name.clone())));
                }
            }
            if ui.small_button("Delete").clicked() {
                library.delete_set(id);
                set_set_filter(None);
                persist_library(library);
            }
            let non_empty = library
                .sets
                .iter()
                .find(|s| s.id == id)
                .map(|s| !s.preset_ids.is_empty())
                .unwrap_or(false);
            if non_empty && ui.small_button("Export set…").clicked() {
                if let Ok(json) = crate::domain::style_preset::export_set_json(library, id) {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Rubric tools", &["rubric-tools.json"])
                        .set_file_name(
                            library
                                .sets
                                .iter()
                                .find(|s| s.id == id)
                                .map(|s| format!("{}.rubric-tools.json", s.name))
                                .unwrap_or_else(|| "set.rubric-tools.json".to_string()),
                        )
                        .save_file()
                    {
                        let tmp = path.with_extension("tmp");
                        if std::fs::write(&tmp, json.as_bytes()).is_ok()
                            && std::fs::rename(&tmp, &path).is_ok()
                        {
                            ui_state.status = format!("Exported set to {}", path.display());
                        } else {
                            ui_state.status = "Export failed".to_string();
                        }
                    }
                }
            }
        }
        if ui.small_button("Import set…").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Rubric tools", &["rubric-tools.json"])
                .pick_file()
            {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    match crate::domain::style_preset::import_set_json(library, &text) {
                        Ok(n) => {
                            ui_state.status = format!("Imported {n} style(s)");
                            persist_library(library);
                        }
                        Err(e) => ui_state.status = format!("Import failed: {e}"),
                    }
                }
            }
        }
    });

    // Inline rename editor for a set.
    let rename_draft = RENAME_SET.with(|c| c.borrow().clone());
    if let Some((id, draft)) = rename_draft {
        ui.horizontal(|ui| {
            let mut d = draft.clone();
            ui.add(egui::TextEdit::singleline(&mut d).desired_width(120.0));
            if ui.button("Save").clicked() {
                if library.rename_set(id, d.clone()) {
                    ui_state.status = "Set renamed".to_string();
                    persist_library(library);
                } else {
                    ui_state.status = "Set name empty or duplicate".to_string();
                }
                RENAME_SET.with(|c| *c.borrow_mut() = None);
            }
            if ui.button("Cancel").clicked() {
                RENAME_SET.with(|c| *c.borrow_mut() = None);
            }
            // update draft
            RENAME_SET.with(|c| {
                if let Some((_, dd)) = c.borrow_mut().as_mut() {
                    *dd = d;
                }
            });
        });
    }

    let current_filter = set_filter();
    let filtered_ids: Option<Vec<Uuid>> = current_filter.and_then(|id| {
        library
            .sets
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.preset_ids.clone())
    });

    if library.is_empty() {
        ui.label("No saved styles yet.");
        ui.label(
            egui::RichText::new("Save a style below, then Apply it any time.")
                .font(TEXT_CAPTION)
                .color(egui::Color32::from_gray(120)),
        );
    } else {
        let visible: Vec<StylePreset> = library
            .iter()
            .filter(|p| match &filtered_ids {
                Some(ids) => ids.contains(&p.id),
                None => true,
            })
            .cloned()
            .collect();
        if visible.is_empty() {
            ui.label("No presets in this set yet — add from the ⋯ menu on any row");
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
        }
        ui.separator();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for preset in &visible {
                    // Add-to-set submenu on each row.
                    ui.horizontal(|ui| {
                        project_mutated |= draw_preset_row(ui, library, project, ui_state, preset);
                        ui.menu_button("⋯", |ui| {
                            for s in &sets {
                                if ui.button(format!("Add to {}", s.name)).clicked() {
                                    library.add_to_set(s.id, preset.id);
                                    persist_library(library);
                                }
                            }
                            if ui.button("Remove from set").clicked() {
                                if let Some(sid) = library.set_of(preset.id) {
                                    library.remove_from_set(sid, preset.id);
                                    persist_library(library);
                                }
                            }
                        });
                    });
                }
            });
    }
    ui.separator();

    draw_save_section(ui, library, project, ui_state);

    project_mutated
}

/// Affordance for a style row's "Apply to Selection" action (UI audit
/// P1-6): the button is ALWAYS rendered — it only *enables* when something
/// is selected, with an honest hint explaining the disabled state. Hiding
/// the control made the row's action set depend on the selection, and the
/// button vanished entirely at exactly the moment the user would look for it.
/// Returns `(enabled, disabled_hint)`; the hint is only shown while disabled.
fn apply_to_selection_affordance(has_selection: bool) -> (bool, &'static str) {
    (has_selection, "Select a markup first")
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

    // Wrapped, not fixed: the row's actions total ~280 px against the
    // 210 px default panel width, so plain `horizontal` clipped the trailing
    // buttons off-panel (UI audit P1-6). Wrapping keeps every action visible.
    ui.horizontal_wrapped(|ui| {
        draw_swatch(ui, preset.style.stroke);
        let name_resp = ui
            .selectable_label(is_armed, preset.name.as_str())
            .on_hover_text(format!("{} style", preset.kind.label()));
        if name_resp.clicked() {
            // The name doubles as the arm toggle, mirroring the tool
            // palette's selectable labels.
            set_armed(ui_state, preset, is_armed);
        }
        // Double-click toggles Drawing/Style mode for presets that captured
        // geometry (audit B1/B5). A preset without geometry is style-only.
        if name_resp.double_clicked() && preset.can_draw() {
            let new_mode = !drawing_mode_for(preset);
            set_drawing_mode(preset.id, new_mode);
            // Re-arm with the new mode if this preset is currently armed.
            let is_armed_now = ui_state
                .pending_style_preset
                .as_ref()
                .is_some_and(|p| p.id == preset.id);
            if is_armed_now {
                set_armed(ui_state, preset, false);
            }
            ui_state.status = format!(
                "'{}' mode: {}",
                preset.name,
                if new_mode { "Drawing" } else { "Style" }
            );
        }
        if preset.can_draw() {
            let mode = if drawing_mode_for(preset) {
                "Draw"
            } else {
                "Style"
            };
            ui.small(mode);
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
        // author/subject/status fill rules of `apply_to`). Always rendered:
        // disabled with a hint when nothing is selected (UI audit P1-6).
        // This is the one row action that mutates the project, so it asks
        // for an undo snapshot.
        let (apply_enabled, apply_hint) = apply_to_selection_affordance(has_selection);
        if ui
            .add_enabled(
                apply_enabled,
                egui::Button::new("Apply to Selection").min_size(egui::vec2(88.0, 24.0)),
            )
            .on_hover_text("Apply this style to every selected markup")
            .on_disabled_hover_text(apply_hint)
            .clicked()
        {
            project_mutated = apply_preset_to_selection(project, ui_state, preset);
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
            library.remove(preset.id);
            persist_library(library);
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
                .id(egui::Id::unique(("tool_chest_rename", preset.id))),
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
                persist_library(library);
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
            .id(egui::Id::unique("tool_chest_save_name")),
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
    persist_library(library);
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
        persist_library(library);
        ui_state.tool_chest_save_name.clear();
        ui_state.status = format!("Style '{name}' saved");
    }
}

/// Arm (or stop) a Tool Chest style preset. While armed, `canvas::finish_markup`
/// styles every non-stamp markup created from now on with the preset's look.
///
/// In Drawing mode the armed copy keeps its captured geometry so the canvas
/// places an exact duplicate; in Style mode the geometry is dropped so the
/// canvas keeps the pre-Drawing-Mode style-only creation path.
fn set_armed(ui_state: &mut UiState, preset: &StylePreset, stop: bool) {
    if stop {
        ui_state.pending_style_preset = None;
        ui_state.status = format!("Style '{}' stopped", preset.name);
    } else {
        let drawing = drawing_mode_for(preset);
        let armed = if drawing {
            preset.clone()
        } else {
            preset.without_geometry()
        };
        ui_state.pending_style_preset = Some(armed);
        ui_state.status = if drawing {
            format!("'{}' active — next markup draws a duplicate", preset.name)
        } else {
            format!("'{}' active — next markup uses its style", preset.name)
        };
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

/// Number of recent entries visible given the current expanded state.
pub(crate) fn visible_recent_count(len: usize, max: usize, expanded: bool) -> usize {
    let max = max.clamp(1, 32);
    if expanded {
        len.min(max)
    } else {
        len.min(4).min(max)
    }
}

/// Auto-record a placed markup as a Recent Tool (gap #3). Builds a capturing
/// preset via `StylePreset::from_markup`, dedupes against the front of the
/// list (same name+kind+geometry signature moves to front), inserts at front
/// and truncates to the configured max. Session-only state.
pub(crate) fn record_recent(ui_state: &mut UiState, m: &Markup) {
    let name = m.kind.label().to_string();
    let preset = StylePreset::from_markup(m, name);
    let signature = recent_signature(&preset);
    if let Some(pos) = ui_state
        .recent_tools
        .iter()
        .position(|p| recent_signature(p) == signature)
    {
        let item = ui_state.recent_tools.remove(pos);
        ui_state.recent_tools.insert(0, item);
    } else {
        ui_state.recent_tools.insert(0, preset);
    }
    let max = ui_state.tool_chest_max_recents.clamp(1, 32);
    ui_state.recent_tools.truncate(max);
}

fn recent_signature(p: &StylePreset) -> (String, MarkupKind, Option<String>) {
    // Geometry debug string as a cheap equality key alongside name+kind.
    (
        p.name.clone(),
        p.kind,
        p.geometry.as_ref().map(|g| format!("{g:?}")),
    )
}

/// One recent-tool row: arms/disarms and toggles Drawing/Style mode exactly
/// like a saved preset row, but lives in the session-only recents list.
fn draw_recent_row(
    ui: &mut Ui,
    project: &mut Project,
    ui_state: &mut UiState,
    preset: &StylePreset,
) -> bool {
    let mut project_mutated = false;
    let is_armed = ui_state
        .pending_style_preset
        .as_ref()
        .is_some_and(|p| p.id == preset.id);
    let has_selection = !ui_state.selected.is_empty();

    // Same wrapped layout as the library row (UI audit P1-6).
    ui.horizontal_wrapped(|ui| {
        draw_swatch(ui, preset.style.stroke);
        let name_resp = ui
            .selectable_label(is_armed, preset.name.as_str())
            .on_hover_text(format!("{} style", preset.kind.label()));
        if name_resp.clicked() {
            set_armed(ui_state, preset, is_armed);
        }
        if name_resp.double_clicked() && preset.can_draw() {
            let new_mode = !drawing_mode_for(preset);
            set_drawing_mode(preset.id, new_mode);
            let is_armed_now = ui_state
                .pending_style_preset
                .as_ref()
                .is_some_and(|p| p.id == preset.id);
            if is_armed_now {
                set_armed(ui_state, preset, false);
            }
            ui_state.status = format!(
                "'{}' mode: {}",
                preset.name,
                if new_mode { "Drawing" } else { "Style" }
            );
        }
        if preset.can_draw() {
            let mode = if drawing_mode_for(preset) {
                "Draw"
            } else {
                "Style"
            };
            ui.small(mode);
        }
        let apply_label = if is_armed { "Stop" } else { "Apply" };
        if ui
            .add_sized(egui::vec2(42.0, 24.0), egui::Button::new(apply_label))
            .clicked()
        {
            set_armed(ui_state, preset, is_armed);
        }
        // Always rendered, disabled with a hint when nothing is selected
        // (UI audit P1-6) — same affordance as the library row.
        let (apply_enabled, apply_hint) = apply_to_selection_affordance(has_selection);
        if ui
            .add_enabled(
                apply_enabled,
                egui::Button::new("Apply to Selection").min_size(egui::vec2(88.0, 24.0)),
            )
            .on_disabled_hover_text(apply_hint)
            .clicked()
        {
            project_mutated = apply_preset_to_selection(project, ui_state, preset);
        }
    });
    project_mutated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geom::Rect;
    use crate::domain::{Markup, MarkupGeometry, MarkupKind};
    use crate::ui::app::UiState;

    fn sample_markup() -> Markup {
        Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(10.0, 20.0, 30.0, 40.0)),
            "Alice",
        )
    }

    #[test]
    fn record_inserts_at_front() {
        let mut ui_state = UiState::default();
        ui_state.tool_chest_max_recents = 8;
        let m = sample_markup();
        record_recent(&mut ui_state, &m);
        assert_eq!(ui_state.recent_tools.len(), 1);
        // second distinct markup (different geometry) inserts at front
        let mut m2 = sample_markup();
        m2.geometry = MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 5.0, 5.0));
        record_recent(&mut ui_state, &m2);
        assert_eq!(ui_state.recent_tools.len(), 2);
        assert_eq!(ui_state.recent_tools[0].kind, MarkupKind::Rectangle);
    }

    #[test]
    fn record_dedupe_moves_to_front() {
        let mut ui_state = UiState::default();
        ui_state.tool_chest_max_recents = 8;
        let m = sample_markup();
        record_recent(&mut ui_state, &m);
        let mut m2 = sample_markup();
        m2.geometry = MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 5.0, 5.0));
        record_recent(&mut ui_state, &m2);
        // re-record original moves it to front
        record_recent(&mut ui_state, &m);
        assert_eq!(ui_state.recent_tools.len(), 2);
    }

    #[test]
    fn record_truncates_at_max() {
        let mut ui_state = UiState::default();
        ui_state.tool_chest_max_recents = 2;
        for i in 0..5 {
            let mut m = sample_markup();
            // Capture normalises position to the origin, so distinct recents
            // must differ in a property that survives normalisation: size.
            m.geometry = MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0 + i as f32, 1.0));
            record_recent(&mut ui_state, &m);
        }
        assert_eq!(ui_state.recent_tools.len(), 2);
    }

    #[test]
    fn clear_empties() {
        let mut ui_state = UiState::default();
        ui_state.tool_chest_max_recents = 8;
        let m = sample_markup();
        record_recent(&mut ui_state, &m);
        ui_state.recent_tools.clear();
        assert!(ui_state.recent_tools.is_empty());
    }

    #[test]
    fn visible_recent_count_logic() {
        assert_eq!(visible_recent_count(10, 8, false), 4);
        assert_eq!(visible_recent_count(10, 8, true), 8);
        assert_eq!(visible_recent_count(3, 8, false), 3);
        assert_eq!(visible_recent_count(3, 2, true), 2);
    }

    // UI audit P1-6: "Apply to Selection" is always part of the row; only
    // its enabled state flips, so the action never silently disappears.
    #[test]
    fn apply_to_selection_is_disabled_with_a_hint_when_nothing_is_selected() {
        let (enabled, hint) = apply_to_selection_affordance(false);
        assert!(!enabled, "no selection must disable the action");
        assert!(hint.contains("Select"), "the hint must tell the user what to do, got {hint:?}");
    }

    #[test]
    fn apply_to_selection_enables_when_something_is_selected() {
        let (enabled, _hint) = apply_to_selection_affordance(true);
        assert!(enabled, "a selection must enable the action");
    }
}

/// Small filled colour square previewing a preset's stroke colour.
fn draw_swatch(ui: &mut Ui, color: Color) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 3.0, to_color32(color));
}
