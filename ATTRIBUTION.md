# Attribution — Third-Party Components

Rubric itself is **dual-licensed under MIT OR Apache-2.0**
(`LICENSE-MIT` / `LICENSE-APACHE`); all first-party source code,
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
| `ureq` | 2.x | MIT OR Apache-2.0 | HTTP client for update checks (sync) |
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

- UI text uses **egui's bundled fonts** (`epaint_default_fonts` crate): a
  small set of open-licence fonts (OFL-1.1 and Ubuntu Font Licence v1.0
  terms) carried inside the egui/eframe dependency tree — recorded here per
  the asset-attribution rule; not shipped as separate files by Rubric, and
  permitted for bundling/redistribution by their terms. System fonts beyond
  those are not bundled. Branding typography suggestions in
  `assets/branding.md` are optional references, not bundled files.
- `assets/logo.svg`, `assets/icons/*` and the generated icon set are
  original Rubric artwork (`scripts/gen-icons.sh`).
- No Creative-Commons or "non-commercial" licensed assets are bundled
  anywhere in the tree.

## Dependency licence policy (Part II §19.2/§19.3 automated)

Licence classes are enforced in CI, not just documented:

- **Allowed:** permissive — MIT, MIT-0, Apache-2.0, ISC, BSD-2/3, Zlib,
  BSL-1.0, Unicode-3.0, CC0-1.0, CDLA-Permissive-2.0.
- **Allowed with conditions:** weak/limited copyleft — MPL-2.0.
- **Denied without maintainer approval:** strong copyleft (GPL-2.0,
  GPL-3.0, AGPL-3.0) and source-available/proprietary terms.
- Gate: `cargo-deny` runs on every push/PR (`.github/workflows/security.yml`
  against `deny.toml`), and PR-time `dependency-review` blocks high-severity
  advisories and denied licences. A software bill of materials (SPDX +
  CycloneDX JSON over `Cargo.lock`) is generated as workflow artefacts.

## AI tooling used during the build (Part II §19.7)

Recorded so redistribution rights are transparent:

- **Models:** free-tier hosted models only (OpenCode free tier and NVIDIA
  free tier; identifiers and role mapping live in `docs/current-model-map.md`).
  No local model hosting.
- **Terms:** provider terms of use apply to generation-time use; withheld
  outputs whose terms forbid redistribution are not committed. All source,
  documents and assets in this repository are treated as original project
  work authored for it, and met the same review bar as human contributions.
- **Provenance basis:** original code + public standards + capability-level
  inspiration only (`docs/inspiration.md`); no scraped corpora, no
  reverse-engineering output, no copied trade dress.
