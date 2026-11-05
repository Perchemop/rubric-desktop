# Implementation Plan

## Phase 0 — Environment, brand, repo, skeleton
- [x] Invent original name: Rubric
- [x] Create fresh repo at ~/rubric
- [x] .gitignore excludes external material, secrets, target
- [x] Branding: logo.svg (512x512), icon PNGs 16/32/48/64/128/256/512/1024
- [x] assets/branding.md, README.md, AGENTS.md
- [x] LICENSE (MIT)
- [x] rust-toolchain.toml
- [x] Cargo.toml (eframe/egui/serde/uuid/thiserror/tracing/chrono/image)
- [x] docs/architecture.md, docs/data-model.md, docs/test-plan.md, docs/assumptions-and-questions.md
- [x] IMPLEMENTATION_PLAN.md, REQUIREMENTS_TRACEABILITY.md, DECISIONS.md, PROGRESS.md, CHANGELOG.md
- [x] Domain model complete (geom, units, style, markup, document, project) with tests
- [x] Data layer (store, error, JSON persistence)
- [x] UI skeleton (app, canvas, toolbar, inspector, theme, icons)
- [x] cargo clippy/fmt configured
- [x] Local git init + commit

## Phase 1 — Feature definition and requirements
- [x] Define the product feature inventory for a PDF markup/measurement app
- [x] Map features → Rubric domain: markups, layers/toolsets, review workflow (statuses), measurements, stamps, callouts, clouds, ink, annotations, snapshots, etc.
- [x] Update REQUIREMENTS_TRACEABILITY.md with traceability (requirement → file/module/test)
- [x] Record gaps, assumptions, and scope exclusions in assumptions-and-questions.md + DECISIONS.md
- [x] Create docs/spec/README.md summarising the requirement set

## Phase 2 — Vertical slice 1: Core markup + canvas (interactive)
- [x] Domain types complete (MarkupKind 23 variants, Geometry variants, ReviewStatus, Measurement)
- [x] Undo/redo (History snapshots, bounded 128)
- [x] Selection + move/translate
- [x] Draw rect-based shapes (rect/ellipse/cloud/highlight/text/callout/stamp/note) — basic placement
- [x] Draw segment paths (line/arrow/measure length/diameter/radius)
- [x] Draw count marker
- [x] Canvas page rendering + zoom (pan not yet wired to drag except selection)
- [x] Inspector (properties, status, author/subject, comments, measurement display)
- [x] Improve drawing: drag-to-draw for bounds/segment with live preview (polyline/angle still click-based)
- [x] Polyline/Polygon/Perimeter/Area path drawing (multi-point)
- [x] Angle tool (three-point)
- [x] Ink (freehand) with stroke accumulation + commit
- [x] Hit testing improvements (edge/outline vs fill; cloud approximation; ink segments)
- [x] Zoom/pan with mouse wheel + Space/middle-drag

## Phase 3 — Measurements + scale UI
- [x] Scale per page, units_per_point, recompute on geometry change
- [x] Measurement computation (length/perimeter/area/angle/diameter/radius/count)
- [x] Scale editor in inspector/settings (ratio presets: 1:1, 1 in=1 ft, 1 in=10 ft, custom)
- [x] Units dropdown in inspector (active doc/page scale unit)
- [x] Measurement labels rendered on canvas
- [x] Precision/formatting preferences (Settings::precision 0–6, inspector DragValue)

## Phase 4 — Review workflow, layers, toolsets
- [x] ReviewStatus, Comments, Subject/Author
- [x] ToolSet presets (Review/Shapes/Measure)
- [x] Layer visibility/lock (Layer model: visibility/lock, layers panel, mutation gates)
- [x] Status colours + legend
- [x] Stamp library (5 presets: Approved/Rejected/Draft/Check/Cross, glyph rendering, preset picker)
- [x] Callout text editing in-canvas (also Text boxes; auto-open after placement)
- [x] Copy properties / apply visual style to selected markups (F-114; `docs/tasks/v0.2-03-editing-commands.md`)
- [x] Page-scoped z-order: bring forward/backward and to front/back (F-115; same v0.2-03 slice)

## Phase 5 — Persistence + I/O + PDF (non-destructive)
- [x] JSON save/load (.rubric.json)
- [x] Autosave loop (F-502: File > Auto-save enable + interval, dirty-only, one write per interval)
- [x] Project schema versioning (F-507: `schema_version` on save, refuse unsupported version on load)
- [x] Open file dialog (native, rfd) for .rubric.json (PDF filter arrives with import)
- [x] PDF import/rendering (pdfium-render runtime binding; sidecar kept, PDF read-only)
- [x] Export/print (page image PNG via CPU rasterizer + OS print command; report export txt/JSON)
- [x] Recent files (app-level config store: `~/.config/rubric/recent.json`)

