//! Document and page model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::geom::Rect;
use super::markup::Markup;
use super::style::MarkupStyle;
use super::units::Scale;

/// A named group of markups that can be hidden or locked together.
///
/// Layers also define paint order: markups are painted layer by layer in
/// [`Document::layers`] order, and within a layer in [`Document::markups`]
/// vector order (the later markup is drawn on top). See
/// [`Document::markups_in_paint_order`]. Reordering layers therefore changes
/// stacking, and [`Layer::z`] mirrors the layer's position (kept in sync by
/// [`Document::move_layer_up`] / [`Document::move_layer_down`]).
///
/// The first layer is the document's default layer: markups with
/// `layer_id == None` belong to it, and it is kept at the bottom of the layer
/// order so those markups never silently change stacking when other layers are
/// reordered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    /// Stable identifier referenced by [`Markup::layer_id`].
    pub id: Uuid,
    /// Human-readable name (names are not required to be unique).
    pub name: String,
    /// When false, markups on this layer are not painted or hit-tested.
    pub visible: bool,
    /// When true, markups on this layer cannot be edited.
    pub locked: bool,
    /// Stacking hint reserved for future layer ordering; metadata only today.
    pub z: i64,
}

impl Layer {
    /// Create a visible, unlocked layer with a fresh id.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            visible: true,
            locked: false,
            z: 0,
        }
    }

    /// The default layer created for new and legacy documents.
    pub fn default_layer() -> Self {
        Self::new("Layer 1")
    }
}

/// Error returned by [`Document::remove_layer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum LayerError {
    /// No layer with the supplied id exists.
    #[error("layer not found")]
    NotFound,
    /// The document's default layer cannot be removed.
    #[error("cannot remove the default layer")]
    DefaultLayer,
    /// The layer still has markups and must be emptied first.
    #[error("cannot remove a layer that still contains markups")]
    NotEmpty,
}

/// Serde default for documents saved before layers existed.
fn default_layers() -> Vec<Layer> {
    vec![Layer::default_layer()]
}

/// Deserialize a layer list, repairing an empty list from legacy or hand-edited
/// files so every document keeps a default layer.
fn deserialize_layers<'de, D>(deserializer: D) -> Result<Vec<Layer>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut layers = Vec::<Layer>::deserialize(deserializer)?;
    if layers.is_empty() {
        layers.push(Layer::default_layer());
    }
    Ok(layers)
}

/// A single page in a document.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Page {
    /// Page index (0-based) within the document.
    pub index: usize,
    /// Width in PDF points.
    pub width: f32,
    /// Height in PDF points.
    pub height: f32,
    /// Rotation in degrees: 0, 90, 180, 270.
    pub rotation: i32,
    /// Drawing scale applied to this page.
    pub scale: Scale,
}

impl Page {
    /// ISO A4: 210mm × 297mm = 595.28pt × 841.89pt (rounded).
    pub fn a4(index: usize) -> Self {
        Self {
            index,
            width: 595.28,
            height: 841.89,
            rotation: 0,
            scale: Scale::PAPER,
        }
    }

    /// US Letter: 8.5in × 11in = 612pt × 792pt.
    pub fn letter(index: usize) -> Self {
        Self {
            index,
            width: 612.0,
            height: 792.0,
            rotation: 0,
            scale: Scale::PAPER,
        }
    }

    /// Legal: 8.5in × 14in = 612pt × 1008pt.
    pub fn legal(index: usize) -> Self {
        Self {
            index,
            width: 612.0,
            height: 1008.0,
            rotation: 0,
            scale: Scale::PAPER,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width.max(1.0);
        self.height = height.max(1.0);
        self
    }

    pub fn with_scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width, self.height)
    }

    /// Orientation-sensitive bounds (ignores rotation for now; rotation tracked).
    pub fn effective_bounds(&self) -> Rect {
        let (w, h) = if self.rotation % 180 == 0 {
            (self.width, self.height)
        } else {
            (self.height, self.width)
        };
        Rect::new(0.0, 0.0, w, h)
    }
}

/// A document (one or more pages) containing markups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    #[serde(default)]
    pub source_path: Option<String>,
    pub pages: Vec<Page>,
    #[serde(default)]
    pub markups: Vec<Markup>,
    /// Visibility/lock layers, in panel order. Always non-empty for documents
    /// built or loaded through this module; the first layer is the default
    /// layer for markups whose [`Markup::layer_id`] is `None`.
    ///
    /// Markups are painted in [`Document::markups`] order, regardless of the
    /// order of this vector.
    #[serde(default = "default_layers", deserialize_with = "deserialize_layers")]
    pub layers: Vec<Layer>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

