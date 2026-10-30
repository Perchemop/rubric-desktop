//! View-related constants for the UI.

/// Minimum canvas zoom factor.
pub const MIN_ZOOM: f32 = 0.1;

/// Maximum canvas zoom factor.
pub const MAX_ZOOM: f32 = 16.0;

/// Common zoom presets expressed as percentages.
pub const ZOOM_PRESETS: [u32; 3] = [50, 100, 200];

/// Clamp a zoom factor to the allowed range.
#[inline]
pub fn clamp_zoom(zoom: f32) -> f32 {
    zoom.clamp(MIN_ZOOM, MAX_ZOOM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_bounds_are_finite_positive_and_ordered() {
        assert!(MIN_ZOOM.is_finite());
        assert!(MAX_ZOOM.is_finite());
        assert!(MIN_ZOOM > 0.0);
        assert!(MAX_ZOOM > 0.0);
        assert!(MIN_ZOOM < MAX_ZOOM);
    }

    #[test]
    fn clamp_zoom_keeps_values_inside_the_range_unchanged() {
        for zoom in [1.0, 0.5, 15.9] {
            assert_eq!(clamp_zoom(zoom), zoom);
        }
    }

    #[test]
    fn clamp_zoom_clamps_out_of_range_values_to_the_bounds() {
        assert_eq!(clamp_zoom(0.0), MIN_ZOOM);
        assert_eq!(clamp_zoom(-1.0), MIN_ZOOM);
        assert_eq!(clamp_zoom(100.0), MAX_ZOOM);
        assert_eq!(clamp_zoom(f32::INFINITY), MAX_ZOOM);
    }

    #[test]
    fn zoom_presets_are_canonical_and_inside_the_zoom_range() {
        assert_eq!(ZOOM_PRESETS, [50, 100, 200]);
        for preset in ZOOM_PRESETS {
            let zoom = preset as f32 / 100.0;
            assert!(
                (MIN_ZOOM..=MAX_ZOOM).contains(&zoom),
                "preset {preset}% maps to {zoom}, outside [{MIN_ZOOM}, {MAX_ZOOM}]"
            );
        }
    }

    #[test]
    fn clamp_zoom_is_idempotent() {
        for zoom in [0.0, MIN_ZOOM, 0.25, 1.0, 15.9, MAX_ZOOM, 100.0] {
            let clamped = clamp_zoom(zoom);
            assert_eq!(clamp_zoom(clamped), clamped);
        }
    }
}
