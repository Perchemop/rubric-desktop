# DECISIONS.md

Record significant architectural/implementation decisions.

## 2026-10-17: Project name and branding
- **Decision**: Name = "Rubric". Reason: historical "rubric" is a red mark on documents (perfect for markup/annotation), short, punchy, memorable, original.
- **Branding**: SVG logo with red rubric/check mark over document page; indigo palette + red accent.
- **Impact**: Product identity, binary/crate name `rubric`, paths, UI title.

## 2026-10-17: Tech stack
- **Decision**: Rust + egui/eframe (immediate mode). Reason: single binary, cross-platform, free OSS, no Electron/webview overhead; fast iteration, good for canvas + inspectors.
- **Alternatives considered**: iced (more opinionated), slint (declarative), wgpu custom (heavy), Tauri+web (two languages). egui chosen for speed to MVP.
- **Impact**: UI paradigm (immediate mode), API style.

## 2026-10-17: Domain-first architecture
- **Decision**: All business logic in `src/domain/` (pure), data in `src/data/`, UI in `src/ui/`. No UI deps in domain.
- **Rationale**: testability headlessly, clear boundaries, easy to evolve.
- **Impact**: types serialisable (serde), unit tests cheap.

## 2026-10-17: Coordinate space and measurements
- **Decision**: Page space in PDF points (y-down). Scale per page as `units_per_point`. Store `value_points` alongside computed `value`.
- **Rationale**: matches PDF coordinate system; enables rescaling without recomputing from points.
- **Impact**: geometry maths, canvas mapping, measurement display.

## 2026-10-17: Persistence format
- **Decision**: Pretty JSON `.rubric.json` sidecar (separate from PDF). UUIDs + UTC timestamps.
- **Rationale**: human-readable, diffable, non-destructive, easy to debug/version.
- **Impact**: store.rs, file extension, autosave.

## 2026-10-17: Undo/redo strategy
- **Decision**: Full project snapshots in `History`, bounded to 128. Push on committed actions; truncate redo branch on new push. `overwrite_current` for live drags (to avoid history spam).
- **Rationale**: simple, correct, low risk for small docs; easy to reason about.
- **Impact**: memory small (clones); history.push calls needed.

## 2026-10-17: Markup model
- **Decision**: Flat `MarkupKind` enum (23 variants) + `MarkupGeometry` enum. No trait objects for serialisation/simplicity.
- **Rationale**: exhaustive matching, serde-friendly, cheap clones.
- **Impact**: all drawing/rendering/hit-testing switch on kind/geometry.

## 2026-10-17: Measurements
- **Decision**: Compute via geometry: length (segment), perimeter (closed polyline length), area (shoelace), angle (dot product), diameter/radius (segment length), count (fixed). Area unit gets `²` suffix.
- **Rationale**: standard, deterministic measurement behaviour.
- **Impact**: markup.rs recompute_measurement.

## 2026-10-17: Hit testing
- **Decision**: Per-geometry hit tests with tolerance. For filled/highlight, inside bounds counts. For outline rect, only border-near counts (inside excluded). Segment/polyline/ink test point-segment distance.
- **Rationale**: feels intuitive; avoids selecting filled shapes when clicking empty space.
- **Impact**: select/move UX.

## 2026-10-17: Dependencies (free-only)
- **Decision**: eframe/egui, egui_extras, image (png), serde/serde_json, uuid, thiserror, tracing/tracing-subscriber, directories, chrono.
- **Rationale**: all MIT/Apache-2.0; cached locally; no AGPL.
- **Impact**: Cargo.toml, license compliance.

## 2026-10-17: External material stays out of the repo
- **Decision**: Never commit external reference documents, notes about other products, or the build guide. Keep them outside the repo; .gitignore enforces.
- **Rationale**: containment; separate concerns.
- **Impact**: .gitignore, AGENTS.md.

## 2026-10-17: Quality gates
- **Decision**: Enforce `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`, `cargo test` (lib + tests). CI skeleton added.
- **Rationale**: Prevents drift, catches bugs early.
- **Impact**: AGENTS.md, CI.

## 2026-10-19: Phase 1 requirement set adopted
- **Decision**: Adopted Phase 1 requirement set (`docs/spec/README.md`) as the canonical feature inventory; requirements use F-1xx…F-7xx groups and feed REQUIREMENTS_TRACEABILITY.md.
- **Rationale**: one source of truth for scope and priorities; traceability links spec IDs → R-S requirement IDs → planned modules → tests.
- **Impact**: `docs/spec/README.md` (52 F-IDs), REQUIREMENTS_TRACEABILITY.md (R-S-001…R-S-017 + seven group rows), IMPLEMENTATION_PLAN.md Phase 1 ticked.
