# Changelog

All notable changes to this project will be documented here. Format: semantic-ish for this repo (Added/Changed/Fixed/Docs).

## [0.1.0] - 2026-10-17 — Phase 0

### Added
- Original name/brand "Rubric": SVG logo, multi-size PNG icons (16/32/48/64/128/256/512/1024), branding spec.
- Fresh Rust crate `rubric` (egui/eframe 0.36, serde, uuid, thiserror, tracing, chrono, image).
- Domain-first architecture: geom, units (scale + conversions), style (colours/dashes/fill modes), markup (22 kinds, geometry variants, measurements, hit testing, review status, comments), document (pages/markups), project (toolsets, settings, history with undo/redo capped 128).
- Data layer: JSON persistence (`.rubric.json`), app data dir via directories, error types.
- UI skeleton: app shell (panels, menu, shortcuts), canvas (page, markups, selection, basic place/move), toolbar (tool groups), inspector (properties/comments/measurements), theme/icons.
- Tests: unit tests in domain modules + integration tests (6) covering geometry, measurements, serialisation, project lifecycle.
- Docs: README, AGENTS, architecture, data model, test plan, assumptions/questions, implementation plan, requirements traceability, decisions, progress, changelog.

### Notes
- Headless-testable (domain + data). UI requires display at runtime; builds/tests/lint/format work in headless environment.
- No PDF rendering yet (blank page rectangle); to be added later.
- Never commits external reference documents or the build guide.
