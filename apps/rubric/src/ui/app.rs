//! The main egui application: layout, state, and high-level orchestration.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use eframe::{App, CreationContext, Frame};
use egui::{Align, Layout, Pos2, TextureHandle, Ui, Vec2};

use crate::data::{
    autosave, autosave_due, load_prefs, load_project, load_recent, normalise_project_filename,
    print_image, render_page_to_png_bytes, save_prefs, save_project, save_recent, DataError,
};
use crate::domain::units::calibrate_scale;
use crate::domain::{
    auto_save_interval_label, view, AppPrefs, Corner, Document, EditorAction, Grid, History,
    Markup, MarkupGroupKey, MarkupKind, MarkupListSort, MarkupStyle, MeasureUnit, Project, Pt,
    Report, ReportGroupKey, ReviewStatus, StampPreset, StylePreset, StylePresetLibrary, Theme,
    Tool, AUTO_SAVE_INTERVALS,
};
use crate::ui::{canvas, inspector, markups_list, menu, pages, theme, tool_chest, toolbar};
use uuid::Uuid;

/// Changelog embedded at build time for the in-app release-notes popup
/// (guide §9, §12.1). Baking it in means the popup works in a packaged binary,
/// where a source-checkout relative path would not resolve.
const EMBEDDED_CHANGELOG: &str = include_str!("../../../../CHANGELOG.md");

/// CI-managed token ledger embedded at build time (guide §12.1): cumulative
/// tokens since project start plus per-release token metrics.
const EMBEDDED_TOKENS: &str = include_str!("../../../../docs/tokens.json");

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
    /// Two-click page-scale calibration (v0.4 gap #4): `start` is set by the
    /// first click, `end` tracks the cursor until the second click commits
    /// the reference segment and opens the calibration dialog.
    Calibrate { start: Option<Pt>, end: Pt },
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

/// Layer criterion in the Markups List. The default layer is a distinct case
/// because markups on it store `layer_id: None` in the domain model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum MarkupLayerFilter {
    #[default]
    All,
    DefaultLayer,
    Layer(Uuid),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextEditState {
    pub(crate) id: Uuid,
    pub(crate) original: String,
    pub(crate) text: String,
    pub(crate) focus_requested: bool,
}

/// Session-only state for the calibration dialog (v0.4 gap #4): the drawn
/// reference segment plus the user-editable real-world distance it represents.
/// Exists only between the second click and Apply/Cancel; never serialized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CalibrationDialog {
    /// Page the reference segment was drawn on; its scale is rewritten on
    /// Apply (kept so a page switch with the dialog open stays honest).
    pub(crate) page: usize,
    /// Length of the drawn reference segment, in PDF points.
    pub(crate) segment_points: f64,
    /// Real-world distance of the segment. Pre-filled with what the page's
    /// *current* scale says the segment measures, so confirming unchanged
    /// leaves the scale as it is.
    pub(crate) real_units: f64,
    /// Display unit for [`CalibrationDialog::real_units`]; pre-filled from the
    /// page's current scale.
    pub(crate) unit: MeasureUnit,
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
    pub(crate) inspector_mode: InspectorMode,
    pub(crate) zoom: f32,
    pub(crate) pan: Vec2,
    /// Last known canvas viewport size (points), captured each frame in the
    /// central panel so Fit Page can scale to it.
    pub(crate) canvas_size: Vec2,
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
    /// Marquee drag start position in screen space (set when a marquee drag
    /// begins on empty canvas space). UI-only; never serialized.
    pub(crate) marquee_start: Option<Pos2>,
    /// In-canvas text editor state (Text/Callout markups only). UI-only, no
    /// serde: it never leaves the session.
    pub(crate) text_edit: Option<TextEditState>,
    /// Note pin whose floating popover is open (spec F-103). UI-only.
    pub(crate) note_popover: Option<Uuid>,
    /// Draft text in the note popover's comment box (spec F-103). UI-only.
    pub(crate) note_draft: String,
    /// A context-menu action requested this frame (spec F-603). The canvas and
    /// the inspector only *request* actions; `ui()` performs them so history
    /// snapshots and status text follow a single path.
    pub(crate) pending_action: Option<EditorAction>,
    /// Visual style captured by Copy Properties. Session-only; applying review
    /// status or lock state is deliberately not part of this clipboard.
    pub(crate) style_clipboard: Option<MarkupStyle>,
    /// Currently displayed page index (session state; never snapshotted).
    pub(crate) current_page: usize,
    /// True when a committed project change has not reached disk yet (save or
    /// autosave). Session-only; it drives autosave and the save path.
    dirty: bool,
    /// Custom scale editor inputs (paper inches and real units).
    pub(crate) scale_paper_inches: f64,
    pub(crate) scale_real_units: f64,
    /// Canvas grid / snap-to-grid settings (spec F-405). Session state: it
    /// describes how the canvas behaves, never project data, so it is not
    /// persisted into `.rubric.json`.
    pub(crate) grid: Grid,
    /// Whether the Markups List panel is visible (View menu toggle).
    pub(crate) show_markups_list: bool,
    /// Whether the Pages panel is visible (View menu toggle).
    pub(crate) show_pages: bool,
    /// Whether the Tool Chest panel is visible (View menu toggle, v0.4 gap
    /// #7). On by default so saved tools are discoverable.
    pub(crate) show_tool_chest: bool,
    /// Focus/reading mode: hides all side panels so only the canvas is
    /// visible, for distraction-free review (proposal P2-13).
    pub(crate) focus_mode: bool,
    /// Show a small minimap in the corner of the canvas showing page overview
    /// with viewport rectangle (proposal P2-16).
    pub(crate) show_minimap: bool,
    /// Pending quit confirmation when the project has unsaved changes.
    pub(crate) quit_confirm: bool,
    /// Per-frame history enablement snapshot, refreshed at the start of the
    /// frame; the shared context menu reads it to keep Undo/Redo honest.
    pub(crate) can_undo: bool,
    pub(crate) can_redo: bool,
    /// True when at least one selected markup is actually editable — gates
    /// Delete so the menu does not offer a no-op on a locked selection
    /// (full-app audit M7).
    pub(crate) can_delete: bool,
    /// View ▸ Full Screen session state (full-app audit M10).
    pub(crate) fullscreen: bool,
    /// Markups List filter controls. Kept as UI state so the domain filter can
    /// be rebuilt on every frame; none of it is written into `.rubric.json`.
    pub(crate) markup_search: String,
    pub(crate) markup_status: Option<ReviewStatus>,
    pub(crate) markup_author: Option<String>,
    pub(crate) markup_subject: Option<String>,
    pub(crate) markup_kind: Option<MarkupKind>,
    pub(crate) markup_page: Option<usize>,
    pub(crate) markup_layer: MarkupLayerFilter,
    pub(crate) markup_sort: MarkupListSort,
    pub(crate) markup_group: Option<MarkupGroupKey>,
    /// Layer-name edit drafts, keyed by layer id, so a rename in progress
    /// survives the per-frame inspector rebuild. Was a thread-local; moved here
    /// in the full-app audit M24 follow-up so the draft belongs to the session
    /// state rather than a process-global cell.
    pub(crate) layer_name_drafts: HashMap<uuid::Uuid, String>,
    /// Stroke colour chosen in the picker popup but not yet committed. The
    /// popup's button never reliably loses focus, so the actual commit is
    /// debounced by one frame; was a thread-local (full-app audit M24).
    pub(crate) colour_pending: Option<egui::Color32>,
    /// The page whose scale last re-synced the custom-ratio inputs. Used to
    /// detect a page change; was a thread-local (full-app audit M24).
    pub(crate) last_scale_page: Option<usize>,
    /// The selection-handle corner being dragged to resize the selected
    /// markup(s). Set on drag-start over a corner handle, cleared on drag end;
    /// UI-only, never serialized (v0.4 gap #5: resize grips).
    pub(crate) resize_corner: Option<Corner>,
    /// Index of the draggable endpoint/vertex being dragged to reshape the
    /// selected markup (see `Markup::draggable_points`). Set on drag-start
    /// over a point handle, cleared on drag end; UI-only, never serialized
    /// (v0.4 gap #5 follow-up: endpoint/vertex editing).
    pub(crate) resize_point: Option<usize>,
    /// True while a reshape drag (corner handle or point handle) has actually
    /// changed a markup. Cleared when the drag starts and when it ends; drives
    /// the "resize markups" snapshot so a zero-delta drag pushes nothing.
    pub(crate) resize_changed: bool,
    /// Show the release notes popup (guide §9): version history, app usage per
    /// release, token usage totals / per-release metrics.
    pub(crate) show_release_notes: bool,
    /// Open calibration dialog, if any (v0.4 gap #4). Set by the canvas when
    /// the user finishes the two-click reference segment; drawn as a window by
    /// the app shell. UI-only, never serialized.
    pub(crate) calibration: Option<CalibrationDialog>,
    /// Armed Tool Chest preset (v0.4 gap #7). While this is `Some`,
    /// `canvas::finish_markup` styles every non-stamp markup created from now
    /// on with the preset's look (`Markup::with_style`, style only: at
    /// creation there is no review verdict to overwrite and author/subject
    /// already come from the project settings / kind label). Toggled from the
    /// Tool Chest panel; cleared on project switch. UI-only, never serialized.
    pub(crate) pending_style_preset: Option<StylePreset>,
    /// Tool Chest "save as tool" name draft, so the text survives the
    /// per-frame panel rebuild. UI-only.
    pub(crate) tool_chest_save_name: String,
    /// Open Tool Chest rename draft: the preset id plus the editable name, so
    /// the editor survives the per-frame panel rebuild. UI-only.
    pub(crate) tool_chest_rename: Option<(uuid::Uuid, String)>,
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
            canvas_size: Vec2::ZERO,
            page_origin: Pos2::new(40.0, 40.0),
            hint: "Ctrl+S Save · Ctrl+O Open · Ctrl+Z Undo · Ctrl+Y Redo · Ctrl+A Select All · Del Delete · Esc Cancel"
                .to_string(),
            status: "Ready".to_string(),
            draw_cancelled: false,
            last_tool: Tool::Select,
            move_selection: false,
            move_changed: false,
            text_edit: None,
            note_popover: None,
            note_draft: String::new(),
            pending_action: None,
            style_clipboard: None,
            current_page: 0,
            dirty: false,
            scale_paper_inches: 1.0,
            scale_real_units: 10.0,
            grid: Grid::default(),
            marquee_start: None,
            show_markups_list: true,
            show_pages: true,
            show_tool_chest: true,
            focus_mode: false,
            show_minimap: false,
            quit_confirm: false,
            can_undo: false,
            can_redo: false,
            can_delete: false,
            fullscreen: false,
            markup_search: String::new(),
            markup_status: None,
            markup_author: None,
            markup_subject: None,
            markup_kind: None,
            markup_page: None,
            markup_layer: MarkupLayerFilter::All,
            markup_sort: MarkupListSort::default(),
            markup_group: None,
            layer_name_drafts: HashMap::new(),
            colour_pending: None,
            last_scale_page: None,
            resize_corner: None,
            resize_point: None,
            resize_changed: false,
            show_release_notes: false,
            calibration: None,
            pending_style_preset: None,
            tool_chest_save_name: String::new(),
            tool_chest_rename: None,
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
    /// A project install waiting for the user to confirm discarding unsaved
    /// changes (full-app audit B2). `None` when nothing is pending.
    pending_replace: Option<PendingReplace>,
    /// Help ▸ About window visibility (full-app audit M4).
    show_about: bool,
    /// Shows the keyboard shortcuts dialog (proposal 10).
    show_keyboard_shortcuts: bool,
    /// Set when the user closed the update-check window, so it stays closed
    /// until a new check is requested (full-app audit M8).
    update_dismissed: bool,
    /// Set by "Quit Anyway" so the close-request veto (audit M1) lets the
    /// explicitly-confirmed close through instead of re-prompting forever.
    force_close: bool,
    /// Set when the window title needs to be re-synced with `current_path`.
    title_dirty: bool,
    /// Instant of the most recent autosave **attempt** in this session, or
    /// `None` before the first one. Recording the attempt (even a failed one)
    /// rate-limits retries to one per interval instead of one per frame.
    last_autosave: Option<Instant>,
    /// Cache of rendered PDF page textures, keyed by cache PNG path.
    page_textures: HashMap<String, TextureHandle>,
    /// Cache keys whose page render failed this project session. Poisoned so a
    /// failed background render is not attempted again on every repaint frame.
    failed_page_textures: HashSet<String>,
    /// Page-thumbnail textures for the pages sidebar (spec F-404).
    thumb_cache: pages::ThumbCache,
    /// Last PDF background error shown, to avoid spamming status every frame.
    bg_error: Option<String>,
    /// Session-only report export dialog state (no undo snapshot).
    report_export: Option<ReportExportUi>,
    /// Active colour theme (app-level preference, not project data).
    theme: Theme,
    /// UI scale multiplier (app-level preference, not project data).
    ui_scale: f32,
    /// In-progress bug report the user is reviewing before sending (guide §7.1).
    report: Option<BugReportUi>,
    /// Shared update-check state, written by the worker thread and read by the
    /// UI each frame (guide §7.2).
    update: Arc<Mutex<UpdateCheck>>,
    /// Repository used for issue URLs and release checks.
    repo: String,
    /// Tool Chest preset library (v0.4 gap #7). App-level session state: it
    /// survives project switches (a tool chest belongs to the reviewer, not
    /// the project) and is not written into `.rubric.json` or prefs yet —
    /// `StylePreset`/`StylePresetLibrary` are serde-ready if persistence is
    /// added later. Not part of the undo history.
    tool_chest_library: StylePresetLibrary,
}

