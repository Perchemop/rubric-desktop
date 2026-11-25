//! Toolbar Customization Dialog (spec F-608): 4-pane modal
//! (Categories, Commands, Toolbars, Items) with drag-drop, live preview hooks.

use eframe::egui::{self, Ui};
use std::collections::{HashMap, HashSet};

/// Command category (7 total per spec).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    SelectionNavigation,
    MarkupTools,
    MeasurementTools,
    ReviewComment,
    EditModify,
    ViewDisplay,
    FileProject,
    Advanced,
}

impl CommandCategory {
    pub const ALL: &'static [CommandCategory] = &[
        CommandCategory::SelectionNavigation,
        CommandCategory::MarkupTools,
        CommandCategory::MeasurementTools,
        CommandCategory::ReviewComment,
        CommandCategory::EditModify,
        CommandCategory::ViewDisplay,
        CommandCategory::FileProject,
        CommandCategory::Advanced,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            CommandCategory::SelectionNavigation => "Selection & Navigation",
            CommandCategory::MarkupTools => "Markup Tools",
            CommandCategory::MeasurementTools => "Measurement Tools",
            CommandCategory::ReviewComment => "Review & Comment",
            CommandCategory::EditModify => "Edit & Modify",
            CommandCategory::ViewDisplay => "View & Display",
            CommandCategory::FileProject => "File & Project",
            CommandCategory::Advanced => "Advanced",
        }
    }
}

/// A command entry in the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEntry {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub shortcut: String,
    pub category: CommandCategory,
}

/// Dialog session state.
#[derive(Debug, Clone, Default)]
pub struct ToolbarCustomizeDialogState {
    pub open: bool,
    /// Selected categories (multi-select).
    pub selected_categories: HashSet<CommandCategory>,
    /// Search filter in commands pane.
    pub search: String,
    pub search_focus: bool,
    /// Currently selected toolbar id.
    pub selected_toolbar: Option<String>,
    /// Selected item index in items pane.
    pub selected_item: Option<usize>,
    /// New/rename toolbar name buffer.
    pub name_buffer: String,
    pub show_rename: bool,
    pub show_new: bool,
    pub show_delete_confirm: bool,
    pub show_reset_confirm: bool,
    /// Last selected category/toolbar remembered per session (NFR-07).
    pub last_category: Option<CommandCategory>,
    pub last_toolbar: Option<String>,
    /// Lock toggle state mirrored from manager.
    pub global_lock: bool,
}

impl ToolbarCustomizeDialogState {
    pub fn new() -> Self {
        let mut s = Self::default();
        // All categories checked by default.
        for c in CommandCategory::ALL {
            s.selected_categories.insert(*c);
        }
        s
    }
}

