//! Geometry primitives used across the domain.
//!
//! Coordinates are in **PDF points** (1 pt = 1/72 inch) with the origin at the
//! top-left of the page, x to the right and y downward. Keeping a single,
//! explicit coordinate space avoids the classic "which y axis is this?" bug.

use serde::{Deserialize, Serialize};

/// A point in page space (PDF points, y-down).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pt {
    pub x: f32,
    pub y: f32,
}

impl Pt {
    pub const ZERO: Pt = Pt { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(self, other: Pt) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl From<Pt> for [f32; 2] {
    fn from(p: Pt) -> Self {
        [p.x, p.y]
    }
}

/// An axis-aligned rectangle in page space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Build a rectangle from two opposite corners, normalising so that
    /// `w`/`h` are never negative (dragging in any direction works).
    pub fn from_corners(a: Pt, b: Pt) -> Self {
        let x = a.x.min(b.x);
        let y = a.y.min(b.y);
        Self {
            x,
            y,
            w: (a.x - b.x).abs(),
            h: (a.y - b.y).abs(),
        }
    }

    /// Smallest rectangle containing every point, or `None` for an empty slice.
    pub fn from_bounds(points: &[Pt]) -> Option<Rect> {
        let mut iter = points.iter().copied();
        let first = iter.next()?;
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x, first.y, first.x, first.y);
        for p in iter {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
        Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn center(&self) -> Pt {
        Pt::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn area(&self) -> f32 {
        self.w * self.h
    }

    pub fn contains(&self, p: Pt) -> bool {
        p.x >= self.x && p.x <= self.right() && p.y >= self.y && p.y <= self.bottom()
    }

    /// Grow the rectangle outwards by `m` on every side.
    pub fn expanded(&self, m: f32) -> Rect {
        Rect::new(self.x - m, self.y - m, self.w + 2.0 * m, self.h + 2.0 * m)
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::new(x, y, right - x, bottom - y)
    }

    /// Whether two rectangles overlap at all.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x <= other.right()
            && other.x <= self.right()
            && self.y <= other.bottom()
            && other.y <= self.bottom()
    }

    pub fn translated(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.w, self.h)
    }
}

/// Total length of an open polyline through `points`.
pub fn path_length(points: &[Pt]) -> f32 {
    points.windows(2).map(|w| w[0].distance(w[1])).sum()
}

/// Length of a closed loop (polyline plus the closing segment).
pub fn closed_path_length(points: &[Pt]) -> f32 {
    if points.len() < 3 {
        return path_length(points);
    }
    let mut total = path_length(points);
    if let (Some(first), Some(last)) = (points.first(), points.last()) {
        total += first.distance(*last);
    }
    total
}

/// Signed area via the shoelace formula (absolute value returned).
pub fn polygon_area(points: &[Pt]) -> f32 {
    if points.len() < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        sum += a.x * b.y - b.x * a.y;
    }
    (sum / 2.0).abs()
}

/// Angle (degrees, 0..180) at `vertex` formed by rays to `a` and `b`.
pub fn angle_degrees(a: Pt, vertex: Pt, b: Pt) -> f32 {
    let v1 = (a.x - vertex.x, a.y - vertex.y);
    let v2 = (b.x - vertex.x, b.y - vertex.y);
    let dot = v1.0 * v2.0 + v1.1 * v2.1;
    let m1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
    let m2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();
    if m1 == 0.0 || m2 == 0.0 {
        return 0.0;
    }
    let cos = (dot / (m1 * m2)).clamp(-1.0, 1.0);
    cos.acos().to_degrees()
}

/// A full circle inscribed in a square of side `2r` centred on `center`.
pub fn circle_from_center(center: Pt, edge: Pt) -> Rect {
    let r = center.distance(edge);
    Rect::new(center.x - r, center.y - r, 2.0 * r, 2.0 * r)
}

/// Size of a page once its `/Rotate` value is applied for display.
///
/// A quarter turn swaps width and height; half turns and whole turns do not.
/// `quarter_turns` is taken modulo 4, so any value is accepted.
pub fn rotated_page_size(page_w: f32, page_h: f32, quarter_turns: u8) -> (f32, f32) {
    let (w, h) = (page_w.max(0.0), page_h.max(0.0));
    if quarter_turns % 2 == 1 {
        (h, w)
    } else {
        (w, h)
    }
}

