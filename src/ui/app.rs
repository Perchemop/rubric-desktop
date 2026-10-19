//! The main egui application: layout, state, and high-level orchestration.

use std::collections::HashSet;
use std::time::Duration;

use eframe::{App, CreationContext, Frame};
use egui::{Align, Layout, Pos2, Ui, Vec2};

use crate::data::{default_project_path, load_project, save_project};
use crate::domain::{Document, History, MarkupKind, Page, Project, Pt, Scale, Tool};
use crate::ui::{canvas, inspector, theme, toolbar};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DrawState {
    /// No active draft.
    Idle,
    /// Freehand ink path under construction.
    Ink {
        strokes: Vec<Vec<Pt>>,
        current: Vec<Pt>,
    },
    /// Drawing a bounds-based shape (rectangle, ellipse, cloud, text box, ...).
    Bounds {
        start: Pt,
        kind: MarkupKind,
        end: Pt,
    },
    /// Drawing a segment (line, arrow, measure length/diameter/radius).
    Segment {
        start: Pt,
        end: Pt,
        kind: MarkupKind,
    },
    /// Drawing a path (polyline, polygon, measure perimeter/area).
    Path {
        points: Vec<Pt>,
        kind: MarkupKind,
        closed: bool,
    },
    /// Drawing a three-point angle.
    Angle {
        a: Pt,
        vertex: Option<Pt>,
        kind: MarkupKind,
    },
    /// Placing a count marker.
    Count { kind: MarkupKind },
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InspectorMode {
    Selection,
    Properties,
    Comments,
    Measurements,
    Settings,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UiState {
    pub(crate) selected: HashSet<uuid::Uuid>,
    pub(crate) active_tool: Tool,
    inspector_mode: InspectorMode,
    pub(crate) zoom: f32,
    pub(crate) pan: Vec2,
    pub(crate) page_origin: Pos2,
    hint: String,
    pub(crate) status: String,
    /// Set when an in-progress draw is cancelled (Esc) so the still-held drag
    /// does not spawn a fresh draft; cleared when that drag starts or ends.
    pub(crate) draw_cancelled: bool,
    /// True while a drag that began on a selected markup is moving it.
    pub(crate) move_selection: bool,
    dirty: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected: HashSet::new(),
            active_tool: Tool::Select,
            inspector_mode: InspectorMode::Selection,
            zoom: 1.0,
            pan: Vec2::ZERO,
            page_origin: Pos2::new(40.0, 40.0),
            hint: "Ctrl+S: Save  ·  Ctrl+Z: Undo  ·  Ctrl+Shift+Z: Redo  ·  Del: Delete"
                .to_string(),
            status: "Ready".to_string(),
            draw_cancelled: false,
            move_selection: false,
            dirty: false,
        }
    }
}

pub struct RubricApp {
    history: History,
    ui: UiState,
    draw: DrawState,
    /// The last pointer page position (for the status bar).
    last_page_pos: Option<Pt>,
    comment_text: String,
}

