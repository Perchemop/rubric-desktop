# PROGRESS.md

## 2026-10-17 — Phase 0 (initial commit)
**Status**: in progress (verifying build)

### Completed
- Confirmed app spec: a cross-platform PDF markup/measurement desktop app.
- Environment: Rust 1.99.0 stable (aarch64), GUI dev libs present, display available, crates.io reachable, ~676 cached crates.
- Invented name: **Rubric** (original, thematically apt). Designed logo concept.
- Installed `librsvg2-bin` (rsvg-convert). Generated SVG logo + PNG icons (16/32/48/64/128/256/512/1024).
- Created fresh repo structure at ~/rubric.
- .gitignore excludes external material, secrets and build output.
- Branding: `assets/logo.svg`, `assets/icons/*`, `assets/branding.md`.
- Docs: `README.md`, `AGENTS.md`, `docs/architecture.md`, `docs/data-model.md`, `docs/test-plan.md`, `docs/assumptions-and-questions.md`.
- Process docs: `IMPLEMENTATION_PLAN.md`, `REQUIREMENTS_TRACEABILITY.md`, `DECISIONS.md`, `PROGRESS.md`, `CHANGELOG.md`.
- Cargo.toml (eframe/egui 0.36, serde, uuid, thiserror, tracing, chrono, image), rust-toolchain.toml, LICENSE (MIT).
- Domain model complete: `geom`, `units`, `style`, `markup` (23 kinds + geometry + measurements + hit tests), `document`, `project` (history/undo/redo, toolsets). All with unit tests.
- Data layer: `data/error.rs`, `data/store.rs` (JSON save/load, app data dir).
- UI skeleton: `ui/theme.rs`, `ui/icons.rs`, `ui/app.rs` (panels, menu, shortcuts), `ui/canvas.rs` (page rendering, selection, basic draw/place), `ui/toolbar.rs` (tool groups), `ui/inspector.rs` (properties/comments/measurements).
- Integration tests: `tests/domain_tests.rs`.
- Tooling: `scripts/gen-icons.sh`, `.github/workflows/ci.yml`.
- `src/lib.rs`, `src/main.rs` with eframe run_native and embedded icon.

