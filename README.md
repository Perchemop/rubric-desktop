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

## License

MIT License. See [LICENSE](LICENSE).
