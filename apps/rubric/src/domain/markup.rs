//! Markup entities: what a user draws on a page.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::geom::{self, Pt, Rect};
use super::style::{palette, Color, FillMode, MarkupStyle};
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
            MarkupKind::Rectangle
            | MarkupKind::Ellipse
            | MarkupKind::Cloud
            | MarkupKind::Polygon => ToolGroup::Shapes,
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
                // Style-free fallback estimate: the default font size stands
                // in for the markup's own. `Markup::bounds` refines this with
                // the markup's real `style.font_size`.
                text_box_rect(*at, *wrap, text, MarkupStyle::default().font_size)
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
fn text_box_rect(at: Pt, wrap: f32, text: &str, font_size: f32) -> Rect {
    let lines = text.lines().count().max(1) as f32;
    Rect::new(at.x, at.y, wrap.max(1.0), lines * text_line_height(font_size))
}

/// How many [`MarkupKind::MeasureCount`] markers lie on `page`.
///
/// Used for the page's running count total; this counts markers independently
/// of the numbers stored on them.
pub fn count_markers_on_page<'a>(
    markups: impl IntoIterator<Item = &'a Markup>,
    page: usize,
) -> u32 {
    markups
        .into_iter()
        .filter(|m| m.page == page && m.kind == MarkupKind::MeasureCount)
        .count() as u32
}

/// The number for the next count marker placed on `page`.
///
/// One more than the highest number already on the page, so deleting a marker
/// never reissues its number. Numbering restarts at `1` on each page.
pub fn next_count_number<'a>(markups: impl IntoIterator<Item = &'a Markup>, page: usize) -> u32 {
    markups
        .into_iter()
        .filter(|m| m.page == page && m.kind == MarkupKind::MeasureCount)
        .filter_map(|m| match &m.geometry {
            MarkupGeometry::Count { number, .. } => Some(*number),
            _ => None,
        })
        .max()
        .map_or(1, |n| n.saturating_add(1))
}

/// Human-readable running-total label for `total` count markers, e.g.
/// `3 count`.
pub fn count_total_label(total: u32) -> String {
    units::format_value(
        MeasureKind::Count,
        units::MeasureUnit::Point,
        f64::from(total),
    )
}

/// Review workflow status (spec F-301).
///
/// The four-state set is `Open` / `In review` / `Approved` / `Rejected`. It
/// supersedes an earlier, larger enum; no released files exist, so old status
/// strings are intentionally not accepted on load.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    /// Newly created and not yet picked up by a reviewer (default).
    #[default]
    Open,
    /// Actively being reviewed.
    InReview,
    /// Signed off.
    Approved,
    /// Not accepted.
    Rejected,
}

impl ReviewStatus {
    /// All statuses in display order (status picker / legend).
    pub const ALL: [ReviewStatus; 4] = [
        ReviewStatus::Open,
        ReviewStatus::InReview,
        ReviewStatus::Approved,
        ReviewStatus::Rejected,
    ];

    /// Human-readable label shown in the UI.
    pub fn label(self) -> &'static str {
        match self {
            ReviewStatus::Open => "Open",
            ReviewStatus::InReview => "In review",
            ReviewStatus::Approved => "Approved",
            ReviewStatus::Rejected => "Rejected",
        }
    }

    /// Legend/indicator colour (spec F-305). Every status has a distinct hue.
    pub fn color(self) -> Color {
        match self {
            ReviewStatus::Open => palette::GREY,
            ReviewStatus::InReview => palette::AMBER,
            ReviewStatus::Approved => palette::GREEN,
            ReviewStatus::Rejected => palette::RED,
        }
    }
}

/// Built-in stamp presets (spec F-111).
///
/// Stamps convey a review verdict from a fixed library rather than free text.
/// The set mixes status marks (`Approved`, `Rejected`, `Draft`), which draw
/// their word, with utility marks (`Check`, `Cross`), which draw a glyph.
///
/// Colours are grouped by meaning rather than being pairwise-distinct:
/// affirmative marks are green, negative marks red, and in-progress marks
/// amber. This mirrors the visual convention of check/cross marks and does
/// not try to give every preset a unique hue (contrast [`ReviewStatus`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StampPreset {
    /// Signed-off status mark; draws its word.
    Approved,
    /// Not-accepted status mark; draws its word.
    Rejected,
    /// Work-in-progress status mark; draws its word.
    Draft,
    /// Affirmative utility mark; draws a tick glyph.
    Check,
    /// Negative utility mark; draws a cross glyph.
    Cross,
}

impl StampPreset {
    /// All presets in display order (stamp picker).
    pub const ALL: [StampPreset; 5] = [
        StampPreset::Approved,
        StampPreset::Rejected,
        StampPreset::Draft,
        StampPreset::Check,
        StampPreset::Cross,
    ];

