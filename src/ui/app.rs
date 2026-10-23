//! The main egui application: layout, state, and high-level orchestration.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use eframe::{App, CreationContext, Frame};
use egui::{Align, Layout, Pos2, TextureHandle, Ui, Vec2};

use crate::data::{
    load_prefs, load_project, load_recent, normalise_project_filename, print_image,
    render_page_to_png_bytes, save_prefs, save_project, save_recent, DataError,
};
use crate::domain::{
    AppPrefs, Document, History, MarkupKind, Project, Pt, Report, ReportGroupKey, StampPreset,
    Theme, Tool,
};
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReportExportFormat {
    Text,
    Json,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReportExportUi {
    pub(crate) group_key: ReportGroupKey,
    pub(crate) format: ReportExportFormat,
    pub(crate) project_title: String,
    pub(crate) total_markups: usize,
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
    /// Currently displayed page index (session state; never snapshotted).
    pub(crate) current_page: usize,
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
            current_page: 0,
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
    /// Cache of rendered PDF page textures, keyed by cache PNG path.
    page_textures: HashMap<String, TextureHandle>,
    /// Cache keys whose page render failed this project session. Poisoned so a
    /// failed background render is not attempted again on every repaint frame.
    failed_page_textures: HashSet<String>,
    /// Last PDF background error shown, to avoid spamming status every frame.
    bg_error: Option<String>,
    /// Session-only report export dialog state (no undo snapshot).
    report_export: Option<ReportExportUi>,
    /// Active colour theme (app-level preference, not project data).
    theme: Theme,
}

impl RubricApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let prefs = load_prefs();
        theme::apply_theme(&cc.egui_ctx, prefs.theme);
        let project = Project::new("Untitled Rubric Project");
        Self {
            history: History::new(project),
            ui: UiState::default(),
            draw: DrawState::Idle,
            last_page_pos: None,
            comment_text: String::new(),
            current_path: None,
            title_dirty: true,
            page_textures: HashMap::new(),
            failed_page_textures: HashSet::new(),
            bg_error: None,
            report_export: None,
            theme: prefs.theme,
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

    /// Shared reset used by New / Open / Import: installs a project into the
    /// session, clearing history, UI state, draw state, textures and clamping
    /// the current page. `current_path` is `Some` only for an opened file.
    fn install_project(&mut self, project: Project, current_path: Option<PathBuf>, status: String) {
        self.history = History::new(project);
        self.ui = UiState::default();
        self.draw = DrawState::Idle;
        self.current_path = current_path;
        self.ui.status = status;
        self.title_dirty = true;
        self.page_textures.clear();
        self.failed_page_textures.clear();
        self.bg_error = None;
        self.report_export = None;
        if let Some(doc) = self.active_doc() {
            let count = doc.page_count();
            if count > 0 && self.ui.current_page >= count {
                self.ui.current_page = count - 1;
            }
        }
    }

    /// Switch the colour theme, persist it as an app-level preference and
    /// repaint the shell with the new palette.
    fn set_theme(&mut self, ctx: &egui::Context, theme: Theme) {
        self.theme = theme;
        theme::apply_theme(ctx, theme);
        let prefs = AppPrefs { theme };
        match save_prefs(&prefs) {
            Ok(()) => self.ui.status = format!("Theme: {}", theme.label()),
            Err(e) => self.ui.status = format!("Theme changed, but saving failed: {e}"),
        }
    }

    /// Export the currently displayed page as a PNG image.
    fn export_page_image(&mut self, path: PathBuf) -> Result<(), DataError> {
        let mut last_page_pos: Option<Pt> = None;
        let project = self.history.current_mut();
        let bytes = match render_page_to_png_bytes(
            project,
            &self.ui,
            &self.draw,
            &mut self.page_textures,
            &mut self.failed_page_textures,
            &mut self.bg_error,
            &mut last_page_pos,
        ) {
            Ok(b) => b,
            Err(e) => {
                self.ui.status = format!("Export failed: {e}");
                return Err(e);
            }
        };
        match std::fs::write(&path, &bytes) {
            Ok(()) => {
                self.ui.status = format!("Page image exported: {}", path.display());
                Ok(())
            }
            Err(e) => {
                let err = DataError::Io(e);
                self.ui.status = format!("Export failed: {err}");
                Err(err)
            }
        }
    }

    /// Send the currently displayed page to the default printer.
    fn print_page(&mut self) -> Result<(), DataError> {
        let mut last_page_pos: Option<Pt> = None;
        let project = self.history.current_mut();
        let bytes = render_page_to_png_bytes(
            project,
            &self.ui,
            &self.draw,
            &mut self.page_textures,
            &mut self.failed_page_textures,
            &mut self.bg_error,
            &mut last_page_pos,
        )?;
        let temp_path =
            std::env::temp_dir().join(format!("rubric_print_{}.png", uuid::Uuid::new_v4()));
        std::fs::write(&temp_path, &bytes)?;
        match print_image(&temp_path) {
            Ok(()) => {
                self.ui.status = "Print job sent".to_string();
                Ok(())
            }
            Err(e) => {
                self.ui.status = format!("Print failed: {e}");
                Err(e)
            }
        }
    }

    /// Commit a pending in-canvas text edit (if any) using focus-loss semantics.
    /// Returns true when the text actually changed and was written back.
    fn commit_text_edit(&mut self) -> bool {
        canvas::commit_text_edit(&mut self.ui, self.history.current_mut())
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
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                self.install_project(proj, Some(path.clone()), format!("Opened {name}"));
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
                    self.install_project(
                        Project::new("Untitled Rubric Project"),
                        None,
                        "New project created".to_string(),
                    );
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
                if ui.button("Import PDF...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PDF document", &["pdf"])
                        .pick_file()
                    {
                        match crate::data::pdf::read_pdf_info(&path) {
                            Ok(info) => {
                                let stem = path
                                    .file_stem()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "Imported PDF".to_string());
                                let count = info.page_sizes.len().max(1);
                                let project = Project::from_pdf(
                                    stem,
                                    path.to_string_lossy().to_string(),
                                    &info.page_sizes,
                                );
                                self.install_project(
                                    project,
                                    None,
                                    format!("Imported {count} page(s)"),
                                );
                            }
                            Err(e) => {
                                self.ui.status = format!("Import failed: {e}");
                            }
                        }
                    }
                    ui.close();
                }
                if ui.button("Export Report...").clicked() {
                    let project = self.history.current();
                    let total_markups: usize =
                        project.documents.iter().map(|d| d.markups.len()).sum();
                    self.report_export = Some(ReportExportUi {
                        group_key: ReportGroupKey::Status,
                        format: ReportExportFormat::Text,
                        project_title: project.name.clone(),
                        total_markups,
                    });
                    ui.close();
                }
                if ui.button("Export Page Image...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("rubric-page.png")
                        .add_filter("PNG image", &["png"])
                        .save_file()
                    {
                        let _ = self.export_page_image(path);
                    }
                    ui.close();
                }
                if ui.button("Print...").clicked() {
                    let _ = self.print_page();
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
                ui.separator();
                ui.menu_button("Theme", |ui| {
                    for option in Theme::ALL {
                        let mut current = self.theme;
                        if ui
                            .radio_value(&mut current, option, option.label())
                            .clicked()
                            && current != self.theme
                        {
                            self.set_theme(ui.ctx(), current);
                        }
                    }
                });
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
                // Page navigation chrome.
                let count = self.active_doc().map(|d| d.page_count()).unwrap_or(0);
                if count > 0 {
                    let current = self.ui.current_page.min(count - 1);
                    if ui
                        .add_enabled(current + 1 < count, egui::Button::new("▶"))
                        .clicked()
                    {
                        let changed = self.commit_text_edit();
                        if changed {
                            self.push_snapshot_quiet();
                        }
                        self.ui.current_page = (current + 1).min(count - 1);
                        self.draw = DrawState::Idle;
                    }
                    ui.label(format!("{}/{}", current + 1, count));
                    if ui
                        .add_enabled(current > 0, egui::Button::new("◀"))
                        .clicked()
                    {
                        let changed = self.commit_text_edit();
                        if changed {
                            self.push_snapshot_quiet();
                        }
                        self.ui.current_page = current.saturating_sub(1);
                        self.draw = DrawState::Idle;
                    }
                }
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

    /// Switch the active tool, abandoning any in-flight draft gesture exactly
    /// like Escape does (but keeping the current selection). Used by the
    /// unmodified-key tool shortcuts.
    fn select_tool(&mut self, tool: Tool) {
        if self.ui.active_tool == tool {
            return;
        }
        self.ui.active_tool = tool;
        self.ui.draw_cancelled = true;
        self.draw = DrawState::Idle;
        self.ui.move_selection = false;
        let key = tool
            .shortcut_key()
            .map(|k| k.to_ascii_uppercase())
            .unwrap_or('-');
        self.ui.status = format!("{:?} ({key})", tool);
    }
}

/// Map an egui [`Key`] to the char used by [`Tool::shortcut_key`] /
/// [`Tool::from_shortcut_key`], or `None` when the key is not a tool shortcut.
///
/// Letters are the `Key::A`…`Key::Z` named variants (Shift is irrelevant:
/// we match the key, not the typed character). Digits are the top-row
/// `Key::Num1`…`Key::Num0` variants.
fn key_to_shortcut_char(key: egui::Key) -> Option<char> {
    match key {
        egui::Key::V => Some('v'),
        egui::Key::H => Some('h'),
        egui::Key::S => Some('s'),
        egui::Key::U => Some('u'),
        egui::Key::Q => Some('q'),
        egui::Key::T => Some('t'),
        egui::Key::C => Some('c'),
        egui::Key::N => Some('n'),
        egui::Key::M => Some('m'),
        egui::Key::R => Some('r'),
        egui::Key::O => Some('o'),
        egui::Key::D => Some('d'),
        egui::Key::P => Some('p'),
        egui::Key::Y => Some('y'),
        egui::Key::L => Some('l'),
        egui::Key::A => Some('a'),
        egui::Key::I => Some('i'),
        egui::Key::Num1 => Some('1'),
        egui::Key::Num2 => Some('2'),
        egui::Key::Num3 => Some('3'),
        egui::Key::Num4 => Some('4'),
        egui::Key::Num5 => Some('5'),
        egui::Key::Num6 => Some('6'),
        egui::Key::Num7 => Some('7'),
        _ => None,
    }
}

impl App for RubricApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Computed before `ctx.input` so the immutable borrow of `ctx` for
        // `wants_keyboard_input` does not overlap the input closure.
        let wants_kb = ctx.wants_keyboard_input();
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
            if i.key_pressed(egui::Key::PageUp) {
                // Never switch pages mid-edit: the edit context would be lost
                // and the text could be committed onto the wrong page.
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else if self.ui.current_page > 0 {
                    self.ui.current_page -= 1;
                    self.draw = DrawState::Idle;
                }
            }
            if i.key_pressed(egui::Key::PageDown) {
                // Never switch pages mid-edit: the edit context would be lost
                // and the text could be committed onto the wrong page.
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else {
                    let count = self.active_doc().map(|d| d.page_count()).unwrap_or(0);
                    if count > 0 && self.ui.current_page + 1 < count {
                        self.ui.current_page += 1;
                        self.draw = DrawState::Idle;
                    }
                }
            }
            if i.key_pressed(egui::Key::Escape) {
                if self.report_export.is_some() {
                    self.report_export = None;
                } else if self.ui.text_edit.is_some() {
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

            // Unmodified-key tool shortcuts (V = Select, R = Rectangle,
            // 1 = Measure Length, ...). Blocked while editing text, while any
            // widget wants the keyboard, or when Ctrl/Alt/Super is held (so
            // Ctrl+Z/O/S etc. keep working). Shift is allowed: we match the
            // egui `Key` variant, not the typed character.
            if !editing
                && !wants_kb
                && !i.modifiers.ctrl
                && !i.modifiers.alt
                && !i.modifiers.command
            {
                for event in &i.events {
                    if let egui::Event::Key {
                        key, pressed: true, ..
                    } = event
                    {
                        if let Some(c) = key_to_shortcut_char(*key) {
                            if let Some(tool) = Tool::from_shortcut_key(c) {
                                self.select_tool(tool);
                                break;
                            }
                        }
                    }
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
                &mut self.page_textures,
                &mut self.failed_page_textures,
                &mut self.bg_error,
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

        // Report export dialog (session state only).
        let mut do_export = false;
        let mut open = true;
        if let Some(state) = self.report_export.as_mut() {
            egui::Window::new("Export Report")
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Project: {}  ·  {} markups",
                        state.project_title, state.total_markups
                    ));
                    ui.separator();
                    egui::ComboBox::from_label("Group by")
                        .selected_text(state.group_key.as_str())
                        .show_ui(ui, |ui| {
                            for key in ReportGroupKey::ALL {
                                ui.selectable_value(&mut state.group_key, key, key.as_str());
                            }
                        });
                    ui.label("Format");
                    ui.radio_value(
                        &mut state.format,
                        ReportExportFormat::Text,
                        "Plain text (.txt)",
                    );
                    ui.radio_value(&mut state.format, ReportExportFormat::Json, "JSON (.json)");
                    if ui.button("Export…").clicked() {
                        do_export = true;
                    }
                });
        }
        if self.report_export.is_some() && !open {
            self.report_export = None;
        }
        if do_export {
            let export_info = self.report_export.as_ref().map(|s| (s.group_key, s.format));
            if let Some((key, format)) = export_info {
                let report = Report::from_project(self.history.current(), key);
                let (filter_name, ext, suggested) = match format {
                    ReportExportFormat::Text => ("Plain text", "txt", "rubric-report.txt"),
                    ReportExportFormat::Json => ("JSON", "json", "rubric-report.json"),
                };
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(filter_name, &[ext])
                    .set_file_name(suggested)
                    .save_file()
                {
                    let result = match format {
                        ReportExportFormat::Text => {
                            crate::data::export::write_report_text(&path, &report)
                        }
                        ReportExportFormat::Json => {
                            crate::data::export::write_report_json(&path, &report)
                        }
                    };
                    match result {
                        Ok(()) => {
                            self.ui.status = format!("Report exported: {}", path.display());
                            self.report_export = None;
                        }
                        Err(e) => {
                            self.ui.status = format!("Export failed: {e}");
                        }
                    }
                } else {
                    // Cancelled: close without claiming success.
                    self.report_export = None;
                }
            }
        }

        ui.ctx()
            .request_repaint_after(Duration::from_secs_f32(0.25));
    }
}
