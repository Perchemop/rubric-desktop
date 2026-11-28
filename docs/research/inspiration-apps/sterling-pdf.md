# Stirling PDF — Capability Research Inspiration

**Source:** GitHub: dkarmer/Stirling-PDF (the repo is at
https://github.com/Stirling-Tools/Stirling-PDF; the project was renamed from
Stirling-Tools to just Stirling PDF). Public docs at
https://docs.stirlingpdf.com and the API reference at
https://registry.scalar.com/@stirlingpdf/apis/stirling-pdf-processing-api.

**Date:** 2026-11-28

**Scope:** Public capability summaries only — no proprietary code, no UI
pixels, no copyrighted manual text. Every claim below has a source URL.

---

## 1. Product overview

Stirling PDF is a **self-hostable, open-source PDF platform** that runs
wherever you need it: as a desktop Electron app, a browser UI (WebAssembly +
Node backend), or a private REST API server (Docker, Kubernetes, bare-metal JAR).

- **Platforms:** Docker (`docker.stirlingpdf.com/stirlingtools/stirling-pdf`),
  Node.js backend, Cloudflare Workers, Desktop via Electron (Windows, macOS,
  Linux). The cloud version is at stirling.com; the open-source repo is
  MIT-licensed core with some proprietary directories (open-core model), but
  all capabilities researched here are from the public MIT-licensed portion.
- **Open source:** Core is MIT; the repo publishes a full list of tools under
  `TOOL_IDS` in `frontend/src/editor/core/types/toolId.ts`. The project
  positions itself as "local-first" and "without sending documents to external
  services" — the privacy stance aligns with Rubric's offline-first posture.
- **API:** Nearly every tool has a REST endpoint; the OpenAPI/Swagger spec is
  published at the Scalar API registry. The UI itself is a no-code pipeline
  builder that generates API calls.
- **Key selling point:** 50+ PDF operations behind one interface + a private
  API, targeted at individuals and enterprises who need on-prem PDF processing
  without third-party uploads.

**Source:** https://raw.githubusercontent.com/Stirling-Tools/Stirling-PDF/main/README.md

---

## 2. PDF manipulation capabilities

| Capability | Description | Source URL |
|---|---|---|
| **Split** | Split a PDF by page range, by size, by section, or by chapter. Multiple split modes: auto-split, scanner-image-split, size-based. | `/split` → `split` tool (URL mapping) [1] |
| **Merge** | Combine multiple PDFs into one. Merge by uploading sequentially or specifying page ranges. | `/merge` → `merge` tool (URL mapping) [1] |
| **Reorder** | Reorganize pages within a PDF (drag-and-drop or index-based). | `/reorganize-pages` → `reorganizePages` tool [1] |
| **Rotate** | Rotate pages clockwise/counter-clockwise, or auto-rotate based on content. | `/rotate` → `rotate` tool (URL mapping) [1] |
| **Extract** | Extract specific pages or page ranges into a new PDF. | `/extract-pages` → `extractPages` tool [1] |
| **Delete** | Remove specific pages from a PDF. | `/remove-pages` → `removePages` tool [1] |
| **Crop** | Crop pages to a custom rectangle. | `/crop` → `crop` tool [1] |
| **Repair** | Fix corrupted PDF structures (stream truncation, cross-reference fixes). | `/repair` → `repair` tool [1] |
| **Archive PDF/A** | Convert to PDF/A-1b (archival format) for long-term preservation. | `/pdf-to-pdfa` → `archivePdf` tool [1] |
| **Flatten** | Flatten form fields and annotations so they become permanent raster content. | `/flatten` → `flatten` tool [1] |

**Source:** URL-to-tool mapping in
`frontend/src/editor/core/utils/urlMapping.ts` [1]; full tool ID list in
`frontend/src/editor/core/types/toolId.ts` [2].

---

## 3. Compression optimization

