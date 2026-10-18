# Test Plan

## Scope
Headless, fast tests covering domain logic, geometry, measurements, hit testing, persistence. UI rendering not tested (egui).

## Unit tests (in modules)
- geom: from_corners normalises, from_bounds, union/intersect, path length, closed path length, polygon area, angle degrees, circle helpers.
- units: inch/foot conversions, scale roundtrip, formatting suffixes (area ° count).
- markup: length/radius/area/angle/count measurement computation with scales; hit testing for segment/bounds/highlight; translate moves geometry; bounds tight for paths.
- document: add/remove pages, markups per page, find, touch updates modified.
- project: add/remove documents, tool sets, settings, history undo/redo (can_undo/can_redo, truncate redo on push, overwrite_current, clear).

## Integration tests (tests/domain_tests.rs)
- document lifecycle
- markup serialisation roundtrip
- geometry calculations
- measure length and radius with foot_scale
- review status and style defaults
- project save/load JSON roundtrip (pretty, human-readable)

## Quality gates
- `cargo test` — all pass
- `cargo clippy -- -D warnings` — zero warnings
- `cargo fmt --all -- --check` — formatted

## Coverage goals (Phase 0)
Focus on logic correctness: geometry (shoelace, point-segment distance), measurement maths, hit testing edge cases (tolerance, highlight fills vs outline), history branch truncation.

## Running tests
```bash
cd ~/rubric
cargo test --all
cargo test --lib
cargo test --tests
```
