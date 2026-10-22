# Attribution — Third-Party Components

Rubric itself is **MIT licensed** (`LICENSE`); all first-party source code,
tests, docs, and branding assets are original work.

## Direct dependencies (all free/open-source)

| Crate | Version | Licence | Use |
|---|---|---|---|
| `eframe` | 0.36 | MIT OR Apache-2.0 | App framework / windowing |
| `egui` | 0.36 | MIT OR Apache-2.0 | Immediate-mode UI |
| `egui_extras` | 0.36 | MIT OR Apache-2.0 | UI table/layout helpers |
| `image` | 0.25 | MIT OR Apache-2.0 | PNG encode/decode (page render cache, export) |
| `serde` / `serde_json` | 1.x | MIT OR Apache-2.0 | `.rubric.json` persistence |
| `uuid` | 1.x | MIT OR Apache-2.0 | Markup/layer identifiers (v4) |
| `chrono` | 0.4 | MIT OR Apache-2.0 | UTC timestamps |
| `thiserror` | 2.x | MIT OR Apache-2.0 | Domain/data error types |
| `tracing` / `tracing-subscriber` | 0.1 / 0.3 | MIT OR Apache-2.0 | Diagnostics |
| `directories` | 6.x | MIT OR Apache-2.0 | OS config/data directories |
| `rfd` | 0.15 | MIT | Native file dialogs (xdg-portal backend on Linux) |
| `pdfium-render` | 0.9 | MIT OR Apache-2.0 | Rust bindings to PDFium (runtime binding only) |

Full transitive tree is resolved in `Cargo.lock`. AGPL/GPL dependencies are
prohibited (AGENTS.md); all crates above are permissively licensed.

## Runtime component: PDFium

- **What**: PDFium shared library (`libpdfium.so` / `pdfium.dll` /
  `libpdfium.dylib`) used to open and rasterise PDF pages.
- **Licence**: BSD-3-Clause (Chromium project). Source:
  <https://github.com/bblanchon/pdfium-binaries> (pre-built binaries).
- **How it ships**: *Never* bundled with source code and *never* linked at
  compile time. `pdfium-render` binds at run time; CI and developers fetch
  the library on demand (see `docs/development.md`, `RUBRIC_PDFIUM_PATH`).
  Distribution packages that include the library must carry its BSD-3
  licence text alongside it.
- **Rationale**: licence-compatible (BSD-3 is permissive), keeps builds
  pure Rust (decision: `DECISIONS.md` 2026-10-20).

## Fonts and assets

- No third-party font binaries are shipped. UI text uses egui's built-in
  fonts (MIT/Apache-2.0) and system fonts; branding typography suggestions
  in `assets/branding.md` are optional references, not bundled files.
- `assets/logo.svg`, `assets/icons/*` and the generated icon set are
  original Rubric artwork (`scripts/gen-icons.sh`).
