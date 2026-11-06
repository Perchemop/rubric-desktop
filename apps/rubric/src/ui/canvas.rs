//! Canvas: page rendering, tool interaction, drafting and selection.

use std::path::Path;

use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use uuid::Uuid;

use crate::data::pdf::find_text_line_at_point;
use crate::domain::markup::{
    count_markers_on_page, count_total_label, next_count_number, text_line_height,
};
use crate::domain::{
    angle_degrees, format_value, ruler_ticks, BatesConfig, Corner, Grid, Markup, MarkupGeometry,
    MarkupKind, MeasureKind, Page, Project, Pt, Rect as DRect, ReviewStatus, Scale, Tool,
};

use super::theme::to_color32;

/// Map page point to screen space.
fn to_screen(page: Pt, pan: Vec2, zoom: f32, origin: Pos2) -> Pos2 {
    origin + pan + Vec2::new(page.x * zoom, page.y * zoom)
}

/// Map screen point to page space.
fn to_page(screen: Pos2, pan: Vec2, zoom: f32, origin: Pos2) -> Pt {
    let v = screen - origin - pan;
    let inv = if zoom.abs() < 1e-4 { 1.0 } else { 1.0 / zoom };
    Pt::new(v.x * inv, v.y * inv)
}

/// Pick a markup id at page point (frontmost = last in paint order).
fn pick_markup_id(project: &Project, page_idx: usize, p: Pt, tol: f32) -> Option<Uuid> {
    let doc = project.active()?;
    doc.markups_in_paint_order(page_idx)
        .into_iter()
        .rev()
        .find(|m| m.hit_test(p, tol))
        .map(|m| m.id)
}

fn screen_rect(r: DRect, origin: Pos2, pan: Vec2, zoom: f32) -> egui::Rect {
    let min = to_screen(Pt::new(r.x, r.y), pan, zoom, origin);
    let max = to_screen(Pt::new(r.right(), r.bottom()), pan, zoom, origin);
    egui::Rect::from_min_max(min, max)
}

/// Which selection-handle corner, if any, the screen point `pp` is over.
///
/// The hit area is slightly larger than the drawn handle so grabbing a corner
/// does not require pixel precision (v0.4 gap #5: resize grips).
fn handle_at(b: egui::Rect, pp: Pos2, zoom: f32) -> Option<Corner> {
    let radius = 8.0_f32.max(4.5 * zoom.max(0.7));
    let corners = [
        (Corner::TopLeft, b.left_top()),
        (Corner::TopRight, b.right_top()),
        (Corner::BottomLeft, b.left_bottom()),
        (Corner::BottomRight, b.right_bottom()),
    ];
    for (c, p) in corners {
        if (p - pp).length() <= radius {
            return Some(c);
        }
    }
    None
}

/// Which draggable endpoint/vertex of `m`, if any, the screen point `pp` is
/// over (v0.4 gap #5 follow-up). Checked before the bounding-box corner
/// handles so a coincident endpoint wins.
fn point_handle_at(m: &Markup, pp: Pos2, origin: Pos2, pan: Vec2, zoom: f32) -> Option<usize> {
    let radius = 8.0_f32.max(4.5 * zoom.max(0.7));
    m.draggable_points()
        .iter()
        .position(|p| (to_screen(*p, pan, zoom, origin) - pp).length() <= radius)
}

/// Draw a small dark measurement badge with white text at `anchor` (screen
/// space). Used for measurement labels and the running count total.
fn draw_measure_badge(painter: &Painter, anchor: Pos2, label: String) {
    let galley = painter.layout_no_wrap(label, FontId::proportional(12.0), Color32::WHITE);
    let text_rect = galley.rect.translate(Vec2::new(anchor.x, anchor.y));
    painter.rect_filled(text_rect.expand(4.0), 4.0, Color32::from_black_alpha(180));
    painter.galley(anchor, galley, Color32::WHITE);
}

/// Refuse to begin a draft when the markup it would create could not be edited.
///
/// New markups always land on the default layer (`layer_id = None`), so a
/// locked default layer would commit instantly uneditable markups. Fails open
/// when no document is open or the document has no layers at all.
fn creation_allowed(project: &Project, ui_state: &mut super::app::UiState) -> bool {
    let locked = project
        .active()
        .and_then(|d| d.default_layer())
        .is_some_and(|l| l.locked);
    if locked {
        ui_state.status = "Layer locked — new markups cannot be created".to_string();
    }
    locked
}

/// Whether a drafted path is worth committing: it must have enough points
/// (`min`, which encodes whether the path is closed) and at least one edge
/// long enough to be meaningful — coincident points, which double-clicks and
/// repeated clicks produce, do not count.
fn path_points_ok(points: &[Pt], min: usize, _closed: bool) -> bool {
    points.len() >= min && points.windows(2).any(|w| w[0].distance(w[1]) > 0.5)
}

/// Hit tolerance expressed in page units for a given zoom, so a fixed screen
/// pixel budget (~6 px) is used at every zoom level.
fn hit_tolerance_page(zoom: f32) -> f32 {
    let z = zoom.max(0.1);
    (6.0 / z).clamp(0.5, 30.0)
}

/// Whether an ink stroke is worth committing: it must contain at least two
/// distinct points and a small amount of total length.
fn ink_stroke_ok(strokes: &[Vec<Pt>]) -> bool {
    let point_count: usize = strokes.iter().map(|s| s.len()).sum();
    let total_len: f32 = strokes
        .iter()
        .flat_map(|s| s.windows(2))
        .map(|w| w[0].distance(w[1]))
        .sum();
    point_count >= 2 && (point_count >= 3 || total_len >= 1.0)
}

/// Apply the active stamp preset to a freshly created stamp markup. Non-stamp
/// markups pass through untouched.
fn finish_markup(m: Markup, ui_state: &super::app::UiState) -> Markup {
    if m.kind == MarkupKind::Stamp {
        let preset = ui_state.active_stamp;
        let mut m = m.with_stamp(preset);
        m.style = m.style.with_stroke(preset.color());
        m
    } else if let Some(preset) = &ui_state.pending_style_preset {
        // Tool Chest apply-at-creation (v0.4 gap #7): style only. Author /
        // subject already come from project settings / the kind label; there is
        // no review verdict to overwrite at creation time.
        m.with_style(preset.style)
    } else {
        m
    }
}

/// Shortest reference segment (page points) accepted for drag calibration
/// (v0.4 gap #4). A shorter gesture carries no meaningful ratio and is
/// cancelled instead of opening the calibration dialog.
const MIN_CALIBRATION_POINTS: f32 = 1.0;

/// Whether a measured reference segment is long enough to calibrate from.
fn calibration_segment_ok(points: f32) -> bool {
    points.is_finite() && points >= MIN_CALIBRATION_POINTS
}

/// Anchor point in page space for a measurement label.
fn measure_anchor(m: &Markup) -> Pt {
    match &m.geometry {
        MarkupGeometry::Segment { a, b } => Pt::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5),
        MarkupGeometry::Path { points, .. } => {
            if points.is_empty() {
                m.bounds().center()
            } else {
                let n = points.len() as f32;
                let (sx, sy) = points
                    .iter()
                    .fold((0.0f32, 0.0f32), |(x, y), p| (x + p.x, y + p.y));
                Pt::new(sx / n, sy / n)
            }
        }
        MarkupGeometry::Angle { vertex, .. } => *vertex,
        MarkupGeometry::Count { at, .. } => *at,
        _ => m.bounds().center(),
    }
}

/// Canvas grid (spec F-405). Guides only: drawn under the markups and skipped
/// entirely when the pitch would be too fine to resolve, so a far zoom never
/// produces an unreadable wash.
fn draw_grid(painter: &Painter, bounds: DRect, grid: Grid, pan: Vec2, zoom: f32, origin: Pos2) {
    if !grid.should_draw(zoom) {
        return;
    }
    let step = grid.spacing as f64;
    let stroke = egui::Stroke::new(1.0, Color32::from_rgb(203, 209, 221));
    let cols = (bounds.right() as f64 / step).ceil() as i32;
    for k in 0..=cols {
        let x = (k as f64 * step) as f32;
        let a = to_screen(Pt::new(x, bounds.y), pan, zoom, origin);
        let b = to_screen(Pt::new(x, bounds.bottom()), pan, zoom, origin);
        painter.line_segment([a, b], stroke);
    }
    let rows = (bounds.bottom() as f64 / step).ceil() as i32;
    for k in 0..=rows {
        let y = (k as f64 * step) as f32;
        let a = to_screen(Pt::new(bounds.x, y), pan, zoom, origin);
        let b = to_screen(Pt::new(bounds.right(), y), pan, zoom, origin);
        painter.line_segment([a, b], stroke);
    }
}

/// Rulers (spec F-406): labelled ticks in the margin just outside the page, so a
/// canvas position can be read in page points. Tick spacing follows the 1/2/5
/// ladder in `domain::grid`, chosen from the zoom so labels never collide.
#[allow(clippy::too_many_arguments)]
fn draw_rulers(
    painter: &Painter,
    ui: &Ui,
    page_rect: egui::Rect,
    bounds: DRect,
    pan: Vec2,
    zoom: f32,
    origin: Pos2,
    scale: &Scale,
) {
    const RULER_H: f32 = 18.0;
    const RULER_W: f32 = 30.0;
    let text_c = ui.visuals().text_color();
    let tick_c = Color32::from_rgba_unmultiplied(
        text_c.r(),
        text_c.g(),
        text_c.b(),
        text_c.a().saturating_mul(6) / 10,
    );
    let tick = egui::Stroke::new(1.0, tick_c);
    let font = FontId::proportional(9.0);
    // Aim for labelled ticks at least ~72 px apart.
    let min_step = 72.0 / (zoom as f64).max(1e-3);

    // Top ruler: page x.
    let base_y = page_rect.min.y;
    let top = base_y - RULER_H;
    if top > -RULER_H {
        painter.line_segment(
            [
                Pos2::new(page_rect.min.x, base_y - 1.0),
                Pos2::new(page_rect.max.x, base_y - 1.0),
            ],
            tick,
        );
        for v in ruler_ticks(bounds.x as f64, bounds.right() as f64, min_step) {
            let x = to_screen(Pt::new(v as f32, bounds.y), pan, zoom, origin).x;
            painter.line_segment(
                [Pos2::new(x, base_y - 5.0), Pos2::new(x, base_y - 1.0)],
                tick,
            );
            let units = scale.points_to_units(v);
            painter.text(
                Pos2::new(x + 2.0, top + 2.0),
                Align2::LEFT_TOP,
                format!("{v:.0}"),
                font.clone(),
                text_c,
            );
            painter.text(
                Pos2::new(x + 2.0, top + 12.0),
                Align2::LEFT_TOP,
                format!("{units:.2}{}", scale.unit.suffix()),
                font.clone(),
                text_c.gamma_multiply(0.6),
            );
        }
    }

    // Left ruler: page y.
    let base_x = page_rect.min.x;
    let left = base_x - RULER_W;
    if left > -RULER_W {
        painter.line_segment(
            [
                Pos2::new(base_x - 1.0, page_rect.min.y),
                Pos2::new(base_x - 1.0, page_rect.max.y),
            ],
            tick,
        );
        for v in ruler_ticks(bounds.y as f64, bounds.bottom() as f64, min_step) {
            let y = to_screen(Pt::new(bounds.x, v as f32), pan, zoom, origin).y;
            painter.line_segment(
                [Pos2::new(base_x - 5.0, y), Pos2::new(base_x - 1.0, y)],
                tick,
            );
            let units = scale.points_to_units(v);
            painter.text(
                Pos2::new(left + 2.0, y + 2.0),
                Align2::LEFT_TOP,
                format!("{v:.0}"),
                font.clone(),
                text_c,
            );
            painter.text(
                Pos2::new(left + 2.0, y + 12.0),
                Align2::LEFT_TOP,
                format!("{units:.2}{}", scale.unit.suffix()),
                font.clone(),
                text_c.gamma_multiply(0.6),
            );
        }
    }
}

