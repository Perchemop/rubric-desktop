# Development — Workflow and Conventions

## Constraints (facts)

- The local machine is memory-constrained; **all heavy work runs in GitHub
  Actions** (user directive 2026-10-19, AGENTS.md "CI-first").
- The repository is public with unlimited CI minutes.
- Subagents run on free OpenCode model tiers only.

## Local vs CI

**No `cargo` command runs on this machine — ever** (user directive
2026-10-24, superseding the earlier "fmt locally is fine" exception). Format
check, clippy, tests, and builds are all CI-only. When CI reports a fmt or
clippy failure, fix the source by hand and push again.

| Task | Where |
|---|---|
| Editing, reading, docs | Local |
| Read-only `git status/diff`, `gh run view` | Local |
| `cargo fmt --check`, `clippy`, `test`, release build | **CI only** — never locally |

## Task workflow (guide §5, §8)

Every change is a **Task** in the CTO.new-style cycle
`Chat (plan) → Task (execute) → Pull Request (review) → Merge`:

1. **Chat / plan** — pick the next unfinished slice (`PROGRESS.md`,
   `IMPLEMENTATION_PLAN.md`) and write a Task brief (requirement IDs, files
   likely touched, acceptance criteria, definition of done).
2. **Task / execute** — implement via the subagent pipeline in
   `.opencode/agents/`: `coder` plus the stage specialists it needs
   (`domain-dev` → `test-writer` → `ui-dev` → `docs-writer`), with `fixer`
   handling fmt/clippy/test failures. Subagents never commit or push; the parent
   agent does.
3. Commit a small focused Task, push, and open/update a
   **Pull Request** (the CI runner drives this).
4. **Review** — GitHub Actions (the free Task Runner) runs fmt → clippy
   `-D warnings` → `cargo test --all` → release build; the `reviewer` subagent
   and the human review the PR. Route failures to `fixer`.
5. **Merge** — the **human** merges once satisfied (trust but verify, guide
   §9). Record exact results in `PROGRESS.md` / `CHANGELOG.md`; update
   `REQUIREMENTS_TRACEABILITY.md`, `docs/parity.md` and `ROADMAP.md`. Never claim a feature is
   done before CI is green.

## Definition of Done (per Task)

See guide §8 and `docs/qc/checklist.md`: requirements implemented, tests
covering normal + failure paths, CI green, **Pull Request opened/updated for
review** (using `.github/PULL_REQUEST_TEMPLATE.md`), ROADMAP/CHANGELOG/parity
updated when a tracked item completes, no fake success states, change small and
reviewable, and no heavy work run locally.

## Testing

- Unit tests live beside the code in `src/domain/*` and `src/data/*`;
  integration tests in `tests/domain_tests.rs`.
- Headless tests must not require a display.
- PDF-rendering tests hard-assert when `RUBRIC_PDFIUM_PATH` is set (CI sets
  it after fetching PDFium) and skip gracefully when unset locally.

## Environment variables

- `RUBRIC_PDFIUM_PATH` — directory (or file) of the PDFium shared library
  for `data::pdf` (CI sets `/home/runner/pdfium/lib`).

## Originality

Rubric's code, docs and assets are original work, derived from public
standards, our own spec (`docs/spec/README.md`) and capability-level
inspiration (`docs/inspiration.md`). Third-party dependencies are
permissively licensed and listed in `ATTRIBUTION.md`.
