# AGENTS.md

This document provides guidance for AI coding agents working on **Rubric**.

## Purpose
Build a free, cross-platform Rust PDF markup/measurement desktop app (egui/eframe). Adhere to the build guide; never commit/push any notes about other products or the build guide itself. Maintain quality gates.

## Codebase rules
- Rust only, 100% free OSS.
- Domain-first: `src/domain/` is pure (no UI), fully serialisable with serde, testable headlessly.
- Formatting: `cargo fmt --all -- --check` and formatted code. No trailing whitespace.
- Linting: `cargo clippy -- -D warnings` (treat warnings as errors).
- Testing: `cargo test` before committing. Add/extend tests when changing behaviour or fixing bugs.
- Error handling: use `thiserror` for domain/data errors; avoid panics in logic; prefer `Result`.
- Conventions: snake_case, modules flat and explicit; doc comments for public API.
- Dependencies: prefer well-maintained, MIT/Apache-2.0 crates.
- UI: egui/eframe; viewport with app icon; responsive panels; keyboard shortcuts (Ctrl+S save, Ctrl+Z undo, Ctrl+Shift+Z redo, Esc/Del).
- Persistence: JSON (serde_json), `.rubric.json` extension, human-readable; include UUIDs and timestamps.
- Git: fresh repo, never commit external reference material; commit small, focused slices; messages imperative and descriptive.

## Testing strategy
- Unit tests in `src/domain/*` modules and `tests/domain_tests.rs`.
- Headless tests must not require a display. UI is integration surface; logic lives in domain.
- Roundtrip tests: serialise → deserialise and assert equality.

## Implementation guidance
- Build in vertical slices; update PROGRESS.md, CHANGELOG.md, DECISIONS.md, REQUIREMENTS_TRACEABILITY.md as you go.
- When adding features: define domain types first, add tests, then UI.
- Measurements: recompute on geometry change using page scale; store `value_points` to enable rescaling.
- Hit testing: use tolerance; prefer segment/bounds hit tests per geometry.
- Undo/redo: project snapshots in `History` (bounded to 128). Push snapshot on destructive/committed actions; avoid spamming during live drags.

## Do not
- Commit any notes about other products or external reference material.
- Introduce AGPL/GPL dependencies without approval.
- Mix UI and business logic.
- Push without running quality gates.