### In progress
- Verifying: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`.

### Blockers
- None.

### Next
1. `cargo test` — all unit/integration tests pass
2. `cargo clippy -- -D warnings`
3. `cargo fmt --all -- --check`
4. `git init && git add . && git commit`
5. Phase 1: define the feature requirement set and update traceability
6. Phase 2: improve drawing (drag-to-draw, polyline/polygon/angle/ink preview), zoom/pan UX

## 2026-10-18 — Phase 0 verified via GitHub CI + subagent suite added
**Status**: Phase 0 committed and pushed; CI enforces quality gates

### Completed
- Phase 0 committed (`f984665`) and pushed to GitHub; `main` branch.
- CI workflow (`.github/workflows/ci.yml`) runs `cargo fmt --check`,
  `cargo clippy -D warnings`, `cargo test --all`, and a release build on every
  push — the machine-freezing local test problem is worked around by letting
  GitHub Actions run the test gate.
- Formatting applied and committed (`d157759`, `0dd6844`); area measurement
  rescaling fix landed (area scales with `units_per_point²`).
- Subagent suite added under `.opencode/agents/` (symlinked into
  `~/.config/opencode/agents/` so the home-directory session can launch them):
  - `domain-dev` (deepseek-v4.1-flash-free#high) — domain types + unit tests
  - `ui-dev` (beam-free#high) — egui/eframe UI slices
  - `test-writer` (mimo-v2.6-flash-free) — test coverage only
  - `docs-writer` (big-pickle) — process docs upkeep
  - `fixer` (space-bunny-free#high) — build-gate failure fixes
  - `reviewer` (glm-5.3-flash-free#high) — read-only review (pre-existing)
  All run as `subagent` mode with allowlisted shell access (fmt/check/clippy
  and read-only git); `cargo test` and git push are reserved for CI and the
  parent agent respectively. Smoke-tested `domain-dev` successfully.

### Active
- Waiting on GitHub CI run results for the pushed commits.

### Blocked
- None.

## 2026-10-19 — CI green + CI-first policy
**Status**: Phase 0 fully verified by GitHub Actions

### Completed
- GitHub billing resolved; re-run of CI run `39121051225` passed all 8 steps:
  fmt check, clippy (`-D warnings`), full test suite, and release build —
  **first fully green run**; Phase 0 quality gates are verified.
- Local machine tidied: temporary tooling downloads and resident node
  processes removed (~1.9 GB freed). The machine holds no external reference
  material — original work only, per the build guide.
- CI-first policy adopted (user directive): all CPU/RAM-intensive steps
  (`cargo test`, `clippy`, `check`, release builds) run only in GitHub Actions;
  the desktop host runs at most `cargo fmt` and read-only git. Documented in
  `AGENTS.md` and enforced in every subagent's permissions.
- `Swatinem/rust-cache@v2` added to the workflow so CI compile steps are
  cached across runs.
- Local zombie processes from interrupted runs (stale cargo/rustc/clippy) were
  identified as the cause of the memory pressure behind desktop restarts and
  cleared.

### Active
- Phase 1: define the feature requirement set and update traceability.

### Blocked
- None.

## 2026-10-19 — Phase 1 complete (feature definition and requirements)
**Status**: Phase 1 done via the docs-writer agent; spec adopted as canonical inventory

### Completed
- Created `docs/spec/README.md` — the requirement set: one-paragraph overview,
  conventions, non-goals, and 52 feature IDs in seven groups (F-1xx markups/tools 16,
  F-2xx measurements/scale 9, F-3xx review workflow 5, F-4xx navigation/pages 6,
  F-5xx persistence/I/O 7, F-6xx UI/UX polish 5, F-7xx packaging 4), each row with a
  priority (MVP / Ph 2-6 / later) mapped to IMPLEMENTATION_PLAN.md phases.
- `REQUIREMENTS_TRACEABILITY.md`: new "Phase 1 requirement set" section with
  R-S-001…R-S-017 (spec area → F-IDs → planned modules, TODO/PARTIAL), seven new
  F-1xx…F-7xx group rows in the Feature requirements table, and the stale
  "(22 kinds)" count corrected to 23 (MarkupKind has 23 variants).
- `docs/assumptions-and-questions.md`: "Scope decisions (Phase 1)" (pure-Rust PDF
  renderer first, sidecar-only MVP persistence, image/print export instead of
  PDF-embedded annotations, target status set Open/In review/Approved/Rejected)
  plus questions 17-22 (PDF crate, stamp library, report format, export scope,
  layer model, page navigation).
- `DECISIONS.md`: 2026-10-19 entry adopting the spec as canonical inventory;
  stale 22→23 markup-kind count fixed there too.
- `IMPLEMENTATION_PLAN.md`: Phase 1 checklist ticked; stale 22→23 count fixed.
- `CHANGELOG.md`: Unreleased "Added" entry for the Phase 1 requirement set;
  stale counts fixed (incl. the 0.1.0 note) and noted under "Fixed".
- `docs/data-model.md`: markup-kind count corrected to 23.

### Active
- Phase 2: drag-to-draw, polyline/polygon/angle/ink, zoom/pan UX.

### Blocked
- None. (The `docs/spec/` gitignore rule that blocked the spec commit was
  narrowed by the parent; the original requirement set is now tracked.)

## 2026-10-19 — Phase 2 slice A: interactive canvas (drag-to-draw, zoom, pan)
**Status**: implemented + reviewed + fixed; awaiting CI verification

### Completed
- Domain (`domain-dev`, src/domain/markup.rs): hit-testing upgrades — filled
  shapes hit interior, stroke-only hit near outline only; true elliptical edge;
  cloud approximated by rect border (documented); even-odd interior test for
  filled closed paths/polygon; ink segment-based incl. single-point dot hits.
  8 new unit tests; public `hit_test` signature unchanged.
- UI (`ui-dev`, src/ui/{canvas,app}.rs): drag-to-draw with live preview for
  bounds/segment tools; ink freehand accumulation (2 screen-px subsample);
  wheel zoom clamped 10–1600% anchored on cursor; Space/middle-drag pan;
  Del/Backspace delete with one snapshot; Esc cancels previews.
- Review (`reviewer`): found 2 blockers (stray brace, unused var), 2 majors
  (Esc mid-drag re-spawn, selection drag-move snapshot spam), minors.
- Fixes (`fixer`): all blockers + Esc-cancel flag (`draw_cancelled`),
  single-snapshot-per-drag for selection moves (`move_selection`), dead-code
  tail removed; `cargo fmt --all -- --check` clean.
- Deferred follow-ups (reviewer findings): one-frame stale transform on
  mid-drag wheel zoom; wheel-direction/platform convention check on desktop;
  single-point ink tap rendering; unfilled-rect vs Bounds tolerance unification.

### Active
- Pushing slice A; CI is the compile/clippy/test gate.

### Blocked
- None.

## 2026-10-19 — Phase 2 slice B: multi-point drawing (polyline/polygon/angle)
**Status**: implemented + reviewed + fixed; awaiting CI verification

### Completed
- Slice A CI-verified green (`ae87bab`): fmt, clippy, tests, release build —
  2 follow-up fixes from CI (egui 0.36 `smooth_scroll_delta`, unused import).
- UI (`ui-dev`): multi-point Path drawing (Polygon/Polyline/Perimeter/Area)
  — click-to-append, rubber-band + auto-close preview, commit on
  double-click/right-click/Enter, Esc cancels; three-click Angle tool with
  live ray preview; click-fallback no longer commits degenerate geometry.
- Review (`reviewer`, second pass): found 1 CRITICAL (`&mut Pt` vs owned `Pt`
  in angle-commit arm — would not compile), 2 MEDIUM (stale draft on tool
  switch; segment tools committing wrong-typed Bounds on click), 3 LOW
  (double-click duplicate vertex, point-distinctness validation, fake-keyboard
  Enter/click conflicts).
- Fixes (`fixer`): all six applied; `last_tool` draft reset on tool switch,
  `path_points_ok` helper at all three commit sites, `clicked_by(Primary)` +
  `wants_keyboard_input` guards; fmt clean.

### Active
- Slice B CI-verified green (`8ea3e5c`); Phase 2 complete.

### Blocked
- None.

## 2026-10-19 — Phase 3: measurement labels + scale editor
**Status**: CI-verified green (`e76a816`) — fmt, clippy, tests (incl. 3 new units tests), release build. 1 CI follow-up fix (`Painter::layout_no_wrap` for egui 0.36).

### Completed
- UI (`ui-dev`): measurement labels on canvas — screen-space 12 pt text with
  semi-transparent rounded backing, formatted via domain
  `Measurement::label(page.scale.unit)`, anchored to geometry (segment
  midpoint, path centroid, angle vertex, count point), gated on
  `settings.show_measurements`.
- Inspector Scale section: preset buttons (1:1, 1 in = 1 ft, 1 in = 10 ft,
  1 in = 50 ft), display-unit combo, custom ratio (paper inches : real units)
  with editable DragValue + Apply. Scale/unit changes recompute the active
  page's measurements and push one undo snapshot per committed edit (not per
  keystroke).
- Review (`reviewer`): 1 BLOCKER (E0499 overlapping borrows in the inspector
  call), 1 MAJOR (display-unit switch silently rescaled physical
  measurements), 1 MEDIUM (paper-inches display-only), 2 LOW (no-op preset
  snapshots, dead per-frame describe block), 2 INFO (comment add had no
  snapshot; missing tests).
- Fixes (`fixer`): removed the redundant `status: &mut String` param (writes
  `ui_state.status`); new domain helper `Scale::with_display_unit` preserving
  physical length across unit switches (`upp_new = upp_old · ppu(old) /
  ppu(new)`) with 3 unit tests (physical-length preservation, identity,
  round-trip); real DragValue for paper inches; `same_scale` dedupe so no-op
  clicks never snapshot; `edited` gate stops per-frame `touch()` churn;
  comment add now snapshots once. fmt clean.

### Active
- Label/scale slice CI-verified green (`e76a816`); precision slice pushed
  next (see entry below).

### Blocked
- None.

## 2026-10-19 — Phase 3 completion: precision preference
**Status**: CI-verified green (`b778f1c`) — fmt, clippy, all 11 new tests, release build. Phase 3 fully complete.

### Completed
- Domain (`domain-dev`): `Settings::precision` (u8, serde default 3 — legacy
  `.rubric.json` loads unchanged), `set_precision` clamped 0..=6,
  `format_value_precise` + `DEFAULT_PRECISION`/`MAX_PRECISION` consts,
  `Measurement::label_with(unit, precision)`; `format_value`/`label` kept as
  wrappers with identical behaviour. Angle fixed at 2 decimals, count at 0.
  11 new tests (formatting, clamping, legacy JSON load, roundtrip).
- UI (`ui-dev`): inspector Precision DragValue (0..=6) — `changed()` applies
  live, snapshot only on `drag_stopped() || lost_focus()` with a frame-start
  before-value diff (one snapshot per edit, none per keystroke); canvas +
  inspector measurement displays use `label_with(…, settings.precision)`.
- Review (`reviewer`): no material defects; recommended push. Findings: CHANGELOG
  entry missing (added), info-level note that hand-edited out-of-range
  `precision` in JSON loads unclamped but is clamped at every formatter/writer
  (accepted — `format_value_precise` clamps defensively).

### Active
- Pushing precision slice; Phase 3 then fully complete.

### Blocked
- None.

## 2026-10-19 — Phase 4 slice 1: layer visibility/lock
**Status**: CI-verified green (`dd4bbe8`) — fmt, clippy, all tests incl. 15 new layer tests, release build. 1 CI follow-up fix (test-module import path).

### Completed
- Domain (`domain-dev`): `Layer { id, name, visible, locked, z }` +
  `LayerError` (NotFound/DefaultLayer/NotEmpty); `Markup.layer_id: Option<Uuid>`
  with None = per-document default layer (legacy files migrate invisibly;
  empty layer lists repaired on load); document ops incl. `add_layer`,
  `remove_layer` (refuses default/populated), visibility/lock setters,
  `is_markup_visible`/`is_markup_editable` (locked blocks editing; hidden
  blocks paint/hit only; unknown ids fail open). Clarified `ToolSet` doc (it
  is a palette preset, not a layer). 13 unit + 2 integration tests.
- UI (`ui-dev`): inspector Layers section (eye/lock toggles, add, delete with
  `LayerError` → status), layer ComboBox for selected markup; canvas paint,
  labels, and pick filter on visibility; move-drag, Del, and inspector
  write-back gated on editability.
- Review (`reviewer`): HIGH unused `Layer` import (certain CI fail); MEDIUM
  comment add on locked markup destroyed typed text; MEDIUM drawing on a
  locked default layer created uneditable markups; LOW zero-delta drag
  snapshot, stale-id delete snapshot, handles on hidden selection.
- Fixes (`fixer`): all — import removed; `add_enabled_ui(editable)` for the
  comment UI; new `creation_allowed` helper refuses draft starts with a
  status message when the default layer is locked (incl. the click-placement
  branch); `UiState::move_changed` flags real translations only (frame-local
  flag would fail — egui clears DRAGGED on release); `delete_selected` skips
  stale ids; selection handles skipped for invisible selections. fmt clean.
- Follow-ups recorded: layer rename/reorder APIs missing in domain;
  subject/author/status inspector edits don't snapshot (pre-existing
  asymmetry); active-layer selection for new markups.

### Active
- Pushing layer slice; CI is the compile/clippy/test gate (15 new tests).

### Blocked
- None.

## 2026-10-19 — Phase 4 slice 2: status remap + legend
**Status**: CI-verified green (`e2bff4d`) — fmt, clippy, all tests incl. 6 new status tests, release build. Reviewed clean; no fixer round needed.

### Completed
- Domain (`domain-dev`, src/domain/markup.rs): `ReviewStatus` remapped to the
  spec F-301 set — `Open` (default) / `In review` / `Approved` / `Rejected`
  (serde snake_case); old variant set (None/Accepted/Completed/Cancelled/Alert)
  dropped under a recorded scope decision (no released files, rename is free).
  Distinct palette colour mapping via `ReviewStatus::color()` (grey / amber /
  green / red). One canvas break site fixed. 6 new domain tests incl.
  legacy-string rejection.
- UI (`ui-dev`): inspector "Status legend" section (swatch + label + live
  count per status), colour swatches in the status ComboBox, canvas status
  dots for non-Open markups (dot guard falls back to `Open`).
- Review (`reviewer`): no material defects; push recommended. Findings: docs
  drift (README, data-model, assumptions still listed the old status set —
  fixed in this entry) + pre-existing status-snapshot follow-up (subject/
  author/status inspector edits don't push an undo snapshot).

### Active
- Pushing slice 2; CI is the compile/clippy/test gate (6 new tests).

### Blocked
- None.

## 2026-10-19 — Phase 4 slice 3: stamp library
**Status**: implemented + reviewed (push recommended) + one-line fix folded in; awaiting CI verification

### Completed
- Domain (`domain-dev`): `StampPreset { Approved, Rejected, Draft, Check,
  Cross }` — `ALL`, word `label()`, drawn `glyph()` (words for status marks,
  ✓/✗ for check/cross), meaning-grouped `color()` (green affirmative / red
  negative / amber draft); `Markup.stamp: Option<StampPreset>` serde-defaulted
  (legacy stamps load None → plain rect), `with_stamp` builder; 7 tests
  (serde snake_case, legacy load, roundtrip, labels/glyphs/colours).
- UI (`ui-dev`): `UiState.active_stamp` (default Approved); all 10
  `Markup::new` creation sites route through `finish_markup` (preset +
  stroke colour applied ONLY for Stamp kind — reviewer verified no data
  pollution on other kinds); canvas glyph rendering centred with
  shrink-to-fit, <12 px threshold, legacy None unchanged; inspector preset
  list for selected stamps (editable-gated, one snapshot per real change)
  and tool-options ComboBox for placement (UiState only, no snapshot).
- Review (`reviewer`): push recommended; data-pollution check passed.
  Findings: no-op preset re-click snapshot (fixed inline — single-condition
  guard avoiding clippy collapsible_if), glyph re-layout perf nit + preset
  colour vs custom-stroke disagreement (accepted: preset supplies colour),
  and a manual check needed in-app that ✓/✗ render (egui default font
  coverage not verifiable headlessly).
- Follow-ups recorded: glyph re-layout nit; toolbar changes skipped.

### Active
- Pushing stamp slice; CI is the compile/clippy/test gate (7 new tests).

### Blocked
- None.

## 2026-10-20 — Phase 4 slice 4: in-canvas text editing (F-104/F-105)
**Status**: CI-verified green (`117bddb`) — fmt, clippy, all tests incl. new Text/Callout geometry test, release build. Phase 4 complete. 1 CI follow-up (unused `mut`).

### Completed
- UI (`ui-dev`, two rounds): double-click entry on Text/Callout in Select
  mode (visibility/editability gated) → multiline `TextEdit` overlay anchored
  to the markup's screen rect, focus requested once, commit on focus loss
  (one snapshot only if text changed), Esc cancels; Enter stays a newline.
  All interaction arms (click/draw/move/delete) gated on `text_edit.is_none()`;
  move-drag silently cancelled when an edit starts mid-gesture.
- Placement bug fixed: Text/Callout creation built bare `Bounds` geometry —
  now builds `Text { at, wrap, text }` / `Callout { rect, tail, text }`
  (callout drag anchors the tail at the press point), and placement auto-opens
  the editor for both kinds.
- Review (`reviewer`): findings — (1) MEDIUM stale `text_edit` after
  undo/menu-delete mid-edit could dead-lock the canvas; (2) MEDIUM empty Text
  markup invisible but hit-testable; (3) LOW wrap ignored in final render;
  (4) LOW no roundtrip/hit-test tests for Text/Callout geometry; (5) NIT
  status message clobbered same-frame.
- Fixes (`fixer`, glm-5.3-flash-free): all five — undo/redo/menu-delete
  gated during edit with status hint (plus proactive `collapsible_if` merge in
  the adjacent Del block), stale-state backstop (`else` branch clears
  `text_edit` when the markup is gone), empty-Text low-alpha outline,
  `Painter::layout` wrap-width rendering (signature verified against vendored
  egui 0.36.2), new `text_and_callout_geometry_roundtrip_and_hit_test` domain
  test, and `push_snapshot_quiet` so "Text updated" survives the frame.
  fmt clean. Agent def change: `fixer` repointed `space-bunny-free` →
  `glm-5.3-flash-free#high` (model retired).