/// Default command registry (subset). Real registry lives in domain; this is a
/// UI-side fallback so the dialog is populated even before domain ships.
pub fn default_commands() -> Vec<CommandEntry> {
    let mut v = Vec::new();
    let push = |v: &mut Vec<CommandEntry>, id, name, icon, shortcut, cat| {
        v.push(CommandEntry {
            id: id.into(),
            name: name.into(),
            icon: icon.into(),
            shortcut: shortcut.into(),
            category: cat,
        });
    };
    push(&mut v, "select", "Select", "sel", "V", CommandCategory::SelectionNavigation);
    push(&mut v, "pan", "Pan", "pan", "H", CommandCategory::SelectionNavigation);
    push(&mut v, "zoom_in", "Zoom In", "zi", "+", CommandCategory::SelectionNavigation);
    push(&mut v, "zoom_out", "Zoom Out", "zo", "-", CommandCategory::SelectionNavigation);
    push(&mut v, "zoom_fit", "Zoom Fit", "zf", "F", CommandCategory::SelectionNavigation);
    push(&mut v, "highlight", "Highlight", "hl", "", CommandCategory::MarkupTools);
    push(&mut v, "strikeout", "Strikeout", "sk", "", CommandCategory::MarkupTools);
    push(&mut v, "underline", "Underline", "ul", "", CommandCategory::MarkupTools);
    push(&mut v, "rectangle", "Rectangle", "rc", "", CommandCategory::MarkupTools);
    push(&mut v, "ellipse", "Ellipse", "el", "", CommandCategory::MarkupTools);
    push(&mut v, "length", "Length", "ln", "", CommandCategory::MeasurementTools);
    push(&mut v, "area", "Area", "ar", "", CommandCategory::MeasurementTools);
    push(&mut v, "calibrate", "Calibrate Scale", "ca", "", CommandCategory::MeasurementTools);
    push(&mut v, "add_comment", "Add Comment", "cm", "", CommandCategory::ReviewComment);
    push(&mut v, "status", "Status Dropdown", "st", "", CommandCategory::ReviewComment);
    push(&mut v, "copy_props", "Copy Properties", "cp", "", CommandCategory::EditModify);
    push(&mut v, "paste_props", "Paste Properties", "pp", "", CommandCategory::EditModify);
    push(&mut v, "theme_toggle", "Theme Toggle", "tt", "", CommandCategory::ViewDisplay);
    push(&mut v, "grid", "Show/Hide Grid", "gr", "", CommandCategory::ViewDisplay);
    push(&mut v, "new", "New", "nw", "Ctrl+N", CommandCategory::FileProject);
    push(&mut v, "open", "Open", "op", "Ctrl+O", CommandCategory::FileProject);
    push(&mut v, "save", "Save", "sv", "Ctrl+S", CommandCategory::FileProject);
    push(&mut v, "prefs", "Preferences", "pf", "", CommandCategory::Advanced);
    push(&mut v, "profiles", "Profiles Management", "pr", "", CommandCategory::Advanced);
    // Divider pseudo-command (always available).
    v.push(CommandEntry {
        id: "divider".into(),
        name: "Divider".into(),
        icon: "|".into(),
        shortcut: "".into(),
        category: CommandCategory::SelectionNavigation,
    });
    v
}

