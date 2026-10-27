# Changelog

All notable changes to this project will be documented here. Format: semantic-ish for this repo (Added/Changed/Fixed/Docs).

## [Unreleased]

### Added
- v0.2-01 domain foundation for reviewer workflows: `MarkupList` snapshot
  (case-insensitive filters incl. comment/summary search, stable sorts,
  grouping with numeric page order), `StylePreset`/`StylePresetLibrary`
  (capture/apply, 64-entry cap, upsert dedupe) and `DiffResult`
  (`diff_documents`/`diff_projects`, 2-dp geometry fingerprints,
  style/locked-aware change detection). All serde-serialisable and
  headless-tested, plus integration coverage in `tests/domain_tests.rs`.
- Markups List UI (R-V02-01): resizable bottom panel built on the `MarkupList`
  domain model, with search and status/author/subject/layer filters, every
  sort and group mode, row selection that jumps to the markup's page, empty
  states, Clear filters, and a View → Show Markups List toggle.
- Selection editing completion (F-114/F-115): Copy/Paste Properties apply only
  the visual style to selected editable markups (one undo snapshot, locked
  targets skipped), and page-scoped Bring Forward/Backward/To Front/To Back
  reorder markups without crossing page boundaries. Commands are available in
  the shared canvas/inspector context menu and the Edit menu.
- Autosave wiring (F-502): `File → Auto-save` enables it and picks an interval;
  a dirty project is written at most once per interval to
  `<name>.autosave.rubric.json` beside the project (app data dir when
  untitled), and the status bar reports the path or the failure.
- `docs/guide-compliance.md`: rule-by-rule review of the local `guide.md` and
  `addendum.md`, with evidence, the deliberately-adapted rules, and six tracked
  gaps.
- `.ci-required.txt`: the authoritative CI checks that decide `ci.sh`'s verdict
  (guide II.20.4 / II.25).
- Project schema versioning (F-507): every `.rubric.json` is stamped with
  `schema_version`; legacy files without it load as the current version, and
  `load_project` refuses a newer/unknown version instead of mis-reading it.
- Stirling PDF added as capability-level inspiration (client-directed):
  original capability summary in `docs/research/stirling-pdf-capabilities.md`,
  inspiration table row and radar addendum; no code or dependency adopted.
- Community and governance files (operating guide Part II / opensource.guide
  alignment): `CODE_OF_CONDUCT.md`, `SECURITY.md` (private reporting, threat
  model, incident response), `ACCESSIBILITY.md`, `VISION.md`,
  `GOVERNANCE.md`, `CONTRIBUTORS.md`, and issue templates for bugs, feature
  requests and accessibility reports (`accessibility_report.yml`).
- CI security workflow `.github/workflows/security.yml`: secret-pattern
  scan (`scripts/scan-secrets.sh`), PR dependency review, `cargo-deny`
  licence/advisory gate (`deny.toml`), SBOM generation (SPDX + CycloneDX),
  and CodeQL SAST for the workspace.
- `.github/dependabot.yml`: weekly cargo + GitHub-Actions update PRs.
- Repository security posture: branch protection on `main` (required check,
  admin-enforced), private vulnerability reporting, Dependabot security
  updates, Discussions enabled, area/a11y labels.
- `docs/metrics.md` — which project metrics are tracked and why.

### Changed
- Redo is now **Ctrl+Y** (`Ctrl+Z` remains undo); the hint bar, README, and the
  F-601 shortcut record were updated.
- `README.md`: adds a "Try Rubric without Rust" release-download path, corrects
  the workspace layout, and documents the contributor build/test commands.
- `docs/qc/checklist.md`: the per-Task merge step now matches the 2026-10-26
  owner directive (agent merges once CI is green and findings are addressed).
- `docs/current-model-map.md`: refreshed with the live 2026-10-27 free-model
  catalog and the Context7/`gh` tooling status.
- `AGENTS.md`: owner directive 2026-10-26 — agents merge completed PRs once CI
  is green and review findings are addressed, rather than waiting for a manual
  merge click.
- `.github/PULL_REQUEST_TEMPLATE.md`: UI changes now must pass the
  accessibility checklist items.
- `CONTRIBUTING.md`: links the conduct/security/accessibility paths.
- Global workflows pin every GitHub Action to a commit SHA (supply-chain
  hardening, guide Part II §20.3).