/// Update-check progress shared between the worker thread and the UI.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct UpdateCheck {
    /// A check has been started and has not finished yet.
    pub(crate) running: bool,
    /// Result of the most recent completed check.
    pub(crate) result: Option<update_check::UpdateStatus>,
}

/// A project install waiting on the "replace unsaved project?" confirmation
/// (full-app audit B2). New / Open / Import / Recent all funnel through
/// [`RubricApp::install_project`], which parks the install here while the
/// current project has unsaved changes.
#[derive(Debug, Clone, PartialEq)]
struct PendingReplace {
    project: Project,
    current_path: Option<PathBuf>,
    status: String,
}

/// Session-only bug report dialog state (guide §7.1): the captured report plus
/// the editable user notes. Never persisted, never uploaded without consent.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BugReportUi {
    pub(crate) report: crash_report::CrashReport,
    pub(crate) notes: String,
    /// User-supplied GitHub token; held in memory only, never written to disk.
    pub(crate) token: String,
    /// Use the browser fallback instead of a token submission.
    pub(crate) open_in_browser: bool,
}

impl BugReportUi {
    pub(crate) fn new(environment: crash_report::Environment, placeholder_summary: &str) -> Self {
        Self {
            report: crash_report::CrashReport::manual(environment, placeholder_summary),
            notes: String::new(),
            token: String::new(),
            open_in_browser: true,
        }
    }
}

impl RubricApp {
    pub fn new(cc: &CreationContext<'_>, parked_crash: Option<crash_report::CrashReport>) -> Self {
        let prefs = load_prefs();
        theme::apply_theme(&cc.egui_ctx, prefs.theme);
        cc.egui_ctx.set_pixels_per_point(prefs.ui_scale);
        let project = Project::new("Untitled Rubric Project");
        Self {
            pending_replace: None,
            history: History::new(project),
            ui: UiState {
                show_markups_list: prefs.show_markups_list,
                show_tool_chest: prefs.show_tool_chest,
                zoom: prefs.zoom,
                current_page: prefs.current_page,
                ..UiState::default()
            },
            draw: DrawState::Idle,
            last_page_pos: None,
            comment_text: String::new(),
            current_path: None,
            show_about: false,
            show_keyboard_shortcuts: false,
            update_dismissed: false,
            force_close: false,
            title_dirty: true,
            last_autosave: None,
            page_textures: HashMap::new(),
            failed_page_textures: HashSet::new(),
            thumb_cache: pages::ThumbCache::default(),
            bg_error: None,
            report_export: None,
            theme: prefs.theme,
            ui_scale: prefs.ui_scale,
            // A crash report parked by a previous run (audit M3) is surfaced in
            // the review-and-send dialog once; nothing is sent without consent.
            report: parked_crash.map(|report| BugReportUi {
                report,
                notes: String::new(),
                token: String::new(),
                open_in_browser: true,
            }),
            update: Arc::new(Mutex::new(UpdateCheck::default())),
            repo: "Perchemop/rubric-desktop".to_string(),
            tool_chest_library: prefs.tool_chest_library.clone(),
        }
    }

