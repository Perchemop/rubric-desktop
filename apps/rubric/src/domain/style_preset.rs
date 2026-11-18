//! Markup style reuse via style presets.
//!
//! A [`StylePreset`] captures visual style and metadata from a markup and can
//! be applied to another markup without affecting geometry or comments.
//!
//! A preset may also capture the source markup's **geometry**, normalised so
//! its bounding box starts at the origin (Drawing Mode, guide audit B1/B5).
//! Re-placing such a preset at a point recreates an exact duplicate of the
//! original markup instead of only re-styling a new one; a preset without
//! captured geometry keeps the style-only behaviour, and legacy preset files
//! (which have no geometry fields) load unchanged.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::markup::{Markup, MarkupKind, ReviewStatus};
use super::markup_geometry::MarkupGeometry;
use super::style::MarkupStyle;

/// A reusable style and metadata preset for markups.
///
/// The preset records a visual style, kind, author, subject and status. It may
/// also record the source markup's geometry ([`Self::geometry`]), which turns
/// arming it into "place an exact duplicate" (Drawing Mode). Applying a preset
/// with [`Self::apply_to`] never touches geometry, id, page, comments,
/// measurement, lock state or creation timestamps, and it skips locked targets
/// entirely.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StylePreset {
    pub id: Uuid,
    pub name: String,
    pub kind: MarkupKind,
    pub style: MarkupStyle,
    pub author: String,
    pub subject: String,
    pub status: ReviewStatus,
    /// Legacy capture of the source markup's per-markup lock. A *style*
    /// preset neither captures nor imposes lock state any more; the field is
    /// kept only so previously saved presets still load. Presets captured
    /// today always store `false`.
    pub locked: bool,
    pub created: DateTime<Utc>,
    /// Captured geometry, translated so its bounding box's top-left corner sits
    /// at the origin (page points) so placement is click-relative.
    ///
    /// When present, re-placing the preset creates an exact duplicate of the
    /// captured markup offset to the click point (Drawing Mode); when absent
    /// the preset is style-only, exactly as before this field existed. Legacy
    /// preset files have no such field and therefore load with `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<MarkupGeometry>,
}

