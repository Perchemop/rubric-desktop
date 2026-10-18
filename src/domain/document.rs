//! Document and page model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::geom::Rect;
use super::markup::Markup;
use super::units::Scale;

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

    pub fn clear_markups(&mut self) {
        if !self.markups.is_empty() {
            self.markups.clear();
            self.touch();
        }
    }

    pub fn markups_on_page(&self, page: usize) -> Vec<&Markup> {
        self.markups.iter().filter(|m| m.page == page).collect()
    }

    pub fn find_markup(&self, id: Uuid) -> Option<&Markup> {
        self.markups.iter().find(|m| m.id == id)
    }

    pub fn find_markup_mut(&mut self, id: Uuid) -> Option<&mut Markup> {
        self.markups.iter_mut().find(|m| m.id == id)
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
