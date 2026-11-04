//! UI theme and styling helpers (egui).
//!
//! Two palettes are supported: the light default and a dark variant derived
//! from Rubric's own brand palette (`assets/branding.md`). Markup colours
//! themselves live in the domain (`src/domain/style.rs`); this module only
//! dresses the application shell.

use egui::{Color32, CornerRadius, FontFamily, FontId, Shadow, Stroke};

use crate::domain::style::Color;
use crate::domain::Theme;

/// Type scale tokens (pixels), applied uniformly across light/dark themes.
pub const TEXT_SIZE_TITLE: f32 = 20.0;
pub const TEXT_SIZE_HEADING: f32 = 16.0;
pub const TEXT_SIZE_SUBHEADING: f32 = 14.0;
pub const TEXT_SIZE_BODY: f32 = 13.0;
pub const TEXT_SIZE_BODY_SMALL: f32 = 12.0;
pub const TEXT_SIZE_SECONDARY: f32 = 11.0;

/// Monospace token used for numeric readouts (cursor coordinates, scale, units).
pub const TEXT_MONOSPACE: FontId = FontId::new(TEXT_SIZE_BODY, FontFamily::Monospace);

/// Spacing grid (pixels).
pub const SPACING_4: f32 = 4.0;
pub const SPACING_8: f32 = 8.0;
pub const SPACING_12: f32 = 12.0;
pub const SPACING_16: f32 = 16.0;
pub const SPACING_24: f32 = 24.0;

