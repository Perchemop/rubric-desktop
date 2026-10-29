# Guide & Addendum Compliance

Rule-by-rule record of how Rubric satisfies the local operating manual
(`guide.md`) and its `addendum.md`. Both are kept **local and untracked**; this
file is the only surviving statement of that review, so it names the rules and
the evidence without reproducing the manuals.

Hierarchy honoured when the two conflict (guide.md §0, addendum preamble):
**addendum wins on client interaction / Rust QA / session startup; guide wins on
clean-room / legal.**

Legend: **Met** = implemented as written · **Adapted** = implemented with a
recorded deviation · **Deferred** = tracked gap with a plan · **N/A** = not
applicable to this project.

Review dates: **2026-10-27**, re-reviewed **2026-10-28** after `guide.md` was
revised. The revision adds **§II.29** (the mandatory personal-data/secret scan
gate, owner directive 2026-10-27), extends §II.24 and §II.25, generalises Part
II to suite-wide wording (`<app>` instead of named products), and moves the
git-identity rule to §II.26.7. Reviewed against `guide.md` (Part I §0–18, Part
II §II.18–§II.29) and `addendum.md` (§A–§O).

---

## Part I — guide.md

| Rule | Status | Evidence / deviation |
| --- | --- | --- |
| §0 Discovery before code | Met | `docs/project-brief.md`, `docs/scope.md`, `docs/inspiration.md`, `docs/product-name.md`, `docs/assumptions-and-questions.md` |
| §0 Never commit the guide | Met | `guide.md`/`addendum.md` are untracked; only their requirements live here |
| §1 Finished-app mission | Met | real persistence + validation, headless tests, CI gates; `ROADMAP.md` |
| §2.1 Public black-box research only | Met | `docs/research/*.md` are original summaries + URLs; human-gated 2026-10-24 |
| §2.2 No RE / no proprietary paste / no trade dress | Met | `docs/inspiration.md` clean-room statement; all first-party code/assets |
| §2.3 Research after v0.1.0 + human gate | Met | `docs/research/bluebeam-studio-capabilities.md`, `adobe-acrobat-capabilities.md`, `stirling-pdf-capabilities.md`; approval recorded in `IMPLEMENTATION_PLAN.md` |
| §2.4 Research method ladder | Met | recorded in `docs/current-model-map.md` |
| §2.5 Legal checklist every release | **Deferred** | checklist exists in `docs/qc/checklist.md`; the per-release run + CI-run-ID record (II.19.5) has not yet been written for a v0.2.0 release — see gap G5 |
| §3 Stack (defaults) | Adapted | Rust + egui/eframe as recommended; persistence is pretty JSON `.rubric.json` instead of SQLite (DECISIONS 2026-10-17, "Persistence format") |
| §4 Local test + Actions for heavy work | Met | no `cargo` locally; `ci.sh` drives GitHub Actions; repo public |
| §4 Early CI + release workflows | Met | `.github/workflows/ci.yml`, `release.yml`, `package.yml`, `security.yml` |
| §5 Chat → Task → PR | Met | `docs/tasks/*`, PR template, `docs/process-feedback.md` |
| §5.2 Free model policy | Met | OpenCode free → NVIDIA free → keys; live map in `docs/current-model-map.md` (refreshed 2026-10-27) |
| §5.3 Agent roles | Met | `.opencode/agents/` (orchestrator, coder, domain-dev, ui-dev, test-writer, fixer, reviewer, docs-writer, explorer, designer, ci-runner) |
| §5.5 Git identity on every git action | Met (re-homed 2026-10-30) | commits are authored `Rubric Dev <dev@example.com>` (ci.sh asserts it); the rule is now recorded in `AGENTS.md` (gap G6 resolved); PR/push authorship follows the pushing account (platform limitation, documented) |
| §6.1 v0.1.0 hard gate | Met | `v0.1.0` tag + honest release notes; criteria table in `ROADMAP.md` |
| §6.2 Roadmap from evidence | Met | readiness % moved only with linked CI runs; see `PROGRESS.md` |
| §6.3 Release-notes standard | Met | F-template notes; latest release v0.1.0 |
| §7 Suite crash-report + update-check | Met | `crates/crash-report`, `crates/update-check`; user-token only |
| §8.2 Definition of Done | Met | per-slice in `docs/qc/checklist.md`; updated to the owner merge directive |
| §8.3 PR template + Rust QA | Met | `.github/PULL_REQUEST_TEMPLATE.md` includes §M.3 Rust QA |
| §8.4 Human gate on research | Met | 2026-10-24 approval; Stirling PDF client-directed approval |
| §9 Finished-product gates | Deferred (in part) | logic/CI/legal gates met for current scope; the **visual-evidence** gate needs the `egui_kittest` refactor — gap G4 |
| §10 Original UI + SVG assets | Met | `docs/design/brand.md`, `assets/` original SVG, `ATTRIBUTION.md` |
| §11 Tracking files | Met (2026-10-30) | all present, including `docs/capability-inventory.md` — gap G3 closed |
| §12 Repository layout | Met | workspace plus domain/data/ui split |
| §13 Stress-test mitigations | Met | v0.1.0 gate, PR checklist, early release workflow, human research gate, no local heavy builds |

