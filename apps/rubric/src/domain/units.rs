//! Measurement units and scale handling.
//!
//! A PDF page is measured in points. A drawing, however, is measured in real
//! world units (feet, metres, ...). A [`Scale`] captures that relationship with
//! a single number: how many *real* units one point represents. This keeps all
//! measurement maths trivial and obviously correct.

use serde::{Deserialize, Serialize};

/// A real-world unit of length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasureUnit {
    Point,
    Inch,
    Foot,
    Yard,
    Mile,
    Millimeter,
    Centimeter,
    Meter,
    Kilometer,
}

impl MeasureUnit {
    pub const ALL: [MeasureUnit; 9] = [
        MeasureUnit::Point,
        MeasureUnit::Inch,
        MeasureUnit::Foot,
        MeasureUnit::Yard,
        MeasureUnit::Mile,
        MeasureUnit::Millimeter,
        MeasureUnit::Centimeter,
        MeasureUnit::Meter,
        MeasureUnit::Kilometer,
    ];

    /// Short suffix used in labels, e.g. `12.30 ft`.
    pub fn suffix(self) -> &'static str {
        match self {
            MeasureUnit::Point => "pt",
            MeasureUnit::Inch => "in",
            MeasureUnit::Foot => "ft",
            MeasureUnit::Yard => "yd",
            MeasureUnit::Mile => "mi",
            MeasureUnit::Millimeter => "mm",
            MeasureUnit::Centimeter => "cm",
            MeasureUnit::Meter => "m",
            MeasureUnit::Kilometer => "km",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            MeasureUnit::Point => "Points",
            MeasureUnit::Inch => "Inches",
            MeasureUnit::Foot => "Feet",
            MeasureUnit::Yard => "Yards",
            MeasureUnit::Mile => "Miles",
            MeasureUnit::Millimeter => "Millimetres",
            MeasureUnit::Centimeter => "Centimetres",
            MeasureUnit::Meter => "Metres",
            MeasureUnit::Kilometer => "Kilometres",
        }
    }

    /// How many points make up one of this unit (1 pt = 1/72 in).
    pub fn points_per_unit(self) -> f64 {
        match self {
            MeasureUnit::Point => 1.0,
            MeasureUnit::Inch => 72.0,
            MeasureUnit::Foot => 864.0,
            MeasureUnit::Yard => 2592.0,
            MeasureUnit::Mile => 4_561_920.0,
            MeasureUnit::Millimeter => 72.0 / 25.4,
            MeasureUnit::Centimeter => 720.0 / 25.4,
            MeasureUnit::Meter => 7200.0 / 25.4,
            MeasureUnit::Kilometer => 7_200_000.0 / 25.4,
        }
    }
}

/// The drawing scale of a page: `units_per_point` real units per PDF point.
///
/// Example: a 1:100 plan drawn such that 1 pt on paper equals 100 pt on site,
/// expressed in metres, would carry a suitable `units_per_point`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub unit: MeasureUnit,
    pub units_per_point: f64,
}

impl Scale {
    /// A scale where one point is one point (paper space / no scaling).
    pub const PAPER: Scale = Scale {
        unit: MeasureUnit::Point,
        units_per_point: 1.0,
    };

    pub fn new(unit: MeasureUnit, units_per_point: f64) -> Self {
        Self {
            unit,
            units_per_point: units_per_point.max(f64::EPSILON),
        }
    }

    /// Build a scale from a known mapping, e.g. `1 inch == 10 feet`.
    pub fn from_ratio(unit: MeasureUnit, known_units: f64, known_points: f64) -> Self {
        let upp = if known_points.abs() < f64::EPSILON {
            1.0
        } else {
            (known_units / known_points).abs()
        };
        Self::new(unit, upp)
    }

    /// Re-express this scale in another display unit while preserving the
    /// physical length it represents.
    ///
    /// `units_per_point` is re-derived from the old unit's size so a given
    /// point distance maps to the same real-world length before and after the
    /// conversion (e.g. a `10 ft` reading stays ten feet when shown in
    /// metres). Converting to the unit already in use is the identity.
    pub fn with_display_unit(self, unit: MeasureUnit) -> Self {
        let upp = self.units_per_point * self.unit.points_per_unit() / unit.points_per_unit();
        Self::new(unit, upp)
    }

