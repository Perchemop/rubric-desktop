//! Dialog drawing code extracted from `ui/app.rs` to reduce the god-file size.
//!
//! Each function takes `&mut RubricApp` and `&egui::Context` and draws the
//! corresponding dialog if its visibility flag is set.

use eframe::egui::{self, Ui};

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
    app.ui.status = "Rubric — free, cross-platform PDF markup & measurement".to_string();
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
        .fixed_size(egui::vec2(600.0, 500.0))
        .show(ctx, |ui| {
            ui.heading("Rubric Keyboard Shortcuts");
            ui.separator();
            ui.label("File:");
            ui.label("Ctrl+N — New Project        Ctrl+O — Open Project        Ctrl+S — Save Project        Ctrl+Shift+S — Save Project As…");
            ui.label("Ctrl+I — Import PDF        Ctrl+P — Print        Ctrl+W / Cmd+Q — Quit");
            ui.separator();
            ui.label("Edit:");
            ui.label("Ctrl+Z — Undo        Ctrl+Y — Redo        Ctrl+A — Select All        Del / Backspace — Delete");
            ui.separator();
            ui.label("View:");
            ui.label("Zoom In / Out / Fit Page        PageUp / PageDown        Grid / Snap to Grid        Full Screen (Esc)");
            ui.label("Auto-save on/off        Theme cycling");
            ui.separator();
            ui.label("Tools:");
            ui.label("Select / Highlight / Rectangle / Ellipse / Line / Arrow / Pen / Text Box");
            ui.label("Calibrate Scale (click two points on canvas)");
            ui.label("Measurement tools: Length / Diameter / Radius / Perimeter / Area / Angle");
            ui.separator();
            ui.label("Markup actions:");
            ui.label("Ctrl+D — Duplicate        Ctrl+C / Ctrl+V — Copy / Paste Properties");
            ui.label("Bring Forward / Send Backward / Bring To Front / Send To Back");
            ui.separator();
            ui.label("General:");
            ui.label("Esc — Close dialog / Cancel operation");
            ui.label("Ctrl+Shift+S — Save Project As…");
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
                        match level {
                            1 => ui.heading(text),
                            2 => ui.heading(text),
                            _ => ui.label(text),
                        }
                    } else if line.starts_with('-') || line.starts_with('*') {
                        ui.label(format!("  {}", line));
                    } else if line.starts_with('|') && line.contains("---") {
                        // skip markdown table separators
                    } else if line.starts_with('|') {
                        ui.label(line);
                    } else {
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
/// silently.
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
                "The current project has unsaved changes that will be lost. \
                 Save first (Ctrl+S) to keep them.",
            );
        });
    if !open {
        app.pending_replace = None;
    }
}

/// Calibration dialog (v0.4 gap #4): confirm the real distance of the
/// drawn reference segment and rewrite the page scale.
pub(crate) fn draw_calibration_dialog(app: &mut RubricApp, ctx: &egui::Context) {
    app.draw_calibration_dialog(ctx);
}
