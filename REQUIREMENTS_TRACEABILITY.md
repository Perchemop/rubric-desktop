# REQUIREMENTS_TRACEABILITY

This tracks mapping from requirements to implementation (files, modules, tests). Keep updated as work progresses.

## High-level requirements
| ID | Requirement | Status | Trace |
|---|-----------|--------|------|
| R-G-001 | Rust only, free OSS | DONE | Cargo.toml deps MIT/Apache-2.0 |
| R-G-002 | Cross-platform (Linux/macOS/Windows) | DONE | eframe/egui, target-agnostic code |
| R-G-003 | Original branding (name, logo, icons, palette) | DONE | assets/logo.svg, assets/icons/*, assets/branding.md |
| R-G-004 | Fresh git repo; exclude external material | DONE | .gitignore, local repo |
| R-G-005 | Domain-first, testable headlessly | DONE | src/domain pure; tests in src + tests/ |
| R-G-006 | Build/test/clippy/fmt gates | DONE | scripts, CI skeleton, AGENTS.md |
| R-G-007 | Small vertical slices; docs updated | IN PROG | plan/traceability/progress/decisions/changelog |
| R-G-008 | No cloud/collaboration features | DONE | out of scope (noted in assumptions) |

## Domain model requirements (Slice 1)
| ID | Requirement | Status | Trace |
|---|-----------|--------|------|
| R-D-001 | Geometry: Pt, Rect, path length, area (shoelace), angle | DONE | src/domain/geom.rs + unit tests |
| R-D-002 | Units + scale (points-based; units_per_point) | DONE | src/domain/units.rs |
| R-D-003 | Markup types (Review/Shapes/Paths/Measure) | DONE | src/domain/markup.rs (22 kinds) |
| R-D-004 | Measurement computation (length/perimeter/area/angle/diam/radius/count) | DONE | markup.rs recompute_measurement |
| R-D-005 | Hit testing with tolerance | DONE | markup.rs hit_test + tests |
| R-D-006 | Document/Page model | DONE | src/domain/document.rs |
| R-D-007 | Project + ToolSet + Settings + History (undo/redo) | DONE | src/domain/project.rs |
| R-D-008 | JSON serialization roundtrip | DONE | serde derive; tests/domain_tests.rs |
| R-D-009 | Review status + comments | DONE | markup.rs + project.rs |

## UI requirements (Slice 1)
| ID | Requirement | Status | Trace |
|---|-----------|--------|------|
| R-U-001 | eframe app with panels (top/left/right/bottom/center) | DONE | src/ui/app.rs |
| R-U-002 | Tool palette (groups + Select) | DONE | src/ui/toolbar.rs |
| R-U-003 | Canvas (page, markups, selection, basic draw) | DONE | src/ui/canvas.rs |
| R-U-004 | Inspector (props, comments, measurements) | DONE | src/ui/inspector.rs |
| R-U-005 | Keyboard shortcuts (Ctrl+S/Z/Shift+Z/Del/Esc) | DONE | app.rs update loop |
| R-U-006 | Selection + move/translate | DONE | canvas interaction |
| R-U-007 | App icon + viewport title | DONE | main.rs + icons.rs |

## Persistence
| ID | Requirement | Status | Trace |
|---|-----------|--------|------|
| R-P-001 | Save/load .rubric.json | DONE | src/data/store.rs |
| R-P-002 | Pretty JSON, human-readable | DONE | serde_json::to_string_pretty |
| R-P-003 | App data dir resolution | DONE | directories crate |

## Feature requirements
| ID | Feature | Target in Rubric | Status | Notes |
|---|--------|----------------|--------|-------|
| F-001 | Markups (highlights, lines, shapes, text, callouts, stamps) | MarkupKind + Geometry | DONE (core) | expand tool set over time |
| F-002 | Measurements (length/perimeter/area/angle/count) | Measurement + recompute | DONE | add diameter/radius UI polish |
| F-003 | Review workflow (statuses, comments, subjects) | ReviewStatus, Comment | DONE | expand status set if needed |
| F-004 | Layers/Toolsets | ToolSet + Layer field | PARTIAL | expand visibility/lock/sorting |
| F-005 | Scale/calibration | Page.scale + units | DONE | UI editor needed |
| F-006 | Page navigation/thumbnails | Document.pages | TODO | add UI |
| F-007 | Non-destructive markup (sidecar) | .rubric.json separate | DONE | PDF embedding later optional |

## Gaps / exclusions
- PDF rendering/import/export — TODO (Phase 5)
- Multi-page UI — TODO
- Advanced drawing (polygon/polyline/angle/ink with preview) — TODO
- Zoom/pan with mouse wheel + space drag — TODO
- Cloud/ink rendering polish — TODO
- Packaging (AppImage/DMG/MSI) — TODO

## Verification
- All domain unit tests pass: `cargo test --lib`
- Integration tests pass: `cargo test --tests`
- Clippy clean: `cargo clippy -- -D warnings`
- Formatted: `cargo fmt --all -- --check`
- Builds: `cargo build` (debug/release)
