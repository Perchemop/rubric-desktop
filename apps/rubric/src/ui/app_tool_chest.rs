//! Tool chest panel orchestration.
//!
//! The Tool Chest now lives as a tab inside the consolidated left panel
//! (`ui/app_left_panel.rs`, UI redesign T1). The module keeps the snapshot
//! handling comment: only apply-to-selection mutates the project and asks
//! for a snapshot — the consolidated panel returns that flag already.
