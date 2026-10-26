//! Markup style reuse via style presets.
//!
//! A [`StylePreset`] captures visual style and metadata from a markup and can
//! be applied to another markup without affecting geometry or comments.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::markup::{Markup, MarkupKind, ReviewStatus};
use super::style::MarkupStyle;

/// A reusable style and metadata preset for markups.
///
/// The preset records a visual style, kind, author, subject, status and lock
/// state. Applying a preset never touches geometry, id, page, comments,
/// measurement or creation timestamps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StylePreset {
    pub id: Uuid,
    pub name: String,
    pub kind: MarkupKind,
    pub style: MarkupStyle,
    pub author: String,
    pub subject: String,
    pub status: ReviewStatus,
    pub locked: bool,
    pub created: DateTime<Utc>,
}

impl StylePreset {
    /// Capture a preset from an existing markup.
    pub fn from_markup(markup: &Markup, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind: markup.kind,
            style: markup.style,
            author: markup.author.clone(),
            subject: markup.subject.clone(),
            status: markup.status,
            locked: markup.locked,
            created: Utc::now(),
        }
    }

    /// Apply this preset to `markup`.
    ///
    /// - Replaces the visual [`MarkupStyle`].
    /// - Sets `subject` only if the target's subject is empty.
    /// - Sets `author` only if the target's author is empty.
    /// - Overwrites `status` and `locked`.
    /// - Does not modify geometry, id, page, comments, measurement or `created`.
    pub fn apply_to(&self, markup: &mut Markup) {
        markup.style = self.style;
        if markup.subject.is_empty() {
            markup.subject = self.subject.clone();
        }
        if markup.author.is_empty() {
            markup.author = self.author.clone();
        }
        markup.status = self.status;
        markup.locked = self.locked;
        markup.touch();
    }
}

/// A collection of reusable style presets.
///
/// The library keeps at most 64 presets. Adding a preset with an existing id
/// moves it to the back (upsert dedupe); when full the oldest preset is
/// dropped.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StylePresetLibrary {
    pub presets: Vec<StylePreset>,
}

impl StylePresetLibrary {
    const MAX_PRESETS: usize = 64;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.presets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.presets.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, StylePreset> {
        self.presets.iter()
    }

    /// Add or replace a preset.
    ///
    /// If a preset with the same `id` exists it is removed first (dedupe), the
    /// new preset is appended, and the total is capped at [`MAX_PRESETS`] by
    /// dropping the oldest entries.
    pub fn add(&mut self, preset: StylePreset) {
        self.presets.retain(|p| p.id != preset.id);
        self.presets.push(preset);
        while self.presets.len() > Self::MAX_PRESETS {
            self.presets.remove(0);
        }
    }

    /// Remove a preset by id.
    pub fn remove(&mut self, id: Uuid) -> bool {
        let len_before = self.presets.len();
        self.presets.retain(|p| p.id != id);
        self.presets.len() != len_before
    }

    pub fn find(&self, id: Uuid) -> Option<&StylePreset> {
        self.presets.iter().find(|p| p.id == id)
    }

    /// Find a preset by name, case-insensitive.
    pub fn find_by_name(&self, name: &str) -> Option<&StylePreset> {
        self.presets
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
    }

    /// Rename a preset.
    pub fn rename(&mut self, id: Uuid, name: impl Into<String>) -> bool {
        if let Some(p) = self.presets.iter_mut().find(|p| p.id == id) {
            p.name = name.into();
            true
        } else {
            false
        }
    }