pub fn to_color32(c: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

pub fn from_color32(c: Color32) -> Color {
    let [r, g, b, a] = c.to_array();
    Color { r, g, b, a }
}

pub fn stroke_width_to_egui(w: f32) -> f32 {
    w.max(0.25)
}

pub fn egui_stroke(c: Color, w: f32) -> Stroke {
    Stroke::new(stroke_width_to_egui(w), to_color32(c))
}

/// Brand accent (Rubric red-orange) used for selection and highlights.
const ACCENT: Color32 = Color32::from_rgb(228, 87, 46);
/// Lighter brand accent (brand "Accent 2") used as the dark-theme link colour
/// so link text clears WCAG AA on the deep-indigo surface.
const ACCENT_2: Color32 = Color32::from_rgb(255, 122, 69);
/// Deep indigo used as the dark surface colour.
const INK_900: Color32 = Color32::from_rgb(20, 23, 43);
/// Slightly lighter indigo for raised surfaces in the dark theme.
const INK_700: Color32 = Color32::from_rgb(31, 36, 64);
/// Body text on the light chrome (audited, spec F-605).
const LIGHT_TEXT: Color32 = Color32::from_rgb(32, 34, 48);
/// Body text on the dark chrome (audited, spec F-605).
const DARK_TEXT: Color32 = Color32::from_rgb(232, 234, 242);

/// Elevation shadow tokens (P1-6). Three levels for chrome depth:
///
///   - ELEVATION_1: subtle lift for buttons, chips, small popovers
///   - ELEVATION_2: panel surfaces (inspector, tool chest, markups list)
///   - ELEVATION_3: modal dialogs and floating windows
///
/// Shadows use a soft indigo blur on dark themes, a muted grey blur on light.
const ELEVATION_1: Shadow = Shadow {
    offset: [0, 1],
    blur: 4,
    spread: 0,
    color: Color32::from_rgba_premultiplied(0, 0, 0, 60),
};
const ELEVATION_2: Shadow = Shadow {
    offset: [0, 2],
    blur: 8,
    spread: 0,
    color: Color32::from_rgba_premultiplied(0, 0, 0, 80),
};

/// Elevation shadow tokens for the dark theme (deeper contrast on indigo).
const ELEVATION_1_DARK: Shadow = Shadow {
    offset: [0, 1],
    blur: 6,
    spread: 0,
    color: Color32::from_rgba_premultiplied(0, 0, 0, 120),
};
const ELEVATION_2_DARK: Shadow = Shadow {
    offset: [0, 3],
    blur: 12,
    spread: 0,
    color: Color32::from_rgba_premultiplied(0, 0, 0, 140),
};

/// High-contrast text on the light high-contrast surface (pure black).
const HC_LIGHT_TEXT: Color32 = Color32::from_rgb(0, 0, 0);
/// High-contrast text on the dark high-contrast surface (pure white).
const HC_DARK_TEXT: Color32 = Color32::from_rgb(255, 255, 255);
/// High-contrast link on white (dark blue, clears WCAG AA on white).
const HC_LIGHT_LINK: Color32 = Color32::from_rgb(0, 0, 180);
/// High-contrast link on black (pale blue, clears WCAG AA on black).
const HC_DARK_LINK: Color32 = Color32::from_rgb(130, 190, 255);
/// High-contrast selection fill on white (dark blue with white selection text).
const HC_LIGHT_SELECT: Color32 = Color32::from_rgb(0, 0, 180);
/// High-contrast selection fill on black (amber with black selection text).
const HC_DARK_SELECT: Color32 = Color32::from_rgb(255, 214, 0);

/// Apply the given theme to an egui context.
///
/// Dark mode keeps the document page itself light (paper stays paper); the
/// surrounding chrome uses the brand's indigo palette.
///
/// When `high_contrast` is set (a11y A3 / proposal P2-15), both themes switch
/// to pure black-on-white or white-on-black text and surfaces with stronger
/// widget borders, trading brand colour for maximum legibility.
pub fn apply_theme(ctx: &egui::Context, theme: Theme, high_contrast: bool) {
    let mut visuals = match theme {
        Theme::Light => egui::Visuals::light(),
        Theme::Dark => egui::Visuals::dark(),
    };

    if high_contrast {
        match theme {
            Theme::Light => {
                visuals.panel_fill = Color32::WHITE;
                visuals.window_fill = Color32::WHITE;
                visuals.extreme_bg_color = Color32::WHITE;
                visuals.faint_bg_color = Color32::from_rgb(240, 240, 240);
                visuals.override_text_color = Some(HC_LIGHT_TEXT);
                visuals.hyperlink_color = HC_LIGHT_LINK;
            }
            Theme::Dark => {
                visuals.panel_fill = Color32::BLACK;
                visuals.window_fill = Color32::BLACK;
                visuals.extreme_bg_color = Color32::BLACK;
                visuals.faint_bg_color = Color32::from_rgb(20, 20, 20);
                visuals.override_text_color = Some(HC_DARK_TEXT);
                visuals.hyperlink_color = HC_DARK_LINK;
            }
        }
        // Strong 1 px borders on every widget so edges never rely on subtle
        // surface tints to be visible.
        let border = Stroke::new(1.0, visuals.override_text_color.unwrap_or(HC_LIGHT_TEXT));
        visuals.widgets.noninteractive.bg_stroke = border;
        visuals.widgets.inactive.bg_stroke = border;
        visuals.widgets.hovered.bg_stroke = border;
        visuals.widgets.active.bg_stroke = border;
        visuals.widgets.open.bg_stroke = border;
        match theme {
            Theme::Light => {
                visuals.selection.bg_fill = HC_LIGHT_SELECT;
                visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
            }
            Theme::Dark => {
                visuals.selection.bg_fill = HC_DARK_SELECT;
                visuals.selection.stroke = Stroke::new(1.0, Color32::BLACK);
            }
        }
    } else {
        match theme {
            Theme::Light => {
                visuals.panel_fill = Color32::from_rgb(250, 250, 252);
                visuals.window_fill = Color32::from_rgb(252, 252, 253);
                visuals.extreme_bg_color = Color32::from_rgb(235, 238, 244);
                visuals.faint_bg_color = Color32::from_rgb(240, 242, 247);
                visuals.override_text_color = Some(LIGHT_TEXT);
            }
            Theme::Dark => {
                visuals.panel_fill = INK_700;
                visuals.window_fill = INK_700;
                visuals.extreme_bg_color = INK_900;
                visuals.faint_bg_color = Color32::from_rgb(38, 43, 72);
                visuals.override_text_color = Some(DARK_TEXT);
                visuals.hyperlink_color = ACCENT_2;
            }
        }
    }

    // Brand-accent interaction colours. High-contrast mode sets its own
    // selection fill/stroke above, so it is left untouched here.
    if !high_contrast {
        visuals.selection.bg_fill = ACCENT;
        visuals.selection.stroke = Stroke::new(1.0, ACCENT);
        visuals.widgets.hovered.bg_fill = match theme {
            Theme::Light => Color32::from_rgb(228, 231, 240),
            Theme::Dark => Color32::from_rgb(46, 52, 86),
        };
        visuals.widgets.active.bg_fill = match theme {
            Theme::Light => Color32::from_rgb(214, 219, 232),
            Theme::Dark => Color32::from_rgb(58, 65, 104),
        };
        visuals.widgets.open.bg_fill = visuals.widgets.active.bg_fill;
    }

    // Consistent widget interaction: 4 px corner rounding for all interactive
    // widgets.
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4u8);
    visuals.widgets.active.corner_radius = CornerRadius::same(4u8);
    visuals.widgets.open.corner_radius = CornerRadius::same(4u8);

    // Elevation shadows for chrome depth (P1-6).
    visuals.window_shadow = match theme {
        Theme::Light => ELEVATION_2,
        Theme::Dark => ELEVATION_2_DARK,
    };
    visuals.popup_shadow = match theme {
        Theme::Light => ELEVATION_1,
        Theme::Dark => ELEVATION_1_DARK,
    };

    // egui 0.36 keeps one style per theme: select the theme and install the
    // palette in that theme's style slot.
    let egui_theme = match theme {
        Theme::Light => egui::Theme::Light,
        Theme::Dark => egui::Theme::Dark,
    };
    ctx.set_theme(egui_theme);
    ctx.set_visuals_of(egui_theme, visuals);

    // Type scale tokens applied to both themes (proposal 11: typography pass).
    let mut style = (*ctx.style_of(egui_theme)).clone();
    style.text_styles = [
        (
            egui::TextStyle::Small,
            FontId::new(TEXT_SIZE_SECONDARY, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Body,
            FontId::new(TEXT_SIZE_BODY, FontFamily::Proportional),
        ),
        (egui::TextStyle::Monospace, TEXT_MONOSPACE),
        (
            egui::TextStyle::Button,
            FontId::new(TEXT_SIZE_BODY, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Heading,
            FontId::new(TEXT_SIZE_HEADING, FontFamily::Proportional),
        ),
    ]
    .into();
    ctx.set_style_of(egui_theme, style);
}

/// A small, ergonomic icon/label helper.
#[allow(dead_code)]
pub struct IconLabel {
    text: &'static str,
}

impl IconLabel {
    #[allow(dead_code)]
    pub fn new(t: &'static str) -> Self {
        Self { text: t }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_theme_selects_matching_theme_and_visuals() {
        let ctx = egui::Context::default();
        apply_theme(&ctx, Theme::Dark, false);
        assert_eq!(ctx.theme(), egui::Theme::Dark);
        assert!(ctx.style_of(egui::Theme::Dark).visuals.dark_mode);

        apply_theme(&ctx, Theme::Light, false);
        assert_eq!(ctx.theme(), egui::Theme::Light);
        assert!(!ctx.style_of(egui::Theme::Light).visuals.dark_mode);
    }

    /// Spec F-607: the typography token pass must set all standard text styles
    /// with the correct font sizes for both themes.
    #[test]
    fn typography_tokens_applied_to_both_themes() {
        let ctx = egui::Context::default();

        apply_theme(&ctx, Theme::Light, false);
        let light_styles = ctx.style_of(egui::Theme::Light).text_styles.clone();
        assert_eq!(
            light_styles.get(&egui::TextStyle::Body).unwrap().size,
            TEXT_SIZE_BODY
        );
        assert_eq!(
            light_styles.get(&egui::TextStyle::Small).unwrap().size,
            TEXT_SIZE_SECONDARY
        );
        assert_eq!(
            light_styles.get(&egui::TextStyle::Button).unwrap().size,
            TEXT_SIZE_BODY
        );
        assert_eq!(
            light_styles.get(&egui::TextStyle::Heading).unwrap().size,
            TEXT_SIZE_HEADING
        );

        apply_theme(&ctx, Theme::Dark, false);
        let dark_styles = ctx.style_of(egui::Theme::Dark).text_styles.clone();
        assert_eq!(
            dark_styles.get(&egui::TextStyle::Body).unwrap().size,
            TEXT_SIZE_BODY
        );
        assert_eq!(
            dark_styles.get(&egui::TextStyle::Monospace).unwrap().size,
            TEXT_SIZE_BODY
        );
        assert_eq!(
            dark_styles.get(&egui::TextStyle::Heading).unwrap().size,
            TEXT_SIZE_HEADING
        );
    }

    /// Spec F-605: the shipped chrome palettes must clear WCAG AA for body
    /// text, and AA-large for the accent-on-surface selection/link pairs.
    #[test]
    fn shipped_palettes_meet_wcag_contrast() {
        use crate::domain::a11y::{audit, AuditPair, WcagLevel};

        let light_panel = Color32::from_rgb(250, 250, 252);
        let light_window = Color32::from_rgb(252, 252, 253);
        let light_extreme = Color32::from_rgb(235, 238, 244);
        let dark_faint = Color32::from_rgb(38, 43, 72);

        let pairs = vec![
            AuditPair::new(
                "light body text on panel",
                from_color32(LIGHT_TEXT),
                from_color32(light_panel),
                WcagLevel::AaNormal,
            ),
            AuditPair::new(
                "light body text on window",
                from_color32(LIGHT_TEXT),
                from_color32(light_window),
                WcagLevel::AaNormal,
            ),
            AuditPair::new(
                "light body text on extreme bg",
                from_color32(LIGHT_TEXT),
                from_color32(light_extreme),
                WcagLevel::AaNormal,
            ),
            AuditPair::new(
                "dark body text on panel",
                from_color32(DARK_TEXT),
                from_color32(INK_700),
                WcagLevel::AaNormal,
            ),
            AuditPair::new(
                "dark body text on extreme bg",
                from_color32(DARK_TEXT),
                from_color32(INK_900),
                WcagLevel::AaNormal,
            ),
            AuditPair::new(
                "dark body text on faint bg",
                from_color32(DARK_TEXT),
                from_color32(dark_faint),
                WcagLevel::AaNormal,
            ),
            // Accent is a selection *fill*, not body text: AA-large applies.
            AuditPair::new(
                "light text on accent selection",
                from_color32(LIGHT_TEXT),
                from_color32(ACCENT),
                WcagLevel::AaLarge,
            ),
            AuditPair::new(
                "dark text on accent selection",
                from_color32(DARK_TEXT),
                from_color32(ACCENT),
                WcagLevel::AaLarge,
            ),
            AuditPair::new(
                "dark link on panel",
                from_color32(ACCENT_2),
                from_color32(INK_700),
                WcagLevel::AaNormal,
            ),
        ];

        let issues = audit(&pairs);
        let report = issues
            .iter()
            .map(|i| i.describe())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(issues.is_empty(), "WCAG failures:\n{report}");
    }

    /// Spec F-605 / proposal P2-15: the high-contrast palettes must clear WCAG
    /// AAA for text-on-surface (pure black/white) and AA for links and the
    /// filled selection (white-on-navy and black-on-amber).
    #[test]
    fn high_contrast_palettes_meet_wcag() {
        use crate::domain::a11y::{audit, AuditPair, WcagLevel};

        let pairs = vec![
            AuditPair::new(
                "hc light body text on white",
                from_color32(HC_LIGHT_TEXT),
                from_color32(Color32::WHITE),
                WcagLevel::AaaNormal,
            ),
            AuditPair::new(
                "hc dark body text on black",
                from_color32(HC_DARK_TEXT),
                from_color32(Color32::BLACK),
                WcagLevel::AaaNormal,
            ),
            AuditPair::new(
                "hc light link on white",
                from_color32(HC_LIGHT_LINK),
                from_color32(Color32::WHITE),
                WcagLevel::AaNormal,
            ),
            AuditPair::new(
                "hc dark link on black",
                from_color32(HC_DARK_LINK),
                from_color32(Color32::BLACK),
                WcagLevel::AaNormal,
            ),
            AuditPair::new(
                "hc light selection text on fill",
                from_color32(Color32::WHITE),
                from_color32(HC_LIGHT_SELECT),
                WcagLevel::AaLarge,
            ),
            AuditPair::new(
                "hc dark selection text on fill",
                from_color32(Color32::BLACK),
                from_color32(HC_DARK_SELECT),
                WcagLevel::AaLarge,
            ),
        ];

        let issues = audit(&pairs);
        let report = issues
            .iter()
            .map(|i| i.describe())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(issues.is_empty(), "WCAG failures:\n{report}");
    }

    /// Spec F-605 / proposal P2-15: enabling high contrast switches both themes
    /// to pure black/white text and solid surfaces.
    #[test]
    fn high_contrast_flag_sets_pure_surfaces() {
        let ctx = egui::Context::default();

        apply_theme(&ctx, Theme::Light, true);
        let light = ctx.style_of(egui::Theme::Light).visuals.clone();
        assert_eq!(light.panel_fill, Color32::WHITE);
        assert_eq!(light.override_text_color, Some(HC_LIGHT_TEXT));

        apply_theme(&ctx, Theme::Dark, true);
        let dark = ctx.style_of(egui::Theme::Dark).visuals.clone();
        assert_eq!(dark.panel_fill, Color32::BLACK);
        assert_eq!(dark.override_text_color, Some(HC_DARK_TEXT));
    }
}
