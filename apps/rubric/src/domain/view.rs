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