    pub fn points_to_units(&self, points: f64) -> f64 {
        points * self.units_per_point
    }

    pub fn units_to_points(&self, units: f64) -> f64 {
        units / self.units_per_point
    }

    /// Human readable description, e.g. `1 in = 10 ft`.
    ///
    /// The left-hand anchor is always **one inch** (72 PDF points), so the
    /// label reads correctly whatever the display unit: a 1:1 scale in inches
    /// describes as `1 in = 1.000 in`, and a 1 in = 1 ft plan as
    /// `1 in = 1.000 ft`.
    pub fn describe(&self) -> String {
        // One inch is 72 PDF points (1 pt = 1/72 in). That fixed anchor is
        // exactly what the `1 in = …` left-hand side claims, so the conversion
        // must start from 72 pt — not from one *display* unit.
        const POINTS_PER_INCH: f64 = 72.0;
        let per_unit = 1.0 / self.units_per_point; // points per real unit
        let real = self.points_to_units(POINTS_PER_INCH);
        format!(
            "1 {} = {:.3} {}  ({} pt / {})",
            short_anchor(),
            real,
            self.unit.suffix(),
            trim(per_unit),
            self.unit.suffix()
        )
    }
}

/// Build a page scale by calibrating against a drawn reference segment of
/// known real-world length (v0.4 gap #4: drag calibration).
///
/// `segment_points` is the length the user drew on the page, in PDF points;
/// `real_units` is the real-world distance that segment represents, expressed
/// in `unit`. The returned [`Scale`] is what the UI writes into
/// `Page.scale`, after which every measurement on the page recomputes from
/// its stored raw points (see [`crate::domain::Markup::recompute_measurement`]).
///
/// Degenerate inputs are refused rather than divided through:
///
/// * a non-finite segment length or real distance yields [`Scale::PAPER`]
///   (1 pt = 1 pt), the same honest fallback an uncalibrated page carries;
/// * a zero-length segment mirrors [`Scale::from_ratio`] and degrades to one
///   unit per point in the requested unit — the UI refuses such segments
///   before the calibration dialog opens, so this is a safety net only.
pub fn calibrate_scale(segment_points: f32, real_units: f64, unit: MeasureUnit) -> Scale {
    if !segment_points.is_finite() || !real_units.is_finite() {
        return Scale::PAPER;
    }
    Scale::from_ratio(unit, real_units, f64::from(segment_points))
}

fn short_anchor() -> &'static str {
    "in"
}

fn trim(v: f64) -> String {
    let s = format!("{v:.4}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// A quantity produced by a measurement tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasureKind {
    Length,
    Perimeter,
    Area,
    Angle,
    Diameter,
    Radius,
    Count,
}

impl MeasureKind {
    pub fn label(self) -> &'static str {
        match self {
            MeasureKind::Length => "Length",
            MeasureKind::Perimeter => "Perimeter",
            MeasureKind::Area => "Area",
            MeasureKind::Angle => "Angle",
            MeasureKind::Diameter => "Diameter",
            MeasureKind::Radius => "Radius",
            MeasureKind::Count => "Count",
        }
    }

    /// Whether the result is an area (affects unit suffix: `ft²` vs `ft`).
    pub fn is_area(self) -> bool {
        matches!(self, MeasureKind::Area)
    }
}

/// Default number of decimal places for linear and area measurements.
pub const DEFAULT_PRECISION: u8 = 3;

/// Largest accepted number of decimal places.
pub const MAX_PRECISION: u8 = 6;

/// Clamp a requested precision to the supported range `0..=6`.
pub fn clamp_precision(precision: u8) -> u8 {
    precision.min(MAX_PRECISION)
}