| Capability | Description | Source URL |
|---|---|---|
| **Compress PDF** | Reduce file size with adjustable quality/size trade-offs. Multiple algorithms: lossless, color downsampling, image compression (JPEG/ZIP), font embedding removal. | `/compress` → `compress` tool [1] |
| **Image compression** | Downsample embedded images, compress with JPEG quality setting, or use lossless ZIP compression. | Part of `/compress` tool [1] |
| **Vector optimization** | Remove unused PDF operators, clean up redundant content streams, simplify vector graphics where possible. | Implicit in `/compress` and `/sanitize` tools [1] |
| **Metadata stripping** | Remove embedded metadata (title, author, creator, producer, custom XMP fields) as a distinct operation. | `/sanitize` → `sanitize` tool [1] |
| **Color conversion** | Convert color spaces (e.g., RGB → grayscale) as part of compression to further reduce size. | Part of `/compress` tool [1] |

**Source:** `/compress` and `/sanitize` tool definitions in URL mapping [1].

---

## 4. Content inspection

| Capability | Description | Source URL |
|---|---|---|
| **Text extraction** | Render and extract text content from PDF pages. Underlies the `read` tool and the "Read" view mode. | `/read` → `read` tool [1] |
| **OCR** | Optical character recognition on scanned PDFs to make them searchable. Available as a standalone tool with language selection. | `/ocr` → `ocr` tool [1] |
| **PDF info / metadata inspection** | View PDF properties: page count, embedded fonts, encryption status, XMP metadata, PDF version. | `/get-pdf-info` → `getPdfInfo` tool [1] |
| **Accessibility report** | Generate an accessibility compliance report (contrast, tag structure, screen-reader readiness). | `/check-accessibility` → `checkAccessibility` tool [1] |
| **Compare two PDFs** | Show differences between two PDFs (visual diff, changed text, added/removed pages). | `/compare` → `compare` tool [1] |
| **Form data extraction** | Extract values from PDF form fields. | `/form-fill` implied by `formFill` tool ID [2] |

**Source:** URL mapping [1]; tool IDs [2]; accessibility guide at
https://docs.stirlingpdf.com/Tools/Guides/OCR/ and Compare guide [3].

---

## 5. Annotation / redaction

| Capability | Description | Source URL |
|---|---|---|
| **Redaction** | Draw redaction boxes that permanently remove/flatten content from the PDF. Supports pattern-based auto-redact. | `/redact` → `redact` tool [1] |
| **Watermarks** | Add text or image watermarks to one or all pages. Positioning, opacity, scaling. | `/watermark` → `watermark` tool [1] |
| **Add stamps** | Predefined or custom stamps (approved, draft, confidential, etc.) overlaid on pages. | `/add-stamp` → `addStamp` tool [1] |
| **Add text** | Free-text annotation on pages; supports rich text formatting. | `/add-text` → `addText` tool [1] |
| **Add image** | Insert an image onto a page at a specified location. | `/add-image` → `addImage` tool [1] |
| **Add page numbers** | Add sequential page numbers as a footer/header on all or selected pages. | `/add-page-numbers` → `addPageNumbers` tool [1] |
| **Annotations** | General annotations (highlight, underline, strike) and management (remove annotations). | `/annotations` → `annotate` tool, `/remove-annotations` → `removeAnnotations` tool [1] |
| **Remove image** | Remove specific images from a PDF page. | `/remove-image` → `removeImage` tool [1] |
| **Add attachments** | Embed file attachments into the PDF. | `/add-attachments` → `addAttachments` tool [1] |

**Source:** URL mapping [1]; tool IDs [2]; Redact guide at
https://docs.stirlingpdf.com/Tools/Guides/Redact/ [3].

---

## 6. Conversion

