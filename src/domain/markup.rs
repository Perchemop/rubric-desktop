//! Markup entities: what a user draws on a page.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::geom::{self, Pt, Rect};
use super::style::{Color, FillMode, MarkupStyle, palette};
use super::units::{self, MeasureKind, Scale};

/// The tool / shape family of a markup.
///
/// This is deliberately a flat enum rather than a trait object tree: it is
/// trivially serialisable, exhaustively matchable, and cheap to clone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkupKind {
    // Text markups
    Highlight,
    Strikeout,
    Underline,
    Squiggly,
    Text,
    Callout,
    Note,
    Stamp,
    // Shape markups
    Rectangle,
    Ellipse,
    Cloud,
    Polygon,
    Polyline,
    // Path markups
    Line,
    Arrow,
    Ink,
    // Measurements
    MeasureLength,
    MeasurePerimeter,
    MeasureArea,
    MeasureAngle,
    MeasureDiameter,
    MeasureRadius,
    MeasureCount,
}

/// Broad grouping used to lay out the tool palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolGroup {
    Review,
    Shapes,
    Paths,
    Measure,
}

impl MarkupKind {
    pub const ALL: [MarkupKind; 23] = [
        MarkupKind::Highlight,
        MarkupKind::Strikeout,
        MarkupKind::Underline,
        MarkupKind::Squiggly,
        MarkupKind::Text,
        MarkupKind::Callout,
        MarkupKind::Note,
        MarkupKind::Stamp,
        MarkupKind::Rectangle,
        MarkupKind::Ellipse,
        MarkupKind::Cloud,
        MarkupKind::Polygon,
        MarkupKind::Polyline,
        MarkupKind::Line,
        MarkupKind::Arrow,
        MarkupKind::Ink,
        MarkupKind::MeasureLength,
        MarkupKind::MeasurePerimeter,
        MarkupKind::MeasureArea,
        MarkupKind::MeasureAngle,
        MarkupKind::MeasureDiameter,
        MarkupKind::MeasureRadius,
        MarkupKind::MeasureCount,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MarkupKind::Highlight => "Highlight",
            MarkupKind::Strikeout => "Strikeout",
            MarkupKind::Underline => "Underline",
            MarkupKind::Squiggly => "Squiggly",
            MarkupKind::Text => "Text Box",
            MarkupKind::Callout => "Callout",
            MarkupKind::Note => "Sticky Note",
            MarkupKind::Stamp => "Stamp",
            MarkupKind::Rectangle => "Rectangle",
            MarkupKind::Ellipse => "Ellipse",
            MarkupKind::Cloud => "Cloud",
            MarkupKind::Polygon => "Polygon",
            MarkupKind::Polyline => "Polyline",
            MarkupKind::Line => "Line",
            MarkupKind::Arrow => "Arrow",
            MarkupKind::Ink => "Pen",
            MarkupKind::MeasureLength => "Length",
            MarkupKind::MeasurePerimeter => "Perimeter",
            MarkupKind::MeasureArea => "Area",
            MarkupKind::MeasureAngle => "Angle",
            MarkupKind::MeasureDiameter => "Diameter",
            MarkupKind::MeasureRadius => "Radius",
            MarkupKind::MeasureCount => "Count",
        }
    }

    /// Short glyph shown on the tool button (keeps the UI font-independent).
    pub fn glyph(self) -> &'static str {
        match self {
            MarkupKind::Highlight => "▀",
            MarkupKind::Strikeout => "∅",
            MarkupKind::Underline => "▁",
            MarkupKind::Squiggly => "≈",
            MarkupKind::Text => "T",
            MarkupKind::Callout => "◨",
            MarkupKind::Note => "▤",
            MarkupKind::Stamp => "★",
            MarkupKind::Rectangle => "▭",
            MarkupKind::Ellipse => "◯",
            MarkupKind::Cloud => "☁",
            MarkupKind::Polygon => "⬠",
            MarkupKind::Polyline => "∿",
            MarkupKind::Line => "╱",
            MarkupKind::Arrow => "➤",
            MarkupKind::Ink => "✎",
            MarkupKind::MeasureLength => "↔",
            MarkupKind::MeasurePerimeter => "▢",
            MarkupKind::MeasureArea => "▣",
            MarkupKind::MeasureAngle => "∠",
            MarkupKind::MeasureDiameter => "⌀",
            MarkupKind::MeasureRadius => "⟶",
            MarkupKind::MeasureCount => "#",
        }
    }

    pub fn group(self) -> ToolGroup {
        match self {
            MarkupKind::Highlight
            | MarkupKind::Strikeout
            | MarkupKind::Underline
            | MarkupKind::Squiggly
            | MarkupKind::Text
            | MarkupKind::Callout
            | MarkupKind::Note
            | MarkupKind::Stamp => ToolGroup::Review,
            MarkupKind::Rectangle | MarkupKind::Ellipse | MarkupKind::Cloud | MarkupKind::Polygon => {
                ToolGroup::Shapes
            }
            MarkupKind::Polyline | MarkupKind::Line | MarkupKind::Arrow | MarkupKind::Ink => {
                ToolGroup::Paths
            }
            MarkupKind::MeasureLength
            | MarkupKind::MeasurePerimeter
            | MarkupKind::MeasureArea
            | MarkupKind::MeasureAngle
            | MarkupKind::MeasureDiameter
            | MarkupKind::MeasureRadius
            | MarkupKind::MeasureCount => ToolGroup::Measure,
        }
    }

    pub fn is_measure(self) -> bool {
        self.group() == ToolGroup::Measure
    }

    /// The measurement quantity this tool produces, if any.
    pub fn measure_kind(self) -> Option<MeasureKind> {
        Some(match self {
            MarkupKind::MeasureLength => MeasureKind::Length,
            MarkupKind::MeasurePerimeter => MeasureKind::Perimeter,
            MarkupKind::MeasureArea => MeasureKind::Area,
            MarkupKind::MeasureAngle => MeasureKind::Angle,
            MarkupKind::MeasureDiameter => MeasureKind::Diameter,
            MarkupKind::MeasureRadius => MeasureKind::Radius,
            MarkupKind::MeasureCount => MeasureKind::Count,
            _ => return None,
        })
    }

    /// Default review colour per tool family.
    pub fn default_style(self) -> MarkupStyle {
        let base = MarkupStyle::default();
        match self {
            MarkupKind::Highlight => base
                .with_stroke(palette::YELLOW)
                .with_width(14.0)
                .with_fill_mode(FillMode::Highlight),
            MarkupKind::Strikeout | MarkupKind::Underline | MarkupKind::Squiggly => {
                base.with_stroke(palette::RED).with_width(2.0)
            }
            MarkupKind::Text | MarkupKind::Note | MarkupKind::Callout => {
                base.with_stroke(palette::BLACK)
            }
            MarkupKind::Stamp => base.with_stroke(palette::GREEN).with_width(3.0),
            MarkupKind::Cloud => base
                .with_stroke(palette::BLUE)
                .with_width(2.0)
                .with_fill_mode(FillMode::None),
            MarkupKind::Ink => base.with_stroke(palette::RED).with_width(2.5),
            MarkupKind::MeasureLength
            | MarkupKind::MeasurePerimeter
            | MarkupKind::MeasureArea
            | MarkupKind::MeasureAngle
            | MarkupKind::MeasureDiameter
            | MarkupKind::MeasureRadius => base.with_stroke(palette::TEAL).with_width(1.5),
            _ => base,
        }
    }
}

