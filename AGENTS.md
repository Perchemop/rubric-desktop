# AGENTS.md

This document provides guidance for AI coding agents working on **Rubric**.

## Purpose
Build a free, cross-platform Rust PDF markup/measurement desktop app (egui/eframe). Adhere to the build guide (kept outside the repo, never committed/pushed); other products may be named only as capability-level inspiration in `docs/inspiration.md`. Maintain quality gates.

## Operating model (CTO.new-style free process, guide §5)

Every unit of work follows one cycle, emulating cto.new but entirely on free
models and GitHub Actions:

    Chat (plan)  →  Task (execute)  →  Pull Request (review)  →  Merge

- **Chat (planning)** — `orchestrator` clarifies the goal, writes a Task brief
  (requirement IDs, files likely touched, acceptance criteria, definition of
  done) and updates `IMPLEMENTATION_PLAN.md`. It never edits application code
  and never runs heavy builds.
- **Task (execution)** — `coder` (with the stage specialists it needs) makes one
  small, bounded vertical slice, updates tests and tracking files, then commits
  and pushes. GitHub Actions is the free **Task Runner** that verifies it.
- **Pull Request (review)** — the change lands as a PR; the CI run plus the
  `reviewer` subagent and the human review it (trust but verify, guide §9).
- **Merge** — the **agent merges** once CI is green and review findings are
  addressed (owner directive 2026-10-26). The owner may still ask to review a
  PR first; otherwise do not wait for a manual merge.
- **PR template** — every PR uses `.github/PULL_REQUEST_TEMPLATE.md` (acceptance criteria, tests, CI, no proprietary material); reviewers reject PRs that fail its checklist (guide §8.3).
- **Roadmap** — `ROADMAP.md` tracks staged milestones and readiness %; percentages move **only from verified evidence** (guide §6.2).
- **Research gate** — public research lives in `docs/research/*.md` as original summaries + URLs, and needs explicit human approval before dependent implementation Tasks (guide §2.3, §8.4).

## Codebase rules
- Rust only, 100% free OSS.
- Domain-first: `src/domain/` is pure (no UI), fully serialisable with serde, testable headlessly.
- Formatting: rustfmt conventions, checked by CI. No trailing whitespace.
- Linting: `cargo clippy -- -D warnings` (treat warnings as errors).
- Testing: `cargo test` before committing. Add/extend tests when changing behaviour or fixing bugs.
- Error handling: use `thiserror` for domain/data errors; avoid panics in logic; prefer `Result`.
- Conventions: snake_case, modules flat and explicit; doc comments for public API.
- Dependencies: prefer well-maintained, MIT/Apache-2.0 crates.
- UI: egui/eframe; viewport with app icon; responsive panels; keyboard shortcuts (Ctrl+S save, Ctrl+Z undo, Ctrl+Shift+Z redo, Esc/Del).
- Persistence: JSON (serde_json), `.rubric.json` extension, human-readable; include UUIDs and timestamps.
- Git: fresh repo, never commit external reference material; commit small, focused slices; messages imperative and descriptive.

## Testing strategy
- Unit tests in `src/domain/*` modules and `tests/domain_tests.rs`.
- Headless tests must not require a display. UI is integration surface; logic lives in domain.
- Roundtrip tests: serialise → deserialise and assert equality.

## Implementation guidance
- Build in vertical slices; update PROGRESS.md, CHANGELOG.md, DECISIONS.md, REQUIREMENTS_TRACEABILITY.md as you go.
- When adding features: define domain types first, add tests, then UI.
- Measurements: recompute on geometry change using page scale; store `value_points` to enable rescaling.
- Hit testing: use tolerance; prefer segment/bounds hit tests per geometry.
- Undo/redo: project snapshots in `History` (bounded to 128). Push snapshot on destructive/committed actions; avoid spamming during live drags.

## Subagent pipeline
Specialised subagents live in `.opencode/agents/`: `orchestrator` (Chat/planning)
and `coder` (Task execution) are the process roles; the build is split by stage
into `domain-dev` (types + unit tests) → `test-writer` (coverage) → `ui-dev`
(egui slices) → `reviewer` (read-only review) → `docs-writer` (process docs) →
`fixer` (fmt/clippy/check failures), with `ci-runner` driving the GitHub Actions
Task Runner. All run on free OpenCode models in `subagent` mode. Conventions:
subagents never commit or push — the parent agent does, and it opens the Pull
Request.

## Suite infrastructure (guide §7)
`crates/crash-report` and `crates/update-check` are shared crates consumed by
the app; a future suite app reuses them rather than reinventing crash/update
behaviour.

- **crash-report**: installs the panic hook, captures app version / OS /
  scrubbed backtrace, shows a review-and-cancel dialog, then either creates a
  GitHub issue with a **user-supplied** token or opens a pre-filled New Issue
  URL. A privileged bot token must never be embedded in the client.
- **update-check**: queries the GitHub Releases API, compares against the
  running version, fails soft offline, and caches briefly (no API abuse).

## CI-first (no heavy local runs)
**No cargo command is ever run on the local machine — not even `cargo fmt`,
`cargo check`, or a "small" test.** All Rust toolchain work (format check,
lint, tests, builds, packaging) runs in **GitHub Actions** — the free cloud
Task Runner. The desktop host is memory-constrained and must never execute the
toolchain. Formatting problems reported by CI are fixed by editing files by
hand and re-running CI.

Workflow: edit → commit → push → open/update a **Pull Request** → read the CI
run (`gh run view <id>`) → fix failures by editing the source → push again.
The human reviews and merges the Pull Request. Every PR runs fmt check,
clippy (`-D warnings`), full tests, and a release build.

## Local machine (hard rule)

- Read-only git, file editing, `gh`, and reading CI logs are the only local
  activities.
- **Never run any `cargo` command locally** (no fmt/check/test/build/clippy),
  and never launch the GUI here as part of "testing" more than a quick
  user-driven check the human asks for.
- If CI reports a formatting or lint failure, fix it by editing the source and
  pushing again — CI is both the checker and the fixer of record.

## Do not
- Commit external reference material or notes about other products;
  capability-level inspiration belongs only in `docs/inspiration.md`.
- Add anything that is not original work. All code, docs and assets must be
  original (public standards + our own spec + capability-level inspiration
  only).
- Introduce AGPL/GPL dependencies without approval.
- Mix UI and business logic.
- Push without running quality gates.
- Leave a green, review-clean PR unmerged unless the owner asked to review it
  first (owner directive 2026-10-26: agents merge completed PRs).

## Completion mandate (mandatory)

Every agent in this repository works **each slice and each step through to
overall completion of the application**, not to a convenient stopping point.

- The queue is the repository's own records: `ROADMAP.md`,
  `IMPLEMENTATION_PLAN.md`, `PROGRESS.md`, `REQUIREMENTS_TRACEABILITY.md`. If an
  in-scope item is unfinished, the work is unfinished.
- Do not pause between slices to ask whether to continue. Continue.
- Never present a summary as though the work were finished while items remain.
- The application is complete only when the quality gates pass (guide Part I §9,
  the §2.5 per-release legal checklist, and CI green on the default branch).

## Questions and user input — always via the question tool

If you need a decision, clarification, or input from the owner, **present it
with the `question` tool**. Never bury a question in prose, never assume an
answer, and never stall silently.

Ask only when genuinely blocked (an unconfirmed destructive action, two
readings of the requirement that imply different implementations, or a fact
that exists nowhere in the repo or the docs). Offer concrete options with a
recommendation first, then act on the answer and carry on.
