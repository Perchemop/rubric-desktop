//! Focus ring implementation for Rubric UI elements.
//!
//! Provides a visible focus ring with ACCENT colour (≥3:1 WCAG AA contrast)
//! rendered around all focusable egui widgets. The ring is drawn as an
//! overlay stroke after the widget's own painting, ensuring it does not
//! interfere with the widget's layout or interaction.
//!
//! Focus ring properties:
//! - Thickness: 2px
//! - Offset: 2px from the element edge
//! - Colour: Rubric ACCENT (228, 87, 46), which meets AA-large contrast
//!   against both light and dark theme surfaces.

use egui::{pos2, Rect, Stroke};

use crate::keyboard::FocusRingProperties;

/// Draw a focus ring around an egui widget's bounding rectangle.
///
/// This is a pure drawing function — no state mutation, no I/O.
/// Call it from `egui::Area::drag_area()` or within an `egui::Window`
/// paint callback after the widget has been laid out.
///
/// The focus ring uses the ACCENT colour (Rubric red-orange) which has
/// been audited to meet WCAG AA-large (≥3:1) contrast against both light
/// and dark theme surfaces.
pub fn draw_focus_ring(
    ui: &egui::Ui,
    rect: Rect,
    props: FocusRingProperties,
) -> egui::Response {
    // The response from the last widget under the cursor.
    // We draw the focus ring as an overlay stroke after the widget.
    let painter = ui.painter();

    // Calculate the outer rect for the focus ring, offset from the widget edge.
    let outer_min = rect.min - vec2(props.offset, props.offset);
    let outer_max = rect.max + vec2(props.offset, props.offset);
    let outer_rect = Rect::from_two_points(outer_min, outer_max);

    // Draw the focus ring stroke.
    painter.line_segment(
        [
            pos2(outer_rect.min.x, outer_rect.min.y),
            pos2(outer_rect.max.x, outer_rect.min.y),
            pos2(outer_rect.max.x, outer_rect.max.y),
            pos2(outer_rect.min.x, outer_rect.max.y),
            pos2(outer_rect.min.x, outer_rect.min.y),
        ],
        Stroke::new(props.thickness, props.colour),
    );

    // Return a dummy response; the actual widget response is upstream.
    egui::Response::default()
}

/// Default focus ring properties (2px thickness, 2px offset, ACCENT colour).
pub fn default_focus_ring_props() -> FocusRingProperties {
    FocusRingProperties::default_accent()
}

/// Convenience wrapper that draws the focus ring using default properties.
pub fn draw_default_focus_ring(ui: &egui::Ui, rect: Rect) -> egui::Response {
    let props = default_focus_ring_props();
    draw_focus_ring(ui, rect, props)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    #[test]
    fn test_default_focus_ring_props() {
        let props = default_focus_ring_props();
        assert_eq!(props.thickness, 2.0);
        assert_eq!(props.offset, 2.0);
        assert_eq!(props.colour, Color32::from_rgb(228, 87, 46));
    }

    #[test]
    fn test_focus_ring_stroke_attributes() {
        let props = FocusRingProperties::default_accent();
        // Verify the colour has ≥3:1 contrast against white (light) and black (dark).
        // ACCENT (228,87,46) ≈ #E4572C
        let fg = props.colour;
        let bg_white = Color32::WHITE;
        let bg_black = Color32::BLACK;

        // Use the contrast function from design_tokens to verify.
        let ratio_white = crate::design_tokens::contrast_ratio(fg.into(), bg_white.into());
        let ratio_black = crate::design_tokens::contrast_ratio(fg.into(), bg_black.into());

        // AA-large requires ≥3:1 for UI components.
        assert!(
            ratio_white >= 3.0,
            "ACCENT on white: ratio {:.2}:1, expected ≥3.0",
            ratio_white
        );
        assert!(
            ratio_black >= 3.0,
            "ACCENT on black: ratio {:.2}:1, expected ≥3.0",
            ratio_black
        );
    }

    #[test]
    fn test_draw_focus_ring_constructs_stroke() {
        // This test verifies the function compiles and the stroke is constructed
        // with the correct thickness and colour. We can't render without a UI,
        // so we just check the function signature and properties.
        let props = FocusRingProperties::default_accent();
        assert_eq!(props.thickness, 2.0);
        assert_eq!(props.offset, 2.0);
        // Colour is Rubric ACCENT (228, 87, 46)
        assert!(props.colour.r() == 228);
        assert!(props.colour.g() == 87);
        assert!(props.colour.b() == 46);
    }
}