# Stirling PDF – Capability-level summary

**Purpose:** original summary of publicly available capability information,
added as inspiration for Rubric's post-v0.2 milestones (client-directed,
2026-10-26).

**Sources (public):**
- Repository: https://github.com/Stirling-Tools/Stirling-PDF
- Documentation / feature list: https://docs.stirlingpdf.com
- Licence (core MIT, open-core with proprietary directories): https://github.com/Stirling-Tools/Stirling-PDF/blob/main/LICENSE

**What it is (one paragraph, own words):** a self-hostable, open-source PDF
toolbox that runs in the browser, as a desktop client, or on a server. It
packages many small single-purpose PDF operations behind one interface and a
REST API, and positions itself as local/private ("without sending documents
to external services").

**Capability themes observed (capabilities only, not implementation):**

1. Page organisation
   - Merge, split, reorder, rotate, extract and delete pages.
   - Multi-page layout operations (combine N pages per sheet).

2. Content editing & redaction
   - Add text, images and shapes onto pages.
   - Redaction: draw areas that are removed/flattened from the content.
   - Watermarks and stamping.

3. Conversion & OCR
   - Convert to/from office formats, images, and HTML.
   - OCR to make scanned pages searchable.

4. Signatures & security
   - Drawn/typed signature placement.
   - Password protection, permissions, metadata editing.

5. Comparison & automation
   - Compare two PDFs and show differences.
   - Multi-step pipelines chaining operations, plus a REST API for automation.

6. Privacy stance
   - Local execution as a headline feature; aligns with Rubric's offline-first
     posture.

**Rubric relevance:** Rubric is a markup/measurement *review* tool, not a
general PDF manipulator. The capability themes that could translate into
Rubric's roadmap without changing its identity are:

- **Compare** (theme 5): directly validates the v0.2-01 `DiffResult` domain
  model; a future UI slice can present markup-level diffs between two saved
  `.rubric.json` projects — a capability reviewers ask for (also noted in
  `adobe-acrobat-capabilities.md`).
- **Redaction** (theme 2): a draw-to-redact markup kind is a natural extension
  of the existing markup model, though real content removal needs PDF writing,
  which is out of scope while the PDF stays read-only.
- **Page organisation** (theme 1): page extract/reorder/rotate would require
  PDF writing; recorded as a future research item only.
- **Signatures** (theme 4): a drawn-signature markup kind could reuse the ink
  geometry; candidate for a later milestone.
- Watermarks/OCR/conversion/pipelines (themes 3, 5): out of scope for Rubric's
  review focus; not planned.

**Clean-room note:** capability concepts only. Stirling PDF's core is MIT, but
the repository contains proprietary directories (open-core) — those parts are
not consulted at all. No code, UI text, screenshots or layouts are copied;
Rubric's implementation remains original (see `docs/inspiration.md`).

**Human gate:** client requested this inspiration addition directly
(2026-10-26); recorded in `DECISIONS.md`.
