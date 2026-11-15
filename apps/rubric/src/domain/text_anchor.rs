//! Glyph-range text anchoring.
//!
//! Text-anchored markups (highlight, strikeout, underline and squiggly) are
//! stored as one extracted text line: an origin point, the line's width and its
//! height. That is enough to draw a whole-line band, but not enough to draw
//! only the words a user selected, and it cannot express a drag-selection at
//! all.
//!
//! This module supplies the missing pure layer. A [`GlyphLine`] holds the
//! per-character boxes of one extracted line in page points (y-down), and a
//! [`GlyphRange`] is a half-open index range into it. From those two,
//! [`GlyphLine::range_bounds`] gives the tight on-page rectangle of a
//! selection, [`GlyphLine::range_for_rect`] turns a marquee back into a range,
//! [`GlyphLine::index_at`] answers "which character is under the pointer", and
//! [`GlyphLine::range_of_substring`] recovers the range of the text a markup
//! already stores — so glyph-tight anchoring needs no change to the persisted
//! schema.
//!
//! Everything here is serialisable and headless: the PDF text extractor
//! produces the boxes and the canvas draws the result, but neither owns these
//! rules.

use serde::{Deserialize, Serialize};

use super::geom::{Pt, Rect};

/// A half-open range of glyph indices within one extracted text line.
///
/// `start` is inclusive and `end` is exclusive, so `GlyphRange::new(0, 0)` is
/// empty and `GlyphRange::new(2, 2)` is empty too. Constructors normalise a
/// reversed pair instead of panicking, because the range can come from a
/// backwards drag or from deserialised data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GlyphRange {
    /// Index of the first covered glyph.
    pub start: usize,
    /// Index one past the last covered glyph.
    pub end: usize,
}

impl GlyphRange {
    /// Build a range from two indices, normalising a reversed pair so that
    /// `start <= end`.
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    /// The range covering every glyph of a line of `len` glyphs.
    pub fn all(len: usize) -> Self {
        Self { start: 0, end: len }
    }

    /// The range covering exactly the glyph at `index`.
    ///
    /// Saturates rather than overflowing at the top of the index space, so a
    /// hostile deserialised value cannot panic.
    pub fn single(index: usize) -> Self {
        Self {
            start: index,
            end: index.saturating_add(1),
        }
    }

    /// Fit the range inside a line of `len` glyphs.
    ///
    /// A range that starts past the end of the line collapses to the empty
    /// range at the end, which keeps callers free of special cases.
    pub fn clamped(self, len: usize) -> Self {
        let start = self.start.min(len);
        let end = self.end.min(len).max(start);
        Self { start, end }
    }

    /// Number of glyphs covered.
    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Whether the range covers no glyphs.
    pub fn is_empty(self) -> bool {
        self.end <= self.start
    }

    /// Whether `index` falls inside the range.
    pub fn contains(self, index: usize) -> bool {
        index >= self.start && index < self.end
    }
}

/// One character's box on the page, in page points (y-down).
///
/// The character is kept so a line can be turned back into text for report
/// export, substring search and accessibility descriptions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphBox {
    /// The character this box draws.
    pub ch: char,
    /// The character's box in page points (y-down).
    pub rect: Rect,
}

impl GlyphBox {
    /// A character box.
    pub fn new(ch: char, rect: Rect) -> Self {
        Self { ch, rect }
    }
}

/// One extracted text line: its per-character boxes and its line height.
///
/// `glyphs` is ordered left to right, so a glyph's index is also its position
/// in the line's text. `line_height` is carried for rendering only; bounds come
/// from the glyph boxes, so a degenerate height cannot make a markup
/// unhittable.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GlyphLine {
    /// The line's characters, left to right.
    pub glyphs: Vec<GlyphBox>,
    /// Rendered line height in page points, clamped to be non-negative.
    pub line_height: f32,
}

impl GlyphLine {
    /// Build a line from its glyph boxes and line height.
    ///
    /// `line_height` is clamped to at least zero, which also maps `NaN` to zero,
    /// so a malformed extractor result cannot poison later geometry.
    pub fn new(glyphs: Vec<GlyphBox>, line_height: f32) -> Self {
        Self {
            glyphs,
            line_height: line_height.max(0.0),
        }
    }

    /// Number of glyphs on the line.
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// Whether the line holds no glyphs.
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// The line's text, left to right.
    pub fn text(&self) -> String {
        self.glyphs.iter().map(|g| g.ch).collect()
    }

    /// The text covered by `range`, after clamping it to the line.
    pub fn text_range(&self, range: GlyphRange) -> String {
        let range = range.clamped(self.glyphs.len());
        self.glyphs[range.start..range.end]
            .iter()
            .map(|g| g.ch)
            .collect()
    }

    /// Tight bounding box of the whole line, or `None` when it has no glyphs.
    pub fn bounds(&self) -> Option<Rect> {
        self.range_bounds(GlyphRange::all(self.glyphs.len()))
    }