## Part II — guide.md

| Rule | Status | Evidence / deviation |
| --- | --- | --- |
| II.18 Naming/branding | Met | `docs/product-name.md`, `docs/design/brand.md` |
| II.19 Legal depth | Met (2026-10-30) | clean-room, trademark, secrets and research rules met; **licence is dual MIT OR Apache-2.0** (`LICENSE-MIT` / `LICENSE-APACHE`, Cargo `license = "MIT OR Apache-2.0"`), Q-LIC-1 resolved in `docs/assumptions-and-questions.md` |
| II.19.5 Release checklist recorded in PROGRESS | **Deferred** | gap G5 (next release) |
| II.20 Security audit | Met | `SECURITY.md` threat model; `deny.toml`, `scripts/scan-secrets.sh`, CodeQL, SBOM |
| II.20.4 Authoritative checks in `.ci-required.txt` | Met (fixed 2026-10-27) | root `.ci-required.txt` added — gap G1. On pull requests it lists the always-present checks; **CodeQL is excluded from the PR verdict** because it is skipped on PRs, and it still runs on pushes to `main` (and on schedule), where it is read from the run status directly. `Dependency review` is PR-only, which is exactly the flow `ci.sh` uses |
| II.21 Accessibility | Adapted | `ACCESSIBILITY.md` + a11y PR checklist; full pass is tracked as F-605 (v0.3); egui/AccessKit limits stated honestly |
| II.22 Marketing / no false claims | Met | README/VISION wording; no affiliation or "drop-in replacement" claims |
| II.23 Maintainer sustainability | Met | `GOVERNANCE.md`, small slices, evidence-based roadmap |
| II.24.1 Per-slice DoD | Met | slice docs record exact CI runs in `PROGRESS.md` |
| II.24.1 No personal data / secret in the change | Met (gate added 2026-10-28) | `scripts/scan-personal-data.sh` + the required `Personal data scan` check; a historical home-path leak was removed in PR #37 (history purge still open — gap G7) |
| II.24.2 Per-phase DoD | Met | traceability ↔ plan consistent; user directives recorded in `DECISIONS.md` (Ctrl+Y 2026-10-27; the II.29 scan gate 2026-10-28); `docs/capability-inventory.md` present (G3 closed 2026-10-30) |
| II.24.3 Release/packaging | Met (asset scan added 2026-10-28) | multi-target CI artefacts + `Cargo.lock`; `release.yml` runs `scripts/scan-release-assets.sh` before publishing; round-trip tests |
| II.25 Launch readiness | Deferred (in part) | community files, branch protection, scanner gates present and now include the personal-data scan; capability inventory present (G3 closed); automated visual evidence is the remaining gap G4 |
| II.26 Open questions for the human | Open (one item) | II.26.1 local-test wording applied; II.26.6 licence choice open as `Q-LIC-1`; II.26.7 git-identity rule re-homed in `AGENTS.md` (G6 resolved 2026-10-30) |
| II.27 Designer role | Met (G4 deferred) | design system/parity via `docs/design/brand.md` + `docs/parity.md` + `docs/capability-inventory.md` (II.27.3, added 2026-10-30); II.27.12 visual evidence is gap G4 |
| II.27.12 Visual evidence for egui apps | **Deferred** | the guide lists Rubric's shell shape among those `egui_kittest` cannot drive today; needs the `Host`-seam refactor — gap G4. No fake harness is used |
| II.28 Naming conventions | Adapted | crate/binary names follow cargo; the release artefacts use `arm64` rather than the guide's `aarch64` — recorded as a `DECISIONS.md` deviation (II.28.3) |
| II.29 Personal-data/secrets scan gate | Met (added 2026-10-28) | four-part scan done; `scripts/scan-personal-data.sh` + `Personal data scan` required check; release-asset scan wired; the historical leak was purged from **all history** on 2026-10-28 (owner-approved `git filter-repo`, force-push, protection restored) — gap G7 resolved |

