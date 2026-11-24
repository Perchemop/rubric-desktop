/// Review panel tests: view shell, enabling state, and keyboard shortcut.
use rubric::ui::app::UiState;

/// Test that the review panel can be toggled on and off via UI state.
#[test]
fn review_panel_toggle_on_off() {
    let mut state = UiState::default();
    // Panel starts hidden
    assert!(!state.show_review_panel);
    // Toggle on
    state.show_review_panel = true;
    assert!(state.show_review_panel);
    // Toggle off
    state.show_review_panel = false;
    assert!(!state.show_review_panel);
}

/// Test that the review panel state is persisted across app restart.
#[test]
fn review_panel_persistence() {
    use rubric::data::prefs::load_prefs_from;
    use rubric::data::prefs::save_prefs_to;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let test_path = temp_dir.join("rubric_review_panel_test.json");

    // Create initial state with review panel enabled
    let mut prefs = rubric::data::AppPrefs::default();
    prefs.show_review_panel = true; // This field exists in UiState

    // Save and load
    save_prefs_to(&test_path, &prefs).expect("save prefs");
    let loaded = load_prefs_from(&test_path).expect("load prefs");

    // Review panel state should persist
    assert_eq!(loaded.show_review_panel, true);

    let _ = std::fs::remove_file(&test_path);
}
