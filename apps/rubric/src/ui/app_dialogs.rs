//! Dialog drawing code extracted from `ui/app.rs` to reduce the god-file size.
//!
//! Each function takes `&mut RubricApp` and `&egui::Context` and draws the
//! corresponding dialog if its visibility flag is set.

use chrono::Utc;
use eframe::egui;

use crate::ui::app::RubricApp;

/// About Rubric window (full-app audit M4): version, licence and links.
pub(crate) fn draw_about_window(app: &mut RubricApp, ctx: &egui::Context) {
    if !app.show_about {
        return;
    }
    let mut open = true;
    egui::Window::new("About Rubric")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Rubric");
            ui.label("Free, cross-platform PDF markup & measurement");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Version:");
                ui.label(env!("CARGO_PKG_VERSION"));
            });
            ui.horizontal(|ui| {
                ui.label("Licence:");
                ui.label("MIT");
            });
            ui.hyperlink_to(
                "Source & releases",
                "https://github.com/Perchemop/rubric-desktop",
            );
            ui.separator();
            if ui.button("Close").clicked() {
                app.show_about = false;
            }
        });
    if !open {
        app.show_about = false;
    }
    // No status write: this runs every frame the window is open and used to
    // stomp the transient status line with the tagline (UI audit P1-6).
}

/// Keyboard shortcuts dialog (proposal 10): lists all discoverable
/// shortcuts grouped by category, for reference while working.
pub(crate) fn draw_keyboard_shortcuts(app: &mut RubricApp, ctx: &egui::Context) {
    if !app.show_keyboard_shortcuts {
        return;
    }
    let mut open = true;
    egui::Window::new("Keyboard Shortcuts")
        .open(&mut open)
        .resizable(false)
        .fixed_size(egui::vec2(700.0, 600.0))
        .show(ctx, |ui| {
            ui.heading("Rubric Keyboard Shortcuts");
            ui.separator();
            ui.label("The following shortcuts are defined in the application:");
            ui.separator();

            // File actions
            ui.label("File:");
            ui.label("Ctrl+N — New Project");
            ui.label("Ctrl+O — Open Project");
            ui.label("Ctrl+S — Save Project");
            ui.label("Ctrl+Shift+S — Save Project As…");
            ui.separator();

            // Edit actions
            ui.label("Edit:");
            ui.label("Ctrl+Z — Undo");
            ui.label("Ctrl+Y — Redo");
            ui.label("Ctrl+A — Select All");
            ui.label("Del / Backspace — Delete");
            ui.separator();

            // View actions
            ui.label("View:");
            ui.label("Zoom In / Out / Fit Page");
            ui.label("PageUp / PageDown");
            ui.label("Grid / Snap to Grid");
            ui.label("Full Screen (Esc)");
            ui.separator();

            // Tools — all 25 tools from TOOL_METADATA
            ui.label("Tools:");
            ui.separator();
            ui.label("Single-key tool shortcuts (press the key to activate):");
            ui.horizontal_wrapped(|ui| {
                for meta in crate::domain::project::TOOL_METADATA.iter() {
                    let shortcut = meta.shortcut.map(|c| c.to_ascii_uppercase()).unwrap_or('_');
                    if ui.button(format!("{}", shortcut).as_str()).clicked() {
                        // Activate the tool — could be expanded later
                    }
                    ui.label(format!("{} ({})", meta.label, shortcut));
                }
            });
            ui.separator();

            // Calibrate and measurements
            ui.label("Calibrate Scale (click two points on canvas)");
            ui.label("Measurement tools: Length / Diameter / Radius / Perimeter / Area / Angle");
            ui.separator();

            // Markup actions
            ui.label("Markup actions:");
            ui.label("Ctrl+D — Duplicate");
            ui.label("Bring Forward / Send Backward / Bring To Front / Send To Back");
            ui.separator();

            // General
            ui.label("General:");
            ui.label("Esc — Close dialog / Cancel operation");
        });
    if !open {
        app.show_keyboard_shortcuts = false;
    }
}

