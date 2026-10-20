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

## 2026-10-19: ReviewStatus remapped to the spec status set
- **Decision**: `ReviewStatus` remapped to the spec F-301 set — `Open` (default) / `In review` / `Approved` / `Rejected` (serde snake_case), superseding the earlier 6-variant enum (None/Accepted/Rejected/Completed/Cancelled/Alert). Each status gets a distinct palette colour (`ReviewStatus::color()`) surfaced via an inspector legend, status-combo swatches, and canvas status dots.
- **Rationale**: the spec status set adopted in Phase 1 is canonical; legacy status strings are rejected by design — no released files existed, so the rename was free (scope decision).
- **Impact**: src/domain/markup.rs (variant set, colour mapping, 6 new tests incl. legacy-string rejection), src/ui/inspector.rs (Status legend), src/ui/canvas.rs (status dots); `.rubric.json` files containing old status strings no longer load.

## 2026-10-20: Native dialogs via rfd; recents are app-level, not project data
- **Decision**: native file dialogs via the `rfd` crate (0.15, MIT/Apache-2.0; pure-Rust xdg-portal backend on Linux — no GTK system dependency). Recent-files tracking moved out of `Project.settings` into an app-level store: `RecentList` domain type (dedupe/move-to-front/cap at 10) persisted by `data::recent` to the OS config dir (`~/.config/rubric/recent.json` on Linux), load-failing soft to an empty list.
- **Rationale**: keeping recents inside `Settings` would write the user's absolute local paths into shareable `.rubric.json` files and lose the list between sessions anyway; app-level config keeps review documents clean and makes recents global across projects. rfd over GTK dialogs keeps the dependency tree pure Rust.
- **Impact**: Cargo.toml (`rfd`), src/domain/project.rs (`RecentList`), src/data/recent.rs (new), src/ui/app.rs (File menu: Open Ctrl+O / Save Ctrl+S / Save As / Recent submenu, title bar, Quit via eframe teardown). Runtime caveat: on Linux the portal backend needs a running `xdg-desktop-portal` service; minimal window-manager setups may need one installed (recorded in docs/assumptions-and-questions.md).
