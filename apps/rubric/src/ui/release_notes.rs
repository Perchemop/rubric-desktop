//! In-app release-notes popup (guide §9, §12.1).
//!
//! The changelog (`CHANGELOG.md`) and the CI-managed token ledger
//! (`docs/tokens.json`) are embedded at build time so the popup works in a
//! packaged binary, not only in a source checkout.

use egui::{Align, Layout};

use crate::ui::app::RubricApp;
use crate::ui::theme;

/// Changelog embedded at build time.
const EMBEDDED_CHANGELOG: &str = include_str!("../../../../CHANGELOG.md");

/// CI-managed token ledger embedded at build time.
const EMBEDDED_TOKENS: &str = include_str!("../../../../docs/tokens.json");

/// A single parsed release entry from the changelog.
#[derive(Clone, Debug)]
pub struct Release {
    pub version: String,
    pub date: String,
    pub sections: Vec<(String, Vec<String>)>,
}

/// Parse a Keep-a-Changelog style document into release entries.
pub fn parse_releases(changelog: &str) -> Vec<Release> {
    let mut releases: Vec<Release> = Vec::new();
    let mut current: Option<Release> = None;
    let mut current_section: Option<String> = None;

    for raw in changelog.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if trimmed.starts_with("## [") {
            if let Some(r) = current.take() {
                releases.push(r);
            }
            let (version, date) = parse_header(trimmed);
            current = Some(Release {
                version,
                date,
                sections: Vec::new(),
            });
            current_section = None;
            continue;
        }
        let Some(ref mut rel) = current else { continue };

        if trimmed.starts_with("### ") {
            let heading = trimmed.trim_start_matches("### ").trim().to_string();
            current_section = Some(heading.clone());
            if !rel.sections.iter().any(|(h, _)| h == &heading) {
                rel.sections.push((heading, Vec::new()));
            }
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let bullet = trimmed.trim_start_matches(['-', '*']).trim().to_string();
            if bullet.is_empty() {
                continue;
            }
            if let Some(sec) = &current_section {
                if let Some(s) = rel.sections.iter_mut().find(|(h, _)| h == sec) {
                    s.1.push(bullet);
                }
            } else {
                // Top-level bullets attach to an implicit "Notes" section.
                if !rel.sections.iter().any(|(h, _)| h == "Notes") {
                    rel.sections.push(("Notes".to_string(), Vec::new()));
                }
                if let Some(s) = rel.sections.iter_mut().find(|(h, _)| h == "Notes") {
                    s.1.push(bullet);
                }
            }
        }
    }
    if let Some(r) = current {
        releases.push(r);
    }
    releases
}

fn parse_header(header: &str) -> (String, String) {
    // "## [0.4.10] - 2026-11-16"  or  "## [Unreleased]"
    let inner = header
        .trim_start_matches('#')
        .trim()
        .trim_start_matches('[');
    let mut parts = inner.splitn(2, ']');
    let version = parts.next().unwrap_or("").trim().to_string();
    let rest = parts.next().unwrap_or("").trim();
    let date = rest.trim_start_matches('-').trim().to_string();
    (version, date)
}

fn token_for(ledger: &serde_json::Value, version: &str) -> Option<u64> {
    ledger.get("releases")?.as_array()?.iter().find_map(|r| {
        if r.get("release_version")?.as_str()? == version {
            r.get("release_tokens")?.as_u64()
        } else {
            None
        }
    })
}

pub fn draw_release_notes(app: &mut RubricApp, ctx: &egui::Context) {
    if !app.ui.show_release_notes {
        return;
    }

    // Esc closes the popup (keyboard accessibility).
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.ui.show_release_notes = false;
        return;
    }

    let mut open = true;
    egui::Window::new("Release Notes")
        .open(&mut open)
        .resizable(true)
        .default_width(720.0)
        .default_height(560.0)
        .show(ctx, |ui| {
            ui.heading("Rubric Release Notes");
            ui.add_space(theme::SPACING_8);
            ui.separator();

            let current_version = env!("CARGO_PKG_VERSION");
            let ledger: serde_json::Value =
                serde_json::from_str(EMBEDDED_TOKENS).unwrap_or_default();
            let total = ledger
                .get("total_tokens_since_start")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            ui.horizontal(|ui| {
                ui.label(format!("Current version: {current_version}"));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.small(format!("Cumulative tokens used: {total}"));
                });
            });
            ui.separator();
            ui.add_space(theme::SPACING_8);

            let releases = parse_releases(EMBEDDED_CHANGELOG);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for release in &releases {
                    ui.add_space(theme::SPACING_8);
                    let title = if release.date.is_empty() {
                        release.version.clone()
                    } else {
                        format!("{}  —  {}", release.version, release.date)
                    };
                    ui.strong(title);
                    if let Some(t) = token_for(&ledger, &release.version) {
                        ui.small(format!("Tokens this release: {t}"));
                    }
                    ui.add_space(theme::SPACING_4);

                    for (heading, bullets) in &release.sections {
                        ui.label(egui::RichText::new(heading).strong());
                        for b in bullets {
                            ui.label(format!("• {b}"));
                        }
                        ui.add_space(theme::SPACING_4);
                    }
                    ui.separator();
                }
                if releases.is_empty() {
                    ui.label("No release notes found.");
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Check for Updates…").clicked() {
                    app.begin_update_check(ctx);
                }
                if ui.button("View on GitHub").clicked() {
                    let _ = open_url(&format!("https://github.com/{}/releases", app.repo));
                }
                if ui.button("Close").clicked() {
                    app.ui.show_release_notes = false;
                }
            });
        });

    if !open {
        app.ui.show_release_notes = false;
    }
}

fn open_url(url: &str) -> Result<(), String> {
    // Reuse the helper in app module if public; otherwise simple.
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
        Ok(())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = url;
        Err("unsupported platform".to_string())
    }
}