    /// All presets for a given kind.
    pub fn for_kind(&self, kind: MarkupKind) -> Vec<&StylePreset> {
        self.presets.iter().filter(|p| p.kind == kind).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geom::{Pt, Rect};
    use crate::domain::markup::{Markup, MarkupGeometry, MarkupKind};
    use crate::domain::style::{palette, FillMode, MarkupStyle};

    fn sample_markup() -> Markup {
        let mut m = Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(10.0, 20.0, 30.0, 40.0)),
            "Alice",
        );
        m.subject = "Subject A".to_string();
        m.style = MarkupStyle::default().with_stroke(palette::RED);
        m
    }

    #[test]
    fn from_markup_capture_then_apply_roundtrip() {
        let original = sample_markup();
        let preset = StylePreset::from_markup(&original, "Red Rect");
        let mut target = Markup::new(
            1,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 5.0, 5.0)),
            "Bob",
        );
        target.subject = "".to_string();
        let geom_before = target.geometry.clone();
        let id_before = target.id;
        preset.apply_to(&mut target);

        // Style applied
        assert_eq!(target.style.stroke, palette::RED);
        // Subject filled because it was empty
        assert_eq!(target.subject, "Subject A");
        // Author preserved because target author is non-empty
        assert_eq!(target.author, "Bob");
        // Kind unchanged
        assert_eq!(target.kind, MarkupKind::Rectangle);
        // Geometry untouched
        assert_eq!(target.geometry, geom_before);
        assert_eq!(target.id, id_before);
        // Comments unchanged
        assert!(target.comments.is_empty());
    }

    #[test]
    fn apply_preserves_non_empty_fields() {
        let mut source = sample_markup();
        source.author = "Carol".to_string();
        source.subject = "Keep Me".to_string();
        let preset = StylePreset::from_markup(&source, "Keep");
        let mut target = Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(1.0, 1.0, 1.0, 1.0)),
            "David",
        );
        target.subject = "Existing Subject".to_string();
        preset.apply_to(&mut target);
        // Author not overwritten
        assert_eq!(target.author, "David");
        // Subject not overwritten
        assert_eq!(target.subject, "Existing Subject");
    }

    #[test]
    fn library_cap_and_dedupe() {
        let mut lib = StylePresetLibrary::new();
        let mut base = sample_markup();
        for i in 0..70 {
            let mut m = base.clone();
            m.id = Uuid::new_v4();
            let preset = StylePreset::from_markup(&m, format!("Preset {i}"));
            lib.add(preset);
        }
        assert_eq!(lib.len(), 64);
        // First 6 were evicted
        assert!(!lib.presets.iter().any(|p| p.name == "Preset 0"));
        // Last added present
        assert!(lib.presets.iter().any(|p| p.name == "Preset 69"));

        // Dedupe: re-add same id moves to back
        let first = lib.presets[10].clone();
        let len_before = lib.len();
        lib.add(first.clone());
        assert_eq!(lib.len(), len_before);
        assert_eq!(lib.presets.last().unwrap().id, first.id);
    }

    #[test]
    fn library_find_rename_by_name_case_insensitive() {
        let mut lib = StylePresetLibrary::new();
        let preset = StylePreset::from_markup(&sample_markup(), "MyPreset");
        lib.add(preset.clone());
        assert!(lib.find(preset.id).is_some());
        assert!(lib.find_by_name("mypRESET").is_some());
        assert!(lib.rename(preset.id, "Renamed"));
        assert_eq!(lib.find(preset.id).unwrap().name, "Renamed");
        assert!(!lib.rename(Uuid::new_v4(), "X"));
    }

    #[test]
    fn for_kind_filters() {
        let mut lib = StylePresetLibrary::new();
        let mut m1 = sample_markup();
        m1.kind = MarkupKind::Highlight;
        let p1 = StylePreset::from_markup(&m1, "H");
        let mut m2 = sample_markup();
        m2.kind = MarkupKind::Rectangle;
        let p2 = StylePreset::from_markup(&m2, "R");
        lib.add(p1);
        lib.add(p2);
        let kinds = lib.for_kind(MarkupKind::Highlight);
        assert_eq!(kinds.len(), 1);
        assert_eq!(kinds[0].kind, MarkupKind::Highlight);
    }

    #[test]
    fn style_preset_serialize_roundtrip() {
        let preset = StylePreset::from_markup(&sample_markup(), "Test");
        let json = serde_json::to_string(&preset).unwrap();
        let back: StylePreset = serde_json::from_str(&json).unwrap();
        // Full equality: every field (id, name, style, created, status,
        // locked, ...) must survive the round trip.
        assert_eq!(preset, back);
    }

    #[test]
    fn apply_to_touches_modified() {
        let preset = StylePreset::from_markup(&sample_markup(), "Test");
        let mut target = Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(1.0, 1.0, 1.0, 1.0)),
            "David",
        );
        let before = target.modified;
        preset.apply_to(&mut target);
        assert!(target.modified >= before);
    }
}
