//! Pages sidebar: page thumbnails and click-to-jump navigation (spec F-404).
//!
//! The sidebar only renders when a document is open and has pages; a project
//! with no PDF still shows numbered placeholders so navigation stays available.

use eframe::egui::{self, Color32, FontId, Rect, Sense, StrokeKind, TextureHandle, Vec2};

use crate::data::render_worker::RenderWorker;
use crate::domain::thumb_layout;
use crate::domain::thumb_render::{RenderJob, RenderKey};
use std::collections::HashMap;
use std::path::Path;

/// State owned by the app for the sidebar: uploaded page thumbnails plus the
/// pages whose render already failed, so a broken page is not retried every
/// frame (same poisoning rule the main canvas uses). `pending` holds the keys
/// submitted to the worker and not yet drained, so a render in flight is not
/// re-submitted every frame.
#[derive(Default)]
pub(crate) struct ThumbCache {
    textures: HashMap<String, TextureHandle>,
    failed: std::collections::HashSet<String>,
    pending: std::collections::HashSet<String>,
}

impl ThumbCache {
    /// Absorb one drained worker result for `key_str`: clear the pending
    /// marker, then upload the texture on success or poison the key on
    /// failure (same rule the main canvas uses for its background cache).
    /// Called from the central drain in
    /// [`crate::ui::app::RubricApp::route_render_result`].
    pub(crate) fn absorb_result(
        &mut self,
        key_str: &str,
        pixels: &crate::data::render_worker::ThumbPixels,
        ctx: &egui::Context,
    ) {
        self.pending.remove(key_str);
        if pixels.outcome.is_ok() {
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [pixels.width as usize, pixels.height as usize],
                &pixels.rgba,
            );
            let texture = ctx.load_texture(key_str.to_owned(), image, egui::TextureOptions::LINEAR);
            self.textures.insert(key_str.to_owned(), texture);
        } else {
            self.failed.insert(key_str.to_owned());
        }
    }
}

/// Minimum width of one thumbnail cell, and the gutter between cells.
const MIN_CELL: f64 = 96.0;
const GAP: f64 = 8.0;
/// Width thumbnails are rasterised at. Fixed so a resize re-uses the cache.
/// Also the routing discriminator in
/// [`crate::ui::app::RubricApp::route_render_result`]: drained results at
/// this width are sidebar thumbnails, any other width is a canvas
/// page-background attempt.
pub(crate) const THUMB_WIDTH_PX: u32 = 192;

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
    worker: &RenderWorker,
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
        let per_row = layout.per_row.max(1);

        for row in 0..layout.rows {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = GAP as f32;
                ui.spacing_mut().item_spacing.y = 0.0;
                for col in 0..per_row {
                    let index = row * per_row + col;
                    if index >= page_count {
                        break;
                    }
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(cell_w, layout.cell_h as f32),
                        Sense::click(),
                    );

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
                        let key = RenderKey {
                            pdf_id: path.display().to_string(),
                            page: index,
                            width_px: THUMB_WIDTH_PX,
                        };
                        let key_str = key.to_string();

                        // Check cache first.
                        if let Some(texture) = cache.textures.get(&key_str) {
                            loaded = Some(texture.clone());
                        } else if !cache.failed.contains(&key_str)
                            && !cache.pending.contains(&key_str)
                        {
                            // Enqueue for async render; `pending` stops a
                            // duplicate submit on every frame while the key is
                            // in flight.
                            cache.pending.insert(key_str.clone());
                            worker.submit(RenderJob { key, priority: 0 });
                        }

                        match loaded {
                            Some(texture) => {
                                let uv =
                                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
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
                    }

                    let caption_rect = Rect::from_min_size(
                        thumb_rect.max,
                        Vec2::new(cell_w, layout.caption_h as f32),
                    );
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
            ui.add_space(GAP as f32);
        }
    });
    jumped
}
