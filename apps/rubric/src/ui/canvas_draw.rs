//! Canvas drawing primitives: grid, rulers, Bates overlay, minimap, selection.
//!
//! All functions take screen-space/page-space coordinates and draw directly
//! onto an egui [`Painter`].

use eframe::egui::{self, Align2, Color32, FontId, Painter, Pos2, Stroke, StrokeKind, Ui, Vec2};

use crate::domain::bates::BatesConfig;
use crate::domain::geom::Pt;
use crate::domain::markup::Markup;
use crate::domain::{ruler_ticks, Grid, Page, Rect as DRect, Scale};
use crate::ui::canvas_geom::{screen_rect, to_screen};

/// Draw a small dark measurement badge with white text at `anchor` (screen
/// space). Used for measurement labels and the running count total.
pub(crate) fn draw_measure_badge(painter: &Painter, anchor: Pos2, label: String) {
    let galley = painter.layout_no_wrap(label, FontId::proportional(12.0), Color32::WHITE);
    let text_rect = galley.rect.translate(Vec2::new(anchor.x, anchor.y));
    painter.rect_filled(text_rect.expand(4.0), 4.0, Color32::from_black_alpha(180));
    painter.galley(anchor, galley, Color32::WHITE);
}

/// Canvas grid (spec F-405). Guides only: drawn under the markups and skipped
/// entirely when the pitch would be too fine to resolve, so a far zoom never
/// produces an unreadable wash.
pub(crate) fn draw_grid(
    painter: &Painter,
    bounds: DRect,
    grid: Grid,
    pan: Vec2,
    zoom: f32,
    origin: Pos2,
) {
    if !grid.should_draw(zoom) {
        return;
    }
    let step = grid.spacing as f64;
    let stroke = egui::Stroke::new(1.0, Color32::from_rgb(203, 206, 221));
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

/// Rulers (spec F-406): labelled ticks in the margin just outside the page, so
/// a canvas position can be read in page points. Tick spacing follows the
/// 1/2/5 ladder in `domain::grid`, chosen from the zoom so labels never
/// collide.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_rulers(
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

/// Draw the Bates stamp for one page as a non-destructive overlay
/// (spec F-804/F-805).
///
/// The stamp is ghosted in the live editor so it reads as a preview, and solid
/// in export/print (both paths reuse `draw_canvas`). Nothing is created in the
/// project and the source PDF is never modified; this is purely a painted
/// overlay derived from [`BatesConfig`].
pub(crate) fn draw_bates_overlay(
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
pub(crate) fn draw_minimap(
    painter: &Painter,
    page_bounds: DRect,
    viewport_rect: egui::Rect,
    zoom: f32,
    canvas_size: Vec2,
    origin: Pos2,
    pan: Vec2,
) {
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_selection(painter: &Painter, origin: Pos2, pan: Vec2, zoom: f32, m: &Markup) {
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
