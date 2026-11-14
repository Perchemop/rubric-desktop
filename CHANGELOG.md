# Changelog

All notable changes to this project will be documented here. Format: semantic-ish for this repo (Added/Changed/Fixed/Docs).

## [0.4.0] - 2026-11-14

### Added

- **Async page thumbnails** (UI audit P1-2, PR #160; approved design in
  `docs/research/async-thumbnails.md`): Pages sidebar thumbnails now render
  and decode off the UI thread via a long-lived worker
  (`apps/rubric/src/data/render_worker.rs`), driven by pure, headless-tested
  domain types in `apps/rubric/src/domain/thumb_render.rs` (`RenderKey`,
  `RenderJob`, `RenderOutcome`, `RenderQueueState`, `TextureLru`).
  `ui/pages.rs` lays out every page and submits one deduped job per page of
  the document (no scroll culling yet), drains results per frame and poisons
  failures so a broken page is not retried every frame; the
  textures/pending/failed sets are bounded by the page count and reset on the
  next document switch. The old dormant
  `RenderQueue`/`RenderResult`/`RepaintSignal`/`begin_render_thumb`
  scaffolding and the synchronous `load_thumb` were removed. Scope is the
  Pages sidebar only — the canvas page-background path stays synchronous
  (explicit follow-up slice). PDFium calls remain mutex-serialised
  (`thread_safe` is a default feature of the pinned `pdfium-render` 0.9.4,
  so no Cargo change was needed).

### Fixed

- **UI/UX audit P0 round** (2026-11-07, PR #148; findings in
  `docs/audit-ui-2026-11-07.md`): the Replace Unsaved Project dialog is no
  longer a dead end (Save First / Discard & Replace / Cancel — New, Open,
  Import and Recent are usable again while dirty); PDFs can be imported by
  drag-and-drop and a first-run "Import a PDF to begin" card with a working
  button replaces the blank-page void; the toolbar is legible in dark and
  high-contrast themes (colours derived from the active visuals instead of
  hardcoded black/grey); toolbar buttons name their own AccessKit nodes
  (fixing a `labelled_by`-to-nowhere panic for kittest and screen readers)
  and size their hover/click targets to the visible pill; Ctrl+Shift+S no
  longer also fires Save; unwired shortcut hints (Ctrl+N, Ctrl+W/Cmd+Q,
  Esc-fullscreen) were removed and Export PDF / Export Page Image / Print are
  gated with honest hover text; the About window no longer overwrites the
  status line every frame.

