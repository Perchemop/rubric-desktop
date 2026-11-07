//! Main menu bar (File / Edit / View / Help).
//!
//! Extracted from `ui/app.rs` to reduce the god-file size. All methods called
//! from here must be `pub(crate)` on `RubricApp`.

use eframe::egui::{self, Ui};

use std::path::{Path, PathBuf};

use crate::data::pdf;
use crate::domain::project::Project;
use crate::ui::app::RubricApp;

/// Draw the top menu bar. Called from [`RubricApp::draw_ui`].
pub(crate) fn draw_menu_bar(app: &mut RubricApp, ui: &mut Ui) {
    use egui::MenuBar;
    MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New Project").clicked() {
                app.install_project(
                    Project::new("Untitled Rubric Project"),
                    None,
                    "New project created".to_string(),
                );
                ui.close();
            }
            if crate::ui::app::menu_shortcut_button(ui, "Open Project...", "Ctrl+O") {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Rubric project", &["rubric.json"])
                    .pick_file()
                {
                    app.open_project_path(path);
                }
                ui.close();
            }
            if crate::ui::app::menu_shortcut_button(ui, "Import PDF…", "Ctrl+I") {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PDF document", &["pdf"])
                    .pick_file()
                {
                    match pdf::read_pdf_info(&path) {
                        // Audit M9: a 0-page PDF used to report "Imported
                        // 1 page(s)" and fabricate an unrenderable page.
                        Ok(info) if info.page_sizes.is_empty() => {
                            app.ui.status = "Import failed: the PDF has no pages".to_string();
                        }
                        Ok(info) => {
                            let stem = path
                                .file_stem()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Imported PDF".to_string());
                            let count = info.page_sizes.len();
                            let project = Project::from_pdf(
                                stem,
                                path.to_string_lossy().to_string(),
                                &info.page_sizes,
                            );
                            app.install_project(project, None, format!("Imported {count} page(s)"));
                        }
                        Err(e) => {
                            app.ui.status = format!("Import failed: {e}");
                        }
                    }
                }
                ui.close();
            }
            let total_markups: usize = app
                .history
                .current()
                .documents
                .iter()
                .map(|d| d.markups.len())
                .sum();
            let report_enabled = total_markups > 0;
            let has_source_pdf = app
                .history
                .current()
                .source_pdf
                .as_deref()
                .is_some_and(|p| !p.is_empty());
            if ui
                .add_enabled(report_enabled, egui::Button::new("Export Report…"))
                .on_disabled_hover_text("No markups to report yet")
                .clicked()
            {
                let project = app.history.current();
                let total_markups: usize = project.documents.iter().map(|d| d.markups.len()).sum();
                app.report_export = Some(crate::ui::app::ReportExportUi {
                    group_key: crate::domain::ReportGroupKey::Status,
                    format: crate::ui::app::ReportExportFormat::Text,
                    project_title: project.name.clone(),
                    total_markups,
                });
                ui.close();
            }
            if ui
                .add_enabled(has_source_pdf, egui::Button::new("Export Page Image…"))
                .on_disabled_hover_text("Import a PDF before exporting an image")
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("rubric-page.png")
                    .add_filter("PNG image", &["png"])
                    .save_file()
                {
                    let _ = app.export_page_image(path);
                }
                ui.close();
            }
            if ui
                .add_enabled(has_source_pdf, egui::Button::new("Export PDF…"))
                .on_disabled_hover_text("Import a PDF before exporting")
                .clicked()
            {
                let source_pdf = app.history.current().source_pdf.clone().unwrap_or_default();
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("rubric-export.pdf")
                    .add_filter("PDF document", &["pdf"])
                    .save_file()
                {
                    if !source_pdf.is_empty() {
                        if let Some(doc) = app.active_doc() {
                            let bates = app.history.current().settings.bates.clone();
                            match pdf::export_pdf_to(
                                Path::new(&source_pdf),
                                &path,
                                &doc.markups,
                                &bates,
                            ) {
                                Ok(()) => {
                                    app.ui.status = format!("PDF exported to {}", path.display());
                                }
                                Err(e) => {
                                    app.ui.status = format!("PDF export failed: {e}");
                                }
                            }
                        }
                    } else {
                        app.ui.status = "Nothing to export — import a PDF first".to_string();
                    }
                }
                ui.close();
            }
            // Flatten-to-PDF (gap #1): a path from markups back into a PDF.
            // Both items need the imported source PDF — without one there is no
            // page content to flatten the markups onto — so they are disabled
            // until a PDF is imported.
            let export_page_pdf = ui
                .add_enabled(has_source_pdf, egui::Button::new("Export Page to PDF…"))
                .on_disabled_hover_text("Import a PDF before flattening a page");
            if export_page_pdf.clicked() {
                app.export_current_page_to_pdf();
                ui.close();
            }
            let export_all_pdf = ui
                .add_enabled(
                    has_source_pdf,
                    egui::Button::new("Export All Pages to PDF…"),
                )
                .on_disabled_hover_text("Import a PDF before flattening pages");
            if export_all_pdf.clicked() {
                app.export_all_pages_to_pdf_action();
                ui.close();
            }
            if ui
                .add_enabled(has_source_pdf, egui::Button::new("Print…"))
                .on_disabled_hover_text("Import a PDF before printing")
                .clicked()
            {
                let _ = app.print_page();
                ui.close();
            }
            // Recent submenu: app-level recents (data layer), not project
            // settings — reads/writes `recent.json` in the config dir.
            ui.menu_button("Recent", |ui| {
                let recents = crate::data::load_recent();
                if recents.is_empty() {
                    ui.label("No recent projects");
                } else {
                    for r in recents.as_slice() {
                        let path = PathBuf::from(r);
                        let label = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| r.clone());
                        let resp = ui.button(&label);
                        resp.clone().on_hover_text(r.as_str());
                        if resp.clicked() {
                            app.open_project_path(path);
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button("Clear Recent").clicked() {
                        let mut recent = crate::data::load_recent();
                        recent.clear();
                        // Report the real outcome: don't claim success when
                        // the recents file could not be rewritten (audit M2).
                        app.ui.status = match crate::data::save_recent(&recent) {
                            Ok(()) => "Recent list cleared".to_string(),
                            Err(e) => format!("Could not clear the recent list: {e}"),
                        };
                        ui.close();
                    }
                }
            });
            if crate::ui::app::menu_shortcut_button(ui, "Save Project", "Ctrl+S") {
                app.save_project_action();
                ui.close();
            }
            if crate::ui::app::menu_shortcut_button(ui, "Save Project As…", "Ctrl+Shift+S") {
                app.save_project_as_action();
                ui.close();
            }
            // Autosave controls (spec F-502). These edit `Settings`, which
            // is project data, so each change pushes one undo snapshot.
            ui.menu_button("Auto-save", |ui| {
                let mut enabled = app.history.current().settings.auto_save;
                if ui.checkbox(&mut enabled, "Enable Auto-save").changed() {
                    app.history.current_mut().settings.auto_save = enabled;
                    app.ui.status = if enabled {
                        "Auto-save on".to_string()
                    } else {
                        "Auto-save off".to_string()
                    };
                    app.push_snapshot_quiet();
                }
                ui.separator();
                let current = app.history.current().settings.auto_save_interval_secs;
                // Keep a non-preset persisted interval honest by showing it.
                let mut options = crate::domain::AUTO_SAVE_INTERVALS.to_vec();
                if !options.contains(&current) {
                    options.insert(0, current);
                }
                let mut chosen = current;
                for secs in options {
                    ui.selectable_value(
                        &mut chosen,
                        secs,
                        crate::domain::auto_save_interval_label(secs),
                    );
                }
                if chosen != current {
                    app.history.current_mut().settings.auto_save_interval_secs = chosen;
                    app.ui.status = format!(
                        "Auto-save every {}",
                        crate::domain::auto_save_interval_label(chosen)
                    );
                    app.push_snapshot_quiet();
                }
            });
            ui.separator();
            if ui.button("Quit").clicked() {
                if app.ui.dirty {
                    app.ui.quit_confirm = true;
                } else {
                    // Let eframe tear down normally instead of exiting the
                    // process outright.
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });
        ui.menu_button("Edit", |ui| {
            if ui
                .add_enabled(
                    app.history.can_undo(),
                    egui::Button::new("Undo").shortcut_text("Ctrl+Z"),
                )
                .clicked()
            {
                app.apply_editor_action(crate::domain::EditorAction::Undo);
                ui.close();
            }
            if ui
                .add_enabled(
                    app.history.can_redo(),
                    egui::Button::new("Redo").shortcut_text("Ctrl+Y"),
                )
                .clicked()
            {
                app.apply_editor_action(crate::domain::EditorAction::Redo);
                ui.close();
            }
            ui.separator();
            // Select All (full-app audit M10): select every editable,
            // visible markup on the current page — the delete and
            // apply-style paths already operate on the whole selection.
            if crate::ui::app::menu_shortcut_button(ui, "Select All", "Ctrl+A") {
                app.select_all_on_page();
                ui.close();
            }
            ui.separator();
            crate::ui::menu::draw_selection_commands(ui, &mut app.ui);
        });
        ui.menu_button("View", |ui| {
            if crate::ui::app::menu_shortcut_button(ui, "Zoom In", "Ctrl++") {
                app.set_zoom_level(app.ui.zoom * 1.25);
                ui.close();
            }
            if crate::ui::app::menu_shortcut_button(ui, "Zoom Out", "Ctrl+-") {
                app.set_zoom_level(app.ui.zoom * 0.8);
                ui.close();
            }
            if crate::ui::app::menu_shortcut_button(ui, "Fit Page", "Ctrl+0") {
                app.fit_page();
                ui.close();
            }
            // Drag calibration (v0.4 gap #4): selecting the tool starts a
            // two-click reference segment on the canvas; the dialog that
            // follows writes the page scale.
            if ui
                .button("Calibrate Scale (click two points on canvas)…")
                .clicked()
            {
                app.select_tool(crate::domain::Tool::Calibrate);
                ui.close();
            }
            ui.separator();
            // Page navigation in the menu (full-app audit M10): gated at
            // document boundaries so the menu never silently does nothing.
            let count = app.active_doc().map(|d| d.page_count()).unwrap_or(0);
            let current = app.ui.current_page.min(count.saturating_sub(1));
            if ui
                .add_enabled(
                    current > 0,
                    egui::Button::new("Previous Page").shortcut_text("PageUp"),
                )
                .on_disabled_hover_text("Already on the first page")
                .clicked()
            {
                app.jump_to_page(current.saturating_sub(1));
            }
            if ui
                .add_enabled(
                    current + 1 < count,
                    egui::Button::new("Next Page").shortcut_text("PageDown"),
                )
                .on_disabled_hover_text("Already on the last page")
                .clicked()
            {
                app.jump_to_page(current + 1);
            }
            if ui.button("Full Screen").clicked() {
                // Read the live state so the toggle is truthful even if the
                // WM changed it out from under us.
                let fullscreen = ui
                    .ctx()
                    .input(|i| i.viewport().fullscreen)
                    .unwrap_or(app.ui.fullscreen);
                app.ui.fullscreen = !fullscreen;
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
                app.ui.status = if app.ui.fullscreen {
                    "Full screen on (Esc-exit via the View menu)".to_string()
                } else {
                    "Full screen off".to_string()
                };
            }
            ui.separator();
            for pct in crate::domain::view::ZOOM_PRESETS {
                let zoom = pct as f32 / 100.0;
                if ui.button(format!("{}%", pct)).clicked() {
                    app.ui.zoom = crate::domain::view::clamp_zoom(zoom);
                    app.ui.status = format!("Zoom {}%", pct);
                }
            }
            ui.separator();
            // Custom zoom entry (type any percentage).
            ui.horizontal(|ui| {
                ui.label("Custom zoom:");
                let mut custom_zoom = (app.ui.zoom * 100.0) as i32;
                if ui
                    .add(
                        egui::DragValue::new(&mut custom_zoom)
                            .speed(1)
                            .range(
                                (crate::domain::view::MIN_ZOOM * 100.0) as i32
                                    ..=(crate::domain::view::MAX_ZOOM * 100.0) as i32,
                            )
                            .suffix(" %"),
                    )
                    .changed()
                {
                    app.ui.zoom = crate::domain::view::clamp_zoom(custom_zoom as f32 / 100.0);
                    app.ui.status = format!("Zoom {}%", custom_zoom);
                }
            });
            ui.separator();
            // Canvas guides (F-405): grid visibility, snapping and pitch.
            // Grid lines are only drawn when the pitch resolves at the
            // current zoom, so the tick stays honest even when off.
            let mut grid_visible = app.ui.grid.visible;
            if ui.checkbox(&mut grid_visible, "Show Grid").changed() {
                app.ui.grid.visible = grid_visible;
            }
            if ui.checkbox(&mut app.ui.grid.snap, "Snap to Grid").clicked() {
                app.ui.status = if app.ui.grid.snap {
                    "Snap to grid on".to_string()
                } else {
                    "Snap to grid off".to_string()
                };
            }
            ui.menu_button("Grid Spacing", |ui| {
                let mut spacing = app.ui.grid.spacing;
                for s in crate::domain::Grid::SPACINGS {
                    if ui
                        .selectable_value(&mut spacing, s, format!("{s} pt"))
                        .clicked()
                    {
                        app.ui.grid.spacing = s;
                        app.ui.status = format!("Grid spacing → {s} pt");
                    }
                }
            });
            ui.separator();
            {
                let mut show_measurements = app.history.current().settings.show_measurements;
                if ui
                    .checkbox(&mut show_measurements, "Show Measurements")
                    .changed()
                {
                    app.history.current_mut().settings.show_measurements = show_measurements;
                    app.ui.status = if show_measurements {
                        "Measurements shown".to_string()
                    } else {
                        "Measurements hidden".to_string()
                    };
                    app.push_snapshot_quiet();
                }
            }
            ui.separator();
            ui.menu_button("Theme", |ui| {
                for option in crate::domain::Theme::ALL {
                    let mut current = app.theme;
                    if ui
                        .radio_value(&mut current, option, option.label())
                        .clicked()
                        && current != app.theme
                    {
                        app.set_theme(ui.ctx(), current);
                    }
                }
            });
            let mut high_contrast = app.high_contrast;
            if ui
                .checkbox(&mut high_contrast, "High Contrast")
                .on_hover_text("Maximum contrast for low-vision use (WCAG AAA text)")
                .changed()
            {
                app.set_high_contrast(ui.ctx(), high_contrast);
            }
            ui.separator();
            if ui
                .checkbox(&mut app.ui.show_tool_chest, "Show Tool Chest")
                .changed()
            {
                app.ui.status = if app.ui.show_tool_chest {
                    "Tool Chest shown".to_string()
                } else {
                    "Tool Chest hidden".to_string()
                };
                app.save_app_prefs();
            }
            if ui.checkbox(&mut app.ui.focus_mode, "Focus Mode").changed() {
                app.ui.status = if app.ui.focus_mode {
                    "Focus mode on — panels hidden".to_string()
                } else {
                    "Focus mode off".to_string()
                };
                app.save_app_prefs();
            }
            if ui
                .checkbox(&mut app.ui.show_markups_list, "Show Markups List")
                .changed()
            {
                app.ui.status = if app.ui.show_markups_list {
                    "Markups List shown".to_string()
                } else {
                    "Markups List hidden".to_string()
                };
                app.save_app_prefs();
            }
            if ui.checkbox(&mut app.ui.show_pages, "Show Pages").changed() {
                app.ui.status = if app.ui.show_pages {
                    "Pages panel shown".to_string()
                } else {
                    "Pages panel hidden".to_string()
                };
                app.save_app_prefs();
            }
        });
        ui.menu_button("Help", |ui| {
            if ui.button("About Rubric").clicked() {
                app.show_about = !app.show_about;
                ui.close();
            }
            if ui.button("Check for Updates…").clicked() {
                app.ui.status = "Checking for updates…".to_string();
                app.begin_update_check(ui.ctx());
                ui.close();
            }
            if ui.button("Report a Bug…").clicked() {
                app.report = Some(crate::ui::app::BugReportUi::new(
                    crash_report::Environment::capture(env!("CARGO_PKG_VERSION")),
                    "describe what you were doing when the problem happened",
                ));
                ui.close();
            }
            ui.separator();
            if ui.button("Keyboard Shortcuts…").clicked() {
                app.show_keyboard_shortcuts = !app.show_keyboard_shortcuts;
                ui.close();
            }
            if ui.button("User Guide…").clicked() {
                app.show_user_guide = !app.show_user_guide;
                ui.close();
            }
        });
    });
}
