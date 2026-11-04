//! Bates numbering (spec F-801–F-805): sequential page stamps for legal and
//! document-review workflows.
//!
//! A *Bates number* is a unique, page-ordered identifier placed on each page of
//! a document set (for example `SMITH000001`, `SMITH000002`). Rubric applies it
//! as a **non-destructive overlay**: the stamp is drawn into the composed page
//! image on export/print and previewed on the canvas; the source PDF is never
//! modified. Capability-level research lives in `docs/research/bates-numbering.md`.

use serde::{Deserialize, Serialize};

/// Minimum zero-pad width for the numeric body.
pub const MIN_BATES_DIGITS: u8 = 1;
/// Maximum zero-pad width for the numeric body.
pub const MAX_BATES_DIGITS: u8 = 12;
/// Default zero-pad width (`SMITH000001` → six digits).
pub const DEFAULT_BATES_DIGITS: u8 = 6;
/// Upper bound for the start value, kept below `u64::MAX` so the UI drag value
/// and `value + offset` arithmetic can never overflow in practice.
pub const MAX_BATES_START: u64 = 99_999_999;

/// Page corner a Bates stamp is anchored to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BatesCorner {
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    TopRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner (default: the conventional Bates position).
    #[default]
    BottomRight,
}

impl BatesCorner {
    /// Every corner, in menu order.
    pub const ALL: [BatesCorner; 4] = [
        BatesCorner::TopLeft,
        BatesCorner::TopRight,
        BatesCorner::BottomLeft,
        BatesCorner::BottomRight,
    ];

    /// Human-readable label for menus.
    pub fn label(self) -> &'static str {
        match self {
            BatesCorner::TopLeft => "Top left",
            BatesCorner::TopRight => "Top right",
            BatesCorner::BottomLeft => "Bottom left",
            BatesCorner::BottomRight => "Bottom right",
        }
    }

    /// Whether the stamp hugs the right edge (text is right-aligned).
    pub fn is_right(self) -> bool {
        matches!(self, BatesCorner::TopRight | BatesCorner::BottomRight)
    }

    /// Whether the stamp hugs the top edge.
    pub fn is_top(self) -> bool {
        matches!(self, BatesCorner::TopLeft | BatesCorner::TopRight)
    }
}

/// Inclusive page range the stamps apply to, using 0-based page indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BatesRange {
    /// First page (0-based).
    pub from: usize,
    /// Last page (0-based, inclusive); `None` means "through the last page".
    pub to: Option<usize>,
}

impl BatesRange {
    /// Whether `page` (0-based) is inside the range.
    pub fn includes(&self, page: usize) -> bool {
        page >= self.from && self.to.is_none_or(|to| page <= to)
    }
}

/// Bates numbering configuration (spec F-801–F-803).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatesConfig {
    /// Whether stamping is active. Disabled by default so a fresh project never
    /// stamps without the reviewer asking for it.
    pub enabled: bool,
    /// Free-text prefix before the numeric body (for example a party or matter
    /// code).
    pub prefix: String,
    /// Free-text suffix after the numeric body.
    pub suffix: String,
    /// Value of the first in-range page; subsequent pages increment by one.
    pub start: u64,
    /// Zero-pad width of the numeric body.
    pub digits: u8,
    /// Corner the stamp is anchored to.
    pub corner: BatesCorner,
    /// Inset from the page edges, in page points.
    pub margin: f32,
    /// Stamp font size, in page points.
    pub font_size: f32,
    /// Pages the stamps apply to.
    pub range: BatesRange,
}

impl Default for BatesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            prefix: "RUBRIC".to_string(),
            suffix: String::new(),
            start: 1,
            digits: DEFAULT_BATES_DIGITS,
            corner: BatesCorner::BottomRight,
            margin: 24.0,
            font_size: 10.0,
            range: BatesRange::default(),
        }
    }
}

impl BatesConfig {
    /// Set the zero-pad width, clamped to the supported range.
    ///
    /// Prefer this over assigning the field directly so an out-of-range value
    /// from a UI widget can never be persisted.
    pub fn set_digits(&mut self, digits: u8) {
        self.digits = digits.clamp(MIN_BATES_DIGITS, MAX_BATES_DIGITS);
    }

    /// The zero-pad width actually used when formatting (defensive clamp so a
    /// hand-edited project file cannot format an absurd width).
    pub fn effective_digits(&self) -> usize {
        self.digits.clamp(MIN_BATES_DIGITS, MAX_BATES_DIGITS) as usize
    }

    /// Format the numeric body for absolute value `value`.
    pub fn format_number(&self, value: u64) -> String {
        let width = self.effective_digits();
        let body = format!("{:0width$}", value, width = width);
        format!("{}{}{}", self.prefix, body, self.suffix)
    }

    /// The stamp value for `page` (0-based), or `None` when the page is outside
    /// the configured range.
    ///
    /// Numbering starts at [`BatesConfig::start`] on the first page *in range*
    /// and advances by one for each subsequent in-range page, so a range that
    /// begins mid-document still yields the expected sequence.
    pub fn value_for_page(&self, page: usize) -> Option<u64> {
        if !self.range.includes(page) {
            return None;
        }
        Some(self.start + (page - self.range.from) as u64)
    }

    /// The full stamp string for `page`, or `None` when the page is not stamped.
    pub fn stamp_for_page(&self, page: usize) -> Option<String> {
        self.value_for_page(page).map(|v| self.format_number(v))
    }