### Active
- Pushing slice 4; CI is the compile/clippy/test gate.

### Blocked
- None.

## 2026-10-20 — Phase 5 slice 1: native file dialogs + recent files
**Status**: CI-verified green (`dba80a6`) — fmt, clippy, all tests incl. `RecentList`/`normalise_project_filename`, release build. 1 CI follow-up (test formatted `ThreadId` with `{}` instead of `{:?}`).

### Completed
- Parent: added `rfd = "0.15"` (MIT/Apache-2.0, pure-Rust xdg-portal backend
  on Linux — no GTK).
- Domain (`domain-dev` → reworked by `fixer`): recents started as
  `Settings.recent_paths`, then — reviewer finding — were moved out of
  project settings entirely to avoid writing local paths into shareable
  files. Final shape: `RecentList` domain type (serde-transparent string
  array, dedupe/move-to-front/cap at `MAX_RECENT = 10`, remove/clear) with
  6 tests + a legacy-field load test.
- Data (`fixer`): new `src/data/recent.rs` — `recent_config_path()` (OS
  config dir via `directories`, mirroring `project_dir`), `load_recent()`
  (soft-fails to empty), `save_recent()` (pretty JSON); `normalise_project_filename`
  helper in store.rs (case-insensitive `.rubric.json` suffix, `.json`
  replaced not stacked) with 3 tests.
- UI (`ui-dev` → reworked by `fixer`): File menu rework — Open Project…
  (Ctrl+O) via rfd, Save Project (Ctrl+S routes through current path, no
  more false-success override), Save Project As…, Recent submenu (full path
  on hover, self-pruning on missing files, Clear Recent), Quit via
  `ViewportCommand::Close`; `current_path` session state + window title
  "<file> — Rubric"; zero undo snapshots for any recents maintenance.
