# PDFium runtime

Rubric renders PDF pages through **PDFium**, linked at run time (never at
build time — see `Cargo.toml` and DECISIONS.md, 2026-10-20 "PDF crate choice").
That keeps every build pure Rust, but it means the PDFium shared library must be
present **next to the executable or on the system** for PDF import to work.

The build is unaffected: everything else (markups, measurements, review
workflow, `.rubric.json` persistence, export and print) works without PDFium,
and Rubric reports a clear status message instead of crashing when a PDF is
opened without the library.

## Where Rubric looks for the library

Position 1 wins; discovery stops at the first match.

1. `RUBRIC_PDFIUM_PATH` — either the shared library itself or the directory
   containing it (`.../lib/libpdfium.so`, `libpdfium.dylib` or `pdfium.dll`).
2. The directory the `rubric` executable lives in (and its `lib/` subdirectory).
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

Official archives ship **without** bundling PDFium, because the library is
platform-specific and carries its own licence terms; users choose the official
binary or their distribution's package. Packaging it into the installers is
tracked as a known follow-up (see `docs/assumptions-and-questions.md`).
