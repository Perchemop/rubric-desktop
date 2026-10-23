# Changelog

All notable changes to this project will be documented here. Format: semantic-ish for this repo (Added/Changed/Fixed/Docs).

## [Unreleased]

### Added
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

### Fixed
- Area measurement rescaling: area values now scale with `units_per_point²` (previously used the linear factor, and carried an unnecessary cast).
- Switching the display unit now preserves physical measurements (`Scale::with_display_unit` converts `units_per_point` via `points_per_unit`) instead of silently reinterpreting the scale ratio.
- Undo history no longer spams snapshots during selection drags (one snapshot per completed drag instead of dozens).
- Esc now abandons an in-progress drag-to-draw instead of re-anchoring the draft.
- Tool switching now discards an in-progress multi-point draft (no ghost preview or stale Enter/right-click commits).
- Segment tools (line/arrow/length/diameter/radius) no longer commit wrong-typed geometry on a bare click; multi-point commits validate distinct points; double-click no longer adds a duplicate final vertex.
- Enter/Space fake-clicks while a text field has focus can no longer trigger canvas commits.
- Docs: stale markup-kind count corrected — `MarkupKind` has 23 variants (traceability, data model, plan, progress, decisions, and the 0.1.0 note said 22).

### Changed
- CI verified fully green (fmt, clippy, tests, release build) after billing resolution; process docs updated accordingly.

## [0.1.0] - 2026-10-17 — Phase 0

### Added
- Original name/brand "Rubric": SVG logo, multi-size PNG icons (16/32/48/64/128/256/512/1024), branding spec.
- Fresh Rust crate `rubric` (egui/eframe 0.36, serde, uuid, thiserror, tracing, chrono, image).
- Domain-first architecture: geom, units (scale + conversions), style (colours/dashes/fill modes), markup (23 kinds, geometry variants, measurements, hit testing, review status, comments), document (pages/markups), project (toolsets, settings, history with undo/redo capped 128).
- Data layer: JSON persistence (`.rubric.json`), app data dir via directories, error types.
- UI skeleton: app shell (panels, menu, shortcuts), canvas (page, markups, selection, basic place/move), toolbar (tool groups), inspector (properties/comments/measurements), theme/icons.
- Tests: unit tests in domain modules + integration tests (6) covering geometry, measurements, serialisation, project lifecycle.
- Tooling: .gitignore (excludes secrets/target/local notes), LICENSE (MIT), rust-toolchain.toml, CI skeleton, scripts/gen-icons.sh.
- Docs: README, AGENTS, architecture, data model, test plan, assumptions/questions, implementation plan, requirements traceability, decisions, progress, changelog.

### Notes
- Headless-testable (domain + data). UI requires display at runtime; builds/tests/lint/format work in headless environment.
- No PDF rendering yet (blank page rectangle); to be added later.
- Never commits external reference documents or the build guide.