- Review (`reviewer`): all rfd/egui APIs verified against published sources
  (rfd 0.15.3, egui 0.36.2 — including ViewportCommand::Title and the
  borrow-sensitive Recent submenu). Findings: false-success save status
  (fixed), recents-in-settings design (reworked per parent decision),
  process::exit Quit (fixed), extension edges (fixed), xdg-portal runtime
  caveat (documented in DECISIONS.md + assumptions).
- Parent decisions recorded in DECISIONS.md: recents are app-level config,
  rfd for dialogs.

### Active
- Slice 1 complete. Next: Phase 5 slice 2 — PDF import/rendering (crate
  choice: lopdf vs pdfium-render, assumptions Q2/Q6).

### Blocked
- None.

## 2026-10-21 — Phase 5 slice 4: page image export (F-505a)
**Status**: complete, CI-green (`39577186482`, 1m41s, all steps)

### Completed
- Data: `src/data/render.rs` — `rasterize_primitives` (2×2 supersampled
  triangle rasterizer with texture sampling + alpha blend + scissor clip) and
  `render_page_to_png_bytes` (headless egui frame of `draw_canvas` over the
  cached page image, PNG encode); 4 pure-math unit tests.
- UI: File → Export Page Image… (rfd save dialog, suggested `rubric-page.png`)
  + `export_page_image` method; honest status on success/failure/cancel;
  session-state only, no snapshot/.rubric.json change.
- Print intentionally deferred (R-S-015 stays PARTIAL — image export only).

### Active
- None.

### Blocked
- None.

## 2026-10-21 — Phase 5 slice 4: page image export + print (F-505)
**Status**: complete, CI-green (`39589815002`, 1m50s, all steps; 131 tests pass)

### Completed
- Data: `src/data/render.rs` — `rasterize_primitives` (2×2 supersampled
  barycentric triangle rasterizer: per-vertex colour interpolation, texture
  sampling incl. the font atlas at `TextureId::default()`, partial texture
  deltas, scissor clip, src-over alpha blend) and `render_page_to_png_bytes`
  (headless egui frame of `draw_canvas` over the cached page image → PNG
  encode); 4 pure-math unit tests.
- Data: `src/data/print.rs` — pure `print_command` (lp / lpr / powershell
  Start-Process-Print) + `print_image` (spawn + wait, honest error); 3 tests.
- UI: File → Export Page Image… and Print… (rfd save dialog for the image,
  direct print for the job); `export_page_image` / `print_page` methods with
  honest status on success/failure/cancel; session-state only.
- Traceability: R-S-015 → DONE. F-505 complete (image export + print).

### Notes / risks
- The renderer is CPU-only and re-uses the canvas drawing code, so exports
  match the on-screen view (background + overlays + text labels).
- Page-number semantics still use `Page.index` as the PDF page number; a
  dedicated `Page::source_page` field is a planned follow-up.

### Active
- None.

### Blocked
- None.

## 2026-10-21 — Phase 5 slice 3: summary report + export (F-304)
**Status**: complete, CI-green (`39576158411`, 1m41s, all steps)

### Completed
- Domain: `src/domain/report.rs` (Report/ReportSection/ReportEntry/StatusCounts;
  grouping by Status/Author/Subject with empty→"(none)" last, deterministic
  to_text, measurement detail via page scale + precision, 15 unit tests).
- Data: `src/data/export.rs` writers (text + pretty JSON + trailing newline)
  with temp-file round-trip/error tests; DataError reused (Io/Json variants).
- UI: File → Export Report… window (grouping ComboBox, format radios,
  project/markup summary) driving an rfd save dialog; honest status on
  success/failure/cancel; session-state only, no undo snapshot.
- Traceability: R-S-009 → DONE; R-S-014 → DONE (PDF import); R-S-015 →
  PARTIAL (image+print still pending).

### Active
- None.

### Blocked
- None.

## 2026-10-21 — Phase 5 slice 2: PDF import, page rendering, page navigation
**Status**: complete, CI-green (`39558422915`, 4m34s, all steps)

### Completed
- Domain: `Project.source_pdf` (serde-defaulted), `Project::from_pdf`
  (Letter/A4 per page, empty→A4 fallback); 6 new domain tests.
- Data: `src/data/pdf.rs` — runtime PDFium binding via `pdfium-render` 0.9,
  hash-keyed PNG render cache with invalidation on PDF replacement, library
  discovery (`RUBRIC_PDFIUM_PATH` → exe dir → system); `DataError::{Pdf,
  PdfLibrary}`; 8 pdf tests incl. cache-key/invalidation and xref checks.
- CI: workflow fetches the PDFium tarball to `$HOME/pdfium` and sets
  `RUBRIC_PDFIUM_PATH` for the test step — verified live: all lib-dependent
  tests ran and passed against real PDFium (96 + 10 tests total).
- UI: Import PDF… (rfd), shared `install_project` path, `current_page` with
  top-bar + PageUp/PageDown navigation (keys guarded during in-canvas text
  editing — reviewer finding 3), lazy background page textures (~144dpi)
  with per-cache-key failure poisoning (reviewer finding 4), inspector
  follows the current page.
- Reviewer gate findings 3+4 fixed; two compile-level CI failures
  (format-string field access, `too_many_arguments`) fixed in follow-up
  commits `7546082` and `b65831a`.

### Active
- None.

### Blocked
- None.

## 2026-10-21 — Guide-conformant discovery/tracking docs + inspiration policy
**Status**: complete (docs-only, no code change)

### Completed
- Human directive: product references allowed as **capability-level
  inspiration**. Recorded in DECISIONS.md.
- Created the guide-required doc set: `docs/project-brief.md` (§0 answers,
  Facts/Assumptions/Decisions marked), `docs/scope.md` (in/out + MVP),
  `docs/product-name.md` (Rubric, locked), `docs/inspiration.md`
  (capability-level, three reference products),
  `ATTRIBUTION.md` (12 direct crates + PDFium BSD-3 runtime + assets),
  `docs/development.md` (CI-first workflow, slice pipeline, env vars),
  `docs/qc/checklist.md` (slice/phase/release/attestation checklists + run
  log).
- Reworded README originality section, AGENTS.md purpose and Do-not list,
  and the docs-writer agent rule to the new inspiration policy.

### Active
- Phase 5 PDF-import slice still uncommitted; reviewer findings 3–4
  (PageUp/Down guard while editing, background-retry perf) pending before
  push.

### Blocked
- None.

## 2026-10-22 — Guide v2 re-implementation + originality rewrite
**Status**: complete (docs/config; no application code change)

### Completed
- Adopted the updated build guide: free-model policy is now **OpenCode free
  first, then NVIDIA free** (the earlier alternate provider is gone; no
  local model hosting). Added `docs/current-model-map.md` (required by guide
  §5.2/§6) recording the role→model map for OpenCode free plus the NVIDIA
  fallback tier, with a note that IDs are re-discovered live per session.
- `.opencode/agents/*.md` retain placeholder free-model defaults; the
  orchestrator re-discovers and overrides per session, per guide §5.2.
- Rewrote all originality wording across `README.md`, `AGENTS.md`,
  `ATTRIBUTION.md`, `.gitignore`, `docs/spec/README.md`,
  `docs/project-brief.md`, `docs/inspiration.md`, `docs/development.md`,
  `docs/qc/checklist.md`, `DECISIONS.md`, `CHANGELOG.md`, `PROGRESS.md` and
  `.opencode/agents/docs-writer.md`. Statements are now positive and
  forward-facing: Rubric's code, docs and assets are original work derived
  from public standards, our own spec and capability-level inspiration.
  Removed the earlier defensive/narrative passages and the internal audit
  record.

