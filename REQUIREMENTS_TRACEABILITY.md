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
| R-S-001 | Markup tools: highlight, text markups, note, text, callout, shapes, cloud, polyline/polygon, line/arrow, ink, stamps | F-101–F-111 | src/domain/markup.rs, src/ui/toolbar.rs, src/ui/canvas.rs | PARTIAL (only F-103 note popover open) |
| R-S-002 | Drag-to-draw with live preview (bounds/segment/path/angle/ink) | F-112 | src/ui/canvas.rs (DrawState) | DONE |
| R-S-003 | Selection editing: select/move/delete, copy properties, z-order | F-113–F-115 | src/ui/canvas.rs, src/ui/inspector.rs, src/domain/project.rs | PARTIAL (F-114 copy properties, F-115 z-order open) |
| R-S-004 | Layer visibility and lock | F-116 | src/domain/project.rs (ToolSet → Layer), src/ui/inspector.rs | DONE |
| R-S-005 | Measurements: length, perimeter, area, angle, diameter, radius, count | F-201–F-206 | src/domain/units.rs, src/domain/markup.rs | DONE |
| R-S-006 | Calibration and units: scale presets, custom ratio, unit selection | F-207, F-208 | src/domain/units.rs, src/domain/document.rs, src/ui/inspector.rs | DONE |
| R-S-007 | Measurement labels on canvas + precision/format preferences | F-209 | src/ui/canvas.rs, src/ui/inspector.rs | DONE |
| R-S-008 | Review workflow: statuses (Open/In review/Approved/Rejected), comments, author/subject | F-301–F-303 | src/domain/markup.rs (ReviewStatus, remapped 2026-10-19), src/domain/project.rs | DONE |
| R-S-009 | Summary report (markups by status/author/subject, exportable) | F-304 | src/domain/report.rs (new), src/data/export.rs (new), src/ui/app.rs | DONE |
| R-S-010 | Status colours and legend | F-305 | src/ui/inspector.rs (Status legend + swatches), src/domain/markup.rs (`ReviewStatus::color()`), src/ui/canvas.rs (status dots) | DONE |
| R-S-011 | Canvas navigation: zoom, pan, page navigation, thumbnails | F-401–F-404 | src/ui/app.rs, src/ui/canvas.rs, src/ui/pages.rs, src/domain/thumbs.rs | PARTIAL (F-404 thumbnails done 2026-10-23) |
| R-S-012 | Grid, snap, rulers | F-405, F-406 | src/domain/grid.rs, src/ui/canvas.rs, src/ui/app.rs (View menu) | DONE |
| R-S-013 | Persistence: `.rubric.json` sidecar, autosave, recent files, native dialogs, schema version | F-501–F-503, F-506, F-507 | src/data/store.rs, src/ui/app.rs | PARTIAL |
| R-S-014 | PDF import and page rendering | F-504 | src/data/pdf.rs (new), src/ui/canvas.rs | DONE |
| R-S-015 | Export (rendered image) and print | F-505 | src/data/export.rs (new), src/data/render.rs (new), src/data/print.rs (new), src/ui/app.rs | DONE |
| R-S-016 | UX polish: shortcuts, dark theme, context menus, undo/redo, accessibility | F-601–F-605 | src/ui/app.rs, src/ui/theme.rs, src/domain/project.rs, src/domain/theme.rs, src/data/prefs.rs | PARTIAL |
| R-S-017 | Packaging: Linux AppImage, macOS DMG, Windows MSI, CI release artifacts | F-701–F-704 | .github/workflows/release.yml, .github/workflows/package.yml, packaging/wix/, docs/pdfium.md | DONE |

## Feature requirements

Legacy IDs F-00x predate the spec; group rows F-1xx…F-7xx reference `docs/spec/README.md` (2026-10-19).

