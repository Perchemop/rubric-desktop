# Development — Workflow and Conventions

## Constraints (facts)

- The local machine is memory-constrained; **all heavy work runs in GitHub
  Actions** (user directive 2026-10-19, AGENTS.md "CI-first").
- The repository is public with unlimited CI minutes.
- Subagents run on free OpenCode model tiers only.

## Local vs CI

| Task | Where |
|---|---|
| Editing, reading, docs | Local |
| `cargo fmt --all` (+ `--check`) | Local (trivial) |
| Read-only `git status/diff`, `gh run view` | Local |
| `cargo check`, `clippy`, `test`, release build | **CI only** — never locally |

## Slice workflow (guide §8)

1. Pick the next unfinished vertical slice (`PROGRESS.md`).
2. Implement via the subagent pipeline in `.opencode/agents/`:
   `domain-dev` (types + unit tests) → `test-writer` → `ui-dev` →
   `reviewer` (read-only) → `docs-writer` → `fixer` (fmt/clippy/test
   failures). Subagents never commit or push; the parent agent does.
3. `cargo fmt --all`, commit a small focused slice, push to `main`.
4. Watch CI (`gh run view <id>`): fmt → clippy `-D warnings` →
   `cargo test --all` → release build. Route failures to `fixer`.
5. Record exact results in `PROGRESS.md` / `CHANGELOG.md`; update
   `REQUIREMENTS_TRACEABILITY.md`. Never claim a feature is done before CI
   is green (guide §9).

## Definition of Done (per slice)

See guide §8 and `docs/qc/checklist.md`: requirements implemented, tests
covering normal + failure paths, CI green, tracking docs updated, no fake
success states, change small and reviewable.

## Testing

- Unit tests live beside the code in `src/domain/*` and `src/data/*`;
  integration tests in `tests/domain_tests.rs`.
- Headless tests must not require a display.
- PDF-rendering tests hard-assert when `RUBRIC_PDFIUM_PATH` is set (CI sets
  it after fetching PDFium) and skip gracefully when unset locally.

## Environment variables

- `RUBRIC_PDFIUM_PATH` — directory (or file) of the PDFium shared library
  for `data::pdf` (CI sets `/home/runner/pdfium/lib`).