impl MarkupStyle {
    fn with_fill_mode(mut self, m: FillMode) -> Self {
        self.fill_mode = m;
        self
    }
}

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
    Ink { strokes: Vec<Vec<Pt>> },
    /// A text box anchored at `at`, wrapping within `wrap` points.
    Text { at: Pt, wrap: f32, text: String },
    /// A callout: text box plus a tail pointing at something.
    Callout { rect: Rect, tail: Pt, text: String },
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
            MarkupGeometry::Text { at, wrap, text } => {
                let lines = (text.lines().count().max(1)) as f32;
                Rect::new(at.x, at.y, *wrap, lines * 16.0)
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

/// Review workflow status, mirroring a standard reviewer's status set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    #[default]
    None,
    Accepted,
    Rejected,
    Completed,
    Cancelled,
    Alert,
}

impl ReviewStatus {
    pub const ALL: [ReviewStatus; 6] = [
        ReviewStatus::None,
        ReviewStatus::Accepted,
        ReviewStatus::Rejected,
        ReviewStatus::Completed,
        ReviewStatus::Cancelled,
        ReviewStatus::Alert,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ReviewStatus::None => "None",
            ReviewStatus::Accepted => "Accepted",
            ReviewStatus::Rejected => "Rejected",
            ReviewStatus::Completed => "Completed",
            ReviewStatus::Cancelled => "Cancelled",
            ReviewStatus::Alert => "Alert",
        }
    }

    pub fn color(self) -> Color {
        match self {
            ReviewStatus::None => palette::GREY,
            ReviewStatus::Accepted => palette::GREEN,
            ReviewStatus::Rejected => palette::RED,
            ReviewStatus::Completed => palette::BLUE,
            ReviewStatus::Cancelled => palette::GREY,
            ReviewStatus::Alert => palette::AMBER,
        }
    }
}

