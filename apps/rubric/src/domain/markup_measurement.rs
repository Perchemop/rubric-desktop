//! Measurement results attached to measure markups.
//!
//! A [`Measurement`] holds both a scaled value and the raw PDF-point value so
//! measurements can be recomputed loss-free when the page scale changes.

use serde::{Deserialize, Serialize};

use super::units::{self, MeasureKind, MeasureUnit};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measurement_label_honours_precision_and_default() {
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
        assert_eq!(
            angle.label_with(units::MeasureUnit::Point, 6),
            "90.00°"
        );
    }

    #[test]
    fn measurement_serde_roundtrip() {
        let m = Measurement {
            kind: MeasureKind::Area,
            value: 12.5,
            value_points: 12.5,
        };
        let json = serde_json::to_string(&m).expect("serialize");
        let back: Measurement = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(m, back);
    }
}