    /// Kick off an update check on a worker thread (guide §7.2). The UI loop is
    /// never blocked; the result arrives via the shared `update` state and the
    /// thread requests a repaint when done. `egui::Context` is cheap to clone,
    /// so the worker keeps its own handle for the repaint signal.
    fn begin_update_check(&mut self, ctx: &egui::Context) {
        self.update_dismissed = false;
        {
            let mut state = self.update.lock().unwrap_or_else(|p| p.into_inner());
            if state.running {
                return;
            }
            state.running = true;
            state.result = None;
        }
        let shared = Arc::clone(&self.update);
        let ctx = ctx.clone();
        let repo = self.repo.clone();
        let current = env!("CARGO_PKG_VERSION").to_string();
        std::thread::spawn(move || {
            let mut checker = update_check::UpdateChecker::new();
            let status = checker.check(&repo, &current);
            if let Ok(mut state) = shared.lock() {
                state.running = false;
                state.result = Some(status);
            }
            ctx.request_repaint();
        });
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
        // Selection is session state, not project data: clearing it must not
        // mark the project dirty or trigger an autosave of unchanged content.
        self.ui.selected.clear();
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

    /// Duplicate the selected markups, nudged slightly, and select the clones
    /// (Edit ▸ Duplicate, Ctrl+D; v0.4 gap #6).
    fn duplicate_selected(&mut self) {
        let ids: Vec<Uuid> = self.ui.selected.iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let new_ids = self
            .active_doc_mut()
            .map(|doc| doc.duplicate_markups(&ids, (12.0, 12.0)))
            .unwrap_or_default();
        if new_ids.is_empty() {
            self.ui.status = "Selection is locked".to_string();
        } else {
            self.ui.selected = new_ids.iter().copied().collect();
            self.push_snapshot("duplicate markups");
            let n = new_ids.len();
            self.ui.status = format!("Duplicated {n} markup{}", if n == 1 { "" } else { "s" });
        }
    }

    /// Perform a context-menu [`EditorAction`] (spec F-603). Shared by the
    /// canvas and inspector context menus and by the Edit/View menu bar so a
    /// command has exactly one implementation.
    fn apply_editor_action(&mut self, action: EditorAction) {
        match action {
            EditorAction::Undo => {
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else if self.history.undo() {
                    self.ui.status = "Undo".to_string();
                    // Undo/redo change the in-memory project, so it is unsaved.
                    self.ui.dirty = true;
                    self.clear_selection();
                }
            }
            EditorAction::Redo => {
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else if self.history.redo() {
                    self.ui.status = "Redo".to_string();
                    self.ui.dirty = true;
                    self.clear_selection();
                }
            }
            EditorAction::DeleteSelection => {
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else {
                    self.delete_selected();
                }
            }
            EditorAction::Duplicate => {
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else {
                    self.duplicate_selected();
                }
            }
            EditorAction::ClearSelection => {
                self.clear_selection();
                self.ui.status = "Selection cleared".to_string();
            }
            EditorAction::CopyProperties => self.copy_selected_style(),
            EditorAction::PasteProperties => self.paste_selected_style(),
            EditorAction::BringForward
            | EditorAction::SendBackward
            | EditorAction::BringToFront
            | EditorAction::SendToBack => self.apply_z_order(action),
            EditorAction::ZoomIn => {
                self.ui.zoom = (self.ui.zoom * 1.25).clamp(view::MIN_ZOOM, view::MAX_ZOOM);
            }
            EditorAction::ZoomOut => {
                self.ui.zoom = (self.ui.zoom / 1.25).clamp(view::MIN_ZOOM, view::MAX_ZOOM);
            }
            EditorAction::FitPage => {
                let (page_w, page_h) = self
                    .active_doc()
                    .and_then(|doc| doc.pages.get(self.ui.current_page))
                    .map(|p| (p.width.max(1.0), p.height.max(1.0)))
                    .unwrap_or((595.0, 842.0));
                let avail = self.ui.canvas_size;
                if avail.x > 0.0 && avail.y > 0.0 {
                    let margin = 40.0_f32;
                    let zx = (avail.x - margin) / page_w;
                    let zy = (avail.y - margin) / page_h;
                    self.ui.zoom = zx.min(zy).clamp(view::MIN_ZOOM, view::MAX_ZOOM);
                } else {
                    self.ui.zoom = 1.0;
                }
                self.ui.pan = Vec2::ZERO;
            }
            EditorAction::BatchStatus(status) => self.apply_batch_status(status),
        }
    }

    /// Set the canvas zoom level, clamped to valid bounds, and update status.
    fn set_zoom_level(&mut self, zoom: f32) {
        self.ui.zoom = zoom.clamp(view::MIN_ZOOM, view::MAX_ZOOM);
        self.ui.status = format!("Zoom {:.0}%", self.ui.zoom * 100.0);
        self.save_app_prefs();
    }

    /// Fit the current page to the canvas viewport (View ▸ Fit Page).
    fn fit_page(&mut self) {
        let (page_w, page_h) = self
            .active_doc()
            .and_then(|doc| doc.pages.get(self.ui.current_page))
            .map(|p| (p.width.max(1.0), p.height.max(1.0)))
            .unwrap_or((595.0, 842.0));
        let avail = self.ui.canvas_size;
        if avail.x > 0.0 && avail.y > 0.0 {
            let margin = 40.0_f32;
            let zx = (avail.x - margin) / page_w;
            let zy = (avail.y - margin) / page_h;
            self.ui.zoom = zx.min(zy).clamp(view::MIN_ZOOM, view::MAX_ZOOM);
        } else {
            self.ui.zoom = 1.0;
        }
        self.ui.pan = Vec2::ZERO;
        self.ui.status = "Fit page".to_string();
        self.save_app_prefs();
    }

    /// Set the review status of every selected editable markup to `status`.
    fn apply_batch_status(&mut self, status: ReviewStatus) {
        let ids: Vec<Uuid> = self.ui.selected.iter().copied().collect();
        if ids.is_empty() {
            self.ui.status = "Select at least one markup to change status".to_string();
            return;
        }
        let changed = self
            .active_doc_mut()
            .map(|doc| doc.apply_status_to(&ids, status))
            .unwrap_or(0);
        if changed == 0 {
            self.ui.status = "No editable markups selected".to_string();
        } else {
            self.push_snapshot("batch status");
            self.ui.status = format!(
                "Status set to {} for {} markup{}",
                status.label(),
                changed,
                if changed == 1 { "" } else { "s" }
            );
        }
    }

    /// Select every editable markup on the current page (Edit ▸ Select All,
    /// Ctrl+A; full-app audit M10). Shared by the menu entry and the keyboard
    /// shortcut so the two cannot diverge.
    fn select_all_on_page(&mut self) {
        let page = self.ui.current_page;
        let ids: Vec<Uuid> = self
            .active_doc()
            .map(|d| {
                d.markups
                    .iter()
                    .filter(|m| m.page == page && d.is_markup_editable(m))
                    .map(|m| m.id)
                    .collect()
            })
            .unwrap_or_default();
        self.ui.selected = ids.into_iter().collect();
        let n = self.ui.selected.len();
        self.ui.status = if n == 0 {
            "Nothing to select on this page".to_string()
        } else {
            format!("Selected {n} markup{}", if n == 1 { "" } else { "s" })
        };
    }

    /// The selected id when exactly one markup is selected.
    fn single_selected(&self) -> Option<Uuid> {
        let mut ids = self.ui.selected.iter();
        let id = *ids.next()?;
        ids.next().is_none().then_some(id)
    }

    /// Copy the selected markup's visual style into the session clipboard.
    fn copy_selected_style(&mut self) {
        let Some(id) = self.single_selected() else {
            self.ui.status = "Select exactly one markup to copy properties".to_string();
            return;
        };
        let style = self
            .active_doc()
            .and_then(|doc| doc.find_markup(id))
            .map(|markup| markup.style);
        match style {
            Some(style) => {
                self.ui.style_clipboard = Some(style);
                self.ui.status = "Properties copied".to_string();
            }
            None => self.ui.status = "Markup not found".to_string(),
        }
    }

    /// Apply the copied visual style to every selected editable markup.
    fn paste_selected_style(&mut self) {
        let Some(style) = self.ui.style_clipboard else {
            self.ui.status = "Copy properties first".to_string();
            return;
        };
        let ids: Vec<Uuid> = self.ui.selected.iter().copied().collect();
        if ids.is_empty() {
            self.ui.status = "Select at least one markup to paste properties".to_string();
            return;
        }
        let changed = match self.active_doc_mut() {
            Some(doc) => doc.apply_style_to(&ids, style),
            None => 0,
        };
        if changed == 0 {
            self.ui.status = "No editable markups selected".to_string();
        } else {
            self.push_snapshot("paste properties");
            self.ui.status = format!("Properties applied to {changed} markup(s)");
        }
    }

    /// Move the single selected markup in its page's paint order.
    fn apply_z_order(&mut self, action: EditorAction) {
        let Some(id) = self.single_selected() else {
            self.ui.status = "Select exactly one markup to change its order".to_string();
            return;
        };
        let result = match self.active_doc_mut() {
            Some(doc) => {
                let exists = doc.find_markup(id).is_some();
                let editable = doc
                    .find_markup(id)
                    .is_some_and(|markup| doc.is_markup_editable(markup));
                if !exists {
                    Err("Markup not found")
                } else if !editable {
                    Err("Selection is locked")
                } else {
                    Ok(match action {
                        EditorAction::BringForward => doc.bring_forward(id),
                        EditorAction::SendBackward => doc.send_backward(id),
                        EditorAction::BringToFront => doc.bring_to_front(id),
                        EditorAction::SendToBack => doc.send_to_back(id),
                        _ => false,
                    })
                }
            }
            None => Err("No active document"),
        };
        let (changed_message, edge_message) = match action {
            EditorAction::BringForward => ("Brought forward", "Already at the front"),
            EditorAction::SendBackward => ("Sent backward", "Already at the back"),
            EditorAction::BringToFront => ("Brought to front", "Already at the front"),
            EditorAction::SendToBack => ("Sent to back", "Already at the back"),
            _ => ("Order changed", "Already at that position"),
        };
        match result {
            Ok(true) => {
                self.push_snapshot(action.label());
                self.ui.status = changed_message.to_string();
            }
            Ok(false) => self.ui.status = edge_message.to_string(),
            Err(message) => self.ui.status = message.to_string(),
        }
    }

    /// Update-check status window (guide §7.2). Shows one honest state and a
    /// link out to the release page; never blocks and never auto-installs.
    fn draw_update_result(&mut self, ctx: &egui::Context) {
        if self.update_dismissed {
            return; // The user closed the window; keep it closed (audit M8).
        }
        let snapshot = self
            .update
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        if !snapshot.running && snapshot.result.is_none() {
            return; // Nothing to show until a check has been requested.
        }
        let mut open = true;
        egui::Window::new("Check for Updates")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                if snapshot.running {
                    ui.spinner();
                    ui.label("Checking GitHub Releases…");
                    return;
                }
                match snapshot.result.as_ref() {
                    Some(update_check::UpdateStatus::UpToDate) => {
                        ui.label(format!(
                            "Rubric {} is up to date.",
                            env!("CARGO_PKG_VERSION")
                        ));
                    }
                    Some(update_check::UpdateStatus::Available {
                        latest_version,
                        url,
                    }) => {
                        ui.label(format!(
                            "A newer release is available: {} (running {}).",
                            latest_version,
                            env!("CARGO_PKG_VERSION")
                        ));
                        if ui.button("Open release page").clicked() {
                            let _ = open_url(url);
                        }
                    }
                    Some(update_check::UpdateStatus::Unknown) => {
                        ui.label("Could not check for updates (offline or rate-limited).");
                        if ui.button("Try again").clicked() {
                            self.begin_update_check(ctx);
                        }
                    }
                    None => {}
                }
            });
        if !open {
            let mut state = self.update.lock().unwrap_or_else(|p| p.into_inner());
            state.result.take();
            // Keep the window closed even while the check is still running
            // (audit M8): previously `running` kept re-opening it every frame.
            self.update_dismissed = true;
        }
    }

    /// Bug report dialog (guide §7.1): review the captured report, add notes,
    /// then choose how it is delivered. Nothing is sent without consent, and a
    /// privileged token is never embedded in the app.
    fn draw_bug_report(&mut self, ctx: &egui::Context) {
        let Some(report_ui) = self.report.as_mut() else {
            return;
        };
        let mut open = true;
        let mut action: Option<BugReportAction> = None;
        egui::Window::new("Report a Bug")
            .open(&mut open)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.label(
                    "Everything below stays on this machine until you press Send. \
                     Review it — the backtrace has paths scrubbed, but you know \
                     your data best.",
                );
                ui.separator();
                ui.label("What happened (your words):");
                ui.text_edit_multiline(&mut report_ui.notes);
                ui.separator();
                ui.collapsing("Technical details (scrubbed)", |ui| {
                    ui.monospace(report_ui.report.to_issue_body());
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut report_ui.open_in_browser,
                        true,
                        "Open pre-filled issue in browser",
                    );
                    ui.radio_value(
                        &mut report_ui.open_in_browser,
                        false,
                        "Send with my GitHub token",
                    );
                });

                if !report_ui.open_in_browser {
                    ui.label(
                        "Your token is used once, in memory, and never saved. \
                         Leave this empty to use the browser instead.",
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut report_ui.token)
                            .password(true)
                            .hint_text("ghp_… (optional)"),
                    );
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        action = Some(BugReportAction::Cancel);
                    }
                    if ui.button("Send").clicked() {
                        action = Some(BugReportAction::Send);
                    }
                });
            });
        if !open {
            action = Some(BugReportAction::Cancel);
        }
        match action {
            Some(BugReportAction::Cancel) => {
                self.report = None;
                self.ui.status = "Bug report cancelled — nothing was sent".to_string();
            }
            Some(BugReportAction::Send) => {
                let report_ui = self.report.as_ref().expect("report open");
                let mut report = report_ui.report.clone();
                report.user_notes = report_ui.notes.clone();
                let issue = crash_report::Issue::from_report(&report);
                if report_ui.open_in_browser {
                    let url = issue.new_issue_url(&self.repo);
                    let _ = open_url(&url);
                    self.ui.status = "Opening the pre-filled issue in your browser".to_string();
                    self.report = None;
                } else {
                    let token = report_ui.token.clone();
                    let repo = self.repo.clone();
                    match crash_report::submit(&repo, &issue, Some(&token)) {
                        Ok(()) => {
                            self.ui.status = "Bug report sent — thank you".to_string();
                            self.report = None;
                        }
                        Err(e) => {
                            self.ui.status = format!("Could not send: {e}");
                            // Keep the dialog open so nothing the user typed is lost.
                        }
                    }
                }
            }
            None => {}
        }
    }

    /// Release-notes popup (guide §9, §12.1): version history, detailed app
    /// usage per release, and token usage totals / per-release metrics. The
    /// changelog and the CI-managed token ledger are **embedded at build time**
    /// ([`EMBEDDED_CHANGELOG`] / [`EMBEDDED_TOKENS`]) so the popup works in a
    /// packaged binary, not only in a source checkout.
    fn draw_release_notes(&mut self, ctx: &egui::Context) {
        if !self.ui.show_release_notes {
            return;
        }
        let mut open = true;
        egui::Window::new("Release Notes")
            .open(&mut open)
            .resizable(true)
            .default_width(700.0)
            .default_height(520.0)
            .show(ctx, |ui| {
                ui.heading("Rubric Release Notes");
                ui.separator();

                let current_version = env!("CARGO_PKG_VERSION");
                let ledger: serde_json::Value =
                    serde_json::from_str(EMBEDDED_TOKENS).unwrap_or_default();
                let total = ledger
                    .get("total_tokens_since_start")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                ui.horizontal(|ui| {
                    ui.label(format!("Current version: {current_version}"));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.small(format!("Cumulative tokens used: {total}"));
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Per-release token metrics from the ledger (guide §12.1).
                    if let Some(releases) = ledger.get("releases").and_then(|v| v.as_array()) {
                        let mut any = false;
                        for r in releases {
                            let v = r.get("release_version").and_then(|v| v.as_str());
                            let t = r.get("release_tokens").and_then(|v| v.as_u64());
                            if let (Some(v), Some(t)) = (v, t) {
                                if !any {
                                    ui.strong("Token usage per release");
                                    any = true;
                                }
                                ui.small(format!("{v}: {t} tokens"));
                            }
                        }
                        if any {
                            ui.separator();
                        }
                    }

                    // Changelog sections for the current version and Unreleased.
                    let mut shown = false;
                    for line in EMBEDDED_CHANGELOG.lines() {
                        let trimmed = line.trim_start();
                        if trimmed.starts_with("## [") {
                            if shown
                                && !trimmed.contains(&format!("[{current_version}]"))
                                && !trimmed.contains("[Unreleased]")
                            {
                                break; // Stop at the next version section
                            }
                            if trimmed.contains(&format!("[{current_version}]"))
                                || trimmed.contains("[Unreleased]")
                            {
                                shown = true;
                            }
                        }
                        if shown {
                            if trimmed.starts_with("## [") {
                                ui.add_space(8.0);
                                ui.strong(line.trim_start_matches('#').trim());
                            } else {
                                ui.label(line);
                            }
                        }
                    }
                    if !shown {
                        ui.label("No release notes found for this version.");
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Check for Updates…").clicked() {
                        self.begin_update_check(ctx);
                    }
                    if ui.button("View on GitHub").clicked() {
                        let _ = open_url(&format!("https://github.com/{}/releases", self.repo));
                    }
                    if ui.button("Close").clicked() {
                        self.ui.show_release_notes = false;
                    }
                });
            });
        if !open {
            self.ui.show_release_notes = false;
        }
    }

    /// Close the calibration dialog without changing any scale. No snapshot:
    /// nothing was committed.
    fn cancel_calibration(&mut self) {
        if self.ui.calibration.take().is_some() {
            self.ui.status = "Calibration cancelled".to_string();
            self.draw = DrawState::Idle;
        }
    }

    /// Apply a confirmed calibration (v0.4 gap #4): write the new scale to the
    /// calibrated page through the same write path the Scale inspector uses
    /// (page scale + recompute that page's measurements from their stored raw
    /// points), then push exactly one undo snapshot.
    fn apply_calibration(&mut self, dialog: CalibrationDialog) {
        if !dialog.real_units.is_finite() || dialog.real_units <= 0.0 {
            // Rejected: keep the dialog open so the typed value can be fixed.
            self.ui.calibration = Some(dialog);
            self.ui.status = "Scale value must be positive".to_string();
            return;
        }
        self.ui.calibration = None;
        let new_scale =
            calibrate_scale(dialog.segment_points as f32, dialog.real_units, dialog.unit);
        if let Some(doc) = self.history.current_mut().active_mut() {
            if let Some(page) = doc.pages.get_mut(dialog.page) {
                page.scale = new_scale;
            }
            let scale = doc
                .pages
                .get(dialog.page)
                .map(|p| p.scale)
                .unwrap_or(new_scale);
            for m in doc.markups.iter_mut().filter(|m| m.page == dialog.page) {
                m.recompute_measurement(scale);
            }
            doc.touch();
        }
        self.ui.status = format!("Scale calibrated → {}", new_scale.describe());
        self.push_snapshot_quiet();
        self.draw = DrawState::Idle;
    }

    /// Calibration dialog (v0.4 gap #4): confirm the real-world distance of
    /// the drawn reference segment. Pre-filled with the segment's length under
    /// the page's current scale, so confirming unchanged is a no-op; changing
    /// the value (or unit) recalibrates the page.
    fn draw_calibration_dialog(&mut self, ctx: &egui::Context) {
        let Some(state) = self.ui.calibration.as_mut() else {
            return;
        };
        let mut open = true;
        let mut cancelled = false;
        let mut apply = false;
        egui::Window::new("Calibrate Page Scale")
            .open(&mut open)
            .resizable(false)
            .default_width(560.0)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Drawn segment: {:.1} pt on page {}",
                    state.segment_points,
                    state.page + 1
                ));
                ui.separator();
                ui.label("How long is this distance in the real world?");
                ui.horizontal(|ui| {
                    let _ = ui.add(
                        egui::DragValue::new(&mut state.real_units)
                            .speed(0.1)
                            .range(0.001..=1_000_000.0),
                    );
                    egui::ComboBox::from_label("Unit")
                        .selected_text(state.unit.name())
                        .show_ui(ui, |ui| {
                            for unit in MeasureUnit::ALL {
                                ui.selectable_value(&mut state.unit, unit, unit.name());
                            }
                        });
                });
                ui.small(
                    "Measurements on this page are re-read from the new scale. \
                     Ctrl+Z restores the previous one.",
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Calibrate").clicked() {
                        apply = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancelled = true;
                    }
                });
            });
        if !open || cancelled {
            self.cancel_calibration();
        } else if apply {
            if let Some(dialog) = self.ui.calibration {
                self.apply_calibration(dialog);
            }
        }
    }

    fn push_snapshot(&mut self, reason: &str) {
        let proj = self.history.current().clone();
        self.history.push(proj);
        // A committed change is exactly what auto-save should persist.
        self.ui.dirty = true;
        self.ui.status = format!("Snapshot: {reason}");
    }

    /// Push a snapshot without touching the status bar. Used when the canvas
    /// already set a more specific status earlier in the same frame.
    fn push_snapshot_quiet(&mut self) {
        let proj = self.history.current().clone();
        self.history.push(proj);
        self.ui.dirty = true;
    }

    /// Shared reset used by New / Open / Import: installs a project into the
    /// session, clearing history, UI state, draw state, textures and clamping
    /// the current page. `current_path` is `Some` only for an opened file.
    ///
    /// While the current project has unsaved changes the install is parked in
    /// [`Self::pending_replace`] behind a confirmation, so New / Open / Import
    /// / Recent can no longer discard work silently (full-app audit B2).
    /// Returns `true` when the install happened now, `false` when it was
    /// parked behind the confirmation.
    fn install_project(
        &mut self,
        project: Project,
        current_path: Option<PathBuf>,
        status: String,
    ) -> bool {
        if self.ui.dirty {
            self.pending_replace = Some(PendingReplace {
                project,
                current_path,
                status,
            });
            self.ui.status =
                "Unsaved changes — save first (Ctrl+S) or confirm to replace".to_string();
            return false;
        }
        self.install_project_now(project, current_path, status);
        true
    }

    /// The unconditional install (confirmation already resolved).
    fn install_project_now(
        &mut self,
        project: Project,
        current_path: Option<PathBuf>,
        status: String,
    ) {
        self.history = History::new(project);
        self.ui = UiState::default();
        self.draw = DrawState::Idle;
        self.current_path = current_path;
        self.ui.status = status;
        self.title_dirty = true;
        // A freshly installed project has never been autosaved this session.
        self.last_autosave = None;
        self.page_textures.clear();
        self.failed_page_textures.clear();
        self.thumb_cache = pages::ThumbCache::default();
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
        let prefs = AppPrefs {
            theme,
            show_markups_list: self.ui.show_markups_list,
            show_tool_chest: self.ui.show_tool_chest,
            show_pages: self.ui.show_pages,
            zoom: self.ui.zoom,
            current_page: self.ui.current_page,
            tool_chest_library: self.tool_chest_library.clone(),
            ui_scale: self.ui_scale,
            focus_mode: self.ui.focus_mode,
            show_minimap: self.ui.show_minimap,
        };
        match save_prefs(&prefs) {
            Ok(()) => self.ui.status = format!("Theme: {}", theme.label()),
            Err(e) => self.ui.status = format!("Theme changed, but saving failed: {e}"),
        }
    }

    /// Persist all app-level preferences (panel visibility, zoom, page).
    fn save_app_prefs(&mut self) {
        let prefs = AppPrefs {
            theme: self.theme,
            show_markups_list: self.ui.show_markups_list,
            show_tool_chest: self.ui.show_tool_chest,
            show_pages: self.ui.show_pages,
            zoom: self.ui.zoom,
            current_page: self.ui.current_page,
            tool_chest_library: self.tool_chest_library.clone(),
            ui_scale: self.ui_scale,
            focus_mode: self.ui.focus_mode,
            show_minimap: self.ui.show_minimap,
        };
        if let Err(e) = save_prefs(&prefs) {
            self.ui.status = format!("Workspace saved, but preferences write failed: {e}");
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
                self.ui.status = format!("Print failed: {e}");
                return Err(e);
            }
        };
        let temp_path =
            std::env::temp_dir().join(format!("rubric_print_{}.png", uuid::Uuid::new_v4()));
        if let Err(e) = std::fs::write(&temp_path, &bytes) {
            let err = DataError::Io(e);
            self.ui.status = format!("Print failed: {err}");
            return Err(err);
        }
        let result = print_image(&temp_path);
        // Don't leak the temporary PNG, regardless of print outcome.
        let _ = std::fs::remove_file(&temp_path);
        match result {
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

    /// Flatten the currently displayed page — with its visible markups — into
    /// a new single-page PDF (File ▸ Export Page to PDF...).
    ///
    /// Paint order matches the canvas, so the flattened page shows exactly
    /// what the user sees; hidden markups are intentionally left out. The
    /// save dialog runs first, and cancelling it is not an error: the status
    /// line stays untouched.
    fn export_current_page_to_pdf(&mut self) {
        let Some(doc) = self.active_doc() else {
            self.ui.status = "Export failed: no document is open".to_string();
            return;
        };
        let page = self.ui.current_page;
        let markups: Vec<Markup> = doc
            .markups_in_paint_order(page)
            .into_iter()
            .cloned()
            .collect();
        let source_pdf = self
            .history
            .current()
            .source_pdf
            .clone()
            .unwrap_or_default();
        if source_pdf.is_empty() {
            self.ui.status = "Export failed: import a PDF before flattening a page".to_string();
            return;
        }
        let default_name = format!("rubric-page-{}.pdf", page + 1);
        let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .add_filter("PDF document", &["pdf"])
            .save_file()
        else {
            return;
        };
        match crate::data::export::export_page_to_pdf(Path::new(&source_pdf), page, &path, &markups)
        {
            Ok(()) => {
                self.ui.status = format!("Flattened page {} to PDF: {}", page + 1, path.display());
            }
            Err(e) => {
                self.ui.status = format!("Export failed: {e}");
            }
        }
    }

    /// Flatten every page of the source PDF — with its markups — into a new
    /// PDF (File ▸ Export All Pages to PDF...).
    fn export_all_pages_to_pdf_action(&mut self) {
        let source_pdf = self
            .history
            .current()
            .source_pdf
            .clone()
            .unwrap_or_default();
        if source_pdf.is_empty() {
            self.ui.status = "Export failed: import a PDF before flattening pages".to_string();
            return;
        }
        let Some(doc) = self.active_doc() else {
            self.ui.status = "Export failed: no document is open".to_string();
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("rubric-flattened.pdf")
            .add_filter("PDF document", &["pdf"])
            .save_file()
        else {
            return;
        };
        match crate::data::export::export_all_pages_to_pdf(
            Path::new(&source_pdf),
            &path,
            &doc.markups,
        ) {
            Ok(()) => {
                self.ui.status = format!("Flattened all pages to PDF: {}", path.display());
            }
            Err(e) => {
                self.ui.status = format!("Export failed: {e}");
            }
        }
    }

    /// Commit a pending in-canvas text edit (if any) using focus-loss semantics.
    /// Returns true when the text actually changed and was written back.
    fn commit_text_edit(&mut self) -> bool {
        canvas::commit_text_edit(&mut self.ui, self.history.current_mut())
    }

    /// Switch to another page (spec F-404), committing any in-flight text edit
    /// first. Shared by the top-bar arrows, Page Up/Down and the pages sidebar,
    /// so all three agree on snapshot and draft-abandon semantics.
    fn jump_to_page(&mut self, index: usize) -> bool {
        let count = self.active_doc().map(|d| d.page_count()).unwrap_or(0);
        if count == 0 || index >= count || index == self.ui.current_page {
            return false;
        }
        if self.commit_text_edit() {
            self.push_snapshot_quiet();
        }
        self.ui.current_page = index;
        self.draw = DrawState::Idle;
        self.save_app_prefs();
        true
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
                let installed =
                    self.install_project(proj, Some(path.clone()), format!("Opened {name}"));
                // Record the open in the app-level recents store only when the
                // project actually replaced the session (audit B2: a deferred
                // install behind the unsaved-changes confirm is not an open).
                // A failed save (e.g. read-only config dir) must not fail the
                // open.
                if installed {
                    let mut recent = load_recent();
                    recent.push(path.to_string_lossy().to_string());
                    let _ = save_recent(&recent);
                }
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

    /// Run an autosave when the project is dirty and its interval has elapsed
    /// (spec F-502). The write is small and synchronous on the UI thread; the
    /// schedule is rate-limited so a failed attempt retries at the next
    /// interval rather than every frame.
    fn maybe_autosave(&mut self) {
        let settings = &self.history.current().settings;
        if !autosave_due(
            self.ui.dirty,
            settings.auto_save,
            settings.auto_save_interval_secs,
            self.last_autosave,
            Instant::now(),
        ) {
            return;
        }
        // Record the attempt first so a failure cannot busy-loop the loop.
        self.last_autosave = Some(Instant::now());
        let path = self.current_path.clone();
        match autosave(self.history.current(), path.as_deref()) {
            Ok(Some(path)) => {
                // Audit M2: the autosave writes the *recovery sidecar*
                // (`<stem>.autosave.rubric.json`), not the project file, so the
                // project stays dirty — clearing the flag here suppressed the
                // Quit confirmation while the real `.rubric.json` was stale.
                // Only `save_project_to` may clear `dirty`.
                self.ui.status = format!("Recovery snapshot saved: {}", path.display());
            }
            Ok(None) => {
                // No writable location (no project path and no data dir). Leave
                // the project dirty; File → Save As resolves it for good.
            }
            Err(e) => {
                self.ui.status = format!("Autosave failed: {e}");
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
                if menu_shortcut_button(ui, "Open Project...", "Ctrl+O") {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Rubric project", &["rubric.json"])
                        .pick_file()
                    {
                        self.open_project_path(path);
                    }
                    ui.close();
                }
                if menu_shortcut_button(ui, "Import PDF…", "Ctrl+I") {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PDF document", &["pdf"])
                        .pick_file()
                    {
                        match crate::data::pdf::read_pdf_info(&path) {
                            // Audit M9: a 0-page PDF used to report "Imported
                            // 1 page(s)" and fabricate an unrenderable page.
                            Ok(info) if info.page_sizes.is_empty() => {
                                self.ui.status = "Import failed: the PDF has no pages".to_string();
                            }
                            Ok(info) => {
                                let stem = path
                                    .file_stem()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "Imported PDF".to_string());
                                let count = info.page_sizes.len();
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
                if ui.button("Export PDF...").clicked() {
                    let source_pdf = self
                        .history
                        .current()
                        .source_pdf
                        .clone()
                        .unwrap_or_default();
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("rubric-export.pdf")
                        .add_filter("PDF document", &["pdf"])
                        .save_file()
                    {
                        if !source_pdf.is_empty() {
                            if let Some(doc) = self.active_doc() {
                                match crate::data::pdf::export_pdf_to(
                                    Path::new(&source_pdf),
                                    &path,
                                    &doc.markups,
                                ) {
                                    Ok(()) => {
                                        self.ui.status =
                                            format!("PDF exported to {}", path.display());
                                    }
                                    Err(e) => {
                                        self.ui.status = format!("PDF export failed: {e}");
                                    }
                                }
                            }
                        }
                    }
                    ui.close();
                }
                // Flatten-to-PDF (gap #1): a path from markups back into a
                // PDF. Both items need the imported source PDF — without one
                // there is no page content to flatten the markups onto — so
                // they are disabled until a PDF is imported.
                let has_source_pdf = self
                    .history
                    .current()
                    .source_pdf
                    .as_deref()
                    .is_some_and(|p| !p.is_empty());
                let export_page_pdf = ui
                    .add_enabled(has_source_pdf, egui::Button::new("Export Page to PDF..."))
                    .on_disabled_hover_text("Import a PDF before flattening a page");
                if export_page_pdf.clicked() {
                    self.export_current_page_to_pdf();
                    ui.close();
                }
                let export_all_pdf = ui
                    .add_enabled(
                        has_source_pdf,
                        egui::Button::new("Export All Pages to PDF..."),
                    )
                    .on_disabled_hover_text("Import a PDF before flattening pages");
                if export_all_pdf.clicked() {
                    self.export_all_pages_to_pdf_action();
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
                            // Report the real outcome: don't claim success when
                            // the recents file could not be rewritten (audit M2).
                            self.ui.status = match save_recent(&recent) {
                                Ok(()) => "Recent list cleared".to_string(),
                                Err(e) => format!("Could not clear the recent list: {e}"),
                            };
                            ui.close();
                        }
                    }
                });
                if menu_shortcut_button(ui, "Save Project", "Ctrl+S") {
                    self.save_project_action();
                    ui.close();
                }
                if ui.button("Save Project As...").clicked() {
                    self.save_project_as_action();
                    ui.close();
                }
                // Autosave controls (spec F-502). These edit `Settings`, which
                // is project data, so each change pushes one undo snapshot.
                ui.menu_button("Auto-save", |ui| {
                    let mut enabled = self.history.current().settings.auto_save;
                    if ui.checkbox(&mut enabled, "Enable Auto-save").changed() {
                        self.history.current_mut().settings.auto_save = enabled;
                        self.ui.status = if enabled {
                            "Auto-save on".to_string()
                        } else {
                            "Auto-save off".to_string()
                        };
                        self.push_snapshot_quiet();
                    }
                    ui.separator();
                    let current = self.history.current().settings.auto_save_interval_secs;
                    // Keep a non-preset persisted interval honest by showing it.
                    let mut options = AUTO_SAVE_INTERVALS.to_vec();
                    if !options.contains(&current) {
                        options.insert(0, current);
                    }
                    let mut chosen = current;
                    for secs in options {
                        ui.selectable_value(&mut chosen, secs, auto_save_interval_label(secs));
                    }
                    if chosen != current {
                        self.history.current_mut().settings.auto_save_interval_secs = chosen;
                        self.ui.status =
                            format!("Auto-save every {}", auto_save_interval_label(chosen));
                        self.push_snapshot_quiet();
                    }
                });
                ui.separator();
                if ui.button("Quit").clicked() {
                    if self.ui.dirty {
                        self.ui.quit_confirm = true;
                    } else {
                        // Let eframe tear down normally instead of exiting the
                        // process outright.
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui
                    .add_enabled(
                        self.history.can_undo(),
                        egui::Button::new("Undo").shortcut_text("Ctrl+Z"),
                    )
                    .clicked()
                {
                    self.apply_editor_action(EditorAction::Undo);
                    ui.close();
                }
                if ui
                    .add_enabled(
                        self.history.can_redo(),
                        egui::Button::new("Redo").shortcut_text("Ctrl+Y"),
                    )
                    .clicked()
                {
                    self.apply_editor_action(EditorAction::Redo);
                    ui.close();
                }
                ui.separator();
                // Select All (full-app audit M10): select every editable,
                // visible markup on the current page — the delete and
                // apply-style paths already operate on the whole selection.
                if menu_shortcut_button(ui, "Select All", "Ctrl+A") {
                    self.select_all_on_page();
                    ui.close();
                }
                ui.separator();
                menu::draw_selection_commands(ui, &mut self.ui);
            });
            ui.menu_button("View", |ui| {
                if menu_shortcut_button(ui, "Zoom In", "Ctrl++") {
                    self.set_zoom_level(self.ui.zoom * 1.25);
                    ui.close();
                }
                if menu_shortcut_button(ui, "Zoom Out", "Ctrl+-") {
                    self.set_zoom_level(self.ui.zoom * 0.8);
                    ui.close();
                }
                if menu_shortcut_button(ui, "Fit Page", "Ctrl+0") {
                    self.fit_page();
                    ui.close();
                }
                // Drag calibration (v0.4 gap #4): selecting the tool starts a
                // two-click reference segment on the canvas; the dialog that
                // follows writes the page scale.
                if ui.button("Calibrate Page Scale…").clicked() {
                    self.select_tool(Tool::Calibrate);
                    ui.close();
                }
                ui.separator();
                // Page navigation in the menu (full-app audit M10): the arrows
                // and PageUp/PageDown were the only paths before.
                if menu_shortcut_button(ui, "Previous Page", "PageUp") {
                    self.jump_to_page(self.ui.current_page.saturating_sub(1));
                }
                if menu_shortcut_button(ui, "Next Page", "PageDown") {
                    let count = self.active_doc().map(|d| d.page_count()).unwrap_or(0);
                    if self.ui.current_page + 1 < count {
                        self.jump_to_page(self.ui.current_page + 1);
                    }
                }
                if ui.button("Full Screen").clicked() {
                    // Read the live state so the toggle is truthful even if the
                    // WM changed it out from under us.
                    let fullscreen = ui
                        .ctx()
                        .input(|i| i.viewport().fullscreen)
                        .unwrap_or(self.ui.fullscreen);
                    self.ui.fullscreen = !fullscreen;
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
                    self.ui.status = if self.ui.fullscreen {
                        "Full screen on (Esc-exit via the View menu)".to_string()
                    } else {
                        "Full screen off".to_string()
                    };
                }
                ui.separator();
                for pct in view::ZOOM_PRESETS {
                    let zoom = pct as f32 / 100.0;
                    if ui.button(format!("{}%", pct)).clicked() {
                        self.ui.zoom = view::clamp_zoom(zoom);
                        self.ui.status = format!("Zoom {}%", pct);
                    }
                }
                ui.separator();
                // Canvas guides (F-405): grid visibility, snapping and pitch.
                // Grid lines are only drawn when the pitch resolves at the
                // current zoom, so the tick stays honest even when off.
                let mut grid_visible = self.ui.grid.visible;
                if ui.checkbox(&mut grid_visible, "Show Grid").changed() {
                    self.ui.grid.visible = grid_visible;
                }
                if ui
                    .checkbox(&mut self.ui.grid.snap, "Snap to Grid")
                    .clicked()
                {
                    self.ui.status = if self.ui.grid.snap {
                        "Snap to grid on".to_string()
                    } else {
                        "Snap to grid off".to_string()
                    };
                }
                ui.menu_button("Grid Spacing", |ui| {
                    let mut spacing = self.ui.grid.spacing;
                    for s in Grid::SPACINGS {
                        if ui
                            .selectable_value(&mut spacing, s, format!("{s} pt"))
                            .clicked()
                        {
                            self.ui.grid.spacing = s;
                            self.ui.status = format!("Grid spacing → {s} pt");
                        }
                    }
                });
                ui.separator();
                {
                    let mut show_measurements = self.history.current().settings.show_measurements;
                    if ui
                        .checkbox(&mut show_measurements, "Show Measurements")
                        .changed()
                    {
                        self.history.current_mut().settings.show_measurements = show_measurements;
                        self.ui.status = if show_measurements {
                            "Measurements shown".to_string()
                        } else {
                            "Measurements hidden".to_string()
                        };
                        self.push_snapshot_quiet();
                    }
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
                ui.separator();
                if ui
                    .checkbox(&mut self.ui.show_tool_chest, "Show Tool Chest")
                    .changed()
                {
                    self.ui.status = if self.ui.show_tool_chest {
                        "Tool Chest shown".to_string()
                    } else {
                        "Tool Chest hidden".to_string()
                    };
                    self.save_app_prefs();
                }
                if ui.checkbox(&mut self.ui.focus_mode, "Focus Mode").changed() {
                    self.ui.status = if self.ui.focus_mode {
                        "Focus mode on — panels hidden".to_string()
                    } else {
                        "Focus mode off".to_string()
                    };
                    self.save_app_prefs();
                }
                if ui
                    .checkbox(&mut self.ui.show_markups_list, "Show Markups List")
                    .changed()
                {
                    self.ui.status = if self.ui.show_markups_list {
                        "Markups List shown".to_string()
                    } else {
                        "Markups List hidden".to_string()
                    };
                    self.save_app_prefs();
                }
                if ui.checkbox(&mut self.ui.show_pages, "Show Pages").changed() {
                    self.ui.status = if self.ui.show_pages {
                        "Pages panel shown".to_string()
                    } else {
                        "Pages panel hidden".to_string()
                    };
                    self.save_app_prefs();
                }
                if ui
                    .checkbox(&mut self.ui.show_minimap, "Show Minimap")
                    .changed()
                {
                    self.ui.status = if self.ui.show_minimap {
                        "Minimap shown".to_string()
                    } else {
                        "Minimap hidden".to_string()
                    };
                    self.save_app_prefs();
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("About Rubric").clicked() {
                    self.show_about = !self.show_about;
                    ui.close();
                }
                if ui.button("Check for Updates…").clicked() {
                    self.ui.status = "Checking for updates…".to_string();
                    self.begin_update_check(ui.ctx());
                    ui.close();
                }
                if ui.button("Report a Bug…").clicked() {
                    self.report = Some(BugReportUi::new(
                        crash_report::Environment::capture(env!("CARGO_PKG_VERSION")),
                        "describe what you were doing when the problem happened",
                    ));
                    ui.close();
                }
                ui.separator();
                if ui.button("Keyboard Shortcuts…").clicked() {
                    self.show_keyboard_shortcuts = !self.show_keyboard_shortcuts;
                    ui.close();
                }
            });
        });
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            toolbar::draw_toolbar(ui, &mut self.ui.active_tool, &mut self.ui.status);

            // Active-tool chip (P0-4): a persistent pill showing what tool is
            // currently active, with its shortcut key. Always visible so the
            // user never has to hunt through the left panel to know the mode.
            ui.separator();
            egui::Frame::new()
                .fill(ui.visuals().selection.bg_fill)
                .stroke(ui.visuals().selection.stroke)
                .show(ui, |ui| {
                    let tool_name = self
                        .ui
                        .active_tool
                        .to_kind()
                        .map(|k| k.label())
                        .unwrap_or("Select");
                    let shortcut = self
                        .ui
                        .active_tool
                        .shortcut_key()
                        .map(|k| format!(" ({})", k.to_ascii_uppercase()))
                        .unwrap_or_default();
                    ui.label(format!("{}{}", tool_name, shortcut));
                });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(self.ui.status.clone());
                ui.label(format!("{:.0}%", self.ui.zoom * 100.0));
                // Page navigation chrome.
                let count = self.active_doc().map(|d| d.page_count()).unwrap_or(0);
                if count > 0 {
                    let current = self.ui.current_page.min(count - 1);
                    if ui
                        .add_enabled(current > 0, egui::Button::new("◀"))
                        .on_hover_text("Previous page")
                        .clicked()
                    {
                        self.jump_to_page(current.saturating_sub(1));
                    }
                    ui.label(format!("Page {}/{}", current + 1, count));
                    if ui
                        .add_enabled(current + 1 < count, egui::Button::new("▶"))
                        .on_hover_text("Next page")
                        .clicked()
                    {
                        self.jump_to_page(current + 1);
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
        let doc_title = self.active_doc().map(|d| d.title.clone());
        let page_count = self.active_doc().map(|d| d.page_count());

        ui.horizontal(|ui| {
            ui.small(self.ui.hint.clone());
            if let Some(p) = self.last_page_pos {
                ui.small(format!("Cursor: {:.1},{:.1} pt", p.x, p.y));
            }
            if let (Some(pages), Some(title)) = (page_count, doc_title) {
                let plural = if pages == 1 { "" } else { "s" };
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Version label (guide §9): subtle, non-intrusive, clickable for release notes.
                    if ui
                        .small(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .on_hover_text("Click for release notes")
                        .clicked()
                    {
                        self.ui.show_release_notes = true;
                    }
                    ui.small(format!("{pages} page{plural}"));
                    ui.small(title);
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
        self.ui.status = format!("{} ({key})", tool.label());
    }
}

/// What the user chose in the bug-report dialog this frame.
enum BugReportAction {
    Cancel,
    Send,
}

/// Open a URL with the platform opener (browser for issue/release links).
fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("could not open browser: {e}"))
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("could not open browser: {e}"))
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("could not open browser: {e}"))
    }
}

/// A menu button that renders its keyboard shortcut on the right (full-app
/// audit M5). Returns true when clicked.
fn menu_shortcut_button(ui: &mut Ui, label: &str, shortcut: &str) -> bool {
    ui.add(egui::Button::new(label).shortcut_text(shortcut))
        .clicked()
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
        egui::Key::K => Some('k'),
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
        // Refresh the enablement snapshot the shared context menu reads, so
        // Undo/Redo are greyed out whenever the history stack cannot serve
        // them (audit M19).
        self.ui.can_undo = self.history.can_undo();
        self.ui.can_redo = self.history.can_redo();
        // Refresh Delete enablement so the menu does not offer a no-op when the
        // whole selection is locked (full-app audit M7).
        let can_delete = self.active_doc().is_some_and(|doc| {
            self.ui.selected.iter().any(|id| {
                doc.find_markup(*id)
                    .is_some_and(|m| doc.is_markup_editable(m))
            })
        });
        self.ui.can_delete = can_delete;
        // Computed before `ctx.input` so the immutable borrow of `ctx` for
        // `egui_wants_keyboard_input` does not overlap the input closure.
        let wants_kb = ctx.egui_wants_keyboard_input();
        ctx.input(|i| {
            let editing = self.ui.text_edit.is_some();
            // `wants_kb` gates every destructive/shortcut branch: while the
            // user is typing in ANY egui text field (Markups List search,
            // inspector subject/author/layer-rename/comment), Backspace and
            // Delete must edit the text, not delete the selected markup, and
            // Ctrl+Z/Ctrl+Y must not run history under the user's hands
            // (full-app audit B1).
            if (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace))
                && !editing
                && !wants_kb
            {
                self.delete_selected();
            }
            if i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift && !wants_kb
            {
                // Never mutate history mid-edit: undoing past the markup's
                // creation would leave a stale `text_edit` pointing at a
                // markup that no longer exists.
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else if self.history.undo() {
                    self.ui.dirty = true;
                    self.clear_selection();
                }
            }
            if i.modifiers.command && i.key_pressed(egui::Key::Y) && !wants_kb {
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else if self.history.redo() {
                    self.ui.dirty = true;
                    self.clear_selection();
                }
            }
            if i.modifiers.command && i.key_pressed(egui::Key::O) {
                // Native dialog must run on the main thread; the logic() handler
                // runs on it, so calling rfd here is safe.
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Rubric project", &["rubric.json"])
                    .pick_file()
                {
                    self.open_project_path(path);
                }
            }
            if i.modifiers.command && i.key_pressed(egui::Key::S) {
                // save_project_to sets an accurate "Saved" / "Save failed: …"
                // status; no override here.
                self.save_project_action();
            }
            if i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::S) {
                self.save_project_as_action();
            }
            if i.modifiers.command && i.key_pressed(egui::Key::I) {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PDF document", &["pdf"])
                    .pick_file()
                {
                    match crate::data::pdf::read_pdf_info(&path) {
                        Ok(info) if info.page_sizes.is_empty() => {
                            self.ui.status = "Import failed: the PDF has no pages".to_string();
                        }
                        Ok(info) => {
                            let stem = path
                                .file_stem()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Imported PDF".to_string());
                            let count = info.page_sizes.len();
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
            }
            if i.modifiers.command && i.key_pressed(egui::Key::P) {
                let _ = self.print_page();
            }
            if i.modifiers.command && i.key_pressed(egui::Key::Plus) {
                self.set_zoom_level(self.ui.zoom * 1.25);
            }
            if i.modifiers.command && i.key_pressed(egui::Key::Minus) {
                self.set_zoom_level(self.ui.zoom * 0.8);
            }
            if i.modifiers.command && i.key_pressed(egui::Key::Num0) {
                self.fit_page();
            }
            if i.modifiers.command && i.key_pressed(egui::Key::A) && !wants_kb {
                // Select All on the current page (full-app audit M5/M10); the
                // same path as Edit ▸ Select All.
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else {
                    self.select_all_on_page();
                }
            }
            if i.modifiers.command && i.key_pressed(egui::Key::D) && !wants_kb {
                // Duplicate the selection (v0.4 gap #6); same path as the menu.
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else {
                    self.duplicate_selected();
                }
            }
            if i.key_pressed(egui::Key::PageUp) && !wants_kb {
                // Never switch pages mid-edit: the edit context would be lost
                // and the text could be committed onto the wrong page.
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else if self.ui.current_page > 0 {
                    self.jump_to_page(self.ui.current_page - 1);
                }
            }
            if i.key_pressed(egui::Key::PageDown) && !wants_kb {
                // Never switch pages mid-edit: the edit context would be lost
                // and the text could be committed onto the wrong page.
                if self.ui.text_edit.is_some() {
                    self.ui.status = "Finish or cancel text edit first".to_string();
                } else {
                    let count = self.active_doc().map(|d| d.page_count()).unwrap_or(0);
                    if count > 0 && self.ui.current_page + 1 < count {
                        self.jump_to_page(self.ui.current_page + 1);
                    }
                }
            }
            if i.key_pressed(egui::Key::Escape) {
                if self.report_export.is_some() {
                    self.report_export = None;
                } else if self.ui.calibration.is_some() {
                    // Close the calibration dialog without touching the scale
                    // (v0.4 gap #4): no snapshot, nothing changed.
                    self.cancel_calibration();
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

        // Audit M1: the OS close button (and any window-manager close) must not
        // discard unsaved work. egui 0.36 exits the root viewport after this
        // frame unless `CancelClose` is sent, so veto the close while the
        // project is dirty and route it through the same confirmation as
        // File ▸ Quit.
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested && self.ui.dirty && !self.force_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.ui.quit_confirm = true;
            self.ui.status = "The project has unsaved changes — save or confirm to quit".into();
        }
        self.force_close = false;
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
        // Pages sidebar (spec F-404). Only shown when the document has pages
        // and the panel is enabled in the View menu.
        let page_count = self.active_doc().map(|d| d.page_count()).unwrap_or(0);
        if page_count > 0 && self.ui.show_pages && !self.ui.focus_mode {
            let source_pdf = self.history.current().source_pdf.clone();
            egui::Panel::left("pages")
                .default_size(168.0)
                .min_size(120.0)
                .max_size(260.0)
                .resizable(true)
                .show(ui, |ui| {
                    if let Some(index) = pages::draw_pages(
                        ui,
                        page_count,
                        self.ui.current_page.min(page_count - 1),
                        source_pdf.as_deref().map(std::path::Path::new),
                        &mut self.thumb_cache,
                    ) {
                        self.jump_to_page(index);
                    }
                });
        }
        egui::Panel::left("tools")
            .default_size(200.0)
            .min_size(180.0)
            .show(ui, |ui| self.left_panel(ui));
        // Tool Chest (v0.4 gap #7): the "My Tools" panel over the app-level
        // preset library. Armed presets travel via
        // `ui.pending_style_preset`, which `canvas::finish_markup` consumes
        // at creation time; only apply-to-selection mutates the project and
        // asks for a snapshot.
        let mut chest_mutated = false;
        if self.ui.show_tool_chest && !self.ui.focus_mode {
            let library = &mut self.tool_chest_library;
            let project = self.history.current_mut();
            let ui_state = &mut self.ui;
            egui::Panel::left("tool_chest")
                .default_size(210.0)
                .min_size(170.0)
                .resizable(true)
                .show(ui, |ui| {
                    chest_mutated = tool_chest::draw_tool_chest(ui, library, project, ui_state);
                });
        }
        if chest_mutated {
            self.push_snapshot("tool chest apply");
        }
        let mut inspector_mutated = false;
        if !self.ui.focus_mode {
            let inspector_response = egui::Panel::right("inspector")
                .default_size(280.0)
                .min_size(240.0)
                .show(ui, |ui| {
                    inspector_mutated = self.right_panel(ui);
                })
                .response;
            // Inspector context menu (spec F-603): the same menu the canvas offers.
            let _ = inspector_response.context_menu(|ui| {
                menu::draw_editor_context_menu(ui, &mut self.ui);
            });
        }
        egui::Panel::bottom("status").show(ui, |ui| self.bottom_bar(ui));

        let mut list_action = None;
        let mut status_changed = false;
        if self.ui.show_markups_list && !self.ui.focus_mode {
            let project = self.history.current_mut();
            let ui_state = &mut self.ui;
            egui::Panel::bottom("markups_list")
                .default_size(190.0)
                .min_size(110.0)
                .max_size(420.0)
                .resizable(true)
                .show(ui, |ui| {
                    let (action, sc) = markups_list::draw_markups_list(ui, project, ui_state);
                    list_action = action;
                    status_changed = sc;
                });
        }
        if status_changed {
            self.push_snapshot_quiet();
        }
        if let Some(action) = list_action {
            self.ui.selected.clear();
            self.ui.selected.insert(action.id);
            self.ui.status = format!("Selected markup on page {}", action.page + 1);
            if action.page != self.ui.current_page {
                self.jump_to_page(action.page);
            }
        }

        let prior_status = self.ui.status.clone();
        let show_minimap = self.ui.show_minimap;
        let mut need_snapshot = false;
        egui::CentralPanel::default().show(ui, |ui| {
            self.ui.canvas_size = ui.available_size();
            need_snapshot = canvas::draw_canvas(
                ui,
                self.history.current_mut(),
                &mut self.ui,
                &mut self.draw,
                &mut self.last_page_pos,
                &mut self.page_textures,
                &mut self.failed_page_textures,
                &mut self.bg_error,
                false, // is_export: show chrome in live editor
                show_minimap,
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
        // Execute any context-menu action requested this frame (spec F-603).
        if let Some(action) = self.ui.pending_action.take() {
            self.apply_editor_action(action);
        }

        // Update-check result (guide §7.2): honest state — running, up to date,
        // available, or could-not-check. Never blocks the UI loop.
        self.draw_update_result(ui.ctx());

        // Release-notes popup (guide §9, §12.1): opened from the subtle
        // bottom-right version label. Content is embedded at build time.
        self.draw_release_notes(ui.ctx());

        // Bug report dialog (guide §7.1): preview + editable notes; nothing is
        // sent until the user explicitly chooses to.
        self.draw_bug_report(ui.ctx());

        // Calibration dialog (v0.4 gap #4): confirm the real distance of the
        // drawn reference segment and rewrite the page scale.
        self.draw_calibration_dialog(ui.ctx());

        // About window (full-app audit M4): version, licence and links instead
        // of a transient status line.
        if self.show_about {
            let mut open = true;
            egui::Window::new("About Rubric")
                .open(&mut open)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.heading("Rubric");
                    ui.label("Free, cross-platform PDF markup & measurement");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Version:");
                        ui.label(env!("CARGO_PKG_VERSION"));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Licence:");
                        ui.label("MIT");
                    });
                    ui.hyperlink_to(
                        "Source & releases",
                        "https://github.com/Perchemop/rubric-desktop",
                    );
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
            if !open {
                self.show_about = false;
            }
            self.ui.status = "Rubric — free, cross-platform PDF markup & measurement".to_string();
        }

        // Keyboard shortcuts dialog (proposal 10): lists all discoverable
        // shortcuts grouped by category, for reference while working.
        if self.show_keyboard_shortcuts {
            let mut open = true;
            egui::Window::new("Keyboard Shortcuts")
                .open(&mut open)
                .resizable(false)
                .fixed_size(egui::vec2(600.0, 500.0))
                .show(ui.ctx(), |ui| {
                    ui.heading("Rubric Keyboard Shortcuts");
                    ui.separator();
                    ui.label("File:");
                    ui.label("Ctrl+N — New Project        Ctrl+O — Open Project        Ctrl+S — Save Project        Ctrl+Shift+S — Save Project As…");
                    ui.label("Ctrl+I — Import PDF        Ctrl+P — Print        Ctrl+W / Cmd+Q — Quit");
                    ui.separator();
                    ui.label("Edit:");
                    ui.label("Ctrl+Z — Undo        Ctrl+Y — Redo        Ctrl+A — Select All        Del / Backspace — Delete");
                    ui.separator();
                    ui.label("View:");
                    ui.label("Zoom In / Out / Fit Page        PageUp / PageDown        Grid / Snap to Grid        Full Screen (Esc)");
                    ui.label("Auto-save on/off        Theme cycling");
                    ui.separator();
                    ui.label("Tools:");
                    ui.label("Select / Highlight / Rectangle / Ellipse / Line / Arrow / Pen / Text Box");
                    ui.label("Calibrate Page Scale (two-click reference segment)");
                    ui.label("Measurement tools: Length / Diameter / Radius / Perimeter / Area / Angle");
                    ui.separator();
                    ui.label("Markup actions:");
                    ui.label("Ctrl+D — Duplicate        Ctrl+C / Ctrl+V — Copy / Paste Properties");
                    ui.label("Bring Forward / Send Backward / Bring To Front / Send To Back");
                    ui.separator();
                    ui.label("General:");
                    ui.label("Esc — Close dialog / Cancel operation");
                    ui.label("Ctrl+Shift+S — Save Project As…");
                });
            if !open {
                self.show_keyboard_shortcuts = false;
            }
        }

        if self.ui.quit_confirm {
            let mut open = true;
            egui::Window::new("Unsaved Changes")
                .open(&mut open)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("The project has unsaved changes.");
                    ui.horizontal(|ui| {
                        // Save first, then quit — but only when the save really
                        // landed. A cancelled Save As or a failed write leaves
                        // the project dirty, so the dialog stays open instead of
                        // quitting on top of unsaved work.
                        if ui.button("Save and Quit").clicked() {
                            self.save_project_action();
                            if !self.ui.dirty {
                                self.ui.quit_confirm = false;
                                self.force_close = true;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                        if ui.button("Quit Anyway").clicked() {
                            self.ui.quit_confirm = false;
                            // Tell the close veto this close was confirmed
                            // (audit M1); otherwise it would re-prompt.
                            self.force_close = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Cancel").clicked() {
                            self.ui.quit_confirm = false;
                        }
                    });
                });
            if !open {
                self.ui.quit_confirm = false;
            }
        }

        // Replace-unsaved-project confirmation (full-app audit B2): New / Open
        // / Import / Recent park their install here instead of discarding work
        // silently.
        if self.pending_replace.is_some() {
            let mut open = true;
            egui::Window::new("Replace Unsaved Project")
                .open(&mut open)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(
                        "The current project has unsaved changes that will be lost. \
                         Save first (Ctrl+S) to keep them.",
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Discard Changes").clicked() {
                            if let Some(pending) = self.pending_replace.take() {
                                self.install_project_now(
                                    pending.project,
                                    pending.current_path,
                                    pending.status,
                                );
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.pending_replace = None;
                            self.ui.status = "Replace cancelled — project kept".to_string();
                        }
                    });
                });
            if !open {
                self.pending_replace = None;
                self.ui.status = "Replace cancelled — project kept".to_string();
            }
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

        // Autosave (spec F-502): run at the end of the frame so it sees every
        // committed change from this pass.
        self.maybe_autosave();

        ui.ctx()
            .request_repaint_after(Duration::from_secs_f32(0.25));
    }
}
