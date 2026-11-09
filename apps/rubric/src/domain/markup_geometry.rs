//! Geometry of a markup, in page points.
//!
//! This module owns the [`MarkupGeometry`] enum and the pure geometry helpers
//! used for bounds, hit testing, and text layout. It does not depend on
//! [`crate::domain::Markup`], so there is no circular dependency: `markup.rs`
//! imports from here, not the other way around.

use serde::{Deserialize, Serialize};

use super::geom::{self, Pt, Rect};

/// Geometry of a markup, in page points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkupGeometry {
    /// Rectangular/elliptical/cloud family; the shape kind decides rendering.
    Bounds(Rect),
    /// A single straight segment (line, arrow, length/diameter/radius measure).
    Segment { a: Pt, b: Pt },
    /// An open or closed path (polyline, polygon, perimeter, area).
    Path { points: Vec<Pt>, closed: bool },
    /// Freehand pen strokes.
    ///
    /// The model holds any number of strokes in one markup, but the canvas
    /// currently commits exactly one stroke per drag gesture; a multi-stroke
    /// pen tool would append to `strokes` rather than produce a new markup.
    /// Importers and other producers may supply more than one.
    Ink { strokes: Vec<Vec<Pt>> },
    /// A text box anchored at `at`, wrapping within `wrap` points.
    Text { at: Pt, wrap: f32, text: String },
    /// A callout: text box plus a tail pointing at something.
    Callout { rect: Rect, tail: Pt, text: String },
    /// Text-anchored markup (highlight/strikeout/underline/squiggly) anchored
    /// to a specific text line. `at` is the text baseline start point, `text`
    /// is the marked-up text, `line_height` is the per-line height in points,
    /// `line_count` is the number of text lines covered, and `text_width` is
    /// the text width in page points.
    TextAnchor {
        at: Pt,
        text: String,
        line_height: f32,
        line_count: usize,
        text_width: f32,
    },
    /// A three-point angle measurement (vertex is the middle point).
    Angle { a: Pt, vertex: Pt, b: Pt },
    /// A count marker (a numbered dot).
    Count { at: Pt, number: u32 },
}

impl MarkupGeometry {
    /// Tight bounding box in page space.
    pub fn bounds(&self) -> Rect {
        match self {
            MarkupGeometry::Bounds(r) => *r,
            MarkupGeometry::Segment { a, b } => Rect::from_corners(*a, *b),
            MarkupGeometry::Path { points, .. } => {
                Rect::from_bounds(points).unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
            }
            MarkupGeometry::Ink { strokes } => {
                let all: Vec<Pt> = strokes.iter().flatten().copied().collect();
                Rect::from_bounds(&all).unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
            }
            MarkupGeometry::Text { at, wrap, text } => text_box_rect(
                *at,
                *wrap,
                text,
                super::style::MarkupStyle::default().font_size,
            ),
            MarkupGeometry::TextAnchor {
                at,
                line_height,
                line_count,
                text_width,
                ..
            } => {
                // The renderer draws from `at` downward for
                // `line_height * line_count` points and `text_width` to the
                // right, so the bounding box covers exactly that area (and
                // hit-testing / page-crop checks agree with what is drawn).
                Rect::new(
                    at.x,
                    at.y,
                    text_width.max(1.0),
                    (line_height * (*line_count).max(1) as f32).max(1.0),
                )
            }
            MarkupGeometry::Callout { rect, tail, .. } => {
                rect.union(&Rect::new(tail.x, tail.y, 0.0, 0.0))
            }
            MarkupGeometry::Angle { a, vertex, b } => {
                Rect::from_bounds(&[*a, *vertex, *b]).unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
            }
            MarkupGeometry::Count { at, .. } => Rect::new(at.x - 8.0, at.y - 8.0, 16.0, 16.0),
        }
    }

    /// Move the geometry by a delta.
    pub fn translate(&mut self, dx: f32, dy: f32) {
        let t = |p: &mut Pt| {
            p.x += dx;
            p.y += dy;
        };
        match self {
            MarkupGeometry::Bounds(r) => {
                r.x += dx;
                r.y += dy;
            }
            MarkupGeometry::Segment { a, b } => {
                t(a);
                t(b);
            }
            MarkupGeometry::Path { points, .. } => points.iter_mut().for_each(t),
            MarkupGeometry::Ink { strokes } => {
                strokes.iter_mut().flatten().for_each(t);
            }
            MarkupGeometry::Text { at, .. } => t(at),
            MarkupGeometry::TextAnchor { at, .. } => t(at),
            MarkupGeometry::Callout { rect, tail, .. } => {
                rect.x += dx;
                rect.y += dy;
                t(tail);
            }
            MarkupGeometry::Angle { a, vertex, b } => {
                t(a);
                t(vertex);
                t(b);
            }
            MarkupGeometry::Count { at, .. } => t(at),
        }
    }
}

