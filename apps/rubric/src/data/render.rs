//! Headless rendering of a canvas page to a PNG image (spec F-505).
//!
//! The canvas is drawn once inside a headless `egui::Context` frame; the
//! resulting shapes are tessellated and rasterised on the CPU, so the exported
//! image matches the page content (page background + markup overlays,
//! including text labels and stamps) without any GPU or display. The export is
//! view-independent: it rasterises the page at a fixed [`EXPORT_DPI`], not the
//! user's current pan/zoom editor view.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use eframe::egui::{
    self, Color32, ColorImage, Context, Pos2, Rect, TextureHandle, TextureId, Vec2,
};
use image::ImageFormat;

use crate::data::error::DataError;
use crate::data::pdf;
use crate::domain::geom::Pt;
use crate::domain::Project;
use crate::ui::app::{DrawState, UiState};
use crate::ui::canvas;

/// Dots per inch used by page-image export and print rasterisation.
///
/// 150 DPI keeps a Letter/A4 page at a comfortable few-megapixel surface while
/// staying well inside the engines' `u16` pixmap limits. Exports must not
/// depend on the live editor view, so [`render_page_to_png_bytes`] pins this
/// zoom and zeroes pan/page origin on its `UiState` clone: the page is drawn
/// from the origin, filling the off-screen surface edge-to-edge.
const EXPORT_DPI: f32 = 150.0;

/// Rasterise a slice of tessellated egui primitives into a colour image.
///
/// Pure CPU work: 2×2 supersampled barycentric triangle rasterisation with
/// per-vertex colour interpolation, nearest-neighbour texture sampling
/// (multiplied by the vertex colour / tint) and src-over alpha blending.
/// Each primitive is scissor-clipped to its `clip_rect`. An empty input (or
/// an all-transparent result) yields a fully transparent image.
pub fn rasterize_primitives(
    clipped: &[egui::ClippedPrimitive],
    size: [usize; 2],
    texture_lookup: impl Fn(TextureId) -> Option<ColorImage>,
) -> ColorImage {
    let mut img = ColorImage::new(size, vec![Color32::TRANSPARENT; size[0] * size[1]]);
    if size[0] == 0 || size[1] == 0 {
        return img;
    }
    for cp in clipped {
        let clip = cp.clip_rect;
        match &cp.primitive {
            egui::epaint::Primitive::Mesh(mesh) => {
                rasterize_mesh(&mut img, mesh, clip, &texture_lookup);
            }
            egui::epaint::Primitive::Callback(_) => {}
        }
    }
    img
}