/// Draw a single markup.
fn draw_markup(
    painter: &Painter,
    origin: Pos2,
    pan: Vec2,
    zoom: f32,
    m: &Markup,
    show_measurements: bool,
) {
    let style = m.style;
    let stroke_c = to_color32(style.stroke);
    let fill_c = to_color32(style.fill.with_alpha(style.fill_mode.alpha(style.fill.a)));
    let stroke_w = (style.width * zoom).max(0.25);
    let stroke_eg = egui::Stroke::new(stroke_w, stroke_c);

    match &m.geometry {
        MarkupGeometry::Bounds(r) => {
            let sr = screen_rect(*r, origin, pan, zoom);
            match m.kind {
                MarkupKind::Rectangle
                | MarkupKind::Stamp
                | MarkupKind::Note
                | MarkupKind::Callout
                | MarkupKind::Text => {
                    if fill_c.a() > 0 {
                        painter.rect_filled(sr, egui::CornerRadius::same(6), fill_c);
                    }
                    painter.rect_stroke(
                        sr,
                        egui::CornerRadius::same(6),
                        stroke_eg,
                        egui::StrokeKind::Middle,
                    );
                }
                MarkupKind::Ellipse => {
                    let radius = Vec2::new(sr.width() * 0.5, sr.height() * 0.5);
                    if fill_c.a() > 0 {
                        painter.add(egui::Shape::ellipse_filled(sr.center(), radius, fill_c));
                    }
                    painter.add(egui::Shape::ellipse_stroke(sr.center(), radius, stroke_eg));
                }
                MarkupKind::Cloud => {
                    // A simple cloud: a few overlapping oval bumps around the bounds.
                    let cx = sr.center().x;
                    let cy = sr.center().y;
                    let rx = sr.width() * 0.30;
                    let ry = sr.height() * 0.30;
                    let bumps = [
                        (Pos2::new(cx - rx * 0.7, cy - ry * 0.3), Vec2::new(rx, ry)),
                        (Pos2::new(cx + rx * 0.7, cy - ry * 0.3), Vec2::new(rx, ry)),
                        (Pos2::new(cx, cy - ry * 0.9), Vec2::new(rx * 1.1, ry)),
                        (Pos2::new(cx, cy + ry * 0.7), Vec2::new(rx * 1.2, ry)),
                    ];
                    for (center, radius) in bumps {
                        painter.add(egui::Shape::ellipse_stroke(center, radius, stroke_eg));
                    }
                }
                _ => {
                    if fill_c.a() > 0 {
                        painter.rect_filled(sr, egui::CornerRadius::ZERO, fill_c);
                    }
                    painter.rect_stroke(
                        sr,
                        egui::CornerRadius::ZERO,
                        stroke_eg,
                        egui::StrokeKind::Middle,
                    );
                }
            }
        }
        MarkupGeometry::TextAnchor {
            at,
            text,
            line_height,
            text_width,
            ..
        } => {
            // Text-anchored markup: draw line(s) at text line positions,
            // anchored to text rather than a bounding box. `at` is the
            // top-left of the text line in page space (y-down); every value
            // is scaled into screen space before it reaches the painter.
            let sa = to_screen(*at, pan, zoom, origin);
            let w = text_width.max(text.chars().count() as f32 * line_height * 0.6) * zoom;
            let h = *line_height * zoom;
            match m.kind {
                MarkupKind::Highlight => {
                    let fill = style.fill;
                    if fill.a > 0 {
                        // Draw fill highlight over text line area
                        let hl_rect = egui::Rect::from_min_size(sa, Vec2::new(w, h));
                        painter.rect_filled(hl_rect, egui::CornerRadius::ZERO, fill_c);
                    }
                    painter.rect_stroke(
                        egui::Rect::from_min_size(sa, Vec2::new(w, h)),
                        egui::CornerRadius::ZERO,
                        stroke_eg,
                        egui::StrokeKind::Inside,
                    );
                }
                MarkupKind::Strikeout => {
                    // Line through middle of text line
                    let y = sa.y + h * 0.5;
                    painter.line_segment([Pos2::new(sa.x, y), Pos2::new(sa.x + w, y)], stroke_eg);
                }
                MarkupKind::Underline => {
                    // Line at bottom of text line
                    let y = sa.y + h - stroke_w * 0.5;
                    painter.line_segment([Pos2::new(sa.x, y), Pos2::new(sa.x + w, y)], stroke_eg);
                }
                MarkupKind::Squiggly => {
                    // Zig-zag near the bottom of the text line
                    let y = sa.y + h - stroke_w * 0.5;
                    let step = (w / 16.0).clamp(3.0, 12.0);
                    let mut pts: Vec<Pos2> = Vec::new();
                    let mut x = sa.x;
                    let mut up = true;
                    while x < sa.x + w {
                        let dy = if up { -step * 0.5 } else { step * 0.5 };
                        pts.push(Pos2::new(x, y + dy));
                        x += step;
                        up = !up;
                    }
                    pts.push(Pos2::new(sa.x + w, y));
                    if pts.len() >= 2 {
                        painter.add(egui::Shape::line(pts, stroke_eg));
                    }
                }
                _ => {}
            }
        }
        MarkupGeometry::Segment { a, b } => {
            let sa = to_screen(*a, pan, zoom, origin);
            let sb = to_screen(*b, pan, zoom, origin);
            painter.line_segment([sa, sb], stroke_eg);
            if matches!(m.kind, MarkupKind::Arrow) {
                let dir = (sb - sa).normalized();
                let perp = Vec2::new(-dir.y, dir.x);
                let base = sb - dir * 12.0 * zoom;
                let left = base + perp * 6.0 * zoom;
                let right = base - perp * 6.0 * zoom;
                painter.line_segment([sb, left], stroke_eg);
                painter.line_segment([sb, right], stroke_eg);
            }
        }
        MarkupGeometry::Path { points, closed } => {
            let pts: Vec<Pos2> = points
                .iter()
                .map(|p| to_screen(*p, pan, zoom, origin))
                .collect();
            if pts.len() >= 2 {
                if *closed {
                    painter.add(egui::Shape::closed_line(pts, stroke_eg));
                } else {
                    painter.add(egui::Shape::line(pts, stroke_eg));
                }
            }
        }
        MarkupGeometry::Ink { strokes } => {
            for s in strokes {
                let pts: Vec<Pos2> = s.iter().map(|p| to_screen(*p, pan, zoom, origin)).collect();
                if pts.len() >= 2 {
                    painter.add(egui::Shape::line(pts, stroke_eg));
                }
            }
        }
        MarkupGeometry::Text { at, wrap, text } => {
            let sa = to_screen(*at, pan, zoom, origin);
            if text.is_empty() {
                // Empty text paints no glyphs; draw the bounds as a faint
                // outline so the markup stays visible where it hit-tests
                // (e.g. a Text the user placed and immediately cancelled).
                let sr = screen_rect(m.bounds(), origin, pan, zoom);
                painter.rect_stroke(
                    sr,
                    egui::CornerRadius::same(4),
                    egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(30, 144, 255, 89)),
                    egui::StrokeKind::Middle,
                );
            } else {
                // Wrap at `wrap` points (scaled to screen) to match the
                // in-canvas editor; the `bounds()` line count is only an
                // approximation of the wrapped height.
                let font = FontId::proportional(11.0 * zoom.max(0.8));
                let galley =
                    painter.layout(text.as_str().to_string(), font, stroke_c, *wrap * zoom);
                painter.galley(sa, galley, stroke_c);
            }
        }
        MarkupGeometry::Callout { rect, tail, text } => {
            let sr = screen_rect(*rect, origin, pan, zoom);
            let sta = to_screen(*tail, pan, zoom, origin);
            painter.line_segment([sta, sr.center()], stroke_eg);
            if fill_c.a() > 0 {
                painter.rect_filled(sr, egui::CornerRadius::same(8), fill_c);
            }
            painter.rect_stroke(
                sr,
                egui::CornerRadius::same(8),
                stroke_eg,
                egui::StrokeKind::Middle,
            );
            painter.text(
                sr.min,
                Align2::LEFT_TOP,
                text,
                FontId::proportional(10.0 * zoom.max(0.7)),
                stroke_c,
            );
        }
        MarkupGeometry::Angle { a, vertex, b } => {
            let sa = to_screen(*a, pan, zoom, origin);
            let sv = to_screen(*vertex, pan, zoom, origin);
            let sb = to_screen(*b, pan, zoom, origin);
            painter.line_segment([sv, sa], stroke_eg);
            painter.line_segment([sv, sb], stroke_eg);
            // The angle value is the markup's measurement label, so it obeys
            // the same "show measurements" preference as the overlay badges.
            if show_measurements {
                let ang = format!("{:.0}°", angle_degrees(*a, *vertex, *b));
                painter.text(
                    sv + Vec2::new(4.0, 4.0),
                    Align2::LEFT_TOP,
                    ang,
                    FontId::proportional(9.0 * zoom.max(0.7)),
                    stroke_c,
                );
            }
        }
        MarkupGeometry::Count { at, number } => {
            let sa = to_screen(*at, pan, zoom, origin);
            painter.circle_filled(sa, 10.0 * zoom.max(0.7), Color32::from_rgb(30, 30, 36));
            // The dot stays visible; its number is the count measurement, so
            // it is hidden with the other measurement labels.
            if show_measurements {
                painter.text(
                    sa,
                    Align2::CENTER_CENTER,
                    number.to_string(),
                    FontId::proportional(10.0 * zoom.max(0.7)),
                    Color32::WHITE,
                );
            }
        }
    }

    // Stamp preset glyph (spec F-111): centred, colour-matched, shrink-to-fit.
    if m.kind == MarkupKind::Stamp {
        if let Some(preset) = m.stamp {
            let sr = screen_rect(m.bounds(), origin, pan, zoom);
            if sr.width() >= 12.0 && sr.height() >= 12.0 {
                let color = to_color32(preset.color());
                let glyph = preset.glyph();
                let mut font_size = (sr.height() * 0.4).clamp(8.0, 64.0);
                for _ in 0..8 {
                    let fid = FontId::proportional(font_size);
                    let g = painter.layout_no_wrap(glyph.to_string(), fid, color);
                    if g.size().x <= sr.width() * 0.9 || font_size <= 6.0 {
                        break;
                    }
                    font_size *= 0.85;
                }
                let fid = FontId::proportional(font_size);
                let galley = painter.layout_no_wrap(glyph.to_string(), fid, color);
                let pos = sr.center() - galley.size() * 0.5;
                painter.galley(pos, galley, color);
            }
        }
    }

    if m.status != ReviewStatus::Open {
        let b = screen_rect(m.bounds(), origin, pan, zoom);
        let dot = Pos2::new(b.right() + 4.0, b.top() - 4.0);
        painter.circle_filled(dot, 5.0 * zoom.max(0.7), to_color32(m.status.color()));
    }
}

