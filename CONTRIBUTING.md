# Contributing to Rubric

Thank you for considering a contribution. Rubric is a free, open-source PDF
markup/review/measurement tool built as original work.

## Licence terms for contributions

By opening a pull request you agree that your contribution is licensed under
the project licence — **MIT** (`LICENSE`) — via a DCO-style sign-off:

```
Signed-off-by: Your Name <your.email@example.com>
```

(`git commit -s` adds this line.) This certifies that you wrote the change or
have the right to submit it, and that you licence it to the project on those
terms. No CLA is required.

> Note: dual-licensing the project as **MIT OR Apache-2.0** (the Rust
> ecosystem convention) is recorded as open question **Q-LIC-1** in
> `docs/assumptions-and-questions.md`; until it is decided, the single
> MIT licence applies everywhere — keep crate metadata and docs consistent
> with that.

## Ground rules

- **Original work only.** Do not submit code, assets, icons, text, or UI
  copied from commercial products, and do not submit material derived from
  reverse engineering. Public documentation and open standards are fine
  references; cite them as URLs.
- **No secrets.** Never commit API keys, tokens, or credentials.
- **Small, focused PRs.** One logical change per pull request, using the PR
  template (`.github/PULL_REQUEST_TEMPLATE.md`).
- **Tests.** Behaviour changes ship with tests (domain unit tests headlessly;
  integration tests in `tests/`).
- **CI is the verifier.** `cargo fmt --all -- --check`, `cargo clippy
  -- -D warnings`, and `cargo test --all` run on GitHub Actions for every PR.
  Do not run full builds locally on constrained machines.

## Development setup

```bash
git clone https://github.com/Perchemop/rubric-desktop.git
cd rubric-desktop
cargo build            # debug build
cargo test --all       # tests
cargo fmt --all        # formatting
```

Optional, for PDF import: place the PDFium shared library beside the built
executable or set `RUBRIC_PDFIUM_PATH` — see `docs/pdfium.md`.

## Reporting bugs and crashes

Use GitHub Issues. The in-app crash reporter (Help ▸ Report a Bug) opens a
pre-filled issue; you can edit or cancel before anything is sent, and it never
requires a token to be embedded in the app. See the privacy note in `README.md`.

- Bugs: use the **bug report** template.
- Accessibility barriers: use the **accessibility report** template (see
  `ACCESSIBILITY.md`) — these are triaged at bug priority.
- Security vulnerabilities: **never** a public issue — follow `SECURITY.md`
  (private reporting).

## Conduct and support

- Behaviour expectations and the report path: `CODE_OF_CONDUCT.md`.
- How decisions are made: `GOVERNANCE.md`; what the product is for:
  `VISION.md`.
- Every merged contribution is credited in `CONTRIBUTORS.md`.

## What to expect

- Issues are triaged regularly; accessibility reports are treated with the
  same priority as high-severity bugs.
- PRs are reviewed by CI + the reviewer subagent; the agent merges once CI is
  green and findings are addressed (owner directive 2026-10-26).
- Security vulnerabilities are handled privately per `SECURITY.md` (no public
  issues). Typical first response is within 7 days.

## Questions


Open a discussion or issue — the roadmap in `ROADMAP.md` shows what is planned
and what is already done.
