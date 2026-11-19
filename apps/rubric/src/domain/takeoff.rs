//! Takeoff conditions and quantity scheduling.
//! Pure domain types: no egui imports, fully serde, head-testable.
//!
//! Defines `TakeoffCondition`, `ConditionKind`, and `TakeoffSettings`.
//! Conditions group measurements (lengths, areas, perimeters, wall surfaces, counts)
//! for a quantity schedule that can be exported as CSV.

use thiserror::Error;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::style::Color;

/// Kind of takeoff condition — determines which measure families may be assigned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionKind {
    /// Sum of lengths of assigned Length/Diameter/Radius measures.
    Linear,
    /// Sum of areas of assigned Area measures.
    Area,
    /// Sum of perimeters of assigned Perimeter measures.
    Perimeter,
    /// Derived: sum of (assigned run length) × height.
    /// WallSurface accepts Perimeter and Length measures as "run".
    WallSurface,
    /// Number of assigned Count markers.
    Count,
}

impl ConditionKind {
    /// Human-readable label shown in the UI (title case, matching spec).
    pub fn label(self) -> &'static str {
        match self {
            ConditionKind::Linear => "Linear",
            ConditionKind::Area => "Area",
            ConditionKind::Perimeter => "Perimeter",
            ConditionKind::WallSurface => "Wall surface",
            ConditionKind::Count => "Count",
        }
    }

    /// Square‑marker suffix for area‑type conditions.
    ///
    /// The actual display unit (m, ft, in, etc.) comes from the page's
    /// [`crate::domain::units::Scale`]; this method returns only the
    /// “²” marker that indicates the value is an area, or "" for linear/perimeter/count.
    pub fn unit_suffix(self) -> &'static str {
        match self {
            ConditionKind::Area | ConditionKind::WallSurface => "²",
            ConditionKind::Linear | ConditionKind::Perimeter | ConditionKind::Count => "",
        }
    }

    /// All condition kinds, in declaration order.
    pub const ALL: [ConditionKind; 5] = [
        ConditionKind::Linear,
        ConditionKind::Area,
        ConditionKind::Perimeter,
        ConditionKind::WallSurface,
        ConditionKind::Count,
    ];
}

/// A named condition that groups measurements for a quantity schedule.
///
/// `height` is `Some(>0.0)` only for `WallSurface` kinds; it is `None` for all other kinds.
/// The colour is used for the panel swatch and the tint applied to assigned markups.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TakeoffCondition {
    pub id: Uuid,
    pub name: String,
    pub kind: ConditionKind,
    /// Wall height for WallSurface conditions, in the page display unit.
    /// Must be `Some(> 0.0)` for WallSurface, `None` otherwise.
    pub height: Option<f64>,
    pub colour: Color,
    pub created: DateTime<chrono::Utc>,
    pub modified: DateTime<chrono::Utc>,
}

/// Serialised takeoff settings stored on the project.
///
/// The `conditions` field defaults to an empty vector, so legacy `.rubric.json`
/// files that omit the takeoff section load with no conditions (roundtrip invariant).
/// Errors raised by takeoff condition operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TakeoffError {
    #[error("condition name cannot be empty")]
    EmptyName,
    #[error("a condition with that name already exists (case-insensitive)")]
    DuplicateName,
    #[error("no condition found with id {0}")]
    NotFound(uuid::Uuid),
    #[error("measure kind does not match condition kind")]
    MeasureMismatch,
    #[error("condition cap (128) reached")]
    CapReached,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TakeoffSettings {
    #[serde(default)]
    pub conditions: Vec<TakeoffCondition>,
}

impl Default for TakeoffSettings {
    fn default() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::units::MeasureKind;

    #[test]
    fn condition_kind_all_contains_five_variants_exactly_once() {
        assert_eq!(ConditionKind::ALL.len(), 5);
        for (i, kind) in ConditionKind::ALL.iter().enumerate() {
            for other in ConditionKind::ALL.iter().skip(i + 1) {
                assert_ne!(kind, other, "duplicate variant in ConditionKind::ALL");
            }
        }
    }