/// Estimated height of one rendered text line, in page points.
///
/// The renderer lays text out at roughly 1.35 × the font size; bounds and hit
/// testing must use the same factor so a markup is hittable everywhere it is
/// drawn.
const TEXT_LINE_HEIGHT_FACTOR: f32 = 1.35;

/// Line height for `font_size`, clamped to the style's 6 pt minimum so a
/// degenerate deserialised font size can never collapse the box.
///
/// Public so the canvas text editor sizes its overlay with the exact same
/// metrics the domain uses for bounds and hit testing.
pub fn text_line_height(font_size: f32) -> f32 {
    TEXT_LINE_HEIGHT_FACTOR * font_size.max(6.0)
}

/// The on-page rectangle of a text box anchored at `at`, wrapping within
/// `wrap` points: one [`text_line_height`] per line, with the wrap width
/// clamped to at least 1 pt so a degenerate stored wrap cannot make the box
/// unhittable.
pub fn text_box_rect(at: Pt, wrap: f32, text: &str, font_size: f32) -> Rect {
    let lines = text.lines().count().max(1) as f32;
    Rect::new(
        at.x,
        at.y,
        wrap.max(1.0),
        lines * text_line_height(font_size),
    )
}

/// Perpendicular distance from `p` to the segment `a`-`b`.
pub(crate) fn distance_to_segment(p: Pt, a: Pt, b: Pt) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq <= f32::EPSILON {
        return p.distance(a);
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / len_sq).clamp(0.0, 1.0);
    let proj = Pt::new(a.x + t * dx, a.y + t * dy);
    p.distance(proj)
}

/// Whether `p` lies inside the axis-aligned ellipse inscribed in `r`.
fn ellipse_contains(r: Rect, p: Pt) -> bool {
    let a = r.w / 2.0;
    let b = r.h / 2.0;
    if a <= 0.0 || b <= 0.0 {
        return false;
    }
    let c = r.center();
    let nx = (p.x - c.x) / a;
    let ny = (p.y - c.y) / b;
    nx * nx + ny * ny <= 1.0
}

/// Minimum distance from `p` to the four edges of `r` (its border).
fn distance_to_rect_border(r: Rect, p: Pt) -> f32 {
    let tl = Pt::new(r.x, r.y);
    let tr = Pt::new(r.right(), r.y);
    let br = Pt::new(r.right(), r.bottom());
    let bl = Pt::new(r.x, r.bottom());
    distance_to_segment(p, tl, tr)
        .min(distance_to_segment(p, tr, br))
        .min(distance_to_segment(p, br, bl))
        .min(distance_to_segment(p, bl, tl))
}

/// Even-odd (ray casting) point-in-polygon test.
///
/// Boundary results are implementation-defined, so callers should also accept
/// a near-edge hit (see [`polyline_hit`]).
fn point_in_polygon(points: &[Pt], p: Pt) -> bool {
    if points.len() < 3 {
        return false;
    }
    let n = points.len();
    let mut inside = false;
    for (i, a) in points.iter().enumerate() {
        let b = points[(i + n - 1) % n];
        if (a.y > p.y) != (b.y > p.y) {
            let x = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if p.x < x {
                inside = !inside;
            }
        }
    }
    inside
}

