//! Split Wheel overlay — 4-zone drop indicator shown when dragging a panel
//! over an existing docked panel.

use crate::ui::panel_manager::{PanelManager, SplitZone};
use egui::{Context, Pos2, Rect, Ui, Vec2};
use std::time::{Duration, Instant};

/// Animated 4-zone split wheel overlay.
pub struct SplitWheelOverlay {
    pub visible: bool,
    pub center: Pos2,
    pub size: f32,
    pub appear_at: Option<Instant>,
    pub duration: Duration,
}

impl Default for SplitWheelOverlay {
    fn default() -> Self {
        Self {
            visible: false,
            center: Pos2::ZERO,
            size: 80.0,
            appear_at: None,
            duration: Duration::from_millis(100),
        }
    }
}

impl SplitWheelOverlay {
    pub fn show(&mut self, center: Pos2) {
        self.visible = true;
        self.center = center;
        self.appear_at = Some(Instant::now());
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.appear_at = None;
    }

    /// Alpha for the appear animation (0..=1).
    pub fn alpha(&self) -> f32 {
        match self.appear_at {
            Some(t) => {
                let elapsed = t.elapsed().as_secs_f32();
                let total = self.duration.as_secs_f32().max(0.001);
                (elapsed / total).clamp(0.0, 1.0)
            }
            None => 0.0,
        }
    }

    /// Hit-test a point against the 4 zones. Returns the zone if inside.
    pub fn hit_zone(&self, pos: Pos2) -> Option<SplitZone> {
        if !self.visible {
            return None;
        }
        let half = self.size * 0.5;
        let rect = Rect::from_center_size(self.center, Vec2::splat(self.size));
        if !rect.contains(pos) {
            return None;
        }
        // Divide into 4 quadrants.
        let dx = pos.x - self.center.x;
        let dy = pos.y - self.center.y;
        let zone = if dx.abs() > dy.abs() {
            if dx < 0.0 {
                SplitZone::Left
            } else {
                SplitZone::Right
            }
        } else {
            if dy < 0.0 {
                SplitZone::Top
            } else {
                SplitZone::Bottom
            }
        };
        let _ = half;
        Some(zone)
    }

    pub fn render(&self, ctx: &Context, manager: &mut PanelManager) {
        if !self.visible {
            return;
        }
        // Escape cancels.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            manager.cancel_drag();
            // cannot mutate self here easily; caller handles hide
        }
        let alpha = self.alpha();
        let _ = (ctx, manager, alpha);
    }

    pub fn render_in_ui(&self, ui: &mut Ui) {
        if !self.visible {
            return;
        }
        let rect = Rect::from_center_size(self.center, Vec2::splat(self.size));
        ui.painter()
            .rect_stroke(rect, 4.0, (1.0, egui::Color32::from_rgb(80, 140, 220)));
        // zone labels
        ui.painter().text(
            rect.left_center(),
            egui::Align2::LEFT_CENTER,
            "◀",
            egui::FontId::default(),
            egui::Color32::WHITE,
        );
    }
}
