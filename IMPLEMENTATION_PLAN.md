# Implementation Plan

## Phase 0 — Environment, brand, repo, skeleton
- [x] Invent original name: Rubric
- [x] Create fresh repo at ~/rubric
- [x] .gitignore excludes external material, secrets, target
- [x] Branding: logo.svg (512x512), icon PNGs 16/32/48/64/128/256/512/1024
- [x] assets/branding.md, README.md, AGENTS.md
- [x] LICENSE (MIT)
- [x] rust-toolchain.toml
- [x] Cargo.toml (eframe/egui/serde/uuid/thiserror/tracing/chrono/image)
- [x] docs/architecture.md, docs/data-model.md, docs/test-plan.md, docs/assumptions-and-questions.md
- [x] IMPLEMENTATION_PLAN.md, REQUIREMENTS_TRACEABILITY.md, DECISIONS.md, PROGRESS.md, CHANGELOG.md
- [x] Domain model complete (geom, units, style, markup, document, project) with tests
- [x] Data layer (store, error, JSON persistence)
- [x] UI skeleton (app, canvas, toolbar, inspector, theme, icons)
- [x] cargo clippy/fmt configured
- [x] Local git init + commit

## Phase 1 — Feature definition and requirements
- [ ] Define the product feature inventory for a PDF markup/measurement app
- [ ] Map features → Rubric domain: markups, layers/toolsets, review workflow (statuses), measurements, stamps, callouts, clouds, ink, annotations, snapshots, etc.
- [ ] Update REQUIREMENTS_TRACEABILITY.md with traceability (requirement → file/module/test)
- [ ] Record gaps, assumptions, and scope exclusions in assumptions-and-questions.md + DECISIONS.md
- [ ] Create docs/spec/README.md summarising the requirement set

## Phase 2 — Vertical slice 1: Core markup + canvas (interactive)
- [x] Domain types complete (MarkupKind 22 variants, Geometry variants, ReviewStatus, Measurement)
- [x] Undo/redo (History snapshots, bounded 128)
- [x] Selection + move/translate
- [x] Draw rect-based shapes (rect/ellipse/cloud/highlight/text/callout/stamp/note) — basic placement
- [x] Draw segment paths (line/arrow/measure length/diameter/radius)
- [x] Draw count marker
- [x] Canvas page rendering + zoom (pan not yet wired to drag except selection)
- [x] Inspector (properties, status, author/subject, comments, measurement display)
- [ ] Improve drawing: drag-to-draw for bounds/segment/path/angle/ink with live preview
- [ ] Polyline/Polygon/Perimeter/Area path drawing (multi-point)
- [ ] Angle tool (three-point)
- [ ] Ink (freehand) with stroke accumulation + commit
- [ ] Hit testing improvements (edge/outline vs fill; cloud/ink)
- [ ] Zoom/pan with mouse wheel + Space/middle-drag

## Phase 3 — Measurements + scale UI
- [x] Scale per page, units_per_point, recompute on geometry change
- [x] Measurement computation (length/perimeter/area/angle/diameter/radius/count)
- [ ] Scale editor in inspector/settings (ratio presets: 1:1, 1 in=1 ft, 1 in=10 ft, custom)
- [ ] Units dropdown in inspector (active doc/page scale unit)
- [ ] Measurement labels rendered on canvas
- [ ] Precision/formatting preferences

## Phase 4 — Review workflow, layers, toolsets
- [x] ReviewStatus, Comments, Subject/Author
- [x] ToolSet presets (Review/Shapes/Measure)
- [ ] Layer visibility/lock (expand ToolSet to Layer)
- [ ] Status colours + legend
- [ ] Stamp library (basic presets)
- [ ] Callout text editing in-canvas

## Phase 5 — Persistence + I/O + PDF (non-destructive)
- [x] JSON save/load (.rubric.json)
- [x] Autosave hook (stub exists)
- [ ] Open file dialog (native) for .rubric.json and PDF
- [ ] PDF import/rendering (lopdf or pdfium-render) — slice later; keep sidecar
- [ ] Export/print
- [ ] Recent files

## Phase 6 — Polish, packaging, CI
- [x] Basic UI panels, shortcuts (Ctrl+S/Z/Shift+Z/Esc/Del)
- [ ] Keyboard/tool shortcuts, context menus
- [ ] Grid, snap, rulers
- [ ] Dark theme
- [ ] Icon/desktop entry, AppImage/DMG/MSI
- [x] CI workflow skeleton (.github/workflows/ci.yml)
- [ ] Benchmarks/tests expansion
- [ ] Changelog updates per slice

## Slice strategy
Build vertically: domain + tests → minimal UI → drawing + selection → save/load → measurements → review → polish. Keep each slice small, compiling, tested.