/// Whether `p` is within `tol` of any segment of a polyline.
///
/// A closed path also tests the closing segment; a single-point stroke acts as
/// a dot. Used by ink, polyline and closed path hit testing.
fn polyline_hit(p: Pt, points: &[Pt], closed: bool, tol: f32) -> bool {
    match points.len() {
        0 => false,
        1 => p.distance(points[0]) <= tol,
        n => {
            points
                .windows(2)
                .any(|w| distance_to_segment(p, w[0], w[1]) <= tol)
                || (closed && distance_to_segment(p, points[n - 1], points[0]) <= tol)
        }
    }

    pub use self::text_box_rect;
    pub use self::text_line_height;
    pub use self::MarkupGeometry;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn bounds_segment_is_corner_to_corner() {
            let g = MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(100.0, 50.0),
            };
            assert_eq!(g.bounds(), Rect::new(0.0, 0.0, 100.0, 50.0));
        }

        #[test]
        fn bounds_path_is_tight() {
            let g = MarkupGeometry::Path {
                points: vec![
                    Pt::new(10.0, 10.0),
                    Pt::new(30.0, 12.0),
                    Pt::new(20.0, 40.0),
                ],
                closed: true,
            };
            assert_eq!(g.bounds(), Rect::new(10.0, 10.0, 20.0, 30.0));
        }

        #[test]
        fn bounds_text_anchor_covers_drawn_area() {
            let g = MarkupGeometry::TextAnchor {
                at: Pt::new(20.0, 30.0),
                text: "Hello".to_string(),
                line_height: 14.0,
                line_count: 1,
                text_width: 100.0,
            };
            assert_eq!(g.bounds(), Rect::new(20.0, 30.0, 100.0, 14.0));
        }

        #[test]
        fn bounds_count_is_sixteen_point_dot() {
            let g = MarkupGeometry::Count {
                at: Pt::new(10.0, 10.0),
                number: 1,
            };
            assert_eq!(g.bounds(), Rect::new(2.0, 2.0, 16.0, 16.0));
        }

        #[test]
        fn translate_moves_all_geometry_variants() {
            let mut g = MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 10.0, 10.0));
            g.translate(5.0, -3.0);
            assert_eq!(g.bounds(), Rect::new(5.0, -3.0, 10.0, 10.0));

            let mut g = MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(10.0, 10.0),
            };
            g.translate(1.0, 2.0);
            match g {
                MarkupGeometry::Segment { a, b } => {
                    assert_eq!(a, Pt::new(1.0, 2.0));
                    assert_eq!(b, Pt::new(11.0, 12.0));
                }
                _ => unreachable!(),
            }
        }

        #[test]
        fn text_box_rect_clamps_wrap_to_at_least_one_pt() {
            let r = text_box_rect(Pt::new(0.0, 0.0), 0.0, "hi", 12.0);
            assert_eq!(r.w, 1.0);
        }

        #[test]
        fn text_line_height_clamps_to_six_pt_minimum() {
            assert!((text_line_height(0.0) - 1.35 * 6.0).abs() < f32::EPSILON);
            assert!((text_line_height(12.0) - 1.35 * 12.0).abs() < f32::EPSILON);
        }

        #[test]
        fn distance_to_segment_handles_zero_length() {
            assert!(
                (distance_to_segment(Pt::new(3.0, 4.0), Pt::new(0.0, 0.0), Pt::new(0.0, 0.0))
                    - 5.0)
                    .abs()
                    < f32::EPSILON
            );
        }

        #[test]
        fn ellipse_contains_center_and_edge() {
            let r = Rect::new(0.0, 0.0, 100.0, 100.0);
            assert!(ellipse_contains(r, Pt::new(50.0, 50.0)));
            assert!(!ellipse_contains(r, Pt::new(0.0, 0.0)));
        }

        #[test]
        fn point_in_polygon_square() {
            let points = vec![
                Pt::new(0.0, 0.0),
                Pt::new(100.0, 0.0),
                Pt::new(100.0, 100.0),
                Pt::new(0.0, 100.0),
            ];
            assert!(point_in_polygon(&points, Pt::new(50.0, 50.0)));
            assert!(!point_in_polygon(&points, Pt::new(150.0, 50.0)));
        }

        #[test]
        fn polyline_hit_single_point_acts_as_dot() {
            assert!(polyline_hit(
                Pt::new(11.0, 10.0),
                &[Pt::new(10.0, 10.0)],
                false,
                2.0
            ));
            assert!(!polyline_hit(
                Pt::new(20.0, 10.0),
                &[Pt::new(10.0, 10.0)],
                false,
                2.0
            ));
        }

        #[test]
        fn markup_geometry_serde_roundtrip() {
            let geometries = [
                MarkupGeometry::Ink {
                    strokes: vec![vec![Pt::new(0.0, 0.0), Pt::new(5.0, 5.0)]],
                },
                MarkupGeometry::Path {
                    points: vec![Pt::new(0.0, 0.0), Pt::new(5.0, 0.0), Pt::new(5.0, 5.0)],
                    closed: true,
                },
                MarkupGeometry::Bounds(Rect::new(1.0, 2.0, 3.0, 4.0)),
            ];
            for geom in geometries {
                let json = serde_json::to_string(&geom).expect("serialize");
                let back: MarkupGeometry = serde_json::from_str(&json).expect("deserialize");
                assert_eq!(geom, back);
            }
        }
}
