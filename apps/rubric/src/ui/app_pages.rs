//! Pages sidebar panel orchestration.
//!
//! The pages thumbnail strip now lives inside the consolidated left panel
//! (`ui/app_left_panel.rs`, UI redesign T1). The View ▸ Pages toggle now
//! shows/hides the whole consolidated panel, so this module is intentionally
//! empty of drawing code; page navigation still runs through
//! `RubricApp::jump_to_page`.
