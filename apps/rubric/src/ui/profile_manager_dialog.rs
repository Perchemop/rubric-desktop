//! Manage Profiles dialog (spec F-606, section 6.7).
//!
//! Lists profiles with name/type/modified columns, a preview area, search/filter,
//! and actions: New, Duplicate, Rename, Delete, Export. Import and Set Default
//! land with F-606 completion (their buttons are intentionally absent so no
//! visible control is a no-op). Predefined profiles disable Delete/Rename.
//! Integrates with ProfileStore.

use crate::domain::profile::ProfileSummary;
use anyhow;
use eframe::egui;

const OPEN_KEY: &str = "rubric.profile_manager.open";
const SEARCH_KEY: &str = "rubric.profile_manager.search";
/// Profile name awaiting its new name in the rename prompt (F-606 6.7).
const RENAME_KEY: &str = "rubric.profile_manager.rename";
/// Text buffer of the rename prompt.
const RENAME_INPUT_KEY: &str = "rubric.profile_manager.rename_input";

#[allow(deprecated)]
pub fn open(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(OPEN_KEY), true));
}

#[allow(deprecated)]
pub fn draw(app: &mut crate::ui::app::RubricApp, ctx: &egui::Context) {
    let mut open = ctx
        .data(|d| d.get_temp::<bool>(egui::Id::new(OPEN_KEY)))
        .unwrap_or(false);
    if !open {
        return;
    }

    let mut search = ctx
        .data(|d| d.get_temp::<String>(egui::Id::new(SEARCH_KEY)))
        .unwrap_or_default();

    let mut close = false;
    let mut delete_target: Option<String> = None;
    // (old_name, new_name) confirmed by the rename prompt below.
    let mut rename_target: Option<(String, String)> = None;
    // Profile whose Rename button was pressed; drives the rename prompt.
    let mut rename_pending: Option<String> = ctx
        .data(|d| d.get_temp::<String>(egui::Id::new(RENAME_KEY)))
        .unwrap_or(None);
    let mut rename_input = ctx
        .data(|d| d.get_temp::<String>(egui::Id::new(RENAME_INPUT_KEY)))
        .unwrap_or_default();
    let mut duplicate_target: Option<String> = None;
    let mut export_target: Option<String> = None;
    // Import and Set Default are not wired yet (F-606 completion).
    // Their buttons are intentionally omitted so no visible control is a
    // no-op; the intent survives here as a comment.

    egui::Window::new("Manage Profiles")
        .open(&mut open)
        .resizable(true)
        .default_size(egui::vec2(720.0, 420.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut search);
            });
            ui.separator();

            // Load profiles from ProfileStore
            let profiles = match app.profile_store.list_profiles() {
                Ok(p) => p,
                Err(e) => {
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        format!("Failed to load profiles: {}", e),
                    );
                    Vec::new()
                }
            };

            let filtered: Vec<&ProfileSummary> = profiles
                .iter()
                .filter(|p| {
                    search.is_empty() || p.name.to_lowercase().contains(&search.to_lowercase())
                })
                .collect();

            ui.columns(2, |cols| {
                // List view
                cols[0].group(|ui| {
                    egui::Grid::new("profile_list")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Name");
                            ui.label("Type");
                            ui.label("Modified");
                            ui.end_row();
                            for p in &filtered {
                                ui.label(&p.name);
                                ui.label(if p.predefined { "Predefined" } else { "Custom" });
                                let modified = p
                                    .modified_at
                                    .as_ref()
                                    .map(|t| t.to_rfc3339())
                                    .unwrap_or_else(|| "—".to_string());
                                ui.label(&modified);
                                ui.horizontal(|ui| {
                                    let is_predefined = p.predefined;
                                    if ui
                                        .add_enabled(!is_predefined, egui::Button::new("Rename"))
                                        .clicked()
                                    {
                                        rename_pending = Some(p.name.clone());
                                        rename_input = p.name.clone();
                                    }
                                    if ui
                                        .add_enabled(!is_predefined, egui::Button::new("Delete"))
                                        .clicked()
                                    {
                                        delete_target = Some(p.name.clone());
                                    }
                                    if ui.button("Duplicate").clicked() {
                                        duplicate_target = Some(p.name.clone());
                                    }
                                    if ui.button("Export").clicked() {
                                        export_target = Some(p.name.clone());
                                    }
                                });
                                ui.end_row();
                            }
                        });
                });
                // Preview area
                cols[1].group(|ui| {
                    ui.heading("Preview");
                    ui.label("Profile name, description, and layout summary");
                    ui.label("(thumbnail rendering wired by domain layer)");
                });
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("New").clicked() {
                    crate::ui::save_profile_dialog::open(ctx);
                }
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
            // Keyboard: Escape closes.
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                close = true;
            }
        });

    // Rename prompt (small modal; opens from a row's Rename button). Confirm
    // is enabled only for a non-empty name that does not collide with an
    // existing profile.
    if let Some(old_name) = rename_pending.clone() {
        let mut confirm = false;
        let mut cancel = false;
        egui::Window::new(format!("Rename '{}'", old_name))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("New name:");
                ui.text_edit_singleline(&mut rename_input);
                let trimmed = rename_input.trim().to_string();
                let duplicate = !trimmed.is_empty()
                    && trimmed != old_name
                    && app.profile_store.load_profile(&trimmed).is_ok();
                if duplicate {
                    ui.colored_label(
                        ui.visuals().warn_fg_color,
                        "A profile with that name already exists.",
                    );
                }
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !trimmed.is_empty() && !duplicate,
                            egui::Button::new("Rename"),
                        )
                        .clicked()
                    {
                        confirm = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if confirm {
            rename_target = Some((old_name.clone(), rename_input.trim().to_string()));
            rename_pending = None;
            rename_input.clear();
        }
        if cancel {
            rename_pending = None;
            rename_input.clear();
        }
    }

    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::new(SEARCH_KEY), search);
        d.insert_temp(egui::Id::new(RENAME_KEY), rename_pending);
        d.insert_temp(egui::Id::new(RENAME_INPUT_KEY), rename_input.clone());
    });

    if close || !open {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(OPEN_KEY), false));
    }

    // Handle actions
    if let Some(name) = delete_target {
        match app.profile_store.delete_profile(&name) {
            Ok(()) => app.set_status(
                crate::ui::app::StatusKind::Success,
                format!("Deleted profile '{}'", name),
            ),
            Err(e) => app.set_status(
                crate::ui::app::StatusKind::Error,
                format!("Delete failed: {}", e),
            ),
        }
    }
    if let Some((old_name, new_name)) = rename_target {
        // Implemented via ProfileStore rename (delete + save with new name)
        match rename_profile(&mut app.profile_store, &old_name, &new_name) {
            Ok(()) => app.set_status(
                crate::ui::app::StatusKind::Success,
                format!("Renamed '{}' to '{}'", old_name, new_name),
            ),
            Err(e) => app.set_status(
                crate::ui::app::StatusKind::Error,
                format!("Rename failed: {}", e),
            ),
        }
    }
    if let Some(name) = duplicate_target {
        match duplicate_profile(&mut app.profile_store, &name) {
            Ok(new_name) => app.set_status(
                crate::ui::app::StatusKind::Success,
                format!("Duplicated '{}' as '{}'", name, new_name),
            ),
            Err(e) => app.set_status(
                crate::ui::app::StatusKind::Error,
                format!("Duplicate failed: {}", e),
            ),
        }
    }
    if let Some(name) = export_target {
        if let Err(e) = export_profile_dialog(&app.profile_store, &name) {
            app.set_status(
                crate::ui::app::StatusKind::Error,
                format!("Export failed: {}", e),
            );
        }
    }
}

