# Architecture

## Goals
- Domain-first: all core logic (geometry, markups, measurements, documents, projects, history) is pure Rust and testable without a GUI.
- Cross-platform desktop: eframe/egui for one binary across Linux/macOS/Windows.
- Local-first: projects saved as JSON; no network, no cloud.
- Deterministic: UUIDs, timestamps (UTC), explicit coordinate spaces.
- Extensible: easy to add markup kinds, measure kinds, units, persistence backends.

## Layers
- `src/domain/`: core types and algorithms (geom, units, style, markup, document, project)
- `src/data/`: serialization/persistence (store, error)
- `src/ui/`: rendering and interaction (app, canvas, toolbar, inspector, theme, icons)
- `tests/`: black-box/domain tests
- `assets/`: branding assets

## Coordinate system
PDF points (1 pt = 1/72 inch), origin at top-left, y down. All canvas/geometry maths in page space. Screen mapping: `screen = origin + pan + (pt * zoom)`.

## Rendering
egui immediate mode. Canvas renders page rect, then all markups, selection handles, draft. Hit testing against page space.

## State management
- `History<Project>` holds snapshots (bounded 128) for undo/redo.
- `UiState` tracks selection, tool, inspector mode, camera (zoom/pan/origin), canvas size, hints/status, dirty flag.
- `DrawState` tracks in-progress drawing (Idle/Ink/Bounds/Segment/Path/Angle/Count).

## Measurements
`Page.scale` defines `units_per_point`. `Measurement { value_points, value, kind }`. `Markup::recompute_measurement(scale)` converts page-space lengths/areas to real units; angle/count treated separately. Radius = half diameter segment length.

## Persistence
Projects serialised to `.rubric.json` as pretty JSON via serde_json. Timestamps are `DateTime<Utc>` (chrono). UUIDs for entities.

## Error handling
`DataError` (thiserror) covers IO/serde/path issues. Domain logic returns `Option`/`Result` as appropriate; avoid panics.

## Concurrency
Single-threaded UI + sync domain logic. No async needed for slice 1.

## Test strategy
Headless unit tests: geometry, measurements, hit testing, serialization roundtrips. Integration test `tests/domain_tests.rs` covers project/document lifecycle and persistence.
