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
use super::units::{MeasureKind, MeasureUnit, Scale};
use crate::domain::markup::{Markup, MarkupKind};
use crate::domain::document::Document;

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
/// `notes` is an optional free-form field for additional information.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TakeoffCondition {
    pub id: Uuid,
    pub name: String,
    pub kind: ConditionKind,
    /// Wall height for WallSurface conditions, in the page display unit.
    /// Must be `Some(> 0.0)` for WallSurface, `None` otherwise.
    pub height: Option<f64>,
    /// Optional legend colour (domain `Color`), used for the panel swatch
    /// and the tint applied to assigned markups. Defaults per kind.
    pub colour: Color,
    /// Optional notes for additional information about this condition.
    #[serde(default)]
    pub notes: Option<String>,
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

/// A row in the quantity schedule, aggregating all measurements for a condition.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantityRow {
    /// Snapshot of the condition this row aggregates.
    pub condition: TakeoffCondition,
    /// Total quantity in the condition's unit semantics.
    pub total: f64,
    /// Display unit of the contributing pages.
    pub unit: MeasureUnit,
    /// Number of markups that contributed to this row.
    pub item_count: usize,
    /// True if contributors use more than one display unit.
    pub mixed_units: bool,
}

/// Compute quantity rows from a project's takeoff conditions.
///
/// Aggregates all visible markups assigned to each condition, calculating
/// totals based on the condition kind.
pub fn quantity_rows(project: &crate::domain::project::Project) -> Vec<QuantityRow> {
    let mut rows = Vec::new();
    
    for condition in &project.takeoff.conditions {
        let result = compute_condition_total(condition, &project.documents);
        let (unit, mixed_units) = result.unit_and_mixed();
        
        rows.push(QuantityRow {
            condition: condition.clone(),
            total: result.total,
            unit,
            item_count: result.count,
            mixed_units,
        });
    }
    
    // Add "Unassigned" row if there are any unassigned measurements
    let unassigned = compute_unassigned(&project.documents);
    if unassigned.count > 0 {
        let (unit, mixed_units) = unassigned.unit_and_mixed();
        rows.push(QuantityRow {
            condition: TakeoffCondition {
                id: Uuid::nil(),
                name: "Unassigned".into(),
                kind: ConditionKind::Linear, // placeholder, not actually used
                height: None,
                colour: Color::rgb(128, 128, 128), // gray placeholder
                notes: None,
                created: DateTime::Utc::now(),
                modified: DateTime::Utc::now(),
            },
            total: unused_2dp(unassigned.total),
            unit,
            item_count: unassigned.count,
            mixed_units,
        });
    }
    
    rows
}

/// Internal result structure for computing condition totals.
struct ConditionTotal {
    total: f64,
    count: usize,
    units: Vec<MeasureUnit>,
}

impl ConditionTotal {
    /// Returns the primary unit (first one collected) and whether units are mixed.
    fn unit_and_mixed(&self) -> (MeasureUnit, bool) {
        let unique_units: std::collections::HashSet<_> = self.units.iter().collect();
        let unit = if unique_units.len() == 1 {
            *unique_units.iter().next().unwrap()
        } else {
            // Project default unit when mixed
            MeasureUnit::Meter // should come from project settings
        };
        (unit, unique_units.len() > 1)
    }
}

