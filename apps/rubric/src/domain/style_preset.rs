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

/// A named grouping of presets ("tool set"). A preset belongs to at most one
/// set; deleting a set leaves its presets in the library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresetSet {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub preset_ids: Vec<Uuid>,
}

/// A collection of reusable style presets.
///
/// The library keeps at most 64 presets. Adding a preset with an existing id
/// moves it to the back (upsert dedupe); when full the oldest preset is
/// dropped. Presets may be grouped into named [`PresetSet`]s (gap #7).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StylePresetLibrary {
    pub presets: Vec<StylePreset>,
    #[serde(default)]
    pub sets: Vec<PresetSet>,
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

    /// Create a new named set. Returns a reference to it.
    pub fn create_set(&mut self, name: impl Into<String>) -> &PresetSet {
        let set = PresetSet {
            id: Uuid::new_v4(),
            name: name.into(),
            preset_ids: Vec::new(),
        };
        self.sets.push(set);
        self.sets.last().unwrap()
    }

    /// Rename a set. The new name must be non-empty and unique among sets
    /// (case-insensitive). Returns `true` on success.
    pub fn rename_set(&mut self, id: Uuid, name: impl Into<String>) -> bool {
        let name = name.into();
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return false;
        }
        if self
            .sets
            .iter()
            .any(|s| s.id != id && s.name.eq_ignore_ascii_case(trimmed))
        {
            return false;
        }
        if let Some(s) = self.sets.iter_mut().find(|s| s.id == id) {
            s.name = trimmed.to_string();
            true
        } else {
            false
        }
    }

    /// Delete a set. Its presets stay in the library — only the grouping is
    /// removed. Returns `true` if the set existed.
    pub fn delete_set(&mut self, id: Uuid) -> bool {
        let before = self.sets.len();
        self.sets.retain(|s| s.id != id);
        self.sets.len() != before
    }

    /// Add a preset to a set, removing it from any other set first so a preset
    /// belongs to at most one set. Returns `true` if the set exists.
    pub fn add_to_set(&mut self, set_id: Uuid, preset_id: Uuid) -> bool {
        for s in &mut self.sets {
            if s.id != set_id {
                s.preset_ids.retain(|p| *p != preset_id);
            }
        }
        if let Some(s) = self.sets.iter_mut().find(|s| s.id == set_id) {
            if !s.preset_ids.contains(&preset_id) {
                s.preset_ids.push(preset_id);
            }
            true
        } else {
            false
        }
    }

    /// Remove a preset from a set. Returns `true` if it was present.
    pub fn remove_from_set(&mut self, set_id: Uuid, preset_id: Uuid) -> bool {
        if let Some(s) = self.sets.iter_mut().find(|s| s.id == set_id) {
            let before = s.preset_ids.len();
            s.preset_ids.retain(|p| *p != preset_id);
            s.preset_ids.len() != before
        } else {
            false
        }
    }

    /// The set containing a given preset, if any.
    pub fn set_of(&self, preset_id: Uuid) -> Option<Uuid> {
        self.sets
            .iter()
            .find(|s| s.preset_ids.contains(&preset_id))
            .map(|s| s.id)
    }

    /// Sets paired with their member counts, for UI display.
    pub fn sets_with_counts(&self) -> Vec<(PresetSet, usize)> {
        self.sets
            .iter()
            .map(|s| (s.clone(), s.preset_ids.len()))
            .collect()
    }
}

/// Serialisable shape for an exported tool set (`.rubric-tools.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolSetExport {
    pub schema: u32,
    pub name: String,
    pub presets: Vec<StylePreset>,
}

/// Serialize one set (and its member presets) to pretty JSON. Pure function so
/// it can be tested without file dialogs.
pub fn export_set_json(lib: &StylePresetLibrary, set_id: Uuid) -> Result<String, String> {
    let set = lib
        .sets
        .iter()
        .find(|s| s.id == set_id)
        .ok_or_else(|| "set not found".to_string())?;
    let mut presets = Vec::new();
    for pid in &set.preset_ids {
        if let Some(p) = lib.find(*pid) {
            presets.push(p.clone());
        }
    }
    let export = ToolSetExport {
        schema: 1,
        name: set.name.clone(),
        presets,
    };
    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

/// Import a tool set from JSON, adding presets as independent copies (fresh
/// ids, renamed on case-insensitive duplicate) and creating a new set. Returns
/// the number of presets imported.
pub fn import_set_json(lib: &mut StylePresetLibrary, json: &str) -> Result<usize, String> {
    let parsed: ToolSetExport = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if parsed.schema != 1 {
        return Err("unsupported schema version".to_string());
    }
    let mut set_name = parsed.name.clone();
    if lib
        .sets
        .iter()
        .any(|s| s.name.eq_ignore_ascii_case(&set_name))
    {
        set_name = format!("{set_name} (imported)");
    }
    let mut new_ids = Vec::new();
    for mut p in parsed.presets.clone() {
        p.id = Uuid::new_v4();
        if lib.find_by_name(&p.name).is_some() {
            p.name = format!("{} (imported)", p.name);
        }
        let id = p.id;
        lib.add(p);
        new_ids.push(id);
    }
    lib.sets.push(PresetSet {
        id: Uuid::new_v4(),
        name: set_name,
        preset_ids: new_ids,
    });
    Ok(parsed.presets.len())
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
    }

    #[test]
    fn style_preset_library_serialize_roundtrip() {
        let mut lib = StylePresetLibrary::new();
        lib.add(StylePreset::from_markup(&sample_markup(), "X"));
        let json = serde_json::to_string(&lib).unwrap();
        let back: StylePresetLibrary = serde_json::from_str(&json).unwrap();
        assert_eq!(lib, back);
    }

    #[test]
    fn legacy_library_json_without_sets_loads() {
        let json = r#"{"presets":[]}"#;
        let lib: StylePresetLibrary = serde_json::from_str(json).unwrap();
        assert!(lib.sets.is_empty());
    }
}