/// Format a raw numeric measurement with the default precision.
///
/// This is a convenience wrapper around [`format_value_precise`] using
/// [`DEFAULT_PRECISION`], so existing call sites keep their behaviour.
pub fn format_value(kind: MeasureKind, unit: MeasureUnit, value: f64) -> String {
    format_value_precise(kind, unit, value, DEFAULT_PRECISION)
}

/// Format a raw numeric measurement with the unit's suffix and `precision`
/// decimal places.
///
/// The precision preference controls linear measurements (length, perimeter,
/// diameter, radius) and areas (the same number of decimal places, suffix
/// squared). Two kinds deliberately ignore it:
///
/// * [`MeasureKind::Angle`] is always shown with two decimals (`90.00°`):
///   angular precision is conventionally fixed and independent of drawing
///   units.
/// * [`MeasureKind::Count`] is always shown as an integer (`3 count`) because
///   counts are whole numbers.
///
/// `precision` is clamped to `0..=`[`MAX_PRECISION`] (0..=6), so callers may
/// pass UI values without pre-validating them.
pub fn format_value_precise(
    kind: MeasureKind,
    unit: MeasureUnit,
    value: f64,
    precision: u8,
) -> String {
    if kind == MeasureKind::Angle {
        return format!("{value:.2}°");
    }
    if kind == MeasureKind::Count {
        return format!("{value:.0} count");
    }
    let precision = usize::from(clamp_precision(precision));
    let suffix = unit.suffix();
    if kind.is_area() {
        format!("{value:.precision$} {suffix}²")
    } else {
        format!("{value:.precision$} {suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inch_is_72_points() {
        assert!((MeasureUnit::Inch.points_per_unit() - 72.0).abs() < 1e-9);
        assert!((MeasureUnit::Foot.points_per_unit() - 864.0).abs() < 1e-9);
    }

    #[test]
    fn ratio_scale_computes_units_per_point() {
        // 1 inch on paper == 10 feet on site.
        let s = Scale::from_ratio(MeasureUnit::Foot, 10.0, 72.0);
        assert!((s.units_per_point - 10.0 / 72.0).abs() < 1e-9);
        // 720 points (10 in) should read 100 ft.
        assert!((s.points_to_units(720.0) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn round_trip_units() {
        let s = Scale::new(MeasureUnit::Meter, 0.01);
        let pts = 1234.0;
        assert!((s.units_to_points(s.points_to_units(pts)) - pts).abs() < 1e-6);
    }

    #[test]
    fn display_unit_conversion_preserves_physical_length() {
        // 1 inch on paper == 1 foot on site.
        let feet = Scale::from_ratio(MeasureUnit::Foot, 1.0, 72.0);
        let metres = feet.with_display_unit(MeasureUnit::Meter);
        let pts = 720.0; // 10 inches on paper
        assert!((feet.points_to_units(pts) - 10.0).abs() < 1e-9);
        let expected = feet.points_to_units(pts) * MeasureUnit::Foot.points_per_unit()
            / MeasureUnit::Meter.points_per_unit();
        assert!((metres.points_to_units(pts) - expected).abs() < 1e-9);
    }

    #[test]
    fn display_unit_conversion_to_same_unit_is_identity() {
        let s = Scale::from_ratio(MeasureUnit::Foot, 10.0, 72.0);
        let same = s.with_display_unit(MeasureUnit::Foot);
        assert_eq!(same.unit, MeasureUnit::Foot);
        assert!((same.units_per_point - s.units_per_point).abs() < f64::EPSILON);
    }

    #[test]
    fn display_unit_conversion_round_trips() {
        let s = Scale::from_ratio(MeasureUnit::Foot, 10.0, 72.0);
        let back = s
            .with_display_unit(MeasureUnit::Meter)
            .with_display_unit(MeasureUnit::Foot);
        assert_eq!(back.unit, MeasureUnit::Foot);
        assert!((back.units_per_point - s.units_per_point).abs() < 1e-12);
    }

    #[test]
    fn describe_anchors_on_one_inch() {
        // 1 in on paper == 1 ft on site.
        let plan = Scale::from_ratio(MeasureUnit::Foot, 1.0, 72.0);
        let text = plan.describe();
        assert!(
            text.starts_with("1 in = 1.000 ft"),
            "unexpected describe(): {text}"
        );

        // 1:1 in inches: one inch of paper is one real inch.
        let full_size = Scale::new(MeasureUnit::Inch, 1.0 / 72.0);
        let text = full_size.describe();
        assert!(
            text.starts_with("1 in = 1.000 in"),
            "unexpected describe(): {text}"
        );
    }

    #[test]
    fn describe_reports_points_per_display_unit() {
        // 1 in = 1 ft => 72 points per foot.
        let plan = Scale::from_ratio(MeasureUnit::Foot, 1.0, 72.0);
        assert!(
            plan.describe().contains("(72 pt / ft)"),
            "unexpected describe(): {}",
            plan.describe()
        );
    }

    #[test]
    fn formatting_suffixes() {
        assert_eq!(
            format_value(MeasureKind::Area, MeasureUnit::Foot, 12.5),
            "12.500 ft²"
        );
        assert_eq!(
            format_value(MeasureKind::Angle, MeasureUnit::Point, 90.0),
            "90.00°"
        );
    }

    #[test]
    fn precise_formatting_respects_zero_decimal_places() {
        assert_eq!(
            format_value_precise(MeasureKind::Length, MeasureUnit::Foot, 12.0, 0),
            "12 ft"
        );
        assert_eq!(
            format_value_precise(MeasureKind::Length, MeasureUnit::Foot, 12.4, 0),
            "12 ft"
        );
        assert_eq!(
            format_value_precise(MeasureKind::Length, MeasureUnit::Foot, 12.6, 0),
            "13 ft"
        );
    }

    #[test]
    fn precise_formatting_uses_requested_decimals() {
        assert_eq!(
            format_value_precise(MeasureKind::Length, MeasureUnit::Foot, 12.34, 1),
            "12.3 ft"
        );
        assert_eq!(
            format_value_precise(MeasureKind::Perimeter, MeasureUnit::Meter, 1.23456, 3),
            "1.235 m"
        );
        assert_eq!(
            format_value_precise(
                MeasureKind::Diameter,
                MeasureUnit::Millimeter,
                1.234_567_89,
                6
            ),
            "1.234568 mm"
        );
    }

    #[test]
    fn precise_formatting_clamps_precision_to_supported_range() {
        assert_eq!(DEFAULT_PRECISION, 3);
        assert_eq!(MAX_PRECISION, 6);
        assert_eq!(clamp_precision(0), 0);
        assert_eq!(clamp_precision(6), 6);
        assert_eq!(clamp_precision(7), 6);
        assert_eq!(clamp_precision(255), 6);
        assert_eq!(
            format_value_precise(MeasureKind::Length, MeasureUnit::Foot, 1.234_567_89, 99),
            "1.234568 ft"
        );
        assert_eq!(
            format_value_precise(MeasureKind::Length, MeasureUnit::Foot, 1.234_567_89, 99),
            format_value_precise(
                MeasureKind::Length,
                MeasureUnit::Foot,
                1.234_567_89,
                MAX_PRECISION
            )
        );
    }

    #[test]
    fn precise_area_uses_same_precision_and_squared_suffix() {
        assert_eq!(
            format_value_precise(MeasureKind::Area, MeasureUnit::Foot, 12.3456, 1),
            "12.3 ft²"
        );
        assert_eq!(
            format_value_precise(MeasureKind::Area, MeasureUnit::Foot, 12.0, 0),
            "12 ft²"
        );
    }

    #[test]
    fn precise_formatting_keeps_angle_and_count_fixed() {
        for precision in [0, 1, 3, 6, 99] {
            assert_eq!(
                format_value_precise(MeasureKind::Angle, MeasureUnit::Point, 90.0, precision),
                "90.00°"
            );
            assert_eq!(
                format_value_precise(MeasureKind::Count, MeasureUnit::Foot, 7.0, precision),
                "7 count"
            );
        }
    }

    #[test]
    fn default_formatting_matches_explicit_default_precision() {
        let cases = [
            (MeasureKind::Length, 12.3456),
            (MeasureKind::Area, 12.3456),
            (MeasureKind::Angle, 90.0),
            (MeasureKind::Count, 7.0),
        ];
        for (kind, value) in cases {
            assert_eq!(
                format_value(kind, MeasureUnit::Foot, value),
                format_value_precise(kind, MeasureUnit::Foot, value, DEFAULT_PRECISION)
            );
        }
    }

    #[test]
    fn calibrate_scale_maps_a_known_distance() {
        // A 72 pt segment known to be one foot: 1 in = 1 ft.
        let s = calibrate_scale(72.0, 1.0, MeasureUnit::Foot);
        assert_eq!(s.unit, MeasureUnit::Foot);
        assert!((s.units_per_point - 1.0 / 72.0).abs() < 1e-9);
        // A 720 pt wall now reads 10 ft.
        assert!((s.points_to_units(720.0) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn calibrate_scale_works_in_metric_units() {
        // A 100 pt segment known to be exactly two metres.
        let s = calibrate_scale(100.0, 2.0, MeasureUnit::Meter);
        assert!((s.points_to_units(100.0) - 2.0).abs() < 1e-9);
        assert!((s.units_to_points(2.0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn calibrate_scale_refuses_non_finite_inputs() {
        // A non-finite reference carries no ratio: fall back to the paper
        // scale instead of producing a NaN or infinite units_per_point.
        assert_eq!(
            calibrate_scale(f32::NAN, 10.0, MeasureUnit::Foot),
            Scale::PAPER
        );
        assert_eq!(
            calibrate_scale(72.0, f64::NAN, MeasureUnit::Foot),
            Scale::PAPER
        );
        assert_eq!(
            calibrate_scale(f32::INFINITY, 1.0, MeasureUnit::Foot),
            Scale::PAPER
        );
    }

    #[test]
    fn calibrate_scale_zero_length_falls_back_to_one_unit_per_point() {
        // Mirrors `Scale::from_ratio`: a zero-length reference would divide by
        // zero, so it degrades to 1 unit/pt in the requested unit. The UI
        // refuses such segments before the dialog opens; this is the net.
        let s = calibrate_scale(0.0, 10.0, MeasureUnit::Foot);
        assert_eq!(s.unit, MeasureUnit::Foot);
        assert!((s.units_per_point - 1.0).abs() < 1e-12);
    }

    #[test]
    fn calibrate_scale_ignores_signs() {
        // Distances are magnitudes; a reversed drag direction must not
        // produce a negative scale.
        let s = calibrate_scale(-72.0, -1.0, MeasureUnit::Foot);
        assert!((s.units_per_point - 1.0 / 72.0).abs() < 1e-9);
    }

    #[test]
    fn calibrate_scale_result_serde_roundtrip() {
        let s = calibrate_scale(72.0, 1.0, MeasureUnit::Foot);
        let json = serde_json::to_string(&s).expect("serialize");
        let back: Scale = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, s);
    }

    #[test]
    fn calibrated_scale_recomputes_existing_measurements() {
        // The full calibration write path: a markup measured under the old
        // scale reads a different value once the page is recalibrated, while
        // its stored raw points stay the source of truth.
        use crate::domain::{Markup, MarkupGeometry, MarkupKind, Pt};
        let mut m = Markup::new(
            0,
            MarkupKind::MeasureLength,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(720.0, 0.0),
            },
            "t",
        );
        m.recompute_measurement(Scale::PAPER);
        let before = m.measurement.expect("measured under paper scale");
        assert!((before.value - 720.0).abs() < 1e-9);

        m.recompute_measurement(calibrate_scale(72.0, 1.0, MeasureUnit::Foot));
        let after = m.measurement.expect("measured after calibration");
        assert!(
            (after.value - 10.0).abs() < 1e-6,
            "720 pt on a 72 pt = 1 ft page must read 10 ft, got {}",
            after.value
        );
        assert!((after.value_points - before.value_points).abs() < 1e-9);
    }
}