/// A threaded comment attached to a markup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub author: String,
    pub text: String,
    pub created: DateTime<Utc>,
}

impl Comment {
    pub fn new(author: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            author: author.into(),
            text: text.into(),
            created: Utc::now(),
        }
    }
}

/// A measurement result attached to a measure markup.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub kind: MeasureKind,
    /// Value in the page's scale units.
    pub value: f64,
    /// Value in raw PDF points (kept for loss-free re-scaling).
    pub value_points: f64,
}

impl Measurement {
    pub fn label(&self, unit: units::MeasureUnit) -> String {
        units::format_value(self.kind, unit, self.value)
    }
}

/// A single markup on a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Markup {
    pub id: Uuid,
    /// Index of the page this markup lives on.
    pub page: usize,
    pub kind: MarkupKind,
    pub geometry: MarkupGeometry,
    pub style: MarkupStyle,
    pub author: String,
    pub subject: String,
    #[serde(default)]
    pub status: ReviewStatus,
    /// Optional group/layer name (used for tool sets and layers).
    #[serde(default)]
    pub layer: Option<String>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub measurement: Option<Measurement>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

impl Markup {
    pub fn new(page: usize, kind: MarkupKind, geometry: MarkupGeometry, author: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            page,
            kind,
            geometry,
            style: kind.default_style(),
            author: author.into(),
            subject: kind.label().to_string(),
            status: ReviewStatus::None,
            layer: None,
            locked: false,
            comments: Vec::new(),
            measurement: None,
            created: now,
            modified: now,
        }
    }

    pub fn with_style(mut self, style: MarkupStyle) -> Self {
        self.style = style;
        self.style.font_size = self.style.font_size.max(6.0);
        self
    }

    pub fn bounds(&self) -> Rect {
        self.geometry.bounds()
    }

    /// Hit test with a tolerance in page points.
    pub fn hit_test(&self, p: Pt, tolerance: f32) -> bool {
        match &self.geometry {
            MarkupGeometry::Bounds(r) => {
                if self.kind == MarkupKind::Highlight || self.style.fill_mode != FillMode::None {
                    r.expanded(tolerance).contains(p)
                } else {
                    // Outline hit test: near the border, or inside for text-ish markups.
                    let outer = r.expanded(tolerance);
                    let inner = Rect::new(
                        r.x + tolerance,
                        r.y + tolerance,
                        (r.w - 2.0 * tolerance).max(0.0),
                        (r.h - 2.0 * tolerance).max(0.0),
                    );
                    outer.contains(p) && !inner.contains(p)
                }
            }
            MarkupGeometry::Segment { a, b } => distance_to_segment(p, *a, *b) <= tolerance.max(3.0),
            MarkupGeometry::Path { points, closed } => {
                let mut pts = points.clone();
                if *closed {
                    if let Some(first) = points.first() {
                        pts.push(*first);
                    }
                }
                pts.windows(2)
                    .any(|w| distance_to_segment(p, w[0], w[1]) <= tolerance.max(3.0))
            }
            MarkupGeometry::Ink { strokes } => strokes
                .iter()
                .any(|s| s.windows(2).any(|w| distance_to_segment(p, w[0], w[1]) <= tolerance.max(3.0))),
            MarkupGeometry::Text { at, wrap, text } => {
                let h = (text.lines().count().max(1) as f32) * 16.0;
                Rect::new(at.x, at.y, *wrap, h).expanded(tolerance).contains(p)
            }
            MarkupGeometry::Callout { rect, tail, .. } => {
                rect.expanded(tolerance).contains(p) || p.distance(*tail) <= tolerance.max(4.0)
            }
            MarkupGeometry::Angle { a, vertex, b } => {
                p.distance(*vertex) <= tolerance.max(4.0)
                    || distance_to_segment(p, *vertex, *a) <= tolerance.max(3.0)
                    || distance_to_segment(p, *vertex, *b) <= tolerance.max(3.0)
            }
            MarkupGeometry::Count { at, .. } => p.distance(*at) <= tolerance.max(10.0),
        }
    }

    /// Recompute the measurement (if this is a measure markup) using `scale`.
    pub fn recompute_measurement(&mut self, points: Scale) {
        let Some(kind) = self.kind.measure_kind() else {
            self.measurement = None;
            return;
        };
        let raw = match (&self.geometry, kind) {
            (MarkupGeometry::Segment { a, b }, MeasureKind::Length)
            | (MarkupGeometry::Segment { a, b }, MeasureKind::Diameter)
            | (MarkupGeometry::Segment { a, b }, MeasureKind::Radius) => a.distance(*b) as f64,
            (MarkupGeometry::Path { points, .. }, MeasureKind::Perimeter) => {
                geom::closed_path_length(points) as f64
            }
            (MarkupGeometry::Path { points, .. }, MeasureKind::Area) => {
                geom::polygon_area(points) as f64
            }
            (MarkupGeometry::Angle { a, vertex, b }, MeasureKind::Angle) => {
                geom::angle_degrees(*a, *vertex, *b) as f64
            }
            (MarkupGeometry::Count { number, .. }, MeasureKind::Count) => *number as f64,
            _ => 0.0,
        };
        // Radius is half the segment length; diameter is the full length.
        let raw = match kind {
            MeasureKind::Radius => raw / 2.0,
            _ => raw,
        };
        let value = match kind {
            MeasureKind::Angle | MeasureKind::Count => raw,
            MeasureKind::Area => raw * ((points.units_per_point as f64).powi(2)),
            _ => points.points_to_units(raw),
        };
        self.measurement = Some(Measurement {
            kind,
            value,
            value_points: raw,
        });
    }

    pub fn touch(&mut self) {
        self.modified = Utc::now();
    }

    /// Plain-text summary line used in lists and exports.
    pub fn summary(&self) -> String {
        let mut s = self.subject.clone();
        if let Some(m) = &self.measurement {
            s.push_str(": ");
            s.push_str(&units::format_value(
                m.kind,
                units::MeasureUnit::Point,
                m.value_points,
            ));
        }
        s
    }

    /// A short colour/kind-free description used by tests and logs.
    pub fn describe(&self) -> String {
        format!("{:?} on page {} by {}", self.kind, self.page, self.author)
    }
}

