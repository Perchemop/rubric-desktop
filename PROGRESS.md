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

## Summary
Phases 1–4 complete and CI-green. Phase 5 complete (dialogs, recents, PDF
import, report export, page image export, print). Phase 6 in progress
(dark theme). CI-first throughout.