- `ATTRIBUTION.md`: adds the CI-enforced dependency licence policy and the
  AI-tooling terms record; `docs/current-model-map.md` records the research
  method and model-terms rules.
- Crate `license` metadata unified on `MIT` to match `LICENSE`; dual
  "MIT OR Apache-2.0" adoption is open question Q-LIC-1.

### Fixed
- Markup diff no longer misses style-only or lock-only changes; `StylePreset::apply_to`
  refreshes `modified`; `diff_projects` treats a project without an active
  document as empty instead of reporting "no changes"; Page groups sort
  numerically (1, 2, 10); duplicate IDs in malformed documents are diffed once.
- `release.yml`: the publish step finalises a draft release after uploading
  assets (a re-created tag turns the existing Release entry into a draft,
  which had left v0.1.0 unpublished until edited).
- Dependency hygiene findings from the new `cargo-deny` gate: yanked
  `clipboard-win` 5.4.1 → 5.4.3 in `Cargo.lock`; explicit `version` on the
  two workspace path dependencies (cargo-deny treats versionless specs as
  wildcards); `epaint_default_fonts` font licences (OFL-1.1,
  Ubuntu-font-1.0) documented in `deny.toml` and `ATTRIBUTION.md`.

## [0.1.0] - 2026-10-24

### Summary

First functional, installable build of **Rubric** — a free, cross-platform PDF
markup, review and measurement tool. The full reviewed scope (52 feature IDs,
17 requirement groups) is implemented and CI-verified; what remains is polish
and packaging follow-ups, listed honestly below. Built and verified on Linux
(aarch64 and x86_64), with macOS and Windows artefacts produced by CI.

### Roadmap progress

- Requirement groups: 13/17 fully done, 4 partial (small residuals), 0 not started.
- Feature IDs: 46/52 done (88 %).
- Evidence: `REQUIREMENTS_TRACEABILITY.md`, CI runs, and hands-on verification
  of the installed binary on the primary platform (Linux aarch64).
- Details and next milestones: `ROADMAP.md`.

### Added
- Original name/brand "Rubric": SVG logo, multi-size PNG icons (16/32/48/64/128/256/512/1024), branding spec.
- Fresh Rust crate `rubric` (egui/eframe 0.36, serde, uuid, thiserror, tracing, chrono, image).
- Domain-first architecture: geom, units (scale + conversions), style (colours/dashes/fill modes), markup (23 kinds, geometry variants, measurements, hit testing, review status, comments), document (pages/markups), project (toolsets, settings, history with undo/redo capped 128).
- Data layer: JSON persistence (`.rubric.json`), app data dir via directories, error types.
- UI skeleton: app shell (panels, menu, shortcuts), canvas (page, markups, selection, basic place/move), toolbar (tool groups), inspector (properties/comments/measurements), theme/icons.
- Tests: unit tests in domain modules + integration tests (6) covering geometry, measurements, serialisation, project lifecycle.
- Tooling: .gitignore (excludes secrets/target/local notes), LICENSE (MIT), rust-toolchain.toml, CI skeleton, scripts/gen-icons.sh.
- Docs: README, AGENTS, architecture, data model, test plan, assumptions/questions, implementation plan, requirements traceability, decisions, progress, changelog.


