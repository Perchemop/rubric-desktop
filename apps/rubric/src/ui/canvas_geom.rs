//! Coordinate transformations for the canvas: page space <-> screen space.
//!
//! Page space uses PDF points (y-down, origin top-left). Screen space uses
//! egui pixels (y-down, origin top-left of the viewport), transformed by pan
//! and zoom.

use eframe::egui::{Pos2, Vec2};

use crate::domain::geom::Pt;
use crate::domain::Rect as DRect;

/// Map page point to screen space.
pub fn to_screen(page: Pt, pan: Vec2, zoom: f32, origin: Pos2) -> Pos2 {
    origin + pan + Vec2::new(page.x * zoom, page.y * zoom)
}

/// Map screen point to page space.
pub fn to_page(screen: Pos2, pan: Vec2, zoom: f32, origin: Pos2) -> Pt {
    let v = screen - origin - pan;
    let inv = if zoom.abs() < 1e-4 { 1.0 } else { 1.0 / zoom };
    Pt::new(v.x * inv, v.y * inv)
}

/// Map a domain [`Rect`] to screen space.
pub fn screen_rect(r: DRect, origin: Pos2, pan: Vec2, zoom: f32) -> egui::Rect {
    let min = to_screen(Pt::new(r.x, r.y), pan, zoom, origin);
    let max = to_screen(Pt::new(r.right(), r.bottom()), pan, zoom, origin);
    egui::Rect::from_min_max(min, max)
}
