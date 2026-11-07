//! Original icon buttons drawn with egui's painter — no external font or
//! image dependencies. Icons are simple, geometric, and consistent with
//! Rubric's brand (spec F-603 context menu, toolbar tool buttons).

use eframe::egui::{
    Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
    WidgetInfo, WidgetType,
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

    // Measure text width without painting it (avoids double-painting and
    // double space allocation which breaks click targeting in wrapped
    // layouts). Paint the text manually below so icon and text share one
    // clickable rect.
    let text_galley = ui.ctx().fonts_mut(|f| {
        f.layout(
            label.to_owned(),
            FontId::new(13.0, FontFamily::Proportional),
            Color32::BLACK,
            f32::INFINITY,
        )
    });
    let text_width = text_galley.size().x;
    let total_width: f32 = icon_size + gap + text_width;

    let mut rect = ui.available_rect_before_wrap();
    rect = rect.shrink(2.0);
    rect.set_width(total_width.min(rect.width()));

    let response = ui.allocate_rect(rect, Sense::click());
    // Expose the visible label to Assistive Technology (AccessKit /
    // kittest): the click target is named after the text painted inside
    // it, so icon-paired buttons are screen-reader operable. The text is
    // painted manually (there is no `Ui::label` widget to point at), so
    // the button's own AccessKit node carries the name directly — a
    // `labelled_by` relation would reference a node that never exists in
    // the tree, and AccessKit consumers (including kittest) panic while
    // resolving the dangling id.
    response.widget_info(|| {
        egui::WidgetInfo::labeled(WidgetType::Button, response.enabled(), label.to_owned())
    });
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
        Vec2::new(text_width, icon_size),
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

/// Icon: strikeout (horizontal line through text center).
pub(crate) fn draw_strikeout_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(2.0, Color32::from_rgb(228, 87, 46));
    painter.line_segment([rect.left_center(), rect.right_center()], stroke);
    painter.circle_filled(rect.left_center(), 2.0, Color32::from_rgb(228, 87, 46));
    painter.circle_filled(rect.right_center(), 2.0, Color32::from_rgb(228, 87, 46));
}

/// Icon: underline (line at text baseline).
pub(crate) fn draw_underline_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(2.0, Color32::from_rgb(228, 87, 46));
    painter.line_segment([rect.left_top(), rect.right_top()], stroke);
    // small caps to read as a baseline.
    painter.circle_filled(
        Pos2::new(rect.left(), rect.bottom() - 2.0),
        1.5,
        Color32::from_rgb(228, 87, 46),
    );
    painter.circle_filled(
        Pos2::new(rect.right(), rect.bottom() - 2.0),
        1.5,
        Color32::from_rgb(228, 87, 46),
    );
}

/// Icon: squiggly (zig-zag annotation line).
pub(crate) fn draw_squiggly_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(228, 87, 46));
    let steps = 6;
    let step_w = rect.width() / steps as f32;
    let mid = rect.center().y;
    let mut pts: Vec<Pos2> = Vec::new();
    for i in 0..=steps {
        let x = rect.left() + step_w * i as f32;
        let y = if i % 2 == 0 { mid - 3.0 } else { mid + 3.0 };
        pts.push(Pos2::new(x, y));
    }
    painter.add(egui::Shape::line(pts, stroke));
}

/// Icon: callout (text box with leader tail).
pub(crate) fn draw_callout_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    let box_rect =
        Rect::from_min_size(rect.min, Vec2::new(rect.width() * 0.6, rect.height() * 0.6));
    painter.rect_stroke(box_rect, 2.0, stroke, StrokeKind::Middle);
    // leader tail from box corner toward bottom-right.
    let head = box_rect.right_bottom();
    painter.line_segment([head, rect.right_bottom()], stroke);
    painter.line_segment(
        [
            rect.right_bottom(),
            rect.right_bottom() + Vec2::new(-4.0, 0.0),
        ],
        stroke,
    );
}

/// Icon: sticky note (dog-eared corner).
pub(crate) fn draw_note_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    painter.rect_stroke(rect, 2.0, stroke, StrokeKind::Middle);
    // dog-eared top-right corner.
    let head = rect.right_top();
    painter.line_segment([head, head + Vec2::new(-8.0, 0.0)], stroke);
    painter.line_segment([head, head + Vec2::new(0.0, 8.0)], stroke);
    painter.line_segment(
        [head + Vec2::new(-8.0, 0.0), head + Vec2::new(0.0, 8.0)],
        stroke,
    );
}

/// Icon: stamp (square border with review mark).
pub(crate) fn draw_stamp_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(228, 87, 46));
    painter.rect_stroke(rect, 3.0, stroke, StrokeKind::Middle);
    // review check glyph.
    let center = rect.center();
    painter.line_segment(
        [center + Vec2::new(-4.0, 0.0), center + Vec2::new(0.0, 4.0)],
        stroke,
    );
    painter.line_segment(
        [center + Vec2::new(0.0, 4.0), center + Vec2::new(6.0, -4.0)],
        stroke,
    );
}

/// Icon: cloud (bumpy outline).
pub(crate) fn draw_cloud_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    let cx = rect.center().x;
    let cy = rect.center().y;
    let rx = rect.width() * 0.35;
    let ry = rect.height() * 0.35;
    let pts: Vec<Pos2> = (0..=8)
        .map(|i| {
            let angle = (i as f32) * std::f32::consts::PI / 4.0;
            let r = if i % 2 == 0 { 1.0 } else { 0.75 };
            Pos2::new(cx + angle.cos() * rx * r, cy + angle.sin() * ry * r)
        })
        .collect();
    painter.add(egui::Shape::line(pts.clone(), stroke));
    painter.line_segment([pts[8], pts[0]], stroke);
}

