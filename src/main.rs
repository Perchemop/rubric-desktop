//! Binary entry point: launches the eframe app.

use eframe::{egui, NativeOptions, Result};

use rubric::ui::RubricApp;
use rubric::ui::icons::load_icon_arc;

fn main() -> Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

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
