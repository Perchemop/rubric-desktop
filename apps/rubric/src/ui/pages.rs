//! Pages sidebar: page thumbnails and click-to-jump navigation (spec F-404).
//!
//! The sidebar only renders when a document is open and has pages; a project
//! with no PDF still shows numbered placeholders so navigation stays available.

use eframe::egui::{self, Color32, FontId, Rect, Sense, StrokeKind, TextureHandle, Vec2};

use crate::data::pdf;
use crate::domain::thumb_layout;
use std::collections::HashMap;
use std::path::Path;

/// State owned by the app for the sidebar: uploaded page thumbnails plus the
/// pages whose render already failed, so a broken page is not retried every
/// frame (same poisoning rule the main canvas uses).
#[derive(Default)]
pub(crate) struct ThumbCache {
    textures: HashMap<String, TextureHandle>,
    failed: std::collections::HashSet<String>,
}

/// Minimum width of one thumbnail cell, and the gutter between cells.
const MIN_CELL: f64 = 96.0;
const GAP: f64 = 8.0;
/// Width thumbnails are rasterised at. Fixed so a resize re-uses the cache.
const THUMB_WIDTH_PX: u32 = 192;

/// Draw the sidebar and return the page the user jumped to, if any.
///
/// Returns `None` when nothing was clicked; the caller owns the document-level
/// side effects (text-edit commit, undo snapshot) so this module stays a view.
pub(crate) fn draw_pages(
    ui: &mut egui::Ui,
    page_count: usize,
    current_page: usize,
    source_pdf: Option<&Path>,
    cache: &mut ThumbCache,
) -> Option<usize> {
    let mut jumped = None;

    ui.horizontal(|ui| {
        ui.heading("Pages");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.small(format!("{} page(s)", page_count))
        });
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        let available = ui.available_width() as f64;
        let layout = thumb_layout(available, page_count, MIN_CELL, GAP);
        let cell_w = layout.cell_w as f32;
        let thumb_h = (layout.cell_h - layout.caption_h) as f32;

        for index in 0..page_count {
            // Row breaks are the scroll area's job; each cell is one widget in
            // a horizontal strip that wraps because the width was pre-computed
            // to fit `per_row` cells.
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(cell_w, layout.cell_h as f32), Sense::click());

            let thumb_rect = Rect::from_min_size(rect.min, Vec2::new(cell_w, thumb_h));
            let is_current = index == current_page;
            let stroke = if is_current {
                egui::Stroke::new(2.0, ui.visuals().selection.bg_fill)
            } else {
                egui::Stroke::new(1.0, ui.visuals().widgets.inactive.bg_fill)
            };
            ui.painter().rect(
                thumb_rect,
                3.0,
                ui.visuals().extreme_bg_color,
                stroke,
                StrokeKind::Middle,
            );

            // Rendered preview when the project is backed by a PDF; otherwise a
            // numbered placeholder, which is honest about there being no image.
            let mut loaded = None;
            if let Some(path) = source_pdf {
                // Rendering a *document* page, not the current one: the id
                // encodes that, so the main canvas texture never reuses it.
                let key = format!("{}:{index}:{THUMB_WIDTH_PX}", path.display());
                loaded = load_thumb(ui, path, index, &key, cache);
            }
            match loaded {
                Some(texture) => {
                    let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                    ui.painter()
                        .image(texture.id(), thumb_rect, uv, Color32::WHITE);
                }
                None => {
                    let centre = thumb_rect.center();
                    ui.painter().text(
                        centre,
                        egui::Align2::CENTER_CENTER,
                        format!("{}", index + 1),
                        FontId::proportional(thumb_h * 0.45),
                        ui.visuals().text_color(),
                    );
                }
            }

            let caption_rect =
                Rect::from_min_size(thumb_rect.max, Vec2::new(cell_w, layout.caption_h as f32));
            let caption_colour = if is_current {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().text_color()
            };
            ui.painter().text(
                caption_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{}", index + 1),
                FontId::proportional(11.0),
                caption_colour,
            );
            if response.clicked() && !is_current {
                jumped = Some(index);
                ui.ctx().request_repaint();
            }
        }
    });

    jumped
}

/// Upload (or re-use) the texture for one page thumbnail.
///
/// Rasterisation is delegated to `data::pdf`, which keeps a shared on-disk
/// cache, so a thumbnail that the main canvas already rendered costs nothing.
fn load_thumb(
    ui: &egui::Ui,
    path: &Path,
    page_index: usize,
    key: &str,
    cache: &mut ThumbCache,
) -> Option<TextureHandle> {
    if let Some(texture) = cache.textures.get(key) {
        return Some(texture.clone());
    }
    if cache.failed.contains(key) {
        return None;
    }
    let texture = match pdf::render_page_to_cache(path, page_index, THUMB_WIDTH_PX)
        .map_err(|e| e.to_string())
        .and_then(|cache_path| std::fs::read(&cache_path).map_err(|e| e.to_string()))
        .and_then(|bytes| {
            image::load_from_memory(&bytes)
                .map(|img| img.to_rgba8())
                .map_err(|e| e.to_string())
        }) {
        Ok(rgba) => {
            let (w, h) = rgba.dimensions();
            let image =
                egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
            let t = ui
                .ctx()
                .load_texture(key.to_string(), image, egui::TextureOptions::LINEAR);
            Some(t)
        }
        Err(_) => None,
    };
    match texture {
        Some(t) => {
            cache.textures.insert(key.to_string(), t.clone());
            Some(t)
        }
        None => {
            cache.failed.insert(key.to_string());
            None
        }
    }
}