impl Document {
    pub fn new(title: impl Into<String>, pages: Vec<Page>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            source_path: None,
            pages,
            markups: Vec::new(),
            layers: default_layers(),
            created: now,
            modified: now,
        }
    }

    pub fn empty(title: impl Into<String>) -> Self {
        Self::new(title, vec![Page::a4(0)])
    }

    pub fn add_page(&mut self, mut page: Page) {
        page.index = self.pages.len();
        self.pages.push(page);
        self.touch();
    }

    pub fn remove_page(&mut self, index: usize) {
        if index < self.pages.len() {
            self.pages.remove(index);
            for (i, p) in self.pages.iter_mut().enumerate() {
                p.index = i;
            }
            self.markups.retain(|m| m.page < self.pages.len());
            self.touch();
        }
    }

    pub fn add_markup(&mut self, mut m: Markup) {
        if self.pages.is_empty() {
            self.pages.push(Page::a4(0));
        }
        if m.page >= self.pages.len() {
            m.page = self.pages.len().saturating_sub(1);
        }
        if let Some(p) = self.pages.get(m.page) {
            m.recompute_measurement(p.scale);
        }
        self.markups.push(m);
        self.touch();
    }

    pub fn remove_markup(&mut self, id: Uuid) {
        if let Some(pos) = self.markups.iter().position(|m| m.id == id) {
            self.markups.remove(pos);
            self.touch();
        }
    }

    /// Move `id` one step later in paint order among markups on its page.
    ///
    /// Paint order is layer-major (see [`Document::markups_in_paint_order`]), so
    /// a one-step move stays within the markup's page **and layer**: moving past
    /// a markup on another layer would have no visible effect. Returns `false`
    /// when the markup does not exist or is already last in its layer on that
    /// page.
    pub fn bring_forward(&mut self, id: Uuid) -> bool {
        let Some(index) = self.markups.iter().position(|m| m.id == id) else {
            return false;
        };
        let (page, layer) = (
            self.markups[index].page,
            self.layer_id_for(&self.markups[index]),
        );
        let Some(next) = self
            .markups
            .iter()
            .enumerate()
            .skip(index + 1)
            .find(|(_, m)| m.page == page && self.layer_id_for(m) == layer)
            .map(|(i, _)| i)
        else {
            return false;
        };
        self.markups.swap(index, next);
        self.touch();
        true
    }

    /// Move `id` one step earlier in paint order among markups on the same page
    /// **and layer**.
    ///
    /// See [`Document::bring_forward`] for the page/layer-scoped rule.
    pub fn send_backward(&mut self, id: Uuid) -> bool {
        let Some(index) = self.markups.iter().position(|m| m.id == id) else {
            return false;
        };
        let (page, layer) = (
            self.markups[index].page,
            self.layer_id_for(&self.markups[index]),
        );
        let Some(previous) = self.markups[..index]
            .iter()
            .rposition(|m| m.page == page && self.layer_id_for(m) == layer)
        else {
            return false;
        };
        self.markups.swap(index, previous);
        self.touch();
        true
    }

    /// Move `id` to the top of its layer on that page (drawn on top within the
    /// layer).
    ///
    /// Returns `false` when the markup does not exist or is already last in its
    /// layer on that page.
    pub fn bring_to_front(&mut self, id: Uuid) -> bool {
        let Some(index) = self.markups.iter().position(|m| m.id == id) else {
            return false;
        };
        let (page, layer) = (
            self.markups[index].page,
            self.layer_id_for(&self.markups[index]),
        );
        let Some(last) = self
            .markups
            .iter()
            .rposition(|m| m.page == page && self.layer_id_for(m) == layer)
        else {
            return false;
        };
        if last == index {
            return false;
        }
        let markup = self.markups.remove(index);
        self.markups.insert(last, markup);
        self.touch();
        true
    }

    /// Move `id` to the bottom of its layer on that page (drawn at the bottom
    /// within the layer).
    ///
    /// Returns `false` when the markup does not exist or is already first in its
    /// layer on that page.
    pub fn send_to_back(&mut self, id: Uuid) -> bool {
        let Some(index) = self.markups.iter().position(|m| m.id == id) else {
            return false;
        };
        let (page, layer) = (
            self.markups[index].page,
            self.layer_id_for(&self.markups[index]),
        );
        let Some(first) = self
            .markups
            .iter()
            .position(|m| m.page == page && self.layer_id_for(m) == layer)
        else {
            return false;
        };
        if first == index {
            return false;
        }
        let markup = self.markups.remove(index);
        self.markups.insert(first, markup);
        self.touch();
        true
    }

    /// Apply `style` to every editable markup in `ids` and return the number
    /// changed.
    ///
    /// Missing and locked ids are skipped. Only the visual style (and each
    /// markup's `modified` timestamp) changes; geometry, metadata, review state,
    /// comments, lock state and measurements are untouched.
    pub fn apply_style_to(&mut self, ids: &[Uuid], style: MarkupStyle) -> usize {
        let mut changed = 0;
        for id in ids {
            let editable = self
                .find_markup(*id)
                .is_some_and(|markup| self.is_markup_editable(markup));
            if !editable {
                continue;
            }
            if let Some(markup) = self.find_markup_mut(*id) {
                markup.apply_style(style);
                changed += 1;
            }
        }
        if changed > 0 {
            self.touch();
        }
        changed
    }

    pub fn clear_markups(&mut self) {
        if !self.markups.is_empty() {
            self.markups.clear();
            self.touch();
        }
    }

    pub fn markups_on_page(&self, page: usize) -> Vec<&Markup> {
        self.markups.iter().filter(|m| m.page == page).collect()
    }

    /// Visible markups on `page` in paint order (back to front).
    ///
    /// Paint order is layer-major: markups are grouped by their resolved layer
    /// ([`Document::layer_id_for`]) in [`Document::layers`] order, and within a
    /// layer they keep [`Document::markups`] vector order. This is the order the
    /// canvas paints in, and hit testing walks it in reverse.
    ///
    /// A markup whose resolved layer is not in `layers` (a hand-edited or future
    /// file) is fail-open: it is appended in vector order rather than dropped.
    pub fn markups_in_paint_order(&self, page: usize) -> Vec<&Markup> {
        let mut out: Vec<&Markup> = Vec::new();
        for layer in &self.layers {
            for m in &self.markups {
                if m.page == page
                    && self.is_markup_visible(m)
                    && self.layer_id_for(m) == Some(layer.id)
                {
                    out.push(m);
                }
            }
        }
        for m in &self.markups {
            if m.page != page || !self.is_markup_visible(m) {
                continue;
            }
            if !self
                .layers
                .iter()
                .any(|l| Some(l.id) == self.layer_id_for(m))
            {
                out.push(m);
            }
        }
        out
    }

    /// Rename a layer. Blank (whitespace-only) names are refused so a layer
    /// cannot become unlabelled. Returns `false` when the layer is missing or
    /// the name is blank.
    pub fn rename_layer(&mut self, id: Uuid, name: &str) -> bool {
        let name = name.trim();
        if name.is_empty() {
            return false;
        }
        let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) else {
            return false;
        };
        if layer.name == name {
            return false;
        }
        layer.name = name.to_string();
        self.touch();
        true
    }

    /// Move a layer one step later in paint order (towards the top).
    ///
    /// The default layer (always index 0) is pinned at the bottom so markups
    /// with `layer_id == None` never change stacking implicitly; moving it, or
    /// moving a layer that is already last, returns `false`. [`Layer::z`] is
    /// rewritten to the layer's index so the hint stays truthful.
    pub fn move_layer_up(&mut self, id: Uuid) -> bool {
        self.move_layer(id, -1)
    }

    /// Move a layer one step earlier in paint order (towards the bottom).
    ///
    /// See [`Document::move_layer_up`]; the default layer is pinned.
    pub fn move_layer_down(&mut self, id: Uuid) -> bool {
        self.move_layer(id, 1)
    }

    /// Shared layer-reorder helper: `delta` is `-1` for up (later in paint
    /// order) or `1` for down, with the default layer pinned at index 0.
    fn move_layer(&mut self, id: Uuid, delta: i32) -> bool {
        let Some(index) = self.layers.iter().position(|l| l.id == id) else {
            return false;
        };
        let target = index as i32 + delta;
        // Pin the default layer (index 0) at the bottom: neither it nor the
        // layer directly above it may swap past index 0.
        if index == 0 || target <= 0 || target as usize >= self.layers.len() {
            return false;
        }
        self.layers.swap(index, target as usize);
        for (z, layer) in self.layers.iter_mut().enumerate() {
            layer.z = z as i64;
        }
        self.touch();
        true
    }

    pub fn find_markup(&self, id: Uuid) -> Option<&Markup> {
        self.markups.iter().find(|m| m.id == id)
    }

    pub fn find_markup_mut(&mut self, id: Uuid) -> Option<&mut Markup> {
        self.markups.iter_mut().find(|m| m.id == id)
    }

    /// The document's default layer (the first layer in [`Document::layers`]).
    pub fn default_layer(&self) -> Option<&Layer> {
        self.layers.first()
    }

    /// Identifier of the default layer, or `Uuid::nil()` if the layer list is
    /// empty (only reachable for values built outside this module).
    pub fn default_layer_id(&self) -> Uuid {
        self.layers.first().map(|l| l.id).unwrap_or_default()
    }

    /// Resolve the layer a markup belongs to.
    ///
    /// A markup with [`Markup::layer_id`] `None` belongs to the default layer.
    /// `None` is returned only when the document has no layers at all.
    pub fn layer_id_for(&self, markup: &Markup) -> Option<Uuid> {
        markup
            .layer_id
            .or_else(|| self.default_layer().map(|l| l.id))
    }

    /// Look up a layer by id.
    pub fn layer(&self, id: Uuid) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Add a visible, unlocked layer and return its id.
    ///
    /// Blank names are replaced with `Layer N` (where `N` is the next slot),
    /// and the [`Layer::z`] hint is set above every existing layer. The new
    /// layer is appended, so it does not affect paint order.
    pub fn add_layer(&mut self, name: impl Into<String>) -> Uuid {
        let name = name.into();
        let name = name.trim();
        let name = if name.is_empty() {
            format!("Layer {}", self.layers.len() + 1)
        } else {
            name.to_string()
        };
        let z = self
            .layers
            .iter()
            .map(|l| l.z)
            .max()
            .unwrap_or(-1)
            .saturating_add(1);
        let id = Uuid::new_v4();
        self.layers.push(Layer {
            id,
            name,
            visible: true,
            locked: false,
            z,
        });
        self.touch();
        id
    }

    /// Remove an empty, non-default layer.
    ///
    /// Policy: the default layer can never be removed, and a layer that still
    /// contains markups is refused rather than silently relocating
    /// annotations. Callers that want to delete a populated layer must move its
    /// markups first (for example by setting [`Markup::layer_id`] to `None` to
    /// return them to the default layer).
    pub fn remove_layer(&mut self, id: Uuid) -> Result<(), LayerError> {
        if self.layer(id).is_none() {
            return Err(LayerError::NotFound);
        }
        if self.layers.first().is_some_and(|l| l.id == id) {
            return Err(LayerError::DefaultLayer);
        }
        if self
            .markups
            .iter()
            .any(|m| self.layer_id_for(m) == Some(id))
        {
            return Err(LayerError::NotEmpty);
        }
        self.layers.retain(|l| l.id != id);
        self.touch();
        Ok(())
    }

    /// Set layer visibility; returns `false` when the layer does not exist.
    ///
    /// Hidden layers affect painting and hit testing only; they do not lock
    /// editing. See [`Document::is_markup_editable`].
    pub fn set_layer_visible(&mut self, id: Uuid, visible: bool) -> bool {
        let changed = match self.layers.iter_mut().find(|l| l.id == id) {
            Some(layer) => {
                let changed = layer.visible != visible;
                layer.visible = visible;
                changed
            }
            None => return false,
        };
        if changed {
            self.touch();
        }
        true
    }

    /// Set layer locking; returns `false` when the layer does not exist.
    ///
    /// Locked layers block editing of their markups but do not hide them.
    pub fn set_layer_locked(&mut self, id: Uuid, locked: bool) -> bool {
        let changed = match self.layers.iter_mut().find(|l| l.id == id) {
            Some(layer) => {
                let changed = layer.locked != locked;
                layer.locked = locked;
                changed
            }
            None => return false,
        };
        if changed {
            self.touch();
        }
        true
    }

    /// Markups assigned to `id`, including default-layer markups when `id` is
    /// the default layer.
    pub fn markups_on_layer(&self, id: Uuid) -> Vec<&Markup> {
        self.markups
            .iter()
            .filter(|m| self.layer_id_for(m) == Some(id))
            .collect()
    }

    /// Whether the layer identified by `id` should be painted/hit-tested.
    ///
    /// `None` refers to the default layer. Unknown ids and a missing default
    /// layer are treated as visible (fail-open) so markups with stale layer
    /// references never become unreachable.
    pub fn is_layer_visible(&self, id: Option<Uuid>) -> bool {
        self.layer_for(id).is_none_or(|l| l.visible)
    }

    /// Whether a markup's layer is currently visible.
    pub fn is_markup_visible(&self, markup: &Markup) -> bool {
        self.is_layer_visible(markup.layer_id)
    }

    /// Whether a markup can be edited.
    ///
    /// A markup is editable when neither it nor its layer is locked. Layer
    /// visibility deliberately does not affect editability: a hidden layer can
    /// still be modified (for example via a future layer panel or inspector).
    pub fn is_markup_editable(&self, markup: &Markup) -> bool {
        !markup.locked && self.layer_for(markup.layer_id).is_none_or(|l| !l.locked)
    }

    /// The layer a markup resolves to, falling back to the default layer when
    /// [`Markup::layer_id`] is `None`.
    fn layer_for(&self, id: Option<Uuid>) -> Option<&Layer> {
        match id {
            Some(id) => self.layer(id),
            None => self.default_layer(),
        }
    }

    pub fn touch(&mut self) {
        self.modified = Utc::now();
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn primary_page(&self) -> Option<Page> {
        self.pages.first().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::markup::{MarkupGeometry, MarkupKind};

    fn markup() -> Markup {
        Markup::new(
            0,
            MarkupKind::Rectangle,
            MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 10.0, 10.0)),
            "tester",
        )
    }

    fn page_ids(doc: &Document, page: usize) -> Vec<Uuid> {
        doc.markups_on_page(page)
            .into_iter()
            .map(|markup| markup.id)
            .collect()
    }

    #[test]
    fn z_order_stays_within_the_page_and_stops_at_edges() {
        let mut doc = Document::empty("Doc");
        doc.add_page(Page::a4(1));
        let mut a = markup();
        a.page = 0;
        let mut b = markup();
        b.page = 0;
        let mut c = markup();
        c.page = 0;
        let mut x = markup();
        x.page = 1;
        let mut y = markup();
        y.page = 1;
        let (a_id, b_id, c_id, x_id, y_id) = (a.id, b.id, c.id, x.id, y.id);
        // Interleave the pages so skipping over a page boundary is visible.
        for markup in [a, b, x, c, y] {
            doc.add_markup(markup);
        }

        assert_eq!(page_ids(&doc, 0), vec![a_id, b_id, c_id]);
        assert_eq!(page_ids(&doc, 1), vec![x_id, y_id]);
        assert!(doc.bring_forward(b_id), "skips the page-1 markup");
        assert_eq!(page_ids(&doc, 0), vec![a_id, c_id, b_id]);
        assert_eq!(page_ids(&doc, 1), vec![x_id, y_id]);
        assert!(doc.send_backward(b_id));
        assert_eq!(page_ids(&doc, 0), vec![a_id, b_id, c_id]);
        assert!(!doc.send_backward(a_id), "already at the back");
        assert!(
            !doc.bring_forward(c_id),
            "only following markup is on another page"
        );
        assert_eq!(page_ids(&doc, 1), vec![x_id, y_id]);
        assert!(doc.bring_to_front(a_id));
        assert_eq!(page_ids(&doc, 0), vec![b_id, c_id, a_id]);
        assert_eq!(page_ids(&doc, 1), vec![x_id, y_id]);
        assert!(!doc.bring_to_front(a_id), "already at the front");
        assert!(doc.send_to_back(a_id));
        assert_eq!(page_ids(&doc, 0), vec![a_id, b_id, c_id]);
        assert!(!doc.send_to_back(a_id), "already at the back");

        assert!(doc.bring_forward(x_id));
        assert_eq!(page_ids(&doc, 1), vec![y_id, x_id]);
        assert!(doc.send_backward(x_id));
        assert_eq!(page_ids(&doc, 1), vec![x_id, y_id]);
        assert!(!doc.send_backward(x_id), "already at the back");
        assert!(!doc.bring_forward(y_id), "already at the front");
    }

    #[test]
    fn apply_style_to_skips_locked_and_missing_markups() {
        let mut doc = Document::empty("Doc");
        let mut first = markup();
        first.author = "First".to_string();
        first.subject = "Keep me".to_string();
        first.status = crate::domain::markup::ReviewStatus::Approved;
        let first_id = first.id;
        let mut locked = markup();
        locked.locked = true;
        let locked_id = locked.id;
        let locked_style = locked.style;
        doc.add_markup(first);
        doc.add_markup(locked);

        let style = MarkupStyle::default().with_stroke(crate::domain::style::palette::BLUE);
        let changed = doc.apply_style_to(&[first_id, locked_id, Uuid::new_v4()], style);

        assert_eq!(changed, 1);
        let first = doc.find_markup(first_id).unwrap();
        assert_eq!(first.style, style);
        assert_eq!(first.author, "First");
        assert_eq!(first.subject, "Keep me");
        assert_eq!(first.status, crate::domain::markup::ReviewStatus::Approved);
        assert_eq!(doc.find_markup(locked_id).unwrap().style, locked_style);
    }

    #[test]
    fn new_document_has_one_visible_unlocked_default_layer() {
        let doc = Document::empty("Doc");
        assert_eq!(doc.layers.len(), 1);
        let layer = doc.default_layer().expect("default layer");
        assert_eq!(layer.name, "Layer 1");
        assert!(layer.visible);
        assert!(!layer.locked);
        assert_eq!(layer.z, 0);
        assert_eq!(doc.default_layer_id(), layer.id);
        assert_eq!(doc.layer(layer.id), Some(layer));
    }

    #[test]
    fn add_layer_creates_visible_unlocked_layer_with_rising_z() {
        let mut doc = Document::empty("Doc");
        let second = doc.add_layer("Notes");
        let third = doc.add_layer("   ");
        assert_eq!(doc.layers.len(), 3);
        assert_eq!(doc.layer(second).unwrap().name, "Notes");
        assert_eq!(doc.layer(third).unwrap().name, "Layer 3");
        assert!(doc.layer(second).unwrap().visible);
        assert!(!doc.layer(second).unwrap().locked);
        assert!(doc.layer(second).unwrap().z > doc.default_layer().unwrap().z);
        assert!(doc.layer(third).unwrap().z > doc.layer(second).unwrap().z);
        assert_ne!(second, third);
    }

    #[test]
    fn set_layer_visibility_and_locking() {
        let mut doc = Document::empty("Doc");
        let id = doc.add_layer("Notes");
        assert!(doc.set_layer_visible(id, false));
        assert!(!doc.layer(id).unwrap().visible);
        assert!(doc.set_layer_locked(id, true));
        assert!(doc.layer(id).unwrap().locked);

        let missing = Uuid::new_v4();
        assert!(!doc.set_layer_visible(missing, false));
        assert!(!doc.set_layer_locked(missing, true));
    }

    #[test]
    fn remove_layer_refuses_default_and_populated_layers() {
        let mut doc = Document::empty("Doc");
        let default = doc.default_layer_id();
        assert_eq!(doc.remove_layer(default), Err(LayerError::DefaultLayer));

        let empty = doc.add_layer("Empty");
        assert_eq!(doc.remove_layer(empty), Ok(()));
        assert!(doc.layer(empty).is_none());
        assert_eq!(doc.remove_layer(empty), Err(LayerError::NotFound));

        let busy = doc.add_layer("Busy");
        doc.add_markup(markup().with_layer(busy));
        assert_eq!(doc.remove_layer(busy), Err(LayerError::NotEmpty));
        assert_eq!(doc.markups_on_layer(busy).len(), 1);

        // Moving the markup back to the default layer unblocks removal.
        doc.markups[0].layer_id = None;
        assert_eq!(doc.remove_layer(busy), Ok(()));
    }

    #[test]
    fn markups_resolve_to_default_layer() {
        let mut doc = Document::empty("Doc");
        doc.add_markup(markup());
        let default = doc.default_layer_id();
        assert_eq!(doc.markups[0].layer_id, None);
        assert_eq!(doc.layer_id_for(&doc.markups[0]), Some(default));
        assert_eq!(doc.markups_on_layer(default).len(), 1);
        assert!(doc.is_markup_visible(&doc.markups[0]));
        assert!(doc.is_markup_editable(&doc.markups[0]));
    }

    #[test]
    fn hidden_layer_affects_visibility_but_not_editing() {
        let mut doc = Document::empty("Doc");
        let id = doc.add_layer("Notes");
        doc.add_markup(markup().with_layer(id));

        doc.set_layer_visible(id, false);
        assert!(!doc.is_layer_visible(Some(id)));
        assert!(!doc.is_markup_visible(&doc.markups[0]));
        assert!(
            doc.is_markup_editable(&doc.markups[0]),
            "hiding a layer must not lock editing"
        );

        doc.set_layer_visible(id, true);
        assert!(doc.is_markup_visible(&doc.markups[0]));
    }

    #[test]
    fn locked_layer_blocks_editing_but_not_visibility() {
        let mut doc = Document::empty("Doc");
        let id = doc.add_layer("Notes");
        doc.add_markup(markup().with_layer(id));

        doc.set_layer_locked(id, true);
        assert!(doc.is_markup_visible(&doc.markups[0]));
        assert!(!doc.is_markup_editable(&doc.markups[0]));

        // A per-markup lock also blocks editing, independently of its layer.
        let default = doc.default_layer_id();
        doc.add_markup(markup());
        doc.markups[1].locked = true;
        assert_eq!(doc.layer_id_for(&doc.markups[1]), Some(default));
        assert!(!doc.is_markup_editable(&doc.markups[1]));
    }

    #[test]
    fn unknown_layer_ids_are_fail_open() {
        let mut doc = Document::empty("Doc");
        doc.add_markup(markup().with_layer(Uuid::new_v4()));
        assert!(doc.is_layer_visible(Some(Uuid::new_v4())));
        assert!(doc.is_markup_visible(&doc.markups[0]));
        assert!(doc.is_markup_editable(&doc.markups[0]));
    }

    #[test]
    fn document_with_two_layers_serde_roundtrip() {
        let mut doc = Document::empty("Doc");
        let id = doc.add_layer("Notes");
        doc.set_layer_visible(id, false);
        doc.set_layer_locked(id, true);
        doc.add_markup(markup().with_layer(id));

        let json = serde_json::to_string(&doc).expect("serialize");
        let back: Document = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, doc);
        assert_eq!(back.layers.len(), 2);
        assert_eq!(back.layers[1].id, id);
        assert!(!back.layers[1].visible);
        assert!(back.layers[1].locked);
        assert_eq!(back.markups[0].layer_id, Some(id));
    }

    #[test]
    fn legacy_document_without_layers_loads_default_layer() {
        let mut markup_json = serde_json::to_value(markup()).expect("markup json");
        let markup_obj = markup_json.as_object_mut().expect("markup object");
        markup_obj.remove("layer_id");
        markup_obj.insert("layer".into(), serde_json::Value::String("Review".into()));

        let mut value = serde_json::to_value(Document::empty("Legacy")).expect("document json");
        let doc_obj = value.as_object_mut().expect("document object");
        doc_obj.remove("layers");
        doc_obj.insert(
            "markups".into(),
            serde_json::Value::Array(vec![markup_json]),
        );

        let legacy: Document = serde_json::from_value(value).expect("deserialize legacy");
        assert_eq!(legacy.layers.len(), 1);
        let default = legacy.default_layer_id();
        assert_eq!(legacy.markups.len(), 1);
        assert_eq!(legacy.markups[0].layer_id, None);
        assert_eq!(legacy.markups[0].layer.as_deref(), Some("Review"));
        assert_eq!(legacy.layer_id_for(&legacy.markups[0]), Some(default));
        assert_eq!(legacy.markups_on_layer(default).len(), 1);
        assert!(legacy.is_markup_visible(&legacy.markups[0]));
        assert!(legacy.is_markup_editable(&legacy.markups[0]));
    }

    #[test]
    fn empty_layers_list_is_repaired_on_load() {
        let mut value = serde_json::to_value(Document::empty("Doc")).expect("json");
        value
            .as_object_mut()
            .expect("document object")
            .insert("layers".into(), serde_json::Value::Array(Vec::new()));

        let loaded: Document = serde_json::from_value(value).expect("deserialize");
        assert_eq!(loaded.layers.len(), 1);
        assert_eq!(loaded.layers[0].name, "Layer 1");
    }

    #[test]
    fn rename_layer_updates_name_and_rejects_blank() {
        let mut doc = Document::empty("Doc");
        let id = doc.add_layer("Old");
        assert!(doc.rename_layer(id, "New"));
        assert_eq!(doc.layer(id).unwrap().name, "New");
        assert!(!doc.rename_layer(id, "   "));
        assert_eq!(doc.layer(id).unwrap().name, "New");
        assert!(!doc.rename_layer(Uuid::new_v4(), "X"));
    }

    #[test]
    fn move_layer_up_and_down_respects_default_pin_and_updates_z() {
        let mut doc = Document::empty("Doc");
        let a = doc.add_layer("A");
        let b = doc.add_layer("B");
        // order: default(0), A(1), B(2)
        assert_eq!(doc.layers[0].id, doc.default_layer_id());
        assert_eq!(doc.layers[1].id, a);
        assert_eq!(doc.layers[2].id, b);
        // move B up (swaps with A, which is allowed)
        assert!(doc.move_layer_up(b));
        assert_eq!(doc.layers[1].id, b);
        assert_eq!(doc.layers[2].id, a);
        // move B back down
        assert!(doc.move_layer_down(b));
        assert_eq!(doc.layers[1].id, a);
        assert_eq!(doc.layers[2].id, b);
        // A may not swap past the pinned default layer (index 0)
        assert!(!doc.move_layer_up(a));
        // B is already last
        assert!(!doc.move_layer_down(b));
        // default cannot be moved
        let def = doc.default_layer_id();
        assert!(!doc.move_layer_up(def));
        assert!(!doc.move_layer_down(def));
        // z mirrors position
        for (i, l) in doc.layers.iter().enumerate() {
            assert_eq!(l.z, i as i64);
        }
    }

    #[test]
    fn markups_in_paint_order_respects_layer_order() {
        let mut doc = Document::empty("Doc");
        let l1 = doc.add_layer("L1");
        let l2 = doc.add_layer("L2");
        let mut m_def = markup();
        m_def.page = 0;
        let mut m1 = markup().with_layer(l1);
        m1.page = 0;
        let mut m2 = markup().with_layer(l2);
        m2.page = 0;
        doc.add_markup(m_def.clone());
        doc.add_markup(m1.clone());
        doc.add_markup(m2.clone());
        let order = doc.markups_in_paint_order(0);
        // default first, then L1, then L2
        assert_eq!(order[0].id, m_def.id);
        assert_eq!(order[1].id, m1.id);
        assert_eq!(order[2].id, m2.id);
        // reorder layers: move L2 up (swap with L1)
        assert!(doc.move_layer_up(l2));
        let order2 = doc.markups_in_paint_order(0);
        assert_eq!(order2[1].id, m2.id);
        assert_eq!(order2[2].id, m1.id);
    }

    #[test]
    fn z_order_is_scoped_to_layer_and_page() {
        let mut doc = Document::empty("Doc");
        let l1 = doc.add_layer("L1");
        let l2 = doc.add_layer("L2");
        // Vector order: a(l1), c(l2), b(l1) — c sits between the two L1 markups.
        let mut a = markup().with_layer(l1);
        a.page = 0;
        let mut c = markup().with_layer(l2);
        c.page = 0;
        let mut b = markup().with_layer(l1);
        b.page = 0;
        doc.add_markup(a.clone());
        doc.add_markup(c.clone());
        doc.add_markup(b.clone());
        let ids = |d: &Document| d.markups.iter().map(|m| m.id).collect::<Vec<_>>();
        assert_eq!(ids(&doc), vec![a.id, c.id, b.id]);

        // bring_forward(a) skips c (different layer) and swaps with b.
        assert!(doc.bring_forward(a.id));
        assert_eq!(ids(&doc), vec![b.id, c.id, a.id]);
        // send_backward(a) swaps back with b.
        assert!(doc.send_backward(a.id));
        assert_eq!(ids(&doc), vec![a.id, c.id, b.id]);
        // bring_to_front(a) moves to the top of its layer (index of b).
        assert!(doc.bring_to_front(a.id));
        assert_eq!(ids(&doc), vec![c.id, b.id, a.id]);
        // send_to_back(a) moves below b, still after c in vector terms.
        assert!(doc.send_to_back(a.id));
        assert_eq!(ids(&doc), vec![c.id, a.id, b.id]);
        // b is already last in its layer.
        assert!(!doc.bring_forward(b.id));
        // c is isolated in L2, so neither direction moves it.
        assert!(!doc.bring_forward(c.id));
        assert!(!doc.send_backward(c.id));
    }
}