/// User guide window (Help ▸ User Guide): the embedded USER_GUIDE.md,
/// so it works in a packaged binary, not only in a source checkout.
pub(crate) fn draw_user_guide(app: &mut RubricApp, ctx: &egui::Context) {
    if !app.show_user_guide {
        return;
    }
    let mut open = true;
    egui::Window::new("User Guide")
        .open(&mut open)
        .resizable(true)
        .min_size(egui::vec2(700.0, 500.0))
        .default_size(egui::vec2(900.0, 700.0))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Rubric User Guide");
                ui.separator();
                for line in crate::ui::app::USER_GUIDE.lines() {
                    if line.is_empty() {
                        ui.add_space(4.0);
                    } else if line.starts_with('#') {
                        let level = line.chars().take_while(|c| *c == '#').count();
                        let text = line.trim_start_matches('#').trim();
                        let _ = match level {
                            1 => ui.heading(text),
                            2 => ui.heading(text),
                            _ => ui.label(text),
                        };
                    } else if line.starts_with('-') || line.starts_with('*') {
                        ui.label(format!("  {}", line));
                    } else if line.starts_with('|') && line.contains("---") {
                        // skip markdown table separators
                    } else if line.starts_with('|') {
                        ui.label(line);
                    }
                }
                ui.separator();
                ui.hyperlink_to(
                    "Source & releases",
                    "https://github.com/Perchemop/rubric-desktop",
                );
            });
        });
    if !open {
        app.show_user_guide = false;
    }
}

/// Unsaved-changes confirmation dialog: Save and Quit / Quit Anyway / Cancel.
pub(crate) fn draw_quit_confirm(app: &mut RubricApp, ctx: &egui::Context) {
    if !app.ui.quit_confirm {
        return;
    }
    let mut open = true;
    egui::Window::new("Unsaved Changes")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("The project has unsaved changes.");
            ui.horizontal(|ui| {
                // Save first, then quit — but only when the save really
                // landed. A cancelled Save As or a failed write leaves
                // the project dirty, so the dialog stays open instead of
                // quitting on top of unsaved work.
                if ui.button("Save and Quit").clicked() {
                    app.save_project_action();
                    if !app.ui.dirty {
                        app.ui.quit_confirm = false;
                        app.force_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                if ui.button("Quit Anyway").clicked() {
                    app.ui.quit_confirm = false;
                    // Tell the close veto this close was confirmed
                    // (audit M1); otherwise it would re-prompt.
                    app.force_close = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button("Cancel").clicked() {
                    app.ui.quit_confirm = false;
                }
            });
        });
    if !open {
        app.ui.quit_confirm = false;
    }
}

/// Replace-unsaved-project confirmation (full-app audit B2): New / Open
/// / Import / Recent park their install here instead of discarding work
/// silently. UI audit P0-1: the dialog must offer a real way through —
/// Save First, Discard & Replace, or Cancel — not just a warning label.
pub(crate) fn draw_replace_unsaved(app: &mut RubricApp, ctx: &egui::Context) {
    if app.pending_replace.is_none() {
        return;
    }
    let mut open = true;
    egui::Window::new("Replace Unsaved Project")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(
                "The current project has unsaved changes that will be lost \
                 if you replace it.",
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Save First").clicked() {
                    // save_project_action opens a Save-As dialog when the
                    // project has no path yet; if the save did not land the
                    // project stays dirty and the parked install remains.
                    app.save_project_action();
                    if !app.ui.dirty {
                        app.install_pending_replace();
                    }
                }
                if ui.button("Discard & Replace").clicked() {
                    app.install_pending_replace();
                }
                if ui.button("Cancel").clicked() {
                    app.cancel_pending_replace();
                }
            });
        });
    if !open {
        // Closing the window is a choice to keep the current project.
        app.cancel_pending_replace();
    }
}

/// Calibration dialog (v0.4 gap #4): confirm the real distance of the
/// drawn reference segment and rewrite the page scale.
pub(crate) fn draw_calibration_dialog(app: &mut RubricApp, ctx: &egui::Context) {
    app.draw_calibration_dialog(ctx);
}