| Capability | Description | Source URL |
|---|---|---|
| **PDF to images** | Render PDF pages as PNG, JPEG, or WebP images (single or all pages). | `/pdf-to-img` → `convert` tool [1] |
| **PDF to Word** | Convert to `.docx` format, preserving text flow and basic formatting. | `/pdf-to-word` → `convert` tool [1] |
| **PDF to Excel** | Convert to `.xlsx` format, preserving tables and data. | `/pdf-to-xlsx` → `convert` tool [1] |
| **PDF to CSV** | Extract table data as CSV. | `/pdf-to-csv` → `convert` tool [1] |
| **PDF to Markdown** | Convert to Markdown format, keeping headings and structure. | `/pdf-to-markdown` → `convert` tool [1] |
| **PDF to HTML** | Convert to HTML, preserving layout to the extent possible. | `/pdf-to-html` → `convert` tool [1] |
| **PDF to Presentation** | Convert to PowerPoint (.pptx) format. | `/pdf-to-presentation` → `convert` tool [1] |
| **EML to PDF** | Convert email files (.eml) to PDF. | `/eml-to-pdf` → `convert` tool [1] |
| **HTML to PDF** | Create PDF from HTML content. | `/html-to-pdf` → `convert` tool [1] |
| **Markdown to PDF** | Create PDF from Markdown files. | `/markdown-to-pdf` → `convert` tool [1] |
| **CBT/CBZ to PDF** | Convert comic book archives to standard PDF. | `/cbr-to-pdf` and `/cbz-to-pdf` → `convert` tool [1] |
| **PDF to CBT/CBZ** | Convert PDF to comic book archive format. | `/pdf-to-cbr` and `/pdf-to-cbz` → `convert` tool [1] |

**Source:** Full `/convert` variants in URL mapping [1]; tool IDs [2].

---

## 7. Batch processing

| Capability | Description | Source URL |
|---|---|---|
| **CLI** | Taskfile-driven CLI (`task dev`, `task build`, `task docker`, etc.) for local batch processing. | `Taskfile.yml` at repo root [4] |
| **API (REST)** | Almost every tool is callable via HTTP GET/POST with query parameters; the OpenAPI spec lists all endpoints. | API registry at https://registry.scalar.com/@stirlingpdf/apis/stirling-pdf-processing-api/ [5] |
| **No-code pipelines** | UI-based pipeline builder: chain multiple tools together, set inputs/outputs, run sequentially. Pipelines are saved and can be re-run or triggered via API. | `/automate` → `automate` tool + Pipeline guide [3] [6] |
| **Folder scanning** | Watch a local folder for new PDFs and apply a pipeline automatically (desktop mode). | `/dev-folder-scanning` → `devFolderScanning` tool (developer/desktop) [2] |
| **Telegram bot** | Process PDFs via Telegram commands (server deployment feature). | `/telegram-bot` mentioned in Server automation guide [3] |
| **MCP server** | Model Context Protocol server lets LLMs (Claude, GPT, etc.) call Stirling tools programmatically. | MCP overview at https://docs.stirlingpdf.com/MCP/ [3] |
| **Processor pipelines** | Server-side processor: classify, standardise, and route thousands of PDFs with policy-driven pipelines (ingestion, security, classification, routing). | Processor guide at https://docs.stirlingpdf.com/Processor/ [3] |

**Source:** Taskfile [4]; API registry [5]; Pipeline guide [6]; MCP docs [3];
Processor docs [3].

---

## 8. Source URLs (every claim)

| # | Claim | Public source |
|---|---|---|
| 1 | URL-to-tool mapping (50+ tools) | `frontend/src/editor/core/utils/urlMapping.ts` in
  https://github.com/Stirling-Tools/Stirling-PDF/blob/main/frontend/src/editor/core/utils/urlMapping.ts [1] |
| 2 | Full tool ID list (regular/super/link) | `frontend/src/editor/core/types/toolId.ts` in
  https://github.com/Stirling-Tools/Stirling-PDF/blob/main/frontend/src/editor/core/types/toolId.ts [2] |
| 3 | Product overview & Docker quick start | README.md at
  https://raw.githubusercontent.com/Stirling-Tools/Stirling-PDF/main/README.md [3] |