## Addendum — addendum.md

| Rule | Status | Evidence / deviation |
| --- | --- | --- |
| §A Client/agent role model | Met | plain-language try-steps; agent implements and merges per owner directive |
| §B Non-developer client playbook | Met | `PROGRESS.md` try-steps; `README.md` gains a no-Rust download path (gap G2) |
| §C Phased process (0–8) | Met | currently in the Phase 6 feature loop |
| §D MCP + tooling | Adapted | Context7 MCP available; GitHub operations run through the authenticated `gh` CLI rather than a GitHub MCP server (the server is not in this environment's catalog). Documented here rather than silently assumed |
| §E Task brief template | Met | `docs/tasks/*.md` |
| §F Release notes template | Met | release bodies follow Added/Changed/Fixed/Known limitations |
| §G Failure playbook | Met | CI fixes by editing source + re-running `ci.sh` |
| §H No unearned Craft parity | Met | ROADMAP/README make no such claim |
| §J Non-negotiable constraints | Met | no meta-guide push, no RE, no ROADMAP inflation, CI before client try |
| §K Rust autonomous QA | Adapted | no separate `crates/rubric-core`; the "same-layout equivalent" is the pure `src/domain` module inside `apps/rubric` with headless tests (DECISIONS "Domain-first architecture"). A future extraction is possible but not required for the gate |
| §K.11 README "try without Rust" | Met (fixed 2026-10-27) | README download-from-Releases section added — gap G2 |
| §K.13 Headless tests only | Met | domain/data tests use no GUI; PDF tests bind at run time |
| §M Demolition enforcements | Met | `rust-toolchain.toml`, blocking clippy, `cargo test --all`, no network in default tests, PR Rust QA section |
| §N Process feedback | Met | `docs/process-feedback.md` (append-only) |
| §O Priority card | Met | followed when context is tight |

---

## Gaps and their disposition

| # | Gap | Severity | Disposition |
| --- | --- | --- | --- |
| G1 | `.ci-required.txt` missing (II.20.4/II.25) | Slows-work | **Fixed 2026-10-27** — root file lists the checks that must decide `ci.sh`'s verdict |
| G2 | README had no "download, no Rust" path; repo layout and shortcut table were stale | Slows-work | **Fixed 2026-10-27** — README updated |
| G3 | `docs/capability-inventory.md` (II.27.3) absent | Slows-work | **Fixed 2026-10-30** — capability rows with source URLs, acceptance criteria and evidence, cross-linked from `docs/parity.md` |
| G4 | egui visual evidence (`egui_kittest`, II.27.12) needs a `Host`-seam refactor | Defers a gate | Planned refactor slice — added to `ROADMAP.md` v0.3; no fabricated harness is used in the meantime |
| G5 | §2.5/II.19.5 release legal checklist not yet recorded per release | Blocks a release only | Run and record in `PROGRESS.md` with the CI run ID when v0.2.0 is tagged |
| G6 | The git-identity rule has no home in the revised guide (II.26.7) | Client decision | **Resolved 2026-10-30** — owner directed the rule be re-homed; recorded in `AGENTS.md` (commits/PRs/merges/tags authored `Rubric Dev <dev@example.com>`; PR authorship follows the pushing account) |
| G7 | A maintainer absolute home path was committed and published (II.29.4) | **Resolved 2026-10-28** | Two carriers: `.opencode/agents/*.md` (removed in PR #37) and a tracked `scripts/__pycache__/*.pyc` (untracked, `*.pyc` ignored). Owner approved a purge; every branch, tag and commit was rewritten with `git filter-repo`, force-pushed, and branch protection restored. Residual risk (GitHub cached views/forks) recorded in `SECURITY.md` |

No rule in `guide.md` or `addendum.md` was edited: both remain local and
untracked, per their own "never push" rule and addendum §N.1.
