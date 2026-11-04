# Roadmap

Rubric's plan toward a **finished** application (guide §6, §9). Readiness is
computed **only from verified evidence** — a CI-green `cargo test`/`clippy`
run or hands-on verification on the primary local platform (Linux, aarch64).
Never from intention.

**Evidence source of truth:** `REQUIREMENTS_TRACEABILITY.md` (requirement →
feature → module mapping) and `docs/spec/README.md` (the 52 feature IDs).
This file is the staged, milestone view over those.

---

## Readiness (2026-10-26)

| Measure | Value | Evidence |
|---|---|---|
| Requirement groups fully done | **17 / 17 (100 %)** | `REQUIREMENTS_TRACEABILITY.md` — all R-S rows marked DONE |
| Requirement groups partial | 0 / 17 | — |
| Requirement groups not started | 0 / 17 | — |
| Feature IDs done (52 total) | **52 / 52 (100 %)** | `docs/spec/README.md` reconciled 2026-10-30 |
| Feature IDs open | 0 | — |
| CI | green on `main` | fmt · clippy `-D warnings` · tests · release build |
| Primary-platform install | verified | installed from CI artefact and launched (Linux aarch64, Crostini) |

Change to any percentage requires a linked CI run or a recorded hands-on check
in `PROGRESS.md`.

---

## Milestones

### v0.1.0 — first functional test build (guide §6.1 hard gate)

**Status: criteria met — tag pending.**

