//! UI design tokens and WCAG 2.2 AA contrast validation for egui.
//!
//! This crate defines the semantic colour roles used across the Rubric application
//! UI and provides a pure, headless contrast-audit function that CI can gate on.
//! Colours are stored as raw RGB values (alpha is ignored for contrast); the
//! mapping to egui Visuals happens in the app layer.

use egui::Color32;
use serde::{Deserialize, Serialize};

/// WCAG 2.x contrast requirement level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WcagLevel {
    /// Level AA, normal-size text: contrast ≥ 4.5:1.
    AaNormal,
    /// Level AA, large text and UI components: contrast ≥ 3:1.
    AaLarge,
    /// Level AAA, normal-size text: contrast ≥ 7:1.
    AaaNormal,
}

impl WcagLevel {
    /// The minimum contrast ratio this level requires.
    pub fn min_ratio(self) -> f64 {
        match self {
            WcagLevel::AaNormal => 4.5,
            WcagLevel::AaLarge => 3.0,
            WcagLevel::AaaNormal => 7.0,
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            WcagLevel::AaNormal => "AA normal text",
            WcagLevel::AaLarge => "AA large text / UI",
            WcagLevel::AaaNormal => "AAA normal text",
        }
    }
}

/// Linearise an sRGB channel (0–255) for WCAG luminance calculation.
fn linearise(channel: u8) -> f64 {
    let c = channel as f64 / 255.0;
    if c <= 0.039_28 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Relative luminance of an sRGB colour (WCAG 2.x). Alpha is ignored.
pub fn relative_luminance(col: Color32) -> f64 {
    0.2126 * linearise(col.r()) + 0.7152 * linearise(col.g()) + 0.0722 * linearise(col.b())
}

/// WCAG contrast ratio between two opaque colours, always ≥ 1.
pub fn contrast_ratio(a: Color32, b: Color32) -> f64 {
    let (l1, l2) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

/// Whether the pair meets the given WCAG level.
pub fn meets(level: WcagLevel, fg: Color32, bg: Color32) -> bool {
    contrast_ratio(fg, bg) >= level.min_ratio()
}

/// A single sRGB colour (raw RGB, alpha ignored for contrast).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color(u8, u8, u8);

impl Color {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }

    pub const fn from_egui(c: Color32) -> Self {
        Self::from_rgb(c.r(), c.g(), c.b())
    }
}

impl From<Color32> for Color {
    fn from(c: Color32) -> Self {
        Color::from_egui(c)
    }
}

impl From<Color> for Color32 {
    fn from(c: Color) -> Color32 {
        Color32::from_rgb(c.0, c.1, c.2)
    }
}

/// A palette pair name, foreground and background, audited together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPair {
    pub name: String,
    pub fg: Color,
    pub bg: Color,
    pub level: WcagLevel,
}

impl AuditPair {
    pub fn new(name: impl Into<String>, fg: Color, bg: Color, level: WcagLevel) -> Self {
        Self {
            name: name.into(),
            fg,
            bg,
            level,
        }
    }
}

/// A pair that failed its required contrast.
#[derive(Debug, Clone)]
pub struct A11yIssue {
    pub name: String,
    pub ratio: f64,
    pub required: f64,
    pub level: WcagLevel,
}

impl A11yIssue {
    /// A one-line description for logs, reports and test failures.
    pub fn describe(&self) -> String {
        format!(
            "{}: ratio {:.2}:1 is below the required {:.1}:1 ({})",
            self.name,
            self.ratio,
            self.required,
            self.level.label()
        )
    }
}

/// Audit a set of colour pairs and return every failure, in input order.
pub fn audit(pairs: &[AuditPair]) -> Vec<A11yIssue> {
    pairs
        .iter()
        .filter_map(|p| {
            let ratio = contrast_ratio(p.fg.into(), p.bg.into());
            (ratio < p.level.min_ratio()).then(|| A11yIssue {
                name: p.name.clone(),
                ratio,
                required: p.level.min_ratio(),
                level: p.level,
            })
        })
        .collect()
}

/// Default light theme semantic colour roles (RGB values, alpha ignored for contrast).
pub mod light {
    use super::{Color, WcagLevel};

    /// Panel surface colour (light indigo off-white).
    pub const PANEL: Color = Color::from_rgb(250, 250, 252);
    /// Window background colour (off-white).
    pub const WINDOW: Color = Color::from_rgb(252, 252, 253);
    /// Extreme background (page area far from chrome).
    pub const EXTREME_BG: Color = Color::from_rgb(235, 238, 244);
    /// Body text colour.
    pub const BODY_TEXT: Color = Color::from_rgb(32, 34, 48);
    /// Secondary/ muted text colour.
    pub const SECONDARY_TEXT: Color = Color::from_rgb(100, 105, 120);
    /// Accent colour (Rubric red-orange).
    pub const ACCENT: Color = Color::from_rgb(220, 80, 35);
    /// Accent 2 (link colour in dark theme).
    pub const ACCENT_2: Color = Color::from_rgb(255, 122, 69);
    /// Disabled text colour.
    pub const DISABLED_TEXT: Color = Color::from_rgb(138, 143, 158);
    /// Placeholder text colour.
    pub const PLACEHOLDER_TEXT: Color = Color::from_rgb(160, 165, 180);
    /// Border / outline colour.
    pub const BORDER: Color = Color::from_rgb(100, 100, 105);
    /// Focus ring colour.
    pub const FOCUS_RING: Color = Color::from_rgb(228, 87, 46);
    /// Selection fill colour.
    pub const SELECTION_BG: Color = Color::from_rgb(230, 230, 230);
    /// Selection text colour.
    pub const SELECTION_FG: Color = Color::from_rgb(255, 255, 255);
    /// Link colour.
    pub const LINK: Color = Color::from_rgb(0, 110, 190);
    /// Background colour for forms/input areas.
    pub const INPUT_BG: Color = Color::from_rgb(255, 255, 255);
    /// Hover background colour.
    pub const HOVER_BG: Color = Color::from_rgb(228, 231, 240);
    /// Elevated surface colour (cards, chips).
    pub const ELEVATED_BG: Color = Color::from_rgb(255, 255, 255);
    /// Success colour.
    pub const SUCCESS: Color = Color::from_rgb(46, 168, 106);
    /// Warning colour.
    pub const WARNING: Color = Color::from_rgb(240, 173, 78);
    /// Danger colour.
    pub const DANGER: Color = Color::from_rgb(226, 58, 46);
    /// Muted background colour (striped areas, alternate rows).
    pub const MUTED_BG: Color = Color::from_rgb(245, 245, 245);
}

