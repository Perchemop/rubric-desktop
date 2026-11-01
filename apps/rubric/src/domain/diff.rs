//! Document/project diff for markup changes.

use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::document::Document;
use super::markup::{Markup, MarkupGeometry, MarkupKind, ReviewStatus};
use super::project::Project;
use super::style::MarkupStyle;

/// Compact serialisable snapshot of a markup for diffing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkupSummaryRow {
    pub id: Uuid,
    pub page: usize,
    /// Layer the markup belongs to (`None` = the document's default layer).
    ///
    /// Serde-defaulted so snapshots saved before layers existed still load,
    /// and compared by `rows_equal` so moving a markup between layers is
    /// reported as a change rather than reading as "no change".
    #[serde(default)]
    pub layer_id: Option<Uuid>,
    pub kind: MarkupKind,
    pub author: String,
    pub subject: String,
    pub status: ReviewStatus,
    pub style: MarkupStyle,
    pub locked: bool,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub geometry_fingerprint: String,
}

impl MarkupSummaryRow {
    pub fn from_markup(markup: &Markup) -> Self {
        Self {
            id: markup.id,
            page: markup.page,
            layer_id: markup.layer_id,
            kind: markup.kind,
            author: markup.author.clone(),
            subject: markup.subject.clone(),
            status: markup.status,
            style: markup.style,
            locked: markup.locked,
            created: markup.created,
            modified: markup.modified,
            geometry_fingerprint: geometry_fingerprint(markup),
        }
    }
}

/// A changed markup with before/after snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkupChange {
    pub id: Uuid,
    pub before: Option<MarkupSummaryRow>,
    pub after: Option<MarkupSummaryRow>,
}

/// Result of a markup diff.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DiffResult {
    pub added: Vec<MarkupSummaryRow>,
    pub removed: Vec<MarkupSummaryRow>,
    pub changed: Vec<MarkupChange>,
}

impl DiffResult {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    pub fn total_changes(&self) -> usize {
        self.added.len() + self.removed.len() + self.changed.len()
    }

    pub fn added_by_page(&self) -> BTreeMap<usize, Vec<MarkupSummaryRow>> {
        let mut map: BTreeMap<usize, Vec<MarkupSummaryRow>> = BTreeMap::new();
        for row in &self.added {
            map.entry(row.page).or_default().push(row.clone());
        }
        map
    }

    pub fn removed_by_page(&self) -> BTreeMap<usize, Vec<MarkupSummaryRow>> {
        let mut map: BTreeMap<usize, Vec<MarkupSummaryRow>> = BTreeMap::new();
        for row in &self.removed {
            map.entry(row.page).or_default().push(row.clone());
        }
        map
    }

    pub fn changed_by_page(&self) -> BTreeMap<usize, Vec<MarkupChange>> {
        let mut map: BTreeMap<usize, Vec<MarkupChange>> = BTreeMap::new();
        for change in &self.changed {
            let page = change
                .after
                .as_ref()
                .map(|r| r.page)
                .or_else(|| change.before.as_ref().map(|r| r.page))
                .unwrap_or(0);
            map.entry(page).or_default().push(change.clone());
        }
        map
    }
}

/// Deterministic fingerprint of geometry (rounded to 2 dp).
fn geometry_fingerprint(markup: &Markup) -> String {
    geometry_fingerprint_of(&markup.geometry)
}

