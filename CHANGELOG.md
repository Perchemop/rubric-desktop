# Changelog

All notable changes to this project will be documented here. Format: semantic-ish for this repo (Added/Changed/Fixed/Docs).

## [Unreleased]

### Fixed
- Area measurement rescaling: area values now scale with `units_per_point²` (previously used the linear factor, and carried an unnecessary cast).
- Docs: stale markup-kind count corrected — `MarkupKind` has 23 variants (traceability, data model, plan, progress, decisions, and the 0.1.0 note said 22).

### Added
- Phase 1 requirement set and spec (`docs/spec/README.md`): 52 feature IDs across F-1xx…F-7xx with priorities mapped to implementation phases; `REQUIREMENTS_TRACEABILITY.md` extended with spec-level requirements R-S-001…R-S-017 and seven group rows.
- Subagent suite (`.opencode/agents/`): `domain-dev`, `test-writer`, `ui-dev`, `reviewer`, `docs-writer`, `fixer` — all on free OpenCode models, restricted to `cargo fmt` + read-only git locally.
- CI-first build policy: all heavy gates (`cargo test`, `clippy -D warnings`, `check`, release build) run only in GitHub Actions; workflow now caches builds with `Swatinem/rust-cache@v2`.

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