| ID | Feature | Target in Rubric | Status | Notes |
|---|--------|----------------|--------|-------|
| F-001 | Markups (highlights, lines, shapes, text, callouts, stamps) | MarkupKind + Geometry | DONE (core) | expand tool set over time |
| F-002 | Measurements (length/perimeter/area/angle/count) | Measurement + recompute | DONE | add diameter/radius UI polish |
| F-003 | Review workflow (statuses, comments, subjects) | ReviewStatus, Comment | DONE | spec status set Open/In review/Approved/Rejected landed (F-301, 2026-10-19) |
| F-004 | Layers/Toolsets | ToolSet + Layer field | PARTIAL | visibility/lock landed; rename/reorder pending |
| F-005 | Scale/calibration | Page.scale + units | DONE | inspector UI editor landed (Phase 3) |
| F-006 | Page navigation/thumbnails | Document.pages | TODO | add UI |
| F-007 | Non-destructive markup (sidecar) | .rubric.json separate | DONE | PDF embedding later optional |
| F-1xx | Markups/tools + selection/edit (spec) | MarkupKind, Geometry, ToolSet → Layer | PARTIAL | docs/spec/README.md F-101–F-116; R-S-001–R-S-004; drag/multi-point/ink, text+callout editing (F-104/F-105) and stamp library (F-111) landed 2026-10-19/20; note popover (F-103), copy properties, z-order pending |
| F-2xx | Measurements + scale/calibration (spec) | Measurement, Page.scale, units | PARTIAL | docs/spec/README.md F-201–F-209; R-S-005–R-S-007 |
| F-3xx | Review workflow (spec) | ReviewStatus, Comment, author/subject | PARTIAL | docs/spec/README.md F-301–F-305; R-S-008–R-S-010; F-301/F-305 landed 2026-10-19, F-304 landed 2026-10-21 |
| F-4xx | Navigation/pages (spec) | Document.pages, UiState camera | PARTIAL | docs/spec/README.md F-401–F-406; R-S-011–R-S-012; F-404 (thumbnails), F-405 (grid + snap) and F-406 (rulers) done 2026-10-23 |
| F-5xx | Persistence/I/O (spec) | .rubric.json store, PDF import, export/print | PARTIAL | docs/spec/README.md F-501–F-507; R-S-013–R-S-015; native dialogs + recent files landed 2026-10-20; PDF import landed 2026-10-21; report export landed 2026-10-21; image export + print landed 2026-10-21 |
| F-6xx | UI/UX polish (spec) | shortcuts, theme, history | PARTIAL | docs/spec/README.md F-601–F-605; R-S-016; F-601 (per-tool keys), F-602 (dark theme), F-603 (context menus) done 2026-10-23 |
| F-7xx | Packaging (spec) | CI release build, installers | DONE | docs/spec/README.md F-701–F-704; R-S-017; release.yml + package.yml done 2026-10-23 (installers unsigned — see docs/assumptions-and-questions Q-PACK-1) |
| F-8xx | Community/security/ops readiness (operating guide Part II — not product-spec scope; tracked as R-C-*) | — | DONE (initial wave) | see "Community, security and operating-hygiene requirements" below |

## v0.2 requirements (reviewer/auditor milestone)

| ID | Requirement | Status | Trace |
|---|-----------|--------|-------|
| R-V02-01 | Markups List: snapshot from project/document; filters (status/author/subject/kind/page/layer/search); stable sorts; grouping | DONE | `src/domain/markup_list.rs`; `src/ui/markups_list.rs`; `tests/domain_tests.rs::markup_list_filter_and_sort_from_document`; `docs/tasks/v0.2-01-markup-list-domain.md`; `docs/tasks/v0.2-02-markups-list-ui.md`; PR #34; implementation CI run `40308480821` |
| R-V02-02 | Status workflow audit metadata: reusable style presets (capture/apply + library) and project diff with added/removed/changed by stable id | PARTIAL (domain DONE; preset/diff UI pending) | `src/domain/style_preset.rs`, `src/domain/diff.rs`; `tests/domain_tests.rs::{diff_documents_reports_add_remove_change,style_preset_apply_and_roundtrip}`; CI `40304244261` |