### Added
- Suite infrastructure (guide §7): shared crates `crates/crash-report` and `crates/update-check`, consumed by the app and reusable by any future app in the suite. **crash-report** installs a panic hook that captures the app version, OS and a scrubbed backtrace (home directories and token-like material redacted), shows a review-and-cancel dialog with editable notes, and then submits either through a pre-filled "New Issue" URL (no account in-app) or a **user-supplied** GitHub token held only in memory — a privileged bot token is never embedded. **update-check** queries the GitHub Releases API on demand (Help ▸ Check for Updates…), compares versions semantically (so 0.2.0 correctly beats 0.10.0), fails soft offline, caches for an hour and never polls in the background. 25 new tests across the two crates (12 + 13). The workspace was restructured to the guide's suite layout (`apps/rubric` + `crates/*`) with unchanged build commands and output paths. Privacy is documented in `README.md`.
- Process files required by guide v4 (suite-ready edition): `.github/PULL_REQUEST_TEMPLATE.md` (acceptance criteria, tests, CI, no-proprietary checklist), `ROADMAP.md` (staged milestones with evidence-based readiness: 13/17 requirement groups and 46/52 feature IDs done), `CONTRIBUTING.md` (PR licence terms, DCO sign-off), `docs/design/brand.md` (original mark and palette rationale), and `docs/research/` with the summaries-plus-URLs policy and its human approval gate.
- Packaging: CI release and native installers (spec F-701/F-703/F-704). `.github/workflows/release.yml` builds the release binary for Linux x86_64, macOS arm64 and Windows x86_64 and attaches each archive to a GitHub release on `v*` tags (a manual `workflow_dispatch` publishes a pre-release, so a test run never looks like a real release). `.github/workflows/package.yml` builds the native installers — a Linux **AppImage** (linuxdeploy over an AppDir carrying a `.desktop` entry and appstream metadata, plus a portable tarball fallback), a macOS **`.app`** → **`.dmg`** bundle, and a Windows **MSI** via WiX v4 whose product definition is kept in-repo (`packaging/wix/Product.wxs` with the MIT licence text) so the installer metadata stays reviewable. `docs/pdfium.md` documents the PDFium run-time contract (discovery order `RUBRIC_PDFIUM_PATH` → executable directory → system path, where to obtain binaries, and what a failed import looks like). Installers ship **unsigned** — no signing certificate is committed to this repository (recorded as Q-PACK-1; raster icons as Q-PACK-2).
- Page thumbnails sidebar (spec F-404): a resizable left panel showing one cell per page. PDF-backed projects show a real preview rendered through the shared on-disk page cache (so a page the main canvas already rendered costs nothing); projects with no PDF still show numbered placeholders, so navigation is never a dead control. Clicking a cell jumps to that page. The thumbnail grid packing (`thumb_layout`: columns that fit, rows needed, cell size) lives in `src/domain/thumbs.rs` as pure geometry with tests. Page switching (top-bar arrows, Page Up/Down, sidebar) now shares one `RubricApp::jump_to_page` helper so all three agree on text-edit commit, undo snapshot and draft-abandon semantics. 6 new domain tests.
- Canvas grid, snap-to-grid and rulers (spec F-405/F-406): new pure domain module `src/domain/grid.rs` — [`Grid`] spacing/visibility/snap with nearest-intersection snapping, plus a 1/2/5 `ruler_ticks`/`nice_step` ladder — and canvas rendering in `src/ui/canvas.rs` (`draw_grid` under the markups, skipped when the pitch falls below 6 screen pixels; `draw_rulers` labelled page-point ticks on the top/left margins that follow pan and zoom). Snapping applies to markup *creation anchors* only: hit-testing keeps using the exact pointer position, so a markup that is not grid-aligned remains selectable. View menu gains **Show Grid**, **Snap to Grid** and a **Grid Spacing** submenu (12/24/36/72 pt). Grid state is session-only and never enters `.rubric.json`. 10 new domain tests.
- Editor context menus (spec F-603): a shared menu (`src/ui/menu.rs`) attached to the canvas page response and to the inspector panel response, offering a Tool submenu (all 24 tools with their F-601 shortcut keys), Delete Selected / Clear Selection (disabled with no selection), and Undo / Redo / Zoom In / Zoom Out / Fit Page. Commands are modelled by the new domain enum `EditorAction` (`src/domain/action.rs`: `ALL`, `label`, `is_enabled`) and executed through one path (`RubricApp::apply_editor_action`) shared with the Edit/View menu bar. The canvas opens the menu only on a plain right-click — a right-click during a multi-point draft still commits that draft. 4 new domain tests; the UI renders the model and only *requests* actions (`UiState::pending_action`).
- Per-tool keyboard shortcuts (spec F-601): `Tool::ALL` (all 24 variants in declaration order), `Tool::shortcut_key()` and `Tool::from_shortcut_key()` in `src/domain/project.rs` (letters for markup/shape tools, digits `1`–`7` for measurements, ASCII case-insensitive reverse lookup), plus UI wiring in `src/ui/app.rs` (`select_tool` abandons any in-flight draft gesture exactly like Esc while keeping the selection; `key_to_shortcut_char` maps egui `Key` variants). Only unmodified keys fire — blocked while text editing, while a widget wants the keyboard, or with Ctrl/Alt/Super held (Shift is allowed since the `Key` variant is matched, not the typed character). 9 new domain tests (ALL completeness/order, every tool has a key, pairwise-unique keys, round-trip, case-insensitivity, unknown-key rejection, documented mapping table, serde unaffected).
- Dark theme (spec F-602): View > Theme menu toggles Light/Dark, persisted as an app-level preference (`AppPrefs` in the OS config dir) via `set_theme`; egui 0.36 per-theme visuals use Rubric's brand palette (light paper chrome vs. indigo dark surfaces) while the document page itself stays light. 9 new tests (5 domain `Theme`/`AppPrefs` incl. serde round-trip and legacy default, 3 prefs store incl. corrupt-file fallback and disk round-trip, 1 `apply_theme` theme/visuals selection).
- Page image export + print (spec F-505): `src/data/render.rs` renders the current page headlessly (headless egui frame + CPU triangle rasterizer with texture sampling and alpha blending) to PNG, and `src/data/print.rs` sends the rendered page to the OS print command (lp / lpr / PowerShell Start-Process -Verb Print); File menu gains `Export Page Image…` and `Print…` with honest status messages. 7 new unit tests.
- Phase 5 PDF import + page navigation (spec F-504/F-403): `Project::from_pdf` (per-page Letter/A4 sizing, read-only source path, serde-defaulted `source_pdf`), `data::pdf` runtime binding to PDFium via `pdfium-render` with a hash-keyed PNG render cache and `RUBRIC_PDFIUM_PATH` discovery, lazy background page textures (~144dpi, poisoned per cache key so failed pages aren't retried every frame), top-bar + PageUp/PageDown page navigation (guarded while in-canvas text editing), Import PDF… via native dialog, shared `install_project` path; 14 new tests. CI fetches the PDFium library and runs the lib-dependent tests against it.
- Phase 1 requirement set and spec (`docs/spec/README.md`): 52 feature IDs across F-1xx…F-7xx with priorities mapped to implementation phases; `REQUIREMENTS_TRACEABILITY.md` extended with spec-level requirements R-S-001…R-S-017 and seven group rows.
- Phase 2 slice A — interactive canvas: drag-to-draw with live preview (bounds/segment), ink freehand drawing, wheel zoom anchored on cursor (10–1600%), Space/middle-button pan, Del to delete selection.
- Phase 2 slice B — multi-point drawing: click-to-append polyline/polygon/perimeter/area paths with rubber-band preview (commit via double-click/right-click/Enter, Esc cancels), three-click angle tool with live ray preview.
- Phase 3 — measurement labels rendered on canvas (domain-formatted, geometry-anchored, screen-space size); inspector scale editor with presets (1:1, 1 in = 1/10/50 ft), display-unit combo, and custom paper-inches : real-units ratio; scale/unit edits recompute page measurements with one undo snapshot per commit.
- Measurement precision preference: `Settings::precision` (0–6 decimal places, serde default 3 so older `.rubric.json` files load unchanged), `format_value_precise`/`Measurement::label_with`, and an inspector DragValue that commits one snapshot per edit. Angle stays at 2 decimals, counts at 0.
- Layer visibility/lock: `Layer` domain model (visible/locked/z, serde-migrated — legacy `.rubric.json` files load onto a per-document default layer), document ops (`add_layer`, `remove_layer` refusing default/populated layers, visibility/lock setters, `is_markup_visible`/`is_markup_editable` predicates, fail-open on unknown ids), inspector Layers section (eye/lock toggles, add/delete with status feedback), layer ComboBox for the selected markup, and canvas/inspector/delete gates so hidden markups aren't painted or hit-tested and locked markups can't be moved, deleted, or edited. 15 new tests.
- Stamp library (spec F-111): `StampPreset` domain enum (Approved/Rejected/Draft/Check/Cross with word labels, drawn glyphs ✓/✗, meaning-grouped palette colours), `Markup.stamp: Option<StampPreset>` (serde-defaulted — legacy stamps load as None and render unchanged), `with_stamp` builder; canvas renders the preset glyph centred with shrink-to-fit; stamp creation applies the active preset + colour; inspector preset picker for selected stamps (one snapshot per real change) and a tool-options selector for placement (7 new tests).
- In-canvas text editing (spec F-104/F-105): double-click to edit Text and Callout markups in place (multiline overlay anchored to the markup, focus-loss commit with one snapshot only on real change, Esc cancels); placement now creates proper `Text { at, wrap, text }` / `Callout { rect, tail, text }` geometry instead of bare bounds, with the editor auto-opening on creation; interaction arms (draw/move/delete/undo) suppressed while editing; empty text markups get a visible outline.
- Native file dialogs + recent files (spec F-5xx): rfd-based Open (Ctrl+O) and Save/Save As (Ctrl+S routes through the current path, `.rubric.json` extension normalised case-insensitively) with a Recent submenu (hover shows full paths, missing files self-prune, Clear Recent); recents persist app-level in the OS config dir (`RecentList` domain type + `data::recent` store) — never written into shareable project files; window title shows the active file; Quit goes through eframe teardown. `rfd` uses the pure-Rust xdg-portal backend on Linux (no GTK dependency).
- Hit testing v2: filled shapes hit interior, stroke-only shapes hit outline only; elliptical edge test; polygon even-odd interior test; ink segment-based (incl. dot strokes) — 8 new unit tests.
- Status workflow per spec (F-301/F-305): `ReviewStatus` remapped to Open / In review / Approved / Rejected with distinct palette colours, inspector status legend (live counts per status), colour swatches in the status selector, coloured status dots on canvas. 6 new tests; legacy status strings no longer load — sanctioned since no released files exist.
- Subagent suite (`.opencode/agents/`): `domain-dev`, `test-writer`, `ui-dev`, `reviewer`, `docs-writer`, `fixer` — all on free OpenCode models, restricted to `cargo fmt` + read-only git locally.
- CI-first build policy: all heavy gates (`cargo test`, `clippy -D warnings`, `check`, release build) run only in GitHub Actions; workflow now caches builds with `Swatinem/rust-cache@v2`.


### Changed
- CI verified fully green (fmt, clippy, tests, release build) after billing resolution; process docs updated accordingly.


### Fixed
- Area measurement rescaling: area values now scale with `units_per_point²` (previously used the linear factor, and carried an unnecessary cast).
- Switching the display unit now preserves physical measurements (`Scale::with_display_unit` converts `units_per_point` via `points_per_unit`) instead of silently reinterpreting the scale ratio.
- Undo history no longer spams snapshots during selection drags (one snapshot per completed drag instead of dozens).
- Esc now abandons an in-progress drag-to-draw instead of re-anchoring the draft.
- Tool switching now discards an in-progress multi-point draft (no ghost preview or stale Enter/right-click commits).
- Segment tools (line/arrow/length/diameter/radius) no longer commit wrong-typed geometry on a bare click; multi-point commits validate distinct points; double-click no longer adds a duplicate final vertex.
- Enter/Space fake-clicks while a text field has focus can no longer trigger canvas commits.
- Docs: stale markup-kind count corrected — `MarkupKind` has 23 variants (traceability, data model, plan, progress, decisions, and the 0.1.0 note said 22).


### Docs
- Adopt the guide §5 **CTO.new-style free process**: `Chat (plan) → Task (execute) → Pull Request (review) → Merge`. `AGENTS.md` documents the operating model; `docs/development.md` reframes the slice workflow as Tasks with a PR-for-review step; `.opencode/agents/orchestrator.md` becomes the Chat/planning role and a new `.opencode/agents/coder.md` is the Task executor; `docs/qc/checklist.md` Definition of Done gains the Pull Request + human-merge items. Added the required `docs/parity.md` capability-parity tracker. GitHub Actions is named the free "Task Runner" throughout.


### Known limitations

- PDF import needs the PDFium shared library next to the executable (or `RUBRIC_PDFIUM_PATH`); every other feature works without it (see `docs/pdfium.md`).
- Planned follow-ups: note popovers (F-103), copy properties (F-114), z-order (F-115), autosave wiring (F-502), `schema_version` (F-507), accessibility pass (F-605) — tracked in `ROADMAP.md`.
- Installers are unsigned (no signing certificate in the repository, Q-PACK-1); desktop icons are SVG-only pending a raster set (Q-PACK-2).
- Crash reports and update checks are on-demand only; nothing is sent or queried without an explicit user action (see README privacy section).

### Notes
- Headless-testable (domain + data). UI requires display at runtime; builds/tests/lint/format work in headless environment.
- PDF import binds to PDFium at run time (`docs/pdfium.md`); the page renders as a blank rectangle until the library is present.
- Never commits external reference documents or the build guide.