/// Render the modal dialog. Returns true if config changed (live preview).
pub fn show(
    ctx: &egui::Context,
    state: &mut ToolbarCustomizeDialogState,
    commands: &[CommandEntry],
    toolbars: &mut Vec<crate::ui::toolbar_runtime::ToolbarConfig>,
    lock: &mut crate::ui::toolbar_runtime::ToolbarLockState,
) -> bool {
    if !state.open {
        return false;
    }
    let mut changed = false;

    // Keyboard: Escape closes.
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.open = false;
        return changed;
    }

    egui::Window::new("Customize Toolbars")
        .collapsible(false)
        .resizable(true)
        .default_width(900.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Ctrl+F to search commands");
            });

            ui.columns(4, |cols| {
                // Pane 1: Categories
                cols[0].group(|ui| {
                    ui.heading("Categories");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for cat in CommandCategory::ALL {
                            let mut checked = state.selected_categories.contains(cat);
                            if ui.checkbox(&mut checked, cat.label()).changed() {
                                if checked {
                                    state.selected_categories.insert(*cat);
                                } else {
                                    state.selected_categories.remove(cat);
                                }
                                changed = true;
                            }
                        }
                    });
                });

                // Pane 2: Commands
                cols[1].group(|ui| {
                    ui.heading("Commands");
                    let search = state.search.clone();
                    ui.text_edit_singleline(&mut state.search);
                    if state.search_focus {
                        // best-effort focus
                    }
                    let filtered: Vec<&CommandEntry> = commands
                        .iter()
                        .filter(|c| {
                            state.selected_categories.contains(&c.category)
                                && (search.is_empty()
                                    || c.name.to_lowercase().contains(&search.to_lowercase())
                                    || c.id.to_lowercase().contains(&search.to_lowercase()))
                        })
                        .collect();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for cmd in filtered {
                            let label = format!("{} {} {}", cmd.icon, cmd.name, cmd.shortcut);
                            let resp = ui.selectable_label(false, label);
                            // Drag source
                            if resp.drag_started() {
                                // store dragged id in egui dnd
                                egui::DragAndDrop::set_payload(ctx, cmd.id.clone());
                            }
                        }
                    });
                });

                // Pane 3: Toolbars
                cols[2].group(|ui| {
                    ui.heading("Toolbars");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for tb in toolbars.iter() {
                            ui.horizontal(|ui| {
                                let mut vis = tb.visible;
                                if ui.checkbox(&mut vis, "").changed() {
                                    // find mutable
                                    if let Some(t) = toolbars.iter_mut().find(|x| x.id == tb.id) {
                                        t.visible = vis;
                                        changed = true;
                                    }
                                }
                                ui.label(&tb.name);
                                if tb.predefined {
                                    ui.small("(predefined)");
                                } else {
                                    ui.small("(custom)");
                                }
                                if lock.global_locked && !tb.predefined {
                                    let ov =
                                        lock.custom_overrides.get(&tb.id).copied().unwrap_or(false);
                                    let mut v = ov;
                                    if ui.checkbox(&mut v, "unlock").changed() {
                                        lock.custom_overrides.insert(tb.id.clone(), v);
                                        changed = true;
                                    }
                                }
                            });
                            if ui
                                .selectable_label(
                                    state.selected_toolbar.as_deref() == Some(&tb.id),
                                    &tb.name,
                                )
                                .clicked()
                            {
                                state.selected_toolbar = Some(tb.id.clone());
                                state.selected_item = None;
                            }
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("New").clicked() {
                            state.show_new = true;
                            state.name_buffer.clear();
                        }
                        let sel_predefined = state
                            .selected_toolbar
                            .as_ref()
                            .and_then(|id| toolbars.iter().find(|t| &t.id == id))
                            .map(|t| t.predefined)
                            .unwrap_or(true);
                        if ui.button("Rename").clicked() && !sel_predefined {
                            state.show_rename = true;
                            state.name_buffer = state
                                .selected_toolbar
                                .as_ref()
                                .and_then(|id| toolbars.iter().find(|t| &t.id == id))
                                .map(|t| t.name.clone())
                                .unwrap_or_default();
                        }
                        let del_enabled = !sel_predefined && state.selected_toolbar.is_some();
                        if ui
                            .add_enabled(del_enabled, egui::Button::new("Delete"))
                            .clicked()
                        {
                            state.show_delete_confirm = true;
                        }
                    });
                    // New toolbar dialog
                    if state.show_new {
                        ui.group(|ui| {
                            ui.label("New toolbar name:");
                            ui.text_edit_singleline(&mut state.name_buffer);
                            if ui.button("Create").clicked() {
                                let name = state.name_buffer.trim().to_string();
                                if !name.is_empty() {
                                    let id =
                                        format!("custom_{}", name.to_lowercase().replace(' ', "_"));
                                    let exists = toolbars
                                        .iter()
                                        .any(|t| t.name.eq_ignore_ascii_case(&name));
                                    if exists {
                                        // error: duplicate
                                    } else if toolbars
                                        .iter()
                                        .filter(|t| !t.predefined)
                                        .count()
                                        >= 20
                                    {
                                        // max reached
                                    } else {
                                        toolbars.push(crate::ui::toolbar_runtime::ToolbarConfig {
                                            id,
                                            name,
                                            predefined: false,
                                            dock: crate::ui::toolbar_runtime::ToolbarDock::Top,
                                            visible: true,
                                            items: Vec::new(),
                                            locked_override: None,
                                        });
                                        changed = true;
                                    }
                                    state.show_new = false;
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                state.show_new = false;
                            }
                        });
                    }
                    if state.show_rename {
                        ui.group(|ui| {
                            ui.label("Rename toolbar:");
                            ui.text_edit_singleline(&mut state.name_buffer);
                            if ui.button("Apply").clicked() {
                                if let Some(id) = state.selected_toolbar.clone() {
                                    if let Some(t) = toolbars.iter_mut().find(|x| x.id == id) {
                                        t.name = state.name_buffer.trim().to_string();
                                        changed = true;
                                    }
                                }
                                state.show_rename = false;
                            }
                            if ui.button("Cancel").clicked() {
                                state.show_rename = false;
                            }
                        });
                    }
                    if state.show_delete_confirm {
                        ui.group(|ui| {
                            ui.label("Delete selected custom toolbar?");
                            if ui.button("Confirm Delete").clicked() {
                                if let Some(id) = state.selected_toolbar.clone() {
                                    toolbars.retain(|t| t.id != id);
                                    state.selected_toolbar = None;
                                    changed = true;
                                }
                                state.show_delete_confirm = false;
                            }
                            if ui.button("Cancel").clicked() {
                                state.show_delete_confirm = false;
                            }
                        });
                    }
                });

                // Pane 4: Items
                cols[3].group(|ui| {
                    ui.heading("Items");
                    // Determine selected toolbar items
                    let selected_id = state.selected_toolbar.clone();
                    let mut items_clone: Vec<crate::ui::toolbar_runtime::ToolbarItem> = selected_id
                        .as_ref()
                        .and_then(|id| toolbars.iter().find(|t| &t.id == id))
                        .map(|t| t.items.clone())
                        .unwrap_or_default();
                    let is_properties = selected_id.as_deref() == Some("properties");

                    // Drop zone for commands -> items
                    let drop_resp =
                        ui.dnd_drop_zone::<String, _>(egui::Frame::canvas(ui.style()), |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (idx, item) in items_clone.iter().enumerate() {
                                    let label = match item {
                                        crate::ui::toolbar_runtime::ToolbarItem::Tool(id) => {
                                            id.clone()
                                        }
                                        crate::ui::toolbar_runtime::ToolbarItem::Divider => {
                                            "— Divider —".to_string()
                                        }
                                        crate::ui::toolbar_runtime::ToolbarItem::Spacer => {
                                            "Spacer".to_string()
                                        }
                                    };
                                    let sel = state.selected_item == Some(idx);
                                    let resp = ui.selectable_label(sel, label);
                                    if resp.clicked() {
                                        state.selected_item = Some(idx);
                                    }
                                    // drag source for reorder/remove
                                    if resp.drag_started() {
                                        egui::DragAndDrop::set_payload(
                                            ctx,
                                            format!("item:{}", idx),
                                        );
                                    }
                                }
                                if items_clone.is_empty() {
                                    ui.label("(empty)");
                                }
                            });
                        });
                    if let Some(payload) = drop_resp.payload::<String>() {
                        let p = payload.clone();
                        if let Some(id) = state.selected_toolbar.clone() {
                            if let Some(tb) = toolbars.iter_mut().find(|t| t.id == id) {
                                if is_properties {
                                    // reject
                                } else if p == "divider" || !p.starts_with("item:") {
                                    // adding command
                                    let new_item = if p == "divider" {
                                        crate::ui::toolbar_runtime::ToolbarItem::Divider
                                    } else {
                                        crate::ui::toolbar_runtime::ToolbarItem::Tool(p.clone())
                                    };
                                    // append at end (drop position not tracked precisely)
                                    tb.items.push(new_item);
                                    changed = true;
                                } else if let Some(rest) = p.strip_prefix("item:") {
                                    if let Ok(from) = rest.parse::<usize>() {
                                        // reorder handled separately; here just remove if dragged to commands? skip
                                        let _ = from;
                                    }
                                }
                            }
                        }
                    }

                    // Delete key support
                    if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
                        if let Some(idx) = state.selected_item {
                            if let Some(id) = state.selected_toolbar.clone() {
                                if let Some(tb) = toolbars.iter_mut().find(|t| t.id == id) {
                                    if !is_properties && idx < tb.items.len() {
                                        tb.items.remove(idx);
                                        state.selected_item = None;
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }
                });
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Reset to Defaults").clicked() {
                    state.show_reset_confirm = true;
                }
                if state.show_reset_confirm {
                    ui.label("Confirm reset?");
                    if ui.button("Yes").clicked() {
                        // reset handled by caller; flag via changed
                        changed = true;
                        state.show_reset_confirm = false;
                    }
                    if ui.button("No").clicked() {
                        state.show_reset_confirm = false;
                    }
                }
                ui.checkbox(&mut state.global_lock, "Lock Toolbars");
                lock.global_locked = state.global_lock;
            });
        });

    changed
}
