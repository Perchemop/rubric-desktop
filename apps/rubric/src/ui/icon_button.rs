//! Original icon buttons drawn with egui's painter — no external font or
//! image dependencies. Icons are simple, geometric, and consistent with
//! Rubric's brand (spec F-603 context menu, toolbar tool buttons).

use eframe::egui::{
    Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};

/// Draw a small geometric icon to the left of a text label inside `ui`.
///
/// `icon` is a closure that paints into a square region. `label` is shown to
/// the right. Returns the button [`Response`] so callers can chain
/// `on_hover_text` or check `.clicked()`.
pub(crate) fn icon_button(
    ui: &mut Ui,
    icon: impl FnOnce(&Painter, Rect),
    label: &str,
    active: bool,
) -> eframe::egui::Response {
    let icon_size = 16.0;
    let gap = 4.0;
    let text = ui.label(label);
    let total_width = icon_size + gap + text.rect.width();
    let mut rect = ui.available_rect_before_wrap();
    rect = rect.shrink(2.0);
    rect.set_width(total_width.min(rect.width()));

    let response = ui
        .allocate_rect(rect, Sense::click())
        // Expose the visible label to Assistive Technology (AccessKit /
        // kittest): the click target is named after the text painted inside
        // it, so icon-paired buttons are screen-reader operable.
        .labelled_by(text.id);
    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            4.0,
            Color32::from_rgb(228, 231, 240).gamma_multiply(0.5),
        );
    }
    if active {
        ui.painter().rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(228, 87, 46)),
            StrokeKind::Middle,
        );
    }

    // Icon square on the left.
    let icon_rect = Rect::from_min_size(rect.min, Vec2::splat(icon_size));
    icon(ui.painter(), icon_rect);

    // Label to the right of the icon.
    let text_rect = Rect::from_min_size(
        rect.min + Vec2::new(icon_size + gap, 0.0),
        Vec2::new(text.rect.width(), icon_size),
    );
    ui.painter().text(
        text_rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::new(13.0, FontFamily::Proportional),
        Color32::BLACK,
    );

    response
}

/// Icon: select / arrow cursor.
pub(crate) fn draw_select_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    // Arrow shaft pointing up-right.
    painter.line_segment([rect.left_center(), rect.right_top()], stroke);
    // Arrow head.
    let head = rect.right_top();
    painter.line_segment([head, head + Vec2::new(-6.0, -2.0)], stroke);
    painter.line_segment([head, head + Vec2::new(-6.0, 2.0)], stroke);
}

/// Icon: highlight marker.
pub(crate) fn draw_highlight_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(2.0, Color32::from_rgb(228, 87, 46));
    painter.line_segment([rect.left_bottom(), rect.right_top()], stroke);
    // Cap at each end.
    painter.circle_filled(rect.left_bottom(), 2.0, Color32::from_rgb(228, 87, 46));
    painter.circle_filled(rect.right_top(), 2.0, Color32::from_rgb(228, 87, 46));
}

/// Icon: rectangle outline.
pub(crate) fn draw_rectangle_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    painter.rect_stroke(rect, 2.0, stroke, StrokeKind::Middle);
}

/// Icon: ellipse outline (approximated with rect).
pub(crate) fn draw_ellipse_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    painter.rect_stroke(rect, rect.height() / 2.0, stroke, StrokeKind::Middle);
}

/// Icon: diagonal line.
pub(crate) fn draw_line_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    painter.line_segment([rect.left_top(), rect.right_bottom()], stroke);
}

/// Icon: arrow (line with head).
pub(crate) fn draw_arrow_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    painter.line_segment([rect.left_center(), rect.right_center()], stroke);
    let head = rect.right_center();
    painter.line_segment([head, head + Vec2::new(-5.0, -3.0)], stroke);
    painter.line_segment([head, head + Vec2::new(-5.0, 3.0)], stroke);
}

/// Icon: text box (T in a square).
pub(crate) fn draw_text_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    painter.rect_stroke(rect, 2.0, stroke, StrokeKind::Middle);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        "T",
        FontId::new(16.0, FontFamily::Proportional),
        Color32::from_rgb(60, 60, 70),
    );
}

/// Icon: ruler / measurement.
pub(crate) fn draw_length_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    // Ruler body.
    painter.line_segment([rect.left_bottom(), rect.right_bottom()], stroke);
    // Tick marks.
    let ticks = 5;
    for i in 0..=ticks {
        let x = rect.left() + (rect.width() * i as f32 / ticks as f32);
        painter.line_segment(
            [
                Pos2::new(x, rect.bottom()),
                Pos2::new(x, rect.bottom() - 6.0),
            ],
            stroke,
        );
    }
    // End caps.
    painter.circle_filled(rect.left_bottom(), 2.0, Color32::from_rgb(60, 60, 70));
    painter.circle_filled(rect.right_bottom(), 2.0, Color32::from_rgb(60, 60, 70));
}