/// Draw the Bates stamp for one page as a non-destructive overlay
/// (spec F-804/F-805).
///
/// The stamp is ghosted in the live editor so it reads as a preview, and solid
/// in export/print (both paths reuse `draw_canvas`). Nothing is created in the
/// project and the source PDF is never modified: this is purely a painted
/// overlay derived from [`BatesConfig`].
fn draw_bates_overlay(
    painter: &Painter,
    origin: Pos2,
    pan: Vec2,
    zoom: f32,
    page: &Page,
    bates: &BatesConfig,
    is_export: bool,
) {
    if !bates.enabled {
        return;
    }
    let Some(stamp) = bates.stamp_for_page(page.index) else {
        return;
    };
    let align = match (bates.corner.is_right(), bates.corner.is_top()) {
        (false, true) => Align2::LEFT_TOP,
        (true, true) => Align2::RIGHT_TOP,
        (false, false) => Align2::LEFT_BOTTOM,
        (true, false) => Align2::RIGHT_BOTTOM,
    };
    let anchor = to_screen(
        bates.anchor_point(page.width, page.height),
        pan,
        zoom,
        origin,
    );
    let font = FontId::proportional((bates.font_size.max(4.0) * zoom).max(4.0));
    let color = if is_export {
        Color32::from_rgb(20, 20, 20)
    } else {
        Color32::from_rgba_unmultiplied(20, 20, 20, 110)
    };
    painter.text(anchor, align, stamp, font, color);
}

/// Draw a minimap in the bottom-right corner of the canvas showing the full
/// page with a viewport rectangle indicating the current view area.
/// Clicking the minimap pans the canvas to the clicked position.
fn draw_minimap(
    painter: &Painter,
    page_bounds: DRect,
    viewport_rect: egui::Rect,
    zoom: f32,
    canvas_size: Vec2,
    origin: Pos2,
    pan: Vec2,
) {
    // Minimap size: fixed pixel size, scaled down to fit
    const MINIMAP_SIZE: f32 = 140.0;
    let aspect = page_bounds.w / page_bounds.h;
    let minimap_w = MINIMAP_SIZE;
    let minimap_h = MINIMAP_SIZE / aspect.clamp(0.1, 10.0);

    // Position: bottom-right corner with padding
    let margin = 10.0;
    let anchor_x = canvas_size.x - margin - minimap_w;
    let anchor_y = canvas_size.y - margin - minimap_h;
    let anchor = Pos2::new(anchor_x, anchor_y);

    // Draw minimap background (page bounds)
    let minimap_rect = egui::Rect::from_min_size(anchor, Vec2::new(minimap_w, minimap_h));
    painter.rect_filled(
        minimap_rect,
        4.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 120),
    );
    painter.rect_stroke(
        minimap_rect,
        4.0,
        Stroke::new(1.0, Color32::WHITE),
        StrokeKind::Middle,
    );

    // Draw page content area scaled into minimap
    let page_w = page_bounds.w;
    let page_h = page_bounds.h;
    let scale = minimap_w / page_w.max(1.0);
    let page_height_scaled = page_h * scale;
    let page_rect_minimap =
        egui::Rect::from_min_size(anchor, Vec2::new(minimap_w, page_height_scaled))
            .translate(Vec2::new(0.0, (minimap_h - page_height_scaled) / 2.0));

    // Draw page bounds rect
    painter.rect_stroke(
        page_rect_minimap,
        0.0,
        Stroke::new(0.5, Color32::from_rgba_unmultiplied(200, 200, 200, 100)),
        StrokeKind::Middle,
    );

    // Draw viewport rectangle
    // Map viewport_rect (in screen space) back to page space, then to minimap space
    let viewport_page_x1 = (viewport_rect.min.x - origin.x - pan.x) / zoom;
    let viewport_page_y1 = (viewport_rect.min.y - origin.y - pan.y) / zoom;
    let viewport_page_x2 = (viewport_rect.max.x - origin.x - pan.x) / zoom;
    let viewport_page_y2 = (viewport_rect.max.y - origin.y - pan.y) / zoom;

    let viewport_minimap = egui::Rect::from_two_pos(
        Pos2::new(
            anchor.x + viewport_page_x1 * scale,
            anchor.y + page_rect_minimap.top() + viewport_page_y1 * scale,
        ),
        Pos2::new(
            anchor.x + viewport_page_x2 * scale,
            anchor.y + page_rect_minimap.top() + viewport_page_y2 * scale,
        ),
    );
    painter.rect_stroke(
        viewport_minimap,
        1.0,
        Stroke::new(1.0, Color32::from_rgb(228, 87, 46)),
        StrokeKind::Middle,
    );
    painter.rect_filled(
        viewport_minimap,
        0.0,
        Color32::from_rgba_unmultiplied(228, 87, 46, 60),
    );

    // Draw minimap border label
    painter.text(
        anchor + Vec2::new(4.0, -8.0),
        Align2::LEFT_TOP,
        "Overview",
        FontId::proportional(10.0),
        Color32::WHITE,
    );
}

/// Draw selection handles for a selected markup.
///
/// Geometries with draggable points (segments, paths, angles) show one handle
/// per endpoint/vertex — dragging it reshapes the markup directly (v0.4 gap #5
/// follow-up). Everything else keeps the bounding-box corner handles plus a
/// centre grab dot; only `Bounds` geometry actually resizes from a corner.
fn draw_selection(painter: &Painter, origin: Pos2, pan: Vec2, zoom: f32, m: &Markup) {
    let b = screen_rect(m.bounds(), origin, pan, zoom);
    let stroke = egui::Stroke::new(1.0, Color32::from_rgb(30, 144, 255));
    let points = m.draggable_points();
    if points.is_empty() {
        let handle = |p: Pos2| painter.circle(p, 4.5 * zoom.max(0.7), Color32::WHITE, stroke);
        handle(b.left_top());
        handle(b.right_top());
        handle(b.left_bottom());
        handle(b.right_bottom());
        handle(b.center());
    } else {
        for p in points {
            let sp = to_screen(p, pan, zoom, origin);
            painter.circle(sp, 4.0 * zoom.max(0.7), Color32::WHITE, stroke);
        }
    }
    painter.rect_stroke(
        b,
        egui::CornerRadius::same(4),
        stroke,
        egui::StrokeKind::Middle,
    );
}

/// Commit a pending in-canvas text edit (if any) using focus-loss semantics.
/// Returns `true` when the text actually changed and was written back.
pub(crate) fn commit_text_edit(ui_state: &mut super::app::UiState, project: &mut Project) -> bool {
    let Some(edit) = ui_state.text_edit.clone() else {
        return false;
    };
    let id = edit.id;
    let new_text = edit.text.clone();
    let original = edit.original.clone();
    let editable = project
        .active()
        .and_then(|d| d.find_markup(id).map(|m| d.is_markup_editable(m)))
        .unwrap_or(false);
    let changed = new_text != original;
    if changed && editable {
        if let Some(doc) = project.active_mut() {
            if let Some(m) = doc.find_markup_mut(id) {
                match &mut m.geometry {
                    MarkupGeometry::Text { text, .. } => *text = new_text.clone(),
                    MarkupGeometry::Callout { text, .. } => *text = new_text.clone(),
                    _ => {}
                }
                m.touch();
            }
            doc.touch();
        }
        ui_state.status = "Text updated".to_string();
    }
    // A fresh Text/Callout that never received any text is removed on cancel/
    // commit so no empty markup is left behind.
    let fresh_empty = original.is_empty() && !changed;
    if fresh_empty {
        if let Some(doc) = project.active_mut() {
            doc.remove_markup(id);
            doc.touch();
        }
        ui_state.status = "Text edit cancelled".to_string();
    }
    ui_state.text_edit = None;
    if fresh_empty {
        true
    } else {
        changed && editable
    }
}

