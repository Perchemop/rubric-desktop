//! Floating panel windows — Windowed (native decorations) and Overlay (frameless,
//! always-on-top) modes.

use crate::domain::workspace::PanelId;
use egui::{Context, Ui, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatMode {
    Windowed,
    Overlay,
}

#[derive(Debug, Clone)]
pub struct FloatingPanelState {
    pub panel: PanelId,
    pub mode: FloatMode,
    pub pos: egui::Pos2,
    pub size: egui::Vec2,
    pub open: bool,
}

impl Default for FloatingPanelState {
    fn default() -> Self {
        Self {
            panel: PanelId::Inspector,
            mode: FloatMode::Windowed,
            pos: egui::Pos2::new(100.0, 100.0),
            size: egui::Vec2::new(320.0, 240.0),
            open: false,
        }
    }
}

pub struct FloatingPanelWindow {
    pub state: FloatingPanelState,
}

impl FloatingPanelWindow {
    pub fn new(panel: PanelId) -> Self {
        Self {
            state: FloatingPanelState {
                panel,
                open: true,
                ..FloatingPanelState::default()
            },
        }
    }

    pub fn toggle_mode(&mut self) {
        self.state.mode = match self.state.mode {
            FloatMode::Windowed => FloatMode::Overlay,
            FloatMode::Overlay => FloatMode::Windowed,
        };
    }

    /// Render the floating window. Returns true if still open.
    pub fn show(&mut self, ctx: &Context, ui_content: impl FnOnce(&mut Ui)) -> bool {
        if !self.state.open {
            return false;
        }
        let mut open = self.state.open;
        let title = self.state.panel.to_string();
        let mut window = Window::new(title)
            .open(&mut open)
            .default_pos(self.state.pos)
            .default_size(self.state.size)
            .resizable(true);
        if self.state.mode == FloatMode::Overlay {
            window = window.title_bar(false);
        }
        window.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(self.state.panel.to_string());
                if ui.small_button("Mode").clicked() {
                    self.toggle_mode();
                }
                if ui.small_button("Close").clicked() {
                    open = false;
                }
            });
            ui.separator();
            ui_content(ui);
        });
        self.state.open = open;
        self.state.open
    }
}