    /// Tight bounding box of the glyphs covered by `range`.
    ///
    /// Returns `None` for an empty or fully out-of-range selection, so callers
    /// can fall back to the stored line geometry instead of drawing a
    /// zero-area band.
    pub fn range_bounds(&self, range: GlyphRange) -> Option<Rect> {
        let range = range.clamped(self.glyphs.len());
        if range.is_empty() {
            return None;
        }
        let mut covered = self.glyphs[range.start..range.end].iter().map(|g| g.rect);
        let first = covered.next()?;
        Some(covered.fold(first, |acc, rect| acc.union(&rect)))
    }

    /// The glyph index under `p`, or the nearest one on the same line.
    ///
    /// A pointer inside a character's box reports that character. A pointer
    /// beside the line but within its vertical band reports the horizontally
    /// nearest glyph (so a click just past the end of a word still selects
    /// it). A pointer outside the band reports `None`, which is how a click on
    /// another line, or off the text entirely, is rejected.
    pub fn index_at(&self, p: Pt) -> Option<usize> {
        if self.glyphs.is_empty() {
            return None;
        }
        if let Some(index) = self.glyphs.iter().position(|g| g.rect.contains(p)) {
            return Some(index);
        }
        let band = self.bounds()?;
        if p.y < band.y || p.y > band.bottom() {
            return None;
        }
        self.glyphs
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let da = (a.rect.center().x - p.x).abs();
                let db = (b.rect.center().x - p.x).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| index)
    }

    /// The range of glyphs that `rect` touches, for marquee text selection.
    ///
    /// Returns an empty range at index zero when nothing overlaps, which the
    /// caller can test with [`GlyphRange::is_empty`].
    pub fn range_for_rect(&self, rect: Rect) -> GlyphRange {
        let mut start = None;
        let mut end = 0usize;
        for (index, glyph) in self.glyphs.iter().enumerate() {
            if glyph.rect.intersects(&rect) {
                if start.is_none() {
                    start = Some(index);
                }
                end = index + 1;
            }
        }
        match start {
            Some(start) => GlyphRange::new(start, end),
            None => GlyphRange::new(0, 0),
        }
    }

    /// The range covering the first occurrence of `needle` at or after glyph
    /// index `from`.
    ///
    /// This is how a stored `TextAnchor`'s text becomes glyph-tight geometry
    /// without a schema change: the anchor keeps the marked-up string, and the
    /// extracted line supplies the boxes. An empty needle or one longer than
    /// the line yields `None`.
    pub fn range_of_substring(&self, needle: &str, from: usize) -> Option<GlyphRange> {
        let needle: Vec<char> = needle.chars().collect();
        if needle.is_empty() || needle.len() > self.glyphs.len() {
            return None;
        }
        let haystack: Vec<char> = self.glyphs.iter().map(|g| g.ch).collect();
        let first = from.min(haystack.len());
        let last = haystack.len() - needle.len();
        if first > last {
            return None;
        }
        (first..=last)
            .find(|&i| haystack[i..i + needle.len()].iter().eq(needle.iter()))
            .map(|i| GlyphRange::new(i, i + needle.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three 10 pt wide glyphs spanning `0..30` at 12 pt tall.
    fn abc() -> GlyphLine {
        GlyphLine::new(
            vec![
                GlyphBox::new('a', Rect::new(0.0, 0.0, 10.0, 12.0)),
                GlyphBox::new('b', Rect::new(10.0, 0.0, 10.0, 12.0)),
                GlyphBox::new('c', Rect::new(20.0, 0.0, 10.0, 12.0)),
            ],
            12.0,
        )
    }

    /// The line "alpha beta" with one 10 pt wide box per character.
    fn alpha_beta() -> GlyphLine {
        let mut glyphs = Vec::new();
        for (i, ch) in "alpha beta".chars().enumerate() {
            let x = i as f32 * 10.0;
            glyphs.push(GlyphBox::new(ch, Rect::new(x, 0.0, 10.0, 12.0)));
        }
        GlyphLine::new(glyphs, 12.0)
    }

    #[test]
    fn glyph_range_normalises_a_reversed_pair() {
        let r = GlyphRange::new(5, 2);
        assert_eq!(r.start, 2);
        assert_eq!(r.end, 5);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn glyph_range_clamps_to_the_line_length() {
        assert_eq!(GlyphRange::new(3, 99).clamped(5), GlyphRange::new(3, 5));
        // Starting past the end collapses to the empty range at the end.
        let past = GlyphRange::new(9, 12).clamped(5);
        assert_eq!(past, GlyphRange::new(5, 5));
        assert!(past.is_empty());
    }

    #[test]
    fn glyph_range_len_and_contains() {
        let r = GlyphRange::new(1, 4);
        assert_eq!(r.len(), 3);
        assert!(!r.contains(0));
        assert!(r.contains(1));
        assert!(r.contains(3));
        assert!(!r.contains(4));
        assert!(GlyphRange::default().is_empty());
    }

    #[test]
    fn glyph_range_single_saturates_at_the_index_limit() {
        let r = GlyphRange::single(usize::MAX);
        assert_eq!(r.start, usize::MAX);
        assert_eq!(r.end, usize::MAX);
        assert!(r.is_empty(), "saturating rather than overflowing");
    }

    #[test]
    fn glyph_line_text_and_text_range() {
        let line = alpha_beta();
        assert_eq!(line.text(), "alpha beta");
        assert_eq!(line.text_range(GlyphRange::new(6, 10)), "beta");
        assert_eq!(line.text_range(GlyphRange::new(0, 5)), "alpha");
        // Out-of-range requests clamp instead of panicking.
        assert_eq!(line.text_range(GlyphRange::new(8, 999)), "ta");
    }

    #[test]
    fn glyph_line_bounds_unions_every_box() {
        assert_eq!(abc().bounds(), Some(Rect::new(0.0, 0.0, 30.0, 12.0)));
        assert_eq!(GlyphLine::default().bounds(), None);
    }

    #[test]
    fn range_bounds_is_tight_for_a_sub_range() {
        let line = abc();
        assert_eq!(
            line.range_bounds(GlyphRange::new(1, 2)),
            Some(Rect::new(10.0, 0.0, 10.0, 12.0))
        );
        assert_eq!(
            line.range_bounds(GlyphRange::new(0, 2)),
            Some(Rect::new(0.0, 0.0, 20.0, 12.0))
        );
    }

    #[test]
    fn range_bounds_is_none_for_an_empty_or_past_the_end_range() {
        let line = abc();
        assert_eq!(line.range_bounds(GlyphRange::new(2, 2)), None);
        assert_eq!(line.range_bounds(GlyphRange::new(9, 12)), None);
    }

    #[test]
    fn index_at_returns_the_containing_glyph() {
        assert_eq!(abc().index_at(Pt::new(15.0, 6.0)), Some(1));
        assert_eq!(abc().index_at(Pt::new(0.0, 0.0)), Some(0));
    }

    #[test]
    fn index_at_snaps_horizontally_beside_the_line() {
        // Left of the line, inside its vertical band: nearest is the first glyph.
        assert_eq!(abc().index_at(Pt::new(-5.0, 6.0)), Some(0));
        // Right of the line: nearest is the last glyph.
        assert_eq!(abc().index_at(Pt::new(60.0, 6.0)), Some(2));
    }

    #[test]
    fn index_at_is_none_outside_the_line_band() {
        assert_eq!(abc().index_at(Pt::new(15.0, 50.0)), None);
        assert_eq!(GlyphLine::default().index_at(Pt::new(0.0, 0.0)), None);
    }

    #[test]
    fn range_for_rect_selects_the_intersecting_glyphs() {
        // A 10 pt wide marquee straddling the a/b boundary covers both.
        let range = abc().range_for_rect(Rect::new(5.0, 0.0, 10.0, 12.0));
        assert_eq!(range, GlyphRange::new(0, 2));
        assert_eq!(abc().text_range(range), "ab");
    }

    #[test]
    fn range_for_rect_is_empty_when_nothing_overlaps() {
        let range = abc().range_for_rect(Rect::new(500.0, 500.0, 10.0, 10.0));
        assert!(range.is_empty());
    }

    #[test]
    fn range_of_substring_finds_a_word_and_respects_the_start_offset() {
        let line = alpha_beta();
        assert_eq!(
            line.range_of_substring("beta", 0),
            Some(GlyphRange::new(6, 10))
        );
        assert_eq!(
            line.range_of_substring("alpha", 0),
            Some(GlyphRange::new(0, 5))
        );
        // Starting after the word finds nothing.
        assert_eq!(line.range_of_substring("beta", 7), None);
    }

    #[test]
    fn range_of_substring_rejects_missing_and_oversized_needles() {
        let line = alpha_beta();
        assert_eq!(line.range_of_substring("gamma", 0), None);
        assert_eq!(line.range_of_substring("", 0), None);
        assert_eq!(line.range_of_substring(&"x".repeat(99), 0), None);
    }

    #[test]
    fn glyph_line_clamps_a_negative_or_nan_line_height() {
        assert_eq!(GlyphLine::new(Vec::new(), -3.0).line_height, 0.0);
        assert_eq!(GlyphLine::new(Vec::new(), f32::NAN).line_height, 0.0);
    }

    #[test]
    fn glyph_line_serde_roundtrip() {
        let line = alpha_beta();
        let json = serde_json::to_string(&line).expect("serialize line");
        let back: GlyphLine = serde_json::from_str(&json).expect("deserialize line");
        assert_eq!(line, back);
    }

    #[test]
    fn glyph_range_serde_roundtrip() {
        let range = GlyphRange::new(3, 7);
        let json = serde_json::to_string(&range).expect("serialize range");
        let back: GlyphRange = serde_json::from_str(&json).expect("deserialize range");
        assert_eq!(range, back);
    }
}