fn rasterize_mesh(
    img: &mut ColorImage,
    mesh: &egui::Mesh,
    clip: Rect,
    texture_lookup: &impl Fn(TextureId) -> Option<ColorImage>,
) {
    // NB: the font atlas is `TextureId::default()`, so every mesh texture is
    // looked up; a missing texture falls back to the plain vertex colour.
    let tex = texture_lookup(mesh.texture_id);
    let w = img.size[0] as i32;
    let h = img.size[1] as i32;
    let clip_min_x = clip.min.x.floor() as i32;
    let clip_min_y = clip.min.y.floor() as i32;
    let clip_max_x = clip.max.x.ceil() as i32;
    let clip_max_y = clip.max.y.ceil() as i32;

    let tri_count = mesh.indices.len() / 3;
    for t in 0..tri_count {
        let i0 = mesh.indices[t * 3] as usize;
        let i1 = mesh.indices[t * 3 + 1] as usize;
        let i2 = mesh.indices[t * 3 + 2] as usize;
        if i0 >= mesh.vertices.len() || i1 >= mesh.vertices.len() || i2 >= mesh.vertices.len() {
            continue;
        }
        let v0 = &mesh.vertices[i0];
        let v1 = &mesh.vertices[i1];
        let v2 = &mesh.vertices[i2];

        // Bounding box of the triangle, clipped to the scissor + image.
        let min_x = (v0.pos.x.min(v1.pos.x).min(v2.pos.x).floor() as i32)
            .max(0)
            .max(clip_min_x);
        let min_y = (v0.pos.y.min(v1.pos.y).min(v2.pos.y).floor() as i32)
            .max(0)
            .max(clip_min_y);
        let max_x = (v0.pos.x.max(v1.pos.x).max(v2.pos.x).ceil() as i32)
            .min(w - 1)
            .min(clip_max_x);
        let max_y = (v0.pos.y.max(v1.pos.y).max(v2.pos.y).ceil() as i32)
            .min(h - 1)
            .min(clip_max_y);
        if min_x > max_x || min_y > max_y {
            continue;
        }

        // Precompute edge-function denominator.
        let ax = v0.pos.x;
        let ay = v0.pos.y;
        let bx = v1.pos.x;
        let by = v1.pos.y;
        let cx = v2.pos.x;
        let cy = v2.pos.y;
        let denom = (by - cy) * (ax - cx) + (cx - bx) * (ay - cy);
        if denom.abs() < 1e-12 {
            continue;
        }
        let inv_denom = 1.0 / denom;

        for py in min_y..=max_y {
            for px in min_x..=max_x {
                // 2x2 supersampling for anti-aliasing.
                let mut accum_r = 0.0f32;
                let mut accum_g = 0.0f32;
                let mut accum_b = 0.0f32;
                let mut accum_a = 0.0f32;
                let mut samples = 0u32;
                for sy in [0.25f32, 0.75f32] {
                    for sx in [0.25f32, 0.75f32] {
                        let x = px as f32 + sx;
                        let y = py as f32 + sy;
                        // Barycentric coordinates.
                        let w0 = ((by - cy) * (x - cx) + (cx - bx) * (y - cy)) * inv_denom;
                        let w1 = ((cy - ay) * (x - cx) + (ax - cx) * (y - cy)) * inv_denom;
                        let w2 = 1.0 - w0 - w1;
                        if w0 < -1e-4 || w1 < -1e-4 || w2 < -1e-4 {
                            continue;
                        }
                        // Interpolate vertex colour.
                        let r = v0.color.r() as f32 * w0
                            + v1.color.r() as f32 * w1
                            + v2.color.r() as f32 * w2;
                        let g = v0.color.g() as f32 * w0
                            + v1.color.g() as f32 * w1
                            + v2.color.g() as f32 * w2;
                        let b = v0.color.b() as f32 * w0
                            + v1.color.b() as f32 * w1
                            + v2.color.b() as f32 * w2;
                        let a = v0.color.a() as f32 * w0
                            + v1.color.a() as f32 * w1
                            + v2.color.a() as f32 * w2;

                        // UV interpolation + texture sampling.
                        let (mut tr, mut tg, mut tb, mut ta) = (r, g, b, a);
                        if let Some(tex_img) = &tex {
                            let u = v0.uv.x * w0 + v1.uv.x * w1 + v2.uv.x * w2;
                            let v = v0.uv.y * w0 + v1.uv.y * w1 + v2.uv.y * w2;
                            let u = u.clamp(0.0, 1.0);
                            let v = v.clamp(0.0, 1.0);
                            let tx = ((u * (tex_img.size[0] as f32 - 1.0)).round() as usize)
                                .min(tex_img.size[0].saturating_sub(1));
                            let ty = ((v * (tex_img.size[1] as f32 - 1.0)).round() as usize)
                                .min(tex_img.size[1].saturating_sub(1));
                            let tp = tex_img.pixels[ty * tex_img.size[0] + tx];
                            // Multiply tint (vertex colour) by texture.
                            tr = tr * tp.r() as f32 / 255.0;
                            tg = tg * tp.g() as f32 / 255.0;
                            tb = tb * tp.b() as f32 / 255.0;
                            ta = ta * tp.a() as f32 / 255.0;
                        }

                        accum_r += tr;
                        accum_g += tg;
                        accum_b += tb;
                        accum_a += ta;
                        samples += 1;
                    }
                }
                if samples == 0 {
                    continue;
                }
                let inv = 1.0 / samples as f32;
                let src_r = (accum_r * inv).clamp(0.0, 255.0) as u8;
                let src_g = (accum_g * inv).clamp(0.0, 255.0) as u8;
                let src_b = (accum_b * inv).clamp(0.0, 255.0) as u8;
                let src_a = (accum_a * inv).clamp(0.0, 255.0) as u8;

                // Src-over alpha blend onto destination.
                let dst = &mut img.pixels[py as usize * img.size[0] + px as usize];
                let sa = src_a as f32 / 255.0;
                let da = dst.a() as f32 / 255.0;
                let out_a = sa + da * (1.0 - sa);
                if out_a <= 0.0 {
                    *dst = Color32::TRANSPARENT;
                    continue;
                }
                let out_r = (src_r as f32 * sa + dst.r() as f32 * da * (1.0 - sa)) / out_a;
                let out_g = (src_g as f32 * sa + dst.g() as f32 * da * (1.0 - sa)) / out_a;
                let out_b = (src_b as f32 * sa + dst.b() as f32 * da * (1.0 - sa)) / out_a;
                *dst = Color32::from_rgba_unmultiplied(
                    out_r.round() as u8,
                    out_g.round() as u8,
                    out_b.round() as u8,
                    (out_a * 255.0).round() as u8,
                );
            }
        }
    }
}

