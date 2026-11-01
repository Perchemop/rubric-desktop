# PDF engine

Rubric renders PDF pages through a **pluggable PDF engine**:

1. **PDFium** (default) — linked at run time (never at build time — see
   `Cargo.toml` and DECISIONS.md, 2026-10-20 "PDF crate choice"). The PDFium
   shared library must be present **next to the executable or on the system**
   for PDF import to work.
2. **hayro** (optional, pure Rust) — build with `cargo build --release
   --features hayro` and no PDFium library is needed at all; hayro-syntax
   probes page sizes and hayro renders pages. PDFium remains the default
   until the corpus-fidelity gate (v0.3-02 Phase 3) validates pixel parity.

Everything else (markups, measurements, review workflow, `.rubric.json`
persistence, export and print) works without either engine's library, and
Rubric reports a clear status message instead of crashing when a PDF cannot
be opened.

## Where Rubric looks for the PDFium library

Position 1 wins; discovery stops at the first match.

1. `RUBRIC_PDFIUM_PATH` — either the shared library itself or the directory
   containing it (`.../lib/libpdfium.so`, `libpdfium.dylib` or `pdfium.dll`).
2. The directory the `rubric` executable lives in, and its `lib/`
   subdirectory.
3. The system library search path.

## Downloading PDFium

Pre-built binaries are published by the PDFium project (BSD-3-Clause):

- <https://github.com/bblanchon/pdfium-binaries/releases> — pick the archive
  matching your platform (`pdfium-linux-x64.tgz`, `pdfium-mac-*`, `pdfium-win-*`).

Examples:

```bash
# Linux — extract next to the executable
mkdir -p lib && tar -xzf pdfium-linux-x64.tgz -C lib

# Windows — unzip pdfium-win-x64.tgz and keep pdfium.dll beside rubric.exe

# Any platform — or just point at wherever you unpacked it
export RUBRIC_PDFIUM_PATH=/path/to/lib
```

Linux packages may also provide PDFium as `libpdfium.so` (Arch,
`pdfium-binaries` AUR-style drop-in or a manual install) — it is found by rule 3.

## Verifying the runtime

Open a PDF via **File ▸ Import PDF…**. The status bar shows "Importing…",
"Imported N page(s)" on success, or an explicit error naming what was missing.
No silent fallback: a failed import is always visible.

## Packaging note

The native installers (AppImage / DMG / MSI) **bundle** the PDFium shared
library produced by the packaging workflow, and the packaging job fails rather
than silently shipping an installer without PDF support; the BSD-3 licence text
travels with the library. The plain release zips ship **without** the library —
drop it beside the executable, point `RUBRIC_PDFIUM_PATH` at it, or use the
pure-Rust build (`--features hayro`) which needs no native library at all.
