//! Markup list view, filtering, sorting and grouping.
//!
//! This module provides a pure-domain snapshot of markups for list display.
//! All types are serde-compatible and headless-testable.

use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::document::Document;
use super::markup::{Markup, MarkupKind, ReviewStatus};
use super::project::Project;
use super::report::NONE_LABEL;
use super::report::normalise_group_value;

/// A filter describing an inclusive query over markups.
///
/// All fields are optional; `None` means the criterion is not applied.  String
/// comparisons are case-insensitive for `authors`, `subjects` and `search`.
/// The `layers` criterion works on the raw [`Markup::layer_id`]; markups with
/// `None` layer_id match only when `include_default_layer` is true.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MarkupListFilter {
    #[serde(default)]
    pub statuses: Option<HashSet<ReviewStatus>>,
    #[serde(default)]
    pub authors: Option<HashSet<String>>,
    #[serde(default)]
    pub subjects: Option<HashSet<String>>,
    #[serde(default)]
    pub layers: Option<HashSet<Uuid>>,
    #[serde(default)]
    pub include_default_layer: bool,
    #[serde(default)]
    pub kinds: Option<HashSet<MarkupKind>>,
    #[serde(default)]
    pub pages: Option<HashSet<usize>>,
    #[serde(default)]
    pub search: Option<String>,
}

impl MarkupListFilter {
    /// Returns true if `markup` satisfies every active criterion in this filter.
    ///
    /// Comment texts are included in the `search` substring test.
    pub fn matches(&self, markup: &Markup) -> bool {
        if let Some(statuses) = &self.statuses {
            if !statuses.contains(&markup.status) {
                return false;
            }
        }
        if let Some(authors) = &self.authors {
            if !authors.iter().any(|a| a.eq_ignore_ascii_case(&markup.author)) {
                return false;
            }
        }
        if let Some(subjects) = &self.subjects {
            if !subjects
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&markup.subject))
            {
                return false;
            }
        }
        if let Some(kinds) = &self.kinds {
            if !kinds.contains(&markup.kind) {
                return false;
            }
        }
        if let Some(pages) = &self.pages {
            if !pages.contains(&markup.page) {
                return false;
            }
        }
        if let Some(layers) = &self.layers {
            match markup.layer_id {
                Some(id) => {
                    if !layers.contains(&id) {
                        return false;
                    }
                }
                None => {
                    if !self.include_default_layer {
                        return false;
                    }
                }
            }
        }
        if let Some(q) = &self.search {
            let query = q.to_lowercase();
            if !query.is_empty() {
                let hay = format!(
                    "{} {} {}",
                    markup.author.to_lowercase(),
                    markup.subject.to_lowercase(),
                    markup
                        .comments
                        .iter()
                        .map(|c| c.text.to_lowercase())
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                if !hay.contains(&query) {
                    return false;
                }
            }
        }
        true
    }

    /// Predicate over a [`MarkupRow`]; comment texts are not available so
    /// `search` checks only author, subject and the row's `summary`.
    pub fn matches_row(&self, row: &MarkupRow) -> bool {
        if let Some(statuses) = &self.statuses {
            if !statuses.contains(&row.status) {
                return false;
            }
        }
        if let Some(authors) = &self.authors {
            if !authors.iter().any(|a| a.eq_ignore_ascii_case(&row.author)) {
                return false;
            }
        }
        if let Some(subjects) = &self.subjects {
            if !subjects
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&row.subject))
            {
                return false;
            }
        }
        if let Some(kinds) = &self.kinds {
            if !kinds.contains(&row.kind) {
                return false;
            }
        }
        if let Some(pages) = &self.pages {
            if !pages.contains(&row.page) {
                return false;
            }
        }
        if let Some(layers) = &self.layers {
            match row.layer_id {
                Some(id) => {
                    if !layers.contains(&id) {
                        return false;
                    }
                }
                None => {
                    if !self.include_default_layer {
                        return false;
                    }
                }
            }
        }
        if let Some(q) = &self.search {
            let query = q.to_lowercase();
            if !query.is_empty() {
                let hay = format!(
                    "{} {} {}",
                    row.author.to_lowercase(),
                    row.subject.to_lowercase(),
                    row.summary.to_lowercase()
                );
                if !hay.contains(&query) {
                    return false;
                }
            }
        }
        true
    }
}

/// Sort order for a markup list.
///
/// `CreatedAsc`/`CreatedDesc` tie-break by markup id string for determinism.
/// Other variants use a stable sort, preserving the original order for ties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkupListSort {
    #[default]
    CreatedAsc,
    CreatedDesc,
    PageAsc,
    PageDesc,
    KindAsc,
    KindDesc,
    ModifiedDesc,
}

