//! UI theme and styling helpers (egui).

use egui::{Color32, Stroke};

use crate::domain::style::Color;

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

pub fn apply_theme(_ctx: &egui::Context) {
    // For now, use default egui light theme (polished enough for Phase 0).
    // We'll tune palette/rounding later.
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
