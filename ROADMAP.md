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
- [x] Process: `docs/capability-inventory.md` (guide II.27.3) — capability rows with source URLs, acceptance criteria and evidence (gap G3 in `docs/guide-compliance.md`) — 2026-10-30
- [ ] Process: extract a `Host`/plain-`Ui` seam from `RubricApp` so `egui_kittest` can render real widgets to PNG as CI visual evidence (guide II.27.12, gap G4)
- [ ] Empty / error states for every panel; no dead controls
- [x] ~~`gitleaks` (or GitHub secret scanning) in CI~~ — done early 2026-10-24:
      push-protected secret scanning at repo level plus an in-CI pattern scan
      (`security.yml`, R-C-009) and CodeQL SAST (R-C-011)

### v1.0.0 — finished (guide §9)

- [ ] All 52 feature IDs done; all 17 requirement groups DONE
- [ ] Multi-platform installers verified hands-on (Linux ✅, macOS, Windows)
- [ ] §2.5 legal checklist clean for a public release
- [ ] README/ROADMAP claims re-audited (no over-claiming)

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