    #[test]
    fn condition_kind_labels() {
        assert_eq!(ConditionKind::Linear.label(), "Linear");
        assert_eq!(ConditionKind::Area.label(), "Area");
        assert_eq!(ConditionKind::Perimeter.label(), "Perimeter");
        assert_eq!(ConditionKind::WallSurface.label(), "Wall surface");
        assert_eq!(ConditionKind::Count.label(), "Count");
    }

    #[test]
    fn condition_kind_unit_suffix() {
        assert_eq!(ConditionKind::Linear.unit_suffix(), "");
        assert_eq!(ConditionKind::Area.unit_suffix(), "²");
        assert_eq!(ConditionKind::Perimeter.unit_suffix(), "");
        assert_eq!(ConditionKind::WallSurface.unit_suffix(), "²");
        assert_eq!(ConditionKind::Count.unit_suffix(), "");
    }

    #[test]
    fn condition_kind_serde_snake_case() {
        let json = serde_json::to_string(&ConditionKind::Linear).expect("serialize");
        assert_eq!(json, "\"linear\"");
        let back: ConditionKind = serde_json::from_str("\"area\"").expect("deserialize");
        assert_eq!(back, ConditionKind::Area);
    }

    #[test]
    fn takeoff_condition_has_required_fields() {
        let cond = TakeoffCondition {
            id: Uuid::new_v4(),
            name: "Carpet".into(),
            kind: ConditionKind::Area,
            height: None,
            colour: Color::rgb(100, 150, 200),
            created: chrono::Utc::now(),
            modified: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&cond).expect("serialize condition");
        let back: TakeoffCondition = serde_json::from_str(&json).expect("deserialize condition");
        assert_eq!(back.name, "Carpet");
        assert_eq!(back.kind, ConditionKind::Area);
        assert_eq!(back.height, None);
    }

    #[test]
    fn takeoff_settings_default_is_empty() {
        let settings = TakeoffSettings::default();
        assert!(settings.conditions.is_empty());
    }

    #[test]
    fn takeoff_settings_serde_roundtrip_all_kinds() {
        let mut settings = TakeoffSettings::default();
        for kind in ConditionKind::ALL {
            settings.conditions.push(TakeoffCondition {
                id: Uuid::new_v4(),
                name: kind.label().into(),
                kind,
                height: if matches!(kind, ConditionKind::Area | ConditionKind::WallSurface) {
                    Some(3.5)
                } else {
                    None
                },
                colour: Color::rgb(100, 150, 200),
                created: chrono::Utc::now(),
                modified: chrono::Utc::now(),
            });
        }
        let json = serde_json::to_string(&settings).expect("serialize settings");
        let back: TakeoffSettings = serde_json::from_str(&json).expect("deserialize settings");
        assert_eq!(back.conditions.len(), 5);
        for (i, cond) in back.conditions.iter().enumerate() {
            assert_eq!(cond.kind, ConditionKind::ALL[i]);
        }
    }

    /// Legacy project (no takeoff field) loads with empty conditions.
    #[test]
    fn legacy_project_loads_empty_takeoff() {
        let json = r#"{
            "id": "a0eebc99-9c0b-4ef8-bb5d-3c3bd1e16bac",
            "name": "Plan",
            "source_pdf": null,
            "documents": [{"id": "doc1", "title": "Plan", "pages": []}],
            "active_document": 0,
            "tool_sets": [],
            "settings": {"author": "Reviewer", "auto_save": false, "auto_save_interval_secs": 60, "ui_scale": 1.0, "show_grid": false, "show_measurements": true, "precision": 3},
            "created": "2026-01-01T00:00:00Z",
            "modified": "2026-01-01T00:00:00Z"
        }"#;
        let project: crate::domain::project::Project = serde_json::from_str(json).expect("deserialize legacy project");
        assert_eq!(project.takeoff.conditions.len(), 0);
    }
}