impl MarkupListSort {
    pub const ALL: [MarkupListSort; 7] = [
        MarkupListSort::CreatedAsc,
        MarkupListSort::CreatedDesc,
        MarkupListSort::PageAsc,
        MarkupListSort::PageDesc,
        MarkupListSort::KindAsc,
        MarkupListSort::KindDesc,
        MarkupListSort::ModifiedDesc,
    ];

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            MarkupListSort::CreatedAsc => "Created Asc",
            MarkupListSort::CreatedDesc => "Created Desc",
            MarkupListSort::PageAsc => "Page Asc",
            MarkupListSort::PageDesc => "Page Desc",
            MarkupListSort::KindAsc => "Kind Asc",
            MarkupListSort::KindDesc => "Kind Desc",
            MarkupListSort::ModifiedDesc => "Modified Desc",
        }
    }

    /// Apply this sort stably in place to `rows`.
    pub fn apply(self, rows: &mut [MarkupRow]) {
        rows.sort_by(|a, b| self.compare(a, b));
    }

    fn compare(&self, a: &MarkupRow, b: &MarkupRow) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        let id_cmp = |a: &MarkupRow, b: &MarkupRow| {
            a.id.to_string().cmp(&b.id.to_string())
        };
        match self {
            MarkupListSort::CreatedAsc => {
                a.created.cmp(&b.created).then_with(|| id_cmp(a, b))
            }
            MarkupListSort::CreatedDesc => {
                b.created.cmp(&a.created).then_with(|| id_cmp(a, b))
            }
            MarkupListSort::PageAsc => a.page.cmp(&b.page),
            MarkupListSort::PageDesc => b.page.cmp(&a.page),
            MarkupListSort::KindAsc => {
                a.kind.label().cmp(&b.kind.label())
            }
            MarkupListSort::KindDesc => {
                b.kind.label().cmp(&a.kind.label())
            }
            MarkupListSort::ModifiedDesc => b.modified.cmp(&a.modified),
        }
    }
}

/// Key used to group rows in a list view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkupGroupKey {
    Status,
    Author,
    Subject,
    Layer,
    Kind,
    Page,
}

impl MarkupGroupKey {
    pub const ALL: [MarkupGroupKey; 6] = [
        MarkupGroupKey::Status,
        MarkupGroupKey::Author,
        MarkupGroupKey::Subject,
        MarkupGroupKey::Layer,
        MarkupGroupKey::Kind,
        MarkupGroupKey::Page,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MarkupGroupKey::Status => "Status",
            MarkupGroupKey::Author => "Author",
            MarkupGroupKey::Subject => "Subject",
            MarkupGroupKey::Layer => "Layer",
            MarkupGroupKey::Kind => "Kind",
            MarkupGroupKey::Page => "Page",
        }
    }

    /// Group value for a full [`Markup`]; layer name resolution is not applied,
    /// the raw id string is used.
    pub fn group_value(&self, markup: &Markup) -> String {
        self.group_value_for_row(&MarkupRow::from_markup(markup), None)
    }

    /// Group value with an explicit layer name resolver.
    pub fn group_value_with_layers<F>(self, markup: &Markup, layer_name: F) -> String
    where
        F: Fn(Uuid) -> String,
    {
        let row = MarkupRow::from_markup(markup);
        let name = |id| layer_name(id);
        self.group_value_for_row(&row, Some(&name))
    }

    /// Group value for a [`MarkupRow`]; layer name is resolved via `layer_name`
    /// when present.
    pub fn group_value_for_row(&self, row: &MarkupRow, layer_name: Option<&dyn Fn(Uuid) -> String>) -> String {
        match self {
            MarkupGroupKey::Status => row.status.label().to_string(),
            MarkupGroupKey::Author => normalise_group_value(&row.author),
            MarkupGroupKey::Subject => normalise_group_value(&row.subject),
            MarkupGroupKey::Kind => row.kind.label().to_string(),
            MarkupGroupKey::Page => (row.page + 1).to_string(),
            MarkupGroupKey::Layer => match row.layer_id {
                Some(id) => match layer_name {
                    Some(f) => f(id),
                    None => id.to_string(),
                },
                None => NONE_LABEL.to_string(),
            },
        }
    }
}

/// A serialisable snapshot of a markup for list display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkupRow {
    pub id: Uuid,
    pub page: usize,
    pub kind: MarkupKind,
    pub author: String,
    pub subject: String,
    pub status: ReviewStatus,
    pub layer_id: Option<Uuid>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub summary: String,
}