impl RubricApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        let project = Project::new("Untitled Rubric Project");
        Self {
            history: History::new(project),
            ui: UiState::default(),
            draw: DrawState::Idle,
            last_page_pos: None,
            comment_text: String::new(),
        }
    }

    fn current_project(&self) -> &Project {
        self.history.current()
    }

    fn active_doc(&self) -> Option<&Document> {
        self.current_project().active()
    }

    fn active_doc_mut(&mut self) -> Option<&mut Document> {
        self.history.current_mut().active_mut()
    }

    fn page_of_active_doc(&self) -> Option<Page> {
        self.active_doc().and_then(|d| d.primary_page())
    }

    fn measure_scale(&self) -> Scale {
        self.page_of_active_doc()
            .map(|p| p.scale)
            .unwrap_or(Scale::PAPER)
    }

    fn clear_selection(&mut self) {
        self.ui.selected.clear();
        self.ui.dirty = true;
    }

    fn delete_selected(&mut self) {
        let ids: Vec<_> = self.ui.selected.iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        if let Some(doc) = self.active_doc_mut() {
            for id in ids {
                doc.remove_markup(id);
            }
        }
        self.clear_selection();
        self.push_snapshot("delete markups");
    }

    fn push_snapshot(&mut self, reason: &str) {
        let proj = self.history.current().clone();
        self.history.push(proj);
        self.ui.status = format!("Snapshot: {reason}");
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        use egui::MenuBar;
        MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Project").clicked() {
                    self.history = History::new(Project::new("Untitled Rubric Project"));
                    self.ui = UiState::default();
                    self.draw = DrawState::Idle;
                    self.ui.status = "New project created".to_string();
                    ui.close();
                }
                if ui.button("Open...").clicked() {
                    if let Some(p) = default_project_path(&self.current_project().name) {
                        if p.exists() {
                            if let Ok(proj) = load_project(&p) {
                                self.history = History::new(proj);
                                self.ui = UiState::default();
                                self.draw = DrawState::Idle;
                                self.ui.status = "Project loaded".to_string();
                            }
                        }
                    }
                    ui.close();
                }
                if ui.button("Save").clicked() {
                    if let Some(p) = default_project_path(&self.current_project().name) {
                        let _ = save_project(self.current_project(), &p);
                        self.ui.dirty = false;
                        self.ui.status = "Project saved".to_string();
                    }
                    ui.close();
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() && self.history.undo() {
                    self.ui.status = "Undo".to_string();
                    self.clear_selection();
                    ui.close();
                }
                if ui.button("Redo").clicked() && self.history.redo() {
                    self.ui.status = "Redo".to_string();
                    self.clear_selection();
                    ui.close();
                }
                ui.separator();
                if ui.button("Delete Selected").clicked() {
                    self.delete_selected();
                    ui.close();
                }
            });
            ui.menu_button("View", |ui| {
                if ui.button("Zoom In").clicked() {
                    self.ui.zoom = (self.ui.zoom * 1.25).clamp(0.1, 16.0);
                }
                if ui.button("Zoom Out").clicked() {
                    self.ui.zoom = (self.ui.zoom / 1.25).clamp(0.1, 16.0);
                }
                if ui.button("Fit Page").clicked() {
                    self.ui.zoom = 1.0;
                    self.ui.pan = Vec2::ZERO;
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("About Rubric").clicked() {
                    self.ui.status =
                        "Rubric — free, cross-platform PDF markup & measurement".to_string();
                    ui.close();
                }
            });
        });
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            toolbar::draw_toolbar(ui, &mut self.ui.active_tool, &mut self.ui.status);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(self.ui.status.clone());
                ui.label(format!("{:.0}%", self.ui.zoom * 100.0));
            });
        });
    }

    fn left_panel(&mut self, ui: &mut Ui) {
        toolbar::draw_tool_groups(
            ui,
            &mut self.ui.active_tool,
            &mut self.ui.inspector_mode,
            &mut self.ui.hint,
        );
    }

    fn right_panel(&mut self, ui: &mut Ui) {
        let scale = self.measure_scale();
        inspector::draw_inspector(
            ui,
            self.history.current_mut(),
            &mut self.ui.selected,
            &mut self.comment_text,
            scale,
            &mut self.ui.status,
        );
    }

    fn bottom_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.small(self.ui.hint.clone());
            if let Some(p) = self.last_page_pos {
                ui.small(format!("Cursor: {:.1},{:.1} pt", p.x, p.y));
            }
            if let Some(doc) = self.active_doc() {
                let pages = doc.page_count();
                let plural = if pages == 1 { "" } else { "s" };
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.small(format!("{pages} page{plural}"));
                    ui.small(doc.title.clone());
                });
            }
        });
    }
}

impl App for RubricApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                self.delete_selected();
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) {
                if i.modifiers.shift {
                    self.history.redo();
                } else {
                    self.history.undo();
                }
                self.clear_selection();
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                if let Some(p) = default_project_path(&self.current_project().name) {
                    let _ = save_project(self.current_project(), &p);
                    self.ui.dirty = false;
                    self.ui.status = "Project saved (Ctrl+S)".to_string();
                }
            }
            if i.key_pressed(egui::Key::Escape) {
                // Abort the draft and consume the current (still held) drag, so
                // canvas.rs does not immediately start a new one.
                self.ui.draw_cancelled = true;
                self.draw = DrawState::Idle;
                self.ui.move_selection = false;
                self.clear_selection();
            }
        });
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui));
        egui::Panel::top("toolbar").show(ui, |ui| self.top_bar(ui));
        egui::Panel::left("tools")
            .default_size(200.0)
            .min_size(180.0)
            .show(ui, |ui| self.left_panel(ui));
        egui::Panel::right("inspector")
            .default_size(280.0)
            .min_size(240.0)
            .show(ui, |ui| self.right_panel(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.bottom_bar(ui));

        let mut need_snapshot = false;
        egui::CentralPanel::default().show(ui, |ui| {
            need_snapshot = canvas::draw_canvas(
                ui,
                self.history.current_mut(),
                &mut self.ui,
                &mut self.draw,
                &mut self.last_page_pos,
            );
        });
        if need_snapshot {
            self.push_snapshot("drawing");
        }

        ui.ctx()
            .request_repaint_after(Duration::from_secs_f32(0.25));
    }
}
