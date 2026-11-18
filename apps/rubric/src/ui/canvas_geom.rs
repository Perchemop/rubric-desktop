//! Coordinate transformations for the canvas: page space <-> screen space.
//!
//! Page space uses PDF points (y-down, origin top-left). Screen space uses
//! egui pixels (y-down, origin top-left of the viewport), transformed by pan
//! and zoom.

use eframe::egui::{Pos2, Vec2};

use crate::domain::geom::Pt;
use crate::domain::markup::MarkupGeometry;
use crate::domain::{Markup, Rect as DomainRect};

/// Number of quarter turns represented by a rotation in degrees.
pub fn quarter_turns(rotation: i32) -> u8 {
    (rotation / 90).rem_euclid(4) as u8
}

/// Map page point to screen space.
pub fn to_screen(page: Pt, pan: Vec2, zoom: f32, origin: Pos2) -> Pos2 {
    origin + pan + Vec2::new(page.x * zoom, page.y * zoom)
}

/// Map a page point to screen space, honouring page rotation.
///
/// The point is first rotated clockwise `rotation` degrees about the page
/// origin (using `page_w`/`page_h` for the page box), then scaled/panned.
pub fn to_screen_rotated(
    page: Pt,
    pan: Vec2,
    zoom: f32,
    origin: Pos2,
    page_w: f32,
    page_h: f32,
    rotation: i32,
) -> Pos2 {
    let t = quarter_turns(rotation);
    let mapped = match t {
        1 => Pt::new(page_h - page.y, page.x),
        2 => Pt::new(page_w - page.x, page_h - page.y),
        3 => Pt::new(page.y, page_w - page.x),
        _ => page,
    };
    origin + pan + Vec2::new(mapped.x * zoom, mapped.y * zoom)
}

/// Map screen point to page space.
pub fn to_page(screen: Pos2, pan: Vec2, zoom: f32, origin: Pos2) -> Pt {
    let v = screen - origin - pan;
    let inv = if zoom.abs() < 1e-4 { 1.0 } else { 1.0 / zoom };
    Pt::new(v.x * inv, v.y * inv)
}

/// Map a screen point to page space, honouring page rotation (inverse of
/// [`to_screen_rotated`]).
pub fn to_page_rotated(
    screen: Pos2,
    pan: Vec2,
    zoom: f32,
    origin: Pos2,
    page_w: f32,
    page_h: f32,
    rotation: i32,
) -> Pt {
    let v = screen - origin - pan;
    let inv = if zoom.abs() < 1e-4 { 1.0 } else { 1.0 / zoom };
    let display = Pt::new(v.x * inv, v.y * inv);
    let t = quarter_turns(rotation);
    match t {
        1 => Pt::new(display.y, page_h - display.x),
        2 => Pt::new(page_w - display.x, page_h - display.y),
        3 => Pt::new(page_w - display.y, display.x),
        _ => display,
    }
}

/// Map a domain [`Rect`] to screen space.
pub fn screen_rect(r: DomainRect, origin: Pos2, pan: Vec2, zoom: f32) -> egui::Rect {
    let min = to_screen(Pt::new(r.x, r.y), pan, zoom, origin);
    let max = to_screen(Pt::new(r.right(), r.bottom()), pan, zoom, origin);
    egui::Rect::from_min_max(min, max)
}

/// Map a domain [`Rect`] to screen space, honouring rotation (uses the rect's
/// own dimensions for the rotation box).
pub fn screen_rect_rotated(
    r: DomainRect,
    origin: Pos2,
    pan: Vec2,
    zoom: f32,
    rotation: i32,
) -> egui::Rect {
    // Build a rect in display space by rotating the corners, then normalise:
    // a 90° turn swaps the corner order, so min/max must be re-sorted or the
    // rect comes back with a negative width.
    let min = to_screen_rotated(Pt::new(r.x, r.y), pan, zoom, origin, r.w, r.h, rotation);
    let max = to_screen_rotated(
        Pt::new(r.right(), r.bottom()),
        pan,
        zoom,
        origin,
        r.w,
        r.h,
        rotation,
    );
    egui::Rect::from_min_max(min.min(max), min.max(max))
}

