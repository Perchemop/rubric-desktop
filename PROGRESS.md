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
- Pushing slice B; CI is the compile/clippy/test gate.

### Blocked
- None.

## Summary
Phase 1 complete; Phase 2 slices A (drag-to-draw/zoom/pan/hit-testing) and B
(multi-point polyline/polygon/angle) implemented via the subagent pipeline
(domain → ui → reviewer → fixer → CI). Next: measurement labels + scale editor
(Phase 3), then review-workflow UI (Phase 4).
