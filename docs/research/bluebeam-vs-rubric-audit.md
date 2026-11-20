# Bluebeam vs Rubric — Capability Gap Analysis

## Method

Audited Rubric source code (apps/rubric/src/) against the four Bluebeam Revu research files in this directory (each is an original behavioural summary with its public source URLs):

- [`bluebeam-toolbars.md`](bluebeam-toolbars.md) (toolbars, toolstrips, Tool Chest, tool sets)
- [`bluebeam-settings.md`](bluebeam-settings.md) (preferences, defaults, shortcuts, export/import)
- [`bluebeam-markup-tools.md`](bluebeam-markup-tools.md) (tool categories, access patterns, properties, list)
- [`bluebeam-ui-layout.md`](bluebeam-ui-layout.md) (window regions, panels, docking, MultiView, status bar)

Rubric files read for this audit: `ui/app.rs`, `ui/app_menu.rs`, `ui/app_topbar.rs`, `ui/app_status.rs`, `ui/toolbar.rs`, `ui/tool_chest.rs`, `ui/app_tool_chest.rs`, `ui/canvas.rs`, `ui/inspector.rs`, `ui/app_inspector.rs`, `ui/markups_list.rs`, `ui/app_markups_list.rs`, `ui/menu.rs`, `ui/app_dialogs.rs`, `ui/pages.rs`, `ui/app_pages.rs`, `domain/project.rs`, `domain/markup.rs`, `domain/markup_comment.rs`, `domain/style.rs`, `domain/style_preset.rs`, `domain/action.rs`, `data/prefs.rs`.

Statuses: **met** (capability exists at the level Bluebeam documents it), **partial** (a subset of the behaviour exists), **missing** (not implemented). Every row names the Rubric evidence.

## A. Toolbars

