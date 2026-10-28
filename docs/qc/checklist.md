# QC Checklists (guide §9)

Run these in order per phase / release. Record each run (date, phase, CI
run ID, outcome) in `PROGRESS.md`; never claim completion without a green
CI run.

## Per Task (Definition of Done)

- [ ] Task is one vertical slice (domain + tests + UI + docs).
- [ ] Unit tests cover happy path, boundary, and failure/edge cases.
- [ ] Headless domain tests need no display; no CPU-heavy work runs locally.
- [ ] CI reports the fmt check green (never run locally — no cargo on this machine).
- [ ] CI green on GitHub Actions (the free Task Runner): fmt → clippy
      `-D warnings` → `test --all` → release build.
- [ ] Pull Request opened/updated for review using
      `.github/PULL_REQUEST_TEMPLATE.md`; the **agent merges once CI is green
      and review findings are addressed** (owner directive 2026-10-26,
      `AGENTS.md`, `DECISIONS.md`), unless the owner asked to review first.
- [ ] `ROADMAP.md` readiness % updated when a tracked item completes — from
      evidence only (CI run or recorded hands-on check).
- [ ] No fake success states / dead controls / placeholder-only data.
- [ ] `CHANGELOG.md`, `PROGRESS.md` updated with **exact** CI results.
- [ ] `REQUIREMENTS_TRACEABILITY.md` statuses match reality.
- [ ] Change is small and reviewable (per guide §8).
- [ ] UI changes only: the PR template's accessibility items are checked
      (keyboard path, visible focus, announced labels/errors, non-colour
      signals, reduced motion) — see `ACCESSIBILITY.md`.
- [ ] New dependencies: allowed licence class (`deny.toml` gate), recorded
      in `ATTRIBUTION.md`; no secrets in the diff (secret scan clean).
- [ ] No personal data in the change: `scripts/scan-personal-data.sh` is clean
      (no non-CI home path, no real email) and the `Personal data scan` check is
      green (guide II.29). Placeholder/seed values are obviously synthetic.

## Per phase

- [ ] All phase F-IDs traced → `Implemented` with CI evidence.
- [ ] `docs/spec/README.md` F-IDs ↔ `IMPLEMENTATION_PLAN.md` ↔
      `REQUIREMENTS_TRACEABILITY.md` consistent (no orphan IDs).
- [ ] Open assumptions reviewed in `docs/assumptions-and-questions.md`
      (answer or mark still-open with Q-IDs).
- [ ] DECISIONS.md has an entry for every user directive this phase.
- [ ] `cargo doc`/public APIs: no warnings introduced; clippy clean.

## Release / packaging

- [ ] Cross-platform build artifacts verified (Linux/Windows/macOS).
- [ ] App launches on a clean machine; sidecar round-trip verified
      (save → reopen → identical state).
- [ ] Version bumped; CHANGELOG has a release entry.
- [ ] No secrets/keys committed (`git log -p` spot check); no local-only
      files in the tree (`.gitignore` respected).
- [ ] LICENSE present; `ATTRIBUTION.md` complete and accurate.

## Originality attestation (every phase)

- [ ] All code, strings, icons, layouts and algorithms are original —
      derived from the original spec (`docs/spec/README.md`), public
      standards, and capability-level inspiration (`docs/inspiration.md`).
- [ ] No proprietary code or assets; third-party dependencies listed in
      `ATTRIBUTION.md`.
- [ ] No absolute local paths or references to local-only files in code/docs.

## Legal checklist (guide §2.5, before each public release)

- [ ] Copyright: no proprietary source/assets or substantial copied text in tree
- [ ] Clean-room: no RE tools or output used as implementation input
- [ ] Trademarks: product name/logo original; no vendor marks in UI or installers
- [ ] Trade dress: UI is original expression, not a slavish visual copy
- [ ] Patents: public non-infringing alternatives preferred; public standards used
- [ ] Licence: MIT clear; NOTICE/ATTRIBUTION complete
- [ ] Secrets: no keys/tokens in git or release binaries; crash reports use user-supplied tokens only
- [ ] Personal data (guide II.29): no real name/email/phone/address/account id,
      no non-CI home path, in the tree, history, metadata or release assets;
      the four-part scan is recorded in `PROGRESS.md` with the CI run ID
- [ ] Privacy: crash reports send only what the user consents to; documented in README
- [ ] Research files: summaries + URLs only; human-approved (`docs/research/`)
- [ ] Claims: no false certification/affiliation/"drop-in replacement" wording
- [ ] Contributors: `CONTRIBUTING.md` states PR licence terms

## Manual QC spot-checks (when UI is touched)

- [ ] Every control does something real; keyboard shortcuts work
      (checklist of shortcuts: guide §9 shortcut table — see spec F-601).
- [ ] Status bar / messages never claim success not backed by a real
      completed operation (no async theatre).
- [ ] Colors/contrast follow `assets/branding.md`; dark mode sanity if
      implemented.
- [ ] Undo/redo never exceeds the 128-snapshot cap; no history entries for
      ephemeral UI state.

### Run log

| Date | Scope | CI run | Outcome |
|---|---|---|---|
| 2026-10-18 | Phase 0 | `39121051225` | Pass (8/8 steps) |