/// Draw and interact with the page canvas. Returns `true` when the caller
/// should snapshot the project into undo history.
// Rendering context, not business logic: the canvas needs live handles to
// several UI-side caches at once; a bundled context struct is a later
// refactor if more handles accumulate.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_canvas(
    ui: &mut Ui,
    project: &mut Project,
    ui_state: &mut super::app::UiState,
    draw_state: &mut super::app::DrawState,
    last_page_pos: &mut Option<Pt>,
    page_textures: &mut std::collections::HashMap<String, egui::TextureHandle>,
    failed_page_textures: &mut std::collections::HashSet<String>,
    bg_error: &mut Option<String>,
    is_export: bool,
    show_minimap: bool,
) -> bool {
    let mut mutated = false;

    // Switching tools abandons whatever draft was in progress: a multi-point
    // path/angle must not survive (and later commit) under a different tool.
    // Discarded without a snapshot, since nothing was committed.
    if ui_state.active_tool != ui_state.last_tool {
        *draw_state = super::app::DrawState::Idle;
        ui_state.last_tool = ui_state.active_tool;
    }

    // The area around the page follows the application theme; the page itself
    // always stays paper-white so exported images match what is measured.
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
    if !is_export {
        let backdrop = ui.visuals().extreme_bg_color;
        painter.rect_filled(response.rect, egui::CornerRadius::ZERO, backdrop);
    }

    // Clamp the displayed page defensively; never panic on an out-of-range index.
    if let Some(doc) = project.active() {
        let count = doc.page_count();
        if count == 0 {
            return false;
        }
        if ui_state.current_page >= count {
            ui_state.current_page = count - 1;
        }
    } else {
        return false;
    }
    let page = match project
        .active()
        .and_then(|d| d.pages.get(ui_state.current_page).copied())
    {
        Some(p) => p,
        None => return false,
    };
    let origin = ui_state.page_origin;
    let pan = ui_state.pan;
    let zoom = ui_state.zoom;

    let page_rect = screen_rect(page.bounds(), origin, pan, zoom);
    let source_pdf = project.source_pdf.clone();
    // Paint the PDF page as a background texture when available; otherwise
    // keep the existing flat white fill. NOTE: first paint of a page may hitch
    // while PDFium rasterises + writes the cache PNG; the cache makes it
    // one-time. No background threads this slice (pdfium is not thread-safe).
    let mut painted_texture = false;
    if let Some(pdf_path) = source_pdf {
        // Track the page's on-screen device width so text stays sharp at high
        // zoom. Oversample slightly and quantise to coarse 256-px steps so
        // tiny zoom changes do not trigger re-rasterisation, while keeping the
        // number of cached widths small.
        let max_width = crate::data::pdf::MAX_RENDER_WIDTH_PX;
        // Floor at 2x the page box so a zoomed-out view is never softer than
        // the old fixed baseline; cap at `max_width` so a huge-format page
        // cannot make `clamp` panic on an inverted range.
        let min_width = ((page.width * 2.0).round() as u32).max(64).min(max_width);
        let desired = (page_rect.width() * 1.25).round() as u32;
        let clamped = desired.clamp(min_width, max_width);
        let target_width = (clamped.div_ceil(256) * 256).min(max_width);
        // Identity of one render attempt. A failed attempt is poisoned for the
        // project session so it is never retried: without this, a broken page
        // would re-rasterise (and re-report its error) on every repaint frame.
        let page_no = page.index;
        let attempt_key = format!("{pdf_path}:{page_no}:{target_width}");
        if !failed_page_textures.contains(&attempt_key) {
            match crate::data::pdf::render_page_to_cache(&pdf_path, page.index, target_width) {
                Ok(cache_path) => {
                    let key = cache_path.to_string_lossy().to_string();
                    if !page_textures.contains_key(&key) {
                        match std::fs::read(&cache_path) {
                            Ok(bytes) => match image::load_from_memory(&bytes) {
                                Ok(img) => {
                                    let rgba = img.to_rgba8();
                                    let (w, h) = rgba.dimensions();
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        [w as usize, h as usize],
                                        rgba.as_raw(),
                                    );
                                    let tex = ui.ctx().load_texture(
                                        key.clone(),
                                        color_image,
                                        egui::TextureOptions::LINEAR,
                                    );
                                    page_textures.insert(key.clone(), tex);
                                }
                                Err(e) => {
                                    failed_page_textures.insert(attempt_key);
                                    let msg = format!("PDF background unavailable: {e}");
                                    if bg_error.as_ref() != Some(&msg) {
                                        ui_state.status = msg.clone();
                                        *bg_error = Some(msg);
                                    }
                                }
                            },
                            Err(e) => {
                                failed_page_textures.insert(attempt_key);
                                let msg = format!("PDF background unavailable: {e}");
                                if bg_error.as_ref() != Some(&msg) {
                                    ui_state.status = msg.clone();
                                    *bg_error = Some(msg);
                                }
                            }
                        }
                    }
                    if let Some(tex) = page_textures.get(&key) {
                        painter.image(
                            tex.id(),
                            page_rect,
                            egui::Rect::from_min_max(
                                egui::Pos2::new(0.0, 0.0),
                                egui::Pos2::new(1.0, 1.0),
                            ),
                            egui::Color32::WHITE,
                        );
                        painted_texture = true;
                    }
                }
                Err(e) => {
                    failed_page_textures.insert(attempt_key);
                    let msg = format!("PDF background unavailable: {e}");
                    if bg_error.as_ref() != Some(&msg) {
                        ui_state.status = msg.clone();
                        *bg_error = Some(msg);
                    }
                }
            }
        }
    }
    if !painted_texture {
        painter.rect_filled(page_rect, egui::CornerRadius::same(12), Color32::WHITE);
    }
    if !is_export {
        painter.rect_stroke(
            page_rect,
            egui::CornerRadius::same(12),
            egui::Stroke::new(1.0, Color32::from_rgb(200, 206, 218)),
            egui::StrokeKind::Middle,
        );
    }

    // Canvas grid (F-405) and rulers (F-406). Both are guides: they are painted
    // before the markups so they always sit under the content.
    if !is_export {
        draw_grid(
            &painter,
            page.bounds(),
            ui_state.grid,
            ui_state.pan,
            ui_state.zoom,
            origin,
        );
        draw_rulers(
            &painter,
            ui,
            page_rect,
            page.bounds(),
            ui_state.pan,
            ui_state.zoom,
            origin,
            &page.scale,
        );
    }

    // Render markups in paint order (layer-major, then vector order). Computed
    // once per frame and reused for the measurement labels below.
    let paint_order: Vec<&crate::domain::Markup> = project
        .active()
        .map(|doc| doc.markups_in_paint_order(page.index).into_iter().collect())
        .unwrap_or_default();
    for m in &paint_order {
        draw_markup(
            &painter,
            origin,
            pan,
            zoom,
            m,
            project.settings.show_measurements,
        );
    }

    // Bates numbering overlay (spec F-804/F-805): a ghosted preview on the live
    // canvas, solid in the exported page image / print (both reuse this path
    // with `is_export = true`).
    draw_bates_overlay(
        &painter,
        origin,
        pan,
        zoom,
        &page,
        &project.settings.bates,
        is_export,
    );

    // Render selection. Handles are not drawn for a selected markup whose
    // layer has since been hidden: it cannot be picked or moved any more, and
    // the selection itself is kept.
    if let Some(doc) = project.active() {
        let selected_ids: Vec<Uuid> = ui_state.selected.iter().copied().collect();
        for id in selected_ids {
            if let Some(m) = doc.find_markup(id) {
                if doc.is_markup_visible(m) {
                    draw_selection(&painter, origin, pan, zoom, m);
                }
            }
        }
    }

    // Measurement labels on canvas.
    if project.settings.show_measurements {
        let unit = page.scale.unit;
        for m in &paint_order {
            // Angle and count already draw their value from geometry (the
            // degree text and the numbered dot), so an overlay label would
            // stack a second copy on the same anchor. Skip it for those.
            if matches!(m.kind, MarkupKind::MeasureAngle | MarkupKind::MeasureCount) {
                continue;
            }
            if let Some(meas) = m.measurement {
                let label = meas.label_with(unit, project.settings.precision);
                let anchor = to_screen(measure_anchor(m), pan, zoom, origin);
                draw_measure_badge(&painter, anchor, label);
            }
        }
        // Running total for the page's count markers, drawn once near the
        // page corner; each marker still draws its own number.
        if let Some(doc) = project.active() {
            let total = count_markers_on_page(&doc.markups, page.index);
            if total > 0 {
                let anchor = page_rect.min + Vec2::new(8.0, 8.0);
                draw_measure_badge(&painter, anchor, count_total_label(total));
            }
        }
    }
    // Release the borrow of `project` held by `paint_order` before we mutate it.
    drop(paint_order);
    let mut commit_edit = false;
    let mut text_edit_committed_this_frame = false;
    if ui_state.text_edit.is_some() {
        let edit_id = ui_state.text_edit.as_ref().unwrap().id;
        let sr = project
            .active()
            .and_then(|doc| doc.find_markup(edit_id))
            .map(|m| match &m.geometry {
                MarkupGeometry::Text { at, wrap, text } => {
                    let lines = (text.lines().count().max(1)) as f32;
                    // Same metrics the domain uses for bounds/hit testing, so
                    // the editor overlay covers exactly what is drawn.
                    let line_h = text_line_height(m.style.font_size);
                    screen_rect(
                        DRect::new(at.x, at.y, wrap.max(1.0), lines * line_h),
                        origin,
                        pan,
                        zoom,
                    )
                }
                MarkupGeometry::Callout { rect, .. } => screen_rect(*rect, origin, pan, zoom),
                _ => screen_rect(m.bounds(), origin, pan, zoom),
            });
        if let Some(sr) = sr {
            let edit = ui_state.text_edit.as_mut().unwrap();
            let resp = ui.put(
                sr,
                egui::TextEdit::multiline(&mut edit.text)
                    .id(egui::Id::new(("rubric_text_edit", edit.id))),
            );
            if !edit.focus_requested {
                resp.request_focus();
                edit.focus_requested = true;
            }
            if resp.lost_focus() {
                commit_edit = true;
            }
        } else {
            // Backstop: the markup being edited is gone from the active
            // document (undo past its creation, menu delete, file switch).
            // Drop the edit without a snapshot or commit; otherwise every
            // interaction arm stays gated on a dead overlay.
            ui_state.text_edit = None;
        }
    }
    if commit_edit {
        // The app shell preserves a status the canvas set this frame when it
        // pushes the text-commit snapshot.
        if commit_text_edit(ui_state, project) {
            mutated = true;
        }
        text_edit_committed_this_frame = true;
    }

    // Floating comment popover for a selected note pin (spec F-103). Rendered
    // above all markups; clicking empty canvas space closes it.
    if let Some(note_id) = ui_state.note_popover {
        let note_screen = project.active().and_then(|doc| {
            doc.find_markup(note_id)
                .map(|m| screen_rect(m.bounds(), origin, pan, zoom))
        });
        if let Some(sr) = note_screen {
            let zoom = ui_state.zoom.max(0.1);
            let popover = egui::Rect::from_min_max(
                Pos2::new(sr.max.x + 8.0 / zoom, sr.min.y),
                Pos2::new(
                    sr.max.x + 8.0 / zoom + 220.0 / zoom,
                    sr.min.y + 160.0 / zoom,
                ),
            );
            // Box painted directly (egui 0.36's Frame has no `none` ctor);
            // the interact response covers the whole popover area for the
            // "click outside" check.
            painter.rect_filled(
                popover,
                egui::CornerRadius::default(),
                ui.visuals().extreme_bg_color,
            );
            painter.rect_stroke(
                popover,
                egui::CornerRadius::default(),
                egui::Stroke::new(1.0, Color32::from_gray(150)),
                egui::StrokeKind::Middle,
            );
            let popover_resp =
                ui.interact(popover, egui::Id::new("note_popover"), egui::Sense::click());
            let inner = popover.shrink(8.0);
            let mut cy = inner.min.y;
            let mut next_row = move |h: f32| {
                let r = egui::Rect::from_min_size(
                    Pos2::new(inner.min.x, cy),
                    egui::vec2(inner.width(), h),
                );
                cy += h + 4.0;
                r
            };
            ui.put(
                next_row(20.0),
                egui::Label::new(egui::RichText::new("Note").heading()),
            );
            ui.put(next_row(1.0), egui::Separator::default());
            if let Some(doc) = project.active() {
                if let Some(m) = doc.find_markup(note_id) {
                    for c in &m.comments {
                        let label = format!("{} · {}", c.author, c.created.format("%H:%M:%S"));
                        ui.put(next_row(16.0), egui::Label::new(label));
                        ui.put(
                            next_row(14.0),
                            egui::Label::new(egui::RichText::new(&c.text).small()),
                        );
                        ui.put(next_row(1.0), egui::Separator::default());
                    }
                }
            }
            let note_editable = project
                .active()
                .and_then(|d| d.find_markup(note_id).map(|m| d.is_markup_editable(m)))
                .unwrap_or(false);
            let (_draft_r, add_resp) = ui
                .add_enabled_ui(note_editable, |ui| {
                    let dr = ui.put(
                        next_row(60.0),
                        egui::TextEdit::multiline(&mut ui_state.note_draft),
                    );
                    let ar = ui.put(next_row(20.0), egui::Button::new("Add comment"));
                    (dr, ar)
                })
                .inner;
            let close_resp = ui.put(next_row(20.0), egui::Button::new("Close"));
            if !note_editable {
                ui.put(next_row(16.0), egui::Label::new("Selection is locked"));
            }
            if add_resp.clicked() && note_editable && !ui_state.note_draft.trim().is_empty() {
                let author = project.settings.author.clone();
                if let Some(doc) = project.active_mut() {
                    if let Some(m) = doc.find_markup_mut(note_id) {
                        if m.add_comment(author.clone(), ui_state.note_draft.clone())
                            .is_some()
                        {
                            ui_state.note_draft.clear();
                            ui_state.status = "Comment added".to_string();
                            m.touch();
                            doc.touch();
                        }
                    }
                }
                mutated = true;
            }
            if close_resp.clicked() {
                ui_state.note_popover = None;
            }
            // Close when clicking elsewhere on the canvas, but not when the
            // click that opened this popover is still over the note itself.
            let click_on_note = response.hover_pos().is_some_and(|hp| sr.contains(hp));
            if response.clicked() && !popover_resp.hovered() && !click_on_note {
                ui_state.note_popover = None;
            }
            if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
                ui_state.note_popover = None;
            }
        } else {
            // The note no longer exists; close the popover.
            ui_state.note_popover = None;
        }
    }

    // Pointer position in page space. `raw_pos` is exact and drives
    // hit-testing — a click must still pick a markup that is not grid-aligned.
    // `page_pos` additionally honours snap-to-grid (F-405) and drives creation,
    // so new markups land on grid intersections.
    let raw_pos = response.hover_pos().map(|h| to_page(h, pan, zoom, origin));
    let page_pos = raw_pos.map(|p| ui_state.grid.snap(p));
    // The status-bar readout reports the *true* pointer (audit M9), matching
    // what hit-testing uses; snapping only affects where new markups land.
    *last_page_pos = raw_pos;

    // Pan state.
    let space_held = ui.ctx().input(|i| i.key_down(egui::Key::Space));
    if space_held {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }

    // Wheel zoom centred on cursor (and horizontal-scroll pan).
    if response.hovered() {
        let scroll = ui.ctx().input(|i| i.smooth_scroll_delta);
        if scroll != Vec2::ZERO {
            if scroll.x.abs() > scroll.y.abs() && scroll.y.abs() < 0.5 {
                ui_state.pan.x += scroll.x;
            } else if scroll.y.abs() > f32::EPSILON {
                if let Some(hover) = response.hover_pos() {
                    let page_before = to_page(hover, pan, zoom, origin);
                    let factor = (scroll.y * 0.001).exp();
                    let new_zoom = crate::domain::view::clamp_zoom(zoom * factor);
                    let new_pan = hover
                        - origin
                        - Vec2::new(page_before.x * new_zoom, page_before.y * new_zoom);
                    ui_state.zoom = new_zoom;
                    ui_state.pan = new_pan;
                }
            }
        }
    }

    let panning = space_held || response.dragged_by(egui::PointerButton::Middle);

    // Pan via middle-button or Space+primary drag.
    if panning && response.dragged() {
        let delta = response.drag_delta();
        if delta != Vec2::ZERO {
            ui_state.pan += delta;
        }
        // Cancel any in-progress draw while panning.
        *draw_state = super::app::DrawState::Idle;
        ui_state.draw_cancelled = true;
    }

    // Cancel in-progress draw if Space pressed mid-drag.
    if space_held && !matches!(*draw_state, super::app::DrawState::Idle) {
        *draw_state = super::app::DrawState::Idle;
        ui_state.draw_cancelled = true;
    }

    // Calibration rubber-band (v0.4 gap #4): while the Calibrate tool is mid
    // gesture the segment end follows the cursor, so the preview below stays
    // attached to it between the two clicks.
    if let Some(pp) = page_pos {
        if let super::app::DrawState::Calibrate { end, .. } = &mut *draw_state {
            *end = pp;
        }
    }

    // Preview for in-progress draw.
    let preview_color = Color32::from_rgb(30, 144, 255);
    match &*draw_state {
        super::app::DrawState::Bounds { start, end, .. } => {
            let r = DRect::from_corners(*start, *end);
            let sr = screen_rect(r, origin, pan, ui_state.zoom);
            painter.rect_stroke(
                sr,
                egui::CornerRadius::ZERO,
                egui::Stroke::new(1.0, preview_color),
                egui::StrokeKind::Middle,
            );
        }
        super::app::DrawState::Segment { start, end, .. } => {
            let sa = to_screen(*start, ui_state.pan, ui_state.zoom, origin);
            let sb = to_screen(*end, ui_state.pan, ui_state.zoom, origin);
            painter.line_segment([sa, sb], egui::Stroke::new(1.5, preview_color));
        }
        super::app::DrawState::Ink { strokes, current } => {
            let stroke = egui::Stroke::new(2.0, preview_color);
            for s in strokes.iter().chain(std::iter::once(current)) {
                let pts: Vec<Pos2> = s
                    .iter()
                    .map(|p| to_screen(*p, ui_state.pan, ui_state.zoom, origin))
                    .collect();
                if pts.len() >= 2 {
                    painter.add(egui::Shape::line(pts, stroke));
                }
            }
        }
        super::app::DrawState::Path { points, closed, .. } => {
            let stroke = egui::Stroke::new(1.5, preview_color);
            let mut pts: Vec<Pos2> = points
                .iter()
                .map(|p| to_screen(*p, ui_state.pan, ui_state.zoom, origin))
                .collect();
            // Rubber-band segment from the last committed point to the cursor.
            if let Some(cursor) = page_pos {
                let sc = to_screen(cursor, ui_state.pan, ui_state.zoom, origin);
                if !pts.is_empty() {
                    if let Some(last) = pts.last().copied() {
                        painter.line_segment([last, sc], stroke);
                    }
                }
                if *closed && pts.len() >= 2 {
                    // Auto-closing segment in preview.
                    if let Some(first) = pts.first().copied() {
                        painter.line_segment([sc, first], stroke);
                    }
                }
                pts.push(sc);
            }
            if pts.len() >= 2 {
                if *closed && pts.len() >= 3 {
                    painter.add(egui::Shape::closed_line(pts, stroke));
                } else {
                    painter.add(egui::Shape::line(pts, stroke));
                }
            }
        }
        super::app::DrawState::Angle { a, vertex, .. } => {
            let stroke = egui::Stroke::new(1.5, preview_color);
            let sa = to_screen(*a, ui_state.pan, ui_state.zoom, origin);
            if let Some(v) = vertex {
                let sv = to_screen(*v, ui_state.pan, ui_state.zoom, origin);
                painter.line_segment([sv, sa], stroke);
                if let Some(cursor) = page_pos {
                    let sc = to_screen(cursor, ui_state.pan, ui_state.zoom, origin);
                    painter.line_segment([sv, sc], stroke);
                }
            } else if let Some(cursor) = page_pos {
                let sc = to_screen(cursor, ui_state.pan, ui_state.zoom, origin);
                painter.line_segment([sa, sc], stroke);
            }
        }
        super::app::DrawState::Calibrate {
            start: Some(start),
            end,
        } => {
            // Reference segment under construction: line from the anchored
            // start to the cursor, plus a live length badge in the page's
            // display unit so the user can aim at the known feature before
            // committing the second click.
            let stroke = egui::Stroke::new(1.5, preview_color);
            let sa = to_screen(*start, ui_state.pan, ui_state.zoom, origin);
            let sb = to_screen(*end, ui_state.pan, ui_state.zoom, origin);
            painter.line_segment([sa, sb], stroke);
            painter.circle_filled(sa, 3.0, preview_color);
            let len = start.distance(*end);
            if len > 0.0 {
                let measured = page.scale.points_to_units(f64::from(len));
                draw_measure_badge(
                    &painter,
                    sb + Vec2::new(8.0, 8.0),
                    format_value(MeasureKind::Length, page.scale.unit, measured),
                );
            }
        }
        _ => {}
    }

    // Helper classification for multi-point tools.
    let is_path_kind = |k: MarkupKind| {
        matches!(
            k,
            MarkupKind::Polygon
                | MarkupKind::Polyline
                | MarkupKind::MeasurePerimeter
                | MarkupKind::MeasureArea
        )
    };
    let path_closed = |k: MarkupKind| {
        matches!(
            k,
            MarkupKind::Polygon | MarkupKind::MeasurePerimeter | MarkupKind::MeasureArea
        )
    };
    // Segment kinds are drag-drawn, never click-placed.
    let is_segment_kind = |k: MarkupKind| {
        matches!(
            k,
            MarkupKind::Line
                | MarkupKind::Arrow
                | MarkupKind::MeasureLength
                | MarkupKind::MeasureDiameter
                | MarkupKind::MeasureRadius
        )
    };

    // Click handling. `clicked_by` (not `clicked`) so a focused widget's fake
    // click cannot place or commit a markup on the canvas.
    if !text_edit_committed_this_frame
        && ui_state.text_edit.is_none()
        && response.clicked_by(egui::PointerButton::Primary)
        && !panning
    {
        let modifiers = ui.ctx().input(|i| i.modifiers);
        let shift = modifiers.shift;
        if let Some(pp) = page_pos {
            if ui_state.active_tool == Tool::Select {
                // Hit-testing ignores the grid (F-405).
                match pick_markup_id(
                    project,
                    page.index,
                    raw_pos.unwrap_or(pp),
                    hit_tolerance_page(zoom),
                ) {
                    Some(id) => {
                        if shift {
                            // Shift+click toggles selection without clearing.
                            if ui_state.selected.contains(&id) {
                                ui_state.selected.remove(&id);
                                ui_state.status =
                                    format!("Selected {} markup(s)", ui_state.selected.len());
                            } else {
                                ui_state.selected.insert(id);
                                ui_state.status =
                                    format!("Selected {} markup(s)", ui_state.selected.len());
                            }
                        } else {
                            ui_state.selected.clear();
                            ui_state.selected.insert(id);
                            ui_state.status = format!("Selected {}", id);
                        }
                        // Note pins open a floating comment popover (F-103);
                        // selecting anything else closes an open one.
                        let is_note = project
                            .active()
                            .and_then(|d| d.find_markup(id))
                            .is_some_and(|m| m.kind == MarkupKind::Note);
                        if is_note {
                            ui_state.note_popover = Some(id);
                            ui_state.note_draft.clear();
                        } else {
                            ui_state.note_popover = None;
                        }
                        // Enter edit mode on double-click for Text/Callout markups.
                        if response.double_clicked() && !shift {
                            let (kind_ok, editable, visible, original) = {
                                let doc = project.active();
                                let m = doc.and_then(|d| d.find_markup(id));
                                let kind_ok = m
                                    .map(|mk| {
                                        matches!(mk.kind, MarkupKind::Text | MarkupKind::Callout)
                                    })
                                    .unwrap_or(false);
                                let editable = doc
                                    .and_then(|d| m.map(|mk| d.is_markup_editable(mk)))
                                    .unwrap_or(false);
                                let visible = doc
                                    .and_then(|d| m.map(|mk| d.is_markup_visible(mk)))
                                    .unwrap_or(false);
                                let original = m
                                    .and_then(|mk| match &mk.geometry {
                                        MarkupGeometry::Text { text, .. } => Some(text.clone()),
                                        MarkupGeometry::Callout { text, .. } => Some(text.clone()),
                                        _ => None,
                                    })
                                    .unwrap_or_default();
                                (kind_ok, editable, visible, original)
                            };
                            if kind_ok {
                                if !editable {
                                    ui_state.status = "Locked layer — edits disabled".to_string();
                                } else if !visible {
                                    // not pickable when hidden; nothing to do
                                } else {
                                    ui_state.text_edit = Some(super::app::TextEditState {
                                        id,
                                        original: original.clone(),
                                        text: original,
                                        focus_requested: false,
                                    });
                                    ui_state.status = "Editing text".to_string();
                                }
                            }
                        }
                    }
                    None => {
                        if !shift {
                            ui_state.selected.clear();
                            ui_state.note_popover = None;
                        }
                    }
                }
                // Marquee drag start: dragging on empty canvas space with
                // Select tool. If a markup was under the pointer, it's a
                // move-drag, not a marquee.
                if response.drag_started()
                    && ui_state.text_edit.is_none()
                    && ui_state.marquee_start.is_none()
                    && pick_markup_id(
                        project,
                        page.index,
                        raw_pos.unwrap_or(pp),
                        hit_tolerance_page(zoom),
                    )
                    .is_none()
                {
                    ui_state.marquee_start = Some(response.hover_pos().unwrap_or(Pos2::ZERO));
                }
                // Marquee drag end: select all markups whose bounds intersect
                // the marquee rectangle.
                if ui_state.marquee_start.is_some() && response.drag_stopped() {
                    let start = ui_state.marquee_start.take().unwrap();
                    let end = response.interact_pointer_pos().unwrap_or(start);
                    let rect = egui::Rect::from_two_pos(start, end);

                    if let Some(doc) = project.active() {
                        for m in doc.markups_in_paint_order(page.index) {
                            let mr = screen_rect(m.bounds(), origin, pan, zoom);
                            if rect.intersects(mr) {
                                ui_state.selected.insert(m.id);
                            }
                        }
                    }
                    let n = ui_state.selected.len();
                    if n > 0 {
                        ui_state.status =
                            format!("Selected {} markup{}", n, if n == 1 { "" } else { "s" });
                    } else {
                        ui_state.status = "Nothing selected".to_string();
                    }
                }
            } else if let Some(kind) = ui_state.active_tool.to_kind() {
                if is_path_kind(kind) {
                    let closed = path_closed(kind);
                    // A double-click in egui is also a click, so the point of
                    // that release must not be appended before committing.
                    if response.double_clicked() {
                        let commit_points = match &*draw_state {
                            super::app::DrawState::Path { points, .. } => points.clone(),
                            _ => Vec::new(),
                        };
                        let min = if closed { 3 } else { 2 };
                        if path_points_ok(&commit_points, min, closed) {
                            let author = project.settings.author.clone();
                            let geom = MarkupGeometry::Path {
                                points: commit_points,
                                closed,
                            };
                            let m = finish_markup(
                                Markup::new(page.index, kind, geom, author),
                                ui_state,
                            );
                            if let Some(doc) = project.active_mut() {
                                doc.add_markup(m);
                            }
                            ui_state.status = format!("Added {}", kind.label());
                            mutated = true;
                        }
                        *draw_state = super::app::DrawState::Idle;
                    } else {
                        // Append point to in-progress path (or start one).
                        match draw_state {
                            super::app::DrawState::Path { points, .. } => {
                                points.push(pp);
                            }
                            // Refused: the default layer is locked.
                            _ if !creation_allowed(project, ui_state) => {}
                            _ => {
                                *draw_state = super::app::DrawState::Path {
                                    points: vec![pp],
                                    kind,
                                    closed,
                                };
                            }
                        }
                    }
                } else if kind == MarkupKind::MeasureAngle {
                    match draw_state {
                        super::app::DrawState::Angle {
                            a,
                            vertex: Some(_v),
                            ..
                        } => {
                            // Third click: commit.
                            let a_pt = *a;
                            let vertex_pt = *_v;
                            let b_pt = pp;
                            let distinct = a_pt != vertex_pt && b_pt != vertex_pt && a_pt != b_pt;
                            if distinct {
                                let author = project.settings.author.clone();
                                let geom = MarkupGeometry::Angle {
                                    a: a_pt,
                                    vertex: vertex_pt,
                                    b: b_pt,
                                };
                                let m = finish_markup(
                                    Markup::new(page.index, kind, geom, author),
                                    ui_state,
                                );
                                if let Some(doc) = project.active_mut() {
                                    doc.add_markup(m);
                                }
                                ui_state.status = format!("Added {}", kind.label());
                                mutated = true;
                            }
                            *draw_state = super::app::DrawState::Idle;
                        }
                        super::app::DrawState::Angle { vertex, .. } => {
                            // Second click: set vertex.
                            *vertex = Some(pp);
                        }
                        _ => {
                            // First click starts the sequence: refuse it when
                            // the default layer is locked.
                            if creation_allowed(project, ui_state) {
                                *draw_state = super::app::DrawState::Angle {
                                    a: pp,
                                    vertex: None,
                                    kind,
                                };
                            }
                        }
                    }
                } else if is_segment_kind(kind) {
                    // Segment tools are drag-drawn; a bare click commits
                    // nothing (and pushes no snapshot).
                } else if !creation_allowed(project, ui_state) {
                    // Default layer locked: refuse to create an uneditable markup.
                } else {
                    let author = project.settings.author.clone();
                    let inv = 1.0 / ui_state.zoom.max(0.1);
                    let geom = match kind {
                        MarkupKind::Text => MarkupGeometry::Text {
                            at: Pt::new(pp.x, pp.y),
                            wrap: 40.0 * inv,
                            text: String::new(),
                        },
                        MarkupKind::Callout => MarkupGeometry::Callout {
                            rect: DRect::new(pp.x, pp.y, 40.0 * inv, 24.0 * inv),
                            tail: Pt::new(pp.x + 20.0 * inv, pp.y + 24.0 * inv + 20.0 * inv),
                            text: String::new(),
                        },
                        MarkupKind::Rectangle
                        | MarkupKind::Ellipse
                        | MarkupKind::Cloud
                        | MarkupKind::Stamp
                        | MarkupKind::Note => {
                            MarkupGeometry::Bounds(DRect::new(pp.x, pp.y, 40.0 * inv, 24.0 * inv))
                        }
                        MarkupKind::Highlight => {
                            // Snap to the text line under the click when one is
                            // close enough: anchor, text and metrics all come
                            // from the real line so the markup tracks the text.
                            // Without a source PDF there is nothing to extract.
                            let line = project.source_pdf.as_deref().and_then(|source| {
                                find_text_line_at_point(
                                    Path::new(source),
                                    page.index,
                                    pp.x,
                                    pp.y,
                                    10.0,
                                )
                            });
                            match line {
                                Some(line) => MarkupGeometry::TextAnchor {
                                    at: line.at,
                                    text: line.text,
                                    line_height: line.line_height,
                                    line_count: 1,
                                    text_width: line.width,
                                },
                                None => MarkupGeometry::TextAnchor {
                                    at: pp,
                                    text: String::new(),
                                    line_height: 12.0 * inv,
                                    line_count: 1,
                                    text_width: 40.0 * inv,
                                },
                            }
                        }
                        MarkupKind::Strikeout => MarkupGeometry::TextAnchor {
                            at: Pt::new(pp.x, pp.y),
                            text: String::new(),
                            line_height: 12.0 * inv,
                            line_count: 1,
                            text_width: 40.0 * inv,
                        },
                        MarkupKind::Underline => MarkupGeometry::TextAnchor {
                            at: Pt::new(pp.x, pp.y),
                            text: String::new(),
                            line_height: 12.0 * inv,
                            line_count: 1,
                            text_width: 40.0 * inv,
                        },
                        MarkupKind::Squiggly => MarkupGeometry::TextAnchor {
                            at: Pt::new(pp.x, pp.y),
                            text: String::new(),
                            line_height: 12.0 * inv,
                            line_count: 1,
                            text_width: 40.0 * inv,
                        },
                        MarkupKind::MeasureCount => {
                            // Number each new marker after the highest already
                            // on this page, so the running count never repeats
                            // (even after a marker is deleted).
                            let number = match project.active() {
                                Some(doc) => next_count_number(&doc.markups, page.index),
                                None => 1,
                            };
                            MarkupGeometry::Count { at: pp, number }
                        }
                        MarkupKind::Ink => MarkupGeometry::Ink {
                            strokes: vec![vec![pp]],
                        },
                        // Every MarkupKind is matched above; this arm stays as
                        // a safe default for kinds added in the future.
                        _ => MarkupGeometry::Bounds(DRect::new(pp.x, pp.y, 40.0 * inv, 24.0 * inv)),
                    };
                    let m = finish_markup(Markup::new(page.index, kind, geom, author), ui_state);
                    let new_id = m.id;
                    let is_text = matches!(kind, MarkupKind::Text | MarkupKind::Callout);
                    if let Some(doc) = project.active_mut() {
                        doc.add_markup(m);
                    }
                    ui_state.status = format!("Added {}", kind.label());
                    mutated = true;
                    if is_text {
                        ui_state.text_edit = Some(super::app::TextEditState {
                            id: new_id,
                            original: String::new(),
                            text: String::new(),
                            focus_requested: false,
                        });
                        ui_state.status = "Editing text".to_string();
                    }
                    if kind == MarkupKind::Note {
                        ui_state.note_popover = Some(new_id);
                        ui_state.note_draft.clear();
                        ui_state.status = "Note added — add a comment".to_string();
                    }
                }
            } else if ui_state.active_tool == Tool::Calibrate {
                // Drag calibration (v0.4 gap #4): click 1 anchors the known
                // distance, click 2 commits the reference segment and opens
                // the calibration dialog. Never creates a markup.
                if ui_state.calibration.is_some() {
                    // The dialog owns the flow; ignore canvas clicks behind it.
                } else {
                    let anchored = match &*draw_state {
                        super::app::DrawState::Calibrate {
                            start: Some(start), ..
                        } => Some(*start),
                        _ => None,
                    };
                    if let Some(start) = anchored {
                        // Second click: commit the reference segment and open
                        // the dialog, pre-filled with the segment's length
                        // under the page's *current* scale.
                        let len = start.distance(pp);
                        if !calibration_segment_ok(len) {
                            // A zero-length reference calibrates nothing:
                            // cancel without a snapshot.
                            *draw_state = super::app::DrawState::Idle;
                            ui_state.status =
                                "Calibration cancelled — segment too short".to_string();
                        } else {
                            let measured = page.scale.points_to_units(f64::from(len));
                            ui_state.calibration = Some(super::app::CalibrationDialog {
                                page: ui_state.current_page,
                                segment_points: f64::from(len),
                                real_units: measured.max(f64::EPSILON),
                                unit: page.scale.unit,
                            });
                            *draw_state = super::app::DrawState::Idle;
                            ui_state.status = "Calibrating — enter the real distance".to_string();
                        }
                    } else {
                        // First click anchors the known-distance segment.
                        *draw_state = super::app::DrawState::Calibrate {
                            start: Some(pp),
                            end: pp,
                        };
                        ui_state.status =
                            "Calibrate: click the second point of the known distance".to_string();
                    }
                }
            }
        }
    }

    // Context menu (spec F-603). Attached only when no multi-point draft is in
    // progress: a right-click during a draft commits that draft (below) rather
    // than opening the menu.
    if matches!(*draw_state, super::app::DrawState::Idle) {
        let _ = response.context_menu(|ui| {
            super::menu::draw_editor_context_menu(ui, ui_state);
        });
    }

    // Right-click commits an in-progress multi-point sequence.
    if response.clicked_by(egui::PointerButton::Secondary) && !panning {
        match draw_state {
            super::app::DrawState::Path {
                points,
                kind,
                closed,
            } => {
                let min = if *closed { 3 } else { 2 };
                if path_points_ok(points, min, *closed) {
                    let author = project.settings.author.clone();
                    let geom = MarkupGeometry::Path {
                        points: points.clone(),
                        closed: *closed,
                    };
                    let m = finish_markup(Markup::new(page.index, *kind, geom, author), ui_state);
                    if let Some(doc) = project.active_mut() {
                        doc.add_markup(m);
                    }
                    ui_state.status = format!("Added {}", kind.label());
                    mutated = true;
                }
                *draw_state = super::app::DrawState::Idle;
            }
            super::app::DrawState::Angle {
                a,
                vertex: Some(v),
                kind,
            } => {
                let kind = *kind;
                // Need a third point; use cursor if available, else cancel.
                if let Some(pp) = page_pos {
                    let distinct = a != v && pp != *v && *a != pp;
                    if distinct {
                        let author = project.settings.author.clone();
                        let geom = MarkupGeometry::Angle {
                            a: *a,
                            vertex: *v,
                            b: pp,
                        };
                        let m =
                            finish_markup(Markup::new(page.index, kind, geom, author), ui_state);
                        if let Some(doc) = project.active_mut() {
                            doc.add_markup(m);
                        }
                        ui_state.status = format!("Added {}", kind.label());
                        mutated = true;
                    }
                }
                *draw_state = super::app::DrawState::Idle;
            }
            super::app::DrawState::Calibrate { .. } => {
                // Right-click abandons the reference segment (no snapshot);
                // the context menu stays closed while a draft is in progress.
                *draw_state = super::app::DrawState::Idle;
                ui_state.status = "Calibration cancelled".to_string();
            }
            _ => {}
        }
    }

    // Enter commits an in-progress multi-point sequence — but not while a text
    // field has focus (Enter there belongs to the widget, not the canvas).
    let enter_pressed = ui.ctx().input(|i| i.key_pressed(egui::Key::Enter));
    let keyboard_taken = ui.ctx().text_edit_focused();
    if enter_pressed && !keyboard_taken && !panning {
        match draw_state {
            super::app::DrawState::Path {
                points,
                kind,
                closed,
            } => {
                let min = if *closed { 3 } else { 2 };
                if path_points_ok(points, min, *closed) {
                    let author = project.settings.author.clone();
                    let geom = MarkupGeometry::Path {
                        points: points.clone(),
                        closed: *closed,
                    };
                    let m = finish_markup(Markup::new(page.index, *kind, geom, author), ui_state);
                    if let Some(doc) = project.active_mut() {
                        doc.add_markup(m);
                    }
                    ui_state.status = format!("Added {}", kind.label());
                    mutated = true;
                }
                *draw_state = super::app::DrawState::Idle;
            }
            super::app::DrawState::Angle {
                a,
                vertex: Some(v),
                kind,
            } => {
                let kind = *kind;
                if let Some(pp) = page_pos {
                    let distinct = *a != *v && pp != *v && *a != pp;
                    if distinct {
                        let author = project.settings.author.clone();
                        let geom = MarkupGeometry::Angle {
                            a: *a,
                            vertex: *v,
                            b: pp,
                        };
                        let m =
                            finish_markup(Markup::new(page.index, kind, geom, author), ui_state);
                        if let Some(doc) = project.active_mut() {
                            doc.add_markup(m);
                        }
                        ui_state.status = format!("Added {}", kind.label());
                        mutated = true;
                    }
                }
                *draw_state = super::app::DrawState::Idle;
            }
            _ => {}
        }
    }

    // Marquee selection overlay: draw while dragging on empty canvas.
    if let Some(start) = ui_state.marquee_start {
        let end = response.interact_pointer_pos().unwrap_or(start);
        let rect = egui::Rect::from_two_pos(start, end);
        painter.rect_stroke(
            rect,
            egui::CornerRadius::same(0),
            egui::Stroke::new(1.0, egui::Color32::from_black_alpha(120)),
            egui::StrokeKind::Middle,
        );
        painter.rect_filled(
            rect,
            egui::CornerRadius::same(0),
            egui::Color32::from_black_alpha(30),
        );
    }

    // Drag-to-draw (bounds/segment/ink only; multi-point tools use clicks).
    let multi_tool = ui_state.active_tool.to_kind().is_some_and(|k| {
        matches!(
            k,
            MarkupKind::Polygon
                | MarkupKind::Polyline
                | MarkupKind::MeasurePerimeter
                | MarkupKind::MeasureArea
                | MarkupKind::MeasureAngle
        )
    });
    if !panning
        && !matches!(ui_state.active_tool, Tool::Select | Tool::Calibrate)
        && !multi_tool
        && ui_state.text_edit.is_none()
    {
        // A drag cancelled with Esc stays cancelled until this drag ends; the
        // pointer is still down, so we must not spawn a fresh draft from it.
        if response.drag_started() || response.drag_stopped() {
            ui_state.draw_cancelled = false;
        }

        if response.dragged() {
            // Start a new draw if idle and this drag was not cancelled.
            // `creation_allowed` refuses the draft when the default layer is
            // locked (and reports it in the status bar).
            if matches!(*draw_state, super::app::DrawState::Idle)
                && !ui_state.draw_cancelled
                && creation_allowed(project, ui_state)
            {
                if let Some(pp) = page_pos {
                    if let Some(kind) = ui_state.active_tool.to_kind() {
                        match kind {
                            MarkupKind::Ink => {
                                *draw_state = super::app::DrawState::Ink {
                                    strokes: Vec::new(),
                                    current: vec![pp],
                                };
                            }
                            MarkupKind::Line
                            | MarkupKind::Arrow
                            | MarkupKind::MeasureLength
                            | MarkupKind::MeasureDiameter
                            | MarkupKind::MeasureRadius => {
                                *draw_state = super::app::DrawState::Segment {
                                    start: pp,
                                    end: pp,
                                    kind,
                                };
                            }
                            _ => {
                                *draw_state = super::app::DrawState::Bounds {
                                    start: pp,
                                    end: pp,
                                    kind,
                                };
                            }
                        }
                    }
                }
            }
            // Update in-progress draw.
            if let Some(pp) = page_pos {
                match draw_state {
                    super::app::DrawState::Bounds { end, .. } => *end = pp,
                    super::app::DrawState::Segment { end, .. } => *end = pp,
                    super::app::DrawState::Ink { current, .. } => {
                        if let Some(last) = current.last().copied() {
                            let dscreen = to_screen(pp, ui_state.pan, ui_state.zoom, origin)
                                .distance(to_screen(last, ui_state.pan, ui_state.zoom, origin));
                            if dscreen >= 2.0 {
                                current.push(pp);
                            }
                        } else {
                            current.push(pp);
                        }
                    }
                    _ => {}
                }
            }
        }

        if response.drag_stopped() {
            // Commit.
            if ui_state.active_tool.to_kind().is_some() {
                let author = project.settings.author.clone();
                match std::mem::replace(draw_state, super::app::DrawState::Idle) {
                    super::app::DrawState::Bounds { start, end, kind } => {
                        let r = DRect::from_corners(start, end);
                        // Drop degenerate (near-zero extent) drags without a
                        // snapshot.
                        if r.w.abs() < 1.0 || r.h.abs() < 1.0 {
                            // nothing
                        } else {
                            let geom = match kind {
                                MarkupKind::Text => MarkupGeometry::Text {
                                    at: Pt::new(r.x, r.y),
                                    wrap: r.w,
                                    text: String::new(),
                                },
                                MarkupKind::Callout => MarkupGeometry::Callout {
                                    rect: r,
                                    // The press point anchors the tail; the dragged
                                    // rect is the box (see doc comment in task).
                                    tail: start,
                                    text: String::new(),
                                },
                                _ => MarkupGeometry::Bounds(r),
                            };
                            let m = finish_markup(
                                Markup::new(page.index, kind, geom, author),
                                ui_state,
                            );
                            let new_id = m.id;
                            let is_text = matches!(kind, MarkupKind::Text | MarkupKind::Callout);
                            if let Some(doc) = project.active_mut() {
                                doc.add_markup(m);
                            }
                            ui_state.status = format!("Added {}", kind.label());
                            mutated = true;
                            if is_text {
                                ui_state.text_edit = Some(super::app::TextEditState {
                                    id: new_id,
                                    original: String::new(),
                                    text: String::new(),
                                    focus_requested: false,
                                });
                                ui_state.status = "Editing text".to_string();
                            }
                            if kind == MarkupKind::Note {
                                ui_state.note_popover = Some(new_id);
                                ui_state.note_draft.clear();
                                ui_state.status = "Note added — add a comment".to_string();
                            }
                        }
                    }
                    super::app::DrawState::Segment { start, end, kind } => {
                        if start.distance(end) < 1.0 {
                            // drop degenerate
                        } else {
                            let geom = MarkupGeometry::Segment { a: start, b: end };
                            let m = finish_markup(
                                Markup::new(page.index, kind, geom, author),
                                ui_state,
                            );
                            if let Some(doc) = project.active_mut() {
                                doc.add_markup(m);
                            }
                            ui_state.status = format!("Added {}", kind.label());
                            mutated = true;
                        }
                    }
                    super::app::DrawState::Ink {
                        mut strokes,
                        current,
                    } => {
                        if !current.is_empty() {
                            strokes.push(current);
                        }
                        if !strokes.is_empty() && ink_stroke_ok(&strokes) {
                            let geom = MarkupGeometry::Ink { strokes };
                            let m = finish_markup(
                                Markup::new(page.index, MarkupKind::Ink, geom, author),
                                ui_state,
                            );
                            if let Some(doc) = project.active_mut() {
                                doc.add_markup(m);
                            }
                            ui_state.status = "Added Ink".to_string();
                            mutated = true;
                        }
                    }
                    other => {
                        *draw_state = other;
                    }
                }
            }
            *draw_state = super::app::DrawState::Idle;
        }
    }

    // If an edit just started, silently drop any in-progress move/resize drag
    // so no stray snapshot is pushed for this gesture.
    if ui_state.text_edit.is_some() {
        ui_state.move_selection = false;
        ui_state.move_changed = false;
        ui_state.resize_corner = None;
        ui_state.resize_point = None;
        ui_state.resize_changed = false;
    }
    // Drag to resize the selection from a corner handle (v0.4 gap #5).
    // A drag starting on a corner handle of a selected, editable markup
    // resizes it; dragging the body or center handle moves it (below).
    if !panning && matches!(ui_state.active_tool, Tool::Select) && ui_state.text_edit.is_none() {
        if response.drag_started() {
            if let Some(pp) = raw_pos {
                if let Some(doc) = project.active() {
                    let screen_pp = to_screen(pp, pan, zoom, origin);
                    let mut picked_point = None;
                    let mut picked_corner = None;
                    for id in &ui_state.selected {
                        let editable = doc
                            .find_markup(*id)
                            .map(|m| doc.is_markup_editable(m))
                            .unwrap_or(false);
                        if !editable || !ui_state.selected.contains(id) {
                            continue;
                        }
                        let Some(m) = doc.find_markup(*id) else {
                            continue;
                        };
                        // Point handles first: an endpoint/vertex grab is more
                        // specific than a bounding-box corner grab.
                        if let Some(idx) = point_handle_at(m, screen_pp, origin, pan, zoom) {
                            picked_point = Some(idx);
                            break;
                        }
                        if matches!(m.geometry, MarkupGeometry::Bounds(_)) {
                            let b = screen_rect(m.bounds(), origin, pan, zoom);
                            if let Some(c) = handle_at(b, screen_pp, ui_state.zoom) {
                                picked_corner = Some(c);
                                break;
                            }
                        }
                    }
                    if picked_point.is_some() || picked_corner.is_some() {
                        ui_state.resize_point = picked_point;
                        ui_state.resize_corner = picked_corner;
                        ui_state.move_selection = false;
                        ui_state.resize_changed = false;
                    }
                }
            }
        }

        if let Some(corner) = ui_state.resize_corner {
            let delta = if response.dragged() {
                response.drag_delta()
            } else {
                Vec2::ZERO
            };
            if delta != Vec2::ZERO {
                let dx = delta.x / ui_state.zoom;
                let dy = delta.y / ui_state.zoom;
                let sel: Vec<Uuid> = ui_state.selected.iter().copied().collect();
                if let Some(doc) = project.active_mut() {
                    for id in sel {
                        let editable = doc
                            .find_markup(id)
                            .map(|m| doc.is_markup_editable(m))
                            .unwrap_or(false);
                        if !editable {
                            continue;
                        }
                        if let Some(m) = doc.find_markup_mut(id) {
                            if let MarkupGeometry::Bounds(r) = &m.geometry {
                                let old = match corner {
                                    Corner::TopLeft => Pt::new(r.x, r.y),
                                    Corner::TopRight => Pt::new(r.right(), r.y),
                                    Corner::BottomLeft => Pt::new(r.x, r.bottom()),
                                    Corner::BottomRight => Pt::new(r.right(), r.bottom()),
                                };
                                let new_pt = Pt::new(old.x + dx, old.y + dy);
                                if m.resize_bounds(corner, new_pt) {
                                    if m.kind.is_measure() {
                                        m.recompute_measurement(page.scale);
                                    }
                                    ui_state.resize_changed = true;
                                }
                            }
                        }
                    }
                }
            }
            // Snapshot once, when the drag ends (never per frame), and only
            // when the drag actually resized a markup.
            if response.drag_stopped() {
                ui_state.resize_corner = None;
                mutated |= ui_state.resize_changed;
                ui_state.resize_changed = false;
            }
        } else if let Some(index) = ui_state.resize_point {
            // Drag to reshape the markup from a draggable endpoint/vertex
            // (v0.4 gap #5 follow-up). The same index applies to every
            // selected markup, mirroring how a corner drag reshapes each.
            let delta = if response.dragged() {
                response.drag_delta()
            } else {
                Vec2::ZERO
            };
            if delta != Vec2::ZERO {
                let dx = delta.x / ui_state.zoom;
                let dy = delta.y / ui_state.zoom;
                let sel: Vec<Uuid> = ui_state.selected.iter().copied().collect();
                if let Some(doc) = project.active_mut() {
                    for id in sel {
                        let editable = doc
                            .find_markup(id)
                            .map(|m| doc.is_markup_editable(m))
                            .unwrap_or(false);
                        if !editable {
                            continue;
                        }
                        if let Some(m) = doc.find_markup_mut(id) {
                            let Some(old) = m.draggable_points().get(index).copied() else {
                                continue;
                            };
                            let new_pt = Pt::new(old.x + dx, old.y + dy);
                            if m.move_point(index, new_pt) {
                                if m.kind.is_measure() {
                                    m.recompute_measurement(page.scale);
                                }
                                ui_state.resize_changed = true;
                            }
                        }
                    }
                }
            }
            // Snapshot once, when the drag ends (never per frame), and only
            // when the drag actually reshaped a markup.
            if response.drag_stopped() {
                ui_state.resize_point = None;
                mutated |= ui_state.resize_changed;
                ui_state.resize_changed = false;
            }
        }
    }

    // Drag to move the selection (when not drawing/panning).
    if !panning && matches!(ui_state.active_tool, Tool::Select) && ui_state.text_edit.is_none() {
        if response.drag_started() {
            // Only a drag that starts on a selected, editable markup moves it;
            // dragging from empty space must not touch the selection or history.
            // Hit-testing ignores the grid (F-405): a drag must grab the markup
            // the pointer is actually over, grid-aligned or not.
            let picked = raw_pos.and_then(|pp| {
                pick_markup_id(project, page.index, pp, hit_tolerance_page(ui_state.zoom))
            });
            if let Some(id) = picked {
                let editable = project
                    .active()
                    .and_then(|d| d.find_markup(id).map(|m| d.is_markup_editable(m)))
                    .unwrap_or(false);
                if editable {
                    let shift = ui.ctx().input(|i| i.modifiers.shift);
                    if shift {
                        ui_state.selected.insert(id);
                    } else {
                        ui_state.selected.clear();
                        ui_state.selected.insert(id);
                    }
                }
            }
            let can_move = picked
                .map(|id| {
                    let editable = project
                        .active()
                        .and_then(|d| d.find_markup(id).map(|m| d.is_markup_editable(m)))
                        .unwrap_or(false);
                    ui_state.resize_corner.is_none()
                        && ui_state.resize_point.is_none()
                        && ui_state.selected.contains(&id)
                        && editable
                })
                .unwrap_or(false);
            ui_state.move_selection = can_move;
            // Nothing has moved yet in this drag. The flag lives in `UiState`
            // because egui clears `dragged()` on the release frame, so the
            // release frame itself cannot see whether the drag moved anything.
            ui_state.move_changed = false;
        }

        if ui_state.move_selection {
            let delta = if response.dragged() {
                response.drag_delta()
            } else {
                Vec2::ZERO
            };
            if delta != Vec2::ZERO {
                let dx = delta.x / ui_state.zoom;
                let dy = delta.y / ui_state.zoom;
                let sel: Vec<Uuid> = ui_state.selected.iter().copied().collect();
                if let Some(doc) = project.active_mut() {
                    for id in sel {
                        let editable = doc
                            .find_markup(id)
                            .map(|m| doc.is_markup_editable(m))
                            .unwrap_or(false);
                        if !editable {
                            continue;
                        }
                        if let Some(m) = doc.find_markup_mut(id) {
                            m.geometry.translate(dx, dy);
                            m.touch();
                            if m.kind.is_measure() {
                                m.recompute_measurement(page.scale);
                            }
                            ui_state.move_changed = true;
                        }
                    }
                }
            }
            // Snapshot once, when the drag ends (never per frame), and only
            // when the drag actually moved a markup — a click-release on a
            // selected markup with zero delta changes nothing.
            if response.drag_stopped() {
                ui_state.move_selection = false;
                mutated |= ui_state.move_changed;
                ui_state.move_changed = false;
            }
        }
    }

    // Cancel a freshly created empty text edit on Escape without leaving an
    // empty markup behind.
    if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) && ui_state.text_edit.is_some() {
        let (id, original, text) = {
            let e = ui_state.text_edit.as_ref().unwrap();
            (e.id, e.original.clone(), e.text.clone())
        };
        if original.is_empty() && (text.is_empty() || text == original) {
            if let Some(doc) = project.active_mut() {
                doc.remove_markup(id);
                doc.touch();
            }
            ui_state.text_edit = None;
            mutated = true;
            ui_state.status = "Text edit cancelled".to_string();
            ui.ctx()
                .input_mut(|i| i.consume_key(egui::Modifiers::default(), egui::Key::Escape));
        }
    }

    // Draw minimap overlay in corner of canvas
    if show_minimap && !is_export {
        if let Some(doc) = project.active() {
            if let Some(page) = doc.pages.get(ui_state.current_page) {
                let page_bounds = page.bounds();
                let viewport_rect = response.rect;
                draw_minimap(
                    &painter,
                    page_bounds,
                    viewport_rect,
                    zoom,
                    ui.available_size(),
                    origin,
                    pan,
                );
            }
        }
    }

    mutated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_tolerance_scales_with_zoom() {
        assert!((hit_tolerance_page(1.0) - 6.0).abs() < 1e-6);
        assert!((hit_tolerance_page(2.0) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn ink_stroke_ok_rejects_single_point() {
        assert!(!ink_stroke_ok(&[vec![Pt::new(0.0, 0.0)]]));
        assert!(ink_stroke_ok(&[vec![
            Pt::new(0.0, 0.0),
            Pt::new(10.0, 0.0)
        ]]));
    }

    #[test]
    fn calibration_segment_ok_refuses_degenerate_references() {
        assert!(calibration_segment_ok(1.0), "the minimum is accepted");
        assert!(calibration_segment_ok(720.0));
        assert!(!calibration_segment_ok(0.0), "a point calibrates nothing");
        assert!(!calibration_segment_ok(0.5), "sub-point drag is refused");
        assert!(!calibration_segment_ok(-10.0));
        assert!(!calibration_segment_ok(f32::NAN));
        assert!(!calibration_segment_ok(f32::INFINITY));
    }

    #[test]
    fn point_handle_at_finds_endpoints_but_not_the_middle() {
        let m = Markup::new(
            0,
            MarkupKind::Line,
            MarkupGeometry::Segment {
                a: Pt::new(0.0, 0.0),
                b: Pt::new(100.0, 100.0),
            },
            "t",
        );
        // Identity transform: page points land on the same screen coords.
        assert_eq!(
            point_handle_at(&m, Pos2::new(0.0, 0.0), Pos2::ZERO, Vec2::ZERO, 1.0),
            Some(0)
        );
        assert_eq!(
            point_handle_at(&m, Pos2::new(100.0, 100.0), Pos2::ZERO, Vec2::ZERO, 1.0),
            Some(1)
        );
        assert_eq!(
            point_handle_at(&m, Pos2::new(50.0, 50.0), Pos2::ZERO, Vec2::ZERO, 1.0),
            None,
            "the middle of the segment carries no point handle"
        );
    }
}
