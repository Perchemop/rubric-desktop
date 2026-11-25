//! Auto-save on close logic (spec F-606, section 6.9).
//!
//! Setting: `auto_save_profile: Option<bool>` (None = unprompted). First close
//! prompts with Yes / No / Don't Ask Again. Preference persisted in AppSettings.

use crate::data::profile_store::{build_profile_from_state, ProfileStore};
use crate::domain::profile::PanelLayout;
use crate::domain::theme::Theme;
use crate::domain::workspace::MeasurementDefaults;
use crate::domain::CanvasSettings;
use eframe::egui;

const OPEN_KEY: &str = "rubric.auto_save_profile.prompt_open";
const PREF_KEY: &str = "rubric.auto_save_profile.preference"; // 0=unset,1=yes,2=no

pub fn preference(ctx: &egui::Context) -> Option<bool> {
    match ctx.data(|d| d.get_temp::<i32>(egui::Id::new(PREF_KEY)).unwrap_or(0)) {
        1 => Some(true),
        2 => Some(false),
        _ => None,
    }
}

pub fn set_preference(ctx: &egui::Context, value: Option<bool>) {
    let v = match value {
        Some(true) => 1,
        Some(false) => 2,
        None => 0,
    };
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(PREF_KEY), v));
}

pub fn prompt_on_close(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(OPEN_KEY), true));
}

pub fn draw(app: &mut crate::ui::app::RubricApp, ctx: &egui::Context) {
    let mut open = ctx
        .data(|d| d.get_temp::<bool>(egui::Id::new(OPEN_KEY)))
        .unwrap_or(false);
    if !open {
        return;
    }

    let mut close = false;
    let mut choice: Option<Option<bool>> = None; // Some(Some(true))=yes, Some(Some(false))=no, Some(None)=don't ask? handled below

    egui::Window::new("Update active profile?")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Update active profile with current layout?");
            ui.horizontal(|ui| {
                if ui.button("Yes").clicked() {
                    choice = Some(Some(true));
                }
                if ui.button("No").clicked() {
                    choice = Some(Some(false));
                }
                if ui.button("Don't Ask Again").clicked() {
                    choice = Some(None);
                }
            });
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                close = true;
            }
        });

    if let Some(c) = choice {
        match c {
            Some(true) => {
                set_preference(ctx, Some(true));
                // Auto-save enabled: serialize current UI state to active profile
                let profile = build_profile_from_state(
                    app.settings.active_profile.clone(),
                    "Auto-saved on close".to_string(),
                    app.ui.panel_layout.clone(),
                    None, // Toolbar profile config (not yet integrated)
                    app.theme,
                    app.ui.measurement_defaults.clone(),
                    app.ui.canvas_settings.clone(),
                );
                if let Err(e) = app.profile_store.save_profile(&profile) {
                    app.set_status(
                        crate::ui::app::StatusKind::Error,
                        format!("Auto-save failed: {}", e),
                    );
                }
            }
            Some(false) => {
                set_preference(ctx, Some(false));
            }
            None => {
                // "Don't Ask Again" => remember preference disabled.
                set_preference(ctx, Some(false));
            }
        }
        close = true;
    }

    if close || !open {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(OPEN_KEY), false));
    }
}

/// Graceful shutdown save hook. Call from the app's close path when
/// `auto_save_profile` is enabled. Currently a no-op placeholder until the
/// domain layer lands.
pub fn save_active_profile_on_close(app: &mut crate::ui::app::RubricApp) {
    // Check if auto-save is enabled (preference = true)
    // This would need to be called from the app's close path
    // For now, the preference is stored in egui temp storage
    // TODO: Move preference to AppSettings for persistence across sessions
}