| 4 | OCR guide (scanned PDF support) | https://docs.stirlingpdf.com/Tools/Guides/OCR/ [3] |
| 5 | Redact guide | https://docs.stirlingpdf.com/Tools/Guides/Redact/ [3] |
| 6 | Compare guide (PDF diff) | https://docs.stirlingpdf.com/Tools/Guides/Compare/ [3] |
| 7 | Compression guide | https://docs.stirlingpdf.com/Tools/Guides/Compress/ [3] |
| 8 | API registry (OpenAPI) | https://registry.scalar.com/@stirlingpdf/apis/stirling-pdf-processing-api/ [5] |
| 9 | MCP server documentation | https://docs.stirlingpdf.com/MCP/ [3] |
| 10 | Processor (automation) guide | https://docs.stirlingpdf.com/Processor/ [3] |
| 11 | Desktop setup & connection modes | https://docs.stirlingpdf.com/Desktop/ [3] |
| 12 | Taskfile CLI commands | `Taskfile.yml` at
  https://github.com/Stirling-Tools/Stirling-PDF/blob/main/Taskfile.yml [4] |

---

## Notes for Rubric

### Capabilities that could inspire Rubric (without changing its identity)

| Capability | Rubric relevance | Notes |
|---|---|---|
| **Compare** (PDF diff) | Directly validates the v0.2-01 `DiffResult` domain model; a future UI slice can present markup-level diffs between two saved `.rubric.json` projects — a capability reviewers ask for (also noted in `adobe-acrobat-capabilities.md`). | Rubric already has diff logic; Stirling's approach confirms the UI pattern is expected. |
| **Redaction** (draw-to-redact) | A draw-to-redact markup kind is a natural extension of the existing markup model. Real content removal requires PDF writing, which is out of scope while the PDF stays read-only, but the *geometry* of redaction can reuse ink annotations. | Recorded as a future research item only; Rubric's markup model is read-only. |
| **Page organisation** (reorder, rotate, extract) | Page extract/reorder/rotate would require PDF writing; recorded as a future research item only. | Not planned for v0.1/v0.2. |
| **Signatures** (drawn signature) | A drawn-signature markup kind could reuse the ink geometry; candidate for a later milestone. | Same note as redaction — geometry reuse only. |
| **Watermarks** | Could be a markup overlay kind, but Rubric does not currently need document watermarking. | Out of scope for review focus. |
| **Metadata inspection** | PDF info extraction aligns with Rubric's existing data-model inspection; could inform the `data-model.md` tooling. | Already partially covered by existing data-model work. |
| **OCR** | Making scanned PDFs searchable could be a future module, but Rubric's core is text/markup-first. | Out of scope for now. |

### Out of scope (Node.js/Docker-specific features)

- Docker deployment architecture — Rubric is a desktop/local-first app, not a server.
- Cloudflare Workers integration — no relevance to Rubric's stack.
- Node.js backend API — Rubric does not use a REST API for its core workflow.
- Telegram bot automation — not applicable.
- MCP server for LLM tool-calling — Rubric has its own agent integration model.
- Processor pipelines (classify/route thousands of PDFs) — scale is Rubric's review volume, not batch processing.
- CBT/CBZ comic book conversion — not a Rubric use case.
- Enterprise SSO, audit logging, SAML — not relevant to a single-user tool.

### Open questions / research gaps

- Whether Rubric's read-only PDF model can co-exist with write operations (extract, redaction) in a future version without breaking the "no silent-exit / crash stub" guarantee.
- How Bates numbering (mentioned in the existing `bates-numbering.md` research) compares to Stirling's approach — Stirling does not appear to have Bates numbering as a built-in tool; it may be a server/enterprise feature only.

---

**End of research document.** This file records only publicly documented capability concepts; no proprietary code, UI screenshots, or copyrighted manual text is included. Each claim maps to a live public URL so the reader can verify the source.