impl MarkupRow {
    fn from_markup(markup: &Markup) -> Self {
        Self {
            id: markup.id,
            page: markup.page,
            kind: markup.kind,
            author: markup.author.clone(),
            subject: markup.subject.clone(),
            status: markup.status,
            layer_id: markup.layer_id,
            created: markup.created,
            modified: markup.modified,
            summary: markup.summary(),
        }
    }
}

/// A list view of markups, built from a document or project.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MarkupList {
    pub rows: Vec<MarkupRow>,
}

impl MarkupList {
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Build a list from `doc`, preserving markup order.
    pub fn from_document(doc: &Document) -> Self {
        let rows = doc
            .markups
            .iter()
            .map(MarkupRow::from_markup)
            .collect();
        Self { rows }
    }

    /// Build a list from the active document of `project`. Multi-document
    /// listing is future work.
    pub fn from_project(project: &Project) -> Self {
        if let Some(doc) = project.active() {
            Self::from_document(doc)
        } else {
            Self::new()
        }
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, MarkupRow> {
        self.rows.iter()
    }

    pub fn rows(&self) -> &[MarkupRow] {
        &self.rows
    }

    /// Return a new list containing only rows that satisfy `f`.
    /// The original order is preserved.
    pub fn filter(&self, f: &MarkupListFilter) -> Self {
        let rows = self
            .rows
            .iter()
            .filter(|r| f.matches_row(r))
            .cloned()
            .collect();
        Self { rows }
    }

    /// Sort this list in place using `sort`.
    pub fn sort(&mut self, sort: MarkupListSort) {
        sort.apply(&mut self.rows);
    }

    /// Group rows by `key`, resolving layer names via `layer_names`.
    ///
    /// Groups are ordered alphabetically by the group label string.
    pub fn group(&self, key: MarkupGroupKey, layer_names: &dyn Fn(Uuid) -> String) -> Vec<(String, MarkupList)> {
        let mut map: BTreeMap<String, MarkupList> = BTreeMap::new();
        for row in &self.rows {
            let label = key.group_value_for_row(row, Some(layer_names));
            map.entry(label).or_default().rows.push(row.clone());
        }
        map.into_iter().map(|(k, v)| (k, v)).collect()
    }

    /// Count rows per review status.
    pub fn counts_by_status(&self) -> BTreeMap<ReviewStatus, usize> {
        let mut counts = BTreeMap::new();
        for row in &self.rows {
            *counts.entry(row.status).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::document::Document;
    use crate::domain::geom::{Pt, Rect};
    use crate::domain::markup::{Markup, MarkupGeometry, MarkupKind, ReviewStatus};
    use crate::domain::project::Project;

    fn make_markup(id_u128: u128, page: usize, kind: MarkupKind, author: &str, subject: &str, status: ReviewStatus) -> Markup {
        let mut m = Markup::new(page, kind, MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 1.0, 1.0)), author);
        m.id = Uuid::from_u128(id_u128);
        m.subject = subject.to_string();
        m.status = status;
        m
    }

    #[test]
    fn markup_list_from_project_empty() {
        let project = Project::new("Empty");
        let list = MarkupList::from_project(&project);
        assert!(list.is_empty());
    }

    #[test]
    fn markup_list_filter_status_author_subject_kind_page() {
        let mut doc = Document::empty("D");
        doc.add_markup(make_markup(1, 0, MarkupKind::Highlight, "Alice", "Elec", ReviewStatus::Open));
        doc.add_markup(make_markup(2, 1, MarkupKind::Rectangle, "Bob", "Plumb", ReviewStatus::Approved));
        doc.add_markup(make_markup(3, 0, MarkupKind::Highlight, "alice", "Elec", ReviewStatus::Open));

        let list = MarkupList::from_document(&doc);
        let mut filter = MarkupListFilter::default();
        filter.statuses = Some([ReviewStatus::Open].into_iter().collect());
        let filtered = list.filter(&filter);
        assert_eq!(filtered.len(), 2);

        let mut filter = MarkupListFilter::default();
        filter.authors = Some(["bob".to_string()].into_iter().collect());
        let filtered = list.filter(&filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.rows[0].author, "Bob");

        let mut filter = MarkupListFilter::default();
        filter.subjects = Some(["elec".to_string()].into_iter().collect());
        let filtered = list.filter(&filter);
        assert_eq!(filtered.len(), 2);

        let mut filter = MarkupListFilter::default();
        filter.kinds = Some([MarkupKind::Rectangle].into_iter().collect());
        let filtered = list.filter(&filter);
        assert_eq!(filtered.len(), 1);

        let mut filter = MarkupListFilter::default();
        filter.pages = Some([0usize].into_iter().collect());
        let filtered = list.filter(&filter);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn filter_search_case_insensitive_and_comments() {
        let mut m = make_markup(1, 0, MarkupKind::Note, "Alice", "Issue", ReviewStatus::Open);
        use crate::domain::markup::Comment;
        m.comments.push(Comment::new("Carol", "Check valve"));
        let filter = MarkupListFilter {
            search: Some("check VALVE".to_string()),
            ..Default::default()
        };
        assert!(filter.matches(&m));
        // Row search does not include comments — matches row checks summary only
        let row = MarkupRow::from_markup(&m);
        assert!(!filter.matches_row(&row));
    }

    #[test]
    fn filter_layers_include_default() {
        let mut doc = Document::empty("D");
        let layer_id = doc.add_layer("L1");
        let mut m1 = make_markup(1, 0, MarkupKind::Highlight, "A", "S", ReviewStatus::Open);
        m1.layer_id = Some(layer_id);
        let mut m2 = make_markup(2, 0, MarkupKind::Highlight, "B", "S", ReviewStatus::Open);
        m2.layer_id = None;
        doc.add_markup(m1);
        doc.add_markup(m2);
        let list = MarkupList::from_document(&doc);
        let mut filter = MarkupListFilter::default();
        filter.layers = Some([layer_id].into_iter().collect());
        filter.include_default_layer = false;
        let filtered = list.filter(&filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.rows[0].id, Uuid::from_u128(1));
    }

    #[test]
    fn sort_created_tie_break_by_id() {
        use chrono::TimeZone;
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let row_a = MarkupRow {
            id: Uuid::from_u128(2),
            page: 0,
            kind: MarkupKind::Highlight,
            author: "".into(),
            subject: "".into(),
            status: ReviewStatus::Open,
            layer_id: None,
            created: now,
            modified: now,
            summary: "".into(),
        };
        let row_b = MarkupRow {
            id: Uuid::from_u128(1),
            page: 0,
            kind: MarkupKind::Highlight,
            author: "".into(),
            subject: "".into(),
            status: ReviewStatus::Open,
            layer_id: None,
            created: now,
            modified: now,
            summary: "".into(),
        };
        let mut rows = vec![row_a.clone(), row_b.clone()];
        MarkupListSort::CreatedAsc.apply(&mut rows);
        // Tie-break by id string: 1 < 2
        assert_eq!(rows[0].id, Uuid::from_u128(1));
    }

    #[test]
    fn group_ordering_and_membership() {
        let mut doc = Document::empty("D");
        doc.add_markup(make_markup(1, 0, MarkupKind::Highlight, "Alice", "Elec", ReviewStatus::Open));
        doc.add_markup(make_markup(2, 0, MarkupKind::Rectangle, "Bob", "Plumb", ReviewStatus::Approved));
        doc.add_markup(make_markup(3, 0, MarkupKind::Highlight, "Alice", "Elec", ReviewStatus::Approved));

        let list = MarkupList::from_document(&doc);
        let groups = list.group(MarkupGroupKey::Author, &|_| "X".to_string());
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, "Alice");
        assert_eq!(groups[0].1.len(), 2);
        assert_eq!(groups[1].0, "Bob");
    }

    #[test]
    fn counts_by_status() {
        let mut doc = Document::empty("D");
        doc.add_markup(make_markup(1, 0, MarkupKind::Highlight, "A", "S", ReviewStatus::Open));
        doc.add_markup(make_markup(2, 0, MarkupKind::Highlight, "A", "S", ReviewStatus::Approved));
        doc.add_markup(make_markup(3, 0, MarkupKind::Highlight, "A", "S", ReviewStatus::Approved));
        let list = MarkupList::from_document(&doc);
        let counts = list.counts_by_status();
        assert_eq!(*counts.get(&ReviewStatus::Open).unwrap(), 1);
        assert_eq!(*counts.get(&ReviewStatus::Approved).unwrap(), 2);
    }

    #[test]
    fn markup_list_serialize_roundtrip() {
        let mut doc = Document::empty("D");
        doc.add_markup(make_markup(1, 0, MarkupKind::Highlight, "A", "S", ReviewStatus::Open));
        let list = MarkupList::from_document(&doc);
        let json = serde_json::to_string(&list).unwrap();
        let back: MarkupList = serde_json::from_str(&json).unwrap();
        assert_eq!(list, back);
    }

    #[test]
    fn filter_serialize_roundtrip() {
        let filter = MarkupListFilter {
            statuses: Some([ReviewStatus::Open].into_iter().collect()),
            search: Some("foo".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&filter).unwrap();
        let back: MarkupListFilter = serde_json::from_str(&json).unwrap();
        assert_eq!(filter, back);
    }
}
