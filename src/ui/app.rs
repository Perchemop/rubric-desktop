//! The main egui application: layout, state, and high-level orchestration.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use eframe::{App, CreationContext, Frame};
use egui::{Align, Layout, Pos2, Ui, Vec2};

use crate::data::{
    load_project, load_recent, normalise_project_filename, save_project, save_recent,
};
use crate::domain::{Document, History, MarkupKind, Project, Pt, StampPreset, Tool};
use crate::ui::{canvas, inspector, theme, toolbar};
use uuid::Uuid;

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
pub(crate) struct TextEditState {
    pub(crate) id: Uuid,
    pub(crate) original: String,
    pub(crate) text: String,
    pub(crate) focus_requested: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UiState {
    pub(crate) selected: HashSet<uuid::Uuid>,
    pub(crate) active_tool: Tool,
    pub(crate) active_stamp: StampPreset,
    inspector_mode: InspectorMode,
    pub(crate) zoom: f32,
    pub(crate) pan: Vec2,
    pub(crate) page_origin: Pos2,
    hint: String,
    pub(crate) status: String,
    /// Set when an in-progress draw is cancelled (Esc) so the still-held drag
    /// does not spawn a fresh draft; cleared when that drag starts or ends.
    pub(crate) draw_cancelled: bool,
    /// The tool that was active when the last canvas frame ran. Used to detect
    /// a tool switch and abort any draft that is still in progress.
    pub(crate) last_tool: Tool,
    /// True while a drag that began on a selected markup is moving it.
    pub(crate) move_selection: bool,
    /// True once the current move-selection drag has actually translated a
    /// markup. Cleared when the drag starts and when it ends; drives the
    /// "move markups" snapshot so a zero-delta drag pushes nothing.
    pub(crate) move_changed: bool,
    /// In-canvas text editor state (Text/Callout markups only). UI-only, no
    /// serde: it never leaves the session.
    pub(crate) text_edit: Option<TextEditState>,
    dirty: bool,
    /// Custom scale editor inputs (paper inches and real units).
    pub(crate) scale_paper_inches: f64,
    pub(crate) scale_real_units: f64,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected: HashSet::new(),
            active_tool: Tool::Select,
            active_stamp: StampPreset::Approved,
            inspector_mode: InspectorMode::Selection,
            zoom: 1.0,
            pan: Vec2::ZERO,
            page_origin: Pos2::new(40.0, 40.0),
            hint: "Ctrl+S: Save  ·  Ctrl+Z: Undo  ·  Ctrl+Shift+Z: Redo  ·  Del: Delete"
                .to_string(),
            status: "Ready".to_string(),
            draw_cancelled: false,
            last_tool: Tool::Select,
            move_selection: false,
            move_changed: false,
            text_edit: None,
            dirty: false,
            scale_paper_inches: 1.0,
            scale_real_units: 10.0,
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
    /// Session-only association with a file on disk. Not part of the project
    /// snapshot, so it never undoes/redoes with history.
    current_path: Option<PathBuf>,
    /// Set when the window title needs to be re-synced with `current_path`.
    title_dirty: bool,
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
            current_path: None,
            title_dirty: true,
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

    fn clear_selection(&mut self) {
        self.ui.selected.clear();
        self.ui.dirty = true;
    }

    fn delete_selected(&mut self) {
        let ids: Vec<_> = self.ui.selected.iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let mut deleted = false;
        if let Some(doc) = self.active_doc_mut() {
            for id in ids {
                // Skip ids that no longer exist (e.g. a stale selection after an
                // undo): only a real removal counts as a deletion.
                let Some(m) = doc.find_markup(id) else {
                    continue;
                };
                if !doc.is_markup_editable(m) {
                    continue;
                }
                doc.remove_markup(id);
                deleted = true;
            }
        }
        if deleted {
            self.clear_selection();
            self.push_snapshot("delete markups");
        } else {
            self.ui.status = "Selection is locked".to_string();
        }
    }

    fn push_snapshot(&mut self, reason: &str) {
        let proj = self.history.current().clone();
        self.history.push(proj);
        self.ui.status = format!("Snapshot: {reason}");
    }

    /// Push a snapshot without touching the status bar. Used when the canvas
    /// already set a more specific status earlier in the same frame.
    fn push_snapshot_quiet(&mut self) {
        let proj = self.history.current().clone();
        self.history.push(proj);
    }

    /// Open (or re-open) a project file, replacing the current session.
    fn open_project_path(&mut self, path: PathBuf) {
        if !path.exists() {
            // Drop the missing file from the app-level recents. App state,
            // not project state — no undo snapshot, and a read-only config
            // dir must not turn this into an open failure.
            let pstr = path.to_string_lossy().to_string();
            self.ui.status = format!("Missing: {pstr}");
            let mut recent = load_recent();
            recent.remove(&pstr);
            let _ = save_recent(&recent);
            return;
        }
        match load_project(&path) {
            Ok(proj) => {
                self.history = History::new(proj);
                self.ui = UiState::default();
                self.draw = DrawState::Idle;
                self.current_path = Some(path.clone());
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                self.ui.status = format!("Opened {name}");
                self.title_dirty = true;
                // Record the open in the app-level recents store. A failed
                // save (e.g. read-only config dir) must not fail the open.
                let mut recent = load_recent();
                recent.push(path.to_string_lossy().to_string());
                let _ = save_recent(&recent);
            }
            Err(e) => {
                self.ui.status = format!("Open failed: {e}");
            }
        }
    }

    /// Save the current project to `path`, updating session state on success.
    fn save_project_to(&mut self, path: PathBuf) {
        // Normalise the extension at the UI boundary: save_project writes
        // exactly what it is given.
        let path = PathBuf::from(normalise_project_filename(
            path.to_string_lossy().into_owned(),
        ));
        match save_project(self.history.current(), &path) {
            Ok(_) => {
                self.current_path = Some(path.clone());
                self.ui.dirty = false;
                self.ui.status = "Saved".to_string();
                self.title_dirty = true;
                // Record the save in the app-level recents store. App state,
                // not project state — no undo snapshot, and a failed save
                // (e.g. read-only config dir) must not fail the save.
                let mut recent = load_recent();
                recent.push(path.to_string_lossy().to_string());
                let _ = save_recent(&recent);
            }
            Err(e) => {
                self.ui.status = format!("Save failed: {e}");
            }
        }
    }

    fn save_project_action(&mut self) {
        if let Some(p) = self.current_path.clone() {
            self.save_project_to(p);
        } else {
            self.save_project_as_action();
        }
    }

    fn save_project_as_action(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rubric project", &["rubric.json"])
            .set_file_name("project.rubric.json")
            .save_file()
        {
            self.save_project_to(path);
        }
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        use egui::MenuBar;
        MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Project").clicked() {
                    self.history = History::new(Project::new("Untitled Rubric Project"));
                    self.ui = UiState::default();
                    self.draw = DrawState::Idle;
                    self.current_path = None;
                    self.ui.status = "New project created".to_string();
                    self.title_dirty = true;
                    ui.close();
                }
                if ui.button("Open Project...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Rubric project", &["rubric.json"])
                        .pick_file()
                    {
                        self.open_project_path(path);
                    }
                    ui.close();
                }
                // Recent submenu: app-level recents (data layer), not project
                // settings — reads/writes `recent.json` in the config dir.
                ui.menu_button("Recent", |ui| {
                    let recents = load_recent();
                    if recents.is_empty() {
                        ui.label("No recent projects");
                    } else {
                        for r in recents.as_slice() {
                            let path = PathBuf::from(r);
                            let label = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| r.clone());
                            let resp = ui.button(&label);
                            resp.clone().on_hover_text(r.as_str());
                            if resp.clicked() {
                                self.open_project_path(path);
                                ui.close();
                            }
                        }
                        ui.separator();
                        if ui.button("Clear Recent").clicked() {
                            let mut recent = load_recent();
                            recent.clear();
                            let _ = save_recent(&recent);
                            self.ui.status = "Recent list cleared".to_string();
                            ui.close();
                        }
                    }
                });
                if ui.button("Save Project").clicked() {
                    self.save_project_action();
                    ui.close();
                }
                if ui.button("Save Project As...").clicked() {
                    self.save_project_as_action();
                    ui.close();
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    // Let eframe tear down normally instead of exiting the
                    // process outright.
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    if self.ui.text_edit.is_some() {
                        self.ui.status = "Finish or cancel text edit first".to_string();
                    } else if self.history.undo() {
                        self.ui.status = "Undo".to_string();
                        self.clear_selection();
                        ui.close();
                    }
                }
                if ui.button("Redo").clicked() {
                    if self.ui.text_edit.is_some() {
                        self.ui.status = "Finish or cancel text edit first".to_string();
                    } else if self.history.redo() {
                        self.ui.status = "Redo".to_string();
                        self.clear_selection();
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Delete Selected").clicked() {
                    if self.ui.text_edit.is_some() {
                        self.ui.status = "Finish or cancel text edit first".to_string();
                    } else {
                        self.delete_selected();
                        ui.close();
                    }
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

    fn right_panel(&mut self, ui: &mut Ui) -> bool {
        inspector::draw_inspector(
            ui,
            self.history.current_mut(),
            &mut self.ui,
            &mut self.comment_text,
        )
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
            let editing = self.ui.text_edit.is_some();
            if (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) && !editing
            {
                self.delete_selected();
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) {
                // Never mutate history mid-edit: undoing past the markup's
                // creation would leave a stale `text_edit` pointing at a
                // markup that no longer exists.
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else if i.modifiers.shift {
                    self.history.redo();
                    self.clear_selection();
                } else {
                    self.history.undo();
                    self.clear_selection();
                }
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
                // Native dialog must run on the main thread; the logic() handler
                // runs on it, so calling rfd here is safe.
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Rubric project", &["rubric.json"])
                    .pick_file()
                {
                    self.open_project_path(path);
                }
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                // save_project_to sets an accurate "Saved" / "Save failed: …"
                // status; no override here.
                self.save_project_action();
            }
            if i.key_pressed(egui::Key::Escape) {
                if self.ui.text_edit.is_some() {
                    // Discard the in-progress text edit, no snapshot.
                    self.ui.text_edit = None;
                    self.ui.status = "Text edit cancelled".to_string();
                } else {
                    // Abort the draft and consume the current (still held) drag, so
                    // canvas.rs does not immediately start a new one.
                    self.ui.draw_cancelled = true;
                    self.draw = DrawState::Idle;
                    self.ui.move_selection = false;
                    self.clear_selection();
                }
            }
        });
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        if self.title_dirty {
            let title = match &self.current_path {
                Some(p) => format!(
                    "{} — Rubric",
                    p.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| p.to_string_lossy().to_string())
                ),
                None => "Untitled — Rubric".to_string(),
            };
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title(title));
            self.title_dirty = false;
        }
        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui));
        egui::Panel::top("toolbar").show(ui, |ui| self.top_bar(ui));
        egui::Panel::left("tools")
            .default_size(200.0)
            .min_size(180.0)
            .show(ui, |ui| self.left_panel(ui));
        let mut inspector_mutated = false;
        egui::Panel::right("inspector")
            .default_size(280.0)
            .min_size(240.0)
            .show(ui, |ui| {
                inspector_mutated = self.right_panel(ui);
            });
        egui::Panel::bottom("status").show(ui, |ui| self.bottom_bar(ui));

        let prior_status = self.ui.status.clone();
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
            // The canvas can set a more specific status this frame (e.g.
            // "Text updated" from a text-edit commit); keep it instead of
            // overwriting it with the generic snapshot message. Comparing
            // against the pre-canvas status keeps the check frame-scoped.
            if self.ui.status != prior_status {
                self.push_snapshot_quiet();
            } else {
                self.push_snapshot("drawing");
            }
        }
        if inspector_mutated {
            self.push_snapshot("inspector edit");
        }

        ui.ctx()
            .request_repaint_after(Duration::from_secs_f32(0.25));
    }
}
