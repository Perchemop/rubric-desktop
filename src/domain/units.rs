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

    pub fn points_to_units(&self, points: f64) -> f64 {
        points * self.units_per_point
    }

    pub fn units_to_points(&self, units: f64) -> f64 {
        units / self.units_per_point
    }

    /// Human readable description, e.g. `1 in = 10 ft`.
    pub fn describe(&self) -> String {
        let one_point_in_units = self.units_per_point;
        let per_unit = 1.0 / one_point_in_units; // points per real unit
        let anchor = self.unit.points_per_unit();
        // How many real units does one "anchor" (e.g. one inch) represent?
        let real = self.points_to_units(anchor);
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

/// Format a raw numeric measurement with the unit's suffix.
pub fn format_value(kind: MeasureKind, unit: MeasureUnit, value: f64) -> String {
    if kind == MeasureKind::Angle {
        return format!("{value:.2}°");
    }
    if kind == MeasureKind::Count {
        return format!("{value:.0} count");
    }
    let suffix = unit.suffix();
    if kind.is_area() {
        format!("{value:.3} {suffix}²")
    } else {
        format!("{value:.3} {suffix}")
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
}
