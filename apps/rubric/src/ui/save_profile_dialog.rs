//! Save Profile dialog (spec F-606, section 6.8).
//!
//! Saves current UI layout to a new profile via ProfileStore.

use crate::data::profile_store::build_profile_from_state;
use crate::domain::panel::PanelLayout;
use crate::domain::profile::MeasurementDefaults;
use crate::domain::CanvasSettings;
use eframe::egui;

const OPEN_KEY: &str = "rubric.save_profile.open";
const NAME_KEY: &str = "rubric.save_profile.name";
const DESC_KEY: &str = "rubric.save_profile.desc";
const ACTIVE_KEY: &str = "rubric.save_profile.active";

#[allow(deprecated)]
pub fn open(ctx: &egui::Context) {
    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::new(OPEN_KEY), true);
        if d.get_temp::<String>(egui::Id::new(NAME_KEY)).is_none() {
            d.insert_temp(egui::Id::new(NAME_KEY), String::new());
        }
        if d.get_temp::<String>(egui::Id::new(DESC_KEY)).is_none() {
            d.insert_temp(egui::Id::new(DESC_KEY), String::new());
        }
        if d.get_temp::<bool>(egui::Id::new(ACTIVE_KEY)).is_none() {
            d.insert_temp(egui::Id::new(ACTIVE_KEY), true);
        }
    });
}

#[allow(deprecated)]
pub fn draw(app: &mut crate::ui::app::RubricApp, ctx: &egui::Context) {
    let mut open = ctx
        .data(|d| d.get_temp::<bool>(egui::Id::new(OPEN_KEY)))
        .unwrap_or(false);
    if !open {
        return;
    }

    let mut name = ctx
        .data(|d| d.get_temp::<String>(egui::Id::new(NAME_KEY)))
        .unwrap_or_default();
    let mut desc = ctx
        .data(|d| d.get_temp::<String>(egui::Id::new(DESC_KEY)))
        .unwrap_or_default();
    let mut active = ctx
        .data(|d| d.get_temp::<bool>(egui::Id::new(ACTIVE_KEY)))
        .unwrap_or(true);

    let mut close = false;
    let mut commit = false;

    egui::Window::new("Save Profile")
        .open(&mut open)
        .resizable(false)
        .default_size(egui::vec2(420.0, 260.0))
        .show(ctx, |ui| {
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                close = true;
            }
            ui.label("Name (max 64 chars)");
            ui.add(
                egui::TextEdit::singleline(&mut name)
                    .desired_width(320.0)
                    .char_limit(64),
            );
            let name_trimmed = name.trim();
            let name_empty = name_trimmed.is_empty();
            let name_too_long = name_trimmed.chars().count() > 64;
            if name_empty {
                ui.small(
                    egui::RichText::new("Name cannot be empty").color(ui.visuals().error_fg_color),
                );
            }
            if name_too_long {
                ui.small(
                    egui::RichText::new("Name exceeds 64 characters")
                        .color(ui.visuals().error_fg_color),
                );
            }

            ui.label("Description (optional, max 256 chars)");
            ui.add(
                egui::TextEdit::multiline(&mut desc)
                    .desired_width(320.0)
                    .desired_rows(3),
            );
            if desc.chars().count() > 256 {
                ui.small(
                    egui::RichText::new("Description exceeds 256 characters")
                        .color(ui.visuals().error_fg_color),
                );
            }

            ui.checkbox(&mut active, "Set as active profile");

            ui.separator();
            ui.label("Layout preview: current panels, toolbars, theme, measurement defaults.");

            ui.separator();
            ui.horizontal(|ui| {
                let can_commit = !name_empty && !name_too_long && desc.chars().count() <= 256;
                if ui
                    .add_enabled(can_commit, egui::Button::new("OK"))
                    .clicked()
                {
                    commit = true;
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        });

    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::new(NAME_KEY), name.clone());
        d.insert_temp(egui::Id::new(DESC_KEY), desc.clone());
        d.insert_temp(egui::Id::new(ACTIVE_KEY), active);
    });

    if commit {
        // Build profile from current UI state
        // Note: Toolbar profile config not yet integrated; passing None for now
        let profile = build_profile_from_state(
            name.clone(),
            desc.clone(),
            PanelLayout::default(), // Panel layout (not yet integrated)
            None,                   // Toolbar profile config (not yet integrated)
            app.theme,
            MeasurementDefaults::default(),
            CanvasSettings::default(),
        );

        if let Err(e) = app.profile_store.save_profile(&profile) {
            app.set_status(
                crate::ui::app::StatusKind::Error,
                format!("Save failed: {}", e),
            );
        } else if active {
            // Set as active profile
            app.active_profile = name.clone();
            app.set_status(
                crate::ui::app::StatusKind::Success,
                format!(
                    "Profile '{}' saved{}",
                    name,
                    if active { " and set active" } else { "" }
                ),
            );
        }

        close = true;
    }

    if close || !open {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(OPEN_KEY), false));
    }
}
