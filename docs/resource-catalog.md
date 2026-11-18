# External resource catalog adoptions

Recorded under guide §25 use rule 2. The guide §25 catalogs are discovery
surfaces, not implementation sources; every adoption is recorded here with its
catalog, source, licence, pinned version/date and the decision that approved it
before use.

| Catalog | Source | Licence | Version-or-date | Decision (DECISIONS.md ref) |
|---|---|---|---|---|
| | | | | |

No adoptions yet; every future adoption from the guide §25 catalogs is recorded
here before use.

## Tooling already in force (not adopted from the §25 catalogs)

These two components predate the catalogs and are recorded here for
completeness. Licences and versions come from `ATTRIBUTION.md` (the PDFium
binary's pinned build is confirmed in the CI/packaging workflows).

| Component | Source | Licence | Version-or-date | Decision (DECISIONS.md ref) |
|---|---|---|---|---|
| PDFium (bundled runtime library) | `bblanchon/pdfium-binaries` prebuilt binaries; fetch pinned to `chromium/7881` in CI and packaging | BSD-3-Clause (Chromium project) | Chromium build 7881 (`pdfium_7881` bindings) | `DECISIONS.md` 2026-10-20 (runtime-binding choice); `pdfium-render` binding pin 0.9.x |
| hayro (pure-Rust PDF renderer, used for aarch64 and other platforms where PDFium binaries are hard to distribute) | `hayro` / `hayro-syntax` crates (`LaurenzV/hayro`) | MIT OR Apache-2.0 | 0.9 (`ATTRIBUTION.md`) | `DECISIONS.md` 2026-10-31 (adopt a pure-Rust PDF engine) |