/// Default dark theme semantic colour roles (RGB values, alpha ignored for contrast).
pub mod dark {
    use super::{Color, WcagLevel};

    /// Panel surface colour (deep indigo).
    pub const PANEL: Color = Color::from_rgb(20, 23, 43);
    /// Window background colour (deep indigo).
    pub const WINDOW: Color = Color::from_rgb(20, 23, 43);
    /// Extreme background (page area far from chrome).
    pub const EXTREME_BG: Color = Color::from_rgb(38, 43, 72);
    /// Body text colour.
    pub const BODY_TEXT: Color = Color::from_rgb(232, 234, 242);
    /// Secondary/ muted text colour.
    pub const SECONDARY_TEXT: Color = Color::from_rgb(180, 185, 200);
    /// Accent colour (Rubric red-orange).
    pub const ACCENT: Color = Color::from_rgb(220, 80, 35);
    /// Accent 2 (link colour in dark theme).
    pub const ACCENT_2: Color = Color::from_rgb(255, 122, 69);
    /// Disabled text colour.
    pub const DISABLED_TEXT: Color = Color::from_rgb(138, 143, 158);
    /// Placeholder text colour.
    pub const PLACEHOLDER_TEXT: Color = Color::from_rgb(160, 165, 180);
    /// Border / outline colour.
    pub const BORDER: Color = Color::from_rgb(100, 105, 120);
    /// Focus ring colour.
    pub const FOCUS_RING: Color = Color::from_rgb(228, 87, 46);
    /// Selection fill colour.
    pub const SELECTION_BG: Color = Color::from_rgb(230, 230, 230);
    /// Selection text colour.
    pub const SELECTION_FG: Color = Color::from_rgb(255, 255, 255);
    /// Link colour.
    pub const LINK: Color = Color::from_rgb(0, 180, 216);
    /// Background colour for forms/input areas.
    pub const INPUT_BG: Color = Color::from_rgb(38, 43, 72);
    /// Hover background colour.
    pub const HOVER_BG: Color = Color::from_rgb(46, 52, 86);
    /// Elevated surface colour (cards, chips).
    pub const ELEVATED_BG: Color = Color::from_rgb(38, 43, 72);
    /// Success colour.
    pub const SUCCESS: Color = Color::from_rgb(46, 168, 106);
    /// Warning colour.
    pub const WARNING: Color = Color::from_rgb(240, 173, 78);
    /// Danger colour.
    pub const DANGER: Color = Color::from_rgb(226, 58, 46);
    /// Muted background colour (striped areas, alternate rows).
    pub const MUTED_BG: Color = Color::from_rgb(245, 245, 245);
}

/// High-contrast light theme semantic colour roles (pure black/white).
pub mod high_contrast_light {
    use super::{Color, WcagLevel};

    pub const TEXT: Color = Color::from_rgb(0, 0, 0);
    pub const SELECTION_BG: Color = Color::from_rgb(0, 0, 180);
    pub const SELECTION_FG: Color = Color::from_rgb(255, 255, 255);
    pub const LINK: Color = Color::from_rgb(0, 0, 180);
    pub const BACKGROUND: Color = Color::from_rgb(255, 255, 255);
    pub const DISABLED: Color = Color::from_rgb(138, 143, 158);
    pub const BORDER: Color = Color::from_rgb(0, 0, 0);
    pub const FOCUS_RING: Color = Color::from_rgb(0, 0, 0);
}

/// High-contrast dark theme semantic colour roles (pure black/white).
pub mod high_contrast_dark {
    use super::{Color, WcagLevel};

    pub const TEXT: Color = Color::from_rgb(255, 255, 255);
    pub const SELECTION_BG: Color = Color::from_rgb(255, 214, 0);
    pub const SELECTION_FG: Color = Color::from_rgb(0, 0, 0);
    pub const LINK: Color = Color::from_rgb(130, 190, 255);
    pub const BACKGROUND: Color = Color::from_rgb(0, 0, 0);
    pub const DISABLED: Color = Color::from_rgb(138, 143, 158);
    pub const BORDER: Color = Color::from_rgb(255, 255, 255);
    pub const FOCUS_RING: Color = Color::from_rgb(255, 255, 255);
}