### Active
- Phase 6 dark theme (F-602) staged; pending CI on the re-pushed history.

### Blocked
- None.

## 2026-10-23 — Phase 6 dark theme (F-602) CI-verified and merged; model map refreshed
**Status**: complete — CI SUCCESS on `f8cf137` (GitHub Actions run `39775564387`, re-run `39776982743`); PR #1 merged 2026-10-22T12:16:23Z as fast-forward, `origin/main` = `f8cf137`.

### Completed
- CI verdict SUCCESS for commit `f8cf137` (branch `opencode/ci`): fmt check, clippy `-D warnings`, `cargo test`, release build all green. Closes out the "pending CI on the re-pushed history" item (guide-v2 model-policy docs, originality wording rewrite, CI-first subagent config, dark-theme test-only compile fix).
- PR #1 (https://github.com/Perchemop/rubric-desktop/pull/1, base `main`) merged as fast-forward; Phase 6 dark theme (spec F-602) fully landed on main: View > Theme menu (Light/Dark), app-level preference persisted via `AppPrefs` to the OS config dir (`prefs.json`), never inside `.rubric.json`, egui 0.36 per-theme styles. Landing commits: `f9ec330` (feature), `89b3bca` (egui 0.36 `radio_value(current, alternative, atoms)` signature fix in the theme menu), `21357cd` (dark-theme test-only compile errors).
- Reviewer verdict (F-602): PUSH AS-IS — verified domain purity, soft-fail prefs loading, theme never written into `.rubric.json`, theme applied at startup, no undo-history pollution; `cargo fmt --all -- --check` clean. Non-blocking follow-ups: (i) `set_theme` builds a fresh `AppPrefs` — switch to load-modify-save once `AppPrefs` grows more fields; (ii) optional `ctx.request_repaint()` in `set_theme` to decouple repaint from menu interaction; (iii) cosmetic: no dedicated theme test in `tests/domain_tests.rs` (in-module coverage suffices).

### Active
- Next: Phase 6 slice — per-tool keyboard shortcuts (F-601); then context menus (F-603), grid/snap/rulers, packaging (F-7xx).

### Blocked
- None.

## 2026-10-23 — Phase 6 per-tool keyboard shortcuts (F-601) CI-verified and merged
**Status**: complete — CI SUCCESS on `71568556` (GitHub Actions run `39895818631`, job `check` 1m51s: fmt, clippy `-D warnings`, tests, release build); PR #4 (https://github.com/Perchemop/rubric-desktop/pull/4, base `main`) merged by rebase 2026-10-23T03:41:54Z, `origin/main` = `2928bc9`.

### Completed
- Domain (`src/domain/project.rs`): `Tool::ALL` (24 variants, enum declaration order), `Tool::shortcut_key()` (letters for markup/shape tools, digits `1`–`7` for measurements, lowercase) and `Tool::from_shortcut_key()` (ASCII case-insensitive reverse lookup); 9 new unit tests (ALL completeness/order, every tool has a key, pairwise-unique keys, round-trip, case-insensitivity, unknown-key rejection, documented mapping table, serde representation unaffected).
- UI (`src/ui/app.rs`): `select_tool()` switches the active tool and abandons any in-flight draft gesture exactly like Esc while keeping the selection; `key_to_shortcut_char()` maps egui `Key` variants; `logic()` fires tool keys only when unmodified — blocked while editing text, while a widget wants the keyboard, or when Ctrl/Alt/Super is held (Shift allowed).
- Docs: F-601 marked done in `docs/spec/README.md`, `REQUIREMENTS_TRACEABILITY.md` and `IMPLEMENTATION_PLAN.md`; CHANGELOG entry added.

### Active
- Next: context menus (F-603), grid/snap/rulers, packaging (F-7xx).

### Blocked
- None.

## 2026-10-23 — Phase 6 editor context menus (F-603) CI-verified and merged
**Status**: complete — CI SUCCESS on `c86120a` (GitHub Actions run `39902215086`, job `check` 1m56s: fmt, clippy `-D warnings`, tests, release build); PR #6 (https://github.com/Perchemop/rubric-desktop/pull/6, base `main`) merged by rebase 2026-10-23T04:50:36Z, `origin/main` = `0280ea5`.

### Completed
- Domain: new `EditorAction` enum (`src/domain/action.rs`) with `ALL`, `label()` and `is_enabled(selection_count)` (delete/clear need a selection; history/view are always offered); 4 unit tests.
- UI: shared `src/ui/menu.rs` builder attached to the canvas page response (only when no multi-point draft is in progress) and to the inspector panel response; the tool submenu reuses the F-601 shortcut keys; `UiState::pending_action` requests the chosen command and `RubricApp::apply_editor_action` executes it (now also used by the Edit/View menu bar).
- Docs: F-603 marked done in `docs/spec/README.md`, `REQUIREMENTS_TRACEABILITY.md` and `IMPLEMENTATION_PLAN.md`; CHANGELOG + DECISIONS entries added.

### Active
- Next: grid/snap/rulers, packaging (F-7xx).

### Blocked
- None.

## 2026-10-23 — Process alignment with guide §5 (CTO.new-style free process) CI-verified and merged
**Status**: complete — CI SUCCESS on `7bf4f16` (GitHub Actions run `39923530068`, job `check` 1m54s); PR #7 (https://github.com/Perchemop/rubric-desktop/pull/7, base `main`) merged by rebase 2026-10-23T08:24:46Z, `origin/main` = `9d62937`.

### Completed
- Adopted the guide §5 operating model `Chat (plan) → Task (execute) → Pull Request (review) → Merge` across `AGENTS.md`, `docs/development.md` and `docs/qc/checklist.md`; the Definition of Done now requires a Pull Request opened/updated for review and a human merge (trust but verify, guide §9).
- Reframed `.opencode/agents/orchestrator.md` as the Chat/planning role and added `.opencode/agents/coder.md` (Task execution). Subagents still never push; the parent agent commits, pushes and opens the PR.
- Added the required `docs/parity.md` capability-parity tracker (a summary view over `REQUIREMENTS_TRACEABILITY.md`).
- Naming: GitHub Actions is described as the free cloud "Task Runner" wherever the CI-first rule appears.

### Active
- Next: grid/snap/rulers, packaging (F-7xx).

### Blocked
- None.

## 2026-10-23 — Phase 6 grid, snap-to-grid and rulers (F-405/F-406) CI-verified and merged
**Status**: complete — CI SUCCESS on `cbc59cd` (GitHub Actions run `39934068905`, job `check`: fmt, clippy `-D warnings`, tests, release build). PR #9 (https://github.com/Perchemop/rubric-desktop/pull/9, base `main`) merged by rebase, `origin/main` updated.

