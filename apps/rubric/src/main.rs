//! Binary entry point: launches the eframe app.

use eframe::{egui, NativeOptions, Result};

use rubric::ui::icons::load_icon_arc;
use rubric::ui::RubricApp;

fn main() -> Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    // Crash reporting (guide §7.1): capture a scrubbed report on panic and hand
    // it to the UI for review. The report is only *shown* — nothing leaves the
    // machine until the user chooses to send it (Help ▸ Report a Bug…).
    crash_report::install_panic_hook(
        crash_report::Environment::capture(env!("CARGO_PKG_VERSION")),
        |report| {
            // No UI is safe to touch from a panic path, so park the report where
            // the app's next frame picks it up.
            if let Err(e) = std::fs::write(crash_report_path(), report.to_json()) {
                tracing::error!("could not write crash report: {e}");
            }
        },
    );

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rubric — PDF markup, review & measurement")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 640.0])
            .with_icon(load_icon_arc()),
        ..Default::default()
    };

    eframe::run_native(
        "Rubric",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(RubricApp::new(cc)))
        }),
    )
}

/// Where an unhandled crash report is parked for review on the next launch.
pub fn crash_report_path() -> std::path::PathBuf {
    // Same config dir the app already uses for prefs; the file is never
    // uploaded automatically.
    if let Some(dir) = directories::ProjectDirs::from("", "", "rubric") {
        return dir.config_dir().join("crash-report.json");
    }
    std::env::temp_dir().join("rubric-crash-report.json")
}