fn geometry_fingerprint_of(g: &MarkupGeometry) -> String {
    use super::geom::Pt;
    let r = |v: f32| format!("{v:.2}");
    let p = |pt: &Pt| format!("{},{}", r(pt.x), r(pt.y));
    match g {
        MarkupGeometry::Bounds(rect) => {
            format!(
                "bounds:{}:{}:{}:{}",
                r(rect.x),
                r(rect.y),
                r(rect.w),
                r(rect.h)
            )
        }
        MarkupGeometry::Segment { a, b } => format!("segment:{}:{}", p(a), p(b)),
        MarkupGeometry::Path { points, closed } => {
            let pts = points.iter().map(p).collect::<Vec<_>>().join(";");
            format!("path:{}:{}", closed, pts)
        }
        MarkupGeometry::Ink { strokes } => {
            let strokes_s = strokes
                .iter()
                .map(|s| s.iter().map(p).collect::<Vec<_>>().join(";"))
                .collect::<Vec<_>>()
                .join("|");
            format!("ink:{}", strokes_s)
        }
        MarkupGeometry::Text { at, wrap, text } => {
            format!("text:{}:{}:{}", p(at), r(*wrap), text)
        }
        MarkupGeometry::Callout { rect, tail, text } => {
            format!(
                "callout:{}:{}:{}:{}:{}:{}",
                r(rect.x),
                r(rect.y),
                r(rect.w),
                r(rect.h),
                p(tail),
                text
            )
        }
        MarkupGeometry::Angle { a, vertex, b } => format!("angle:{}:{}:{}", p(a), p(vertex), p(b)),
        MarkupGeometry::Count { at, number } => format!("count:{}:{}", p(at), number),
    }
}

fn rows_equal(a: &MarkupSummaryRow, b: &MarkupSummaryRow) -> bool {
    a.page == b.page
        && a.layer_id == b.layer_id
        && a.kind == b.kind
        && a.status == b.status
        && a.style == b.style
        && a.locked == b.locked
        && a.author == b.author
        && a.subject == b.subject
        && a.geometry_fingerprint == b.geometry_fingerprint
}

/// Diff two documents by markup id.
pub fn diff_documents(a: &Document, b: &Document) -> DiffResult {
    let a_map: BTreeMap<Uuid, &Markup> = a.markups.iter().map(|m| (m.id, m)).collect();
    let b_map: BTreeMap<Uuid, &Markup> = b.markups.iter().map(|m| (m.id, m)).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    // Markups are expected to have unique ids (`Uuid`); a malformed document
    // with duplicates is diffed once per id, not once per duplicate row.
    let mut seen_added = HashSet::new();
    for markup in &b.markups {
        if !a_map.contains_key(&markup.id) && seen_added.insert(markup.id) {
            added.push(MarkupSummaryRow::from_markup(markup));
        }
    }

    let mut seen_removed = HashSet::new();
    for markup in &a.markups {
        if !b_map.contains_key(&markup.id) && seen_removed.insert(markup.id) {
            removed.push(MarkupSummaryRow::from_markup(markup));
        }
    }

    let mut seen_changed = HashSet::new();
    for markup_a in &a.markups {
        if !seen_changed.insert(markup_a.id) {
            continue;
        }
        if let Some(markup_b) = b_map.get(&markup_a.id) {
            let row_a = MarkupSummaryRow::from_markup(markup_a);
            let row_b = MarkupSummaryRow::from_markup(markup_b);
            if !rows_equal(&row_a, &row_b) {
                changed.push(MarkupChange {
                    id: markup_a.id,
                    before: Some(row_a),
                    after: Some(row_b),
                });
            }
        }
    }

    DiffResult {
        added,
        removed,
        changed,
    }
}

