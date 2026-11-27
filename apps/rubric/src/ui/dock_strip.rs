//! Dock strips (left/right/bottom) with tab bars, split dividers, recursive splits.

use crate::domain::workspace::{PanelId, Region};
use crate::ui::panel_manager::PanelManager;
use egui::{Ui, Vec2};

/// Minimum panel sizes (NFR-08).
pub const MIN_PANEL_WIDTH: f32 = 200.0;
pub const MIN_PANEL_HEIGHT: f32 = 150.0;

/// Render a vertical dock strip (left or right).
pub fn render_vertical_strip(
    ui: &mut Ui,
    manager: &mut PanelManager,
    region: Region,
    _ctx: &egui::Context,
) {
    let panels: Vec<PanelId> = manager
        .workspace
        .open
        .iter()
        .filter(|p| manager.workspace.region.get(p).cloned() == Some(region))
        .cloned()
        .collect();
        .collect();

    if panels.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        // Tab bar only when >1 panel.
        if panels.len() > 1 {
            ui.horizontal(|ui| {
                for p in &panels {
                    let label = p.to_string();
                    let _ = ui.selectable_label(false, label);
                }
            });
            ui.separator();
        }
        for p in &panels {
            ui.group(|ui| {
                ui.label(p.to_string());
                let _ = (MIN_PANEL_WIDTH, MIN_PANEL_HEIGHT);
            });
        }
        let _ = (manager, region);
    });
}

/// Render the horizontal bottom strip.
pub fn render_bottom_strip(ui: &mut Ui, manager: &mut PanelManager) {
    let panels: Vec<PanelId> = manager
        .workspace
        .open
        .iter()
        .filter(|p| manager.workspace.region.get(p).cloned() == Some(Region::Bottom))
        .cloned()
        .collect();

    if panels.is_empty() {
        return;
    }

    ui.horizontal(|ui| {
        if panels.len() > 1 {
            ui.vertical(|ui| {
                for p in &panels {
                    let _ = ui.selectable_label(false, p.to_string());
                }
            });
            ui.separator();
        }
        for p in &panels {
            ui.group(|ui| {
                ui.label(p.to_string());
            });
        }
        let _ = (manager, MIN_PANEL_WIDTH, MIN_PANEL_HEIGHT, Vec2::ZERO);
    });
}

/// Draggable split divider helper. Returns new ratio if dragged.
pub fn split_divider(ui: &mut Ui, vertical: bool, ratio: &mut f32) {
    let id = ui.scope_id().with("split_divider");
    let (rect, response) = ui.allocate_exact_size(
        if vertical {
            Vec2::new(6.0, ui.available_height())
        } else {
            Vec2::new(ui.available_width(), 6.0)
        },
        egui::Sense::drag(),
    );
    if response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            let total = if vertical {
                ui.available_width() + rect.width()
            } else {
                ui.available_height() + rect.height()
            };
            if total > 0.0 {
                let frac = if vertical {
                    pos.x / total
                } else {
                    pos.y / total
                };
                *ratio = frac.clamp(0.2, 0.8);
            }
        }
    }
    let _ = id;
    ui.painter()
        .rect_filled(rect, 0.0, ui.visuals().widgets.inactive.bg_fill);
}

/// Recursive split rendering placeholder (depth-limited).
pub fn render_split_tree(ui: &mut Ui, _manager: &mut PanelManager, depth: u8) {
    if depth > 4 {
        ui.label("Max split depth reached");
        return;
    }
    ui.label("split pane");
}
