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
| R-D-003 | Markup types (Review/Shapes/Paths/Measure) | DONE | src/domain/markup.rs (23 kinds) |
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

## Phase 1 requirement set (spec-level)

Adopted 2026-10-19 from `docs/spec/README.md` (canonical feature inventory, F-1xx…F-7xx).
Each row maps a spec area to its feature IDs and the planned modules.

| ID | Requirement | Spec IDs | Planned modules | Status |
|---|-----------|----------|-----------------|--------|
| R-S-001 | Markup tools: highlight, text markups, note, text, callout, shapes, cloud, polyline/polygon, line/arrow, ink, stamps | F-101–F-111 | src/domain/markup.rs, src/ui/toolbar.rs, src/ui/canvas.rs | PARTIAL |
| R-S-002 | Drag-to-draw with live preview (bounds/segment/path/angle/ink) | F-112 | src/ui/canvas.rs (DrawState) | TODO |
| R-S-003 | Selection editing: select/move/delete, copy properties, z-order | F-113–F-115 | src/ui/canvas.rs, src/ui/inspector.rs, src/domain/project.rs | PARTIAL |
| R-S-004 | Layer visibility and lock | F-116 | src/domain/project.rs (ToolSet → Layer), src/ui/inspector.rs | TODO |
| R-S-005 | Measurements: length, perimeter, area, angle, diameter, radius, count | F-201–F-206 | src/domain/units.rs, src/domain/markup.rs | PARTIAL |
| R-S-006 | Calibration and units: scale presets, custom ratio, unit selection | F-207, F-208 | src/domain/units.rs, src/domain/document.rs | PARTIAL |
| R-S-007 | Measurement labels on canvas + precision/format preferences | F-209 | src/ui/canvas.rs, src/ui/inspector.rs | TODO |
| R-S-008 | Review workflow: statuses (Open/In review/Approved/Rejected), comments, author/subject | F-301–F-303 | src/domain/markup.rs, src/domain/project.rs | PARTIAL |
| R-S-009 | Summary report (markups by status/author/subject, exportable) | F-304 | src/domain/report.rs (new), src/ui/app.rs | TODO |
| R-S-010 | Status colours and legend | F-305 | src/ui/theme.rs, src/ui/toolbar.rs | TODO |
| R-S-011 | Canvas navigation: zoom, pan, page navigation, thumbnails | F-401–F-404 | src/ui/app.rs, src/ui/canvas.rs | PARTIAL |
| R-S-012 | Grid, snap, rulers | F-405, F-406 | src/ui/canvas.rs | TODO |
| R-S-013 | Persistence: `.rubric.json` sidecar, autosave, recent files, native dialogs, schema version | F-501–F-503, F-506, F-507 | src/data/store.rs, src/ui/app.rs | PARTIAL |
| R-S-014 | PDF import and page rendering | F-504 | src/data/pdf.rs (new), src/ui/canvas.rs | TODO |
| R-S-015 | Export (rendered image) and print | F-505 | src/data/export.rs (new), src/ui/app.rs | TODO |
| R-S-016 | UX polish: shortcuts, dark theme, context menus, undo/redo, accessibility | F-601–F-605 | src/ui/app.rs, src/ui/theme.rs, src/domain/project.rs | PARTIAL |
| R-S-017 | Packaging: Linux AppImage, macOS DMG, Windows MSI, CI release artifacts | F-701–F-704 | packaging files, .github/workflows/ci.yml | TODO |

## Feature requirements

Legacy IDs F-00x predate the spec; group rows F-1xx…F-7xx reference `docs/spec/README.md` (2026-10-19).

| ID | Feature | Target in Rubric | Status | Notes |
|---|--------|----------------|--------|-------|
| F-001 | Markups (highlights, lines, shapes, text, callouts, stamps) | MarkupKind + Geometry | DONE (core) | expand tool set over time |
| F-002 | Measurements (length/perimeter/area/angle/count) | Measurement + recompute | DONE | add diameter/radius UI polish |
| F-003 | Review workflow (statuses, comments, subjects) | ReviewStatus, Comment | DONE | expand status set if needed |
| F-004 | Layers/Toolsets | ToolSet + Layer field | PARTIAL | expand visibility/lock/sorting |
| F-005 | Scale/calibration | Page.scale + units | DONE | UI editor needed |
| F-006 | Page navigation/thumbnails | Document.pages | TODO | add UI |
| F-007 | Non-destructive markup (sidecar) | .rubric.json separate | DONE | PDF embedding later optional |
| F-1xx | Markups/tools + selection/edit (spec) | MarkupKind, Geometry, ToolSet → Layer | PARTIAL | docs/spec/README.md F-101–F-116; R-S-001–R-S-004 |
| F-2xx | Measurements + scale/calibration (spec) | Measurement, Page.scale, units | PARTIAL | docs/spec/README.md F-201–F-209; R-S-005–R-S-007 |
| F-3xx | Review workflow (spec) | ReviewStatus, Comment, author/subject | PARTIAL | docs/spec/README.md F-301–F-305; R-S-008–R-S-010 |
| F-4xx | Navigation/pages (spec) | Document.pages, UiState camera | PARTIAL | docs/spec/README.md F-401–F-406; R-S-011–R-S-012 |
| F-5xx | Persistence/I/O (spec) | .rubric.json store, PDF import, export/print | PARTIAL | docs/spec/README.md F-501–F-507; R-S-013–R-S-015 |
| F-6xx | UI/UX polish (spec) | shortcuts, theme, history | PARTIAL | docs/spec/README.md F-601–F-605; R-S-016 |
| F-7xx | Packaging (spec) | CI release build, installers | TODO | docs/spec/README.md F-701–F-704; R-S-017 |

## Gaps / exclusions
- Canonical inventory: `docs/spec/README.md` (52 F-IDs across F-1xx…F-7xx), adopted 2026-10-19; see Phase 1 requirement set above.
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
