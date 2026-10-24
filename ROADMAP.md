# Roadmap

Rubric's plan toward a **finished** application (guide §6, §9). Readiness is
computed **only from verified evidence** — a CI-green `cargo test`/`clippy`
run or hands-on verification on the primary local platform (Linux, aarch64).
Never from intention.

**Evidence source of truth:** `REQUIREMENTS_TRACEABILITY.md` (requirement →
feature → module mapping) and `docs/spec/README.md` (the 52 feature IDs).
This file is the staged, milestone view over those.

---

## Readiness (2026-10-24)

| Measure | Value | Evidence |
|---|---|---|
| Requirement groups fully done | **13 / 17 (76 %)** | `REQUIREMENTS_TRACEABILITY.md` — R-S rows marked DONE |
| Requirement groups partial | 4 / 17 | R-S-001, R-S-003, R-S-013, R-S-016 (small residual items only) |
| Requirement groups not started | 0 / 17 | — |
| Feature IDs done (52 total) | **46 / 52 (88 %)** | `docs/spec/README.md` reconciled 2026-10-24 |
| Feature IDs open | 6 | F-103, F-114, F-115, F-502, F-507, F-605 (+2 sub-items: F-401 zoom presets, F-004 layer rename/reorder) |
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
| Tag `v0.1.0` published as GitHub Release with honest notes | ⏳ | this milestone's final action |

### v0.1.x — polish the shipped core

Small, low-risk completions; each its own Task + release.

- [ ] F-401: zoom presets (50 / 100 / 200 %) in the View menu
- [ ] F-004: layer rename / reorder in the Inspector
- [ ] F-103: floating note popover on the canvas pin
- [ ] Installer polish: bundle PDFium in every artefact; record checksums

### v0.2.0 — finish the review workflow

- [ ] F-114: copy properties → apply style to other markups
- [ ] F-115: z-order (bring forward / send backward / to front / to back)
- [ ] F-502: wire the existing `store::autosave` into the app loop (interval + status)
- [ ] F-507: `schema_version` field + forward-compatible load

### v0.3.0 — accessibility & robustness

- [ ] F-605: accessibility pass (contrast audit, full keyboard reachability, screen-reader labels)
- [ ] Empty / error states for every panel; no dead controls
- [ ] `gitleaks` (or GitHub secret scanning) enabled in CI

### v1.0.0 — finished (guide §9)

- [ ] All 52 feature IDs done; all 17 requirement groups DONE
- [ ] Multi-platform installers verified hands-on (Linux ✅, macOS, Windows)
- [ ] §2.5 legal checklist clean for a public release
- [ ] README/ROADMAP claims re-audited (no over-claiming)

---

## Sequencing rules

1. One Task = one small vertical slice; PR per Task (`.github/PULL_REQUEST_TEMPLATE.md`).
2. Heavy builds/tests run on GitHub Actions; the local machine only runs the
   produced binary for UX testing (guide §4).
3. Percentages move only with evidence.
4. Nothing is "done" until it is implemented, tested, CI-green, and — where it
   affects the UI — tried on the primary platform.
