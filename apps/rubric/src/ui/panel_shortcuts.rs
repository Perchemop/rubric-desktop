//! Keyboard shortcuts for panels: Ctrl+1..7 toggle, customizable, conflict detection.

use crate::domain::workspace::PanelId;
use crate::ui::panel_manager::PanelManager;
use egui::{Context, Key, Modifiers};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelShortcut {
    pub panel: PanelId,
    pub key: Key,
    pub modifiers: Modifiers,
}

impl PanelShortcut {
    pub fn ctrl_digit(panel: PanelId, digit: usize) -> Self {
        Self {
            panel,
            key: match digit {
                1 => Key::Num1,
                2 => Key::Num2,
                3 => Key::Num3,
                4 => Key::Num4,
                5 => Key::Num5,
                6 => Key::Num6,
                7 => Key::Num7,
                _ => Key::Num1,
            },
            modifiers: Modifiers::CTRL,
        }
    }
}

pub struct PanelShortcutRegistry {
    pub shortcuts: BTreeMap<PanelId, PanelShortcut>,
}

impl Default for PanelShortcutRegistry {
    fn default() -> Self {
        let panels = [
            PanelId::Pages,
            PanelId::Tools,
            PanelId::ToolChest,
            PanelId::Inspector,
            PanelId::MarkupsList,
            PanelId::Quantities,
        ];
        let mut shortcuts = BTreeMap::new();
        for (i, p) in panels.iter().enumerate() {
            shortcuts.insert(*p, PanelShortcut::ctrl_digit(*p, i + 1));
        }
        // Stamps is not in PanelId enum; skip (domain gap).
        Self { shortcuts }
    }
}

impl PanelShortcutRegistry {
    /// Detect conflicts: returns list of panels sharing the same (key, modifiers).
    pub fn conflicts(&self) -> Vec<(PanelId, PanelId)> {
        let mut out = Vec::new();
        let entries: Vec<_> = self.shortcuts.iter().collect();
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                if entries[i].1.key == entries[j].1.key
                    && entries[i].1.modifiers == entries[j].1.modifiers
                {
                    out.push((*entries[i].0, *entries[j].0));
                }
            }
        }
        out
    }

    /// Handle keyboard input this frame. Toggles panel on matching shortcut.
    pub fn handle(&self, ctx: &Context, manager: &mut PanelManager, focused: &mut Option<PanelId>) {
        ctx.input(|input| {
            for (panel, sc) in &self.shortcuts {
                if input.key_pressed(sc.key) && input.modifiers == sc.modifiers {
                    // toggle behavior: hidden->show, visible->focus, focused->hide
                    if manager.is_visible(*panel) {
                        if focused.as_ref() == Some(panel) {
                            manager.toggle_panel(*panel);
                            *focused = None;
                        } else {
                            *focused = Some(*panel);
                        }
                    } else {
                        manager.toggle_panel(*panel);
                        *focused = Some(*panel);
                    }
                }
            }
        });
    }
}