- **Icon controls replace font glyphs** (PR #160): the top-bar page-navigation
  arrows and the Layers inspector's visibility/lock/reorder buttons now use
  painter-drawn icons (triangle / eye / padlock) instead of `◀ ▶ ▲ ▼ 👁 🔒 🔓`
  font glyphs, which the bundled fonts do not cover — the egui 0.37 test
  harness (`MissingGlyphPolicy::Panic`) surfaced them. The controls carry
  their tooltip text as AccessKit labels.

### Changed

- Import pipeline unified: `import_pdf_from_path` / `import_pdf_dialog`
  behind File ▸ Import PDF…, Ctrl+I, drag-and-drop and the canvas CTA
  (previously three inline copies).

- **Dependency refresh** (issue #29, PR #160): `eframe`/`egui`/`egui_extras`
  0.36 → 0.37 (dev-dep `egui_kittest` 0.36.2 → 0.37), `rfd` 0.15 → 0.17, and
  `ureq` 2 → 3 in `crates/crash-report` and `crates/update-check`, adapted to
  the ureq 3 API (`.header()`, `send_json`, `into_body().read_json()`,
  `Error::StatusCode` handling).

- **God-file splits** (PR #160, no behaviour change intended):
  `apps/rubric/src/ui/app.rs` panel orchestration extracted into
  `ui/app_canvas.rs`, `ui/app_inspector.rs`, `ui/app_markups_list.rs`,
  `ui/app_pages.rs`, `ui/app_status.rs` and `ui/app_tool_chest.rs`;
  `domain/markup.rs` split into `domain/markup_comment.rs`,
  `domain/markup_geometry.rs`, `domain/markup_measurement.rs` and
  `domain/markup_stamp.rs` (geometry and hit-test helpers now owned by
  `markup_geometry`; `hit_tolerance_page`/`measure_anchor` moved to
  `ui/canvas_geom.rs`).

- **Signed releases + checksum-verified artefacts** (PR #175): release
  artefacts are now signed with minisign (Ed25519, key ID
  `5DDF86377FAD5DB1`); each release publishes `SHA256SUMS.txt.minisig`
  next to the archives, and the app verifies the signature before applying
  an update. Public key embedded at `assets/release-signing.pub`.

### Docs

- Research notes for async page thumbnails and signed releases recorded as
  approved 2026-11-09 in `docs/research/README.md`; stray tracked
  `apps/rubric/src/ui/inspector.rs.orig` merge artifact removed and
  `*.orig`/`*.bak`/`*.rej` ignored (PR #160).


## [0.3.0] - 2026-11-06

### Added

- **Text-anchored markup completion** (design-audit proposal P2-12, PRs #130/#131,
  merged as #131): highlight/strikeout/underline/squiggly now carry real
  `MarkupGeometry::TextAnchor` geometry. A new data-layer text-extraction path
  (`data/pdf.rs` `page_text_lines`) reads per-character boxes through
  pdfium-render's public page-text API and groups them into text lines in
  **page space (y-down)** — PDF user space is flipped with the page height, so
  anchors land on the drawn text. Click-create for Highlight snaps to the text
  line under the cursor (`find_text_line_at_point` returns the whole line:
  anchor, text, width and line height; without a source PDF it falls back to
  unsnapped defaults without loading PDFium). Strikeout/Underline/Squiggly are
  drawn as text-anchored lines (screen-space correct at any zoom/pan),
  `TextAnchor::bounds()` covers the real drawn area (hit-testing and the PDF
  export fallback agree with what is drawn), and tests cover the serde
  round-trip plus a PDFium-backed extraction test over a generated text PDF.
  Known limitation: character boxes on `/Rotate`d pages are reported in
  un-rotated user space, so anchors there can be offset (documented in
  `page_text_lines`).

- **Bates numbering — flattened-PDF export stamp** (spec F-805, v0.6): the
  flattened **PDF** export now bakes the sequential stamps into each page's
  content stream (`overlay_page_with_markups` + `bates_overlay_ops`), alongside
  the already-covered exported page image and print. Right-corner alignment
  uses a published Helvetica AFM width estimate (the lopdf path has no metrics
  table); the on-canvas preview uses exact egui metrics. A headless test asserts
  each exported page carries its stamp and the `/F1` font resource. The source
  PDF is never modified.

- **Fixed: PDF overlay font reference.** The overlay's `Tf` operator referenced
  the font as `Object::Name(b"/F1")`, which lopdf escapes to the name `/F1`
  while the page resource provides `F1` — so flattened text markups rendered
  with a viewer fallback font. The leading slash is dropped so lopdf writes
  `/F1`; exposed by the new Bates export test.

- **Bates numbering — canvas overlay preview + stamped export/print** (spec
  F-804/F-805, v0.6): when Bates is enabled, `draw_bates_overlay` paints the
  stamp at its configured corner — ghosted on the live canvas and solid in the
  exported page image and print output (both reuse `draw_canvas`). The overlay
  follows pan/zoom and creates no markup, so the source PDF stays untouched.
  `BatesConfig::anchor_point` is a pure, unit-tested domain function. The
  flattened-PDF export path does not yet draw Bates (tracked).

- **Bates numbering — configuration, placement and page range** (spec
  F-801–F-803, first slice of v0.6): a `BatesConfig` value type in the domain
  (`apps/rubric/src/domain/bates.rs`) with prefix/suffix, start number,
  clamped digit padding, corner placement, margin and font size, plus a
  `BatesRange` (inclusive from/to, open-ended). The Inspector gains a
  **Bates numbering** section (under Settings) that edits the config and
  previews the first and last stamp and the honest "N of M pages to stamp"
  count. Non-destructive by design — the sequence is configured here and will be
  baked into the composed export/print output (F-804/F-805) without touching the
  source PDF.

- **High-contrast mode** (a11y A3 / proposal P2-15): a **View ▸ High Contrast**
  toggle switches both light and dark themes to pure black-on-white or
  white-on-black text and solid surfaces with 1 px widget borders and a
  high-contrast selection fill, for low-vision use. Persisted in
  `AppPrefs.high_contrast`; a headless test asserts the high-contrast pairs clear
  WCAG AAA (text) and AA (links/selection).

- **In-app version & release notes** (guide §9, §12.1): a subtle version label in
  the bottom-right corner of the status bar opens a Release Notes popup with the
  changelog (current version + Unreleased), per-release and cumulative token
  metrics, and a Check for Updates / View on GitHub action. The changelog
  (`CHANGELOG.md`) and the token ledger (`docs/tokens.json`) are embedded at
  build time (`include_str!`) so the popup works in a packaged binary, not only
  in a source checkout.

- **Token-usage ledger** (guide §12.1): `docs/tokens.json` records
  `total_tokens_since_start` plus a `releases` array of
  `{release_version, release_tokens}` entries. `scripts/update-token-ledger.py`
  updates it idempotently, and `scripts/token-summary.py` renders the token block
  that every GitHub Release body now includes. The release workflow records the
  tag's metrics on a short-lived branch and merges them via a PR (`main` is
  protected), so bookkeeping never blocks a release.

- **Design & GUI audit** (`docs/audits/design-gui-2026-11.md`, PR #101): post-v0.4 design system, layout, interaction, a11y surface, and polish review. Priority backlog recorded as P0 (icon font/SVG, bundled UI + tabular mono fonts, workspace persistence, active-tool chip, empty states), P1 (elevation tokens, segmented Inspector, virtualised Markups List, single tool metadata source, Tool Chest persistence, G4 Host/Ui seam) and P2 (text-anchored markup, focus/reading mode, dual-unit rulers, UI scale/high-contrast, minimap) items in IMPLEMENTATION_PLAN.md and ROADMAP.md.

- **Menu accelerator completeness** (proposal 10 part 2, PR #103):
  - File ▸ **Import PDF…** gains `Ctrl+I` shortcut hint; `Ctrl+Shift+S` Save As, `Ctrl+P` Print added as keyboard shortcuts
  - View ▸ **Zoom In / Out / Fit Page** gain `Ctrl++` / `Ctrl+-` / `Ctrl+0` shortcuts with menu hints
  - New `set_zoom_level()` and `fit_page()` helper functions; zoom clamped to `view::MIN_ZOOM`/`MAX_ZOOM`

- **Typography & spacing token pass** (proposal 11, PR #102):
  - Type scale constants (11/12/13/14/16/20 px for secondary/body/body/button/heading/title) and spacing grid constants (4/8/12/16/24 px)
  - Type scale applied to all standard egui text styles in both light and dark themes
  - Monospace token (`TEXT_MONOSPACE`) for numeric readouts
  - Headless test `typography_tokens_applied_to_both_themes` asserts all text styles per theme

- **Theme focus & rounding** (proposal 12, PR #100):
  - `corner_radius = 4u8` on hovered/active/open widget visuals for consistent interaction
  - Button hierarchy bg_fill tokens (inactive/hovered/active) for primary/secondary/tertiary distinction
  - `focus_stroke` unavailable in egui 0.36; shape-only focus via corner_radius used instead

- **Keyboard shortcuts dialog** (proposal 10, PR #100):
  - Help ▸ **Keyboard Shortcuts…** dialog listing shortcuts grouped by category (File, Edit, View, Tools, General)

- **Calibration dialog rename** (proposal 9, PR #100):
  - File ▸ **Calibrate Page Scale…** dialog button renamed from "Apply" to "Calibrate" for clarity; `.default_width(560)`

- **v0.4-03**: duplicate selected markups (gap #6, PR #93, commit `8a70739`)
  - `Document::duplicate_markups(ids, offset)` clones each editable markup with
    a fresh id and timestamps, translated geometry, recomputed measurements,
    appended on top of its source; locked and missing ids are skipped
  - `EditorAction::Duplicate` ("Duplicate", `Ctrl+D`) in the Edit and context
    menus; selects the clones and pushes one undo snapshot

- **v0.4-04**: flatten markups to PDF (gap #1, PR #96)
  - `data/export.rs` `export_page_to_pdf(source, page_index, out, markups)`
    renders a single page with its markups overlaid, using lopdf
    `delete_pages` + `prune_objects` + `renumber_objects` to keep only the
    requested page; `export_all_pages_to_pdf` delegates to `export_pdf_to`
  - File ▸ **Export Page to PDF…** and **Export All Pages to PDF…** with native
    `rfd` dialogs, gated on having a source PDF
  - 8 unit tests

- **v0.4-05**: drag calibration (gap #4, PR #96)
  - `Tool::Calibrate` (shortcut `k`; not a markup; `Tool::ALL` now 25 variants)
    with a two-click canvas interaction and a live length badge
  - `calibrate_scale()` in `domain/units.rs` delegates to `Scale::from_ratio`
    (7 tests)
  - `CalibrationDialog` window pre-filled with the measured length; applying
    pushes one undo snapshot

- **v0.4-06**: resize grips (gap #5, first slice, PR #96)
  - `Corner` enum + `Markup::resize_bounds()` in `domain/markup.rs`, clamping to
    a 1×1 pt minimum and skipping non-Bounds geometry (4 tests)
  - Canvas corner-handle hit-testing (`handle_at`) with drag-to-resize; moving
    and resizing are mutually exclusive; one snapshot on drag end

- **v0.4-06 follow-up**: segment-endpoint and path-vertex editing (gap #5
  complete, PR #98, commit `cac048b`)
  - `Markup::draggable_points()` exposes stable-index structural points (segment
    endpoints, every path vertex, all three angle points); empty for bounds,
    ink, text, callout and count markups
  - `Markup::move_point(index, pt)` moves one point and returns `false` when the
    index is out of range; locking and measurement recomputation stay with the
    caller
  - Point handles replace the bounding-box corner handles for those kinds;
    `point_handle_at` hit-testing runs before the corner check, and a drag
    reshapes every selected markup at the same index with one snapshot on drag
    end

- **v0.4-07**: tool chest (gap #7, PR #96)
  - `StylePreset::from_style()` and `StylePresetLibrary::save_named()`
    (case-insensitive dedupe) in `domain/style_preset.rs` (3 new tests)
  - New **My Tools** left panel (`ui/tool_chest.rs`): preset create/rename/
    delete, apply-to-selection, and an apply-at-creation hook in `finish_markup`
    via `UiState::pending_style_preset` (stamps excluded)
  - View ▸ **Show Tool Chest** toggle

- **v0.4-08**: editable Markups List console (gap #8, PR #96)
  - Inline per-row status ComboBox (dimmed when the row is locked) applying
    `apply_status_to`, with one undo snapshot per change
  - Measurement totals footer over the **filtered** view: per-kind subtotals,
    area squared, count markers
  - **Export CSV…** with an `rfd` save dialog (Page, Kind, Author, Subject,
    Status, Measurement, Comments columns); `draw_markups_list` returns
    `(Option<MarkupListAction>, bool)` so the caller can signal a snapshot

### Fixed

- **Build** (PR #96): `Cargo.lock` bumped the yanked `async-executor` 1.14.0 to
  1.14.1 so dependency resolution no longer pulls a withdrawn crate.

- **Audit follow-ups**: additional medium fixes from the v0.3 full-app audit
  - **M18** (PR #90): the canvas renders the PDF page background at a width that
    tracks the current zoom (oversampled, quantised to coarse steps) so page
    text stays sharp when zoomed in, instead of a single ~144 dpi raster
  - **M24** (PR #90): the inspector's layer-name drafts, stroke-colour commit
    debounce and last-scale-page marker moved out of thread-locals into
    `UiState`
  - **WCAG** (PR #90): the Markups List status text uses a darkened
    `ReviewStatus::text_color` on the light theme (the bright amber swatch was
    only ~1.9:1 on white); the dark theme keeps the swatch. A domain test
    asserts WCAG AA against the light panel and stripe
  - **M5** (PR #91): menu items show their keyboard shortcuts (`Button::
    shortcut_text`); `Ctrl+A` Select All added and shared by the menu and the
    keyboard; the hint bar lists Open/Select All/Esc
  - **Quit** (PR #92): the unsaved-changes dialog gained **Save and Quit**,
    which only closes after a successful save

- **v0.3-02 Phase 1**: hayro-syntax page-size probing for pure-Rust PDF engine adoption (H8 follow-up)
  - Added optional `hayro-syntax v0.8` dependency with `hayro` feature flag
  - Implemented `probe_pages_hayro()` using hayro-syntax v0.8 API
  - Integrated with `read_pdf_info()` for PDF page-size detection using pure Rust
  - PDFium remains the default page-size probing engine; hayro used when feature enabled

- **v0.3-02 Phase 2**: hayro rendering pipeline for pure-Rust PDF to PNG conversion (H8 follow-up)
  - Added optional `hayro v0.8` dependency for PDF page rendering
  - Implemented `render_page_hayro()` using hayro's render API
  - Updated `render_page_to_cache()` to use hayro when `--features hayro` is enabled
  - Made PDFium code paths conditional on hayro feature (PDFium code only compiles when hayro is disabled)
  - Added CI test job for hayro feature: `cargo test -p rubric --features hayro`
  - Added `hayro_render_page_to_cache_works` test for hayro-specific PNG rendering
  - PNG output format parity with PDFium renderer

- **v0.3-02 Phase 3**: corpus fidelity tests for hayro engine validation
  - Created `docs/tasks/v0.3-03-corpus-fidelity.md` task brief
  - Plan for ≥95% pixel match validation against PDFium baseline
  - Test corpus of 10 representative PDFs (text, images, vector, rotated pages)

- **v0.3-04**: PDF export/print fixes (M4/M5)
  - Created `docs/tasks/v0.3-04-pdf-export-fix.md` task brief
  - Fix error handling for failed PDF background rendering
  - Fixed-DPI (150 DPI) export instead of view-relative
  - Page-cropped output for export/print
  - Shipped (M5) in PR #88, commit `8bc2601`: the export render pass suppresses
    the backdrop, page border, grid and rulers, and the PDF overlay crops
    markups to the page bounds

- **v0.4-02**: batch status for multi-select (gap #3, PR #88, commit `8bc2601`)
  - `EditorAction::BatchStatus(ReviewStatus)` (`ALL` now 14 entries) and
    `Document::apply_status_to(ids, status)`
  - Status dropdowns in the Inspector and the Markups List when several markups
    are selected; the batch is one undo snapshot (`push_snapshot("batch status")`)
    and reports how many markups changed
  - Locked or non-editable selections are skipped, with an honest status message
    when nothing changes

- **v0.3-05**: lopdf write support
  - Created `docs/tasks/v0.3-05-lopdf-write-support.md` task brief
  - Plan to add lopdf dependency for PDF export functionality
  - Implement File > Export PDF… menu

- **v1.0-0**: installer verification
  - Created `docs/tasks/v1.0-0-installer-verification.md` task brief
  - Verify multi-platform installers (Linux, macOS, Windows)
  - Verify §2.5 legal checklist for public release
  - Re-audit README/ROADMAP claims

### Fixed

- **M4 (v0.3-04, PR #68)** — exporting or printing a page whose PDF background
  cannot be produced (missing PDF, engine failure, or an undecodable render)
  now fails with a clear status message instead of silently writing/printing a
  blank white page. A unit test covers the missing-PDF case.
- **M3 / M2 (v0.3-06, PR #69)** — `save_project` is now atomic (serialise
  first, sibling temp file, `sync_all`, rename over the target; a failed rename
  removes the temp file; unit-tested), and File ▸ Clear Recent reports the real
  outcome instead of claiming success when `save_recent` fails.
- **M7 / M9 (v0.3-07, PR #70)** — the wheel zoom uses the domain `view`
  constants instead of magic numbers, and the status-bar cursor readout reports
  the true pointer (matching what a click selects) rather than the snapped
  point.
- **Full-app audit wave (v0.3-08)** — three blockers and the majority of the
  majors from `docs/audits/v0.3-full-app-audit.md`:
  - **B1** — Backspace/Delete while typing in any text field no longer deletes
    the selection; Ctrl+Z/Ctrl+Y and PageUp/PageDown are keyboard-focus-gated.
  - **B2** — New / Open Project / Import PDF / Recent / Ctrl+O park behind a
    "Replace Unsaved Project?" confirmation instead of discarding work.
  - **B3** — dragging a measurement markup now recomputes its value, so labels
    and autosaves stop lying after a move.
  - **M1** — the OS close button is vetoed while the project is dirty and
    routed through the same confirmation as File ▸ Quit.
  - **M2** — autosave writes only the recovery sidecar and no longer clears
    the dirty flag (status now reads "Recovery snapshot saved").
  - **M3** — a crash report parked by a previous run is now loaded at startup
    and shown once in the review-and-send dialog.
  - **M4** — Help ▸ About shows a real window (version, licence, links).
  - **M7** — Delete is disabled when every selected markup is locked.
  - **M8** — the update-check window can be dismissed while a check runs and a
    poisoned update mutex no longer panics the UI loop.
  - **M9** — a 0-page PDF import is rejected honestly instead of reporting
    "Imported 1 page(s)".
  - **M10** — Edit ▸ Select All, View ▸ Previous/Next Page and View ▸ Full
    Screen.
  - **M11/M14–M16/M20/M21 (canvas)** — the commit-click trap, screen-space hit
    tolerance, degenerate-drag rejection, drag-to-select-then-move, the note
    popover lock check, and empty-markup cleanup on a cancelled first edit.
  - **M17/M25 (PDF engines)** — hayro page-size probing now reports the same
    crop∩media rotation-applied dimensions as PDFium; encrypted PDFs surface a
    single-line honest message on both engines; render-cache writes are
    atomic; the render width is clamped to the u16 pixmap ceiling; PDFium
    discovery also tries the exe-dir `lib/` subdirectory.
  - **M22–M24 (inspector)** — locked markups no longer accept edits that are
    silently discarded (with phantom undo snapshots); "N selected" replaces
    edits landing on an arbitrary hash-order markup; stroke-colour changes
    commit with exactly one snapshot.
  - **M26 (export/print)** — exports and prints render through a fixed 150 DPI
    page transform instead of the user's pan/zoom editor view.
  - **M27 (packaging)** — the installer workflows fetch the pinned PDFium
    runtime themselves and fail rather than ship a PDF-less installer; also
    fixes the Windows portable-zip `$env:LOCALAPPDATA` bash bug.
  - **M28–M30 (docs)** — `docs/pdfium.md` documents both engines; traceability
    and DECISIONS match the actual shipping policy; CONTRIBUTING/development
    merge-authority wording matches the 2026-10-26 directive.

### Added

- `docs/audits/v0.3-full-app-audit.md` — the full-app audit report (10 parallel
  workstreams + capability parity + design tooling).
- `docs/research/v0.4-capability-gap-analysis.md` — code-verified parity
  analysis vs Bluebeam Revu / Adobe Acrobat with the ranked v0.4 gap list.
- `docs/research/design-tooling-2026-11.md` — free/OSS design tooling for an
  egui app (icons, palettes, fonts, `egui_kittest` visual evidence) with an
  adoption plan.
- Domain tests for `remove_page` re-indexing, open vs closed perimeters,
  ruler-tick overshoot/cap, diff layer moves, style-preset lock handling and
  list-filter case folding; canvas tests for hit tolerance and ink-stroke
  validity; data tests for atomic render-cache writes and the render-width
  clamp.

### Planned

- **Bates numbering** (future slice **v0.6**, spec F-801–F-805): sequential page
  stamping for legal/document-review workflows, implemented as a non-destructive
  overlay (source PDF untouched). Queued after the remaining v0.5 slices.
  Capability-level research recorded in `docs/research/bates-numbering.md`;
  capability inventory row 32; requirement R-S-018.

## [0.3-01] - 2026-10-31

### Fixed

Audit-fix wave `v0.3-01` (`docs/audits/v0.2.0-full-audit.md`, all high-severity
defects except H8 (PDFium bundling, deferred to the v0.3 engine slice) and the
in-scope medium findings):

- **H1** — crate versions bumped `0.1.0` → `0.3.0`: binaries self-report the
  right version and Help ▸ Check for Updates no longer reports the running
  build as outdated.
- **H2** — switching a page's display unit now preserves physical lengths
  (the conversion reads the scale *before* the ComboBox write).
- **H3** — count markers number sequentially per page (highest existing number
  + 1; deleting a marker never reissues its number) and a running
  `N count` total badge is drawn per page.
- **H4** — strikeout, underline and squiggly render as their real strokes
  (mid-line, bottom line, zig-zag) instead of plain rectangles, on canvas and
  in PNG export.
- **H5 / H9** — status, author, subject, stroke-colour/width, project name,
  document title and project-author edits are committed: they push one undo
  snapshot and mark the project dirty, so Ctrl+Z reverts them and autosave
  persists them.
- **H6** — print failures (render, temp-write, spool) surface in the status
  line instead of failing silently, and the temporary PNG is always removed.
- **H7** — Fit Page scales the current page to the actual canvas viewport
  (with margin) instead of resetting zoom to 100 %.
- **M1** — File ▸ Quit asks for confirmation when the project has unsaved
  changes.
- **M6** — the Pages panel renders thumbnails in multiple columns using the
  pre-computed layout instead of stacking every cell in one column.
- **M14** — Markups List gains the Kind and Page filters the domain already
  supported.
- **M15 / M16** — angle and count no longer draw two overlapping labels; all
  geometry-carried measurement text (angle degrees, count numbers) obeys the
  Show Measurements preference, which now has a View-menu checkbox.
- **M17** — Scale / unit / precision controls are reachable while a markup is
  selected (the section moved above the selection branch).
- **M18** — `Scale::describe()` anchors on one inch (72 pt), so `1 in = 1 ft`
  describes as `1 in = 1.000 ft`, not `12.000 ft`.
- **M19** — Undo/Redo menu entries are disabled when history cannot serve
  them, in both the Edit menu bar and the shared context menu.
- **M20** — layer rename pushes a snapshot only when the name actually
  changed.
- **M21** — the spec no longer overstates ink: single stroke per gesture is
  the documented canvas behaviour; multi-stroke Ink markups remain a
  domain-level capability.
- Markups List filter row no longer clips the last control (labels moved into
  the combo `selected_text`), truncated cells gained hover tooltips, and the
  top toolbar's tool names now match the Tools panel.

### Added

- Headless tests for `History` (undo/redo round-trip, redo-branch truncation,
  the 128-snapshot cap, overwrite, clear) and the `view` zoom constants
  (bounds, presets, clamping, idempotence), plus count-numbering and
  `Scale::describe()` tests.
- `AGENTS.md` records the 2026-10-31 owner directive on parallel workstreams.

## [0.2.0] - 2026-10-30

### Released

All 52 spec feature IDs and 17 requirement groups implemented and CI-verified,
published as Rubric v0.2.0:
https://github.com/Perchemop/rubric-desktop/releases/tag/v0.2.0

Release assets (unsigned, Q-PACK-1):
- rubric-linux-arm64.zip, rubric-linux-x86_64.zip
- rubric-macos-arm64.zip
- rubric-windows-x86_64.zip

Dual-licensed MIT OR Apache-2.0 (Q-LIC-1 resolved).

Highlights since v0.1.0: note popover with threaded comments (F-103), zoom
presets 50/100/200% (F-401), layer rename/reorder with real layer-major paint
order (F-004), accessibility contrast audit with audited body-text colours and
labelled controls (F-605), capability inventory (guide II.27.3, gap G3 closed),
git-identity rule re-homed (guide II.26.7, gap G6 resolved), release legal
checklist recorded (guide §2.5, gap G5 resolved). Automated UI-screenshot
evidence (guide II.27.12, gap G4) remains a documented deferral.

### Added
- `AGENTS.md` now states the git-identity rule (every git action authored
  `Rubric Dev <dev@example.com>`), re-homing guide II.26.7 — process gap G6
  resolved. Automated UI-screenshot evidence (G4) is confirmed as a documented,
  owner-approved deferral.
- `docs/capability-inventory.md` (guide II.27.3): one reviewable row per
  capability — source URL, acceptance criteria and evidence — closing process
  gap G3. `docs/parity.md` and `docs/guide-compliance.md` updated to match.
- Accessibility pass (F-605): a pure domain module `domain/a11y.rs` computes WCAG
  relative luminance and contrast ratio; a headless test audits the shipped
  chrome palettes (body text passes AA, accent selection/link pairs pass
  AA-large). Body-text colour is now set explicitly per theme (a lighter
  dark-theme link accent), icon-only controls (layer visibility/lock/reorder)
  gained tooltips, and the page-navigation arrows are labelled. See `docs/a11y.md`.
- Note popover (F-103): selecting or creating a Note pin opens a floating comment
  popover positioned beside the pin (threaded comments via `Markup::add_comment`,
  which refuses blank text); the popover has a draft box, Add comment / Close
  buttons, and closes on Escape or click outside / note deletion.
- Zoom presets (F-401): the View menu offers 50 % / 100 % / 200 % buttons,
  backed by the new pure domain `view` module (`MIN_ZOOM`, `MAX_ZOOM`,
  `ZOOM_PRESETS`, `clamp_zoom`).
- Layer rename and reorder (F-004): the Inspector layers panel gains an
  editable name field and ▲/▼ reorder buttons. Paint order is now layer-major
  (`Document::markups_in_paint_order`), so reordering layers changes stacking;
  the default layer is pinned at the bottom and z-order commands are scoped to
  the same page and layer.
- v0.2-01 domain foundation for reviewer workflows: `MarkupList` snapshot
  (case-insensitive filters incl. comment/summary search, stable sorts,
  grouping with numeric page order), `StylePreset`/`StylePresetLibrary`
  (capture/apply, 64-entry cap, upsert dedupe) and `DiffResult`
  (`diff_documents`/`diff_projects`, 2-dp geometry fingerprints,
  style/locked-aware change detection). All serde-serialisable and
  headless-tested, plus integration coverage in `tests/domain_tests.rs`.
- Markups List UI (R-V02-01): resizable bottom panel built on the `MarkupList`
  domain model, with search and status/author/subject/layer filters, every
  sort and group mode, row selection that jumps to the markup's page, empty
  states, Clear filters, and a View → Show Markups List toggle.
- Selection editing completion (F-114/F-115): Copy/Paste Properties apply only
  the visual style to selected editable markups (one undo snapshot, locked
  targets skipped), and page-scoped Bring Forward/Backward/To Front/To Back
  reorder markups without crossing page boundaries. Commands are available in
  the shared canvas/inspector context menu and the Edit menu.
- Autosave wiring (F-502): `File → Auto-save` enables it and picks an interval;
  a dirty project is written at most once per interval to
  `<name>.autosave.rubric.json` beside the project (app data dir when
  untitled), and the status bar reports the path or the failure.
- Personal-data and secret scan gate (guide II.29, owner directive 2026-10-27):
  `scripts/scan-personal-data.sh` fails on a non-CI home path or a real email in
  the tracked tree, `scripts/scan-release-assets.sh` scans extracted release
  archives, and `Personal data scan` is a required check in `.ci-required.txt`.
- Removed a tracked `scripts/__pycache__/*.pyc` that embedded an absolute source
  path (found by the new scan), and ignored `__pycache__/` and `*.pyc` so a
  bytecode cache cannot be committed again.
- `docs/guide-compliance.md`: rule-by-rule review of the local `guide.md` and
  `addendum.md`, re-reviewed 2026-10-28 for the revised guide (II.29 and the
  suite-wide Part II), with evidence, the deliberately-adapted rules, and seven
  tracked gaps.
- `.ci-required.txt`: the authoritative CI checks that decide `ci.sh`'s verdict
  (guide II.20.4 / II.25).
- Project schema versioning (F-507): every `.rubric.json` is stamped with
  `schema_version`; legacy files without it load as the current version, and
  `load_project` refuses a newer/unknown version instead of mis-reading it.
- Stirling PDF added as capability-level inspiration (client-directed):
  original capability summary in `docs/research/stirling-pdf-capabilities.md`,
  inspiration table row and radar addendum; no code or dependency adopted.
- Community and governance files (operating guide Part II / opensource.guide
  alignment): `CODE_OF_CONDUCT.md`, `SECURITY.md` (private reporting, threat
  model, incident response), `ACCESSIBILITY.md`, `VISION.md`,
  `GOVERNANCE.md`, `CONTRIBUTORS.md`, and issue templates for bugs, feature
  requests and accessibility reports (`accessibility_report.yml`).
- CI security workflow `.github/workflows/security.yml`: secret-pattern
  scan (`scripts/scan-secrets.sh`), PR dependency review, `cargo-deny`
  licence/advisory gate (`deny.toml`), SBOM generation (SPDX + CycloneDX),
  and CodeQL SAST for the workspace.
- `.github/dependabot.yml`: weekly cargo + GitHub-Actions update PRs.
- Repository security posture: branch protection on `main` (required check,
  admin-enforced), private vulnerability reporting, Dependabot security
  updates, Discussions enabled, area/a11y labels.
- `docs/metrics.md` — which project metrics are tracked and why.

### Changed
- Redo is now **Ctrl+Y** (`Ctrl+Z` remains undo); the hint bar, README, and the
  F-601 shortcut record were updated.
- `README.md`: adds a "Try Rubric without Rust" release-download path, corrects
  the workspace layout, and documents the contributor build/test commands.
- `docs/qc/checklist.md`: the per-Task merge step now matches the 2026-10-26
  owner directive (agent merges once CI is green and findings are addressed).
- `docs/current-model-map.md`: refreshed with the live 2026-10-27 free-model
  catalog and the Context7/`gh` tooling status.
- `AGENTS.md`: owner directive 2026-10-26 — agents merge completed PRs once CI
  is green and review findings are addressed, rather than waiting for a manual
  merge click.
- `.github/PULL_REQUEST_TEMPLATE.md`: UI changes now must pass the
  accessibility checklist items.
- `CONTRIBUTING.md`: links the conduct/security/accessibility paths.
- Global workflows pin every GitHub Action to a commit SHA (supply-chain
  hardening, guide Part II §20.3).
- `ATTRIBUTION.md`: adds the CI-enforced dependency licence policy and the
  AI-tooling terms record; `docs/current-model-map.md` records the research
  method and model-terms rules.
- Crate `license` metadata unified on `MIT` to match `LICENSE`; dual
  "MIT OR Apache-2.0" adoption is open question Q-LIC-1.

### Fixed
- Markup diff no longer misses style-only or lock-only changes; `StylePreset::apply_to`
  refreshes `modified`; `diff_projects` treats a project without an active
  document as empty instead of reporting "no changes"; Page groups sort
  numerically (1, 2, 10); duplicate IDs in malformed documents are diffed once.
- `release.yml`: the publish step finalises a draft release after uploading
  assets (a re-created tag turns the existing Release entry into a draft,
  which had left v0.1.0 unpublished until edited).
- Dependency hygiene findings from the new `cargo-deny` gate: yanked
  `clipboard-win` 5.4.1 → 5.4.3 in `Cargo.lock`; explicit `version` on the
  two workspace path dependencies (cargo-deny treats versionless specs as
  wildcards); `epaint_default_fonts` font licences (OFL-1.1,
  Ubuntu-font-1.0) documented in `deny.toml` and `ATTRIBUTION.md`.

### Security
- Removed a maintainer absolute home path from published git history (guide
  II.29). It was carried by agent configuration and a tracked Python bytecode
  cache; both were removed from the tree first, then every branch, tag and
  commit was rewritten with `git filter-repo` and force-pushed, and branch
  protection was restored. The `Personal data scan` required check prevents a
  recurrence. The incident is recorded in `SECURITY.md` and the value is not
  reproduced. **Commit SHAs recorded before 2026-10-28 no longer resolve** after
  the rewrite.

## [0.1.0] - 2026-10-24

### Summary

First functional, installable build of **Rubric** — a free, cross-platform PDF
markup, review and measurement tool. The full reviewed scope (52 feature IDs,
17 requirement groups) is implemented and CI-verified; what remains is polish
and packaging follow-ups, listed honestly below. Built and verified on Linux
(aarch64 and x86_64), with macOS and Windows artefacts produced by CI.

### Roadmap progress

- Requirement groups: 13/17 fully done, 4 partial (small residuals), 0 not started.
- Feature IDs: 46/52 done (88 %).
- Evidence: `REQUIREMENTS_TRACEABILITY.md`, CI runs, and hands-on verification
  of the installed binary on the primary platform (Linux aarch64).
- Details and next milestones: `ROADMAP.md`.

### Added
- Original name/brand "Rubric": SVG logo, multi-size PNG icons (16/32/48/64/128/256/512/1024), branding spec.
- Fresh Rust crate `rubric` (egui/eframe 0.36, serde, uuid, thiserror, tracing, chrono, image).
- Domain-first architecture: geom, units (scale + conversions), style (colours/dashes/fill modes), markup (23 kinds, geometry variants, measurements, hit testing, review status, comments), document (pages/markups), project (toolsets, settings, history with undo/redo capped 128).
- Data layer: JSON persistence (`.rubric.json`), app data dir via directories, error types.
- UI skeleton: app shell (panels, menu, shortcuts), canvas (page, markups, selection, basic place/move), toolbar (tool groups), inspector (properties/comments/measurements), theme/icons.
- Tests: unit tests in domain modules + integration tests (6) covering geometry, measurements, serialisation, project lifecycle.
- Tooling: .gitignore (excludes secrets/target/local notes), LICENSE (MIT), rust-toolchain.toml, CI skeleton, scripts/gen-icons.sh.
- Docs: README, AGENTS, architecture, data model, test plan, assumptions/questions, implementation plan, requirements traceability, decisions, progress, changelog.


### Added
- Suite infrastructure (guide §7): shared crates `crates/crash-report` and `crates/update-check`, consumed by the app and reusable by any future app in the suite. **crash-report** installs a panic hook that captures the app version, OS and a scrubbed backtrace (home directories and token-like material redacted), shows a review-and-cancel dialog with editable notes, and then submits either through a pre-filled "New Issue" URL (no account in-app) or a **user-supplied** GitHub token held only in memory — a privileged bot token is never embedded. **update-check** queries the GitHub Releases API on demand (Help ▸ Check for Updates…), compares versions semantically (so 0.2.0 correctly beats 0.10.0), fails soft offline, caches for an hour and never polls in the background. 25 new tests across the two crates (12 + 13). The workspace was restructured to the guide's suite layout (`apps/rubric` + `crates/*`) with unchanged build commands and output paths. Privacy is documented in `README.md`.
- Process files required by guide v4 (suite-ready edition): `.github/PULL_REQUEST_TEMPLATE.md` (acceptance criteria, tests, CI, no-proprietary checklist), `ROADMAP.md` (staged milestones with evidence-based readiness: 13/17 requirement groups and 46/52 feature IDs done), `CONTRIBUTING.md` (PR licence terms, DCO sign-off), `docs/design/brand.md` (original mark and palette rationale), and `docs/research/` with the summaries-plus-URLs policy and its human approval gate.
- Packaging: CI release and native installers (spec F-701/F-703/F-704). `.github/workflows/release.yml` builds the release binary for Linux x86_64, macOS arm64 and Windows x86_64 and attaches each archive to a GitHub release on `v*` tags (a manual `workflow_dispatch` publishes a pre-release, so a test run never looks like a real release). `.github/workflows/package.yml` builds the native installers — a Linux **AppImage** (linuxdeploy over an AppDir carrying a `.desktop` entry and appstream metadata, plus a portable tarball fallback), a macOS **`.app`** → **`.dmg`** bundle, and a Windows **MSI** via WiX v4 whose product definition is kept in-repo (`packaging/wix/Product.wxs` with the MIT licence text) so the installer metadata stays reviewable. `docs/pdfium.md` documents the PDFium run-time contract (discovery order `RUBRIC_PDFIUM_PATH` → executable directory → system path, where to obtain binaries, and what a failed import looks like). Installers ship **unsigned** — no signing certificate is committed to this repository (recorded as Q-PACK-1; raster icons as Q-PACK-2).
- Page thumbnails sidebar (spec F-404): a resizable left panel showing one cell per page. PDF-backed projects show a real preview rendered through the shared on-disk page cache (so a page the main canvas already rendered costs nothing); projects with no PDF still show numbered placeholders, so navigation is never a dead control. Clicking a cell jumps to that page. The thumbnail grid packing (`thumb_layout`: columns that fit, rows needed, cell size) lives in `src/domain/thumbs.rs` as pure geometry with tests. Page switching (top-bar arrows, Page Up/Down, sidebar) now shares one `RubricApp::jump_to_page` helper so all three agree on text-edit commit, undo snapshot and draft-abandon semantics. 6 new domain tests.
- Canvas grid, snap-to-grid and rulers (spec F-405/F-406): new pure domain module `src/domain/grid.rs` — [`Grid`] spacing/visibility/snap with nearest-intersection snapping, plus a 1/2/5 `ruler_ticks`/`nice_step` ladder — and canvas rendering in `src/ui/canvas.rs` (`draw_grid` under the markups, skipped when the pitch falls below 6 screen pixels; `draw_rulers` labelled page-point ticks on the top/left margins that follow pan and zoom). Snapping applies to markup *creation anchors* only: hit-testing keeps using the exact pointer position, so a markup that is not grid-aligned remains selectable. View menu gains **Show Grid**, **Snap to Grid** and a **Grid Spacing** submenu (12/24/36/72 pt). Grid state is session-only and never enters `.rubric.json`. 10 new domain tests.
- Editor context menus (spec F-603): a shared menu (`src/ui/menu.rs`) attached to the canvas page response and to the inspector panel response, offering a Tool submenu (all 24 tools with their F-601 shortcut keys), Delete Selected / Clear Selection (disabled with no selection), and Undo / Redo / Zoom In / Zoom Out / Fit Page. Commands are modelled by the new domain enum `EditorAction` (`src/domain/action.rs`: `ALL`, `label`, `is_enabled`) and executed through one path (`RubricApp::apply_editor_action`) shared with the Edit/View menu bar. The canvas opens the menu only on a plain right-click — a right-click during a multi-point draft still commits that draft. 4 new domain tests; the UI renders the model and only *requests* actions (`UiState::pending_action`).
- Per-tool keyboard shortcuts (spec F-601): `Tool::ALL` (all 24 variants in declaration order), `Tool::shortcut_key()` and `Tool::from_shortcut_key()` in `src/domain/project.rs` (letters for markup/shape tools, digits `1`–`7` for measurements, ASCII case-insensitive reverse lookup), plus UI wiring in `src/ui/app.rs` (`select_tool` abandons any in-flight draft gesture exactly like Esc while keeping the selection; `key_to_shortcut_char` maps egui `Key` variants). Only unmodified keys fire — blocked while text editing, while a widget wants the keyboard, or with Ctrl/Alt/Super held (Shift is allowed since the `Key` variant is matched, not the typed character). 9 new domain tests (ALL completeness/order, every tool has a key, pairwise-unique keys, round-trip, case-insensitivity, unknown-key rejection, documented mapping table, serde unaffected).
- Dark theme (spec F-602): View > Theme menu toggles Light/Dark, persisted as an app-level preference (`AppPrefs` in the OS config dir) via `set_theme`; egui 0.36 per-theme visuals use Rubric's brand palette (light paper chrome vs. indigo dark surfaces) while the document page itself stays light. 9 new tests (5 domain `Theme`/`AppPrefs` incl. serde round-trip and legacy default, 3 prefs store incl. corrupt-file fallback and disk round-trip, 1 `apply_theme` theme/visuals selection).
- Page image export + print (spec F-505): `src/data/render.rs` renders the current page headlessly (headless egui frame + CPU triangle rasterizer with texture sampling and alpha blending) to PNG, and `src/data/print.rs` sends the rendered page to the OS print command (lp / lpr / PowerShell Start-Process -Verb Print); File menu gains `Export Page Image…` and `Print…` with honest status messages. 7 new unit tests.
- Phase 5 PDF import + page navigation (spec F-504/F-403): `Project::from_pdf` (per-page Letter/A4 sizing, read-only source path, serde-defaulted `source_pdf`), `data::pdf` runtime binding to PDFium via `pdfium-render` with a hash-keyed PNG render cache and `RUBRIC_PDFIUM_PATH` discovery, lazy background page textures (~144dpi, poisoned per cache key so failed pages aren't retried every frame), top-bar + PageUp/PageDown page navigation (guarded while in-canvas text editing), Import PDF… via native dialog, shared `install_project` path; 14 new tests. CI fetches the PDFium library and runs the lib-dependent tests against it.
- Phase 1 requirement set and spec (`docs/spec/README.md`): 52 feature IDs across F-1xx…F-7xx with priorities mapped to implementation phases; `REQUIREMENTS_TRACEABILITY.md` extended with spec-level requirements R-S-001…R-S-017 and seven group rows.
- Phase 2 slice A — interactive canvas: drag-to-draw with live preview (bounds/segment), ink freehand drawing, wheel zoom anchored on cursor (10–1600%), Space/middle-button pan, Del to delete selection.
- Phase 2 slice B — multi-point drawing: click-to-append polyline/polygon/perimeter/area paths with rubber-band preview (commit via double-click/right-click/Enter, Esc cancels), three-click angle tool with live ray preview.
- Phase 3 — measurement labels rendered on canvas (domain-formatted, geometry-anchored, screen-space size); inspector scale editor with presets (1:1, 1 in = 1/10/50 ft), display-unit combo, and custom paper-inches : real-units ratio; scale/unit edits recompute page measurements with one undo snapshot per commit.
- Measurement precision preference: `Settings::precision` (0–6 decimal places, serde default 3 so older `.rubric.json` files load unchanged), `format_value_precise`/`Measurement::label_with`, and an inspector DragValue that commits one snapshot per edit. Angle stays at 2 decimals, counts at 0.
- Layer visibility/lock: `Layer` domain model (visible/locked/z, serde-migrated — legacy `.rubric.json` files load onto a per-document default layer), document ops (`add_layer`, `remove_layer` refusing default/populated layers, visibility/lock setters, `is_markup_visible`/`is_markup_editable` predicates, fail-open on unknown ids), inspector Layers section (eye/lock toggles, add/delete with status feedback), layer ComboBox for the selected markup, and canvas/inspector/delete gates so hidden markups aren't painted or hit-tested and locked markups can't be moved, deleted, or edited. 15 new tests.
- Stamp library (spec F-111): `StampPreset` domain enum (Approved/Rejected/Draft/Check/Cross with word labels, drawn glyphs ✓/✗, meaning-grouped palette colours), `Markup.stamp: Option<StampPreset>` (serde-defaulted — legacy stamps load as None and render unchanged), `with_stamp` builder; canvas renders the preset glyph centred with shrink-to-fit; stamp creation applies the active preset + colour; inspector preset picker for selected stamps (one snapshot per real change) and a tool-options selector for placement (7 new tests).
- In-canvas text editing (spec F-104/F-105): double-click to edit Text and Callout markups in place (multiline overlay anchored to the markup, focus-loss commit with one snapshot only on real change, Esc cancels); placement now creates proper `Text { at, wrap, text }` / `Callout { rect, tail, text }` geometry instead of bare bounds, with the editor auto-opening on creation; interaction arms (draw/move/delete/undo) suppressed while editing; empty text markups get a visible outline.
- Native file dialogs + recent files (spec F-5xx): rfd-based Open (Ctrl+O) and Save/Save As (Ctrl+S routes through the current path, `.rubric.json` extension normalised case-insensitively) with a Recent submenu (hover shows full paths, missing files self-prune, Clear Recent); recents persist app-level in the OS config dir (`RecentList` domain type + `data::recent` store) — never written into shareable project files; window title shows the active file; Quit goes through eframe teardown. `rfd` uses the pure-Rust xdg-portal backend on Linux (no GTK dependency).
- Hit testing v2: filled shapes hit interior, stroke-only shapes hit outline only; elliptical edge test; polygon even-odd interior test; ink segment-based (incl. dot strokes) — 8 new unit tests.
- Status workflow per spec (F-301/F-305): `ReviewStatus` remapped to Open / In review / Approved / Rejected with distinct palette colours, inspector status legend (live counts per status), colour swatches in the status selector, coloured status dots on canvas. 6 new tests; legacy status strings no longer load — sanctioned since no released files exist.
- Subagent suite (`.opencode/agents/`): `domain-dev`, `test-writer`, `ui-dev`, `reviewer`, `docs-writer`, `fixer` — all on free OpenCode models, restricted to `cargo fmt` + read-only git locally.
- CI-first build policy: all heavy gates (`cargo test`, `clippy -D warnings`, `check`, release build) run only in GitHub Actions; workflow now caches builds with `Swatinem/rust-cache@v2`.


### Changed
- CI verified fully green (fmt, clippy, tests, release build) after billing resolution; process docs updated accordingly.


### Fixed
- Area measurement rescaling: area values now scale with `units_per_point²` (previously used the linear factor, and carried an unnecessary cast).
- Switching the display unit now preserves physical measurements (`Scale::with_display_unit` converts `units_per_point` via `points_per_unit`) instead of silently reinterpreting the scale ratio.
- Undo history no longer spams snapshots during selection drags (one snapshot per completed drag instead of dozens).
- Esc now abandons an in-progress drag-to-draw instead of re-anchoring the draft.
- Tool switching now discards an in-progress multi-point draft (no ghost preview or stale Enter/right-click commits).
- Segment tools (line/arrow/length/diameter/radius) no longer commit wrong-typed geometry on a bare click; multi-point commits validate distinct points; double-click no longer adds a duplicate final vertex.
- Enter/Space fake-clicks while a text field has focus can no longer trigger canvas commits.
- Docs: stale markup-kind count corrected — `MarkupKind` has 23 variants (traceability, data model, plan, progress, decisions, and the 0.1.0 note said 22).


### Docs
- Adopt the guide §5 **CTO.new-style free process**: `Chat (plan) → Task (execute) → Pull Request (review) → Merge`. `AGENTS.md` documents the operating model; `docs/development.md` reframes the slice workflow as Tasks with a PR-for-review step; `.opencode/agents/orchestrator.md` becomes the Chat/planning role and a new `.opencode/agents/coder.md` is the Task executor; `docs/qc/checklist.md` Definition of Done gains the Pull Request + human-merge items. Added the required `docs/parity.md` capability-parity tracker. GitHub Actions is named the free "Task Runner" throughout.


### Known limitations

- PDF import needs the PDFium shared library next to the executable (or `RUBRIC_PDFIUM_PATH`); every other feature works without it (see `docs/pdfium.md`).
- Planned follow-ups: note popovers (F-103), copy properties (F-114), z-order (F-115), autosave wiring (F-502), `schema_version` (F-507), accessibility pass (F-605) — tracked in `ROADMAP.md`.
- Installers are unsigned (no signing certificate in the repository, Q-PACK-1); desktop icons are SVG-only pending a raster set (Q-PACK-2).
- Crash reports and update checks are on-demand only; nothing is sent or queried without an explicit user action (see README privacy section).

### Notes
- Headless-testable (domain + data). UI requires display at runtime; builds/tests/lint/format work in headless environment.
- PDF import binds to PDFium at run time (`docs/pdfium.md`); the page renders as a blank rectangle until the library is present.
- Never commits external reference documents or the build guide.