### Completed
- Domain: new pure module `src/domain/grid.rs` — `Grid` (spacing / visible / snap) with nearest-intersection `snap`, `spacing_px`/`should_draw` (grid hidden below a 6 px pitch), and the 1/2/5 `nice_step` / `ruler_ticks` ladder for ruler labels. 10 new unit tests (snap rounding incl. halfway, disabled/invalid-spacing identity, draw gating, ladder values, tick coverage/offset/degeneracy/zoom).
- UI: `src/ui/canvas.rs` draws `draw_grid` under the markups and `draw_rulers` as labelled page-point ticks on the top/left margins that follow pan and zoom. Snapping applies to markup **creation anchors** only; hit-testing (`pick_markup_id`) uses the exact pointer position so grid-unaligned markups stay selectable.
- View menu: **Show Grid**, **Snap to Grid**, **Grid Spacing** (12/24/36/72 pt).
- Docs: F-405/F-406 marked done in `docs/spec/README.md`, `REQUIREMENTS_TRACEABILITY.md` (R-S-012 → DONE) and `IMPLEMENTATION_PLAN.md`; CHANGELOG, DECISIONS and `docs/parity.md` updated.
- Note: first CI run (`39932966001`) failed clippy (`neg_cmp_op_on_partial_ord` on the grid bounds guard) and a wrong tick expectation; both fixed and re-verified green.

### Active
- Next: page thumbnails (F-404), then packaging (F-7xx).

### Blocked
- None.

## 2026-10-23 — Phase 5 page thumbnails sidebar (F-404) CI-verified and merged
**Status**: complete — CI SUCCESS on `0c25124` (GitHub Actions run `39940499803`, job `check`: fmt, clippy `-D warnings`, tests, release build). PR #10 (https://github.com/Perchemop/rubric-desktop/pull/10, base `main`) merged by rebase, `origin/main` updated.

### Completed
- Domain: new pure module `src/domain/thumbs.rs` — `thumb_layout(width, count, min_cell, gap)` returning per-column/per-row counts, cell width and cell height (thumbnail box + caption), plus `columns_needed` used by the tests as the "one more column would not fit" oracle. 6 new unit tests (always ≥1 column, columns never shrink as the panel widens, rows = ceil(count/per_row), caption/gap arithmetic).
- UI: new `src/ui/pages.rs` — a resizable left "Pages" panel, shown only when the document has pages. PDF-backed projects render each thumbnail through the shared on-disk `data::pdf` page cache at a fixed 192 px width (a page the main canvas already rendered is a cache hit); non-PDF projects and render failures fall back to a numbered placeholder, so the control is never dead. Clicking a cell jumps.
- App shell: `RubricApp::jump_to_page(index)` is now the single page-switch path — commits an in-flight text edit, snapshots only when text actually changed, abandons the draft. The top-bar arrows and Page Up/Down were refactored onto it, removing three divergent implementations of the same semantics.
- Docs: F-404 done in spec, plan, traceability (R-S-011), CHANGELOG, DECISIONS and `docs/parity.md`.
- Note: two CI runs failed first (`39940025456`, `39940317287`) — a `Debug` derive on a `TextureHandle`, an `Option<&Path>` vs `Option<&str>` mismatch and a mixed-error-type `and_then` chain. All fixed and re-verified green.

### Active
- Next: packaging (F-7xx).

### Blocked
- None.

## 2026-10-23 — Phase 6 packaging: CI release + Linux/macOS/Windows installers (F-701–F-704) CI-verified and merged
**Status**: complete — CI SUCCESS on `f941d69` (GitHub Actions run `39949118025`, job `check`: fmt, clippy `-D warnings`, tests, release build). PR #11 (https://github.com/Perchemop/rubric-desktop/pull/11, base `main`) merged by rebase, `origin/main` updated.

Note: this slice is packaging and documentation only — it adds no application code, so the existing CI gate (fmt/clippy/test/release) is the whole verification. The new `release.yml` and `package.yml` workflows themselves are only exercised on a version tag or `workflow_dispatch`; their YAML and the WiX definition were statically validated locally (`yaml.safe_load`, XML parse). Running a real packaging build is the follow-up step below.

## 2026-10-24 — Packaging smoke-test in CI; three real defects found and fixed
**Status**: complete — fix PR #12 merged (`ccd43d9`), CI SUCCESS; a fresh ARM64 release build was dispatched to install Rubric on this (aarch64) machine.

