# Adobe Acrobat – Capability-level summary

**Purpose:** original summary of publicly available capability information for Rubric v0.2 discovery.

**Sources:**
- Adobe compare files help: https://helpx.adobe.com/acrobat/using/compare-documents.html
- Adobe annotate markup page: https://www.adobe.com/acrobat/how-to/annotate-pdf-online.html
- PCWorld Acrobat Standard vs Pro: https://www.pcworld.com/article/397929/adobe-acrobat-standard-dc-vs-adobe-acrobat-pro-dc.html

**Capability themes observed:**

1. Markup & commenting
   - Annotate with text boxes, sticky notes, shapes, freehand pencil, highlighters, underline, strikethrough.
   - Send files for review with notifications, reminders, deadlines.

2. Document comparison
   - Compare Documents feature highlights differences between two versions of a PDF and generates a summary report.
   - Options for side-by-side, report format, page range settings.
   - For scanned documents OCR can be enabled for text comparison.

3. OCR & editing
   - Acrobat Pro includes OCR that detects image-based PDFs and offers to recognize text, preserving layout.
   - Recognized text becomes searchable and editable.

4. Security & forms
   - Password protection, redaction, hidden information removal.
   - Form creation and e-signatures.

**Rubric relevance:** Document comparison and OCR are capability areas reviewers often request. Rubric is local-first; comparison could be implemented locally as a domain operation comparing two `.rubric.json` projects or two PDFs at the markup level. OCR is out of scope for v0.2 given heavy native deps and privacy constraints; capture as research item for future.

**Notes for design:** Keep clean-room. Use only capability concepts.