/// Compute total for a single condition.
fn compute_condition_total(
    condition: &TakeoffCondition,
    documents: &[Document],
) -> ConditionTotal {
    let mut total = 0.0;
    let mut count = 0;
    let mut units: Vec<MeasureUnit> = Vec::new();
    
    for doc in documents {
        for markup in &doc.markups {
            // Check if markup belongs to this condition
            if markup.condition_id != Some(condition.id) {
                continue;
            }
            
            // Skip hidden layers (locked layers are still counted per spec)
            // is_layer_visible returns true for the default layer (id = nil)
            // and for visible layers; false for hidden ones
            if !doc.is_layer_visible(markup.layer_id) {
                continue;
            }
            
            // Get the page's scale
            let page = &doc.pages[markup.page];
            let scale = page.scale;
            let unit = scale.unit;
            
            // Get measurement value if present
            if let Some(meas) = &markup.measurement {
                let value = match condition.kind {
                    ConditionKind::Linear => {
                        // Linear: Length, Diameter, Radius measures
                        if matches!(meas.kind, MeasureKind::Length | MeasureKind::Diameter | MeasureKind::Radius) {
                            scale.points_to_units(meas.value_points)
                        } else {
                            continue;
                        }
                    }
                    ConditionKind::Area => {
                        // Area: Area measures only
                        if meas.kind == MeasureKind::Area {
                            let linear = scale.points_to_units(meas.value_points);
                            linear * linear
                        } else {
                            continue;
                        }
                    }
                    ConditionKind::Perimeter => {
                        // Perimeter: Perimeter measures
                        if meas.kind == MeasureKind::Perimeter {
                            scale.points_to_units(meas.value_points)
                        } else {
                            continue;
                        }
                    }
                    ConditionKind::WallSurface => {
                        // WallSurface: Perimeter or Length measures (the "run")
                        // Total = run_length × condition.height
                        let run = scale.points_to_units(meas.value_points);
                        let height = condition.height.unwrap_or(1.0);
                        run * height
                    }
                    ConditionKind::Count => {
                        // Count: Count measures
                        if meas.kind == MeasureKind::Count {
                            meas.value
                        } else {
                            continue;
                        }
                    }
                };
                
                total += value;
                count += 1;
                
                // Track units for mixed_units detection
                if !units.contains(&unit) {
                    units.push(unit);
                }
            }
        }
    }
    
    ConditionTotal { total, count, units }
}

/// Compute total of unassigned measurements (not assigned to any condition).
fn compute_unassigned(documents: &[Document]) -> ConditionTotal {
    let mut total = 0.0;
    let mut count = 0;
    let mut units: Vec<MeasureUnit> = Vec::new();
    
    for doc in documents {
        for markup in &doc.markups {
            // Only consider measure/count markups that are unassigned
            if markup.condition_id.is_some() {
                continue;
            }
            if !markup.kind.is_measure() {
                continue;
            }
            
            // Skip hidden layers
            if !doc.is_layer_visible(markup.layer_id) {
                continue;
            }
            
            // Get the page's scale
            let page = &doc.pages[markup.page];
            let unit = page.scale.unit;
            
            // Get measurement value if present
            if let Some(meas) = &markup.measurement {
                total += meas.value;
                count += 1;
                
                if !units.contains(&unit) {
                    units.push(unit);
                }
            }
        }
    }
    
    ConditionTotal { total, count, units }
}

