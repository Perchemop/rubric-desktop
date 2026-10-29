//! Canvas: page rendering, tool interaction, drafting and selection.

use eframe::egui::{self, Align2, Color32, FontId, Painter, Pos2, Sense, Ui, Vec2};
use uuid::Uuid;

use crate::domain::{
    angle_degrees, ruler_ticks, Grid, Markup, MarkupGeometry, MarkupKind, Project, Pt,
    Rect as DRect, ReviewStatus, Tool,
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

/// Apply the active stamp preset to a freshly created stamp markup. Non-stamp
/// markups pass through untouched.
fn finish_markup(m: Markup, ui_state: &super::app::UiState) -> Markup {
    if m.kind == MarkupKind::Stamp {
        let preset = ui_state.active_stamp;
        let mut m = m.with_stamp(preset);
        m.style = m.style.with_stroke(preset.color());
        m
    } else {
        m
    }
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
fn draw_rulers(
    painter: &Painter,
    ui: &Ui,
    page_rect: egui::Rect,
    bounds: DRect,
    pan: Vec2,
    zoom: f32,
    origin: Pos2,
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
            painter.text(
                Pos2::new(x + 2.0, top + 2.0),
                Align2::LEFT_TOP,
                format!("{v:.0}"),
                font.clone(),
                text_c,
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
            painter.text(
                Pos2::new(left + 2.0, y + 2.0),
                Align2::LEFT_TOP,
                format!("{v:.0}"),
                font.clone(),
                text_c,
            );
        }
    }
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
                let galley = painter.layout(text.clone(), font, stroke_c, *wrap * zoom);
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
    ui_state.text_edit = None;
    changed && editable
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
    let backdrop = ui.visuals().extreme_bg_color;
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

    painter.rect_filled(response.rect, egui::CornerRadius::ZERO, backdrop);

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
        let target_width = ((page.width * 2.0).round() as u32).max(64);
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
    painter.rect_stroke(
        page_rect,
        egui::CornerRadius::same(12),
        egui::Stroke::new(1.0, Color32::from_rgb(200, 206, 218)),
        egui::StrokeKind::Middle,
    );

    // Canvas grid (F-405) and rulers (F-406). Both are guides: they are painted
    // before the markups so they always sit under the content.
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
    );

    // Render markups in paint order (layer-major, then vector order).
    if let Some(doc) = project.active() {
        for m in doc.markups_in_paint_order(page.index) {
            draw_markup(&painter, origin, pan, zoom, m);
        }
    }

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
        if let Some(doc) = project.active() {
            let unit = page.scale.unit;
            for m in doc.markups_in_paint_order(page.index) {
                if let Some(meas) = m.measurement {
                    let label = meas.label_with(unit, project.settings.precision);
                    let anchor_page = measure_anchor(m);
                    let anchor = to_screen(anchor_page, pan, zoom, origin);
                    let font = FontId::proportional(12.0);
                    let galley = painter.layout_no_wrap(label, font, Color32::WHITE);
                    let text_rect = galley.rect.translate(Vec2::new(anchor.x, anchor.y));
                    let bg = text_rect.expand(4.0);
                    painter.rect_filled(bg, 4.0, Color32::from_black_alpha(180));
                    painter.galley(anchor, galley, Color32::WHITE);
                }
            }
        }
    }

    // In-canvas text editor overlay (Text/Callout markups). Rendered above all
    // markups/labels; commits on focus loss.
    let mut commit_edit = false;
    if ui_state.text_edit.is_some() {
        let edit_id = ui_state.text_edit.as_ref().unwrap().id;
        let sr = project
            .active()
            .and_then(|doc| doc.find_markup(edit_id))
            .map(|m| match &m.geometry {
                MarkupGeometry::Text { at, wrap, text } => {
                    let lines = (text.lines().count().max(1)) as f32;
                    screen_rect(
                        DRect::new(at.x, at.y, *wrap, lines * 16.0),
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
            let _ = ui.put(
                next_row(60.0),
                egui::TextEdit::multiline(&mut ui_state.note_draft),
            );
            let add_resp = ui.put(next_row(20.0), egui::Button::new("Add comment"));
            let close_resp = ui.put(next_row(20.0), egui::Button::new("Close"));
            if add_resp.clicked() && !ui_state.note_draft.trim().is_empty() {
                if let Some(doc) = project.active_mut() {
                    if let Some(m) = doc.find_markup_mut(note_id) {
                        if m.add_comment(
                            project.settings.author.clone(),
                            ui_state.note_draft.clone(),
                        )
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
    *last_page_pos = page_pos;

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
                    let new_zoom = (zoom * factor).clamp(0.1, 16.0);
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
    if ui_state.text_edit.is_none() && response.clicked_by(egui::PointerButton::Primary) && !panning
    {
        if let Some(pp) = page_pos {
            if ui_state.active_tool == Tool::Select {
                // Hit-testing ignores the grid (F-405).
                match pick_markup_id(project, page.index, raw_pos.unwrap_or(pp), 6.0) {
                    Some(id) => {
                        ui_state.selected.clear();
                        ui_state.selected.insert(id);
                        ui_state.status = format!("Selected {}", id);
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
                        if response.double_clicked() {
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
                        ui_state.selected.clear();
                        ui_state.note_popover = None;
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
                        | MarkupKind::Note
                        | MarkupKind::Highlight => {
                            MarkupGeometry::Bounds(DRect::new(pp.x, pp.y, 40.0 * inv, 24.0 * inv))
                        }
                        MarkupKind::MeasureCount => MarkupGeometry::Count { at: pp, number: 1 },
                        MarkupKind::Ink => MarkupGeometry::Ink {
                            strokes: vec![vec![pp]],
                        },
                        // Remaining kinds (strikeout, underline, squiggly) are
                        // bounds-like and default to a small rect.
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
        && !matches!(ui_state.active_tool, Tool::Select)
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
                        let m =
                            finish_markup(Markup::new(page.index, kind, geom, author), ui_state);
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
                    super::app::DrawState::Segment { start, end, kind } => {
                        let geom = MarkupGeometry::Segment { a: start, b: end };
                        let m =
                            finish_markup(Markup::new(page.index, kind, geom, author), ui_state);
                        if let Some(doc) = project.active_mut() {
                            doc.add_markup(m);
                        }
                        ui_state.status = format!("Added {}", kind.label());
                        mutated = true;
                    }
                    super::app::DrawState::Ink {
                        mut strokes,
                        current,
                    } => {
                        if !current.is_empty() {
                            strokes.push(current);
                        }
                        if !strokes.is_empty() {
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

    // If an edit just started, silently drop any in-progress move drag so no
    // stray snapshot is pushed for this gesture.
    if ui_state.text_edit.is_some() {
        ui_state.move_selection = false;
        ui_state.move_changed = false;
    }
    // Drag to move the selection (when not drawing/panning).
    if !panning && matches!(ui_state.active_tool, Tool::Select) && ui_state.text_edit.is_none() {
        if response.drag_started() {
            // Only a drag that starts on a selected, editable markup moves it;
            // dragging from empty space must not touch the selection or history.
            // Hit-testing ignores the grid (F-405): a drag must grab the markup
            // the pointer is actually over, grid-aligned or not.
            let picked = raw_pos.and_then(|pp| pick_markup_id(project, page.index, pp, 6.0));
            let can_move = picked
                .map(|id| {
                    let editable = project
                        .active()
                        .and_then(|d| d.find_markup(id).map(|m| d.is_markup_editable(m)))
                        .unwrap_or(false);
                    ui_state.selected.contains(&id) && editable
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

    mutated
}
