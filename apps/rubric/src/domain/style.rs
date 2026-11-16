//! Colours, strokes and dash patterns for markups.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::markup::MarkupKind;

/// An RGBA colour, stored unmultiplied (matches how designers specify colour).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    pub fn to_tuple(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::rgb(0, 0, 0)
    }
}

/// Named colours that mirror the common default swatches in review tools.
pub mod palette {
    use super::Color;

    pub const BLACK: Color = Color::rgb(20, 20, 24);
    pub const RED: Color = Color::rgb(226, 58, 46);
    pub const ORANGE: Color = Color::rgb(255, 122, 69);
    pub const AMBER: Color = Color::rgb(240, 173, 78);
    pub const YELLOW: Color = Color::rgb(255, 214, 84);
    pub const GREEN: Color = Color::rgb(46, 168, 106);
    pub const TEAL: Color = Color::rgb(38, 166, 154);
    pub const BLUE: Color = Color::rgb(46, 134, 171);
    pub const INDIGO: Color = Color::rgb(76, 88, 168);
    pub const VIOLET: Color = Color::rgb(140, 86, 196);
    pub const PINK: Color = Color::rgb(232, 106, 160);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const GREY: Color = Color::rgb(138, 143, 158);

    pub const SWATCHES: [Color; 12] = [
        BLACK, RED, ORANGE, AMBER, YELLOW, GREEN, TEAL, BLUE, INDIGO, VIOLET, PINK, GREY,
    ];
}

/// Dash pattern for a stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DashPattern {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

impl DashPattern {
    pub const ALL: [DashPattern; 3] =
        [DashPattern::Solid, DashPattern::Dashed, DashPattern::Dotted];

    pub fn label(self) -> &'static str {
        match self {
            DashPattern::Solid => "Solid",
            DashPattern::Dashed => "Dashed",
            DashPattern::Dotted => "Dotted",
        }
    }

    /// Dash/gap lengths in page points, or `None` for a solid line.
    pub fn segments(self, width: f32) -> Option<[f32; 2]> {
        match self {
            DashPattern::Solid => None,
            DashPattern::Dashed => Some([width * 3.0, width * 2.0]),
            DashPattern::Dotted => Some([width * 0.5, width * 2.0]),
        }
    }
}

/// Optional fill for closed shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FillMode {
    None,
    Solid,
    /// Translucent "highlighter" wash behind the stroke.
    Highlight,
}

impl FillMode {
    pub fn alpha(self, base: u8) -> u8 {
        match self {
            FillMode::None => 0,
            FillMode::Solid => base,
            FillMode::Highlight => (base as f32 * 0.35) as u8,
        }
    }
}

/// Visual style applied to a markup.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MarkupStyle {
    pub stroke: Color,
    pub fill: Color,
    pub fill_mode: FillMode,
    /// Stroke width in points.
    pub width: f32,
    pub dash: DashPattern,
    /// Font size in points (for text-bearing markups).
    pub font_size: f32,
}

impl Default for MarkupStyle {
    fn default() -> Self {
        Self {
            stroke: palette::RED,
            fill: palette::RED,
            fill_mode: FillMode::None,
            width: 2.0,
            dash: DashPattern::Solid,
            font_size: 12.0,
        }
    }
}

impl MarkupStyle {
    pub fn with_stroke(mut self, c: Color) -> Self {
        self.stroke = c;
        self.fill = c;
        self
    }

    pub fn with_width(mut self, w: f32) -> Self {
        self.width = w.max(0.25);
        self
    }
}

/// Per-kind default style overrides, persisted app-level.
///
/// When a kind has an override, new markups of that kind use `style`
/// instead of [`MarkupKind::default_style()`]. Keys are the serde names of
/// [`MarkupKind`] variants so persisted keys never drift from the enum.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DefaultStyles {
    overrides: HashMap<String, MarkupStyle>,
}

impl DefaultStyles {
    /// Record `style` as the per-kind default for `kind`.
    pub fn set_default(&mut self, kind: MarkupKind, style: MarkupStyle) {
        self.overrides.insert(Self::key(kind), style);
    }

    /// Remove any per-kind default override for `kind`.
    pub fn clear_default(&mut self, kind: MarkupKind) {
        self.overrides.remove(&Self::key(kind));
    }

    /// The per-kind default override for `kind`, if one has been set.
    pub fn get_default(&self, kind: MarkupKind) -> Option<&MarkupStyle> {
        self.overrides.get(&Self::key(kind))
    }

    /// The effective style for `kind`: the per-kind override if set,
    /// otherwise [`MarkupKind::default_style()`].
    pub fn effective_style(&self, kind: MarkupKind) -> MarkupStyle {
        self.get_default(kind)
            .copied()
            .unwrap_or_else(|| kind.default_style())
    }

    /// The serde name of `kind`, used as the map key so persisted keys stay in
    /// sync with the enum's serialised representation.
    fn key(kind: MarkupKind) -> String {
        match serde_json::to_value(kind) {
            Ok(serde_json::Value::String(s)) => s,
            _ => kind.label().to_string(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_alpha_modes() {
        assert_eq!(FillMode::None.alpha(255), 0);
        assert_eq!(FillMode::Solid.alpha(255), 255);
        assert!(FillMode::Highlight.alpha(255) < 255);
    }

    #[test]
    fn dash_segments_scale_with_width() {
        assert_eq!(DashPattern::Solid.segments(2.0), None);
        assert!(DashPattern::Dashed.segments(2.0).is_some());
    }

    #[test]
    fn default_styles_effective_uses_override_when_set() {
        let mut ds = DefaultStyles::default();
        assert_eq!(
            ds.effective_style(MarkupKind::Rectangle),
            MarkupKind::Rectangle.default_style(),
            "unset kinds fall back to the coded default"
        );
        let custom = MarkupStyle {
            stroke: palette::BLUE,
            width: 4.0,
            ..MarkupStyle::default()
        };
        ds.set_default(MarkupKind::Rectangle, custom);
        assert_eq!(
            ds.get_default(MarkupKind::Rectangle),
            Some(&custom),
            "get_default returns the override"
        );
        assert_eq!(ds.effective_style(MarkupKind::Rectangle), custom);
        // Other kinds keep their coded default.
        assert_eq!(
            ds.effective_style(MarkupKind::Ellipse),
            MarkupKind::Ellipse.default_style()
        );
        ds.clear_default(MarkupKind::Rectangle);
        assert_eq!(ds.get_default(MarkupKind::Rectangle), None);
        assert_eq!(
            ds.effective_style(MarkupKind::Rectangle),
            MarkupKind::Rectangle.default_style(),
            "clearing restores the coded default"
        );
    }

    #[test]
    fn default_styles_serialise_key_is_serde_name() {
        let mut ds = DefaultStyles::default();
        ds.set_default(MarkupKind::Highlight, MarkupStyle::default());
        let json = serde_json::to_string(&ds).expect("serialize");
        let key = serde_json::to_value(MarkupKind::Highlight)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .expect("a kind serialises as a string");
        assert!(
            json.contains(&format!("\"{key}\"")),
            "persisted key must be the serde name: {json}"
        );
    }

    #[test]
    fn default_styles_serde_roundtrip() {
        let mut ds = DefaultStyles::default();
        ds.set_default(
            MarkupKind::Highlight,
            MarkupStyle::default().with_stroke(palette::YELLOW),
        );
        ds.set_default(MarkupKind::Line, MarkupStyle::default().with_width(3.0));
        let json = serde_json::to_string(&ds).expect("serialize");
        let back: DefaultStyles = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, ds);
    }
}
