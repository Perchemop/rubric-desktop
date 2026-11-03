//! UI theme and styling helpers (egui).
//!
//! Two palettes are supported: the light default and a dark variant derived
//! from Rubric's own brand palette (`assets/branding.md`). Markup colours
//! themselves live in the domain (`src/domain/style.rs`); this module only
//! dresses the application shell.

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke};

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

/// Apply the given theme to an egui context.
///
/// Dark mode keeps the document page itself light (paper stays paper); the
/// surrounding chrome uses the brand's indigo palette.
pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    let mut visuals = match theme {
        Theme::Light => egui::Visuals::light(),
        Theme::Dark => egui::Visuals::dark(),
    };

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

    // Consistent widget interaction: 4 px corner rounding for all interactive
    // widgets.
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4u8);
    visuals.widgets.active.corner_radius = CornerRadius::same(4u8);
    visuals.widgets.open.corner_radius = CornerRadius::same(4u8);

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
        apply_theme(&ctx, Theme::Dark);
        assert_eq!(ctx.theme(), egui::Theme::Dark);
        assert!(ctx.style_of(egui::Theme::Dark).visuals.dark_mode);

        apply_theme(&ctx, Theme::Light);
        assert_eq!(ctx.theme(), egui::Theme::Light);
        assert!(!ctx.style_of(egui::Theme::Light).visuals.dark_mode);
    }

    /// Spec F-607: the typography token pass must set all standard text styles
    /// with the correct font sizes for both themes.
    #[test]
    fn typography_tokens_applied_to_both_themes() {
        let ctx = egui::Context::default();

        apply_theme(&ctx, Theme::Light);
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

        apply_theme(&ctx, Theme::Dark);
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
}
