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
- [x] Define the product feature inventory for a PDF markup/measurement app
- [x] Map features → Rubric domain: markups, layers/toolsets, review workflow (statuses), measurements, stamps, callouts, clouds, ink, annotations, snapshots, etc.
- [x] Update REQUIREMENTS_TRACEABILITY.md with traceability (requirement → file/module/test)
- [x] Record gaps, assumptions, and scope exclusions in assumptions-and-questions.md + DECISIONS.md
- [x] Create docs/spec/README.md summarising the requirement set

## Phase 2 — Vertical slice 1: Core markup + canvas (interactive)
- [x] Domain types complete (MarkupKind 23 variants, Geometry variants, ReviewStatus, Measurement)
- [x] Undo/redo (History snapshots, bounded 128)
- [x] Selection + move/translate
- [x] Draw rect-based shapes (rect/ellipse/cloud/highlight/text/callout/stamp/note) — basic placement
- [x] Draw segment paths (line/arrow/measure length/diameter/radius)
- [x] Draw count marker
- [x] Canvas page rendering + zoom (pan not yet wired to drag except selection)
- [x] Inspector (properties, status, author/subject, comments, measurement display)
- [x] Improve drawing: drag-to-draw for bounds/segment with live preview (polyline/angle still click-based)
- [x] Polyline/Polygon/Perimeter/Area path drawing (multi-point)
- [x] Angle tool (three-point)
- [x] Ink (freehand) with stroke accumulation + commit
- [x] Hit testing improvements (edge/outline vs fill; cloud approximation; ink segments)
- [x] Zoom/pan with mouse wheel + Space/middle-drag

## Phase 3 — Measurements + scale UI
- [x] Scale per page, units_per_point, recompute on geometry change
- [x] Measurement computation (length/perimeter/area/angle/diameter/radius/count)
- [x] Scale editor in inspector/settings (ratio presets: 1:1, 1 in=1 ft, 1 in=10 ft, custom)
- [x] Units dropdown in inspector (active doc/page scale unit)
- [x] Measurement labels rendered on canvas
- [x] Precision/formatting preferences (Settings::precision 0–6, inspector DragValue)

## Phase 4 — Review workflow, layers, toolsets
- [x] ReviewStatus, Comments, Subject/Author
- [x] ToolSet presets (Review/Shapes/Measure)
- [x] Layer visibility/lock (Layer model: visibility/lock, layers panel, mutation gates)
- [x] Status colours + legend
- [x] Stamp library (5 presets: Approved/Rejected/Draft/Check/Cross, glyph rendering, preset picker)
- [x] Callout text editing in-canvas (also Text boxes; auto-open after placement)

## Phase 5 — Persistence + I/O + PDF (non-destructive)
- [x] JSON save/load (.rubric.json)
- [x] Autosave hook (stub exists)
- [x] Open file dialog (native, rfd) for .rubric.json (PDF filter arrives with import)
- [x] PDF import/rendering (pdfium-render runtime binding; sidecar kept, PDF read-only)
- [x] Export/print (page image PNG via CPU rasterizer + OS print command; report export txt/JSON)
- [x] Recent files (app-level config store: `~/.config/rubric/recent.json`)

## Phase 6 — Polish, packaging, CI
- [x] Basic UI panels, shortcuts (Ctrl+S/Z/Shift+Z/Esc/Del)
- [x] Per-tool keyboard shortcuts (F-601: unmodified letter/digit keys, guarded)
- [x] Context menus (F-603: shared canvas + inspector menu)
- [x] Grid, snap, rulers (F-405/F-406: View > Show Grid / Snap to Grid / Grid Spacing; page-point rulers)
- [x] Page thumbnails sidebar (F-404: PDF previews via the shared render cache, numbered placeholders, click to jump)
- [x] Dark theme (F-602: View > Theme light/dark, persisted preference)
- [x] Icon/desktop entry, AppImage/DMG/MSI (F-701–F-704: `.github/workflows/package.yml` + `release.yml`; WiX definition in `packaging/wix/`)
- [x] CI workflow skeleton (.github/workflows/ci.yml)
- [x] Security/community wave 2026-10-24 (guide Part II): security.yml
      (secrets / dependency-review / cargo-deny / SBOM / CodeQL),
      dependabot.yml, deny.toml, branch protection + private vulnerability
      reporting, conduct/security/a11y/vision/governance/contributors docs,
      issue templates, a11y PR checklist, pinned action SHAs
- [ ] Benchmarks/tests expansion
- [x] Changelog updates per slice
- [x] PDFium runtime documented for users (docs/pdfium.md: discovery order, download, verification)
- [ ] Follow-ups recorded as questions: installer code-signing (Q-PACK-1), raster icon set (Q-PACK-2)

## Slice strategy
Build vertically: domain + tests → minimal UI → drawing + selection → save/load → measurements → review → polish. Keep each slice small, compiling, tested.
