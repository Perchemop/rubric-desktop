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

## 2026-10-21: Inspiration policy + guide-conformant docs (human directive)
- **Decision**: (a) Other products **may** be referenced as *feature inspiration*, but only at capability level. (b) Completed the guide's required doc set: `docs/project-brief.md`, `docs/scope.md`, `docs/product-name.md`, `docs/inspiration.md` (capability-level inspiration), `ATTRIBUTION.md` (crate + PDFium licences), `docs/development.md` (CI-first workflow), `docs/qc/checklist.md` (phase/release gates).
- **Rationale**: the guide mandates discovery/scope/inspiration/attribution/QC docs; the human allowed capability-level inspiration references.
- **Impact**: AGENTS.md (purpose + Do-not wording), README.md (originality paragraph citing `docs/inspiration.md` + `ATTRIBUTION.md`), `.opencode/agents/docs-writer.md` (agent rule), new docs listed above; scope unchanged for code.

## 2026-10-23: Theme is an app-level preference
- **Decision**: The colour theme is application-level state, not project data — persisted via the `AppPrefs` domain type (`src/domain/theme.rs`) to the platform config dir as `prefs.json` (`src/data/prefs.rs`), mirroring the 2026-10-20 recents decision that machine-local state never goes into shareable `.rubric.json` project files; loading soft-fails to the Light default so a missing/corrupt file never blocks startup.
- **Rationale**: theme is a per-user/per-machine choice; `.rubric.json` files stay shareable and machine-agnostic.
- **Impact**: `src/domain/theme.rs`, `src/data/prefs.rs`, `src/ui/theme.rs`, `src/ui/app.rs` (`set_theme`).

## 2026-10-23: Context-menu commands run through one executor
- **Decision**: Context menus are modelled by the domain enum `EditorAction` (`src/domain/action.rs`) and rendered by a single shared builder (`src/ui/menu.rs`), attached to both the canvas page response and the inspector panel response. The menu only *requests* an action (`UiState::pending_action`, or sets `active_tool` for the tool submenu); `RubricApp::apply_editor_action` performs it and owns history snapshots and status text — the same method the Edit/View menu bar now calls, so each command has exactly one implementation.
- **Rationale**: keeps UI rendering and command execution separate (no UI/business-logic mixing), lets the canvas and inspector share one menu without duplication, and makes the command set and its applicability headlessly testable. The canvas swallows the context menu while a multi-point draft is in progress so a right-click keeps its existing "commit the draft" meaning.
- **Impact**: new `src/domain/action.rs` and `src/ui/menu.rs`; `src/ui/app.rs` (`UiState::pending_action`, `apply_editor_action`, Edit/View menu bar reuse), `src/ui/canvas.rs`. Existing menu-bar command behaviour is unchanged.

## 2026-10-23: Adopt the guide §5 CTO.new-style free process
- **Decision**: Structure all work as the cycle `Chat (plan) → Task (execute) → Pull Request (review) → Merge` (guide §5). The Orchestrator is the Chat/planning role (writes Task briefs, never edits code); the Coder executes one bounded Task; GitHub Actions is the free "Task Runner"; every change lands as a Pull Request that the **human** reviews and merges.
- **Rationale**: the updated build guide emulates cto.new using only free models and GitHub Actions; keeping the plan/execute/review phases distinct, and requiring human review before merge, is the guide's explicit "trust but verify" quality gate (guide §9).
- **Impact**: `AGENTS.md` (operating model + CI-first wording + no-self-merge rule), `docs/development.md` (Task workflow + DoD), `docs/qc/checklist.md` (PR + human-merge items), `.opencode/agents/orchestrator.md` (Chat framing) and new `.opencode/agents/coder.md`; added `docs/parity.md`. No application code changes.

## 2026-10-23: Page switching shares one helper, thumbnails reuse the page cache
- **Decision**: All three page switches — the top-bar arrows, Page Up/Down, and the new pages sidebar (F-404) — now go through `RubricApp::jump_to_page`, which commits an in-flight text edit, pushes a snapshot only when text actually changed, abandons the draft and sets the page. Thumbnails are rasterised through the existing `data::pdf::render_page_to_cache` at a fixed 192 px width, keyed per page, so a page the main canvas already rendered is a cache hit; a render failure poisons that key and falls back to a numbered placeholder.
- **Rationale**: three separate implementations of the same asynchronous side effects (text-edit commit, snapshot, draft abandonment) is exactly how page-jumping loses an edit; sharing one helper removes that class of bug. Reusing the render cache keeps thumbnail display cheap, and the placeholder fallback avoids an unreadable broken-image box.
- **Impact**: new `src/ui/pages.rs` and `src/domain/thumbs.rs` (pure layout, 6 tests); `src/ui/app.rs` gains `jump_to_page`, `thumb_cache` and the Pages panel; Page Up/Down and the top-bar arrows now call the helper instead of inlining the logic.

## 2026-10-23: Grid is canvas state, not project data
- **Decision**: The canvas grid (`Grid` in `src/domain/grid.rs`: spacing, visibility, snap) and the rulers are session/UI state — never persisted into `.rubric.json`. Snap-to-grid applies to markup *creation anchors* only; hit-testing keeps using the exact pointer position so a markup that is not grid-aligned stays selectable after the grid is switched on. The grid is not drawn when its screen pitch drops below 6 px, and ruler ticks come from a 1/2/5 ladder (`nice_step`) scaled from the zoom.
- **Rationale**: grid and rulers describe how the canvas behaves, not what the document contains (the same machine-local rule as the theme preference); snapping creation but not picking keeps existing documents fully editable; skipping an unresolvable pitch prevents an unreadable moiré wash, and nice ticks keep labels readable at every zoom.
- **Impact**: new `src/domain/grid.rs` (10 unit tests), `src/ui/canvas.rs` (`draw_grid`, `draw_rulers`, snapped creation vs exact hit-testing), `src/ui/app.rs` (`UiState::grid`, View > Show Grid / Snap to Grid / Grid Spacing). No change to file formats, project data or existing commands.