fn rename_profile(
    store: &mut crate::data::profile_store::ProfileStore,
    old_name: &str,
    new_name: &str,
) -> anyhow::Result<()> {
    let mut profile = store.load_profile(old_name)?;
    profile.name = new_name.to_string();
    profile.is_predefined = false;
    store.delete_profile(old_name)?;
    store.save_profile(&profile)?;
    Ok(())
}

fn duplicate_profile(
    store: &mut crate::data::profile_store::ProfileStore,
    name: &str,
) -> anyhow::Result<String> {
    let mut profile = store.load_profile(name)?;
    let base = profile.name.clone();
    let mut new_name = format!("{} (copy)", base);
    let mut counter = 1;
    while store.load_profile(&new_name).is_ok() {
        new_name = format!("{} (copy {})", base, counter);
        counter += 1;
    }
    profile.name = new_name.clone();
    profile.is_predefined = false;
    store.save_profile(&profile)?;
    Ok(new_name)
}

fn export_profile_dialog(
    store: &crate::data::profile_store::ProfileStore,
    name: &str,
) -> anyhow::Result<()> {
    use rfd::FileDialog;
    if let Some(path) = FileDialog::new()
        .add_filter("Rubric Profile", &["rubric-profile"])
        .set_file_name(format!("{}.rubric-profile", name))
        .save_file()
    {
        store.export_profile(name, &path)?;
    }
    Ok(())
}