/// Icon: polygon (closed multi-point shape).
pub(crate) fn draw_polygon_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    let cx = rect.center().x;
    let cy = rect.center().y;
    let r = rect.width() * 0.3;
    let pts: Vec<Pos2> = (0..=5)
        .map(|i| {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / 5.0 - std::f32::consts::PI / 2.0;
            Pos2::new(cx + angle.cos() * r, cy + angle.sin() * r)
        })
        .collect();
    painter.add(egui::Shape::line(pts.clone(), stroke));
    painter.line_segment([pts[5], pts[0]], stroke);
}

/// Icon: polyline (open multi-point stroke).
pub(crate) fn draw_polyline_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    let cx = rect.center().x;
    let cy = rect.center().y;
    let r = rect.width() * 0.3;
    let pts: Vec<Pos2> = (0..=4)
        .map(|i| {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / 4.0 - std::f32::consts::PI / 2.0;
            Pos2::new(cx + angle.cos() * r, cy + angle.sin() * r)
        })
        .collect();
    painter.add(egui::Shape::line(pts, stroke));
}

/// Icon: ink (freehand squiggly stroke).
pub(crate) fn draw_ink_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(2.0, Color32::from_rgb(60, 60, 70));
    let cy = rect.center().y;
    let pts: Vec<Pos2> = (0..=7)
        .map(|i| {
            let x = rect.left() + rect.width() * i as f32 / 7.0;
            let jitter = if i % 2 == 0 { -4.0 } else { 4.0 };
            Pos2::new(x, cy + jitter)
        })
        .collect();
    painter.add(egui::Shape::line(pts, stroke));
}

/// Icon: measure perimeter (closed loop with dimension ticks).
pub(crate) fn draw_measure_perimeter_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    painter.rect_stroke(rect, 2.0, stroke, StrokeKind::Middle);
    // dimension ticks along bottom.
    let y = rect.bottom();
    for x in [rect.left(), rect.center().x, rect.right()] {
        painter.line_segment([Pos2::new(x, y), Pos2::new(x, y - 6.0)], stroke);
    }
}

/// Icon: measure area (hatched fill).
pub(crate) fn draw_measure_area_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    painter.rect_stroke(rect, 2.0, stroke, StrokeKind::Middle);
    // diagonal hatch lines.
    let steps = 4;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x1 = rect.left() + t * rect.width();
        let x2 = rect.left() + t * rect.width();
        painter.line_segment(
            [Pos2::new(x1, rect.top()), Pos2::new(x2, rect.bottom())],
            stroke,
        );
    }
}

/// Icon: measure angle (arc between two lines).
pub(crate) fn draw_measure_angle_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    let vertex = rect.center();
    // two rays from vertex.
    painter.line_segment([vertex, Pos2::new(rect.left(), rect.bottom())], stroke);
    painter.line_segment([vertex, Pos2::new(rect.right(), rect.bottom())], stroke);
    // angle arc near vertex.
    let arc_points: Vec<Pos2> = (0..=6)
        .map(|i| {
            let angle = (i as f32) * std::f32::consts::PI / 4.0;
            let r = 14.0;
            Pos2::new(vertex.x + angle.cos() * r, vertex.y + angle.sin() * r)
        })
        .collect();
    painter.add(egui::Shape::line(arc_points, stroke));
}

/// Icon: measure diameter (circle with line through center).
pub(crate) fn draw_measure_diameter_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    let center = rect.center();
    let r = rect.width() * 0.3;
    // circle.
    let pts: Vec<Pos2> = (0..=12)
        .map(|i| {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / 12.0;
            Pos2::new(center.x + angle.cos() * r, center.y + angle.sin() * r)
        })
        .collect();
    painter.add(egui::Shape::line(pts.clone(), stroke));
    painter.line_segment([pts[0], pts[6]], stroke);
}

/// Icon: measure radius (circle with radius line from center to edge).
pub(crate) fn draw_measure_radius_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    let center = rect.center();
    let r = rect.width() * 0.3;
    let pts: Vec<Pos2> = (0..=12)
        .map(|i| {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / 12.0;
            Pos2::new(center.x + angle.cos() * r, center.y + angle.sin() * r)
        })
        .collect();
    painter.add(egui::Shape::line(pts, stroke));
    painter.line_segment([center, Pos2::new(center.x + r, center.y)], stroke);
}

/// Icon: measure count (numbered circle).
pub(crate) fn draw_measure_count_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    let center = rect.center();
    let r = rect.width() * 0.3;
    let pts: Vec<Pos2> = (0..=12)
        .map(|i| {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / 12.0;
            Pos2::new(center.x + angle.cos() * r, center.y + angle.sin() * r)
        })
        .collect();
    painter.add(egui::Shape::line(pts, stroke));
    painter.text(
        center,
        Align2::CENTER_CENTER,
        "1",
        FontId::new(12.0, FontFamily::Proportional),
        Color32::from_rgb(60, 60, 70),
    );
}

/// Icon: calibrate (ruler with two reference marks).
pub(crate) fn draw_calibrate_icon(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.5, Color32::from_rgb(60, 60, 70));
    // ruler baseline.
    let y = rect.center().y;
    painter.line_segment([rect.left_center(), rect.right_center()], stroke);
    // two reference ticks.
    for x in [
        rect.left() + rect.width() * 0.25,
        rect.left() + rect.width() * 0.75,
    ] {
        painter.line_segment([Pos2::new(x, y - 6.0), Pos2::new(x, y + 6.0)], stroke);
    }
}
