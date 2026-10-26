## Summary
<!-- What this PR does in 1–3 sentences -->

## Type
- [ ] Feature
- [ ] Fix
- [ ] Docs / research (summaries + URLs only)
- [ ] Infra (CI, release, shared crates)
- [ ] Refactor

## Checklist
- [ ] Acceptance criteria for the Task are met with **real** behaviour (no fake UI)
- [ ] Tests added or updated
- [ ] CI is green (or will be after merge of required base)
- [ ] Verification is done in CI; no full builds run locally on a constrained machine
- [ ] No proprietary source, assets, or RE material introduced
- [ ] Public research notes are original summaries + URLs only (if applicable)
- [ ] ROADMAP / CHANGELOG / parity updated if this completes a tracked item
- [ ] Secrets and tokens not committed
- [ ] User-facing strings and UI are original (no copied trade dress)
- [ ] Radar pass done for the capability this PR builds (guide 4A):
      adopt / pilot / watch / reject recorded, or a "no change" line added
      to `docs/repo-radar.md`
- [ ] UI changes only: keyboard path works; focus visible and logical;
      labels/errors announced; colour is not the only signal; reduced-motion
      respected; screen-reader spot check where a platform offers one
      (details in ACCESSIBILITY.md)

## Rust QA (addendum §M.3)
- [ ] Logic lives in the domain module / core crates (no business logic in
      UI paint code), or the exception is justified here
- [ ] Workflow/regression test added or updated (name: `...`)
- [ ] Tests are headless (no GUI / display / real network)
- [ ] No full release build was run on the client laptop
- [ ] CI green on this branch

## Local verification
<!-- How a human can try this on the primary OS -->

## Notes / risks