/// Load (and cache) the PDF page-background texture used by an export or print.
///
/// Returns an error when the page background cannot be produced — a missing
/// PDF, an engine failure, or an undecodable PNG — so the caller reports the
/// problem instead of silently emitting a blank white page (audit M4).
fn load_page_background(
    ctx: &Context,
    pdf_path: impl AsRef<Path>,
    page_index: usize,
    page_width: f32,
    page_textures: &mut HashMap<String, TextureHandle>,
) -> Result<(), DataError> {
    let target_width = ((page_width * 2.0).round() as u32).max(64);
    let cache_path =
        pdf::render_page_to_cache(pdf_path, page_index, target_width).map_err(|e| {
            DataError::Pdf(format!(
                "could not render the PDF page background for export: {e}"
            ))
        })?;

    // Reuse an already-loaded texture for this render; otherwise read, decode
    // and register it.
    let key = cache_path.to_string_lossy().to_string();
    if let std::collections::hash_map::Entry::Vacant(vacant) = page_textures.entry(key.clone()) {
        let bytes = std::fs::read(&cache_path).map_err(DataError::Io)?;
        let img = image::load_from_memory(&bytes).map_err(|e| {
            DataError::Pdf(format!(
                "could not decode the rendered page background {}: {e}",
                cache_path.display()
            ))
        })?;
        let rgba = img.to_rgba8();
        let color_image = ColorImage::from_rgba_unmultiplied(
            [rgba.width() as usize, rgba.height() as usize],
            rgba.as_raw(),
        );
        let handle = ctx.load_texture(key.clone(), color_image, egui::TextureOptions::LINEAR);
        vacant.insert(handle);
    }

    Ok(())
}