/// Map a domain [`Rect`] to screen space, honouring page rotation using the
/// page's own dimensions as the rotation box (not the rect's). Use this for
/// per-markup bounds under a rotated page.
pub fn screen_rect_page_rotated(
    r: DomainRect,
    origin: Pos2,
    pan: Vec2,
    zoom: f32,
    page_w: f32,
    page_h: f32,
    rotation: i32,
) -> egui::Rect {
    let min = to_screen_rotated(
        Pt::new(r.x, r.y),
        pan,
        zoom,
        origin,
        page_w,
        page_h,
        rotation,
    );
    let max = to_screen_rotated(
        Pt::new(r.right(), r.bottom()),
        pan,
        zoom,
        origin,
        page_w,
        page_h,
        rotation,
    );
    // Normalise: a 90° turn swaps the corner order (negative width otherwise).
    egui::Rect::from_min_max(min.min(max), min.max(max))
}

/// Hit tolerance expressed in page units for a given zoom.
pub(crate) fn hit_tolerance_page(zoom: f32) -> f32 {
    let z = zoom.max(0.1);
    (6.0 / z).clamp(0.5, 30.0)
}

/// Anchor point in page space for a measurement label.
pub(crate) fn measure_anchor(m: &Markup) -> Pt {
    match &m.geometry {
        MarkupGeometry::Segment { a, b } => Pt::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5),
        MarkupGeometry::Path { points, .. } => {
            if points.is_empty() {
                m.bounds().center()
            } else {
                let n = points.len() as f32;
                let (sx, sy) = points
                    .iter()
                    .fold((0.0f32, 0.0f32), |(x, y), p| (x + p.x, y + p.y));
                Pt::new(sx / n, sy / n)
            }
        }
        MarkupGeometry::Angle { vertex, .. } => *vertex,
        MarkupGeometry::Count { at, .. } => *at,
        _ => m.bounds().center(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Pos2;

    #[test]
    fn to_page_to_screen_roundtrip_at_every_rotation() {
        let origin = Pos2::new(0.0, 0.0);
        let pan = Vec2::new(0.0, 0.0);
        let zoom = 1.0;
        let pw = 595.0_f32;
        let ph = 842.0_f32;
        let pts = [
            Pt::new(0.0, 0.0),
            Pt::new(pw, ph),
            Pt::new(pw * 0.5, ph * 0.5),
            Pt::new(10.0, 20.0),
        ];
        for turns in 0..4 {
            let rot = turns * 90;
            for p in pts {
                let s = to_screen_rotated(p, pan, zoom, origin, pw, ph, rot);
                let back = to_page_rotated(s, pan, zoom, origin, pw, ph, rot);
                assert!(
                    (back.x - p.x).abs() < 1e-3 && (back.y - p.y).abs() < 1e-3,
                    "roundtrip failed rot={rot} p={p:?} back={back:?}"
                );
            }
        }
    }

    #[test]
    fn screen_rect_uses_rotated_size() {
        let origin = Pos2::new(0.0, 0.0);
        let pan = Vec2::new(0.0, 0.0);
        let zoom = 1.0;
        let r = DomainRect::new(0.0, 0.0, 595.0, 842.0);
        let sr = screen_rect_rotated(r, origin, pan, zoom, 90);
        assert!((sr.width() - 842.0).abs() < 1e-3);
        assert!((sr.height() - 595.0).abs() < 1e-3);
    }

    #[test]
    fn to_page_rotated_is_the_exact_inverse_of_to_screen_rotated() {
        let origin = Pos2::new(3.0, 7.0);
        let pan = Vec2::new(2.0, 5.0);
        let zoom = 2.0;
        let pw = 595.0_f32;
        let ph = 842.0_f32;
        // Sample 10 points: corners, edges, interior.
        let pts = [
            Pt::new(0.0, 0.0),
            Pt::new(pw, 0.0),
            Pt::new(0.0, ph),
            Pt::new(pw, ph),
            Pt::new(pw * 0.5, 0.0),
            Pt::new(0.0, ph * 0.5),
            Pt::new(pw, ph * 0.5),
            Pt::new(pw * 0.5, ph),
            Pt::new(pw * 0.25, ph * 0.75),
            Pt::new(pw * 0.8, ph * 0.3),
        ];
        for turns in 1..=3 {
            let rot = turns * 90;
            for p in pts {
                let s = to_screen_rotated(p, pan, zoom, origin, pw, ph, rot);
                let back = to_page_rotated(s, pan, zoom, origin, pw, ph, rot);
                assert!(
                    (back.x - p.x).abs() < 1e-3 && (back.y - p.y).abs() < 1e-3,
                    "inverse failed rot={rot} p={p:?} back={back:?}"
                );
            }
        }
    }
}