impl MarkupStyle {
    pub fn dash_color(&self) -> Color {
        self.stroke
    }

    pub fn dash_segments(&self) -> Option<[f32; 2]> {
        self.dash.segments(self.width)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn scale() -> Scale {
        Scale::new(units::MeasureUnit::Foot, 1.0 / 72.0) // 1 in = 1 ft
    }

    #[test]
    fn length_measurement_uses_scale() {
        let mut m = Markup::new(
            0,
            MarkupKind::MeasureLength,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(720.0, 0.0),
            },
            "tester",
        );
        m.recompute_measurement(scale());
        let meas = m.measurement.unwrap();
        assert!((meas.value - 10.0).abs() < 1e-6, "720 pt = 10 ft, got {}", meas.value);
    }

    #[test]
    fn area_measurement_is_squared() {
        let mut m = Markup::new(
            0,
            MarkupKind::MeasureArea,
            MarkupGeometry::Path {
                points: vec![
                    Pt::new(0.0, 0.0),
                    Pt::new(72.0, 0.0),
                    Pt::new(72.0, 72.0),
                    Pt::new(0.0, 72.0),
                ],
                closed: true,
            },
            "tester",
        );
        m.recompute_measurement(scale());
        // 1 in x 1 in = 1 ft x 1 ft = 1 ft².
        assert!((m.measurement.unwrap().value - 1.0).abs() < 1e-6);
    }