### Verified by actually running the workflows
- **Release build succeeded on all three original targets** (Linux x86_64, macOS arm64, Windows x86_64) — the release pipeline itself works.
- **macOS `.dmg` build succeeded** in `package.yml`.
- Three defects the smoke-test exposed, all now fixed:
  1. `publish` job failed with `fatal: not a git repository` — it had no `actions/checkout`, which `gh release create` needs to resolve the repo. Fixed (PR #12).
  2. Manual-dispatch runs derived the release tag from the branch name, which would have created a tag colliding with `main`. Dispatch runs now get their own `dispatch-<timestamp>` pre-release tag.
  3. The no-`zip` fallback renamed a `.tar.gz` to `.zip`, producing a mislabeled archive. It now keeps the correct `.tar.gz` name.
  4. Linux AppImage build failed with `cp: cannot create regular file 'AppDir/.../scalable/apps/rubric.svg'` — the icon directory was never created. Fixed by adding the `mkdir -p`.
  5. Windows MSI failed with WiX `WIX0150: Undefined preprocessor variable '$(var.PdfiumDllPath)'` — the workflow now defines `PdfiumDllPath` and `PDFIUM_DLL_FOUND`.
- **Local install on this aarch64 machine**: the x86_64 Linux artifact was downloaded and installed to `~/.local/share/rubric` + `~/.local/bin/rubric`, but it is the wrong architecture for the host (`uname -m` = aarch64) and cannot execute. `release.yml` gained a **`linux-arm64` target on GitHub's `ubuntu-24.04-arm` runner** so a native build is available; that run is in progress.

### Completed
- `.github/workflows/release.yml` (F-704): builds the release binary for Linux x86_64, macOS arm64 and Windows x86_64 (a fail-fast:false matrix so one platform failing does not hide the others), archives each, and attaches them to a GitHub release on `v*` tags; `workflow_dispatch` publishes a pre-release so a test run never looks like a real release.
- `.github/workflows/package.yml` (F-701/F-702/F-703): Linux **AppImage** via linuxdeploy over an AppDir containing a real `.desktop` entry and appstream metadata, plus a portable tarball fallback; macOS `Rubric.app` → `.dmg`; Windows MSI via WiX v4 with the product definition kept in-repo at `packaging/wix/Product.wxs` (MIT licence text included).
- PDFium (the run-time dependency) is bundled next to the executable on all three platforms when present, and its contract — discovery order `RUBRIC_PDFIUM_PATH` → executable directory → system path, where to obtain binaries, what a failed import looks like — is documented in `docs/pdfium.md`.
- Honesty gaps recorded rather than papered over: installers are **unsigned** (no signing secret in the repo → Q-PACK-1) and icons remain the repo's SVG (→ Q-PACK-2 raster follow-up).
- Validation performed locally (lightweight, permitted): all three workflow YAMLs parse (`yaml.safe_load`), and `Product.wxs` is well-formed XML. No build/test was run locally — CI is the verifier.
- Docs: spec F-701–F-704 and R-S-017 → done; `docs/parity.md`, IMPLEMENTATION_PLAN, CHANGELOG, DECISIONS updated; Q-PACK-1..3 added to assumptions.

### Active
- Next: verify this slice in CI, then the recorded follow-ups (installer signing, raster icon set).

### Blocked
- None.

## 2026-10-24 — v0.1.0 tagged; packaging smoke-tests exposed and fixed two more release defects
**Status**: v0.1.0 tag pushed; release build `40047819071` in progress on the fixed workflow.

### Found by smoke-testing the release path
- **Windows artefact never existed**: on Windows runners `zip` is not on `PATH`, so the "fallback" created a `.tar.gz` while `upload-artifact` demanded `rubric-windows-x86_64.zip` — the job failed and `publish` (correctly) skipped. Fixed in `release.yml`: the archive step now uses `zip`, else `7z`, else PowerShell `Compress-Archive`, and asserts the file exists before finishing.
- First v0.1.0 tag run was discarded: the tag was deleted and re-created at the fixed commit, because a re-run would have reused the tag's original (broken) workflow file.
- **`.opencode/` is in ci.sh's ignore list**: changes to agent definitions are silently excluded from ci.sh commits. The agent "no cargo locally" edits were therefore lost in a `git reset --hard` and had to be reapplied and committed with plain `git add`/`commit` on a branch (PR #15). Lesson recorded: anything under `.opencode/` must be committed explicitly.

### State
- v0.1.0 gate (guide §6.1) is otherwise satisfied and evidence-backed; the remaining step is the published GitHub Release from this run.

## 2026-10-24 — Guide v4 (suite-ready) alignment: workspace + shared crates + process files; CI verified
**Status**: complete — CI SUCCESS on `e27ba35` (runs `40045627083` and `40045810962` failed first: a clippy `manual_pattern_char_comparison`, the moved icons path, and an egui `Context` lifetime escaping into the update-check worker; all fixed). PR #14 (https://github.com/Perchemop/rubric-desktop/pull/14, base `main`) merged by rebase, `origin/main` updated.

### Completed
- **Workspace restructure (guide §7.3/§12)**: repo is now a Cargo workspace — `apps/rubric` (the app, package name and build output unchanged) plus shared crates. Icons path adjusted for the new depth; CI commands (`cargo fmt/test/clippy --all`) unchanged.
- **`crates/crash-report`** (guide §7.1, 12 tests): panic hook capturing app version / OS / backtrace with home paths and token-like material redacted; report → issue title/body/labels; pre-filled New Issue URL fallback; submission requires a **user-supplied** token (never embedded); truncation is char-boundary safe.
- **`crates/update-check`** (guide §7.2, 13 tests): GitHub Releases API query, semantic version comparison (0.2.0 > 0.10.0), drafts/prereleases ignored, fail-soft `Unknown` for offline/rate-limit/bad payloads, 1-hour in-memory cache.
- **App wiring**: panic hook installed at startup (report parked in the config dir for review); Help ▸ **Check for Updates…** runs on a worker thread with an honest result window (running / up to date / available with release link / could-not-check, retry); Help ▸ **Report a Bug…** shows a review-and-cancel dialog with editable notes and a browser-first delivery default. Privacy documented in `README.md`.
- **Process files**: `.github/PULL_REQUEST_TEMPLATE.md` (guide §8.3, verbatim), `ROADMAP.md` (evidence-based: 13/17 requirement groups, 46/52 feature IDs done), `CONTRIBUTING.md` (PR licence terms + DCO sign-off), `docs/design/brand.md`, `docs/research/` policy with the §8.4 human gate.
- **Spec reconciliation for honest readiness**: 20 feature-ID notes updated against code evidence (e.g. F-104/F-105 in-canvas editing, F-108 multi-point, F-110 ink, F-112 drag preview, F-207/F-208 scale editor, F-304 report export, F-401 wheel zoom, F-402 pan are all implemented); traceability R-S-005/R-S-006 → DONE. Genuine remaining gaps: F-103, F-114, F-115, F-502, F-507, F-605.
- **Release notes standard (guide §6.3)**: `scripts/release-notes.py` extracts the tagged version's CHANGELOG section (with an explicit disclosure line when falling back to [Unreleased]); `release.yml` now attaches a `SHA256SUMS` file and publishes notes with Summary / Roadmap progress / Added / Changed / Fixed / Known limitations. CHANGELOG restructured: Unreleased work folded into a dated `[0.1.0] - 2026-10-24` section with Summary, Roadmap progress and Known limitations; stale "no PDF rendering" note corrected.
- Docs: AGENTS.md (PR template, roadmap, research gate, suite crates), `docs/development.md`, `docs/qc/checklist.md` (legal checklist §2.5), `docs/current-model-map.md` (research-method ladder), assumptions Q4/Q18/Q26/Q27 answered, DECISIONS entry.

### Active
- Next: verify in CI, merge, then tag **v0.1.0** and confirm the GitHub Release publishes with artefacts + checksums + notes.

### Blocked
- None.

## 2026-10-24 — v0.1.0 published; operating-guide Part II (opensource.guide alignment) implemented
**Status**: Release **v0.1.0 is live** (not draft, not prerelease, "Latest");
Part II requirements implemented and tracked as R-C-001…R-C-012; PR-based
flow now enforced by branch protection.

### v0.1.0 release (final resolution)
- Run `40048805198` failed in `publish` with exit 126: the multi-line
  `steps.collect.outputs.files` value was substituted inline into `run:`,
  the shell executed the second path as a command (one asset landed:
  `rubric-linux-arm64.zip`).
- Fix `1a6ef10` (run `40049997043`, SUCCESS): artefact list passes via env
  var + `mapfile`; all four platform zips + `SHA256SUMS` uploaded.
- The release was in **draft** because deleting the old tag turned the
  existing Release entry into one; finalised via `gh release edit
  --draft=false`; `release.yml` now always finalises drafts after upload
  (commit `7a47a75`, PR #16).

### Part II implementation (this wave)
- Files added: `CODE_OF_CONDUCT.md`, `SECURITY.md` (threat model +
  incident response), `ACCESSIBILITY.md`, `VISION.md`, `GOVERNANCE.md`,
  `CONTRIBUTORS.md`, `.github/ISSUE_TEMPLATE/{bug_report,feature_request,
  accessibility_report,config}.yml`, `.github/dependabot.yml`, `deny.toml`,
  `scripts/scan-secrets.sh`, `.github/workflows/security.yml`,
  `docs/metrics.md`.
- Files updated: PR template (a11y checklist), CONTRIBUTING (links +
  honest licence terms), README (community table), ATTRIBUTION (licence
  policy + AI tooling record), docs/current-model-map (§2.4 + §19.7),
  docs/assumptions-and-questions (Q28 name check, Q29 employer/IP,
  Q-LIC-1, six §26 resolutions), ROADMAP, REQUIREMENTS_TRACEABILITY
  (R-C-001…R-C-012), CHANGELOG, IMPLEMENTATION_PLAN, docs/qc/checklist.
- Repo settings (GitHub API, verified): branch protection on `main`
  (required check `check`, enforce_admins, no force pushes, conversation
  resolution); private vulnerability reporting (204); Dependabot security
  updates enabled; Discussions enabled; labels `security`, `core`, `ui`,
  `a11y` created (existing: good first issue, help wanted, documentation,
  accessibility, bug).
- CI hardening: every workflow action pinned to a commit SHA; new actions
  pinned at v4 (dependency-review `2031cfc0…`, CodeQL `8498dc4c…`,
  sbom-action `fffec3d8…`, cargo-deny-action `c3bbe7e4…`).
- Name check (Q28): unrelated dormant `rubric` crate on crates.io (2020,
  grade/academia) + unrelated PyPI package; no trademark conflict; home
  URL stays `Perchemop/rubric-desktop`.

### Process change (effective immediately)
- Branch protection enforces CI + PR on `main` for everyone (admins
  included): **no more direct pushes**, including docs. All future waves go
  through the Chat → Task → PR → Merge cycle via `ci.sh`.

## 2026-10-24 — Packaging smoke-test: Windows artefact step fixed twice, v0.1.0 re-tagged
**Status**: fix pushed as `7198393`; tag re-created at that commit; release run `40048805198` monitoring in progress.

### Found by smoke-testing the release path
- **First Windows failure**: Windows runners have no `zip` on `PATH` inside bash, so the old fallback produced `rubric-windows-x86_64.tar.gz` while `upload-artifact` demanded `rubric-windows-x86_64.zip` → job failed, `publish` (correctly) skipped. Fixed with a `zip` → `7z` → PowerShell `Compress-Archive` chain plus a `test -f` assertion.
- **Second Windows failure (same step, silent)**: the job aborted ~2s after the build finished with exit code 1 and no captured error. Root cause was diagnosed from the job log: the `command -v powershell.exe` probe is unreliable in git-bash on the Windows runner, and the previous script had no `set -e`/pipefail, so the failure mode was opaque.
- **Final fix** (`release.yml`, commit `7198393`): `set -euo pipefail`; the binary is located by probing both `rubric` and `rubric.exe` (copy fails loudly with a directory listing if neither exists); zip creation uses `zip` when present and otherwise calls `powershell.exe` directly (with a `powershell` fallback), and every branch echoes which tool it used.
- **Release-path lesson**: a `workflow_dispatch` re-run reuses the workflow file *from the tag's commit*, so workflow fixes must move the tag. `v0.1.0` was therefore deleted and re-created at the fixed commit (twice), rather than re-running the existing run.

## 2026-10-26 — v0.2-01 domain slice: Markups List, style presets, project diff — CI verified, reviewer fixes applied
**Status**: domain slice complete and CI-green (final code commit `c03cfb9`, GitHub Actions run `40305909163`: fmt check, clippy `-D warnings`, full tests, release build; security checks also green; the slice commit `7e33fd2` was green in run `40304244261`). PR #32 was merged at `59694b2`; PR #30 was the original feature branch and is closed as superseded. UI panel is the next slice.

### Completed
- `MarkupList` snapshot (`src/domain/markup_list.rs`): case-insensitive filters (status/author/subject/kind/page/layer incl. default-layer handling), search over author/subject + comment texts for full markups (summary line for rows), stable sorts with deterministic id tie-break, grouping by status/author/subject/kind/page/layer. `matches`/`matches_row` now share one criteria helper (`matches_core`), and Page groups sort numerically (1, 2, 10).
- `StylePreset` + `StylePresetLibrary` (`src/domain/style_preset.rs`): capture/apply style, status and lock; fills only empty author/subject; `apply_to` now calls `touch()` so `modified` stays truthful; library capped at 64 with oldest eviction, upsert dedupe and case-insensitive lookup. Full-equality serde roundtrip test.
- `DiffResult` (`src/domain/diff.rs`): diff by markup UUID with 2-dp geometry fingerprints; added/removed/changed with before/after snapshots plus per-page groups; style and locked changes are diff-visible; duplicate ids in malformed documents are diffed once on the added, removed and changed paths; `diff_projects` treats a project without an active document as empty instead of reporting "no changes".
- Tests: module unit tests plus integration coverage in `tests/domain_tests.rs` (list filter/sort from a document, diff add/change with controlled ids, style preset apply + serde roundtrip), Page-group numeric-order test and PageAsc/PageDesc tie-stability test.
- Docs: Stirling PDF added as capability-level inspiration (`docs/inspiration.md`, `docs/research/stirling-pdf-capabilities.md`, radar addendum in `docs/repo-radar.md`, `DECISIONS.md`); PROGRESS, CHANGELOG, ROADMAP, IMPLEMENTATION_PLAN and REQUIREMENTS_TRACEABILITY updated with R-V02-01/R-V02-02.

### Reviewer verdict (FIX FIRST) — all actionables closed
- **MAJOR**: `MarkupSummaryRow`/`rows_equal` omitted `MarkupStyle` (and `locked`), hiding restyling from the diff — both fields added with a style-only/lock-only change test.
- **MEDIUM**: `StylePreset::apply_to` left `modified` stale — `touch()` added.
- **MEDIUM**: `diff_projects` returned an empty result for an absent active document — missing side now treated as an empty document.
- **LOW/INFO**: Page group lexicographic order fixed numerics; `matches`/`matches_row` duplication removed via `matches_core` (search haystacks stay intentionally different); integration tests added; preset roundtrip now full equality; duplicate-id diff input deduped.
- **Deferred to the UI slice**: preset application overwriting review status/lock is documented behaviour; confirm the affordance (and whether it needs a gate) when the Markups List panel lands.

### Process
- CI verification branches are transient: this slice's checks ran on `opencode/ci`, while the original feature branch is `opencode/v0.2-01-markup-list-domain` (PR #30, closed as superseded by #32). Recorded in `docs/process-feedback.md`.

## 2026-10-26 — v0.2-02 Markups List UI (R-V02-01) — reviewer fixes applied
**Status**: implemented on `opencode/v0.2-02-markups-list-ui`; implementation commit `c2687d9` is CI-green (run `40308480821`: fmt, clippy `-D warnings`, tests, release build; security checks also green). Reviewer fixes and docs are in commit `1a5c539`, also CI-green (run `40310526592`, security run `40310526743`). Per the owner directive of 2026-10-26, PR #34 is merged automatically once the final checks are green.

### Completed
- New `apps/rubric/src/ui/markups_list.rs`: resizable bottom panel built on `MarkupList::from_document`, with search plus status/author/subject/layer filters (including the default-layer case), every `MarkupListSort`, every `MarkupGroupKey`, and row selection that selects the markup and jumps to its page.
- Panel integration in `ui/app.rs`: `UiState` filter/sort/group fields, `MarkupLayerFilter`, a View → Show Markups List toggle, and action handling that never pushes an undo snapshot for pure selection.
- Empty states: no active document, no markups, and no filter matches, plus Clear filters. Status is always shown as text, with colour as a secondary cue only.
- Grouping identity fix: the domain label remains the widget `id_salt`; the default layer is renamed only in the visible header, so a layer literally named "Default layer" cannot collide.
- Tests: five headless UI-state helper tests (filter translation, default/named layers, blank/all, case-insensitive dedupe, truncation) and a new domain test proving an empty layer set plus `include_default_layer` keeps only the default layer.
- Docs: `docs/tasks/v0.2-02-markups-list-ui.md`; PROGRESS, CHANGELOG, ROADMAP, IMPLEMENTATION_PLAN, REQUIREMENTS_TRACEABILITY and `docs/parity.md` updated. `AGENTS.md` merge-policy wording is now consistent with the owner directive.

### Reviewer verdict (FIX FIRST) — all actionables closed
- **MEDIUM**: tracking files were not updated — now updated across the roadmap, plan, changelog, traceability, parity and progress records.
- **LOW**: no domain test for the empty-layer-set + default-layer contract — added `filter_layers_empty_set_keeps_only_default_layer`.
- **LOW**: renaming the default group could collide with a layer name — fixed by keeping the original label as `CollapsingHeader::id_salt` and the grid identity, renaming only the visible text.

### Process
- CI runs on the feature branch (`CI_BRANCH=opencode/v0.2-02-markups-list-ui`) so the PR is the record; PR #34 replaces the earlier shared-branch pattern. The merge follows the 2026-10-26 owner directive without a manual merge step.

## Summary
Phases 1–4 complete and CI-green. Phase 5 complete (dialogs, recents, PDF
import, report export, page image export, print). Phase 6 in progress: dark
theme (F-602), per-tool keyboard shortcuts (F-601) and editor context menus
(F-603) merged. Process follows guide §5 (Chat → Task → PR → Merge) with GitHub
Actions as the free Task Runner. CI-first throughout.