## Phase 6 — Polish, packaging, CI
- [x] Basic UI panels, shortcuts (Ctrl+S/Z/Y/Esc/Del)
- [x] Per-tool keyboard shortcuts (F-601: unmodified letter/digit keys, guarded)
- [x] Context menus (F-603: shared canvas + inspector menu)
- [x] Grid, snap, rulers (F-405/F-406: View > Show Grid / Snap to Grid / Grid Spacing; page-point rulers)
- [x] Page thumbnails sidebar (F-404: PDF previews via the shared render cache, numbered placeholders, click to jump)
- [x] Dark theme (F-602: View > Theme light/dark, persisted preference)
- [x] Icon/desktop entry, AppImage/DMG/MSI (F-701–F-704: `.github/workflows/package.yml` + `release.yml`; WiX definition in `packaging/wix/`)
- [x] CI workflow skeleton (.github/workflows/ci.yml)
- [x] Security/community wave 2026-10-24 (guide Part II): security.yml
      (secrets / dependency-review / cargo-deny / SBOM / CodeQL),
      dependabot.yml, deny.toml, branch protection + private vulnerability
      reporting, conduct/security/a11y/vision/governance/contributors docs,
      issue templates, a11y PR checklist, pinned action SHAs
- [ ] Benchmarks/tests expansion
- [x] Changelog updates per slice
- [x] PDFium runtime documented for users (docs/pdfium.md: discovery order, download, verification)
- [ ] Follow-ups recorded as questions: installer code-signing (Q-PACK-1), raster icon set (Q-PACK-2)

