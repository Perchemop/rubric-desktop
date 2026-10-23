//! Canvas grid, snap-to-grid and ruler geometry (spec F-405, F-406).
//!
//! Everything here is pure geometry so the canvas can stay a thin renderer and
//! the rules stay testable without a display.

use crate::domain::geom::Pt;

/// Canvas grid settings.
///
/// These describe how the *canvas* behaves, not what the document contains, so
/// they never enter `.rubric.json` (the same machine-local reasoning as the
/// theme decision in `DECISIONS.md`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    /// Distance between grid lines, in page points.
    pub spacing: f32,
    /// Whether grid lines are drawn under the page.
    pub visible: bool,
    /// Whether markup anchors land on grid intersections.
    pub snap: bool,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            spacing: 36.0,
            visible: true,
            snap: false,
        }
    }
}

impl Grid {
    /// Spacings offered by the View menu, in page points.
    pub const SPACINGS: [f32; 4] = [12.0, 24.0, 36.0, 72.0];

    /// Smallest grid pitch, in screen pixels, worth drawing. Below this the
    /// lines merge into an unreadable wash, so the grid is simply not drawn.
    pub const MIN_SPACING_PX: f32 = 6.0;

    /// Snap a page point to the nearest grid intersection. Returns the point
    /// untouched when snapping is off or the spacing is not usable.
    pub fn snap(&self, p: Pt) -> Pt {
        if !self.snap || self.spacing <= 0.0 || !self.spacing.is_finite() {
            return p;
        }
        Pt::new(snap_value(p.x, self.spacing), snap_value(p.y, self.spacing))
    }

    /// Grid pitch in screen pixels at the given zoom.
    pub fn spacing_px(&self, zoom: f32) -> f32 {
        self.spacing * zoom
    }

    /// Whether the grid is worth drawing at this zoom.
    pub fn should_draw(&self, zoom: f32) -> bool {
        self.visible && self.spacing_px(zoom) >= Self::MIN_SPACING_PX
    }
}

/// Round `v` to the nearest multiple of `step` (never floor: a point halfway
/// between two intersections belongs to the closer one).
fn snap_value(v: f32, step: f32) -> f32 {
    (v / step).round() * step
}

/// Smallest "nice" (1 / 2 / 5 × 10^k) step that is at least `min_step`, so
/// ruler labels stay short and readable at any zoom.
pub fn nice_step(min_step: f64) -> f64 {
    let min = if min_step > 0.0 && min_step.is_finite() {
        min_step
    } else {
        1.0
    };
    let base = 10f64.powf(min.log10().floor());
    for m in [1.0, 2.0, 5.0, 10.0] {
        let step = m * base;
        if step >= min {
            return step;
        }
    }
    10.0 * base
}

/// Tick values covering `[from, to]` (page points) whose spacing is at least
/// `min_step`, aligned to the step so ticks land on round numbers.
///
/// The result is strictly ascending and empty for an empty, inverted or
/// nonsensical range — a ruler must never loop forever.
pub fn ruler_ticks(from: f64, to: f64, min_step: f64) -> Vec<f64> {
    // NaN is not finite, so this also rejects NaN bounds outright.
    if !from.is_finite() || !to.is_finite() || !min_step.is_finite() {
        return Vec::new();
    }
    if to <= from || min_step <= 0.0 {
        return Vec::new();
    }
    let step = nice_step(min_step);
    let mut ticks = Vec::new();
    let mut v = (from / step).ceil() * step;
    // `+ step` guards against a float that lands just past `to` after rounding.
    while v <= to + step * 0.5 {
        ticks.push(v);
        v += step;
    }
    ticks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_grid_is_visible_with_snap_off() {
        let g = Grid::default();
        assert!(g.visible);
        assert!(!g.snap);
        assert!(g.spacing > 0.0);
    }

    #[test]
    fn spacings_are_positive_and_increasing() {
        let mut last = 0.0;
        for s in Grid::SPACINGS {
            assert!(s > last, "spacings must increase: {s} after {last}");
            last = s;
        }
    }

    #[test]
    fn snap_rounds_to_the_nearest_intersection() {
        let g = Grid {
            spacing: 10.0,
            visible: true,
            snap: true,
        };
        assert_eq!(g.snap(Pt::new(1.0, 1.0)), Pt::new(0.0, 0.0));
        assert_eq!(g.snap(Pt::new(4.0, -4.0)), Pt::new(0.0, 0.0));
        // Exactly halfway rounds up (Rust's `f32::round` semantics).
        assert_eq!(g.snap(Pt::new(5.0, 15.0)), Pt::new(10.0, 20.0));
        assert_eq!(g.snap(Pt::new(14.0, 26.0)), Pt::new(10.0, 30.0));
    }

    #[test]
    fn snap_is_off_by_default_and_ignores_bad_spacing() {
        let off = Grid::default();
        let p = Pt::new(13.7, -4.2);
        assert_eq!(off.snap(p), p);

        let broken = Grid {
            spacing: 0.0,
            visible: true,
            snap: true,
        };
        assert_eq!(broken.snap(p), p);
    }

    #[test]
    fn grid_is_drawn_only_when_the_pitch_resolves() {
        let g = Grid::default(); // 36 pt spacing
        assert!(g.should_draw(1.0));
        assert!(!g.should_draw(0.1)); // 3.6 px
        let hidden = Grid {
            visible: false,
            ..g
        };
        assert!(!hidden.should_draw(1.0));
    }

    #[test]
    fn nice_step_uses_a_one_two_five_sequence() {
        assert_eq!(nice_step(1.0), 1.0);
        assert_eq!(nice_step(0.7), 1.0);
        assert_eq!(nice_step(3.0), 5.0);
        assert_eq!(nice_step(5.0), 5.0);
        assert_eq!(nice_step(6.0), 10.0);
        assert_eq!(nice_step(12.0), 20.0);
        assert_eq!(nice_step(250.0), 500.0);
        // Degenerate input falls back to 1 instead of panicking.
        assert_eq!(nice_step(0.0), 1.0);
        assert_eq!(nice_step(f64::NAN), 1.0);
    }

    #[test]
    fn ruler_ticks_cover_the_range_with_round_values() {
        // A 30 pt minimum resolves to a 50 pt step on the 1/2/5 ladder.
        let ticks = ruler_ticks(0.0, 100.0, 30.0);
        assert_eq!(ticks, vec![0.0, 50.0, 100.0]);
        assert!(ticks.windows(2).all(|w| w[0] < w[1]));
        for t in &ticks {
            assert!((0.0..=100.0).contains(t));
        }
        // Step is never finer than requested.
        let step = ticks[1] - ticks[0];
        assert!(step >= 30.0);
    }

    #[test]
    fn ruler_ticks_start_inside_the_range_even_when_offset() {
        let ticks = ruler_ticks(11.0, 40.0, 10.0);
        assert_eq!(ticks, vec![20.0, 30.0, 40.0]);
    }

    #[test]
    fn ruler_ticks_never_return_for_a_degenerate_range() {
        assert!(ruler_ticks(10.0, 10.0, 5.0).is_empty());
        assert!(ruler_ticks(10.0, 5.0, 5.0).is_empty());
        assert!(ruler_ticks(0.0, 8.0, 0.0).is_empty());
        assert!(ruler_ticks(0.0, 8.0, f64::INFINITY).is_empty());
    }

    #[test]
    fn ruler_ticks_scale_with_the_zoom() {
        let near = ruler_ticks(0.0, 600.0, 80.0);
        let far = ruler_ticks(0.0, 600.0, 20.0);
        assert!(far.len() > near.len());
    }
}
