# Research note — pure-Rust PDF engine (`hayro` + `lopdf`)

Status: **approved by the owner 2026-10-31** (direction chosen: adopt `hayro`
(+ `lopdf`) directly, rather than depend on PdfCraft's unpublished workspace).
Original summary written for Rubric; public URLs only; no vendor text/icons/
layouts/algorithms reproduced. This is an implementation input **only** because
the human gate above is recorded.

## Why (capability)

Rubric currently renders PDFs through a **native** library (`pdfium-render` →
PDFium `.so`/`.dll`). That is the root of defect **H8**: the release must ship a
native library per platform, and a fresh install cannot import/render a PDF until
the user supplies it. A pure-Rust engine removes the native dependency entirely
(simpler builds/installers, one fewer supply-chain surface) and unlocks PDF
*writing* (flattened-markup export, page operations).

## Sources (public)

- `hayro` — a family of Rust crates for parsing/interpreting/rendering PDFs;
  read-only by design; a visual regression suite of 1,600+ PDFs. Author: Laurenz
  Stampfl. Repo <https://github.com/LaurenzV/hayro> · crate
  <https://crates.io/crates/hayro> · docs <https://docs.rs/hayro>.
  Crates: `hayro-syntax` (parsing), `hayro-interpret` (interpret → abstract
  `Device`), `hayro` (render pages → bitmaps), `hayro-svg`, plus image decoders
  (`hayro-jpeg2000`, `hayro-jbig2`, `hayro-ccitt`) and `hayro-cmap`/`hayro-postscript`.
  MSRV 1.92. Licence **Apache-2.0 OR MIT** (from 0.5.0 onward; 0.1.0 was
  MIT OR Apache-2.0; mid versions Apache-2.0). Noted development gaps: robustness
  on hostile input, performance, and API stability (pre-1.0).
- `lopdf` — read/write/manipulate PDF object graphs. Repo
  <https://github.com/J-F-Liu/lopdf> · crate <https://crates.io/crates/lopdf>.
  Licence **MIT**.
- Cross-reference (capability evidence that a pure-Rust stack is viable):
  PdfCraft, <https://github.com/storytold/pdfcraft> (MIT OR Apache-2.0), which
  builds on `hayro` + a patched `lopdf`. Recorded as capability-level evidence
  only; no code is taken from it.

## Decision

- **Adopt `hayro` for rendering** (replace `pdfium-render`/PDFium), and
  **`lopdf` for writing** (a later slice: flattened-markup PDF export, page ops).
- Do **not** depend on PdfCraft's crates (unpublished; egui-fork coupling) and do
  **not** copy its source (originality rule).
- **Validate fidelity/robustness against real PDFs before deleting PDFium.** Until
  that passes, keep PDFium as an optional fallback and treat `hayro` as the new
  default only once it renders the corpus correctly (guide §2.5-quality posture).

## Impact

- `apps/rubric/Cargo.toml` (add `hayro`, later `lopdf`; drop `pdfium-render`).
- `apps/rubric/src/data/pdf.rs` (swap the engine behind the existing
  `read_pdf_info` / `render_page_to_cache` / `render_cache_dir` API).
- `deny.toml`, `ATTRIBUTION.md`, `docs/pdfium.md` → engine docs.
- CI + `release.yml` + `package.yml`: remove PDFium fetch/bundle steps once the
  native dependency is gone.

## Gate & caveats

- The human gate (above) is the approval for the *rendering swap* only. Any
  dependent large Task (PDF writing/page ops) needs its own recorded approval.
- `hayro` is pre-1.0: pin an exact version, keep the render-cache/poisoning
  behaviour, and add a corpus fixture test before removing PDFium.
