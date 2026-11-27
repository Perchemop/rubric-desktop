//! Manage Profiles dialog (spec F-606, section 6.7).
//!
//! Lists profiles with name/type/modified columns, a preview area, search/filter,
//! and actions: New, Duplicate, Rename, Delete, Import, Export, Set Default.
//! Predefined profiles disable Delete/Rename. Integrates with ProfileStore.

use crate::domain::profile::ProfileSummary;
use anyhow;
use eframe::egui;

const OPEN_KEY: &str = "rubric.profile_manager.open";
const SEARCH_KEY: &str = "rubric.profile_manager.search";
const RENAME_KEY: &str = "rubric.profile_manager.rename";

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
    let mut rename_target: Option<(String, String)> = None; // (old_name, new_name)
    let mut duplicate_target: Option<String> = None;
    let mut export_target: Option<String> = None;
    let mut import_path: Option<String> = None;
    let mut set_default_target: Option<String> = None;

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
                                        ctx.data_mut(|d| {
                                            d.insert_temp(
                                                egui::Id::new(RENAME_KEY),
                                                p.name.clone(),
                                            );
                                        });
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
                if ui.button("Import…").clicked() {
                    import_path = Some(String::new()); // Will open file dialog via rfd
                }
                if ui.button("Set Default").clicked() {
                    set_default_target = Some(String::new()); // Will prompt for which profile
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

    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::new(SEARCH_KEY), search);
    });

    if close || !open {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(OPEN_KEY), false));
    }

    // Handle actions
    if let Some(name) = delete_target {
        if let Err(e) = app.profile_store.delete_profile(&name) {
            app.set_status(
                crate::ui::app::StatusKind::Error,
                format!("Delete failed: {}", e),
            );
        }
    }
    if let Some((old_name, new_name)) = rename_target {
        // Implemented via ProfileStore rename (delete + save with new name)
        if let Err(e) = rename_profile(&mut app.profile_store, &old_name, &new_name) {
            app.set_status(
                crate::ui::app::StatusKind::Error,
                format!("Rename failed: {}", e),
            );
        }
    }
    if let Some(name) = duplicate_target {
        if let Err(e) = duplicate_profile(&mut app.profile_store, &name) {
            app.set_status(
                crate::ui::app::StatusKind::Error,
                format!("Duplicate failed: {}", e),
            );
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
    if let Some(_path) = import_path {
        // TODO: Use rfd to pick file, then call app.profile_store.import_profile()
    }
    if let Some(_name) = set_default_target {
        // TODO: Implement set default via ProfileStore
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
) -> anyhow::Result<()> {
    let mut profile = store.load_profile(name)?;
    let base = profile.name.clone();
    let mut new_name = format!("{} (copy)", base);
    let mut counter = 1;
    while store.load_profile(&new_name).is_ok() {
        new_name = format!("{} (copy {})", base, counter);
        counter += 1;
    }
    profile.name = new_name;
    profile.is_predefined = false;
    store.save_profile(&profile)?;
    Ok(())
}

fn export_profile_dialog(
    store: &crate::data::profile_store::ProfileStore,
    name: &str,
) -> anyhow::Result<()> {
    use rfd::FileDialog;
    if let Some(path) = FileDialog::new()
        .add_filter("Rubric Profile", &["rubric-profile"])
        .set_file_name(&format!("{}.rubric-profile", name))
        .save_file()
    {
        store.export_profile(name, &path)?;
    }
    Ok(())
}
