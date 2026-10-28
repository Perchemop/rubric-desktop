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

## 2026-10-23: Packaging splits release from installers, and stays CI-only
- **Decision**: Packaging is two workflows with distinct jobs. `release.yml` builds the release binary per target and attaches archives to a GitHub release on `v*` tags; `package.yml` builds native installers (AppImage, DMG, MSI). The Windows installer's product definition is kept in-repo (`packaging/wix/Product.wxs`) rather than generated by an opaque script. PDFium — a run-time dependency — is bundled next to the executable on all three platforms when found, and its discovery/verification contract is documented in `docs/pdfium.md`. Installers ship **unsigned** (no signing secret in the repo) and icons are the repo's SVG (no raster set yet), both recorded as Q-PACK-1/Q-PACK-2 instead of silently omitted.
- **Rationale**: the quality gate (`ci.yml`) must stay the single slow path — re-running the whole test suite inside release builds doubles CI time for no new information, since the release build is the same commit. Keeping Product.wxs in the repo means the installer's declared components (executable, licence, PDFium DLL, start-menu shortcut only) are reviewable; bundling PDFium is justified because importing PDFs is the app's primary purpose, and shipping a binary that fails at runtime would violate "no fake functionality".
- **Impact**: new `.github/workflows/release.yml`, `.github/workflows/package.yml`, `packaging/wix/{Product.wxs,License.rtf,Rubric.Setup.csproj}`, `docs/pdfium.md`; new questions Q-PACK-1..3 in `docs/assumptions-and-questions.md`; spec F-701–F-704 and R-S-017 marked done. `ci.yml` unchanged.

## 2026-10-24: Suite-ready restructure — workspace, shared crash-report and update-check crates
- **Decision**: Adopted guide v4's suite layout (§7.3): the repository is now a Cargo workspace with `apps/rubric` (the desktop app, package name unchanged) and two shared crates, `crates/crash-report` and `crates/update-check`, which the app consumes via path dependencies. Crash reporting installs a panic hook that parks a scrubbed report (version, OS, backtrace with home paths and token-like material redacted) in the app's config dir and offers a review-and-cancel dialog; submission uses a pre-filled GitHub issue URL by default, or a user-supplied token held only in memory. Update checking queries GitHub Releases on explicit user action only, compares versions semantically, fails soft offline, and caches for an hour.
- **Rationale**: guide v4 makes crash reporting and update checks part of the v0.1.0 hard gate (§6.1) and requires the shared-crate layout for suite apps (§7, §13). Extracting them as crates now means any future suite app inherits consistent behaviour instead of duplicating it. `ureq` (sync, MIT/Apache-2.0) was chosen over `reqwest` because the app has its own event loop and one-shot HTTP does not justify an async runtime.
- **Impact**: new `crates/crash-report` (12 tests), `crates/update-check` (13 tests); `apps/rubric/src/main.rs` installs the hook; Help menu gains "Check for Updates…" and "Report a Bug…"; `Cargo.toml` at root is now workspace-only; `src/` and `tests/` moved to `apps/rubric/` (icons path adjusted); README gains a privacy section; CI commands unchanged (`cargo fmt/test/clippy --all`).

## 2026-10-23: Page switching shares one helper, thumbnails reuse the page cache
- **Decision**: All three page switches — the top-bar arrows, Page Up/Down, and the new pages sidebar (F-404) — now go through `RubricApp::jump_to_page`, which commits an in-flight text edit, pushes a snapshot only when text actually changed, abandons the draft and sets the page. Thumbnails are rasterised through the existing `data::pdf::render_page_to_cache` at a fixed 192 px width, keyed per page, so a page the main canvas already rendered is a cache hit; a render failure poisons that key and falls back to a numbered placeholder.
- **Rationale**: three separate implementations of the same asynchronous side effects (text-edit commit, snapshot, draft abandonment) is exactly how page-jumping loses an edit; sharing one helper removes that class of bug. Reusing the render cache keeps thumbnail display cheap, and the placeholder fallback avoids an unreadable broken-image box.
- **Impact**: new `src/ui/pages.rs` and `src/domain/thumbs.rs` (pure layout, 6 tests); `src/ui/app.rs` gains `jump_to_page`, `thumb_cache` and the Pages panel; Page Up/Down and the top-bar arrows now call the helper instead of inlining the logic.

