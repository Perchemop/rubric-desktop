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
- **Merge** — the **human** merges. Agents prepare PRs; they do not self-merge.

## Codebase rules
- Rust only, 100% free OSS.
- Domain-first: `src/domain/` is pure (no UI), fully serialisable with serde, testable headlessly.
- Formatting: `cargo fmt --all -- --check` and formatted code. No trailing whitespace.
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

## CI-first (no heavy local runs)
All CPU/RAM-intensive work runs in **GitHub Actions** — the free cloud Task
Runner — never on the desktop host (it is memory-constrained and restarts under
load): `cargo test`, `cargo clippy`, `cargo check`, and release builds are
CI-only. Locally, run just `cargo fmt --all` (trivial) and read-only git.
Workflow: edit → commit → push → open/update a **Pull Request** → read the CI
run (`gh run view <id>`) → route any failures back through the `fixer`
subagent. The human reviews and merges the Pull Request. Every PR runs fmt
check, clippy (`-D warnings`), full tests, and a release build.

## Do not
- Commit external reference material or notes about other products;
  capability-level inspiration belongs only in `docs/inspiration.md`.
- Add anything that is not original work. All code, docs and assets must be
  original (public standards + our own spec + capability-level inspiration
  only).
- Introduce AGPL/GPL dependencies without approval.
- Mix UI and business logic.
- Push without running quality gates.
- Self-merge a Pull Request: open it for review and let the human merge
  (guide §5, §9).