    /// Word label used in menus, pickers and accessibility text.
    ///
    /// Always a word, even for [`StampPreset::Check`] and
    /// [`StampPreset::Cross`], so UI controls stay readable. Use
    /// [`StampPreset::glyph`] for the content a renderer draws.
    pub fn label(self) -> &'static str {
        match self {
            StampPreset::Approved => "APPROVED",
            StampPreset::Rejected => "REJECTED",
            StampPreset::Draft => "DRAFT",
            StampPreset::Check => "CHECK",
            StampPreset::Cross => "CROSS",
        }
    }

    /// Content a renderer draws for this preset.
    ///
    /// Status marks draw their word; check/cross draw a glyph. Renderers should
    /// centre this within the stamp's bounds.
    pub fn glyph(self) -> &'static str {
        match self {
            StampPreset::Approved => "APPROVED",
            StampPreset::Rejected => "REJECTED",
            StampPreset::Draft => "DRAFT",
            StampPreset::Check => "✓",
            StampPreset::Cross => "✗",
        }
    }

    /// Stamp colour, grouped by meaning (not pairwise-distinct).
    pub fn color(self) -> Color {
        match self {
            StampPreset::Approved => palette::GREEN,
            StampPreset::Rejected => palette::RED,
            StampPreset::Draft => palette::AMBER,
            StampPreset::Check => palette::GREEN,
            StampPreset::Cross => palette::RED,
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
    /// Human-readable label using the default measurement precision.
    pub fn label(&self, unit: units::MeasureUnit) -> String {
        self.label_with(unit, units::DEFAULT_PRECISION)
    }

    /// Human-readable label using `precision` decimal places for linear and
    /// area values.
    ///
    /// See [`units::format_value_precise`] for the angle/count handling rules.
    pub fn label_with(&self, unit: units::MeasureUnit, precision: u8) -> String {
        units::format_value_precise(self.kind, unit, self.value, precision)
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
    /// Legacy free-form group label (from early tool-set presets). Kept for
    /// serialization compatibility; it is not the visibility/lock layer.
    /// Use [`Markup::layer_id`] for layer membership.
    #[serde(default)]
    pub layer: Option<String>,
    /// Id of the [`super::document::Layer`] this markup belongs to.
    ///
    /// `None` means the document's default layer. Legacy files have no such
    /// field and therefore migrate onto the default layer automatically.
    #[serde(default)]
    pub layer_id: Option<Uuid>,
    /// Per-markup lock. Layer locking lives on [`super::document::Layer`].
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub measurement: Option<Measurement>,
    /// Built-in stamp preset drawn inside the stamp bounds (spec F-111).
    ///
    /// `None` means a legacy or blank stamp: renderers draw the plain stamp
    /// rectangle only. Only meaningful when [`Markup::kind`] is
    /// [`MarkupKind::Stamp`]; other kinds ignore it. Legacy files have no such
    /// field and therefore load with no preset.
    #[serde(default)]
    pub stamp: Option<StampPreset>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

impl Markup {
    pub fn new(
        page: usize,
        kind: MarkupKind,
        geometry: MarkupGeometry,
        author: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            page,
            kind,
            geometry,
            style: kind.default_style(),
            author: author.into(),
            subject: kind.label().to_string(),
            status: ReviewStatus::Open,
            layer: None,
            layer_id: None,
            locked: false,
            comments: Vec::new(),
            measurement: None,
            stamp: None,
            created: now,
            modified: now,
        }
    }

    pub fn with_style(mut self, style: MarkupStyle) -> Self {
        self.style = style;
        self.style.font_size = self.style.font_size.max(6.0);
        self
    }

    /// Assign this markup to a layer (builder form). `None` would mean the
    /// default layer, so only explicit ids are accepted here.
    pub fn with_layer(mut self, layer_id: Uuid) -> Self {
        self.layer_id = Some(layer_id);
        self
    }

    /// Attach a built-in stamp preset (builder form).
    ///
    /// Only meaningful when the markup's kind is [`MarkupKind::Stamp`]; other
    /// kinds ignore the preset. The preset supplies the drawn content and
    /// colour; callers should also apply [`StampPreset::color`] to the style.
    pub fn with_stamp(mut self, preset: StampPreset) -> Self {
        self.stamp = Some(preset);
        self
    }

    /// Tight bounding box in page points.
    ///
    /// For text markups the height is estimated from the markup's own
    /// `style.font_size` (see `text_box_rect`), so a 24 pt text occupies its
    /// drawn height; the geometry-only [`MarkupGeometry::bounds`] estimate
    /// falls back to the default font size.
    pub fn bounds(&self) -> Rect {
        match &self.geometry {
            MarkupGeometry::Text { at, wrap, text } => {
                text_box_rect(*at, *wrap, text, self.style.font_size)
            }
            _ => self.geometry.bounds(),
        }
    }

    /// Hit test with a tolerance in page points.
    ///
    /// Filled closed shapes hit anywhere inside; unfilled (stroke-only) shapes
    /// hit only near their outline, within `tolerance` of an edge or segment.
    /// Highlights keep full-bounds hits and the remaining geometry kinds keep
    /// their existing behaviour. The signature is stable for the UI layer.
    pub fn hit_test(&self, p: Pt, tolerance: f32) -> bool {
        let tolerance = tolerance.max(0.0);
        let filled = self.style.fill_mode != FillMode::None;
        match &self.geometry {
            MarkupGeometry::Bounds(r) => match self.kind {
                MarkupKind::Rectangle => {
                    if filled {
                        r.expanded(tolerance).contains(p)
                    } else {
                        distance_to_rect_border(*r, p) <= tolerance
                    }
                }
                MarkupKind::Ellipse => {
                    if filled {
                        ellipse_contains(r.expanded(tolerance), p)
                    } else {
                        // Empty interior: hit only in the band around the
                        // elliptical edge (inside outer, outside inner).
                        ellipse_contains(r.expanded(tolerance), p)
                            && !ellipse_contains(r.expanded(-tolerance), p)
                    }
                }
                MarkupKind::Cloud => {
                    // Approximation: a revision cloud is drawn as scalloped
                    // bumps straddling the rectangle perimeter; the exact
                    // scallop path lives in the renderer. Hit-testing therefore
                    // uses the underlying rectangle border (unfilled) or the
                    // rectangle interior (filled) as a close approximation.
                    if filled {
                        r.expanded(tolerance).contains(p)
                    } else {
                        distance_to_rect_border(*r, p) <= tolerance
                    }
                }
                _ => {
                    // Text-ish markups and highlights are hit anywhere in their
                    // bounds; other unfilled bounds hit within an edge band.
                    if self.kind == MarkupKind::Highlight || filled {
                        r.expanded(tolerance).contains(p)
                    } else {
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
            },
            MarkupGeometry::Segment { a, b } => {
                distance_to_segment(p, *a, *b) <= tolerance.max(3.0)
            }
            MarkupGeometry::Path { points, closed } => {
                let tol = tolerance.max(3.0);
                polyline_hit(p, points, *closed, tol)
                    || (*closed && filled && point_in_polygon(points, p))
            }
            MarkupGeometry::Ink { strokes } => strokes
                .iter()
                .any(|s| polyline_hit(p, s, false, tolerance.max(3.0))),
            MarkupGeometry::Text { at, wrap, text } => {
                text_box_rect(*at, *wrap, text, self.style.font_size)
                    .expanded(tolerance)
                    .contains(p)
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
    ///
    /// A measure kind whose geometry does not fit it (possible for imported or
    /// hand-edited files, e.g. a length measure carrying path geometry) is
    /// left with no measurement rather than a misleading zero value.
    /// Non-measure kinds always clear the measurement. An open perimeter
    /// measures only its drawn segments; a closed one adds the closing edge.
    pub fn recompute_measurement(&mut self, points: Scale) {
        let Some(kind) = self.kind.measure_kind() else {
            self.measurement = None;
            return;
        };
        let raw = match (&self.geometry, kind) {
            (MarkupGeometry::Segment { a, b }, MeasureKind::Length)
            | (MarkupGeometry::Segment { a, b }, MeasureKind::Diameter)
            | (MarkupGeometry::Segment { a, b }, MeasureKind::Radius) => {
                Some(a.distance(*b) as f64)
            }
            (MarkupGeometry::Path { points, closed }, MeasureKind::Perimeter) => {
                let length = if *closed {
                    geom::closed_path_length(points)
                } else {
                    geom::path_length(points)
                };
                Some(length as f64)
            }
            (MarkupGeometry::Path { points, .. }, MeasureKind::Area) => {
                Some(geom::polygon_area(points) as f64)
            }
            (MarkupGeometry::Angle { a, vertex, b }, MeasureKind::Angle) => {
                Some(geom::angle_degrees(*a, *vertex, *b) as f64)
            }
            (MarkupGeometry::Count { number, .. }, MeasureKind::Count) => Some(*number as f64),
            // Unmatched kind/geometry pair: no measurement, not a fake zero.
            _ => None,
        };
        let Some(raw) = raw else {
            self.measurement = None;
            return;
        };
        // Radius is half the segment length; diameter is the full length.
        let raw = match kind {
            MeasureKind::Radius => raw / 2.0,
            _ => raw,
        };
        let value = match kind {
            MeasureKind::Angle | MeasureKind::Count => raw,
            MeasureKind::Area => raw * (points.units_per_point * points.units_per_point),
            _ => points.points_to_units(raw),
        };
        self.measurement = Some(Measurement {
            kind,
            value,
            value_points: raw,
        });
    }

    /// Replace the visual style without changing geometry, metadata, review
    /// status, lock state, comments or measurements. Refreshes `modified`.
    pub fn apply_style(&mut self, style: MarkupStyle) {
        self.style = style;
        self.style.font_size = self.style.font_size.max(6.0);
        self.touch();
    }

    /// Append a comment authored by `author` and return its id.
    ///
    /// Whitespace-only text is refused (`None`) so a note pin cannot collect
    /// empty entries from a stray Enter. Refreshes `modified` on success.
    pub fn add_comment(
        &mut self,
        author: impl Into<String>,
        text: impl Into<String>,
    ) -> Option<Uuid> {
        let text = text.into();
        if text.trim().is_empty() {
            return None;
        }
        let comment = Comment::new(author, text);
        let id = comment.id;
        self.comments.push(comment);
        self.touch();
        Some(id)
    }

    pub fn touch(&mut self) {
        self.modified = Utc::now();
    }

    /// Plain-text summary line used in lists and exports.
    ///
    /// Measurement values are formatted in raw PDF points (uncalibrated); use
    /// [`Markup::summary_with`] to format them in a page's display unit, as
    /// the canvas does.
    pub fn summary(&self) -> String {
        self.summary_with(Scale::PAPER)
    }

    /// Plain-text summary line, formatting any measurement in `scale`'s
    /// display unit.
    ///
    /// The value is recomputed from the stored raw points exactly as
    /// [`Markup::recompute_measurement`] does, so a calibrated page lists
    /// `10.000 ft` where the canvas shows `10.000 ft`.
    pub fn summary_with(&self, scale: Scale) -> String {
        let mut s = self.subject.clone();
        if let Some(m) = &self.measurement {
            s.push_str(": ");
            let value = match m.kind {
                MeasureKind::Angle | MeasureKind::Count => m.value_points,
                _ => scale.points_to_units(m.value_points),
            };
            s.push_str(&units::format_value(m.kind, scale.unit, value));
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
        assert!(
            (meas.value - 10.0).abs() < 1e-6,
            "720 pt = 10 ft, got {}",
            meas.value
        );
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
    fn perimeter_measurement_honours_the_closed_flag() {
        // Three sides of a 72 pt square: 216 pt drawn, 288 pt when closed.
        let points = vec![
            Pt::new(0.0, 0.0),
            Pt::new(72.0, 0.0),
            Pt::new(72.0, 72.0),
            Pt::new(0.0, 72.0),
        ];
        let mut open = Markup::new(
            0,
            MarkupKind::MeasurePerimeter,
            MarkupGeometry::Path {
                points: points.clone(),
                closed: false,
            },
            "t",
        );
        open.recompute_measurement(Scale::PAPER);
        assert!(
            (open.measurement.unwrap().value_points - 216.0).abs() < 1e-6,
            "an open perimeter measures only the drawn segments"
        );

        let mut closed = Markup::new(
            0,
            MarkupKind::MeasurePerimeter,
            MarkupGeometry::Path {
                points,
                closed: true,
            },
            "t",
        );
        closed.recompute_measurement(Scale::PAPER);
        assert!(
            (closed.measurement.unwrap().value_points - 288.0).abs() < 1e-6,
            "a closed perimeter adds the closing segment"
        );
    }

    #[test]
    fn mismatched_measure_kind_and_geometry_leaves_no_measurement() {
        // Importers can pair a measure kind with alien geometry; the markup
        // must carry no measurement rather than a misleading zero.
        let mut m = Markup::new(
            0,
            MarkupKind::MeasureLength,
            MarkupGeometry::Path {
                points: vec![Pt::new(0.0, 0.0), Pt::new(72.0, 0.0)],
                closed: false,
            },
            "t",
        );
        m.recompute_measurement(Scale::PAPER);
        assert_eq!(m.measurement, None);

        // Non-measure kinds still clear a stale measurement.
        let mut r = Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 10.0, 10.0)),
            "t",
        );
        r.measurement = Some(Measurement {
            kind: MeasureKind::Length,
            value: 1.0,
            value_points: 1.0,
        });
        r.recompute_measurement(Scale::PAPER);
        assert_eq!(r.measurement, None);
    }

    #[test]
    fn summary_formats_measurements_in_the_page_unit() {
        let mut m = Markup::new(
            0,
            MarkupKind::MeasureLength,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(720.0, 0.0),
            },
            "t",
        );
        m.recompute_measurement(scale()); // 1 in = 1 ft
        assert_eq!(
            m.summary(),
            "Length: 720.000 pt",
            "uncalibrated fallback formats raw points"
        );
        assert_eq!(
            m.summary_with(scale()),
            "Length: 10.000 ft",
            "the page's display unit, as the canvas shows it"
        );
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
        assert!(
            !rect.hit_test(Pt::new(50.0, 50.0), 2.0),
            "centre of empty rect"
        );

        let highlight = Markup::new(
            0,
            MarkupKind::Highlight,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 100.0, 20.0)),
            "t",
        );
        assert!(
            highlight.hit_test(Pt::new(50.0, 10.0), 2.0),
            "fills are hit inside"
        );
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
    fn apply_style_changes_only_the_visual_style_and_touches() {
        let mut m = Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 10.0, 10.0)),
            "tester",
        );
        m.author = "Author".to_string();
        m.subject = "Subject".to_string();
        m.status = ReviewStatus::Approved;
        m.locked = true;
        let geometry = m.geometry.clone();
        let before = m.modified;
        let style = MarkupStyle::default()
            .with_stroke(palette::BLUE)
            .with_width(4.0);

        m.apply_style(style);

        assert_eq!(m.style, style);
        assert_eq!(m.geometry, geometry);
        assert_eq!(m.author, "Author");
        assert_eq!(m.subject, "Subject");
        assert_eq!(m.status, ReviewStatus::Approved);
        assert!(m.locked);
        assert!(m.modified >= before);
    }

    #[test]
    fn bounds_are_tight_for_paths() {
        let m = Markup::new(
            0,
            MarkupKind::Polygon,
            MarkupGeometry::Path {
                points: vec![
                    Pt::new(10.0, 10.0),
                    Pt::new(30.0, 12.0),
                    Pt::new(20.0, 40.0),
                ],
                closed: true,
            },
            "t",
        );
        assert_eq!(m.bounds(), Rect::new(10.0, 10.0, 20.0, 30.0));
    }

    /// Copy a markup with a different fill mode (the UI's fill toggle).
    fn with_fill(mut m: Markup, mode: FillMode) -> Markup {
        m.style.fill_mode = mode;
        m
    }

    #[test]
    fn filled_rectangle_hits_interior() {
        let base = Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 100.0, 100.0)),
            "t",
        );
        // Default style is stroke-only: the hollow centre is not a hit.
        assert!(!base.hit_test(Pt::new(50.0, 50.0), 2.0), "hollow centre");
        assert!(base.hit_test(Pt::new(0.0, 50.0), 2.0), "left edge");

        let filled = with_fill(base, FillMode::Solid);
        assert!(
            filled.hit_test(Pt::new(50.0, 50.0), 2.0),
            "filled rectangle is hit in its interior"
        );
        assert!(
            !filled.hit_test(Pt::new(150.0, 50.0), 2.0),
            "outside the tolerance band"
        );
    }

    #[test]
    fn unfilled_rectangle_hits_only_near_edge() {
        let m = Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 100.0, 100.0)),
            "t",
        );
        assert!(m.hit_test(Pt::new(50.0, 0.0), 2.0), "top edge");
        assert!(
            m.hit_test(Pt::new(50.0, -1.5), 2.0),
            "just outside the top edge"
        );
        assert!(!m.hit_test(Pt::new(50.0, 50.0), 2.0), "hollow centre");
        assert!(!m.hit_test(Pt::new(-10.0, 50.0), 2.0), "far outside");
    }

    #[test]
    fn ellipse_fill_and_outline_hits() {
        let base = Markup::new(
            0,
            MarkupKind::Ellipse,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 100.0, 100.0)),
            "t",
        );
        assert!(
            !base.hit_test(Pt::new(2.0, 2.0), 2.0),
            "bounding-box corner lies outside the ellipse"
        );
        assert!(
            !base.hit_test(Pt::new(50.0, 50.0), 2.0),
            "unfilled ellipse centre"
        );
        assert!(
            base.hit_test(Pt::new(50.0, 1.0), 2.0),
            "near the top of the outline"
        );

        let filled = with_fill(base, FillMode::Solid);
        assert!(
            filled.hit_test(Pt::new(50.0, 50.0), 2.0),
            "filled ellipse interior"
        );
        assert!(
            !filled.hit_test(Pt::new(2.0, 2.0), 2.0),
            "corner still outside the ellipse"
        );
    }

    #[test]
    fn cloud_hits_near_perimeter_not_interior() {
        let base = Markup::new(
            0,
            MarkupKind::Cloud,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 100.0, 60.0)),
            "t",
        );
        assert_eq!(base.style.fill_mode, FillMode::None);
        assert!(base.hit_test(Pt::new(50.0, 1.0), 2.0), "near top surround");
        assert!(
            base.hit_test(Pt::new(99.0, 30.0), 2.0),
            "near right surround"
        );
        assert!(!base.hit_test(Pt::new(50.0, 30.0), 2.0), "hollow centre");

        let filled = with_fill(base, FillMode::Solid);
        assert!(
            filled.hit_test(Pt::new(50.0, 30.0), 2.0),
            "filled cloud interior"
        );
    }

    #[test]
    fn filled_polygon_hits_interior() {
        let geom = MarkupGeometry::Path {
            points: vec![
                Pt::new(0.0, 0.0),
                Pt::new(100.0, 0.0),
                Pt::new(100.0, 100.0),
                Pt::new(0.0, 100.0),
            ],
            closed: true,
        };
        let base = Markup::new(0, MarkupKind::Polygon, geom, "t");
        assert!(
            !base.hit_test(Pt::new(50.0, 50.0), 2.0),
            "unfilled polygon is hollow"
        );
        assert!(base.hit_test(Pt::new(0.0, 50.0), 2.0), "polygon edge");

        let filled = with_fill(base, FillMode::Solid);
        assert!(
            filled.hit_test(Pt::new(50.0, 50.0), 2.0),
            "filled polygon interior"
        );
        assert!(
            !filled.hit_test(Pt::new(150.0, 50.0), 2.0),
            "outside the polygon"
        );
    }

    #[test]
    fn ink_hits_near_strokes_not_whole_bounds() {
        let m = Markup::new(
            0,
            MarkupKind::Ink,
            MarkupGeometry::Ink {
                strokes: vec![vec![
                    Pt::new(0.0, 0.0),
                    Pt::new(100.0, 0.0),
                    Pt::new(100.0, 100.0),
                ]],
            },
            "t",
        );
        assert!(
            m.hit_test(Pt::new(50.0, 1.0), 2.0),
            "near the horizontal stroke"
        );
        assert!(
            m.hit_test(Pt::new(99.0, 50.0), 2.0),
            "near the vertical stroke"
        );
        assert!(
            !m.hit_test(Pt::new(20.0, 80.0), 2.0),
            "inside the bounding box but far from any stroke"
        );
        assert!(!m.hit_test(Pt::new(200.0, 200.0), 2.0), "far away");
    }

    #[test]
    fn ink_single_point_stroke_acts_as_dot() {
        let m = Markup::new(
            0,
            MarkupKind::Ink,
            MarkupGeometry::Ink {
                strokes: vec![vec![Pt::new(10.0, 10.0)]],
            },
            "t",
        );
        assert!(m.hit_test(Pt::new(11.0, 10.0), 2.0));
        assert!(!m.hit_test(Pt::new(20.0, 10.0), 2.0));
    }

    #[test]
    fn measurement_labels_honour_precision_and_default() {
        let m = Measurement {
            kind: MeasureKind::Length,
            value: 12.3456,
            value_points: 12.3456,
        };
        assert_eq!(m.label(units::MeasureUnit::Foot), "12.346 ft");
        assert_eq!(m.label_with(units::MeasureUnit::Foot, 0), "12 ft");
        assert_eq!(m.label_with(units::MeasureUnit::Foot, 1), "12.3 ft");

        let angle = Measurement {
            kind: MeasureKind::Angle,
            value: 90.0,
            value_points: 90.0,
        };
        assert_eq!(angle.label_with(units::MeasureUnit::Point, 6), "90.00°");
    }

    #[test]
    fn hit_test_geometry_serde_roundtrip() {
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
            let m = Markup::new(0, MarkupKind::Ink, geom, "t");
            let json = serde_json::to_string(&m).expect("serialize");
            let back: Markup = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(m.geometry, back.geometry);
            assert_eq!(m.style.fill_mode, back.style.fill_mode);
        }
    }

    #[test]
    fn text_and_callout_geometry_roundtrip_and_hit_test() {
        let text_m = Markup::new(
            0,
            MarkupKind::Text,
            MarkupGeometry::Text {
                at: Pt::new(10.0, 10.0),
                wrap: 60.0,
                text: "a\nb".to_string(),
            },
            "t",
        );
        let json = serde_json::to_string(&text_m).expect("serialize");
        let back: Markup = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(text_m.geometry, back.geometry);

        // Hit box (per `hit_test`): `at` is the top-left corner, width is
        // `wrap`, height is the estimated line height at the markup's font
        // size (default 12 pt × 1.35 ≈ 16.2 pt per line; 2 lines ≈ 32.4).
        assert!(text_m.hit_test(Pt::new(10.0, 10.0), 2.0), "at the anchor");
        assert!(
            text_m.hit_test(Pt::new(40.0, 30.0), 2.0),
            "inside the wrap box"
        );
        assert!(
            !text_m.hit_test(Pt::new(80.0, 12.0), 2.0),
            "beyond the wrap width"
        );
        assert!(!text_m.hit_test(Pt::new(200.0, 200.0), 2.0), "far away");

        let callout_m = Markup::new(
            0,
            MarkupKind::Callout,
            MarkupGeometry::Callout {
                rect: Rect::new(0.0, 0.0, 50.0, 20.0),
                tail: Pt::new(70.0, 40.0),
                text: "callout".to_string(),
            },
            "t",
        );
        let json = serde_json::to_string(&callout_m).expect("serialize");
        let back: Markup = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(callout_m.geometry, back.geometry);

        assert!(
            callout_m.hit_test(Pt::new(25.0, 10.0), 2.0),
            "inside the rect"
        );
        // The tail is hit-tested as a point with a 4 pt minimum tolerance.
        assert!(
            callout_m.hit_test(Pt::new(70.0, 40.0), 2.0),
            "the tail point hits within tolerance"
        );
        assert!(!callout_m.hit_test(Pt::new(200.0, 200.0), 2.0), "far away");
    }

    #[test]
    fn text_bounds_and_hit_box_scale_with_font_size() {
        let geom = MarkupGeometry::Text {
            at: Pt::new(10.0, 10.0),
            wrap: 60.0,
            text: "a\nb\nc".to_string(),
        };
        let small = Markup::new(0, MarkupKind::Text, geom.clone(), "t");
        let mut big = Markup::new(0, MarkupKind::Text, geom, "t");
        big.style.font_size = 24.0;

        let small_h = small.bounds().h;
        let big_h = big.bounds().h;
        assert!(
            (big_h - small_h * 2.0).abs() < 1e-4,
            "doubling the font size doubles the estimated text height"
        );
        assert!(
            (big_h - 3.0 * 1.35 * 24.0).abs() < 1e-4,
            "3 lines at 24 pt ≈ 3 × 1.35 × 24"
        );

        // bounds() and hit_test() agree: a point below the small box but
        // inside the 24 pt box hits only the big markup.
        let below_small = Pt::new(40.0, 70.0);
        assert!(
            !small.hit_test(below_small, 1.0),
            "a point below the small box must not hit it"
        );
        assert!(big.hit_test(below_small, 1.0), "inside the 24 pt box");
    }

    #[test]
    fn text_with_degenerate_wrap_stays_hittable() {
        let m = Markup::new(
            0,
            MarkupKind::Text,
            MarkupGeometry::Text {
                at: Pt::new(10.0, 10.0),
                wrap: 0.0, // hand-edited or imported file
                text: "one line".to_string(),
            },
            "t",
        );
        // The wrap width is clamped to at least 1 pt so the box cannot
        // collapse into an unhittable sliver.
        assert_eq!(m.bounds().w, 1.0);
        assert!(m.hit_test(Pt::new(10.5, 12.0), 1.0));
        assert!(!m.hit_test(Pt::new(20.0, 12.0), 1.0), "still bounded");
    }

    #[test]
    fn review_status_default_is_open() {
        assert_eq!(ReviewStatus::default(), ReviewStatus::Open);
        let m = Markup::new(
            0,
            MarkupKind::Note,
            MarkupGeometry::Count {
                at: Pt::new(1.0, 1.0),
                number: 1,
            },
            "t",
        );
        assert_eq!(m.status, ReviewStatus::Open);
    }

    #[test]
    fn review_status_all_matches_spec_set() {
        assert_eq!(
            ReviewStatus::ALL,
            [
                ReviewStatus::Open,
                ReviewStatus::InReview,
                ReviewStatus::Approved,
                ReviewStatus::Rejected,
            ]
        );
        assert_eq!(ReviewStatus::ALL.len(), 4);
        assert_eq!(ReviewStatus::Open.label(), "Open");
        assert_eq!(ReviewStatus::InReview.label(), "In review");
        assert_eq!(ReviewStatus::Approved.label(), "Approved");
        assert_eq!(ReviewStatus::Rejected.label(), "Rejected");
    }

    #[test]
    fn review_status_colors_are_pairwise_distinct() {
        for (i, a) in ReviewStatus::ALL.iter().enumerate() {
            for b in ReviewStatus::ALL.iter().skip(i + 1) {
                assert_ne!(a.color(), b.color(), "{a:?} and {b:?} share a colour");
            }
        }
        assert_eq!(ReviewStatus::Open.color(), palette::GREY);
        assert_eq!(ReviewStatus::InReview.color(), palette::AMBER);
        assert_eq!(ReviewStatus::Approved.color(), palette::GREEN);
        assert_eq!(ReviewStatus::Rejected.color(), palette::RED);
    }

    #[test]
    fn review_status_serde_uses_snake_case() {
        let cases = [
            (ReviewStatus::Open, "\"open\""),
            (ReviewStatus::InReview, "\"in_review\""),
            (ReviewStatus::Approved, "\"approved\""),
            (ReviewStatus::Rejected, "\"rejected\""),
        ];
        for (status, json) in cases {
            assert_eq!(serde_json::to_string(&status).expect("serialize"), json);
            let back: ReviewStatus = serde_json::from_str(json).expect("deserialize");
            assert_eq!(back, status);
        }
        assert_eq!(
            serde_json::from_str::<ReviewStatus>("\"in_review\"").expect("deserialize"),
            ReviewStatus::InReview
        );
        assert_eq!(
            serde_json::from_str::<ReviewStatus>("\"open\"").expect("deserialize"),
            ReviewStatus::Open
        );
    }

    #[test]
    fn legacy_review_status_strings_no_longer_load() {
        // The pre-Phase-4 variant set is intentionally rejected; files using it
        // are re-saved. `"rejected"` is excluded because it remains valid.
        for legacy in [
            "\"none\"",
            "\"accepted\"",
            "\"completed\"",
            "\"cancelled\"",
            "\"alert\"",
        ] {
            assert!(
                serde_json::from_str::<ReviewStatus>(legacy).is_err(),
                "legacy status {legacy} should not deserialize"
            );
        }
    }

    #[test]
    fn markup_status_serde_roundtrip_preserves_in_review() {
        let mut m = Markup::new(
            0,
            MarkupKind::Note,
            MarkupGeometry::Count {
                at: Pt::new(3.0, 4.0),
                number: 2,
            },
            "t",
        );
        m.status = ReviewStatus::InReview;
        let json = serde_json::to_string(&m).expect("serialize");
        assert!(json.contains("\"status\":\"in_review\""), "json = {json}");
        let back: Markup = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.status, ReviewStatus::InReview);
    }

    fn count_marker(page: usize, number: u32) -> Markup {
        Markup::new(
            page,
            MarkupKind::MeasureCount,
            MarkupGeometry::Count {
                at: Pt::new(0.0, 0.0),
                number,
            },
            "t",
        )
    }

    #[test]
    fn next_count_number_is_per_page_and_never_reissues() {
        // A non-count markup that happens to carry Count geometry must not
        // influence the numbering.
        let decoy = Markup::new(
            0,
            MarkupKind::Note,
            MarkupGeometry::Count {
                at: Pt::new(0.0, 0.0),
                number: 9,
            },
            "t",
        );
        let markups = vec![
            count_marker(0, 1),
            count_marker(0, 2),
            count_marker(1, 1),
            decoy,
        ];
        assert_eq!(next_count_number(&markups, 0), 3);
        assert_eq!(next_count_number(&markups, 1), 2);
        assert_eq!(next_count_number(&markups, 2), 1, "fresh page starts at 1");
        assert_eq!(next_count_number(&Vec::new(), 0), 1);
    }

    #[test]
    fn count_markers_on_page_counts_only_counters_on_that_page() {
        let decoy = Markup::new(
            0,
            MarkupKind::Note,
            MarkupGeometry::Count {
                at: Pt::new(0.0, 0.0),
                number: 9,
            },
            "t",
        );
        let markups = vec![
            count_marker(0, 1),
            count_marker(0, 5),
            count_marker(1, 1),
            decoy,
        ];
        assert_eq!(count_markers_on_page(&markups, 0), 2);
        assert_eq!(count_markers_on_page(&markups, 1), 1);
        assert_eq!(count_markers_on_page(&markups, 2), 0);
    }

    #[test]
    fn count_total_label_is_an_integer_with_count_suffix() {
        assert_eq!(count_total_label(0), "0 count");
        assert_eq!(count_total_label(3), "3 count");
    }

    #[test]
    fn count_number_survives_serde_roundtrip() {
        let m = count_marker(1, 7);
        let json = serde_json::to_string(&m).expect("serialize");
        let back: Markup = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.geometry, m.geometry);
        assert_eq!(back.kind, MarkupKind::MeasureCount);
        assert_eq!(next_count_number(&[back], 1), 8);
    }

    #[test]
    fn stamp_preset_all_matches_spec_set() {
        assert_eq!(
            StampPreset::ALL,
            [
                StampPreset::Approved,
                StampPreset::Rejected,
                StampPreset::Draft,
                StampPreset::Check,
                StampPreset::Cross,
            ]
        );
        assert_eq!(StampPreset::ALL.len(), 5);
    }

    #[test]
    fn stamp_preset_labels_are_words_glyphs_are_drawn_content() {
        for preset in StampPreset::ALL {
            assert!(!preset.label().is_empty());
            assert!(!preset.glyph().is_empty());
        }
        assert_eq!(StampPreset::Approved.label(), "APPROVED");
        assert_eq!(StampPreset::Rejected.label(), "REJECTED");
        assert_eq!(StampPreset::Draft.label(), "DRAFT");
        assert_eq!(StampPreset::Check.label(), "CHECK");
        assert_eq!(StampPreset::Cross.label(), "CROSS");
        // Status marks draw their word; utility marks draw a glyph.
        assert_eq!(StampPreset::Approved.glyph(), "APPROVED");
        assert_eq!(StampPreset::Rejected.glyph(), "REJECTED");
        assert_eq!(StampPreset::Draft.glyph(), "DRAFT");
        assert_eq!(StampPreset::Check.glyph(), "✓");
        assert_eq!(StampPreset::Cross.glyph(), "✗");
    }

    #[test]
    fn stamp_preset_colors_group_by_meaning() {
        // Affirmative green, negative red, in-progress amber. Unlike
        // ReviewStatus, stamp presets are intentionally not pairwise-distinct.
        assert_eq!(StampPreset::Approved.color(), palette::GREEN);
        assert_eq!(StampPreset::Check.color(), palette::GREEN);
        assert_eq!(StampPreset::Rejected.color(), palette::RED);
        assert_eq!(StampPreset::Cross.color(), palette::RED);
        assert_eq!(StampPreset::Draft.color(), palette::AMBER);
    }

    #[test]
    fn stamp_preset_serde_uses_snake_case() {
        let cases = [
            (StampPreset::Approved, "\"approved\""),
            (StampPreset::Rejected, "\"rejected\""),
            (StampPreset::Draft, "\"draft\""),
            (StampPreset::Check, "\"check\""),
            (StampPreset::Cross, "\"cross\""),
        ];
        for (preset, json) in cases {
            assert_eq!(serde_json::to_string(&preset).expect("serialize"), json);
            let back: StampPreset = serde_json::from_str(json).expect("deserialize");
            assert_eq!(back, preset);
        }
    }

    #[test]
    fn new_stamp_has_no_preset_and_with_stamp_sets_it() {
        let m = Markup::new(
            0,
            MarkupKind::Stamp,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 80.0, 30.0)),
            "t",
        );
        assert_eq!(m.stamp, None);
        let stamped = m.with_stamp(StampPreset::Approved);
        assert_eq!(stamped.stamp, Some(StampPreset::Approved));
        assert_eq!(stamped.kind, MarkupKind::Stamp);
        // Builder leaves the rest of the markup untouched.
        assert_eq!(stamped.page, 0);
    }

    #[test]
    fn legacy_markup_without_stamp_field_loads_as_none() {
        let m = Markup::new(
            0,
            MarkupKind::Stamp,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 80.0, 30.0)),
            "t",
        );
        let mut value = serde_json::to_value(&m).expect("json");
        value.as_object_mut().expect("object").remove("stamp");
        let back: Markup = serde_json::from_value(value).expect("deserialize");
        assert_eq!(back.stamp, None);
    }

    #[test]
    fn markup_stamp_serde_roundtrip_preserves_preset() {
        let m = Markup::new(
            0,
            MarkupKind::Stamp,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 80.0, 30.0)),
            "t",
        )
        .with_stamp(StampPreset::Rejected);
        let json = serde_json::to_string(&m).expect("serialize");
        assert!(json.contains("\"stamp\":\"rejected\""), "json = {json}");
        let back: Markup = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.stamp, Some(StampPreset::Rejected));
    }

    #[test]
    fn add_comment_appends_and_refuses_blank() {
        let mut m = Markup::new(
            0,
            MarkupKind::Note,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 20.0, 20.0)),
            "tester",
        );
        let before = m.modified;
        assert_eq!(m.add_comment("Ann", "   "), None);
        assert!(m.comments.is_empty());
        assert_eq!(m.modified, before, "a refused comment must not touch");

        let id = m.add_comment("Ann", "looks good").expect("comment id");
        assert_eq!(m.comments.len(), 1);
        assert_eq!(m.comments[0].id, id);
        assert_eq!(m.comments[0].author, "Ann");
        assert_eq!(m.comments[0].text, "looks good");
        assert!(m.modified >= before);
    }
}
