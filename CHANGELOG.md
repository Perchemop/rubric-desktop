# Changelog

All notable changes to this project will be documented here. Format: semantic-ish for this repo (Added/Changed/Fixed/Docs).

## [Unreleased]

### Added
- Phase 1 requirement set and spec (`docs/spec/README.md`): 52 feature IDs across F-1xx…F-7xx with priorities mapped to implementation phases; `REQUIREMENTS_TRACEABILITY.md` extended with spec-level requirements R-S-001…R-S-017 and seven group rows.
- Phase 2 slice A — interactive canvas: drag-to-draw with live preview (bounds/segment), ink freehand drawing, wheel zoom anchored on cursor (10–1600%), Space/middle-button pan, Del to delete selection.
- Phase 2 slice B — multi-point drawing: click-to-append polyline/polygon/perimeter/area paths with rubber-band preview (commit via double-click/right-click/Enter, Esc cancels), three-click angle tool with live ray preview.
- Phase 3 — measurement labels rendered on canvas (domain-formatted, geometry-anchored, screen-space size); inspector scale editor with presets (1:1, 1 in = 1/10/50 ft), display-unit combo, and custom paper-inches : real-units ratio; scale/unit edits recompute page measurements with one undo snapshot per commit.
- Measurement precision preference: `Settings::precision` (0–6 decimal places, serde default 3 so older `.rubric.json` files load unchanged), `format_value_precise`/`Measurement::label_with`, and an inspector DragValue that commits one snapshot per edit. Angle stays at 2 decimals, counts at 0.
- Layer visibility/lock: `Layer` domain model (visible/locked/z, serde-migrated — legacy `.rubric.json` files load onto a per-document default layer), document ops (`add_layer`, `remove_layer` refusing default/populated layers, visibility/lock setters, `is_markup_visible`/`is_markup_editable` predicates, fail-open on unknown ids), inspector Layers section (eye/lock toggles, add/delete with status feedback), layer ComboBox for the selected markup, and canvas/inspector/delete gates so hidden markups aren't painted or hit-tested and locked markups can't be moved, deleted, or edited. 15 new tests.
- Hit testing v2: filled shapes hit interior, stroke-only shapes hit outline only; elliptical edge test; polygon even-odd interior test; ink segment-based (incl. dot strokes) — 8 new unit tests.
- Subagent suite (`.opencode/agents/`): `domain-dev`, `test-writer`, `ui-dev`, `reviewer`, `docs-writer`, `fixer` — all on free OpenCode models, restricted to `cargo fmt` + read-only git locally.
- CI-first build policy: all heavy gates (`cargo test`, `clippy -D warnings`, `check`, release build) run only in GitHub Actions; workflow now caches builds with `Swatinem/rust-cache@v2`.

### Fixed
- Area measurement rescaling: area values now scale with `units_per_point²` (previously used the linear factor, and carried an unnecessary cast).
- Switching the display unit now preserves physical measurements (`Scale::with_display_unit` converts `units_per_point` via `points_per_unit`) instead of silently reinterpreting the scale ratio.
- Undo history no longer spams snapshots during selection drags (one snapshot per completed drag instead of dozens).
- Esc now abandons an in-progress drag-to-draw instead of re-anchoring the draft.
- Tool switching now discards an in-progress multi-point draft (no ghost preview or stale Enter/right-click commits).
- Segment tools (line/arrow/length/diameter/radius) no longer commit wrong-typed geometry on a bare click; multi-point commits validate distinct points; double-click no longer adds a duplicate final vertex.
- Enter/Space fake-clicks while a text field has focus can no longer trigger canvas commits.
- Docs: stale markup-kind count corrected — `MarkupKind` has 23 variants (traceability, data model, plan, progress, decisions, and the 0.1.0 note said 22).

### Changed
- CI verified fully green (fmt, clippy, tests, release build) after billing resolution; process docs updated accordingly.

## [0.1.0] - 2026-10-17 — Phase 0

### Added
- Original name/brand "Rubric": SVG logo, multi-size PNG icons (16/32/48/64/128/256/512/1024), branding spec.
- Fresh Rust crate `rubric` (egui/eframe 0.36, serde, uuid, thiserror, tracing, chrono, image).
- Domain-first architecture: geom, units (scale + conversions), style (colours/dashes/fill modes), markup (23 kinds, geometry variants, measurements, hit testing, review status, comments), document (pages/markups), project (toolsets, settings, history with undo/redo capped 128).
- Data layer: JSON persistence (`.rubric.json`), app data dir via directories, error types.
- UI skeleton: app shell (panels, menu, shortcuts), canvas (page, markups, selection, basic place/move), toolbar (tool groups), inspector (properties/comments/measurements), theme/icons.
- Tests: unit tests in domain modules + integration tests (6) covering geometry, measurements, serialisation, project lifecycle.
- Docs: README, AGENTS, architecture, data model, test plan, assumptions/questions, implementation plan, requirements traceability, decisions, progress, changelog.

### Notes
- Headless-testable (domain + data). UI requires display at runtime; builds/tests/lint/format work in headless environment.
- No PDF rendering yet (blank page rectangle); to be added later.
- Never commits external reference documents or the build guide.
