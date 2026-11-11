//! Built-in stamp presets.
//!
//! Stamps convey a review verdict from a fixed library rather than free text.
//! The set mixes status marks (`Approved`, `Rejected`, `Draft`), which draw
//! their word, with utility marks (`Check`, `Cross`), which draw a glyph.

use serde::{Deserialize, Serialize};

use super::style::{palette, Color};

/// Built-in stamp presets (spec F-111).
///
/// Colours are grouped by meaning rather than being pairwise-distinct:
/// affirmative marks are green, negative marks red, and in-progress marks
/// amber. This mirrors the visual convention of check/cross marks and does
/// not try to give every preset a unique hue (contrast
/// [`crate::domain::ReviewStatus`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StampPreset {
    /// Signed-off status mark; draws its word.
    Approved,
    /// Not-accepted status mark; draws its word.
    Rejected,
    /// Work-in-progress status mark; draws its word.
    Draft,
    /// Affirmative utility mark; draws a tick glyph.
    Check,
    /// Negative utility mark; draws a cross glyph.
    Cross,
}

impl StampPreset {
    /// All presets in display order (stamp picker).
    pub const ALL: [StampPreset; 5] = [
        StampPreset::Approved,
        StampPreset::Rejected,
        StampPreset::Draft,
        StampPreset::Check,
        StampPreset::Cross,
    ];

    /// Word label used in menus, pickers and accessibility text.
    ///
    /// Always a word, even for [`StampPreset::Check`] and
    /// [`StampPreset::Cross`], so UI controls stay readable. Use
    /// [`StampPreset::glyph`] for the content a renderer draws.
    pub fn label(self) -> &'static str {
        match self {
            StampPreset::Approved => "APPROVED",
            StampPreset::Rejected => "REJECTED",
            StampPreset::Draft => "DRAFT",
            StampPreset::Check => "CHECK",
            StampPreset::Cross => "CROSS",
        }
    }

    /// Content a renderer draws for this preset.
    ///
    /// Status marks draw their word; check/cross draw a glyph. Renderers should
    /// centre this within the stamp's bounds.
    pub fn glyph(self) -> &'static str {
        match self {
            StampPreset::Approved => "APPROVED",
            StampPreset::Rejected => "REJECTED",
            StampPreset::Draft => "DRAFT",
            StampPreset::Check => "✓",
            StampPreset::Cross => "✗",
        }
    }

    /// Stamp colour, grouped by meaning (not pairwise-distinct).
    pub fn color(self) -> Color {
        match self {
            StampPreset::Approved => palette::GREEN,
            StampPreset::Rejected => palette::RED,
            StampPreset::Draft => palette::AMBER,
            StampPreset::Check => palette::GREEN,
            StampPreset::Cross => palette::RED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamp_preset_all_matches_spec_set() {
        assert_eq!(
            StampPreset::ALL,
            [
                StampPreset::Approved,
                StampPreset::Rejected,
                StampPreset::Draft,
                StampPreset::Check,
                StampPreset::Cross,
            ]
        );
        assert_eq!(StampPreset::ALL.len(), 5);
    }

    #[test]
    fn stamp_preset_labels_are_words_glyphs_are_drawn_content() {
        for preset in StampPreset::ALL {
            assert!(!preset.label().is_empty());
            assert!(!preset.glyph().is_empty());
        }
        assert_eq!(StampPreset::Approved.label(), "APPROVED");
        assert_eq!(StampPreset::Rejected.label(), "REJECTED");
        assert_eq!(StampPreset::Draft.label(), "DRAFT");
        assert_eq!(StampPreset::Check.label(), "CHECK");
        assert_eq!(StampPreset::Cross.label(), "CROSS");
        // Status marks draw their word; utility marks draw a glyph.
        assert_eq!(StampPreset::Approved.glyph(), "APPROVED");
        assert_eq!(StampPreset::Rejected.glyph(), "REJECTED");
        assert_eq!(StampPreset::Draft.glyph(), "DRAFT");
        assert_eq!(StampPreset::Check.glyph(), "✓");
        assert_eq!(StampPreset::Cross.glyph(), "✗");
    }

    #[test]
    fn stamp_preset_colors_group_by_meaning() {
        // Affirmative green, negative red, in-progress amber. Unlike
        // ReviewStatus, stamp presets are intentionally not pairwise-distinct.
        assert_eq!(StampPreset::Approved.color(), palette::GREEN);
        assert_eq!(StampPreset::Check.color(), palette::GREEN);
        assert_eq!(StampPreset::Rejected.color(), palette::RED);
        assert_eq!(StampPreset::Cross.color(), palette::RED);
        assert_eq!(StampPreset::Draft.color(), palette::AMBER);
    }

    #[test]
    fn stamp_preset_serde_uses_snake_case() {
        let cases = [
            (StampPreset::Approved, "\"approved\""),
            (StampPreset::Rejected, "\"rejected\""),
            (StampPreset::Draft, "\"draft\""),
            (StampPreset::Check, "\"check\""),
            (StampPreset::Cross, "\"cross\""),
        ];
        for (preset, json) in cases {
            assert_eq!(serde_json::to_string(&preset).expect("serialize"), json);
            let back: StampPreset = serde_json::from_str(json).expect("deserialize");
            assert_eq!(back, preset);
        }
    }
}