    /// The 0-based index of the last page that will be stamped in a document of
    /// `page_count` pages, or `None` when nothing is stamped.
    pub fn last_stamped_page(&self, page_count: usize) -> Option<usize> {
        if page_count == 0 {
            return None;
        }
        let last = page_count - 1;
        let from = self.range.from.min(last);
        let to = self.range.to.unwrap_or(last).min(last);
        if to < from {
            None
        } else {
            Some(to)
        }
    }

    /// How many pages will be stamped in a document of `page_count` pages.
    pub fn applied_count(&self, page_count: usize) -> usize {
        match self.last_stamped_page(page_count) {
            Some(to) => to - self.range.from.min(page_count - 1) + 1,
            None => 0,
        }
    }

    /// The stamp for the last in-range page, for a range preview.
    pub fn last_stamp(&self, page_count: usize) -> Option<String> {
        self.last_stamped_page(page_count)
            .and_then(|page| self.stamp_for_page(page))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_disabled_and_sane() {
        let b = BatesConfig::default();
        assert!(!b.enabled);
        assert_eq!(b.digits, DEFAULT_BATES_DIGITS);
        assert_eq!(b.corner, BatesCorner::BottomRight);
        assert_eq!(b.range, BatesRange { from: 0, to: None });
    }

    #[test]
    fn format_pads_to_digit_width() {
        let b = BatesConfig {
            prefix: "SMITH".to_string(),
            suffix: "X".to_string(),
            digits: 6,
            ..BatesConfig::default()
        };
        assert_eq!(b.format_number(1), "SMITH000001X");
        assert_eq!(b.format_number(42), "SMITH000042X");
    }

    #[test]
    fn format_without_prefix_or_suffix() {
        let b = BatesConfig {
            prefix: String::new(),
            suffix: String::new(),
            digits: 4,
            ..BatesConfig::default()
        };
        assert_eq!(b.format_number(7), "0007");
    }

    #[test]
    fn digits_clamp_to_supported_range() {
        let mut b = BatesConfig::default();
        b.set_digits(0);
        assert_eq!(b.digits, MIN_BATES_DIGITS);
        b.set_digits(200);
        assert_eq!(b.digits, MAX_BATES_DIGITS);

        // A hand-edited width is clamped defensively at format time too.
        let b2 = BatesConfig {
            digits: 250,
            ..BatesConfig::default()
        };
        assert_eq!(b2.effective_digits(), MAX_BATES_DIGITS as usize);
    }

    #[test]
    fn sequence_advances_one_per_in_range_page() {
        let b = BatesConfig {
            start: 100,
            digits: 4,
            range: BatesRange {
                from: 2,
                to: Some(4),
            },
            ..BatesConfig::default()
        };
        assert_eq!(b.value_for_page(0), None);
        assert_eq!(b.value_for_page(1), None);
        assert_eq!(b.value_for_page(2), Some(100));
        assert_eq!(b.value_for_page(3), Some(101));
        assert_eq!(b.value_for_page(4), Some(102));
        assert_eq!(b.value_for_page(5), None);
    }

    #[test]
    fn open_range_runs_to_the_last_page() {
        let b = BatesConfig {
            start: 1,
            digits: 3,
            range: BatesRange { from: 0, to: None },
            ..BatesConfig::default()
        };
        assert_eq!(b.stamp_for_page(9).as_deref(), Some("RUBRIC010"));
        assert_eq!(b.applied_count(10), 10);
    }

    #[test]
    fn applied_count_handles_empty_and_clamped_ranges() {
        let b = BatesConfig::default();
        assert_eq!(b.applied_count(0), 0);

        let reversed = BatesConfig {
            range: BatesRange {
                from: 5,
                to: Some(2),
            },
            ..BatesConfig::default()
        };
        assert_eq!(reversed.applied_count(10), 0);
        assert_eq!(reversed.last_stamped_page(10), None);

        // A `from` beyond the document is clamped to the last page.
        let beyond = BatesConfig {
            range: BatesRange { from: 99, to: None },
            ..BatesConfig::default()
        };
        assert_eq!(beyond.applied_count(3), 1);
        assert_eq!(beyond.last_stamped_page(3), Some(2));
    }

    #[test]
    fn last_stamp_matches_the_final_page() {
        let b = BatesConfig {
            start: 1,
            digits: 4,
            range: BatesRange {
                from: 0,
                to: Some(3),
            },
            ..BatesConfig::default()
        };
        assert_eq!(b.last_stamp(10).as_deref(), Some("RUBRIC0004"));
        assert_eq!(b.last_stamp(0), None);
    }

    #[test]
    fn corner_helpers_match_position() {
        assert!(BatesCorner::TopRight.is_right());
        assert!(BatesCorner::BottomRight.is_right());
        assert!(!BatesCorner::TopLeft.is_right());
        assert!(BatesCorner::TopLeft.is_top());
        assert!(!BatesCorner::BottomLeft.is_top());
        assert_eq!(BatesCorner::ALL.len(), 4);
        assert_eq!(BatesCorner::BottomRight.label(), "Bottom right");
    }

    #[test]
    fn config_serde_roundtrips() {
        let b = BatesConfig {
            enabled: true,
            prefix: "ABC".to_string(),
            suffix: "Z".to_string(),
            start: 500,
            digits: 8,
            corner: BatesCorner::TopLeft,
            range: BatesRange {
                from: 1,
                to: Some(3),
            },
            ..BatesConfig::default()
        };
        let json = serde_json::to_string(&b).expect("serialize");
        let back: BatesConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, b);
    }

    #[test]
    fn range_includes_is_inclusive() {
        let r = BatesRange {
            from: 2,
            to: Some(5),
        };
        assert!(!r.includes(1));
        assert!(r.includes(2));
        assert!(r.includes(5));
        assert!(!r.includes(6));
    }
}