/// Render the currently displayed page (+ markup overlays) to PNG bytes.
pub(crate) fn render_page_to_png_bytes(
    project: &mut Project,
    ui_state: &UiState,
    draw_state: &DrawState,
    page_textures: &mut HashMap<String, TextureHandle>,
    failed_page_textures: &mut HashSet<String>,
    bg_error: &mut Option<String>,
    last_page_pos: &mut Option<Pt>,
) -> Result<Vec<u8>, DataError> {
    let doc = project
        .active()
        .ok_or_else(|| DataError::NotFound("no document open".into()))?;
    let page_count = doc.page_count();
    if page_count == 0 || ui_state.current_page >= page_count {
        return Err(DataError::NotFound("no page to export".into()));
    }
    let page = doc
        .pages
        .get(ui_state.current_page)
        .cloned()
        .ok_or_else(|| DataError::NotFound("page out of range".into()))?;

    // We need an egui context to load textures and run the frame. Create it first,
    // then handle the page background above is re-done inside using ctx.
    let ctx = Context::default();
    let pixels_per_point = 1.0_f32;

    // Page background texture (loaded now that we have a ctx). A failure here
    // is reported instead of silently exporting a blank page (audit M4).
    if let Some(pdf_path) = &project.source_pdf {
        load_page_background(
            &ctx,
            pdf_path,
            ui_state.current_page,
            page.width,
            page_textures,
        )?;
    }

    // Export/print must rasterise the page itself, not the editor view: the
    // live `UiState` carries the user's zoom and pan, and the canvas draws the
    // page at `page_origin` with chrome around it, so an export built from the
    // live state would be a pan/zoom-dependent screen-DPI screenshot that can
    // even crop the page. Pin an explicit, view-independent transform instead:
    // the page is drawn at [`EXPORT_DPI`] from the origin, filling the
    // off-screen surface edge-to-edge.
    //
    // Suppressing the grid/rulers/backdrop inside `draw_canvas` is a
    // canvas-side follow-up (it needs a flag owned by the canvas module).
    let zoom = EXPORT_DPI / 72.0;
    let w = (page.width * zoom).round().max(1.0);
    let h = (page.height * zoom).round().max(1.0);
    let size = [w as usize, h as usize];

    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h))),
        ..Default::default()
    };

    let mut ui_state = ui_state.clone();
    ui_state.zoom = zoom;
    ui_state.pan = Vec2::ZERO;
    ui_state.page_origin = Pos2::ZERO;
    let mut draw_state = draw_state.clone();
    let full_output = ctx.run_ui(raw_input, |ui| {
        egui::CentralPanel::default().show(ui, |ui| {
            let _ = canvas::draw_canvas(
                ui,
                project,
                &mut ui_state,
                &mut draw_state,
                last_page_pos,
                page_textures,
                failed_page_textures,
                bg_error,
                true, // is_export: suppress chrome (grid, rulers, backdrop)
            );
        });
    });

    let clipped = ctx.tessellate(full_output.shapes, pixels_per_point);

    // Collect texture images produced during the pass (page background + font atlas).
    let mut texture_images: HashMap<TextureId, ColorImage> = HashMap::new();
    for (id, deltas) in &full_output.textures_delta.set {
        for delta in deltas {
            let egui::epaint::ImageData::Color(arc) = &delta.image;
            match delta.pos {
                None => {
                    texture_images.insert(*id, arc.as_ref().clone());
                }
                Some(pos) => {
                    // Partial update (e.g. newly rasterised glyphs): copy the
                    // changed region onto the image we already hold.
                    if let Some(base) = texture_images.get_mut(id) {
                        let src = arc.as_ref();
                        for y in 0..src.size[1] {
                            for x in 0..src.size[0] {
                                let dx = pos[0] + x;
                                let dy = pos[1] + y;
                                if dx < base.size[0] && dy < base.size[1] {
                                    base.pixels[dy * base.size[0] + dx] =
                                        src.pixels[y * src.size[0] + x];
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let texture_lookup = |id: TextureId| -> Option<ColorImage> { texture_images.get(&id).cloned() };
    let img = rasterize_primitives(&clipped, size, texture_lookup);

    // Encode PNG.
    let mut rgba = image::RgbaImage::new(img.size[0] as u32, img.size[1] as u32);
    for (i, px) in img.pixels.iter().enumerate() {
        let x = (i % img.size[0]) as u32;
        let y = (i / img.size[0]) as u32;
        rgba.put_pixel(x, y, image::Rgba([px.r(), px.g(), px.b(), px.a()]));
    }
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    rgba.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| DataError::Io(std::io::Error::other(e.to_string())))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterize_empty_is_transparent() {
        let img = rasterize_primitives(&[], [4, 4], |_| None);
        assert!(img.pixels.iter().all(|p| p.a() == 0));
    }

    #[test]
    fn rasterize_single_red_triangle() {
        let mut mesh = egui::Mesh::default();
        mesh.vertices.push(egui::epaint::Vertex {
            pos: Pos2::new(1.0, 1.0),
            uv: egui::Pos2::new(0.0, 0.0),
            color: Color32::RED,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: Pos2::new(10.0, 1.0),
            uv: egui::Pos2::new(0.0, 0.0),
            color: Color32::RED,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: Pos2::new(1.0, 10.0),
            uv: egui::Pos2::new(0.0, 0.0),
            color: Color32::RED,
        });
        mesh.indices = vec![0, 1, 2];
        let cp = egui::ClippedPrimitive {
            clip_rect: Rect::from_min_size(Pos2::ZERO, Vec2::new(12.0, 12.0)),
            primitive: egui::epaint::Primitive::Mesh(mesh),
        };
        let img = rasterize_primitives(&[cp], [12, 12], |_| None);
        // centre-ish pixel of triangle should be red
        assert_eq!(img.pixels[1 * 12 + 1], Color32::RED);
        // far corner transparent
        assert_eq!(img.pixels[11 * 12 + 11], Color32::TRANSPARENT);
    }

    #[test]
    fn rasterize_respects_clip_rect() {
        let mut mesh = egui::Mesh::default();
        mesh.vertices.push(egui::epaint::Vertex {
            pos: Pos2::new(1.0, 1.0),
            uv: egui::Pos2::new(0.0, 0.0),
            color: Color32::RED,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: Pos2::new(10.0, 1.0),
            uv: egui::Pos2::new(0.0, 0.0),
            color: Color32::RED,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: Pos2::new(1.0, 10.0),
            uv: egui::Pos2::new(0.0, 0.0),
            color: Color32::RED,
        });
        mesh.indices = vec![0, 1, 2];
        // clip rect excludes the triangle entirely
        let cp = egui::ClippedPrimitive {
            clip_rect: Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(10.0, 10.0)),
            primitive: egui::epaint::Primitive::Mesh(mesh),
        };
        let img = rasterize_primitives(&[cp], [12, 12], |_| None);
        assert!(img.pixels.iter().all(|p| p.a() == 0));
    }

    #[test]
    fn rasterize_textured_triangle_samples_texture() {
        // 2x2 texture: top-left red, others blue
        let mut tex = ColorImage::new([2, 2], vec![Color32::BLUE; 4]);
        tex.pixels[0] = Color32::RED;
        let tex_clone = tex.clone();
        let lookup = move |_id: TextureId| Some(tex_clone.clone());

        let mut mesh = egui::Mesh::default();
        // A user texture id is constructible, so the sampling path is exercised.
        mesh.texture_id = TextureId::User(7);
        mesh.vertices.push(egui::epaint::Vertex {
            pos: Pos2::new(1.0, 1.0),
            uv: egui::Pos2::new(0.0, 0.0),
            color: Color32::WHITE,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: Pos2::new(10.0, 1.0),
            uv: egui::Pos2::new(1.0, 0.0),
            color: Color32::WHITE,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: Pos2::new(1.0, 10.0),
            uv: egui::Pos2::new(0.0, 1.0),
            color: Color32::WHITE,
        });
        mesh.indices = vec![0, 1, 2];
        let cp = egui::ClippedPrimitive {
            clip_rect: Rect::from_min_size(Pos2::ZERO, Vec2::new(12.0, 12.0)),
            primitive: egui::epaint::Primitive::Mesh(mesh),
        };
        let img = rasterize_primitives(&[cp], [12, 12], lookup);
        // The sampled texture colour must show through the white vertex tint.
        let centre = img.pixels[1 * 12 + 1];
        assert!(
            centre == Color32::RED || centre == Color32::BLUE,
            "expected a texture-sampled colour, got {centre:?}"
        );
    }

    /// Audit M4: a page background that cannot be produced must surface as an
    /// error, not a silently blank white export/print.
    #[test]
    fn export_background_failure_is_reported() {
        let ctx = Context::default();
        let mut textures: HashMap<String, TextureHandle> = HashMap::new();
        let missing = std::env::temp_dir().join(format!(
            "rubric_missing_background_{}_{:?}.pdf",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_file(&missing);

        let err = load_page_background(&ctx, &missing, 0, 612.0, &mut textures)
            .expect_err("a missing PDF must fail the export background");
        assert!(matches!(err, DataError::Pdf(_)), "got {err:?}");
    }
}
