//! UI theme and styling helpers (egui).
//!
//! Two palettes are supported: the light default and a dark variant derived
//! from Rubric's own brand palette (`assets/branding.md`). Markup colours
//! themselves live in the domain (`src/domain/style.rs`); this module only
//! dresses the application shell.

use egui::{Color32, Stroke};

use crate::domain::style::Color;
use crate::domain::Theme;

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
/// Deep indigo used as the dark surface colour.
const INK_900: Color32 = Color32::from_rgb(20, 23, 43);
/// Slightly lighter indigo for raised surfaces in the dark theme.
const INK_700: Color32 = Color32::from_rgb(31, 36, 64);

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
        }
        Theme::Dark => {
            visuals.panel_fill = INK_700;
            visuals.window_fill = INK_700;
            visuals.extreme_bg_color = INK_900;
            visuals.faint_bg_color = Color32::from_rgb(38, 43, 72);
            visuals.hyperlink_color = ACCENT;
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

    // egui 0.36 keeps one style per theme: select the theme and install the
    // palette in that theme's style slot.
    let egui_theme = match theme {
        Theme::Light => egui::Theme::Light,
        Theme::Dark => egui::Theme::Dark,
    };
    ctx.set_theme(egui_theme);
    ctx.set_visuals_of(egui_theme, visuals);
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
}
