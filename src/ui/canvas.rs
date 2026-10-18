//! Canvas: page rendering, tool interaction, drafting and selection.

use eframe::egui::{self, Align2, Color32, FontId, Painter, Pos2, Sense, Ui, Vec2};
use uuid::Uuid;

use crate::domain::{
    angle_degrees, Markup, MarkupGeometry, MarkupKind, Project, Pt, Rect as DRect, ReviewStatus,
    Tool,
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

/// Pick a markup id at page point (frontmost = highest index).
fn pick_markup_id(project: &Project, page_idx: usize, p: Pt, tol: f32) -> Option<Uuid> {
    let doc = project.active()?;
    let mut ids: Vec<Uuid> = doc
        .markups
        .iter()
        .filter(|m| m.page == page_idx && m.hit_test(p, tol))
        .map(|m| m.id)
        .collect();
    ids.reverse();
    ids.first().copied()
}

fn screen_rect(r: DRect, origin: Pos2, pan: Vec2, zoom: f32) -> egui::Rect {
    let min = to_screen(Pt::new(r.x, r.y), pan, zoom, origin);
    let max = to_screen(Pt::new(r.right(), r.bottom()), pan, zoom, origin);
    egui::Rect::from_min_max(min, max)
}

/// Draw a single markup.
fn draw_markup(painter: &Painter, origin: Pos2, pan: Vec2, zoom: f32, m: &Markup) {
    let style = m.style;
    let stroke_c = to_color32(style.stroke);
    let fill_c = to_color32(style.fill.with_alpha(style.fill_mode.alpha(style.fill.a)));
    let stroke_w = (style.width * zoom).max(0.25);
    let stroke_eg = egui::Stroke::new(stroke_w, stroke_c);

    match &m.geometry {
        MarkupGeometry::Bounds(r) => {
            let sr = screen_rect(*r, origin, pan, zoom);
            match m.kind {
                MarkupKind::Highlight => {
                    let s = egui::Stroke::new((style.width * zoom).max(8.0), stroke_c);
                    painter.rect_filled(sr, egui::CornerRadius::ZERO, fill_c);
                    painter.rect_stroke(sr, egui::CornerRadius::ZERO, s, egui::StrokeKind::Inside);
                }
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
        MarkupGeometry::Text { at, text, .. } => {
            let sa = to_screen(*at, pan, zoom, origin);
            painter.text(
                sa,
                Align2::LEFT_TOP,
                text,
                FontId::proportional(11.0 * zoom.max(0.8)),
                stroke_c,
            );
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
            let ang = format!("{:.0}°", angle_degrees(*a, *vertex, *b));
            painter.text(
                sv + Vec2::new(4.0, 4.0),
                Align2::LEFT_TOP,
                ang,
                FontId::proportional(9.0 * zoom.max(0.7)),
                stroke_c,
            );
        }
        MarkupGeometry::Count { at, number } => {
            let sa = to_screen(*at, pan, zoom, origin);
            painter.circle_filled(sa, 10.0 * zoom.max(0.7), Color32::from_rgb(30, 30, 36));
            painter.text(
                sa,
                Align2::CENTER_CENTER,
                number.to_string(),
                FontId::proportional(10.0 * zoom.max(0.7)),
                Color32::WHITE,
            );
        }
    }

    if m.status != ReviewStatus::None {
        let b = screen_rect(m.bounds(), origin, pan, zoom);
        let dot = Pos2::new(b.right() + 4.0, b.top() - 4.0);
        painter.circle_filled(dot, 5.0 * zoom.max(0.7), to_color32(m.status.color()));
    }
}

/// Draw selection handles for a selected markup.
fn draw_selection(painter: &Painter, origin: Pos2, pan: Vec2, zoom: f32, m: &Markup) {
    let b = screen_rect(m.bounds(), origin, pan, zoom);
    let stroke = egui::Stroke::new(1.0, Color32::from_rgb(30, 144, 255));
    let handle = |p: Pos2| painter.circle(p, 4.5 * zoom.max(0.7), Color32::WHITE, stroke);
    handle(b.left_top());
    handle(b.right_top());
    handle(b.left_bottom());
    handle(b.right_bottom());
    handle(b.center());
    painter.rect_stroke(
        b,
        egui::CornerRadius::same(4),
        stroke,
        egui::StrokeKind::Middle,
    );
}

/// Draw and interact with the page canvas. Returns `true` when the caller
/// should snapshot the project into undo history.
pub(crate) fn draw_canvas(
    ui: &mut Ui,
    project: &mut Project,
    ui_state: &mut super::app::UiState,
    draw_state: &mut super::app::DrawState,
    last_page_pos: &mut Option<Pt>,
) -> bool {
    let mut mutated = false;
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

    painter.rect_filled(
        response.rect,
        egui::CornerRadius::ZERO,
        Color32::from_rgb(235, 238, 244),
    );

    let page = match project.active().and_then(|d| d.primary_page()) {
        Some(p) => p,
        None => return false,
    };
    let origin = ui_state.page_origin;
    let pan = ui_state.pan;
    let zoom = ui_state.zoom;

    let page_rect = screen_rect(page.bounds(), origin, pan, zoom);
    painter.rect_filled(page_rect, egui::CornerRadius::same(12), Color32::WHITE);
    painter.rect_stroke(
        page_rect,
        egui::CornerRadius::same(12),
        egui::Stroke::new(1.0, Color32::from_rgb(200, 206, 218)),
        egui::StrokeKind::Middle,
    );

    // Render markups (add order).
    if let Some(doc) = project.active() {
        for m in doc.markups.iter().filter(|m| m.page == page.index) {
            draw_markup(&painter, origin, pan, zoom, m);
        }
    }

    // Render selection.
    if let Some(doc) = project.active() {
        let selected_ids: Vec<Uuid> = ui_state.selected.iter().copied().collect();
        for id in selected_ids {
            if let Some(m) = doc.find_markup(id) {
                draw_selection(&painter, origin, pan, zoom, m);
            }
        }
    }

    // Pointer position in page space.
    let page_pos = response.hover_pos().map(|h| to_page(h, pan, zoom, origin));
    *last_page_pos = page_pos;

    // Draft preview for a rubber-band bounds shape.
    if let (Some(hover), super::app::DrawState::Bounds { start, .. }) = (page_pos, &*draw_state) {
        let r = DRect::from_corners(*start, hover);
        let sr = screen_rect(r, origin, pan, zoom);
        painter.rect_stroke(
            sr,
            egui::CornerRadius::ZERO,
            egui::Stroke::new(1.0, Color32::from_rgb(30, 144, 255)),
            egui::StrokeKind::Middle,
        );
    }

    // Click handling.
    if response.clicked() {
        if let Some(pp) = page_pos {
            if ui_state.active_tool == Tool::Select {
                match pick_markup_id(project, page.index, pp, 6.0) {
                    Some(id) => {
                        ui_state.selected.clear();
                        ui_state.selected.insert(id);
                        ui_state.status = format!("Selected {}", id);
                    }
                    None => ui_state.selected.clear(),
                }
            } else if let Some(kind) = ui_state.active_tool.to_kind() {
                let author = project.settings.author.clone();
                let inv = 1.0 / zoom.max(0.1);
                let geom = match kind {
                    MarkupKind::Rectangle
                    | MarkupKind::Ellipse
                    | MarkupKind::Cloud
                    | MarkupKind::Text
                    | MarkupKind::Stamp
                    | MarkupKind::Note
                    | MarkupKind::Callout
                    | MarkupKind::Highlight => {
                        MarkupGeometry::Bounds(DRect::new(pp.x, pp.y, 40.0 * inv, 24.0 * inv))
                    }
                    MarkupKind::Line
                    | MarkupKind::Arrow
                    | MarkupKind::MeasureLength
                    | MarkupKind::MeasureDiameter
                    | MarkupKind::MeasureRadius => MarkupGeometry::Segment { a: pp, b: pp },
                    MarkupKind::MeasureCount => MarkupGeometry::Count { at: pp, number: 1 },
                    _ => MarkupGeometry::Bounds(DRect::new(pp.x, pp.y, 40.0 * inv, 24.0 * inv)),
                };
                let m = Markup::new(page.index, kind, geom, author);
                if let Some(doc) = project.active_mut() {
                    doc.add_markup(m);
                }
                ui_state.status = format!("Added {}", kind.label());
                mutated = true;
            }
        }
    }

    // Drag to move the selection.
    if response.dragged() {
        let delta = response.drag_delta();
        if delta != Vec2::ZERO {
            let dx = delta.x / zoom;
            let dy = delta.y / zoom;
            let sel: Vec<Uuid> = ui_state.selected.iter().copied().collect();
            if let Some(doc) = project.active_mut() {
                for id in sel {
                    if let Some(m) = doc.find_markup_mut(id) {
                        m.geometry.translate(dx, dy);
                        m.touch();
                    }
                }
            }
            mutated = true;
        }
    }

    mutated
}