| Bluebeam capability | Rubric status | Gap |
|---|---|---|
| Named customizable toolbars (18–22 named toolbars, each show/hide via Tools ▸ Toolbars) | missing | Rubric has one fixed `Tools` drop-down (`ui/toolbar.rs::draw_tool_dropdown`) plus a fixed left tools panel. No concept of named toolbars or per-toolbar visibility. |
| Customize Toolbars dialog (Categories/Commands/Toolbar/Items panes; add, reorder, hide, remove items; separators/spaces; create, rename, delete toolbars) | missing | No customization dialog of any kind; the tool list is hard-coded in `TOOL_FAMILIES` (`ui/toolbar.rs`). |
| Toolstrips (drag toolbars between top/left/right strips via handles; strips auto-created) | missing | Layout is fixed: one top bar (`ui/app_topbar.rs`), one left tools panel, one right inspector. Nothing is draggable between edges. |
| Lock toolbars (all-or-nothing; handles hidden when locked) | missing | No movable chrome exists to lock; the layout is permanently fixed. Only relevant once toolstrips exist. |
| Properties Toolbar (fixed under menu bar, cannot move, **dynamic** — shows the active markup's editable appearance) | partial | The dynamic properties surface is the right-hand Inspector (`ui/inspector.rs`), and the top bar shows a dynamic active-tool chip (`ui/app_topbar.rs`). No persistent properties strip under the menu bar mirroring the selection. |
| Navigation bar (page/view stepping, split-view controls, zoom/pan/select mode, page-scale indicator, profile selector; F4 toggle) | partial | Page nav + zoom % live in the top bar; scale editing lives in the Inspector's Scale section. No dedicated navigation bar, no profile selector, no F4 toggle. |
| Status bar (snap toggles, reuse toggle, sync, document size, live cursor coordinates) | partial | Rubric's status bar (`ui/app_status.rs`) shows hint text, live cursor coords, page count, document title and the version label. Missing: snap-to-content/markup toggles, reuse toggle, view-sync control, page dimensions, page scale readout. |
| Toolbar configuration stored per Profile (layout follows the active profile) | missing | No profiles at all. Panel visibility/zoom/page persist app-level in `prefs.json` (`data/prefs.rs`), shared by every project. |
| Panel access bar (icon strip opening panels; can auto-hide; contents via a Show menu) | missing | Panel toggles live only in the View menu (`ui/app_menu.rs`); there is no icon strip and no auto-hide. |

**A totals: 0 met, 3 partial, 6 missing (9 rows).**

## B. Tool Chest

| Bluebeam capability | Rubric status | Gap |
|---|---|---|
| Reusable saved markups that re-place on any page without recreation | partial | The Tool Chest (`ui/tool_chest.rs`, `domain/style_preset.rs`) stores style + metadata only. "Apply" arms a style that `canvas::finish_markup` applies to the *next* markup (Properties-Mode-like); it never stores geometry/text and cannot re-place an exact duplicate. |
| Tool sets — named groups of tools, each with its own mini toolbar (expand/collapse, scale toggle, gear menu) | partial | One flat library ("My Tools", capped at 64 presets). No multiple named sets, no per-set toolbar, no collapsed flyout. |
| My Tools — permanent personal set surviving restarts, in every profile | partial | The library round-trips through `prefs.json` (`RubricApp::tool_chest_library` ↔ `AppPrefs`), **but it is only written to disk when some other pref-saving action fires** (theme/zoom/page change/panel toggle — see `save_app_prefs` call sites in `ui/app.rs`, `ui/app_menu.rs`). Saving a style does not persist it; quit without one of those actions loses the tool. |
| Recent Tools — auto-records every placed markup, session-temporary, overflow arrow, Max-Recents setting, Clear Recents | missing | Nothing records tool usage. The chest is purely manual capture. |
| Properties Mode vs Drawing Mode per tool (double-click toggles; icon reflects mode) | missing | Single behaviour (style-only application). No mode concept, no stored geometry. |
| Set Scale per tool set (tools resize to the target drawing's calibration) | missing | No scale on presets; presets apply visual styles only. |
| Pin a tool set to a toolbar (the documented Toolbar↔Chest bridge) | missing | No toolbars exist to pin to (see A). |
| Export / Import / Add-Existing of tool sets (file-based sharing; import = independent copy; add = live reference) | missing | The library exists only inside `prefs.json`; no file export/import path. |
| Shared-set locking (network sets read-only by default, lock icon, checkout) | missing | No sharing, no file format, no locking. |
| Icon display modes per set — Symbol grid vs Detail table | partial | One display: a labelled list with colour swatch (closest to Detail view). No symbol-grid mode. |
| Icon-size slider at panel bottom | missing | Fixed row layout. |
| Tool Chest settings — Comment Persistence, Update Tool Set Item on Reuse | missing | No such settings; placed markups never write back to a stored tool. |
| Sequences & Actions (auto-incrementing text tools; hyperlink actions on tools) | partial | Count markers auto-number per page (`domain/markup.rs::next_count_number`) — a fixed sequence, not a user-definable one; no user-editable sequence tools and no actions. |
| Add to Tool Chest from a right-click on a placed markup, choosing the target set | partial | Capture exists but lives only in the chest panel ("Save Markup as Style") and captures style, not geometry; no context-menu entry and no set picker (there are no sets). |
| Grouped markups saved as a single reusable tool | missing | No grouping of markups exists anywhere in the app. |

**B totals: 0 met, 6 partial, 9 missing (15 rows).**

## C. Markup Tools

| Bluebeam capability | Rubric status | Gap |
|---|---|---|
| Tool categories: Markup / Image / PDF Content / Stamp / Measure / Sketch to Scale (Tools menu sub-menus) | partial | Rubric groups Review / Shapes / Paths / Measure + Calibrate (`domain/project.rs::TOOL_FAMILIES`). Missing families: Image, Sketch to Scale, Form, Signature. Missing tools: Arc, Dimension, Eraser, Typewriter, Cloud+, Image placement/crop, Flag, volume/cutout/dynamic-fill measures. |
| Core markup set (Text Box, Note, Callout, Pen, Highlight, Line, Arrow, Polyline, Rectangle, Ellipse, Polygon, Cloud) | met | All present with drawing gestures in `ui/canvas.rs` (drag bounds/segments/ink, click-path multi-point, 3-point angle, count). |
| PDF Content text tools (Underline, Squiggly, Strikeout; Review Text; auto-hyperlinks) | partial | Underline/Squiggly/Strikeout exist as text-anchored markups (TextAnchor geometry, `ui/canvas.rs` draw paths); Highlight snaps to extracted text lines. Review Text and URL→hyperlink auto-creation are missing. |
| 4-way tool access (Tools menu, toolbars, Tool Chest, keyboard) | partial | Three ways exist: Tools drop-down, canvas/inspector context menu (`ui/menu.rs`), single-key shortcuts. The Tool Chest stores styles, not tools, so it is not a fourth invocation path. |
| Single-letter keyboard shortcuts for markup tools | met | Every tool has a unique key (`domain/project.rs::TOOL_METADATA`, tests assert uniqueness/round-trip); firing is gated away from text fields and modifier combos (`ui/app.rs::run_logic`). Mapping is Rubric's own, not Bluebeam's. |
| Measure shortcuts deliberately separated from markup shortcuts | met | Measure tools use digits 1–7, Calibrate uses K, distinct from the letter block. (Bluebeam uses Shift+Alt combos; the separation, not the exact chords, is the capability.) |
| My Tools numeric hot keys — position-based, shown on the icon, activate the tool *with its saved properties* | missing | Digits 1–7 are bound to measure *tools* (`TOOL_METADATA`), not to Tool Chest positions. No hot keys exist for saved tools. |
| Modifier drawing: Shift constrains lines to H/V/45° and Rectangle/Ellipse to square/circle | missing | `ui/canvas.rs` reads Shift only for selection add/toggle. No constraint on any drawing gesture. |
| Modifier drawing: Ctrl overrides snap while drawing | missing | Creation positions always pass through `ui_state.grid.snap(p)` with no modifier check (hit-testing correctly ignores the grid, but drawing cannot bypass it). |
| Reuse toggle (tool stays active after placing; toggleable, incl. from the Status Bar) | partial | Tools always stay active after placement (nothing resets `active_tool` on commit) — reuse is default-on with no toggle, and no status-bar switch. |
| View/Markup mode split (markup tools disabled until Markup Mode) | missing | Markups are always enabled; there is no mode gate. (Arguably deliberate for a markup-first app; recorded as a gap for parity.) |
| Spacebar pans while a markup tool is active | partial | Space pans when no drag is in progress; pressing Space **mid-drag cancels the draft** (`ui/canvas.rs`) instead of panning with the draft preserved. |

**C totals: 2 met, 5 partial, 5 missing (12 rows).**

## D. Properties Panel

| Bluebeam capability | Rubric status | Gap |
|---|---|---|
| Properties edited after drawing (select → edit) | met | Inspector edits Subject/Author/Status/Layer/stroke on the selected markup and writes back through the document (`ui/inspector.rs`), with snapshot discipline. |
| General section (Author, Subject, Label/Comments, Lock) | partial | Author/Subject/Comments exist. A per-markup **Lock** toggle is missing from the UI: `Markup.locked` exists in the domain and `Document::is_markup_editable` honours it, but only *layer* locking has a control (Inspector Layers section). |
| Appearance section (colour, opacity, fill, line width, line style, line ends) | partial | Only stroke colour + width are editable. `MarkupStyle` already carries fill colour, `FillMode`, `DashPattern` and per-colour alpha — the renderer consumes them (`ui/canvas.rs::draw_markup`) but no UI control exists for fill mode, dash, opacity or line ends. |
| Custom fields section | missing | No custom metadata on markups. |
| Layout section (numeric X/Y/size position fields) | missing | Move/resize is mouse-only (drag, corner grips, point handles). No numeric position/size entry. |
| Options: Add to Tool Chest | partial | The equivalent capture ("Save Markup as Style") lives in the chest panel, not in a properties Options section, and captures style rather than the full markup. |
| Options: Set As Default (changes that tool's default everywhere) | missing | Tool defaults are fixed in code (`MarkupKind::default_style()`). Arming a chest style affects only the current session and is not a persisted default; there is no per-type default store. |
| Apply to All Pages (context-menu action) | missing | No such action exists. |
| Format Painter | met | Copy Properties / Paste Properties (`domain/action.rs`, `RubricApp::copy/paste_selected_style`) copies the style clipboard onto every selected markup — the same capability. |
| Right-click Lock for a markup | missing | See General row: no markup-level lock command in the context menu or inspector. |

**D totals: 2 met, 3 partial, 5 missing (10 rows).**

## E. Markup List

| Bluebeam capability | Rubric status | Gap |
|---|---|---|
| Automatically tracks every markup with author, date, colour, comments | partial | The domain `MarkupList` auto-tracks every markup with page/kind/author/subject/summary/status. No Date column (timestamps exist on `Markup` but are not shown) and no Colour column. |
| Sortable table | met | Sorting is implemented in the pure domain (`MarkupListSort`) and selected from a combo (`ui/markups_list.rs`). Not by clicking column headers (no sortable headers), but the capability exists. |
| Selecting a row jumps the canvas to that markup | met | `MarkupListAction` → selection + `jump_to_page` in `ui/app.rs`. |
| Extensive toggleable columns (Subject, Page, Lock, Status, Checkmark, Author, Date, Colour, X/Y, Comments, measurement values, Label, Sequence, Layer, Space, Legend) | missing | Fixed six-column grid (Page, Type, Author, Subject, Summary, Status). No column chooser, no per-column toggles. |
| Status workflow — default statuses plus custom statuses | partial | Four fixed statuses (Open/In review/Approved/Rejected) with colour legend, per-row editor, batch change from both list and inspector. Custom user-defined statuses are missing. |
| Reply threads attached under a markup's row | missing | Comments are a flat per-markup list (`domain/markup_comment.rs`); no reply-to structure, and comments do not render in the list rows (only in the Inspector/Note popover). |
| Search box | met | Free-text search across author/subject/summary, wired into the domain filter. |
| Filter list with saveable named filter sets | partial | Rich filters exist (status, author, subject, kind, page, layer) plus grouping by status/author/kind/page/layer — grouping exceeds the researched Bluebeam behaviour. Named, saveable filter sets are missing. |
| Import/Export of markups (PDF, XML, BAX, FDF) | partial | CSV export of the filtered view only. No import in any format; no XML/FDF/BAX/PDF-format markup exchange. |
| Summary report publishing | met | File ▸ Export Report produces a grouped (by status/author/kind/etc.) text or JSON summary via `domain/report.rs` + `data/export.rs`. |
| Running measurement total per measurement type at the top of the measurement column | met | Per-kind totals footer over the filtered view (`render_totals_footer`), plus the on-page count total badge in the canvas. |

**E totals: 5 met, 4 partial, 2 missing (11 rows).**

## F. Settings/Preferences

| Bluebeam capability | Rubric status | Gap |
|---|---|---|
| Categorized Preferences dialog (named category list/sidebar) | missing | No preferences dialog. Settings are scattered: View menu (grid, theme, contrast, panels, focus mode), File menu (auto-save), Inspector Settings tab (author, Bates, precision). |
| Search bar that jumps to the owning preferences page | missing | No dialog to search. |
| Category breadth (~28 categories; tabs inside categories, e.g. General's five tabs) | missing | Roughly a dozen user-facing settings exist in total, across four surfaces; no category structure. |
| Dynamic default properties (remember last-used appearance per markup type) | missing | Defaults are static per kind in code. The closest behaviour — armed chest style — is a single session-global override, not per-type memory. |
| Measurement units layering: app-default system (preferences) vs per-document panel state (precision + units) vs OS regional | partial | The per-document layer exists (per-page `Scale` + project-wide `precision`). There is no app-level default-units preference for new pages/projects, so new documents always start at the coded default. |
| Keyboard shortcut manager as a separate dialog (Tools/Commands split, remap, Restore Defaults) | missing | Help ▸ Keyboard Shortcuts is a **read-only** reference list (`ui/app_dialogs.rs::draw_keyboard_shortcuts`). No remapping, no Tools/Commands split, no restore-defaults. |
| Export/import of shortcut files | missing | Not implemented. |
| Whole-preferences export/import (settings tree to a file) | missing | `prefs.json` exists but there is no export/import UI or command. |
| Single-key shortcut suppression while typing (form-field analogue) | met | All single-key shortcuts are suppressed while any egui text field holds focus (`egui_wants_keyboard_input` gating in `run_logic`); tests cover the behaviour. |

**F totals: 1 met, 1 partial, 7 missing (9 rows).**

## G. UI Layout

| Bluebeam capability | Rubric status | Gap |
|---|---|---|
| 8-region window stack (menu bar, properties toolbar, toolstrips, side panels, workspace, navigation bar, bottom panel, status bar) | partial | Present: menu bar, top toolbar, left panels (Pages, Tool Chest), right Inspector, bottom Markups List, central canvas, status bar. Missing as distinct regions: properties toolbar, toolstrips, navigation bar. |
| ~17 named side panels (Tool Chest, Markups List, Thumbnails, Bookmarks, Properties, File Access, Layers, Flags, Links, Search, Measurements, Sets, Spaces, Signatures, Forms, Studio, 3D) | partial | Rubric has 4 docked panels (Pages, Tool Chest, Inspector, Markups List). Layers/Search exist as Inspector sections, not panels. Missing: Bookmarks, Flags, Links, File Access, Measurements panel, Sets, Spaces, Signatures, Forms, Studio, 3D. |
| Panel docking — attach left/right/bottom, float into workspace, split a dock (position wheel), hide, auto-hide access bar, resize, overlay, persistence | partial | Panels are fixed-edge and resizable; visibility toggles persist via `prefs.json` and Focus Mode hides all panels. No drag-to-dock, no floating, no splitting, no auto-hide, no overlay mode. |
| Document tabs (reorderable, hover preview, movable between splits) | missing | The domain supports multiple documents per project (`Project.documents`/`active_document`), but the UI exposes no tab bar or document switcher — one active document at a time. |
| MultiView split workspace (up to 16 independent splits) | missing | Single canvas; no split capability. |
| MultiView Extended (drag a tab into a detached OS window) | missing | Not implemented (eframe viewport support would be needed). |
| Synchronized views (Synchronize Document / Synchronize Page) | missing | Nothing to synchronize without splits. |
| Thumbnails panel — click-to-jump, size slider, page operations (insert/extract/replace/delete/rotate), drag reorder | partial | Click-to-jump works with async thumbnails (`ui/pages.rs`). Missing: size slider, drag reorder, and all page operations (page rotation exists in the domain `Page.rotation` but has no UI; no insert/delete/extract). |
| Bookmarks panel (outline tree, click-to-run, AutoMark) | missing | No bookmarks model or panel. |
| Flags panel (lightweight position markers) | missing | No flags model or panel. |
| Navigation bar (F4 toggle; split controls, pan/zoom/select, profile menu) | partial | Its functions are distributed: page nav and zoom in the top bar, pan/zoom on the canvas, scale in the Inspector. No dedicated bar, no toggle. (Cross-reference row A6.) |

**G totals: 0 met, 5 partial, 6 missing (11 rows).**

## Summary

- Total capabilities audited: **77**
- Met: **10**
- Partial: **27**
- Missing: **40**

Overall shape: Rubric's strongest parity is in the Markups List core (sort/search/filter/group, totals, summary reports), the basic markup tool set, and post-draw property editing. The largest structural gaps are (1) the Tool Chest stores styles rather than reusable markups (no Recent Tools, no Drawing Mode, no sets), (2) there is no preferences/shortcuts management surface, and (3) the window is a fixed layout with no docking, tabs, splits or extra navigation panels.

*(Delta 2026-11-20: four rows moved to met and two missing rows moved to partial since this snapshot — see the update section below. Recomputed snapshot: **14 met / 29 partial / 34 missing** of 77.)*

### Priority gaps for next milestones

Ranked by reviewer value per unit of implementation cost, using what the domain already supports:

1. **Inspector depth (D2, D3, D10)** — expose the existing `MarkupStyle` fields (fill mode, dash, opacity) and add a per-markup Lock toggle and right-click Lock command. The domain is ready; this is UI-only.
2. **Set As Default + dynamic defaults (D7, F4)** — a per-`MarkupKind` default style store (persisted app-level), written by a "Set As Default" action. The keystone for "appearance persists across sessions".
3. **Recent Tools in the Tool Chest (B4)** — auto-capture the last N placed markups as session tools; cheap once presets can hold geometry.
4. **Full-markup presets / Drawing Mode (B1, B5)** — extend `StylePreset` to optionally capture geometry/text so a tool can re-place an exact duplicate; Properties/Drawing mode falls out naturally.
5. **Markup list columns + custom statuses (E4, E5)** — toggleable columns (Date, Colour, Layer, measurement value) and user-defined statuses; the list is already a domain-driven table.
6. **Modifier drawing (C8, C9)** — Shift-constrain and Ctrl-snap-override in `ui/canvas.rs` draw handlers; small, high-daily-use win.
7. **Named tool sets with export/import (B2, B8)** — group presets into sets and serialize sets to `.rubric-tools.json`; also fixes the B3 persistence-on-save defect.
8. **Measurement units app default (F5)** — an app-level default unit applied to new pages, layered over the existing per-page scale.
9. **Layout numeric fields + Apply to All Pages (D5, D8)** — numeric X/Y/size entry and a multi-page apply action in the inspector/context menu.
10. **Page operations in the Pages panel (G8)** — rotate/delete/insert pages (rotation already exists in the domain).
11. **Multi-document tabs (G4)** — the `Project.documents` model exists; a tab bar over `active_document` is the remaining work.
12. **Docking/float panels (G3)** and **split views (G5)** — the largest egui lifts; schedule last, after the capability gaps above are closed.

## Implementation status update (2026-11-20)

The tables above are the **2026-11-15 snapshot**; their row statuses are left
unchanged so the audit remains a dated record. This section is the verified
delta since — every row change is evidenced by a merged PR.

| Row | Capability | Was | Now (evidence) |
|---|---|---|---|
| B1 | Reusable saved markups that re-place without recreation | partial | **met** — presets capture the full markup (geometry + text) and re-place an exact duplicate at the click point (PR #227, merged 2026-11-18) |
| B2 | Tool sets — named groups of tools | partial | **partial (improved)** — multiple named sets on `Project.tool_sets` with a set filter in the chest (PR #229); per-set mini toolbar, scale toggle and gear menu still missing |
| B4 | Recent Tools (auto-record, overflow, Max-Recents, Clear) | missing | **met** — every placement auto-captures a recent tool with overflow arrow, Max-Recents setting and Clear Recents (PR #228); recents are session-scope, matching the researched behaviour |
| B5 | Properties Mode vs Drawing Mode per tool | missing | **met** — double-click toggles Drawing/Style mode and the row label reflects it (PR #227); the toggle is session-only, not persisted with the tool |
| B8 | Export/Import of tool sets | missing | **partial** — set-level JSON export/import (`export_set_json`/`import_set_json`, PR #229); "add existing as live reference" still missing |
| D5 | Layout section (numeric X/Y/size entry) | missing | **met** — numeric position/size entry in the Inspector (PR #227) |
| G4 | Document tabs (reorderable, hover preview, movable) | missing | **partial** — tab bar in the top bar with per-document switch/close (`switch_active_document`/`close_document_tab`, PR #229); reorder, hover preview and moving tabs between splits still missing |

Beyond the audited rows:

- **New capability outside Bluebeam's audited set:** apply one page's scale to
  all pages (`Document::apply_scale_to_all_pages`, PR #227).
- **G3 docking — groundwork only:** `domain/workspace.rs::WorkspaceState`
  landed (PR #230, open at time of writing) plus the v0.6 UI-parity spec
  (`specs/ui-parity-revamp.spec.md`); no docking/float/split UI yet, so the
  G3/G5 statuses stand.
- **B3 defect status — unchanged:** the personal My Tools library is still
  written to `prefs.json` only when some other pref-saving action fires. Tool
  *sets* now persist with the project document, which narrows the practical
  impact but does not close the defect.
- **Takeoff rows (capability inventory 33–36)** sit outside this audit's
  Bluebeam scope; their status is tracked in `docs/capability-inventory.md`,
  `docs/parity.md`, and `docs/research/takeoff-quantities.md` §4 (domain layer
  landed 2026-11-20, PR #230; UI pending).

Priority-gap scoreboard against the ranked list above: **#3 done** (PR #228),
**#4 done** (PR #227), **#7 partial** (sets + export/import landed; the B3
defect it named remains open), **#9 half** (numeric fields done; D8
Apply-to-All-Pages still missing), **#11 done at tab-bar level** (PR #229).
Items #1, #2, #5, #6, #8, #10 and #12 stand as written.

### Notes

- Rows reflect capability presence, not Bluebeam's exact interaction design: Rubric's own mappings (e.g. digits for measure tools, its own shortcut letters) are credited where the capability is equivalent.
- Clean-room note: this audit compares Rubric against original behavioural summaries with URLs (see the four research files). No Bluebeam UI text, icons, screenshots or trade dress may be copied into Rubric; capability-level ideas only (guide §2.1/§2.2). Per the research gate (guide §2.3/§8.4), dependent implementation tasks need explicit human approval of the source research notes.

## Implementation Status (as of 2026-11-20)

Implementation status is tracked in:

- **`docs/parity.md`** — high-level capability parity summary
- **`docs/capability-inventory.md`** — detailed evidence links (code, tests, CI runs)
- **`docs/spec/README.md`** — feature IDs (F-1xx…F-7xx) for implemented features
- **`docs/IMPLEMENTATION_PLAN.md`** — current phase priorities

### Current state summary

| Category | Status |
|----------|--------|
| Core markups (text, shapes, paths, stamps) | Done (F-101–F-116) |
| Measurement tools | Done (F-201–F-209) |
| Review workflow | Done (F-301–F-305) |
| Markups list / review console | Done |
| Persistence, autosave, recents | Done |
| Export & print | Done |
| Undo/redo | Done |
| UI/UX polish (shortcuts, dark theme, context menus) | Done |
| Accessibility | Done |
| Packaging & installers | Done |
| Document compare / diff | Partial (domain done, UI pending) |
| Reusable style presets | Mostly met (full-markup presets + Drawing Mode, PR #227 — see delta table B1/B5; per-tool mode persistence session-only) |

### Next priorities

1. Inspector depth (expose existing style fields: fill mode, dash, opacity, lock)
2. Set as default / dynamic defaults
3. Takeoff quantities completion: condition dialog (T8), prefs wiring (T9), docs & release (T10) — open as PR #230
4. Docking/float/split panels (G3/G5): `WorkspaceState` groundwork landed (PR #230); UI pending
5. B3 defect: My Tools library not persisted on save (narrowed by tool-set persistence, still open)
