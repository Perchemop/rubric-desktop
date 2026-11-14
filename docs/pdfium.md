# PDF engine

Rubric renders PDF pages through a **pluggable PDF engine**:

1. **PDFium** (default on x86_64 + macOS arm64 + Windows) — linked at run time
   (never at build time — see `Cargo.toml` and DECISIONS.md, 2026-10-20 "PDF
   crate choice"). The PDFium shared library is **bundled in all release
   artefacts** (plain zips and package-workflow installers alike) for these
   platforms. The packaging job fails if PDFium is missing.
2. **hayro** (pure Rust, used for Linux aarch64 releases) — build with
   `cargo build --release --features hayro`; no native library needed at all.
   hayro-syntax probes page sizes and hayro renders pages. hayro is the default
   engine for Linux aarch64 because no PDFium pre-built exists for that
   architecture.

Everything else (markups, measurements, review workflow, `.rubric.json`
persistence, export and print) works without either engine's library, and
Rubric reports a clear status message instead of crashing when a PDF cannot
be opened.

## Where Rubric looks for the PDFium library (fallback)

For non-release builds (e.g. `cargo run`), or if the bundled library is
somehow missing, the app falls back to discovery:

1. `RUBRIC_PDFIUM_PATH` — either the shared library itself or the directory
   containing it (`.../lib/libpdfium.so`, `libpdfium.dylib` or `pdfium.dll`).
2. The directory the `rubric` executable lives in, and its `lib/`
   subdirectory.
3. The system library search path.

Release downloads (v0.4.1+) **do not need this fallback** — the library is
already bundled.

## Downloading PDFium (for developers only)

Pre-built binaries are published by the PDFium project (BSD-3-Clause):

- <https://github.com/bblanchon/pdfium-binaries/releases> — pick the archive
  matching your platform (`pdfium-linux-x64.tgz`, `pdfium-mac-*`, `pdfium-win-*`).

Examples:

```bash
# Linux — extract next to the executable (dev builds only)
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

## Packaging note (v0.4.1+)

**All release artefacts bundle the required PDF runtime:**

- Plain zips (`rubric-<os>-<arch>.zip`): PDFium bundled for x86_64 Linux,
  macOS arm64, Windows x86_64; hayro build (no PDFium needed) for Linux
  aarch64.
- Package-workflow installers (AppImage, DMG, MSI, Windows portable zip):
  PDFium bundled inside the installer.

The packaging job **fails** if the required runtime is missing — a PDF-less
artefact must never ship silently. The BSD-3 licence text travels with the
bundled library.