/// Map a rectangle from an un-rotated PDF page into the displayed page space.
///
/// A PDF `/Rotate` value turns the page **clockwise** for display: a page whose
/// media box is `w × h` is shown as `h × w` at 90° and 270°. Text extractors
/// report character boxes in the un-rotated user space, so every such box must
/// pass through this mapping before it can be drawn, hit-tested, or compared
/// with a pointer position — otherwise text-anchored markups land offset on
/// rotated pages.
///
/// `quarter_turns` is taken modulo 4, so `0` and `4` are the identity and any
/// value works. Both corners are mapped and the result normalised, so a
/// rectangle never comes back with a negative width.
pub fn rotate_rect_cw(rect: Rect, page_w: f32, page_h: f32, quarter_turns: u8) -> Rect {
    let w = page_w.max(0.0);
    let h = page_h.max(0.0);
    let map = |p: Pt| -> Pt {
        match quarter_turns % 4 {
            // Top-left of the source becomes the top-right of the display.
            1 => Pt::new(h - p.y, p.x),
            2 => Pt::new(w - p.x, h - p.y),
            // Top-left of the source becomes the bottom-left of the display.
            3 => Pt::new(p.y, w - p.x),
            _ => p,
        }
    };
    let a = map(Pt::new(rect.x, rect.y));
    let b = map(Pt::new(rect.right(), rect.bottom()));
    Rect::from_corners(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_corners_normalises_drag_direction() {
        let a = Rect::from_corners(Pt::new(100.0, 100.0), Pt::new(10.0, 20.0));
        assert_eq!(a, Rect::new(10.0, 20.0, 90.0, 80.0));
        let b = Rect::from_corners(Pt::new(10.0, 20.0), Pt::new(100.0, 100.0));
        assert_eq!(a, b, "dragging either direction yields the same rect");
    }

    #[test]
    fn bounds_and_union() {
        let pts = [Pt::new(0.0, 0.0), Pt::new(5.0, 2.0), Pt::new(-1.0, 9.0)];
        let r = Rect::from_bounds(&pts).unwrap();
        assert_eq!(r, Rect::new(-1.0, 0.0, 6.0, 9.0));
        let u = r.union(&Rect::new(10.0, 10.0, 1.0, 1.0));
        assert_eq!(u, Rect::new(-1.0, 0.0, 12.0, 11.0));
    }

    #[test]
    fn path_and_area_math() {
        let square = [
            Pt::new(0.0, 0.0),
            Pt::new(10.0, 0.0),
            Pt::new(10.0, 10.0),
            Pt::new(0.0, 10.0),
        ];
        assert!((path_length(&square) - 30.0).abs() < 1e-4);
        assert!((closed_path_length(&square) - 40.0).abs() < 1e-4);
        assert!((polygon_area(&square) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn right_angle_detected() {
        let a = Pt::new(0.0, 0.0);
        let v = Pt::new(1.0, 0.0);
        let b = Pt::new(1.0, 1.0);
        assert!((angle_degrees(a, v, b) - 90.0).abs() < 1e-3);
    }

    #[test]
    fn rotated_page_size_swaps_width_and_height_on_quarter_turns() {
        assert_eq!(rotated_page_size(100.0, 200.0, 0), (100.0, 200.0));
        assert_eq!(rotated_page_size(100.0, 200.0, 1), (200.0, 100.0));
        assert_eq!(rotated_page_size(100.0, 200.0, 2), (100.0, 200.0));
        assert_eq!(rotated_page_size(100.0, 200.0, 3), (200.0, 100.0));
        // Out-of-range values wrap and negative sizes clamp.
        assert_eq!(rotated_page_size(100.0, 200.0, 5), (200.0, 100.0));
        assert_eq!(rotated_page_size(-5.0, -7.0, 0), (0.0, 0.0));
    }

    #[test]
    fn rotate_rect_cw_zero_and_full_turns_are_the_identity() {
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(rotate_rect_cw(rect, 100.0, 200.0, 0), rect);
        assert_eq!(rotate_rect_cw(rect, 100.0, 200.0, 4), rect);
    }

    #[test]
    fn rotate_rect_cw_quarter_turn_sends_top_left_to_top_right() {
        // Page 100 x 200 becomes 200 x 100; the source top-left corner lands on
        // the display's top-right, which is the clockwise convention.
        let rect = Rect::new(0.0, 0.0, 10.0, 20.0);
        assert_eq!(
            rotate_rect_cw(rect, 100.0, 200.0, 1),
            Rect::new(180.0, 0.0, 20.0, 10.0)
        );
    }

    #[test]
    fn rotate_rect_cw_half_and_three_quarter_turns() {
        let rect = Rect::new(0.0, 0.0, 10.0, 20.0);
        assert_eq!(
            rotate_rect_cw(rect, 100.0, 200.0, 2),
            Rect::new(90.0, 180.0, 10.0, 20.0)
        );
        // Three quarter turns put the source top-left at the bottom-left.
        assert_eq!(
            rotate_rect_cw(rect, 100.0, 200.0, 3),
            Rect::new(0.0, 90.0, 20.0, 10.0)
        );
    }

    #[test]
    fn rotate_rect_cw_normalises_a_rect_that_spans_the_origin() {
        let rotated = rotate_rect_cw(Rect::new(0.0, 0.0, 100.0, 200.0), 100.0, 200.0, 1);
        assert_eq!(rotated, Rect::new(0.0, 0.0, 200.0, 100.0));
    }
}