## Phase 7 — Rubric v0.2 discovery & design
- [x] Discovery approved: continue Rubric v0.2 slice for reviewers/auditors
- [x] Research captured in docs/research/bluebeam-studio-capabilities.md and docs/research/adobe-acrobat-capabilities.md – capability-level summaries with URLs
- [x] Discovery brief created: docs/discovery/v0.2-discovery.md
- [x] Open questions recorded: Q-V02-1 CSV export, Q-V02-2 diff presentation, Q-V02-3 markup style favorites
- [x] Human approval received 2026-10-24
- [x] Task brief created: docs/tasks/v0.2-01-markup-list-domain.md
- [x] Task brief created: docs/tasks/v0.2-02-markups-list-ui.md
- [x] Task brief created: docs/tasks/v0.2-03-editing-commands.md
- [x] Task brief created: docs/tasks/v0.2-04-persistence-hardening.md
- [x] Inspiration addendum (client-directed): Stirling PDF capability analysis, 2026-10-26
- [x] Task execution complete – `MarkupList`, `StylePreset`/`StylePresetLibrary`, `DiffResult` (commit `7e33fd2`, CI run `40304244261`)
- [x] UI slice: Markups List panel with filter/sort/group, row selection and View toggle (PR #34; implementation commit `c2687d9`)
- [x] Tests for list, filter, diff logic in `src/domain` + `tests/domain_tests.rs`, plus Markups List UI-state helper tests
- [x] Tests for autosave scheduling and the schema-version guard (`src/data/store.rs`, `tests/domain_tests.rs`)

## Phase 8 — v0.3 hardening (audit + engine adoption)
- [x] Full GUI + end-to-end audit of v0.2.0 recorded: docs/audits/v0.2.0-full-audit.md (PR #60)
- [x] Task brief created: docs/tasks/v0.3-01-audit-fixes.md (parallel workstreams A–E + H1)
- [x] Research note approved 2026-10-31: docs/research/hayro-pdf-engine.md (pure-Rust PDF engine, H8 follow-up)
- [x] Audit-fix wave merged with CI green (workstreams A–D + H1 by parallel subagents; WS E tests + M19 context menu by the parent) — PR #62 merged 2026-10-31
- [x] Task brief created: docs/tasks/v0.3-02-hayro-engine.md (H8 follow-up, `hayro`+`lopdf` adoption)
- [x] **Phase 1 (WS A)**: hayro-syntax page-size probing implemented with feature flag, tests, and CI-ready
- [x] **Phase 2 (WS B)**: hayro rendering pipeline – PDF pages rendered to PNG via hayro render function (PR #67)
- [x] **Phase 3 (WS C)**: corpus fidelity tests – validate >95% pixel match vs PDFium baseline (PR #74)
- [x] **Phase 4 (WS D)**: docs/process updates (task briefs, roadmap, traceability, attribution, progress)
- [ ] Engine slice: adopt `hayro` (+ `lopdf`) for full PDF rendering/export

### v0.3-04 — PDF export/print fixes (M4/M5)
- [x] M4: export/print report a failed PDF page background instead of a blank page (+ test)
- [x] M5: fixed-DPI, page-cropped export/print — owner decision: crop to page
  bounds at 150 DPI (`EXPORT_DPI`); the export pass draws the page without
  backdrop, page border, grid or rulers (`draw_canvas(..., is_export = true)`),
  and `export_pdf_to` crops markups via `markup_intersects_page` (PR #88,
  commit `8bc2601`)
- Task brief: `docs/tasks/v0.3-04-pdf-export-fix.md`

### v0.3-05 — lopdf write support
- [x] Add `lopdf` (pinned) for PDF write/export
- [x] File ▸ Export PDF… (markups over the original page) — export_pdf_to()
  renders BBox/Segment/Path/Ink/Text/Callout/Angle/Count overlay; File ▸
  Export PDF… menu with native rfd dialog; honest status messages
- Task brief: `docs/tasks/v0.3-05-lopdf-write-support.md`

### v0.4-01 — multi-select (gap #2)
- [x] Shift+click toggles markups in/out of selection (canvas click handler)
- [x] Marquee drag on empty canvas selects all intersecting markups
- [x] Select All already available via Edit menu
- Task brief: `docs/tasks/v0.4-01-multi-select.md`

### v0.4-02 — batch status (gap #3)
- [x] `EditorAction::BatchStatus(ReviewStatus)` added (`ALL` now 14 entries)
  and `Document::apply_status_to(ids, status)` applies one status to many
  markups
- [x] `RubricApp::apply_batch_status` routes the action through the existing
  undo-snapshot and dirty-marking path (`ui/app.rs`)
- [x] Multi-select status dropdowns in the Inspector and the Markups List
  (`draw_markups_list` now takes `&mut Project`)
- Evidence: PR #88, commit `8bc2601`; CI green on PR #88
- Task brief: not created for this slice; scope recorded in `PROGRESS.md`
  (2026-11-02)

### v0.4-03 — duplicate markups (gap #6)
- [x] `Document::duplicate_markups(ids, offset)` clones editable markups with
  fresh ids, translated geometry, recomputed measurements, appended on top;
  locked/missing ids skipped
- [x] `EditorAction::Duplicate` ("Duplicate", `Ctrl+D`) in the Edit and context
  menus (`ALL` now 15 entries), gated on a non-empty selection
- [x] `RubricApp::duplicate_selected` selects the clones and pushes one
  `"duplicate markups"` snapshot
- [x] Domain test `duplicate_markups_clones_editable_sources_with_fresh_ids_and_offset`
  and action tests
- Evidence: PR #93, commit `8a70739`; CI green on PR #93

### v0.4-04 … v0.4-08 — remaining ranked capability gaps
- [x] **v0.4-04 — flatten-to-PDF (gap #1)**: `data/export.rs`
      `export_page_to_pdf(source, page_index, out, markups)` renders one page
      with its markups overlaid (lopdf `delete_pages` + `prune_objects` +
      `renumber_objects` to keep only the requested page);
      `export_all_pages_to_pdf` delegates to the existing `export_pdf_to`.
      File menu items "Export Page to PDF…" / "Export All Pages to PDF…" with
      native `rfd` dialogs, gated on having a source PDF. 8 unit tests.
- [x] **v0.4-05 — drag calibration (gap #4)**: `Tool::Calibrate` (shortcut `k`,
      not a markup; `Tool::ALL` now 25 variants); `calibrate_scale()` in
      `domain/units.rs` delegates to `Scale::from_ratio` (7 tests); two-click
      canvas interaction with a live length badge; `CalibrationDialog` window
      pre-filled with the measured length; one snapshot on Apply.
- [x] **v0.4-06 — resize grips (gap #5, first slice)**: `Corner` enum +
      `Markup::resize_bounds()` in `domain/markup.rs` (clamps to 1×1 pt; skips
      non-Bounds geometry; 4 tests); canvas corner-handle hit-testing
      (`handle_at`) with drag-to-resize; move and resize mutually exclusive; one
      snapshot on drag end. Segment endpoints/vertices remain follow-ups per the
      gap list.
- [x] **v0.4-07 — tool chest (gap #7)**: `StylePreset::from_style()` +
      `StylePresetLibrary::save_named()` (case-insensitive dedupe) in
      `domain/style_preset.rs` (3 new tests); new `ui/tool_chest.rs` "My Tools"
      left panel (preset CRUD, apply-to-selection, apply-at-creation hook in
      `finish_markup` via `UiState::pending_style_preset`, stamps excluded);
      View ▸ Show Tool Chest toggle.
- [x] **v0.4-08 — editable Markups List console (gap #8)**: inline per-row
      status ComboBox (dimmed when locked) applying `apply_status_to` with one
      snapshot per change; measurement totals footer over the **filtered** view
      (per-kind subtotals, area squared, count markers); "Export CSV…" with an
      `rfd` save dialog (Page, Kind, Author, Subject, Status, Measurement,
      Comments columns). `draw_markups_list` returns
      `(Option<MarkupListAction>, bool)` for snapshot signalling.
- [x] **v0.4-06 follow-up — segment endpoint and vertex editing (gap #5,
      complete)**: `Markup::draggable_points()` in `domain/markup.rs` returns
      stable-index structural points (segment endpoints, every path vertex, all
      three angle points; empty for bounds/ink/text/callout/count) and
      `Markup::move_point(index, pt)` moves one point, returning `false` when
      the index is out of range (lock enforcement and measurement recomputation
      stay with the caller). Canvas point handles replace the bounding-box
      corner handles for those kinds in `draw_selection`; `point_handle_at`
      hit-testing runs before the bounds-corner check; the `resize_point` drag
      reshapes every selected markup at the same index (mirroring corner-drag
      semantics), recomputes measurements for measure kinds and pushes one
      snapshot on drag end; move, corner and point drags are mutually exclusive
      (`UiState::resize_point`). Tests cover `draggable_points` per geometry
      kind, `move_point` for segment/path/angle (including an angle-measurement
      recompute) and `point_handle_at` hit-testing.
- Evidence: PR #96, commits `39f7cc9`…`74ede47`, and PR #98, commit `cac048b`;
  CI green on PR #96 and PR #98. With these slices all eight ranked gaps in
  `docs/research/v0.4-capability-gap-analysis.md` are closed (gap #5 fully:
  Bounds resize plus segment-endpoint and path-vertex editing).

### Audit follow-ups (M18, M24, WCAG, M5, Quit)
- [x] M18: canvas background raster width tracks zoom (PR #90)
- [x] M24: inspector thread-locals moved into `UiState` (PR #90)
- [x] WCAG: Markups List status text uses a text-safe colour on light (PR #90)
- [x] M5: menu shortcut hints + Ctrl+A Select All + hint bar (PR #91)
- [x] Unsaved-changes dialog: Save and Quit (PR #92)

### v1.0-0 — installer verification
- [ ] Multi-platform installer verification (Linux ✅, macOS, Windows)
- [ ] §2.5 legal checklist clean for a public release
- [ ] README/ROADMAP claims re-audited
- Task brief: `docs/tasks/v1.0-0-installer-verification.md`

## Slice strategy
Build vertically: domain + tests → minimal UI → drawing + selection → save/load → measurements → review → polish. Keep each slice small, compiling, tested.

---

### v0.5 — Design polish & systemisation (design & GUI audit, docs/audits/design-gui-2026-11.md, PR #101)

P0 — High impact (audit backlog items 1–5):
- [x] **P0-1**: Icon set for tools, layers, Tool Chest, page nav — original egui-painter vector icons (`ui/icon_button.rs`), no external font/image deps
- [x] **P0-2**: Bundle UI font + tabular monospace for measurement readouts —
  **adapted**: egui/eframe bundle the UI face (Ubuntu-Light, Ubuntu Font
  Licence) and a tabular monospace (Hack, OFL) inside `epaint_default_fonts`,
  already recorded in `ATTRIBUTION.md` and licence-cleared in `deny.toml`
  (`OFL-1.1`/`Ubuntu-font-1.0`); `TEXT_MONOSPACE` routes numeric readouts to it.
  A bespoke brand typeface stays optional (see `assets/branding.md`) and is
  deliberately not vendored here.
- [x] **P0-3**: Persist workspace layout — panel visibility, zoom, page in `AppPrefs`
- [x] **P0-4**: Active-tool chip (always visible); keyboard shortcuts dialog shipped (PR #100)
- [x] **P0-5**: Empty states for primary panels (no document, no markups, empty filtered list)

P1 — Quality (audit backlog items 6–11):
- [x] **P1-6**: Elevation + spacing + radius tokens (`ELEVATION_1/2/3` + corner_radius)
- [x] **P1-7**: Segmented Inspector tabs — tab bar + section gating (Properties / Comments / Measure / Settings content is gated; the read-only Status legend, Layers and Scale stay visible by design)
- [x] **P1-8**: Virtualised Markups List (egui `ScrollArea` viewport culling; async thumbnails pending)
- [x] **P1-9**: Single tool metadata source drives all UIs (`ToolMeta`/`TOOL_METADATA`, PR #117)
- [x] **P1-10**: Persist Tool Chest library in prefs (PR #114)
- [ ] **P1-11**: Host/Ui seam + egui_kittest visual/a11y evidence G4 (guide II.27.12) — documented deferral

P2 — Later (audit backlog items 12–16):
- [ ] **P2-12**: Text-anchored highlight/strikeout (part 1 foundation: `MarkupGeometry::TextAnchor` on `Bounds`-free highlight/strikeout/underline/squiggly; full PDF text extraction + anchoring still pending)
- [x] **P2-13**: Focus / reading mode (PR #118)
- [x] **P2-14**: Dual-unit rulers (PR #115)
- [x] **P2-15**: UI scale + high-contrast preference (UI scale PR #116; high-contrast mode added — View ▸ High Contrast, pure black/white palettes with strong borders, persisted in `AppPrefs.high_contrast`)
- [x] **P2-16**: Optional minimap (PR #119)

Guide §12.1 suite standards (audit approved 2026-11-04):
- [x] Token ledger `docs/tokens.json` + release-workflow update + release-notes reference
- [x] In-app subtle version + release-notes popup (history, usage, token metrics) — embedded at build time
- [ ] Signed releases + in-app checksum-verified auto-update install

Audit follow-ups to track separately:
- [ ] **A1**: Automated AccessKit/snapshots for visual evidence (G4, guide II.27.12) — documented deferral
- [x] **A2**: Focus ring audit — keyboard focus is visible (egui maps
  `has_focus()` to the `active` widget visuals; labels/hyperlinks add a 1 px
  underline); egui 0.36 exposes no separately styleable focus ring
  (`focus_stroke` does not exist) and `debug.show_focused_widget` is a debug
  aid, so the limitation is recorded in `docs/a11y.md` and re-checked on
  toolkit upgrades. High-contrast mode strengthens the focused outline.
- [x] **A4**: Text labels when narrow panels use icons only — audited in
  `docs/a11y.md`: panels carry minimum widths that keep labels visible
  (Tools 180 / Tool Chest 170 / Inspector 240 / Pages 120 px), and the genuinely
  icon-only controls (layer eye/padlock, reorder ▲/▼, page arrows) all carry
  descriptive tooltips; the toolbar pairs every icon with a text label.
- [x] **A3**: High-contrast preference — implemented as the P2-15 high-contrast mode

---

### v0.6 — Bates numbering (future slice; queued after the remaining v0.5 slices)

Sequential page stamping for legal / document-review workflows, implemented as a
**non-destructive overlay** (source PDF untouched). Research note:
`docs/research/bates-numbering.md` (capability-level summary + URLs, guide §2.3
human gate still applies before implementation). Spec: `docs/spec/README.md`
F-8xx. Capability inventory row 32.

- [x] **F-801**: `BatesConfig` domain value type — prefix, suffix, start number,
  digit padding (clamped), `format_number`/`value_for_page`/`stamp_for_page`,
  serde round-trip; unit tests cover padding, clamp, sequence and range.
  (`apps/rubric/src/domain/bates.rs`.)
- [x] **F-802**: Placement + style config — page corner (`BatesCorner`), margin
  and font size, with sensible defaults; edited in the Inspector.
- [x] **F-803**: Page range — `BatesRange` (inclusive from/to, open-ended `to`),
  with an honest "N of M pages to stamp" count and first/last stamp preview.
- [x] **F-804**: Canvas overlay preview — a ghosted stamp at its placement that
  follows pan/zoom and shows only when Bates is enabled; preview only (creates
  no markup, no undo snapshot). `draw_bates_overlay` in `ui/canvas.rs`;
  `BatesConfig::anchor_point` is unit-tested.
- [x] **F-805**: Stamped export + print — the sequence is baked into **all three
    output paths**: the flattened **PDF** (`overlay_page_with_markups` + a
    Helvetica width estimate for right-corner alignment), the exported **page
    image**, and **print** (both reuse `draw_canvas(is_export = true)`). A
    headless test asserts each exported page carries its stamp and the font
    resource. The source PDF is never modified.

Ordering: after the remaining v0.5 audit items (notably the **P2-12**
text-anchored markup completion, which needs a PDF text-extraction path, and
the **P0-2** bundled-font remainder) so the stamp UI can reuse the finished
inspector/toolbar surfaces.

## Slice strategy
Build vertically: domain + tests → minimal UI → drawing + selection → save/load → measurements → review → polish. Keep each slice small, compiling, tested.