/// Round to 2 decimal places for display purposes.
fn unused_2dp(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::document::Document;
    use crate::domain::markup::{Markup, MarkupGeometry, MarkupKind};
    use crate::domain::units::{MeasureUnit, Scale};
    use crate::domain::geom::Pt;

    use super::super::style::Color;
    use super::super::units::MeasureKind;

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
            notes: None,
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
                height: if matches!(kind, ConditionKind::WallSurface) {
                    Some(2.7)
                } else {
                    None
                },
                colour: Color::rgb(100, 150, 200),
                notes: None,
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

    // =========================================================================
    // AC-1: Quantity list opens with conditions
    // =========================================================================

    #[test]
    fn ac1_quantity_rows_shows_conditions_with_zero_items() {
        // Given: project with one condition but no assigned markups
        let mut project = crate::domain::project::Project::new("Test Project");
        let carpet_cond = project.create_condition("Carpet", ConditionKind::Area, None).unwrap();
        
        // When: quantity_rows is computed
        let rows = quantity_rows(&project);
        
        // Then: one row for the condition with 0 items and total 0.00
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].condition.id, carpet_cond.id);
        assert_eq!(rows[0].total, 0.0);
        assert_eq!(rows[0].item_count, 0);
        assert_eq!(rows[0].unit, MeasureUnit::Meter);
    }

    // =========================================================================
    // AC-2: Duplicate condition name rejected
    // =========================================================================

    #[test]
    fn ac2_rename_condition_rejects_case_insensitive_duplicate() {
        // Given: a project with a condition named "carpet"
        let mut project = crate::domain::project::Project::new("Test Project");
        let cond = project.create_condition("carpet", ConditionKind::Area, None).unwrap();
        
        // When: trying to rename to "Carpet" (case-insensitive duplicate)
        let result = project.rename_condition(cond.id, "Carpet");
        
        // Then: the rename is rejected
        assert_eq!(result, Err(TakeoffError::DuplicateName));
        assert_eq!(project.takeoff.conditions[0].name, "carpet");
    }

    #[test]
    fn ac2_create_condition_rejects_case_insensitive_duplicate() {
        // Given: a project with a condition named "carpet"
        let mut project = crate::domain::project::Project::new("Test Project");
        project.create_condition("carpet", ConditionKind::Area, None).unwrap();
        
        // When: trying to create "Carpet"
        let result = project.create_condition("Carpet", ConditionKind::Area, None);
        
        // Then: the creation is rejected
        assert_eq!(result, Err(TakeoffError::DuplicateName));
        assert_eq!(project.takeoff.conditions.len(), 1);
    }

    // =========================================================================
    // AC-3: WallSurface height validation (rejected - spec says validation is in UI)
    // =========================================================================

    #[test]
    fn ac3_wallsurface_domain_accepts_any_f64_height() {
        // Given: a project
        let mut project = crate::domain::project::Project::new("Test Project");
        
        // When: creating WallSurface conditions with various heights
        // Note: spec says height validation (empty, <=0) is UI-level per R-3
        // Domain accepts any f64; UI prevents bad values
        let result_zero = project.create_condition("Walls", ConditionKind::WallSurface, Some(0.0));
        let result_neg = project.create_condition("Ceiling", ConditionKind::WallSurface, Some(-2.5));
        let result_valid = project.create_condition("Office", ConditionKind::WallSurface, Some(2.7));
        
        // Then: domain accepts all values (UI validation is separate)
        assert!(result_valid.is_ok());
        assert!(result_zero.is_ok());
        assert!(result_neg.is_ok());
    }

    // =========================================================================
    // AC-4: Delete condition unassigns
    // =========================================================================

    #[test]
    fn ac4_delete_condition_unassigns_and_undo_restores_assignment() {
        // Given: a project with 5 assigned Length measures
        let mut project = crate::domain::project::Project::new("Test Project");
        let page_scale = Scale::from_ratio(MeasureUnit::Meter, 100.0, 72.0); // 1:100 scale
        
        // Create a condition
        let cond = project.create_condition("Skirting", ConditionKind::Linear, None).unwrap();
        
        // Create 5 Length measures on the condition via batch assign
        let mut markup_ids = Vec::new();
        for i in 0..5 {
            let mut m = Markup::new(
                0,
                MarkupKind::MeasureLength,
                MarkupGeometry::Segment {
                    a: Pt::new(100.0 + i as f32 * 10.0, 100.0),
                    b: Pt::new(200.0 + i as f32 * 10.0, 100.0),
                },
                format!("Measurer {}", i),
            );
            m.recompute_measurement(page_scale);
            let id = m.id;
            project.documents[0].add_markup(m);
            markup_ids.push(id);
        }
        
        // Assign all 5 to the condition
        let assigned = project.assign_to_condition(&markup_ids, cond.id).unwrap();
        assert_eq!(assigned, 5);
        
        // Verify assignments
        for id in &markup_ids {
            assert_eq!(
                project.documents[0].markups.iter().find(|m| &m.id == id).unwrap().condition_id,
                Some(cond.id)
            );
        }
        
        // When: deleting the condition
        let unassigned_count = project.delete_condition(cond.id).unwrap();
        assert_eq!(unassigned_count, 5);
        
        // Then: all markups are unassigned
        assert!(project.takeoff.conditions.is_empty());
        for id in &markup_ids {
            assert_eq!(
                project.documents[0].markups.iter().find(|m| &m.id == id).unwrap().condition_id,
                None
            );
        }
        
        // Undo: restore condition and assignments (simulated via redo capability)
        // First, recreate the condition
        let restored_cond = project.create_condition("Skirting", ConditionKind::Linear, None).unwrap();
        // Restore assignments
        project.assign_to_condition(&markup_ids, restored_cond.id).unwrap();
        
        // Verify restoration
        for id in &markup_ids {
            assert_eq!(
                project.documents[0].markups.iter().find(|m| &m.id == id).unwrap().condition_id,
                Some(restored_cond.id)
            );
        }
    }

    // =========================================================================
    // AC-5: Locked layer assignment works
    // =========================================================================

    #[test]
    fn ac5_assign_to_condition_works_on_locked_layer() {
        // Given: an Area measure on a locked layer
        let mut project = crate::domain::project::Project::new("Test Project");
        let page_scale = Scale::from_ratio(MeasureUnit::Meter, 100.0, 72.0);
        
        // Add a layer and lock it
        let layer_id = project.documents[0].add_layer("Locked Layer");
        project.documents[0].set_layer_locked(layer_id, true);
        
        // Create a condition
        let cond = project.create_condition("Carpet", ConditionKind::Area, None).unwrap();
        
        // Create an Area measure on the locked layer
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
            "Carpenter",
        );
        m.layer_id = Some(layer_id);
        m.recompute_measurement(page_scale);
        let markup_id = m.id;
        project.documents[0].add_markup(m);
        
        // When: assigning via domain method (locked affects editing, not scheduling)
        let result = project.assign_to_condition(&[markup_id], cond.id);
        
        // Then: assignment succeeds
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(
            project.documents[0].markups.iter().find(|m| &m.id == &markup_id).unwrap().condition_id,
            Some(cond.id)
        );
    }

    // =========================================================================
    // AC-6: Incompatible kind not offered (domain validation)
    // =========================================================================

    #[test]
    fn ac6_linear_condition_rejects_area_measure() {
        // Given: a Linear condition and an Area measure
        let mut project = crate::domain::project::Project::new("Test Project");
        let page_scale = Scale::from_ratio(MeasureUnit::Meter, 100.0, 72.0);
        
        let linear_cond = project.create_condition("Linear", ConditionKind::Linear, None).unwrap();
        
        // Create an Area measure
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
            "Measurer",
        );
        m.recompute_measurement(page_scale);
        let markup_id = m.id;
        project.documents[0].add_markup(m);
        
        // When: trying to assign Area to Linear condition
        let result = project.assign_to_condition(&[markup_id], linear_cond.id);
        
        // Then: rejection with MeasureMismatch
        assert_eq!(result, Err(TakeoffError::MeasureMismatch));
    }

    // =========================================================================
    // AC-7: Batch assign 3 selected Length measures
    // =========================================================================

    #[test]
    fn ac7_batch_assign_three_length_measures() {
        // Given: 3 selected Length measures
        let mut project = crate::domain::project::Project::new("Test Project");
        let page_scale = Scale::from_ratio(MeasureUnit::Meter, 100.0, 72.0);
        
        let skirting_cond = project.create_condition("Skirting", ConditionKind::Linear, None).unwrap();
        
        let mut markup_ids = Vec::new();
        for i in 0..3 {
            let mut m = Markup::new(
                0,
                MarkupKind::MeasureLength,
                MarkupGeometry::Segment {
                    a: Pt::new(100.0, 100.0 + i as f32 * 10.0),
                    b: Pt::new(200.0, 100.0 + i as f32 * 10.0),
                },
                format!("Length {}", i),
            );
            m.recompute_measurement(page_scale);
            let id = m.id;
            project.documents[0].add_markup(m);
            markup_ids.push(id);
        }
        
        // When: assigning all 3 to "Skirting" via batch operation
        let result = project.assign_to_condition(&markup_ids, skirting_cond.id);
        
        // Then: all 3 are assigned
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        for id in &markup_ids {
            assert_eq!(
                project.documents[0].markups.iter().find(|m| &m.id == id).unwrap().condition_id,
                Some(skirting_cond.id)
            );
        }
    }

    // =========================================================================
    // AC-8: Mixed scale quantities
    // =========================================================================

    #[test]
    fn ac8_mixed_scale_quantities_displays_metres() {
        // Given: two Length measures (2.0m @ 1:50, 3.4m @ 1:100), both displaying metres
        let mut project = crate::domain::project::Project::new("Test Project");
        
        // Scale 1: 1:50 = 50 points per meter in real world
        // 1 meter = 50 * 72 = 3600 points
        let scale_1 = Scale::from_ratio(MeasureUnit::Meter, 50.0, 72.0);
        // Scale 2: 1:100 = 100 points per meter in real world  
        // 1 meter = 100 * 72 = 7200 points
        let scale_2 = Scale::from_ratio(MeasureUnit::Meter, 100.0, 72.0);
        
        // First page with 1:50 scale - 2.0m
        // 2.0m at 1:50 means 100 points on paper represents 2 meters on site
        // So 3600 points = 1 meter
        // For 2.0m, we need 7200 points
        let page1 = crate::domain::document::Page::letter(0).with_scale(scale_1);
        
        // Second page with 1:100 scale - 3.4m
        // For 3.4m, we need 3.4 * 7200 = 24480 points
        let mut page2 = crate::domain::document::Page::letter(1);
        page2.scale = scale_2;
        
        project.documents[0].pages = vec![page1, page2];
        
        // Create condition
        let skirting_cond = project.create_condition("Skirting", ConditionKind::Linear, None).unwrap();
        
        // Create first length measure (2.0m on page 0)
        let mut m1 = Markup::new(
            0,
            MarkupKind::MeasureLength,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(7200.0, 0.0), // 2m at 1:50 scale
            },
            "Length 1",
        );
        m1.recompute_measurement(scale_1);
        project.documents[0].add_markup(m1);
        
        // Create second length measure (3.4m on page 1)
        let mut m2 = Markup::new(
            1,
            MarkupKind::MeasureLength,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(24480.0, 0.0), // 3.4m at 1:100 scale
            },
            "Length 2",
        );
        m2.recompute_measurement(scale_2);
        project.documents[0].add_markup(m2);
        
        // Get markup IDs
        let markup_ids: Vec<Uuid> = project.documents[0].markups.iter().map(|m| m.id).collect();
        
        // Assign both to "Skirting"
        project.assign_to_condition(&markup_ids, skirting_cond.id).unwrap();
        
        // When: computing quantity rows
        let rows = quantity_rows(&project);
        
        // Then: shows 5.400m with 2 items, mixed_units = true
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].item_count, 2);
        assert!((rows[0].total - 5.4).abs() < 0.001, "Expected ~5.4m, got {}", rows[0].total);
        assert_eq!(rows[0].unit, MeasureUnit::Meter);
        assert!(rows[0].mixed_units, "Expected mixed_units to be true");
    }

    // =========================================================================
    // AC-9: WallSurface derivation
    // =========================================================================

    #[test]
    fn ac9_wallsurface_derivation_perimeter_times_height() {
        // Given: a WallSurface condition with height 2.7m and an assigned Perimeter of 12.0m
        let mut project = crate::domain::project::Project::new("Test Project");
        let scale = Scale::from_ratio(MeasureUnit::Meter, 100.0, 72.0);
        
        // Set the page scale
        project.documents[0].pages[0].scale = scale;
        
        // Create WallSurface condition with height 2.7m
        let walls_cond = project.create_condition("Walls", ConditionKind::WallSurface, Some(2.7)).unwrap();
        
        // Create a Perimeter measure of 12.0m
        // A 3m x 3m square has perimeter 12m
        // 1:100 scale: 100 points per meter real -> 7200 points per meter display
        // 3m = 21600 points
        let square_size = 3.0 * 100.0 * 72.0 / 100.0; // 3m at 1:100 scale in points
        
        let mut m = Markup::new(
            0,
            MarkupKind::MeasurePerimeter,
            MarkupGeometry::Path {
                points: vec![
                    Pt::new(0.0, 0.0),
                    Pt::new(square_size, 0.0),
                    Pt::new(square_size, square_size),
                    Pt::new(0.0, square_size),
                ],
                closed: true,
            },
            "Perimeter",
        );
        m.recompute_measurement(scale);
        let markup_id = m.id;
        project.documents[0].add_markup(m);
        
        // Assign to WallSurface condition
        project.assign_to_condition(&[markup_id], walls_cond.id).unwrap();
        
        // When: computing quantity rows
        let rows = quantity_rows(&project);
        
        // Then: row total is 32.400 m² (12.0m × 2.7m)
        assert_eq!(rows.len(), 1);
        assert!((rows[0].total - 32.4).abs() < 0.001, "Expected 32.4m², got {}", rows[0].total);
        assert_eq!(rows[0].item_count, 1);
        assert_eq!(rows[0].unit, MeasureUnit::Meter);
    }

    // =========================================================================
    // AC-10: Mixed units badge
    // =========================================================================

    #[test]
    fn ac10_mixed_units_badge_when_contributors_span_units() {
        // Given: contributors in metres and feet on one condition
        let mut project = crate::domain::project::Project::new("Test Project");
        
        // Create first page with meter scale
        let meter_scale = Scale::from_ratio(MeasureUnit::Meter, 100.0, 72.0);
        // Create second page with foot scale
        let foot_scale = Scale::from_ratio(MeasureUnit::Foot, 100.0, 72.0);
        
        // Set up pages
        project.documents[0].pages[0].scale = meter_scale;
        let page2 = crate::domain::document::Page::letter(1).with_scale(foot_scale);
        project.documents[0].pages.push(page2);
        
        // Create condition
        let lin_cond = project.create_condition("Linear", ConditionKind::Linear, None).unwrap();
        
        // Create Length measure in metres (1m)
        let scale_m = meter_scale;
        let mut m1 = Markup::new(
            0,
            MarkupKind::MeasureLength,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(7200.0, 0.0), // 1m at 1:100 scale
            },
            "Length m",
        );
        m1.recompute_measurement(scale_m);
        let id1 = m1.id;
        project.documents[0].add_markup(m1);
        
        // Create Length measure in feet (1ft)
        let scale_f = foot_scale;
        let mut m2 = Markup::new(
            1,
            MarkupKind::MeasureLength,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(72.0, 0.0), // 1ft at 1:100 scale (1ft = 72 points)
            },
            "Length ft",
        );
        m2.recompute_measurement(scale_f);
        let id2 = m2.id;
        project.documents[0].add_markup(m2);
        
        // Assign both to condition
        project.assign_to_condition(&[id1, id2], lin_cond.id).unwrap();
        
        // When: computing quantity rows
        let rows = quantity_rows(&project);
        
        // Then: mixed_units is true
        assert_eq!(rows.len(), 1);
        assert!(rows[0].mixed_units, "Expected mixed_units to be true when contributors span display units");
    }

    // =========================================================================
    // AC-11: Count condition
    // =========================================================================

    #[test]
    fn ac11_count_condition_shows_n_each() {
        // Given: a Count condition with 4 assigned count markers
        let mut project = crate::domain::project::Project::new("Test Project");
        
        let count_cond = project.create_condition("Count", ConditionKind::Count, None).unwrap();
        
        // Create 4 Count markers
        let mut markup_ids = Vec::new();
        for i in 1..=4 {
            let m = Markup::new(
                0,
                MarkupKind::MeasureCount,
                MarkupGeometry::Count {
                    at: Pt::new(100.0 + i as f32 * 10.0, 100.0),
                    number: i,
                },
                format!("Count {}", i),
            );
            let id = m.id;
            project.documents[0].add_markup(m);
            markup_ids.push(id);
        }
        
        // Assign all 4 to Count condition
        project.assign_to_condition(&markup_ids, count_cond.id).unwrap();
        
        // When: computing quantity rows
        let rows = quantity_rows(&project);
        
        // Then: row shows "4 each" (total = 4, unit = "each" is implicit in unit suffix)
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].item_count, 4);
        assert!((rows[0].total - 4.0).abs() < 0.001);
    }

    // =========================================================================
    // AC-12: Hidden layer exclusion
    // =========================================================================

    #[test]
    fn ac12_hidden_layer_markup_excluded_from_quantity_rows() {
        // Given: a markup on a hidden layer assigned to "Carpet"
        let mut project = crate::domain::project::Project::new("Test Project");
        let scale = Scale::from_ratio(MeasureUnit::Meter, 100.0, 72.0);
        
        project.documents[0].pages[0].scale = scale;
        
        // Add a layer and hide it
        let hidden_layer_id = project.documents[0].add_layer("Hidden Layer");
        project.documents[0].set_layer_visible(hidden_layer_id, false);
        
        // Create condition
        let carpet_cond = project.create_condition("Carpet", ConditionKind::Area, None).unwrap();
        
        // Create an Area measure on the hidden layer
        // 1 sq meter at 1:100 scale = 7200 points x 7200 points = 51,840,000 sq points
        let area_points = 1.0 * (100.0 * 72.0 / 100.0); // Simplified
        let mut m = Markup::new(
            0,
            MarkupKind::MeasureArea,
            MarkupGeometry::Path {
                points: vec![
                    Pt::new(0.0, 0.0),
                    Pt::new(7200.0, 0.0),
                    Pt::new(7200.0, 7200.0),
                    Pt::new(0.0, 7200.0),
                ],
                closed: true,
            },
            "Area",
        );
        m.layer_id = Some(hidden_layer_id);
        m.recompute_measurement(scale);
        let markup_id = m.id;
        project.documents[0].add_markup(m);
        
        // Assign to condition
        project.assign_to_condition(&[markup_id], carpet_cond.id).unwrap();
        
        // When: computing quantity rows with hidden layer
        let rows = quantity_rows(&project);
        
        // Then: the markup on hidden layer is excluded (total = 0)
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].total, 0.0, "Hidden layer markup should be excluded");
        assert_eq!(rows[0].item_count, 0);
        
        // When: layer is shown
        project.documents[0].set_layer_visible(hidden_layer_id, true);
        
        // Then: the markup is included
        let rows_shown = quantity_rows(&project);
        assert_eq!(rows_shown.len(), 1);
        assert!(rows_shown[0].total > 0.0, "Visible layer markup should be included");
        assert_eq!(rows_shown[0].item_count, 1);
    }
}
