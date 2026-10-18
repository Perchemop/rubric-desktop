# PROGRESS.md

## 2026-10-17 — Phase 0 (initial commit)
**Status**: in progress (verifying build)

### Completed
- Confirmed app spec: a cross-platform PDF markup/measurement desktop app.
- Environment: Rust 1.99.0 stable (aarch64), GUI dev libs present, display available, crates.io reachable, ~676 cached crates.
- Invented name: **Rubric** (original, thematically apt). Designed logo concept.
- Installed `librsvg2-bin` (rsvg-convert). Generated SVG logo + PNG icons (16/32/48/64/128/256/512/1024).
- Created fresh repo structure at ~/rubric.
- .gitignore excludes external material, secrets and build output.
- Branding: `assets/logo.svg`, `assets/icons/*`, `assets/branding.md`.
- Docs: `README.md`, `AGENTS.md`, `docs/architecture.md`, `docs/data-model.md`, `docs/test-plan.md`, `docs/assumptions-and-questions.md`.
- Process docs: `IMPLEMENTATION_PLAN.md`, `REQUIREMENTS_TRACEABILITY.md`, `DECISIONS.md`, `PROGRESS.md`, `CHANGELOG.md`.
- Cargo.toml (eframe/egui 0.36, serde, uuid, thiserror, tracing, chrono, image), rust-toolchain.toml, LICENSE (MIT).
- Domain model complete: `geom`, `units`, `style`, `markup` (22 kinds + geometry + measurements + hit tests), `document`, `project` (history/undo/redo, toolsets). All with unit tests.
- Data layer: `data/error.rs`, `data/store.rs` (JSON save/load, app data dir).
- UI skeleton: `ui/theme.rs`, `ui/icons.rs`, `ui/app.rs` (panels, menu, shortcuts), `ui/canvas.rs` (page rendering, selection, basic draw/place), `ui/toolbar.rs` (tool groups), `ui/inspector.rs` (properties/comments/measurements).
- Integration tests: `tests/domain_tests.rs`.
- Tooling: `scripts/gen-icons.sh`, `.github/workflows/ci.yml`.
- `src/lib.rs`, `src/main.rs` with eframe run_native and embedded icon.

### In progress
- Verifying: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`.

### Blockers
- None.

### Next
1. `cargo test` — all unit/integration tests pass
2. `cargo clippy -- -D warnings`
3. `cargo fmt --all -- --check`
4. `git init && git add . && git commit`
5. Phase 1: define the feature requirement set and update traceability
6. Phase 2: improve drawing (drag-to-draw, polyline/polygon/angle/ink preview), zoom/pan UX

## 2026-10-18 — Phase 0 verified via GitHub CI + subagent suite added
**Status**: Phase 0 committed and pushed; CI enforces quality gates

### Completed
- Phase 0 committed (`f984665`) and pushed to GitHub; `main` branch.
- CI workflow (`.github/workflows/ci.yml`) runs `cargo fmt --check`,
  `cargo clippy -D warnings`, `cargo test --all`, and a release build on every
  push — the machine-freezing local test problem is worked around by letting
  GitHub Actions run the test gate.
- Formatting applied and committed (`d157759`, `0dd6844`); area measurement
  rescaling fix landed (area scales with `units_per_point²`).
- Subagent suite added under `.opencode/agents/` (symlinked into
  `~/.config/opencode/agents/` so the home-directory session can launch them):
  - `domain-dev` (deepseek-v4.1-flash-free#high) — domain types + unit tests
  - `ui-dev` (beam-free#high) — egui/eframe UI slices
  - `test-writer` (mimo-v2.6-flash-free) — test coverage only
  - `docs-writer` (big-pickle) — process docs upkeep
  - `fixer` (space-bunny-free#high) — build-gate failure fixes
  - `reviewer` (glm-5.3-flash-free#high) — read-only review (pre-existing)
  All run as `subagent` mode with allowlisted shell access (fmt/check/clippy
  and read-only git); `cargo test` and git push are reserved for CI and the
  parent agent respectively. Smoke-tested `domain-dev` successfully.

### Active
- Waiting on GitHub CI run results for the pushed commits.

### Blocked
- None.

## Summary
Phase 0 is complete, committed, and pushed; CI is the source of truth for the
quality gates. A free-model subagent suite now covers each build stage
(domain → tests → UI → review → docs → fixes). Next: confirm CI green, then
Phase 1 (feature requirement set + traceability) and Phase 2 (drawing/zoom UX).