| §6.1 requirement | Status | Evidence |
|---|---|---|
| Installable / runnable on primary local OS | ✅ | installed from CI artefact, launches (`PROGRESS.md` 2026-10-24) |
| One real end-to-end workflow with real data model + persistence | ✅ | draw → measure → review → save `.rubric.json` → reopen |
| App identity (original name, window title, theme) | ✅ | "Rubric", themed, light/dark (F-602) |
| Crash stub | ✅ | panic hook + captured report + review/cancel dialog (crash-report crate) |
| Update-check stub | ✅ | version query + GitHub Releases check, fail-soft offline (update-check crate) |
| CI green (fmt, lint, tests, primary OS build) | ✅ | every push to `main` |
| Docs (README build/run, CHANGELOG 0.1.0 entry) | ✅ | README, CHANGELOG |
| Legal baseline (LICENSE, ATTRIBUTION, no secrets/assets) | ✅ | MIT; `ATTRIBUTION.md`; legal checklist §2.5 |
| Tag `v0.1.0` published as GitHub Release with honest notes | ✅ | [Release](https://github.com/Perchemop/rubric-desktop/releases/tag/v0.1.0) — 4 platform zips + SHA256SUMS, release run `40049997043` |

### v0.1.x — polish the shipped core

Small, low-risk completions; each its own Task + release.

- [x] F-401: zoom presets (50 / 100 / 200 %) in the View menu — v0.2-07
- [x] F-004: layer rename / reorder in the Inspector — v0.2-07 (layer-major paint order, default layer pinned)
- [x] F-103: floating note popover on the canvas pin — v0.2-08 (threaded comments via Markup::add_comment)
- [ ] Installer polish: bundle PDFium in every artefact; record checksums

### v0.2.0 — finish the review workflow

- [x] v0.2-01 domain foundation: Markups List (filter/sort/group), style presets, project diff (R-V02-01/02) — CI `40304244261`
- [x] v0.2-02 Markups List UI: resizable panel with search/status/author/subject/layer filters, all sorts and group modes, row selection + page jump, View toggle (R-V02-01) — PR #34
- [x] F-114: copy properties → apply style to other markups — v0.2-03 (visual style only, locked targets skipped, one snapshot)
- [x] F-115: z-order (bring forward / send backward / to front / to back) — v0.2-03 (page-scoped, boundary no-ops)
- [x] F-502: wire the existing `store::autosave` into the app loop (interval + status) — v0.2-04 (File > Auto-save, dirty-only, one write per interval)
- [x] F-507: `schema_version` field + forward-compatible load — v0.2-04 (load refuses unsupported versions)

### v0.3.0 — accessibility & robustness

- [x] F-605: accessibility pass — v0.2-09 (WCAG contrast maths + palette audit test, audited body-text colours, icon-button tooltips, `docs/a11y.md`)
- [x] Audit-fix wave v0.3-01: all 9 high-severity defects from `docs/audits/v0.2.0-full-audit.md` (version bump, unit conversion, count numbering, strikeout/underline/squiggly rendering, inspector commit/dirty, print errors, Fit Page, PDFium bundling tracked separately as H8) plus the in-scope medium findings — parallel subagent workstreams, one PR
- [x] **v0.3-02 Phase 1**: hayro-syntax page-size probing (H8) — PR #66 merged
- [x] **v0.3-02 Phase 2**: hayro rendering pipeline (H8) — PR #67 merged
- [x] **v0.3-02 Phase 3**: corpus fidelity tests (≥95% pixel match vs PDFium) — PR #74 merged
- [x] **v0.3-04 (M4)**: export/print report a failed PDF background instead of a blank page
- [x] **v0.3-04 (M5)**: fixed-DPI, page-cropped export/print — owner decision:
      crop to page bounds at 150 DPI; export render suppresses backdrop, page
      border, grid and rulers; PDF overlay cropped via `markup_intersects_page`
      — PR #88 merged (commit `8bc2601`)
- [x] **v0.3-05**: `lopdf` write support (Export PDF…) — PRs #79-84 merged
- [x] Process: `docs/capability-inventory.md` (guide II.27.3) — capability rows with source URLs, acceptance criteria and evidence (gap G3 in `docs/guide-compliance.md`) — 2026-10-30
- [ ] Process: extract a `Host`/plain-`Ui` seam from `RubricApp` so `egui_kittest` can render real widgets to PNG as CI visual evidence (guide II.27.12, gap G4)
- [ ] Empty / error states for every panel; no dead controls
- [x] ~~`gitleaks` (or GitHub secret scanning) in CI~~ — done early 2026-10-24:
      push-protected secret scanning at repo level plus an in-CI pattern scan
      (`security.yml`, R-C-009) and CodeQL SAST (R-C-011)

### v0.4.0 — capability-gap follow-ups

Items from the ranked gap list in
`docs/research/v0.4-capability-gap-analysis.md`; they are not spec feature IDs,
so they do not move the 52-feature readiness table above.

- [x] **v0.4-01**: multi-select (gap #2) — Shift+click toggle + marquee drag — PR #86 merged
- [x] **v0.4-02**: batch status (gap #3) — `EditorAction::BatchStatus` +
      `Document::apply_status_to`, plus multi-select status dropdowns in the
      Inspector and the Markups List — PR #88 merged (commit `8bc2601`)
- [x] **v0.4-03**: duplicate markups (gap #6) — `Document::duplicate_markups` +
      `EditorAction::Duplicate` (Ctrl+D) — PR #93 merged (commit `8a70739`)
- [x] **v0.4-04**: flatten-to-PDF (gap #1) — per-page `export_page_to_pdf`
      (lopdf page pruning) + `export_all_pages_to_pdf`, File menu entries — PR #96 merged
- [x] **v0.4-05**: drag calibration (gap #4) — `Tool::Calibrate` two-click
      capture, `calibrate_scale()` in `domain/units.rs`, `CalibrationDialog` — PR #96 merged
- [x] **v0.4-06**: resize grips (gap #5) — `Corner` +
      `Markup::resize_bounds()` domain clamp, canvas corner-handle drag, plus
      segment-endpoint/path-vertex point handles and `Markup::move_point()` —
      PR #96 and PR #98 merged (follow-up commit `cac048b`)
- [x] **v0.4-07**: tool chest (gap #7) — `StylePreset::from_style()` +
      `save_named()`, "My Tools" panel (`ui/tool_chest.rs`), apply-at-creation — PR #96 merged
- [x] **v0.4-08**: editable Markups List console (gap #8) — inline status
      combo, filtered-view measurement totals, CSV export — PR #96 merged
- Gaps 1–8 of `docs/research/v0.4-capability-gap-analysis.md` are all closed
      (gap #5 fully: bounds resize, segment endpoints and path vertices — PR #98)
- [x] Audit follow-ups (PRs #90–#92): M18 zoom-sharp page raster, M24
      thread-locals into `UiState`, WCAG status text, M5 menu shortcut hints +
      Ctrl+A, Save-and-Quit

### v1.0.0 — finished (guide §9)

- [x] All 52 feature IDs done; all 17 requirement groups DONE (CI-verified)
- [ ] Multi-platform installers verified hands-on (Linux ✅, macOS, Windows)
- [ ] §2.5 legal checklist clean for a public release
- [ ] README/ROADMAP claims re-audited (no over-claiming)

### v0.5 — Design polish & systemisation (design & GUI audit, PR #101)

**Audit backlog P0 — High impact:**
- [x] Icon font/SVG set for tools, layers, Tool Chest, page nav (original egui-painter vector icons, `ui/icon_button.rs`)
- [ ] Bundle UI font + tabular monospace for measurement readouts (partial: typography tokens + monospace token done; font assets pending)
- [x] Persist workspace layout — panel visibility, zoom, page in prefs (`AppPrefs`, P0-3)
- [x] Active-tool chip (always visible) (P0-4); keyboard shortcuts dialog shipped (PR #100)
- [x] Empty states for primary panels (no document, no markups, empty filtered list) (P0-5)

**Audit backlog P1 — Quality:**
- [x] Elevation + spacing + radius tokens (corner_radius + elevation 0–3, P1-6)
- [x] Segmented Inspector tabs (Selection / Properties / Comments / Measure / Settings) — tab bar + gated section content; Status legend, Layers and Scale stay visible by design
- [x] Virtualised Markups List (egui `ScrollArea` viewport culling; async thumbnails pending)
- [x] Single tool metadata source drives all UIs (`ToolMeta`/`TOOL_METADATA`, P1-9)
- [x] Persist Tool Chest library in prefs (P1-10)
- [ ] Host/Ui seam + egui_kittest visual/a11y evidence G4 (guide II.27.12) — documented deferral

**Audit backlog P2 — Later:**
- [ ] Text-anchored highlight/strikeout (part 1 foundation: `MarkupGeometry::TextAnchor`; full PDF text extraction pending)
- [x] Focus / reading mode (P2-13)
- [x] Dual-unit rulers (P2-14)
- [x] UI scale + high-contrast preference (UI scale PR #116; high-contrast mode via View ▸ High Contrast)

**Guide §12.1 suite standards (audit approved 2026-11-04):**
- [x] Token ledger `docs/tokens.json` + release-workflow update + release-notes reference (guide §12.1)
- [x] In-app subtle version + release-notes popup with history/usage/token metrics (guide §9, §12.1)
- [ ] Signed releases + in-app checksum-verified auto-update install (guide §12.1)

**Audit follow-ups:**
- [ ] Automated AccessKit/snapshots for visual evidence (G4 — documented deferral)
- [ ] Focus ring audit
- [x] High-contrast preference (P2-15)
- [ ] Text labels when narrow panels use icons only

### v0.6 — Bates numbering (future slice)

Queued after the remaining v0.5 audit slices. Sequential page stamping for legal
/ document-review workflows, as a **non-destructive overlay** (source PDF never
modified). Research: `docs/research/bates-numbering.md`; spec F-801–F-805;
capability inventory row 32.

- [x] Configurable prefix / suffix / start number / digit padding (with first/last stamp preview)
- [x] Placement (corner, margin) and stamp font size
- [x] Page range (all / from-to) with an honest "N of M pages" count
- [ ] Ghosted canvas overlay preview
- [ ] Stamped export + print in page order (bakes into the composed output only)

### Community, security and operating hygiene — done 2026-10-24 (guide Part II)

Not product features: tracked as R-C-001…R-C-012 in
`REQUIREMENTS_TRACEABILITY.md`, so the 52-feature readiness above is
unchanged by this work.

- Community files: `CODE_OF_CONDUCT.md`, `SECURITY.md` (threat model +
  incident response), `ACCESSIBILITY.md`, `VISION.md`, `GOVERNANCE.md`,
  `CONTRIBUTORS.md`; issue templates (bug / feature / **accessibility**)
  and the PR template gained the accessibility checklist.
- CI security: `security.yml` — secret-pattern scan, PR dependency review,
  `cargo-deny` licence/advisory gate (`deny.toml`), SBOM (SPDX + CycloneDX),
  CodeQL SAST; `.github/dependabot.yml` (cargo + actions, weekly).
- Repository posture: branch protection on `main` (required check `check`,
  admin-enforced, no force pushes), private vulnerability reporting,
  Dependabot security updates — all enabled and API-verified 2026-10-24.
- Governance/API hygiene: a11y/area labels created; Discussions enabled;
  name-conflict check (Q28), employer/IP note (Q29), licence question
  (Q-LIC-1) and the six Part II §26 decisions recorded in
  `docs/assumptions-and-questions.md`; metrics statement in `docs/metrics.md`.

---

## Sequencing rules

1. One Task = one small vertical slice; PR per Task (`.github/PULL_REQUEST_TEMPLATE.md`).
2. Heavy builds/tests run on GitHub Actions; the local machine only runs the
   produced binary for UX testing (guide §4).
3. Percentages move only with evidence.
4. Nothing is "done" until it is implemented, tested, CI-green, and — where it
   affects the UI — tried on the primary platform.