impl StylePreset {
    /// Capture a preset from an existing markup.
    ///
    /// The markup's geometry is captured too, normalised to the origin (see
    /// [`Self::normalized_geometry`]), so arming the preset can re-place an
    /// exact duplicate. Stamps are the exception: their look comes from the
    /// stamp preset rather than from the geometry, so a stamp is captured
    /// style-only and keeps today's stamp drawing path.
    pub fn from_markup(markup: &Markup, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind: markup.kind,
            style: markup.style,
            author: markup.author.clone(),
            subject: markup.subject.clone(),
            status: markup.status,
            // Lock state is not style: a preset must not capture it (the
            // field remains for serialisation compatibility).
            locked: false,
            created: Utc::now(),
            geometry: (markup.kind != MarkupKind::Stamp)
                .then(|| Self::normalized_geometry(&markup.geometry)),
        }
    }

    /// Capture a preset from a tool's default style rather than an existing
    /// markup (the Tool Chest's "Save current tool as…", v0.4 gap #7).
    ///
    /// Nothing has been drawn or reviewed yet, so `subject` is empty and
    /// `status` is [`ReviewStatus::Open`]; `locked` is always `false` because
    /// lock state is not style (see [`Self::from_markup`]). Nothing was drawn,
    /// so there is no geometry to capture: the preset is style-only.
    pub fn from_style(
        name: impl Into<String>,
        kind: MarkupKind,
        style: MarkupStyle,
        author: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind,
            style,
            author: author.into(),
            subject: String::new(),
            status: ReviewStatus::Open,
            locked: false,
            created: Utc::now(),
            geometry: None,
        }
    }

    /// Translate `geometry` so its bounding box's top-left corner is at the
    /// origin.
    ///
    /// Captured presets store geometry this way so a later placement is a pure
    /// offset: the caller translates the stored geometry onto the click point.
    /// Every variant of [`MarkupGeometry`] is moved by
    /// [`MarkupGeometry::translate`].
    pub fn normalized_geometry(geometry: &MarkupGeometry) -> MarkupGeometry {
        let bounds = geometry.bounds();
        let mut normalized = geometry.clone();
        normalized.translate(-bounds.x, -bounds.y);
        normalized
    }

    /// Whether this preset captured geometry, i.e. whether it can be re-placed
    /// as a duplicate (Drawing Mode) rather than only re-styling new markups.
    pub fn can_draw(&self) -> bool {
        self.geometry.is_some()
    }

    /// The same preset in style-only form: every field identical, but with the
    /// captured geometry dropped.
    ///
    /// The Tool Chest arms this copy for Style Mode so the canvas keeps its
    /// style-only creation path exactly as it was before Drawing Mode existed;
    /// the library entry keeps its geometry so Drawing Mode can use it later.
    pub fn without_geometry(&self) -> Self {
        let mut style_only = self.clone();
        style_only.geometry = None;
        style_only
    }

    /// Apply this preset to `markup`.
    ///
    /// - Replaces the visual [`MarkupStyle`].
    /// - Sets `subject` only if the target's subject is empty.
    /// - Sets `author` only if the target's author is empty.
    /// - Overwrites `status`: a preset captures a review verdict alongside
    ///   the look, and preserving that contract keeps existing presets and
    ///   tests working.
    /// - Never copies `locked`: lock state is not style, so a preset must not
    ///   capture or impose it.
    /// - Skips locked targets entirely (a no-op, like
    ///   `Document::apply_style_to`).
    /// - Does not modify geometry, id, page, comments, measurement or
    ///   `created`.
    pub fn apply_to(&self, markup: &mut Markup) {
        if markup.locked {
            return;
        }
        markup.style = self.style;
        if markup.subject.is_empty() {
            markup.subject = self.subject.clone();
        }
        if markup.author.is_empty() {
            markup.author = self.author.clone();
        }
        markup.status = self.status;
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
    /// Maximum number of saved styles in the library. Oldest entries are
    /// dropped when the cap is exceeded.
    pub const MAX_PRESETS: usize = 64;

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

    /// Save a preset under a name, replacing an existing preset with the same
    /// name (case-insensitive) in place.
    ///
    /// The replacement keeps the *stored* preset's id, so re-saving "My Tool"
    /// updates that one entry instead of creating a near-duplicate; the entry
    /// still moves to the back (MRU order) and the [`MAX_PRESETS`] cap still
    /// applies. Returns the id of the stored preset.
    pub fn save_named(&mut self, preset: StylePreset) -> Uuid {
        let existing_id = self.find_by_name(&preset.name).map(|p| p.id);
        let mut stored = preset;
        if let Some(id) = existing_id {
            stored.id = id;
        }
        let id = stored.id;
        self.add(stored);
        id
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
    use crate::domain::style::{palette, MarkupStyle};

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
        let base = sample_markup();
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
    fn from_style_capture_defaults() {
        let style = MarkupStyle::default()
            .with_stroke(palette::BLUE)
            .with_width(3.0);
        let preset = StylePreset::from_style("Blue Line", MarkupKind::Line, style, "Eve");
        assert_eq!(preset.name, "Blue Line");
        assert_eq!(preset.kind, MarkupKind::Line);
        assert_eq!(preset.style, style);
        assert_eq!(preset.author, "Eve");
        assert!(preset.subject.is_empty(), "nothing has been reviewed yet");
        assert_eq!(preset.status, ReviewStatus::Open);
        assert!(!preset.locked, "lock state is not style");

        // Applying a tool-captured preset to a fresh markup sets only the
        // look: the fresh markup keeps its Open status and kind label subject.
        let mut target = Markup::new(
            0,
            MarkupKind::Line,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(9.0, 9.0),
            },
            "Eve",
        );
        let status_before = target.status;
        let subject_before = target.subject.clone();
        preset.apply_to(&mut target);
        assert_eq!(target.style, style);
        assert_eq!(target.status, status_before);
        assert_eq!(target.subject, subject_before);
    }

    #[test]
    fn library_save_named_dedupes_case_insensitive() {
        let mut lib = StylePresetLibrary::new();
        let first = StylePreset::from_style(
            "My Tool",
            MarkupKind::Rectangle,
            MarkupStyle::default(),
            "A",
        );
        let id = lib.save_named(first);
        assert_eq!(lib.len(), 1);

        // Re-saving a differently-cased name replaces the single entry and
        // keeps its id; the stored preset adopts the new style and name.
        let second = StylePreset::from_style(
            "MY TOOL",
            MarkupKind::Rectangle,
            MarkupStyle::default().with_stroke(palette::BLUE),
            "A",
        );
        let id2 = lib.save_named(second);
        assert_eq!(lib.len(), 1, "same name must not create a second entry");
        assert_eq!(id2, id, "the stored id is kept so callers can re-find it");
        assert_eq!(lib.find(id).unwrap().name, "MY TOOL");
        assert_eq!(lib.find(id).unwrap().style.stroke, palette::BLUE);

        // A different name is a new entry.
        let third =
            StylePreset::from_style("Other", MarkupKind::Ellipse, MarkupStyle::default(), "A");
        let id3 = lib.save_named(third);
        assert_ne!(id3, id);
        assert_eq!(lib.len(), 2);
    }

    #[test]
    fn style_preset_library_serialize_roundtrip() {
        let mut lib = StylePresetLibrary::new();
        lib.save_named(StylePreset::from_style(
            "Saved",
            MarkupKind::Arrow,
            MarkupStyle::default().with_stroke(palette::TEAL),
            "Ann",
        ));
        lib.add(StylePreset::from_markup(&sample_markup(), "From Markup"));
        let json = serde_json::to_string(&lib).unwrap();
        let back: StylePresetLibrary = serde_json::from_str(&json).unwrap();
        assert_eq!(lib, back);
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

    #[test]
    fn from_markup_captures_no_lock_state() {
        let mut locked = sample_markup();
        locked.locked = true;
        let preset = StylePreset::from_markup(&locked, "From Locked");
        assert!(!preset.locked, "a preset must not capture lock state");
    }

    #[test]
    fn apply_to_never_imposes_lock_and_skips_locked_targets() {
        let preset = StylePreset::from_markup(&sample_markup(), "Test");

        // An unlocked target is styled but never locked by a preset.
        let mut target = sample_markup();
        preset.apply_to(&mut target);
        assert!(!target.locked, "a preset must not impose lock state");

        // A locked target is left entirely alone.
        let mut locked_target = sample_markup();
        locked_target.locked = true;
        let before_style = locked_target.style;
        let before_status = locked_target.status;
        let before_modified = locked_target.modified;
        preset.apply_to(&mut locked_target);
        assert_eq!(locked_target.style, before_style);
        assert_eq!(locked_target.status, before_status);
        assert!(locked_target.locked);
        assert_eq!(
            locked_target.modified, before_modified,
            "a locked target must not be touched"
        );
    }

    #[test]
    fn from_markup_captures_geometry_normalised_to_origin() {
        // sample_markup's bounds start at (10, 20); capture must move the
        // whole geometry so its bounding box's top-left is the origin, which
        // is what makes a later placement a pure click-relative offset.
        let preset = StylePreset::from_markup(&sample_markup(), "Draw Rect");
        let geometry = preset.geometry.clone().expect("geometry captured");
        assert_eq!(
            geometry,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 30.0, 40.0))
        );
        let bounds = geometry.bounds();
        assert!(bounds.x.abs() < 1e-6 && bounds.y.abs() < 1e-6);
        assert!(preset.can_draw());
    }

    #[test]
    fn from_markup_normalises_non_bounds_geometry() {
        // A text anchor is offset from the page origin; capture must pull its
        // bounding box back to the origin and keep its text intact.
        let markup = Markup::new(
            0,
            MarkupKind::Highlight,
            MarkupGeometry::TextAnchor {
                at: Pt::new(120.0, 240.0),
                text: "reviewed".to_string(),
                line_height: 12.0,
                line_count: 1,
                text_width: 48.0,
            },
            "Alice",
        );
        let preset = StylePreset::from_markup(&markup, "Highlight");
        let geometry = preset.geometry.clone().expect("geometry captured");
        match &geometry {
            MarkupGeometry::TextAnchor { at, text, .. } => {
                assert!(at.x.abs() < 1e-6 && at.y.abs() < 1e-6);
                assert_eq!(text, "reviewed");
            }
            other => panic!("expected TextAnchor, got {other:?}"),
        }
        let bounds = geometry.bounds();
        assert_eq!((bounds.x, bounds.y), (0.0, 0.0));
    }

    #[test]
    fn from_markup_captures_no_geometry_for_stamps() {
        // Stamps draw from their own preset, not from captured geometry, so a
        // stamp preset stays style-only and keeps today's stamp path.
        let stamp = Markup::new(
            0,
            MarkupKind::Stamp,
            MarkupGeometry::Bounds(Rect::new(5.0, 5.0, 40.0, 20.0)),
            "Alice",
        );
        let preset = StylePreset::from_markup(&stamp, "Approved");
        assert!(!preset.can_draw());
        assert!(preset.geometry.is_none());
    }

    #[test]
    fn from_style_captures_no_geometry() {
        let preset = StylePreset::from_style(
            "Tool",
            MarkupKind::Line,
            MarkupStyle::default(),
            "Eve",
        );
        assert!(!preset.can_draw(), "a tool style has nothing drawn yet");
        assert!(preset.geometry.is_none());
    }

    #[test]
    fn without_geometry_keeps_every_other_field() {
        let preset = StylePreset::from_markup(&sample_markup(), "Draw Rect");
        assert!(preset.can_draw());
        let style_only = preset.without_geometry();
        assert!(!style_only.can_draw());
        assert!(style_only.geometry.is_none());
        assert_eq!(style_only.id, preset.id);
        assert_eq!(style_only.name, preset.name);
        assert_eq!(style_only.kind, preset.kind);
        assert_eq!(style_only.style, preset.style);
        assert_eq!(style_only.author, preset.author);
        assert_eq!(style_only.subject, preset.subject);
        assert_eq!(style_only.status, preset.status);
        assert_eq!(style_only.locked, preset.locked);
        assert_eq!(style_only.created, preset.created);
    }

    #[test]
    fn style_preset_serialize_roundtrip_with_geometry() {
        let preset = StylePreset::from_markup(&sample_markup(), "With Geometry");
        assert!(preset.can_draw(), "from_markup must capture geometry");
        let json = serde_json::to_string(&preset).unwrap();
        assert!(json.contains("\"geometry\""), "geometry must be written");
        let back: StylePreset = serde_json::from_str(&json).unwrap();
        assert_eq!(preset, back, "geometry must survive the round trip");
    }

    #[test]
    fn preset_without_geometry_serializes_without_the_field() {
        // `skip_serializing_if` keeps legacy preset files byte-stable: a
        // style-only preset must not grow a `"geometry": null` key.
        let preset = StylePreset::from_style(
            "Legacy",
            MarkupKind::Rectangle,
            MarkupStyle::default(),
            "Ann",
        );
        let json = serde_json::to_string(&preset).unwrap();
        assert!(!json.contains("geometry"), "no geometry key: {json}");
        let back: StylePreset = serde_json::from_str(&json).unwrap();
        assert_eq!(preset, back);
    }

    #[test]
    fn legacy_preset_json_without_geometry_fields_loads() {
        // A preset file written before Drawing Mode existed has no `geometry`
        // key at all; `#[serde(default)]` must turn that into `None`.
        let preset = StylePreset::from_markup(&sample_markup(), "Old");
        let mut value = serde_json::to_value(&preset).unwrap();
        let object = value
            .as_object_mut()
            .expect("a preset serialises to a JSON object");
        assert!(
            object.remove("geometry").is_some(),
            "precondition: the field is written today"
        );
        let legacy: StylePreset = serde_json::from_value(value).unwrap();
        assert!(
            legacy.geometry.is_none(),
            "legacy presets load as style-only"
        );
        assert!(!legacy.can_draw());
        assert_eq!(legacy.name, preset.name);
        assert_eq!(legacy.kind, preset.kind);
        assert_eq!(legacy.style, preset.style);
    }

    #[test]
    fn library_roundtrip_keeps_captured_geometry() {
        let mut lib = StylePresetLibrary::new();
        lib.save_named(StylePreset::from_markup(&sample_markup(), "Drawn"));
        let json = serde_json::to_string(&lib).unwrap();
        let back: StylePresetLibrary = serde_json::from_str(&json).unwrap();
        assert_eq!(lib, back);
        let stored = back.find_by_name("Drawn").expect("preset survives");
        assert!(stored.can_draw(), "geometry survives the library round trip");
    }
}
