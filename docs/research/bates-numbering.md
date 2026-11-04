# Bates numbering – Capability-level summary

**Purpose:** original, capability-level summary of Bates numbering (Bates
stamping) for a future Rubric slice. Rubric copies no vendor implementation,
UI, wording or algorithm; this note records *what* the capability is and *why*
reviewers use it, with public sources only.

**Sources:**
- Adobe Acrobat – "Add Bates numbering to PDFs":
  https://helpx.adobe.com/acrobat/using/add-bates-numbering-to-pdfs.html
- Bates numbering (general description, legal-discovery context):
  https://en.wikipedia.org/wiki/Bates_numbering
- PDF page geometry / coordinate conventions (public standard):
  https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf

**What the capability is:** a *Bates number* is a unique, sequential identifier
stamped on each page of a document set (for example `SMITH000001`,
`SMITH000002`, …). Legal and document-review workflows use it to reference a
specific page unambiguously across a production. A Bates-stamping capability
lets a user configure the identifier (prefix/suffix, starting number, digit
padding), choose where on the page it prints, optionally restrict it to a page
range, and then apply it — most often as the document is exported, printed or
"burned" to a flattened copy.

**Observable behaviour to match (capability level):**

1. **Configuration** — a free-text prefix and/or suffix wraps a numeric body;
   the body starts at a chosen number and is zero-padded to a chosen digit
   width, so every stamp has the same length. A live preview shows the next
   value (and how it advances) before anything is applied.
2. **Placement** — the stamp prints in a chosen corner/edge with a chosen
   margin, in a chosen font size and colour, without covering existing content.
3. **Page range** — the stamp may apply to every page, or to an inclusive
   from/to range (e.g. only the produced pages of a set).
4. **Preview** — the user can see the stamp at its exact placement on the page
   before applying it.
5. **Apply / bake** — on export or print, the sequential identifiers are drawn
   onto the rendered output in page order, continuing the sequence across pages.
   The number is typically "burned in" (a flattened copy) so it cannot be
   removed by a later reader.

**Rubric relevance:** Rubric already renders a composed page image on export
and print (`data/render.rs`, `data/print.rs`) and maps page space to screen on
the canvas (`ui/canvas.rs`). A Bates overlay is therefore a small, well-fitting
addition: a domain `BatesConfig` value type, an overlay drawn during export and
print, and a ghosted canvas preview. It stays consistent with Rubric's
**non-destructive** rule — the source PDF is never modified; the sequence exists
only in the composed output and the project's own settings.

**Out of scope / not planned:** removing an existing Bates number, OCR of a
stamped page, or editing a source PDF's page content.

**Notes for design:** original UI and wording only; no vendor screenshots or
string tables. Keep the sequence deterministic and page-ordered so a re-export
reproduces the same numbers. Clean-room: capability concepts only.