    #[test]
    fn radius_is_half_diameter() {
        let geom = MarkupGeometry::Segment {
            a: Pt::new(0.0, 0.0),
            b: Pt::new(100.0, 0.0),
        };
        let mut d = Markup::new(0, MarkupKind::MeasureDiameter, geom.clone(), "t");
        let mut r = Markup::new(0, MarkupKind::MeasureRadius, geom, "t");
        d.recompute_measurement(Scale::PAPER);
        r.recompute_measurement(Scale::PAPER);
        assert!((d.measurement.unwrap().value - 100.0).abs() < 1e-6);
        assert!((r.measurement.unwrap().value - 50.0).abs() < 1e-6);
    }

    #[test]
    fn hit_testing_segment_and_bounds() {
        let seg = Markup::new(
            0,
            MarkupKind::Line,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(100.0, 0.0),
            },
            "t",
        );
        assert!(seg.hit_test(Pt::new(50.0, 1.0), 2.0));
        assert!(!seg.hit_test(Pt::new(50.0, 40.0), 2.0));

        let rect = Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 100.0, 100.0)),
            "t",
        );
        assert!(rect.hit_test(Pt::new(0.0, 50.0), 2.0), "on the left edge");
        assert!(!rect.hit_test(Pt::new(50.0, 50.0), 2.0), "centre of empty rect");

        let highlight = Markup::new(
            0,
            MarkupKind::Highlight,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 100.0, 20.0)),
            "t",
        );
        assert!(highlight.hit_test(Pt::new(50.0, 10.0), 2.0), "fills are hit inside");
    }

    #[test]
    fn translate_moves_all_geometry_variants() {
        let mut m = Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 10.0, 10.0)),
            "t",
        );
        m.geometry.translate(5.0, -3.0);
        assert_eq!(m.bounds(), Rect::new(5.0, -3.0, 10.0, 10.0));
    }

    #[test]
    fn bounds_are_tight_for_paths() {
        let m = Markup::new(
            0,
            MarkupKind::Polygon,
            MarkupGeometry::Path {
                points: vec![Pt::new(10.0, 10.0), Pt::new(30.0, 12.0), Pt::new(20.0, 40.0)],
                closed: true,
            },
            "t",
        );
        assert_eq!(m.bounds(), Rect::new(10.0, 10.0, 20.0, 30.0));
    }
}
