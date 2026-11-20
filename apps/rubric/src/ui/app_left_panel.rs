//! Consolidated left panel (UI redesign T1).
//!
//! The three former left-side panels (Pages, Tools, Tool Chest) are merged into
//! one tabbed container so the canvas keeps the screen real estate it needs.
//! The tab strip follows the Bluebeam-style panel model: a single row of tab
//! buttons at the top of the panel, content below.
//!
//! Only one panel is ever mounted at a time, so the minimum width of the left
//! column drops from three stacked panels (168 + 200 + 210 px) to one.

use egui::Ui;

use crate::ui::app::{RubricApp, UiState};

/// Which panel content the consolidated left panel is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum LeftPanel {
    /// Page thumbnails for quick navigation.
    #[default]
    Pages,
    /// Tool families and the active tool picker.
    Tools,
    /// Reusable style presets ("My Tools").
    ToolChest,
}

impl LeftPanel {
    /// Every tab in the strip, in display order.
    pub(crate) const ALL: [LeftPanel; 3] = [LeftPanel::Pages, LeftPanel::Tools, LeftPanel::ToolChest];

    /// Short label shown on the tab button. The Tools tab is named
    /// "Toolbox" (not "Tools") so its accessible name stays unique next to
    /// the top bar's "Tools" drop-down menu — two different controls must
    /// not share one accessible name (guide II.30.7.4).
    pub(crate) fn label(self) -> &'static str {
        match self {
            LeftPanel::Pages => "Pages",
            LeftPanel::Tools => "Toolbox",
            LeftPanel::ToolChest => "My Tools",
        }
    }

    /// Whether the tab has content worth showing right now. The Pages tab is
    /// hidden when no document is open, so the strip does not offer an empty
    /// tab the user cannot use.
    pub(crate) fn is_available(self, app: &RubricApp) -> bool {
        match self {
            LeftPanel::Pages => app.active_doc().map(|d| d.page_count()).unwrap_or(0) > 0,
            LeftPanel::Tools | LeftPanel::ToolChest => true,
        }
    }
}

/// Draw the consolidated left panel.
///
/// Returns `true` when the Tool Chest content mutated the project (the caller
/// pushes the undo snapshot).
pub(crate) fn draw_left_panel(app: &mut RubricApp, ui: &mut Ui) -> bool {
    let mut chest_mutated = false;

    if app.ui.focus_mode {
        return false;
    }

    // The Pages tab needs a document; fall back to Tools when the active tab
    // has no content so the strip never shows an empty panel.
    if !app.ui.left_panel.is_available(app) {
        app.ui.left_panel = LeftPanel::Tools;
    }

    egui::Panel::left("left_panel")
        .default_size(210.0)
        .min_size(180.0)
        .max_size(320.0)
        .resizable(true)
        .show(ui, |ui| {
            draw_tab_strip(ui, &mut app.ui);
            ui.separator();
            match app.ui.left_panel {
                LeftPanel::Pages => draw_pages_content(app, ui),
                LeftPanel::Tools => draw_tools_content(app, ui),
                LeftPanel::ToolChest => chest_mutated = draw_tool_chest_content(app, ui),
            }
        });

    chest_mutated
}

/// The tab strip: one row of buttons, the active one highlighted.
fn draw_tab_strip(ui: &mut Ui, state: &mut UiState) {
    ui.horizontal(|ui| {
        for panel in LeftPanel::ALL {
            if !panel.is_available_current(state) {
                continue;
            }
            let selected = state.left_panel == panel;
            let response = ui.selectable_label(selected, panel.label());
            if response.clicked() {
                state.left_panel = panel;
            }
        }
    });
}

/// Availability check that does not need the whole app (used inside the tab
/// strip, where only UI state is available).
trait LeftPanelAvailability {
    fn is_available_current(&self, state: &UiState) -> bool;
}

impl LeftPanelAvailability for LeftPanel {
    fn is_available_current(&self, state: &UiState) -> bool {
        match self {
            // Pages needs a document; the panel itself handles the empty case,
            // so the tab stays visible but the content falls back to Tools.
            LeftPanel::Pages | LeftPanel::Tools | LeftPanel::ToolChest => {
                let _ = state;
                true
            }
        }
    }
}

/// Pages tab content: the page thumbnail strip.
fn draw_pages_content(app: &mut RubricApp, ui: &mut Ui) {
    let page_count = app.active_doc().map(|d| d.page_count()).unwrap_or(0);
    if page_count == 0 {
        ui.label("No document open.");
        return;
    }
    let source_pdf = app.history.current().source_pdf.clone();

    // Lazily spawn the shared render worker on first use (pages sidebar or
    // canvas); ownership is hoisted to `RubricApp` (Slice 5).
    RubricApp::ensure_render_worker(&mut app.render_worker, ui.ctx());

    if let Some(index) = crate::ui::pages::draw_pages(
        ui,
        page_count,
        app.ui.current_page.min(page_count - 1),
        source_pdf.as_deref().map(std::path::Path::new),
        &mut app.thumb_cache,
        app.render_worker.as_ref().unwrap(),
    ) {
        app.jump_to_page(index);
    }
}

/// Tools tab content: the tool families.
fn draw_tools_content(app: &mut RubricApp, ui: &mut Ui) {
    crate::ui::toolbar::draw_tool_groups(
        ui,
        &mut app.ui.active_tool,
        &mut app.ui.inspector_mode,
        &mut app.ui.hint,
    );
}

/// Tool Chest tab content: reusable style presets.
fn draw_tool_chest_content(app: &mut RubricApp, ui: &mut Ui) -> bool {
    let library = &mut app.tool_chest_library;
    let project = app.history.current_mut();
    let ui_state = &mut app.ui;
    crate::ui::tool_chest::draw_tool_chest(ui, library, project, ui_state)
}
