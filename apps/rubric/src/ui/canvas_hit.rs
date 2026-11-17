//! Hit testing for the canvas: which markup, which handle, which vertex.
//!
//! All functions take screen-space coordinates and convert internally to page
//! space for domain hit tests.

use eframe::egui::{Pos2, Vec2};

use crate::domain::geom::Pt;
use crate::domain::{Corner, Markup, Project};
use crate::ui::canvas_geom::to_screen_rotated;
use uuid::Uuid;

/// Pick a markup id at page point (frontmost = last in paint order).
pub(crate) fn pick_markup_id(project: &Project, page_idx: usize, p: Pt, tol: f32) -> Option<Uuid> {
    let doc = project.active()?;
    doc.markups_in_paint_order(page_idx)
        .into_iter()
        .rev()
        .find(|m| m.hit_test(p, tol))
        .map(|m| m.id)
}

/// Which selection-handle corner, if any, the screen point `pp` is over.
///
/// The hit area is slightly larger than the drawn handle so grabbing a corner
/// does not require pixel precision (v0.4 gap #5: resize grips).
pub(crate) fn handle_at(b: egui::Rect, pp: Pos2, zoom: f32) -> Option<Corner> {
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
///
/// `page_w`, `page_h`, and `rotation` are the page's dimensions and quarter-turn
/// rotation; passed through so hit-testing honours page rotation. Rotation 0
/// behaves identically to the identity mapping.
pub(crate) fn point_handle_at(
    m: &Markup,
    pp: Pos2,
    origin: Pos2,
    pan: Vec2,
    zoom: f32,
    page_w: f32,
    page_h: f32,
    rotation: i32,
) -> Option<usize> {
    let radius = 8.0_f32.max(4.5 * zoom.max(0.7));
    m.draggable_points()
        .iter()
        .position(|p| {
            let screen_p = to_screen_rotated(*p, pan, zoom, origin, page_w, page_h, rotation);
            (screen_p - pp).length() <= radius
        })
}