## Gaps / exclusions
- Canonical inventory: `docs/spec/README.md` (52 F-IDs across F-1xx…F-7xx), adopted 2026-10-19; see Phase 1 requirement set above.
- PDF import — DONE 2026-10-21 (F-503); report/image export + print — DONE 2026-10-21 (F-505/F-506, F-504 printer dialog pending re-verify on CI artefacts)
- Open product residuals: F-103 (note popover), F-114/F-115 (copy props/z-order), F-502, F-507, F-605 (+F-401 zoom presets, F-004 layer rename) — see Roadmap.
- Community/security/ops: done as of 2026-10-24 (R-C-001…R-C-012 below); ongoing duties (Dependabot triage, CodeQL findings triage) recorded in GOVERNANCE.md.

## Verification
- CI gate (the only execution environment for the Rust toolchain here):
  fmt `--check`, clippy `-D warnings`, full workspace tests, release build
  (`.github/workflows/ci.yml`). Contributor machines follow the same
  commands from `CONTRIBUTING.md` on unconstrained hardware.
- Security gate: `secrets`, `dependency-review`, `cargo-deny`, `sbom`,
  `codeql` (`.github/workflows/security.yml`).
- Packaging: `release.yml` (tag builds + GitHub Release publishing) and
  `package.yml` (AppImage/DMG/MSI).

## Community, security and operating-hygiene requirements (guide Part II)
Adopted 2026-10-24 from the operating guide's opensource.guide alignment
(Part II §16–§25). These are process/ops requirements, not product features:
they do not change the 52-product-feature readiness numerator. Ongoing-duty
items stay marked LIVE and are re-verified per release (§2.5-style check).

| ID | Requirement | Status | Trace |
|---|-----------|--------|------|
| R-C-001 | CODE_OF_CONDUCT.md present, original, with enforcement + report path | DONE | CODE_OF_CONDUCT.md |
| R-C-002 | SECURITY.md: private reporting, scope, threat model, incident response | DONE | SECURITY.md; private vulnerability reporting enabled (repo settings, 2026-10-24) |
| R-C-003 | ACCESSIBILITY.md: standard, priorities, limits, report path; a11y issue template + labels | DONE | ACCESSIBILITY.md; .github/ISSUE_TEMPLATE/accessibility_report.yml; labels `a11y`, `accessibility` |
| R-C-004 | Vision/roadmap published; scope-creep anchor | DONE | VISION.md; ROADMAP.md |
| R-C-005 | Governance recorded: decision model, roles, tiebreaker, agent least-privilege, org/bus-factor trigger | DONE | GOVERNANCE.md |
| R-C-006 | Contributor recognition + org backup-admin note | DONE | CONTRIBUTORS.md; GOVERNANCE.md |
| R-C-007 | Issue/PR templates incl. a11y checklist for UI changes | DONE | .github/ISSUE_TEMPLATE/*.yml, config.yml; .github/PULL_REQUEST_TEMPLATE.md |
| R-C-008 | CONTRIBUTING covers bug/feature/setup/local-test commands, response expectations | DONE | CONTRIBUTING.md |
| R-C-009 | CI secret scanning (push protection + in-CI pattern scan) | DONE | repo settings (push protection on, 2026-10-24); scripts/scan-secrets.sh; security.yml `secrets` job |
| R-C-010 | Dependency licence classes + SBOM in CI | DONE/LIVE | deny.toml; security.yml cargo-deny + sbom jobs; ATTRIBUTION.md policy section |
| R-C-011 | SAST/SCA: CodeQL on schedule+main; Dependabot versions; dependency-review on PRs; branch protection, required check, security updates | DONE/LIVE | security.yml; .github/dependabot.yml; repo settings (branch protection + Dependabot security updates enabled 2026-10-24) |
| R-C-012 | Launch/name/metrics hygiene: name-conflict check recorded, one home URL, metrics statement | DONE | docs/assumptions-and-questions.md Q28; README links; docs/metrics.md |

Verification evidence for the settings rows: GitHub API read-outs recorded in
`PROGRESS.md` (2026-10-24).
