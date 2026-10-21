# QC Checklists (guide §9)

Run these in order per phase / release. Record each run (date, phase, CI
run ID, outcome) in `PROGRESS.md`; never claim completion without a green
CI run.

## Per slice (Definition of Done)

- [ ] Feature slice is vertical (domain + tests + UI + docs in one slice).
- [ ] Unit tests cover happy path, boundary, and failure/edge cases.
- [ ] Headless domain tests need no display; no CPU-heavy work runs locally.
- [ ] `cargo fmt --all -- --check` passes locally.
- [ ] CI green on GitHub Actions: fmt → clippy `-D warnings` → `test --all`
      → release build.
- [ ] No fake success states / dead controls / placeholder-only data.
- [ ] `CHANGELOG.md`, `PROGRESS.md` updated with **exact** CI results.
- [ ] `REQUIREMENTS_TRACEABILITY.md` statuses match reality.
- [ ] Change is small and reviewable (per guide §6).

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

## Manual QC spot-checks (when UI is touched)

- [ ] Every control does something real; keyboard shortcuts work
      (checklist of shortcuts: guide §9 shortcut table — see spec F-601).
- [ ] Status bar / messages never claim success not backed by a real
      completed operation (no async theatre).
- [ ] Colors/contrast follow `assets/branding.md`; dark mode sanity if
      implemented.
- [ ] Undo/redo never exceeds the 128-snapshot cap; no history entries for
      ephemeral UI state.