/// Diff two projects by their active documents.
///
/// A project without an active document is treated as having no markups, so
/// an asymmetric comparison surfaces the populated side's markups as
/// added/removed rather than reporting a misleading "no changes".
pub fn diff_projects(a: &Project, b: &Project) -> DiffResult {
    let empty = Document::empty("(none)");
    match (a.active(), b.active()) {
        (Some(doc_a), Some(doc_b)) => diff_documents(doc_a, doc_b),
        (None, Some(doc_b)) => diff_documents(&empty, doc_b),
        (Some(doc_a), None) => diff_documents(doc_a, &empty),
        (None, None) => DiffResult::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::document::{Document, Page};
    use crate::domain::geom::{Pt, Rect};
    use crate::domain::markup::{Markup, MarkupGeometry, MarkupKind, ReviewStatus};
    use crate::domain::project::Project;
    use crate::domain::style::palette;

    fn make_markup(
        id: u128,
        page: usize,
        kind: MarkupKind,
        author: &str,
        subject: &str,
        status: ReviewStatus,
        geom: MarkupGeometry,
    ) -> Markup {
        let mut m = Markup::new(page, kind, geom, author);
        m.id = Uuid::from_u128(id);
        m.subject = subject.to_string();
        m.status = status;
        m
    }

    #[test]
    fn identical_documents_empty_diff() {
        let mut doc = Document::empty("Doc");
        doc.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        let diff = diff_documents(&doc, &doc);
        assert!(diff.is_empty());
    }

    #[test]
    fn add_remove_detection() {
        let mut a = Document::empty("D");
        a.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));

        let mut b = Document::empty("D");
        b.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        b.add_markup(make_markup(
            2,
            0,
            MarkupKind::Rectangle,
            "B",
            "T",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 2.0, 2.0)),
        ));

        let diff = diff_documents(&a, &b);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.removed.len(), 0);
        assert_eq!(diff.changed.len(), 0);
        assert_eq!(diff.added[0].id, Uuid::from_u128(2));

        let diff2 = diff_documents(&b, &a);
        assert_eq!(diff2.removed.len(), 1);
        assert_eq!(diff2.removed[0].id, Uuid::from_u128(2));
    }

    #[test]
    fn change_detection_status_author_geometry_page() {
        let mut a = Document::empty("D");
        a.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        let mut b = Document::empty("D");
        b.add_page(Page::a4(1));
        b.add_markup(make_markup(
            1,
            1,
            MarkupKind::Highlight,
            "B",
            "S",
            ReviewStatus::Approved,
            MarkupGeometry::Bounds(Rect::new(1.0, 0.0, 1.0, 1.0)),
        ));

        let diff = diff_documents(&a, &b);
        assert_eq!(diff.changed.len(), 1);
        let c = &diff.changed[0];
        assert_eq!(c.id, Uuid::from_u128(1));
        assert_eq!(c.before.as_ref().unwrap().status, ReviewStatus::Open);
        assert_eq!(c.after.as_ref().unwrap().status, ReviewStatus::Approved);
        assert_ne!(
            c.before.as_ref().unwrap().author,
            c.after.as_ref().unwrap().author
        );
        assert_ne!(
            c.before.as_ref().unwrap().page,
            c.after.as_ref().unwrap().page
        );
        assert_ne!(
            c.before.as_ref().unwrap().geometry_fingerprint,
            c.after.as_ref().unwrap().geometry_fingerprint
        );
    }

    #[test]
    fn fingerprint_differs_on_geometry_variant() {
        let mut a = Document::empty("D");
        a.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        let mut b = Document::empty("D");
        // Same id, different geometry variant (rect vs segment)
        b.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(1.0, 1.0),
            },
        ));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.changed.len(), 1);
    }

    #[test]
    fn diff_projects_active_documents() {
        let mut a = Project::new("A");
        let mut b = Project::new("B");
        a.active_mut().unwrap().add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        b.active_mut().unwrap().add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Approved,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));

        let diff = diff_projects(&a, &b);
        assert_eq!(diff.changed.len(), 1);

        let empty_project = Project::new("Empty");
        let diff2 = diff_projects(&empty_project, &empty_project);
        assert!(diff2.is_empty());
    }

    #[test]
    fn by_page_grouping() {
        let mut a = Document::empty("D");
        let mut b = Document::empty("D");
        b.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        b.add_page(Page::a4(1));
        b.add_markup(make_markup(
            2,
            1,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        let diff = diff_documents(&a, &b);
        let by_page = diff.added_by_page();
        assert_eq!(by_page.get(&0).unwrap().len(), 1);
        assert_eq!(by_page.get(&1).unwrap().len(), 1);
    }

    #[test]
    fn diff_serialize_roundtrip() {
        let mut a = Document::empty("D");
        a.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        let mut b = Document::empty("D");
        b.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Approved,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        let diff = diff_documents(&a, &b);
        let json = serde_json::to_string(&diff).unwrap();
        let back: DiffResult = serde_json::from_str(&json).unwrap();
        assert_eq!(diff, back);
    }

    #[test]
    fn change_detection_style_and_locked_only() {
        let mut a = Document::empty("D");
        a.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        let mut b = Document::empty("D");
        let mut styled = make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        );
        styled.style.stroke = palette::GREEN;
        styled.locked = true;
        b.add_markup(styled);

        let diff = diff_documents(&a, &b);
        assert_eq!(diff.changed.len(), 1);
        let c = &diff.changed[0];
        assert_ne!(
            c.before.as_ref().unwrap().style,
            c.after.as_ref().unwrap().style,
        );
        assert!(!c.before.as_ref().unwrap().locked);
        assert!(c.after.as_ref().unwrap().locked);
    }

    #[test]
    fn diff_detects_a_layer_move() {
        let mut a = Document::empty("D");
        let layer = a.add_layer("Notes");
        a.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));

        let mut b = Document::empty("D");
        let mut moved = make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        );
        moved.layer_id = Some(layer);
        b.add_markup(moved);

        let diff = diff_documents(&a, &b);
        assert_eq!(
            diff.changed.len(),
            1,
            "a layer move must not read as no change"
        );
        assert_eq!(diff.changed[0].before.as_ref().unwrap().layer_id, None);
        assert_eq!(
            diff.changed[0].after.as_ref().unwrap().layer_id,
            Some(layer)
        );
    }

    #[test]
    fn summary_row_layer_id_defaults_for_legacy_snapshots() {
        let mut a = Document::empty("D");
        a.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        let row = MarkupSummaryRow::from_markup(&a.markups[0]);
        let mut value = serde_json::to_value(&row).expect("json");
        value.as_object_mut().expect("object").remove("layer_id");
        let back: MarkupSummaryRow = serde_json::from_value(value).expect("legacy row loads");
        assert_eq!(back.layer_id, None);
        assert_eq!(back.id, row.id);
    }

    #[test]
    fn duplicate_ids_in_one_document_are_diffed_once() {
        let mut a = Document::empty("D");
        let mut b = Document::empty("D");
        let dup1 = make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        );
        let mut dup2 = dup1.clone();
        dup2.geometry = MarkupGeometry::Bounds(Rect::new(2.0, 2.0, 1.0, 1.0));
        b.add_markup(dup1);
        b.add_markup(dup2);

        let diff = diff_documents(&a, &b);
        assert_eq!(diff.added.len(), 1);

        // The changed path must also collapse duplicate ids to one row.
        let mut a2 = Document::empty("A2");
        a2.add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        let mut b2 = Document::empty("B2");
        let mut changed1 = make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Approved,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        );
        let mut changed2 = changed1.clone();
        changed2.status = ReviewStatus::Rejected;
        b2.add_markup(changed1);
        b2.add_markup(changed2);
        let diff2 = diff_documents(&a2, &b2);
        assert_eq!(diff2.changed.len(), 1);
    }

    #[test]
    fn diff_projects_missing_active_document_treated_as_empty() {
        let mut a = Project::new("A");
        a.active_mut().unwrap().add_markup(make_markup(
            1,
            0,
            MarkupKind::Highlight,
            "A",
            "S",
            ReviewStatus::Open,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)),
        ));
        let mut b = Project::new("B");
        b.documents.clear();

        let diff = diff_projects(&a, &b);
        assert_eq!(diff.removed.len(), 1);
        let diff2 = diff_projects(&b, &a);
        assert_eq!(diff2.added.len(), 1);
        let diff3 = diff_projects(&b, &b);
        assert!(diff3.is_empty());
    }
}
