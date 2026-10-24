# Rubric

**Rubric** is a free, cross-platform PDF markup, review, and measurement application written in Rust. It is a clean, original implementation focused on redlining, review workflows, and precise measurements.

## Features (Phase 0 + Slice 1)

- **Original, professionally branded** design (SVG logo, multi-size icons, polished UI skeleton).
- **Rust only, 100% free/open-source** — no proprietary dependencies; built on egui/eframe.
- **Cross-platform desktop** targets: Linux, macOS, Windows (same Rust codebase).
- **Domain-first architecture**: geometry, markups, measurements, documents, projects and history all pure Rust (easily unit-tested).
- **Markup tools**: Review (Highlight, Strikeout, Underline, Squiggly, Text, Callout, Note, Stamp), Shapes (Rectangle, Ellipse, Cloud, Polygon, Polyline), Paths (Line, Arrow, Ink) and Measurements (Length, Perimeter, Area, Angle, Diameter, Radius, Count).
- **Review workflow**: Status (Open / In review / Approved / Rejected), Subject, Author, Comments, Layer/Toolset grouping.
- **Measurements with scale**: Page scale (units/point), automatic recomputation for measure tools; displays values in chosen units (points/in/ft/mm/cm/m etc).
- **Selection, move/translate, undo/redo** (snapshots), hit testing, inspector for properties/comments/measurements.
- **Persistence**: Save/load projects as pretty-printed JSON (`.rubric.json`) with timestamps and UUIDs.
- **Tooling**: `cargo test`, `cargo clippy`, `cargo fmt`, CI-ready.

## Repository layout

- `src/domain/` — pure business/model logic (geom, units, style, markup, document, project)
- `src/data/` — persistence/errors (JSON store)
- `src/ui/` — egui UI (app, canvas, toolbar, inspector, theme, icons)
- `assets/` — logo and icons
- `docs/` — architecture, data model, test plan, assumptions and questions
- `tests/` — integration/domain tests

## Quick start

```bash
# clone repo (locally created)
cd ~/rubric

# build and run (headless systems can't open GUI; build at least works)
cargo build

# tests + lint
cargo test
cargo clippy -- -D warnings
cargo fmt --all -- --check
```

## Crash reports and update checks (privacy)

Rubric phones home **only when you ask it to**:

- **Crash / bug reports** (Help ▸ Report a Bug): the app captures your Rubric
  version, OS, and a backtrace with file paths scrubbed, and shows you the
  full text first. Nothing is sent until you press **Send**. Delivery is your
  choice: open a pre-filled GitHub issue in your browser (no account needed in
  the app), or paste a **personal** GitHub token, which is used once, kept only
  in memory, and never written to disk or embedded in the app.
- **Update check** (Help ▸ Check for Updates…): one HTTPS request to the
  GitHub Releases API, on demand only — never automatic. Offline or
  rate-limited, the app simply says it could not check.

No telemetry, no analytics, no background polling, no identifiers.

## Original development

Every line of source code, every test, every asset, and every document in
this repository is original work authored for this project. The
implementation is informed only by publicly documented standards (e.g. the
PDF page geometry conventions), the permissively licensed open-source
crates listed in `Cargo.toml`, Rubric's own original requirement set
(`docs/spec/README.md`), and capability-level feature inspiration recorded
in `docs/inspiration.md`. Dependency licences are tracked in
`ATTRIBUTION.md` (project licence: MIT).

## Community and project docs

| File | What it covers |
|---|---|
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to report bugs, propose features, set up and test |
| [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) | Behaviour expectations and report path |
| [`SECURITY.md`](SECURITY.md) | Private vulnerability reporting, threat model, incident response |
| [`ACCESSIBILITY.md`](ACCESSIBILITY.md) | Accessibility commitment, known limits, barrier reports |
| [`VISION.md`](VISION.md) | What Rubric is for — and what it is not |
| [`GOVERNANCE.md`](GOVERNANCE.md) | Who decides what, and how |
| [`CONTRIBUTORS.md`](CONTRIBUTORS.md) | Everyone credited |
| [`ROADMAP.md`](ROADMAP.md) | Staged plan with evidence-based readiness |

## License

MIT License. See [LICENSE](LICENSE).
