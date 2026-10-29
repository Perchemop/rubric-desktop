//! Accessibility helpers (spec F-605).
//!
//! Pure, headless-testable WCAG 2.x contrast maths. The UI layer uses these to
//! audit the shipped palettes; keeping the maths in the domain means the audit
//! runs in `cargo test` without a display.

use super::style::Color;

/// One of the WCAG 2.x contrast requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WcagLevel {
    /// Level AA, normal-size text: contrast ≥ 4.5:1.
    AaNormal,
    /// Level AA, large text and UI components: contrast ≥ 3:1.
    AaLarge,
    /// Level AAA, normal-size text: contrast ≥ 7:1.
    AaaNormal,
}

impl WcagLevel {
    /// The minimum contrast ratio this level requires.
    pub fn min_ratio(self) -> f64 {
        match self {
            WcagLevel::AaNormal => 4.5,
            WcagLevel::AaLarge => 3.0,
            WcagLevel::AaaNormal => 7.0,
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            WcagLevel::AaNormal => "AA normal text",
            WcagLevel::AaLarge => "AA large text / UI",
            WcagLevel::AaaNormal => "AAA normal text",
        }
    }
}

/// A single sRGB channel (0–255) linearised for luminance, per WCAG 2.x.
fn linearise(channel: u8) -> f64 {
    let c = channel as f64 / 255.0;
    if c <= 0.039_28 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Relative luminance of an sRGB colour (WCAG 2.x). Alpha is ignored: the
/// audit composes colours against an opaque background before calling this.
pub fn relative_luminance(color: Color) -> f64 {
    0.2126 * linearise(color.r) + 0.7152 * linearise(color.g) + 0.0722 * linearise(color.b)
}

/// WCAG contrast ratio between two opaque colours, always ≥ 1.
pub fn contrast_ratio(a: Color, b: Color) -> f64 {
    let (l1, l2) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

/// Whether the pair meets `level`.
pub fn meets(level: WcagLevel, fg: Color, bg: Color) -> bool {
    contrast_ratio(fg, bg) >= level.min_ratio()
}

/// A palette pair name, foreground and background, audited together.
#[derive(Debug, Clone)]
pub struct AuditPair {
    pub name: String,
    pub fg: Color,
    pub bg: Color,
    pub level: WcagLevel,
}

impl AuditPair {
    pub fn new(
        name: impl Into<String>,
        fg: Color,
        bg: Color,
        level: WcagLevel,
    ) -> Self {
        Self {
            name: name.into(),
            fg,
            bg,
            level,
        }
    }
}

/// A pair that failed its required contrast.
#[derive(Debug, Clone)]
pub struct A11yIssue {
    pub name: String,
    pub ratio: f64,
    pub required: f64,
    pub level: WcagLevel,
}

impl A11yIssue {
    /// A one-line description for logs, reports and test failures.
    pub fn describe(&self) -> String {
        format!(
            "{}: ratio {:.2}:1 is below the required {:.1}:1 ({})",
            self.name,
            self.ratio,
            self.required,
            self.level.label()
        )
    }
}

/// Audit a set of colour pairs and return every failure, in input order.
pub fn audit(pairs: &[AuditPair]) -> Vec<A11yIssue> {
    pairs
        .iter()
        .filter_map(|p| {
            let ratio = contrast_ratio(p.fg, p.bg);
            (ratio < p.level.min_ratio()).then(|| A11yIssue {
                name: p.name.clone(),
                ratio,
                required: p.level.min_ratio(),
                level: p.level,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::palette;

    fn approx(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn black_on_white_is_the_maximum_ratio() {
        let ratio = contrast_ratio(palette::BLACK, palette::WHITE);
        assert!(approx(ratio, 21.0, 0.5), "expected ~21:1, got {ratio}");
    }

    #[test]
    fn identical_colours_have_ratio_one() {
        assert!(approx(
            contrast_ratio(palette::BLUE, palette::BLUE),
            1.0,
            1e-9
        ));
    }

    #[test]
    fn ratio_is_symmetric() {
        let a = contrast_ratio(palette::RED, palette::WHITE);
        let b = contrast_ratio(palette::WHITE, palette::RED);
        assert!(approx(a, b, 1e-9));
        assert!(a >= 1.0);
    }

    #[test]
    fn known_mid_grey_case() {
        // #767676 on white is the classic WCAG 4.5:1 boundary colour.
        let grey = Color::rgb(118, 118, 118);
        let ratio = contrast_ratio(grey, palette::WHITE);
        assert!(
            approx(ratio, 4.54, 0.05),
            "expected ~4.54:1, got {ratio}"
        );
        assert!(meets(WcagLevel::AaNormal, grey, palette::WHITE));
    }

    #[test]
    fn levels_report_their_minimums() {
        assert_eq!(WcagLevel::AaNormal.min_ratio(), 4.5);
        assert_eq!(WcagLevel::AaLarge.min_ratio(), 3.0);
        assert_eq!(WcagLevel::AaaNormal.min_ratio(), 7.0);
        assert!(WcagLevel::AaNormal.label().contains("AA"));
    }

    #[test]
    fn audit_reports_only_failures() {
        let light_grey = Color::rgb(200, 200, 200);
        let pairs = vec![
            AuditPair::new("ok", palette::BLACK, palette::WHITE, WcagLevel::AaNormal),
            AuditPair::new("bad", light_grey, palette::WHITE, WcagLevel::AaNormal),
        ];
        let issues = audit(&pairs);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].name, "bad");
        assert!(issues[0].ratio < 4.5);
        assert!(issues[0].describe().contains("below the required"));
    }

}