/// Takeoff condition dialog (T8): create or edit a takeoff condition.
/// Shows name, kind, height (WallSurface only), colour swatch, and notes.
/// Esc = cancel, Enter = commit (on focused field).
/// Validates name (non-empty, unique case-insensitively) and height (> 0 for WallSurface).
pub(crate) fn draw_takeoff_dialog(app: &mut RubricApp, ctx: &egui::Context) {
    use crate::domain::style::Color;
    use crate::domain::takeoff::ConditionKind;

    // Take the dialog state out, work with it, and put it back if needed
    let Some(mut state) = app.ui.takeoff_dialog.take() else {
        return;
    };

    let is_editing = state.editing_id.is_some();
    let title = if is_editing {
        "Edit Condition"
    } else {
        "New Condition"
    };

    // Compute existing condition names for duplicate validation (case-insensitive)
    let existing_names: Vec<String> = app
        .history
        .current()
        .takeoff
        .conditions
        .iter()
        .filter(|c| state.editing_id.map(|id| c.id != id).unwrap_or(true))
        .map(|c| c.name.to_lowercase())
        .collect();

    let mut open = true;
    let mut cancelled = false;
    let mut commit = false;

    egui::Window::new(title)
        .open(&mut open)
        .resizable(false)
        .default_width(480.0)
        .show(ctx, |ui| {
            // Handle Esc key globally
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                cancelled = true;
            }

            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            // Name field
            ui.horizontal(|ui| {
                ui.label("Name");
                let name_resp = ui.add(
                    egui::TextEdit::singleline(&mut state.name)
                        .desired_width(280.0)
                        .hint_text("Condition name"),
                );
                if state.focus_name {
                    name_resp.request_focus();
                    state.focus_name = false;
                }
                // Validate on lost focus or Enter
                let name_trimmed = state.name.trim();
                let name_empty = name_trimmed.is_empty();
                let name_duplicate = existing_names.contains(&name_trimmed.to_lowercase());
                if name_empty || name_duplicate {
                    ui.small(
                        egui::RichText::new(if name_empty {
                            "Name cannot be empty"
                        } else {
                            "A condition with this name already exists"
                        })
                        .color(ui.visuals().error_fg_color),
                    );
                }
                // Enter on name field commits if valid
                if name_resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let height_ok = if matches!(state.kind, ConditionKind::WallSurface) {
                        state
                            .height
                            .trim()
                            .parse::<f64>()
                            .map(|v| v > 0.0)
                            .unwrap_or(false)
                    } else {
                        true
                    };
                    if !name_empty && !name_duplicate && height_ok {
                        commit = true;
                    } else if matches!(state.kind, ConditionKind::WallSurface) {
                        state.height_error = true;
                    }
                }
            });

            // Kind combo
            ui.horizontal(|ui| {
                ui.label("Kind");
                let kind_enabled = !state.has_assignments;
                let mut kind = state.kind;
                let _kind_resp = egui::ComboBox::from_label("")
                    .selected_text(kind.label())
                    .show_ui(ui, |ui| {
                        for k in ConditionKind::ALL {
                            ui.selectable_value(&mut kind, k, k.label());
                        }
                    });
                if kind != state.kind && kind_enabled {
                    state.kind = kind;
                    // Reset height when kind changes away from WallSurface
                    if !matches!(kind, ConditionKind::WallSurface) {
                        state.height.clear();
                        state.height_error = false;
                    } else if state.height.is_empty() {
                        // Pre-fill with a reasonable default when switching to WallSurface
                        state.height = "2.7".to_string();
                    }
                }
                if !kind_enabled {
                    ui.small("(locked: unassign markups first to change kind)");
                }
            });

            // Height field (WallSurface only)
            if matches!(state.kind, ConditionKind::WallSurface) {
                ui.horizontal(|ui| {
                    ui.label("Height");
                    let height_resp = ui.add(
                        egui::TextEdit::singleline(&mut state.height)
                            .desired_width(120.0)
                            .hint_text("e.g. 2.7"),
                    );
                    // Show unit suffix from the project default unit
                    let unit = app
                        .history
                        .current()
                        .active()
                        .and_then(|d| d.pages.get(app.ui.current_page).copied())
                        .map(|p| p.scale.unit)
                        .unwrap_or_default();
                    ui.label(unit.suffix());
                    if state.height_error {
                        ui.small(
                            egui::RichText::new("Enter a height greater than 0")
                                .color(ui.visuals().error_fg_color),
                        );
                    }
                    // Enter on height field commits if valid
                    if height_resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let name_trimmed = state.name.trim();
                        let name_ok = !name_trimmed.is_empty()
                            && !existing_names.contains(&name_trimmed.to_lowercase());
                        if name_ok
                            && state
                                .height
                                .trim()
                                .parse::<f64>()
                                .map(|v| v > 0.0)
                                .unwrap_or(false)
                        {
                            commit = true;
                        } else {
                            state.height_error = true;
                        }
                    }
                });
            }

            // Colour swatch
            ui.horizontal(|ui| {
                ui.label("Colour");
                let mut colour = egui::Color32::from_rgba_unmultiplied(
                    state.colour.r,
                    state.colour.g,
                    state.colour.b,
                    state.colour.a,
                );
                let colour_resp = ui
                    .color_edit_button_srgba(&mut colour)
                    .on_hover_text("Click to pick a colour");
                if colour_resp.changed() {
                    state.colour = Color {
                        r: colour.r(),
                        g: colour.g(),
                        b: colour.b(),
                        a: colour.a(),
                    };
                }
            });

            // Notes field
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Notes");
                    let _ = ui.add(
                        egui::TextEdit::multiline(&mut state.notes)
                            .desired_width(440.0)
                            .desired_rows(3)
                            .hint_text("Optional notes"),
                    );
                });
            });

            ui.separator();

            // Buttons
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Delete button (only when editing)
                    if is_editing {
                        if ui.button("Delete").clicked() {
                            state.show_delete_confirm = true;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        cancelled = true;
                    }
                    let can_commit = {
                        let name_trimmed = state.name.trim();
                        let name_ok = !name_trimmed.is_empty()
                            && !existing_names.contains(&name_trimmed.to_lowercase());
                        let height_ok = if matches!(state.kind, ConditionKind::WallSurface) {
                            state
                                .height
                                .trim()
                                .parse::<f64>()
                                .map(|v| v > 0.0)
                                .unwrap_or(false)
                        } else {
                            true
                        };
                        name_ok && height_ok
                    };
                    let ok_btn = ui.add_enabled(can_commit, egui::Button::new("OK"));
                    if ok_btn.clicked() {
                        commit = true;
                    }
                });
            });
        });

    if !open || cancelled {
        app.ui.takeoff_dialog = None;
        return;
    }

    // Handle delete confirmation (shows as a second modal window)
    if state.show_delete_confirm {
        let assigned_count = state.assigned_count;
        let condition_id = state.editing_id;
        let condition_name = state.name.clone();
        let mut delete_confirmed = false;

        egui::Window::new("Delete Condition")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!(
                    "{} measurement{} will become unassigned. Delete '{}'?",
                    assigned_count,
                    if assigned_count == 1 { "" } else { "s" },
                    condition_name
                ));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        delete_confirmed = true;
                    }
                    if ui.button("Cancel").clicked() {
                        // Just close the confirm, keep the edit dialog open
                    }
                });
            });

        if delete_confirmed {
            if let Some(id) = condition_id {
                let unassigned = app.history.current_mut().delete_condition(id).unwrap_or(0);
                app.push_snapshot("delete condition");
                app.set_status(
                    crate::ui::app::StatusKind::Info,
                    format!("Deleted '{}' ({} unassigned)", condition_name, unassigned),
                );
            }
            app.ui.takeoff_dialog = None;
        } else {
            // Cancel was clicked, keep the edit dialog open
            state.show_delete_confirm = false;
            app.ui.takeoff_dialog = Some(state);
            return;
        }
    }

    if commit {
        let name = state.name.trim().to_string();
        let kind = state.kind;
        let colour = state.colour;
        let notes = if state.notes.trim().is_empty() {
            None
        } else {
            Some(state.notes.trim().to_string())
        };
        let height = if matches!(kind, ConditionKind::WallSurface) {
            state.height.trim().parse::<f64>().ok()
        } else {
            None
        };

        if let Some(editing_id) = state.editing_id {
            // Editing existing condition
            if let Err(e) = app
                .history
                .current_mut()
                .rename_condition(editing_id, &name)
            {
                app.set_status(crate::ui::app::StatusKind::Error, format!("{}", e));
            } else {
                // Update kind, height, colour, notes
                if let Some(cond) = app
                    .history
                    .current_mut()
                    .takeoff
                    .conditions
                    .iter_mut()
                    .find(|c| c.id == editing_id)
                {
                    cond.kind = kind;
                    cond.height = height;
                    cond.colour = colour;
                    cond.notes = notes;
                    cond.modified = Utc::now();
                }
                app.push_snapshot("edit condition");
                app.set_status(
                    crate::ui::app::StatusKind::Success,
                    format!("Updated condition '{}'", name),
                );
            }
        } else {
            // Creating new condition
            let result = app
                .history
                .current_mut()
                .create_condition(name.clone(), kind, height);
            match result {
                Ok(cond_id) => {
                    // Apply the dialog's colour and notes to the stored condition.
                    if let Some(cond) = app
                        .history
                        .current_mut()
                        .takeoff
                        .conditions
                        .iter_mut()
                        .find(|c| c.id == cond_id)
                    {
                        cond.colour = colour;
                        cond.notes = notes;
                        cond.modified = Utc::now();
                    }
                    app.push_snapshot("create condition");
                    app.set_status(
                        crate::ui::app::StatusKind::Success,
                        format!("Created condition '{}'", name),
                    );
                }
                Err(e) => {
                    app.set_status(crate::ui::app::StatusKind::Error, format!("{}", e));
                }
            }
        }
        app.ui.takeoff_dialog = None;
    } else {
        // Put the state back if not cancelled or committed
        app.ui.takeoff_dialog = Some(state);
    }
}
