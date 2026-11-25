//! Panel Access Bar — left-edge vertical icon strip with auto-hide.

use crate::domain::workspace::PanelId;
use crate::ui::panel_manager::PanelManager;
use egui::{Context, Ui, SidePanel};
use std::time::{Duration, Instant};

pub struct PanelAccessBar {
    pub auto_hide: bool,
    pub expanded: bool,
    pub collapsed_width: f32,
    pub expanded_width: f32,
    pub anim_duration: Duration,
    pub hover_since: Option<Instant>,
}

impl Default for PanelAccessBar {
    fn default() -> Self {
        Self {
            auto_hide: false,
            expanded: true,
            collapsed_width: 32.0,
            expanded_width: 48.0,
            anim_duration: Duration::from_millis(150),
            hover_since: None,
        }
    }
}

impl PanelAccessBar {
    pub fn width(&self) -> f32 {
        if self.auto_hide && !self.expanded {
            self.collapsed_width
        } else {
            self.expanded_width
        }
    }

    /// Render the access bar. `panels` is the list of dockable panels.
    pub fn show(&mut self, ui: &mut Ui, manager: &mut PanelManager, panels: &[PanelId]) {
        // Right-click toggles auto-hide.
        let resp = ui.interact(
            ui.available_rect_before_wrap(),
            ui.id().with("access_bar"),
            egui::Sense::click(),
        );
        if resp.secondary_clicked() {
            self.auto_hide = !self.auto_hide;
            manager.access_bar_auto_hide = self.auto_hide;
        }

        // Hover expand for auto-hide.
        if self.auto_hide {
            if resp.hovered() {
                if self.hover_since.is_none() {
                    self.hover_since = Some(Instant::now());
                }
                if let Some(t) = self.hover_since {
                    if t.elapsed() >= self.anim_duration {
                        self.expanded = true;
                    }
                }
            } else {
                self.hover_since = None;
                self.expanded = false;
            }
        } else {
            self.expanded = true;
        }
        manager.access_bar_expanded = self.expanded;

        ui.vertical(|ui| {
            for p in panels {
                let visible = manager.is_visible(*p);
                let label = if self.expanded {
                    p.to_string()
                } else {
                    p.to_string().chars().next().unwrap_or('?').to_string()
                };
                let btn = ui.selectable_label(visible, label);
                if btn.clicked() {
                    manager.toggle_panel(*p);
                }
                btn.on_hover_text(format!("{} (toggle)", p));
            }
        });
    }

    pub fn show_ctx(&mut self, ctx: &Context, manager: &mut PanelManager, panels: &[PanelId]) {
        // Convenience: create a side panel.
        egui::SidePanel::left("panel_access_bar")
            .default_width(self.width())
            .show(ctx, |ui| {
                self.show(ui, manager, panels);
            });
    }
}

// Context already imported above.