## 2026-10-23: Grid is canvas state, not project data
- **Decision**: The canvas grid (`Grid` in `src/domain/grid.rs`: spacing, visibility, snap) and the rulers are session/UI state — never persisted into `.rubric.json`. Snap-to-grid applies to markup *creation anchors* only; hit-testing keeps using the exact pointer position so a markup that is not grid-aligned stays selectable after the grid is switched on. The grid is not drawn when its screen pitch drops below 6 px, and ruler ticks come from a 1/2/5 ladder (`nice_step`) scaled from the zoom.
- **Rationale**: grid and rulers describe how the canvas behaves, not what the document contains (the same machine-local rule as the theme preference); snapping creation but not picking keeps existing documents fully editable; skipping an unresolvable pitch prevents an unreadable moiré wash, and nice ticks keep labels readable at every zoom.
- **Impact**: new `src/domain/grid.rs` (10 unit tests), `src/ui/canvas.rs` (`draw_grid`, `draw_rulers`, snapped creation vs exact hit-testing), `src/ui/app.rs` (`UiState::grid`, View > Show Grid / Snap to Grid / Grid Spacing). No change to file formats, project data or existing commands.

## 2026-10-26: Agents merge completed PRs; Markups List is a bottom panel
- **Decision**: The owner directive of 2026-10-26 supersedes the earlier human-merge rule: an agent merges a PR once CI is green and review findings are addressed. The owner can still ask to review a PR first. Separately, the v0.2 Markups List lives in a resizable **bottom panel**, not a new inspector mode: it is a review console that needs width for columns, and the existing inspector already scrolls selection properties, comments, measurements and settings.
- **Rationale**: waiting for a manual merge click adds no verification once the same CI gates and reviewer pass have run, while the owner retains an explicit hold option. A bottom panel preserves the canvas/inspector split and avoids a sixth dead `InspectorMode` variant; the panel is a thin view over `MarkupList`, and row clicks return a `MarkupListAction` so `RubricApp` keeps selection/page/snapshot policy in one place.
- **Impact**: `AGENTS.md` merge wording; `apps/rubric/src/ui/app.rs` (`show_markups_list`, filter/sort/group state, View toggle, action handling); new `apps/rubric/src/ui/markups_list.rs`; `PROGRESS.md`, `CHANGELOG.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `REQUIREMENTS_TRACEABILITY.md`, `docs/parity.md`. No project-file format change.

## 2026-10-26: Stirling PDF added as capability-level inspiration (client-directed)
- **Decision**: Stirling PDF (https://github.com/Stirling-Tools/Stirling-PDF) is added to `docs/inspiration.md` and analysed in `docs/research/stirling-pdf-capabilities.md`, capability-level only. It is explicitly **not** a dependency: wrong stack (Java server platform), and its compare/redaction/signature/page-organisation capabilities are recorded as `watch` for future milestones (compare feeds the v0.2 diff work; the others would need PDF writing, which the read-only-PDF decision currently excludes).
- **Rationale**: the client requested the addition directly (that is the §8.4 human gate). The core is MIT but the repository is open-core with proprietary directories, so the clean-room note in the research file restricts consultation to the public capability surface; no code, UI text, screenshots or layouts are referenced.
- **Impact**: `docs/inspiration.md` (table row), `docs/research/stirling-pdf-capabilities.md` (new), `docs/repo-radar.md` (inspiration-addendum pass), no application code.

## 2026-10-26: Copy Properties copies visual style only; z-order is page-scoped
- **Decision**: F-114's Copy/Paste Properties clipboard holds a `MarkupStyle`, not a full `StylePreset`. Paste changes only the visual style (and `modified`) of selected editable markups; author, subject, review status, lock state, comments, geometry and measurements are untouched, and locked targets are skipped. F-115 z-order operates on the selected markup's page: one-step forward/backward swaps with the nearest same-page neighbour, while front/back move it to the page's paint-order edge; other pages keep their relative order.
- **Rationale**: "copy properties" in review work usually means "make this look like that", but silently copying review status or lock state would let a paste overwrite audit data. Keeping the clipboard visual-only resolves the v0.2-01 deferral without a gate; a future preset-library apply that changes status/lock must present that change explicitly. Page-scoped z-order matches what the canvas paints: moving one markup past a markup on another page has no visible effect and would create confusing cross-page history noise.
- **Impact**: `src/domain/markup.rs` (`Markup::apply_style`), `src/domain/document.rs` (`apply_style_to`, four z-order methods), `src/domain/action.rs` (new actions and enablement), `src/ui/menu.rs` (shared selection commands), `src/ui/app.rs` (`style_clipboard`, executor); `docs/tasks/v0.2-03-editing-commands.md`. No file-format change.

## 2026-10-26: Autosave beside the project; newer schema files are refused on load
- **Decision**: Autosave is off until enabled in `File → Auto-save`, runs only while the project is dirty and at most once per interval (clamped to ≥ 5 s), and writes `<name>.autosave.rubric.json` beside the open project — falling back to the app data directory for an untitled project. The autosave keeps the canonical `.rubric.json` extension so it is a normal file File → Open can read. Separately, every project is stamped with `schema_version` (current 1); a file without the field is version 1, and `load_project` refuses `0` or any newer version with `DataError::UnsupportedSchema` instead of loading it as the current schema.
- **Rationale**: a sibling file is discoverable and travels with the project, while the app-data fallback keeps untitled sessions covered; the canonical extension makes recovery a normal open rather than a bespoke import. Refusing an unknown schema up front prevents silently dropping fields a newer build wrote; a migration seam stays a one-line addition when a version 2 exists. There is deliberately no on-open recovery prompt in this slice — the recovery file is reported in the status bar and can be opened manually.
- **Impact**: `src/domain/project.rs` (`CURRENT_SCHEMA_VERSION`, `Project.schema_version`, interval presets/labels), `src/data/store.rs` (`check_schema_version`, `autosave_path`, `autosave`, `autosave_due`), `src/data/error.rs` (`UnsupportedSchema`), `src/ui/app.rs` (unsaved flag, autosave loop, File → Auto-save), `docs/tasks/v0.2-04-persistence-hardening.md`. `.rubric.json` gains one field.

## 2026-10-26: Redo shortcut is Ctrl+Y (user directive)
- **Decision**: The user asked for redo to be **Ctrl+Y**. `Ctrl+Y` now redoes, `Ctrl+Z` still undoes, and `Ctrl+Shift+Z` is no longer handled (it must not silently undo). The text-edit guard still refuses history changes while an in-canvas text edit is open. The change was implemented in `apply_editor_action`'s keyboard handling plus the `logic()` input branch, and the hint bar, README shortcut table, spec F-601 and agent guidance were made consistent.
- **Rationale**: the directive is explicit and the shortcut is a user-facing contract; leaving `Shift+Z` bound to redo would contradict the requested binding, while leaving it bound to undo would be a surprising side effect of the modifier. `Ctrl+Z` remains undo because that binding was not changed.
- **Impact**: `apps/rubric/src/ui/app.rs` (hint + both undo/redo key paths), `README.md`, `docs/spec/README.md` (F-601), `CHANGELOG.md`, `AGENTS.md`, `.opencode/agents/ui-dev.md`, `IMPLEMENTATION_PLAN.md`. No file-format change.

## 2026-10-27: Personal-data and secret scan gate (owner directive; guide II.29)
- **Decision**: Every change is scanned for personal data and secrets before commit; the scan is a **required** pull-request check and runs again over release assets before they are attached. `scripts/scan-personal-data.sh` fails on a non-CI absolute home path (`/home/<user>`, `C:\Users\<user>`; `/home/runner` allowed) or a non-synthetic email (`@example.*`, GitHub noreply allowed); `scripts/scan-release-assets.sh` extracts each archive and applies the home-path scan, including binaries. The check is `Personal data scan` in `.ci-required.txt`. Real-looking values are a defect: placeholders/fixtures stay obviously synthetic (`dev@example.com`, `Jane Doe`).
- **Rationale**: the owner directive of 2026-10-27 makes this mandatory and treats a missed scan as a failed slice (guide II.29). A real person's name, address, email or account name in a public repository is a published defect that survives in history, forks and cached views, so the gate must block merges rather than warn. The scan is dependency-free (git + grep) so the privacy gate adds no supply-chain surface.
- **Impact**: `scripts/scan-personal-data.sh`, `scripts/scan-release-assets.sh` (new); `.github/workflows/security.yml` (`Personal data scan` job), `.github/workflows/release.yml` (asset scan in `publish`), `.ci-required.txt`; `docs/qc/checklist.md`, `docs/guide-compliance.md`, `PROGRESS.md`. A previously committed maintainer home path was removed from the tree in PR #37 (`58e57d6`); history purge is a separate owner-approved action (gap G7).

## 2026-10-28: Release artefact architecture spelling kept as `arm64` for now
- **Decision**: Rubric's release assets keep `rubric-linux-arm64` / `rubric-macos-arm64` instead of the guide's `rubric-<app>-<os>-<arch>` `aarch64` spelling (guide II.28.3 asks for one spelling across the suite and records the deviation). The artefact names are produced by `.github/workflows/release.yml` and are not parsed by any self-update code in Rubric today, so the rename is safe to do at the next release that touches the names.
- **Rationale**: `aarch64` is the target-triple spelling and the guide's preference, but changing artefact names is a release-naming change with published download URLs attached; doing it silently mid-slice would break existing links. Recording it as a deviation (rather than leaving it inconsistent) satisfies II.28.3's "record and decide".
- **Impact**: recorded here and in `docs/guide-compliance.md`; no code change. The next release-name change switches to `aarch64` and updates the README/release links.
