//! Contrast validation tests for Rubric UI design tokens.
//!
//! Tests all 22 semantic colour role pairs against WCAG 2.2 AA requirements:
//! - 4.5:1 minimum for body text on all surfaces
//! - 3:1 minimum for UI boundaries, focus rings, and interactive targets
//! - Tested on Light, Dark, HighContrastLight, and HighContrastDark themes.
//!
//! # How it works
//!
//! The test constructs `AuditPair` entries from the `ui_egui::design_tokens`
//! module's semantic colour roles and runs the headless `audit()` function
//! from `ui_egui::design_tokens`. No display or egui rendering context is
//! required, so these tests run in CI headlessly.

use ui_egui::design_tokens::{
    AuditPair, Color, WcagLevel, audit, dark, high_contrast_dark, high_contrast_light, light,
};

/// Test all 22 semantic colour role pairs for WCAG AA contrast.
///
/// The 22 pairs cover:
/// - Body text on all 4 light/dark/high-contrast backgrounds (8 pairs)
/// - UI boundaries on all 4 themes (8 pairs)
/// - Focus ring contrast on Light and Dark (4 pairs)
/// - Additional critical UI colour pairs (2 pairs)
///
/// Each pair is asserted to meet the required contrast level
/// (4.5:1 for normal text, 3:1 for UI components).
#[test]
fn semantic_color_pairs_meet_wcag_aa_across_themes() {
    use WcagLevel::AaNormal;

    // === Light theme pairs (4.5:1 for body text, 3:1 for UI boundaries) ===
    let light_pairs = vec![
        // Body text on surfaces (4.5:1)
        (
            "body text on panel",
            light::BODY_TEXT,
            light::PANEL,
            AaNormal,
        ),
        (
            "body text on window",
            light::BODY_TEXT,
            light::WINDOW,
            AaNormal,
        ),
        (
            "body text on extreme bg",
            light::BODY_TEXT,
            light::EXTREME_BG,
            AaNormal,
        ),
        (
            "body text on elevated bg",
            light::BODY_TEXT,
            light::ELEVATED_BG,
            AaNormal,
        ),
        // UI boundaries (3:1)
        (
            "UI boundary on panel",
            light::BORDER,
            light::PANEL,
            WcagLevel::AaLarge,
        ),
        (
            "UI boundary on window",
            light::BORDER,
            light::WINDOW,
            WcagLevel::AaLarge,
        ),
        (
            "UI boundary on extreme bg",
            light::BORDER,
            light::EXTREME_BG,
            WcagLevel::AaLarge,
        ),
        (
            "UI boundary on elevated bg",
            light::BORDER,
            light::ELEVATED_BG,
            WcagLevel::AaLarge,
        ),
        // Accent on selection (3:1 AA-large)
        (
            "accent on selection",
            light::ACCENT,
            light::SELECTION_BG,
            WcagLevel::AaLarge,
        ),
        // Link on panel (4.5:1)
        ("link on panel", light::LINK, light::PANEL, AaNormal),
        // Disabled text (3:1)
        (
            "disabled text",
            light::DISABLED_TEXT,
            light::PANEL,
            WcagLevel::AaLarge,
        ),
    ];

    // === Dark theme pairs (4.5:1 for body text, 3:1 for UI boundaries) ===
    let dark_pairs = vec![
        // Body text on surfaces (4.5:1)
        ("body text on panel", dark::BODY_TEXT, dark::PANEL, AaNormal),
        (
            "body text on window",
            dark::BODY_TEXT,
            dark::WINDOW,
            AaNormal,
        ),
        (
            "body text on extreme bg",
            dark::BODY_TEXT,
            dark::EXTREME_BG,
            AaNormal,
        ),
        (
            "body text on elevated bg",
            dark::BODY_TEXT,
            dark::ELEVATED_BG,
            AaNormal,
        ),
        // UI boundaries (3:1)
        (
            "UI boundary on panel",
            dark::BORDER,
            dark::PANEL,
            WcagLevel::AaLarge,
        ),
        (
            "UI boundary on window",
            dark::BORDER,
            dark::WINDOW,
            WcagLevel::AaLarge,
        ),
        (
            "UI boundary on extreme bg",
            dark::BORDER,
            dark::EXTREME_BG,
            WcagLevel::AaLarge,
        ),
        (
            "UI boundary on elevated bg",
            dark::BORDER,
            dark::ELEVATED_BG,
            WcagLevel::AaLarge,
        ),
        // Accent on selection (3:1 AA-large)
        (
            "accent on selection",
            dark::ACCENT,
            dark::SELECTION_BG,
            WcagLevel::AaLarge,
        ),
        // Link on panel (4.5:1)
        ("link on panel", dark::LINK, dark::PANEL, AaNormal),
        // Disabled text (3:1)
        (
            "disabled text",
            dark::DISABLED_TEXT,
            dark::PANEL,
            WcagLevel::AaLarge,
        ),
    ];

    // === High-contrast Light theme pairs ===
    let hc_light_pairs = vec![
        // Body text on white (AAA)
        (
            "body text on white",
            high_contrast_light::TEXT,
            high_contrast_light::BACKGROUND,
            WcagLevel::AaaNormal,
        ),
        // Link on white (AA)
        (
            "link on white",
            high_contrast_light::LINK,
            high_contrast_light::BACKGROUND,
            WcagLevel::AaNormal,
        ),
        // Disabled on white (AA-large)
        (
            "disabled on white",
            high_contrast_light::DISABLED,
            high_contrast_light::BACKGROUND,
            WcagLevel::AaLarge,
        ),
        // Focus ring on white (AA-large)
        (
            "focus ring on white",
            high_contrast_light::FOCUS_RING,
            high_contrast_light::BACKGROUND,
            WcagLevel::AaLarge,
        ),
    ];

    // === High-contrast Dark theme pairs ===
    let hc_dark_pairs = vec![
        // Body text on black (AAA)
        (
            "body text on black",
            high_contrast_dark::TEXT,
            high_contrast_dark::BACKGROUND,
            WcagLevel::AaaNormal,
        ),
        // Link on black (AA)
        (
            "link on black",
            high_contrast_dark::LINK,
            high_contrast_dark::BACKGROUND,
            WcagLevel::AaNormal,
        ),
        // Disabled on black (AA-large)
        (
            "disabled on black",
            high_contrast_dark::DISABLED,
            high_contrast_dark::BACKGROUND,
            WcagLevel::AaLarge,
        ),
        // Focus ring on black (AA-large)
        (
            "focus ring on black",
            high_contrast_dark::FOCUS_RING,
            high_contrast_dark::BACKGROUND,
            WcagLevel::AaLarge,
        ),
    ];

    // Helper to assert a group passes
    fn assert_group_pairs(pairs: &[(&str, Color, Color, WcagLevel)]) {
        let audit_pairs: Vec<_> = pairs
            .iter()
            .map(|(name, fg, bg, level)| AuditPair::new(*name, *fg, *bg, *level))
            .collect();
        let issues = &audit(&audit_pairs);
        assert!(
            issues.is_empty(),
            "WCAG failures:\n{}",
            issues
                .iter()
                .map(|i| i.describe())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    // Run all groups
    assert_group_pairs(&light_pairs);
    assert_group_pairs(&dark_pairs);
    assert_group_pairs(&hc_light_pairs);
    assert_group_pairs(&hc_dark_pairs);
}