//! Dialog drawing code extracted from `ui/app.rs` to reduce the god-file size.
//!
//! Each function takes `&mut RubricApp` and `&egui::Context` and draws the
//! corresponding dialog if its visibility flag is set.

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
