# Adobe Acrobat — Capability Research (Inspiration App)

- **Date:** 2026-11-28
- **Method:** Public black-box research only. Sources: helpx.adobe.com, adobe.com product pages, public university/IT tutorials, Adobe community forums, YouTube tutorials, and secondary references to the PDF/ISO 32000 annotation model. No proprietary material, no RE output, no verbatim manual text. Every capability claim carries a source URL.
- **Purpose:** Feed Rubric's capability inventory (designer role, guide II.27.B) — markup, measurement, and review capabilities first.

---

## 1. Product overview

**Editions.**
- **Acrobat Reader** is the free tier: viewing, printing, signing, and annotating PDFs. (https://helpx.adobe.com/reader/desktop.html)
- **Acrobat Standard** covers basic PDF work: edit and organize PDFs, convert documents to/from PDF, fill forms, sign documents, request e-signatures, password-protect files. (https://www.adobe.com/acrobat/plans.html)
- **Acrobat Pro** includes everything in Standard plus "enhanced PDF tools and e-signature capabilities" — in practice the advanced tier for content editing, redaction, preflight, and full review/measurement workflows. (https://www.adobe.com/acrobat/plans.html)
- A newer **Acrobat Studio** tier exists (PDF Spaces, AI Assistant, Adobe Express integration). (https://helpx.adobe.com/acrobat/desktop/whats-new/whats-new-acrobat-desktop.html)

**Platforms.**
- Desktop installers are provided for **Windows and macOS** (64-bit; multilingual installers; separate Pro/Standard packages). (https://helpx.adobe.com/acrobat/desktop/get-started/access-the-app/install-acrobat.html, https://helpx.adobe.com/acrobat/desktop/get-started/access-the-app/download-acrobat.html)
- **No current native Linux build.** Adobe's last native Linux Reader was 9.5.5 (2013) and is no longer supported; Linux users are steered to Acrobat's online tools instead. (https://linuxconfig.org/how-to-install-adobe-acrobat-reader-on-ubuntu-20-04-focal-fossa-linux, https://www.adobe.com/acrobat/hub/edit-pdf-linux.html) — **Relevant to Rubric:** Acrobat leaves the Linux desktop PDF-editing niche open; Rubric can compete there.

**Out of scope for Rubric:** cloud storage/Document Cloud, Acrobat Studio/PDF Spaces, Adobe Express integration, e-signature services (Adobe Sign), subscription/licensing model.

---

## 2. Annotation / markup tools

**Standard PDF annotation taxonomy (ISO 32000 family) as reflected in public documentation.** PDF 2.0 defines annotation subtypes including: Text (sticky note), Link, FreeText (callout/text box), Line, Square, Circle, Polygon, PolyLine, Highlight, Underline, Squiggly, StrikeOut, Stamp, Caret (insert-text), Ink (freehand), Popup, FileAttachment, Sound, Movie, Widget (form field), Screen, PrinterMark, TrapNet, Watermark, 3D, Redact, Projection, RichMedia. Acrobat implements the user-facing subset of these in its Comment/Markup toolset. (https://pypdf.readthedocs.io/en/latest/user/reading-pdf-annotations.html; category grouping also at https://www.nutrient.io/guides/document-engine/annotations/introduction-to-annotations/what-are-annotations/ and https://mapsoft.com/posts/pdf-annotations.html)

**Acrobat's Comment toolbar (public tutorials enumerate 13 tools):** Sticky Note, Highlight, Underline, Strikethrough, Note to Replace Text, Insert Text at Cursor, Text Comment, Text Box, Pencil, Eraser, Stamp, File Attachment, and Drawing Tools (shapes). (https://www.youtube.com/watch?v=idvanvVK1Ys)

**Behavioral details.**
- Annotation tools **do not appear by default**; they are opened via the Comment toolset (All tools > Comment in the current UI). (https://www.uhd.edu/computing/training-lms/adobe-markup.html)
- Select text or an image, then use **Add a comment** from the floating toolbar; type and **Post** — the popup note is attached to the selection. (https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/review-documents/add-comments.html)
- **Sticky Note** properties (color, author, icon) are editable via right-click > Properties. (https://www.uhd.edu/computing/training-lms/adobe-markup.html)
- **Pencil** is freehand with changeable color and line thickness; **shape tools** include oval, line, and connected-line/polygon drawing. (https://services.stthomas.edu/TDClient/1898/ClientPortal/KB/Article/111806/Adobe-Acrobat-Reader-How-to-Annotate-PDFs)
- The PC markup hub summarizes the same surface: comments, highlight, underline, strikethrough, direct drawing, shapes, signature placement. (https://www.adobe.com/acrobat/hub/how-to-mark-up-pdfs-pc.html)
- Student-facing Adobe resource confirms sticky notes, highlighting, and underlining as the core markup set. (https://www.adobe.com/uk/acrobat/resources/digital-annotation-tools-for-students.html)
- Because markup is stored as standard PDF annotation objects, third-party parsers (e.g., pypdf) can read Acrobat-created annotations back — good sign for Rubric's round-trip strategy. (https://pypdf.readthedocs.io/en/latest/user/reading-pdf-annotations.html)
- **Deprecated in PDF 2.0:** Movie and Sound annotations are deprecated in favor of RichMedia (introduced in Acrobat 9). Rubric should not carry them. (https://www.loc.gov/preservation/digital/formats/fdd/fdd000474.shtml)
- **Accessibility:** annotations must be either tagged into the Tags tree or marked as artifacts; Acrobat offers a "Tag Annotations" option so new comments are tagged automatically, and highlight/underline markups auto-associate with their text. (https://helpx.adobe.com/acrobat/using/create-verify-pdf-accessibility.html, https://helpx.adobe.com/acrobat/using/editing-document-structure-content-tags.html)

---

## 3. Measurement tools

**Access model.** Measuring lives under **All tools > Measure object > Measuring tool**, showing a measuring toolbar plus a Measurement Info panel. Measuring tools are available to **Reader** users **only if the PDF creator enables measuring functionality**. (https://helpx.adobe.com/acrobat/using/grids-guides-measurements-pdfs.html)

**Tools.**
- **Distance tool** — two clicks measure the distance between two points. (https://helpx.adobe.com/acrobat/using/grids-guides-measurements-pdfs.html)
- **Perimeter tool** — click each vertex, double-click the last point to finish a multi-segment path. (https://helpx.adobe.com/acrobat/using/grids-guides-measurements-pdfs.html)
- **Area tool** — click at least two points, then click the first point again to close the polygon and compute area. (https://helpx.adobe.com/acrobat/using/grids-guides-measurements-pdfs.html)
- **Angle measurement** is achieved through the Perimeter tool: click a start point, draw one segment, click once, draw the second segment, double-click; the angle appears in the tool box. (https://www.fareham.gov.uk/planning/measuring_plans.aspx)

**Feedback and snapping.**
- The **Measurement Info panel** shows current measurement, delta values, scale ratio, and precision while measuring. (https://helpx.adobe.com/acrobat/using/grids-guides-measurements-pdfs.html, https://www.fareham.gov.uk/planning/measuring_plans.aspx)
- **Snap modes:** Snap to Paths, Snap to Endpoints, Snap to Midpoints, Snap to Intersections. (https://helpx.adobe.com/acrobat/using/grids-guides-measurements-pdfs.html)
- Tool options include **Ortho on/off** (orthographic-only measurement lines) and **Show/Hide rulers**. (https://helpx.adobe.com/acrobat/using/grids-guides-measurements-pdfs.html)

**Calibration and scale ratios.**
- **Change the scale ratio** per measurement; public planning-office guidance publishes a drawing-scale-to-measuring-scale mapping (e.g., 1:20 → 1 cm = 0.2 m through 1:10000 → 1 cm = 100 m). (https://www.fareham.gov.uk/planning/measuring_plans.aspx)
- Every completed measurement is **inserted as a comment** carrying the computed distance/perimeter/area value — measurement results are first-class annotations. (https://helpx.adobe.com/acrobat/using/grids-guides-measurements-pdfs.html)
- **Geospatial PDFs:** per-document unit switching (Distance Unit / Area Unit, e.g., miles, km, sq ft, acres) via right-click with the Measuring tool; these reset when the PDF closes. Default units for all geospatial PDFs are set in **Preferences > Measuring (Geo)**, including decimal vs. degrees-minutes-seconds display and signed vs. named direction. (https://helpx.adobe.com/acrobat/desktop/edit-documents/geospatial-pdfs/change-measurements.html)
- **Community pain points Rubric can beat:** users want a persistent default scale ratio (e.g., 1/8"=1'-0") instead of re-entering it per measurement (https://acrobatusers.com/forum/architecture-engineering-and-construction/measuring-tool/), a permanently metric default (https://community.adobe.com/questions-9/change-measure-tool-permanently-to-metric-1260141), and manual editing of a measured value on a drawing (e.g., show 700 mm where the drawing says 600 mm) — currently a workaround territory (https://community.adobe.com/questions-9/measure-tool-how-do-i-edit-measurement-manually-1259532).
- Reader-based measuring of real blueprints is a documented workflow, confirming the feature matters at the free tier. (https://www.youtube.com/watch?v=MXOxAvVp5Ns, https://www.youtube.com/watch?v=LBMu5M9GV3I, https://community.adobe.com/questions-12/measuring-tool-1507240)
- **PDF 2.0 addition (out of scope for Rubric v1):** 3D views may contain 3D measurements and cross-sections. (https://www.pdflib.com/pdf-knowledge-base/pdf-20/new-capabilities/)

---

## 4. Review workflows

**Distributing and collecting reviews.**
- **Send for Review** (Send for Comment on Mac) adds each recipient's comments automatically to the PDF; Acrobat **consolidates all reviewers' comments into one document** with author names and timestamps; reviewers can be added anytime, with deadlines and reminders. (https://www.adobe.com/learn/acrobat/web/gather-feedback-pdf-comments, https://helpx.adobe.com/ee/acrobat/how-to/gather-feedback-pdf-comments.html)
- **Shared reviews by link:** multiple people comment on the same PDF simultaneously, with replies, resolution tracking, tagging of teammates, and emoji reactions. (https://www.adobe.com/acrobat/features/share-and-review-pdfs.html)
- **Tracking:** Home view separates **Shared by you / Shared by others**; per-file actions include Invite Person, Copy Shared Link, Unshare File, Delete Shared File (ends review), Remove Me, and Report Abuse; each file shows name, sharing status, and last-opened timestamp. (https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/manage-reviews/track-reviews.html)
- Community threads document real-world constraints of shared reviews (permissions, sequential commenting by multiple reviewers) — evidence the model is heavily used and imperfect. (https://community.adobe.com/questions-9/pdf-workflow-sequential-commenting-by-multiple-reviewers-revision-safe-1305588, https://community.adobe.com/questions-9/adobe-acrobat-shared-reviews-permissions-and-commenting-1652120, https://resources.ascented.com/ascent-blog/technical-writing-tip-reviewing-comments-in-adobe-acrobat)

**Compare Files.**
- Pick old/new versions (Change File, swap arrows), optionally **Compare text only**, then Compare; results open in a **new document whose first page is a scannable summary of all differences**. (https://helpx.adobe.com/acrobat/using/compare-documents.html)
- **Settings by document type:** reports/spreadsheets/magazine layouts (reflowing content) vs. presentation decks; **scanned documents, drawings, or illustrations** mode for image-based files; the text option suits large documents (250+ pages). (https://helpx.adobe.com/acrobat/using/compare-documents.html)
- **Side-by-side view (Ctrl+\)** shows parallel pages with highlighted change bands; clicking a band opens a pop-up detailing all changes in that band. (https://helpx.adobe.com/acrobat/using/compare-documents.html)

**Stamps.**
- Place stamps from categorized palettes; after placement you can move, resize, rotate, delete, and change **opacity and color** via Properties > Appearance. (https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/use-stamps/add-modify-stamps.html)
- **Custom stamps** are created from PDF, JPEG, PSD, AI, or DWG files (Reader supports PDF only), organized into named categories. (https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/use-stamps/create-custom-stamps.html)
- **Dynamic stamps** auto-populate date, time, and user identity (e.g., Approved/Received) — built as a stamp PDF containing a form text field with a **custom JavaScript calculation script**, stored in the user's Stamps folder. (https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/use-stamps/automate-date-time.html; institutional walkthrough: https://controller.vpfa.fsu.edu/sites/default/files/documents/Quality%20Assurance/Setting%20Up%20a%20Dynamic%20Stamp.pdf)

---

## 5. Document handling (mostly out of scope for Rubric)

- **Content editing** (change, replace, add, delete text; font/format adjustments; AI Assistant chat prompts for replace/rewrite/restructure/redact/format/highlight) is a Pro-tier editing surface — **out of scope for Rubric**. (https://helpx.adobe.com/acrobat/desktop/whats-new/whats-new-acrobat-desktop.html, https://helpx.adobe.com/acrobat/desktop.html)
- **Page organization:** reorder, delete, or rotate pages. (https://helpx.adobe.com/acrobat/desktop/get-started/learn-the-basics/overview.html)
- **Forms:** convert documents/scans into fillable forms with automatic field detection (Prepare a form); fill forms in Reader; import/export form data as FDF/XFDF/XML/TXT and bulk-export responses to CSV/XML — **out of scope for Rubric**, though field-placement UX may inspire measurement-annotation placement. (https://helpx.adobe.com/acrobat/desktop/work-with-pdf-forms/create-forms/convert-to-forms.html, https://helpx.adobe.com/reader/desktop/fill-forms.html, https://helpx.adobe.com/acrobat/using/collecting-pdf-form-data.html)
- **Signatures / e-signature requests** — **out of scope for Rubric**. (https://www.adobe.com/acrobat/plans.html)

---

## 6. UI/UX patterns

- **Three views:** Home (recent files, shared docs, common tools), **All tools** (full searchable tool list), and Document view (multiple PDFs open as **tabs** with Prev/Next tab navigation). (https://helpx.adobe.com/acrobat/desktop/get-started/learn-the-basics/workspace.html)
- **Right panel** hosts bookmarks, page navigation, zoom, viewing options — a customizable navigation pane. (https://helpx.adobe.com/acrobat/desktop/get-started/learn-the-basics/workspace.html)
- **Quick action toolbar** is a **floating, customizable** widget carrying commenting/annotation/form tools for the current workflow. (https://helpx.adobe.com/acrobat/desktop/get-started/learn-the-basics/workspace.html)
- The 2023 "new Acrobat" UI: All tools on the left, document actions (search, save, print, share) on the right, hamburger menu replacing File/Edit/View, contextual pop-up menus, and a customizable floating toolbar; a **Disable New Acrobat** switch existed on Windows. (https://svsu.teamdynamix.com/TDClient/1949/Portal/KB/Article/156603/Adobe-Acrobat-New-Experience-Interface, https://community.adobe.com/questions-9/acrobat-2023-how-to-revert-to-classic-gui-user-interface-1285215)
- **Negative lessons for Rubric:** professional users criticized the modern UI for non-resizable panels, buried tools, tiny icons, and poor contrast — documented in public commentary. (https://pubcom.com/blog/acrobatnew)
- Selecting content surfaces a **floating comment toolbar** with "Add a comment" — contextual tool presentation rather than a fixed palette. (https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/review-documents/add-comments.html)

---

## 7. Import / Export

- **Export PDF:** Convert panel or All tools > Export a PDF; image-format export; **Export Selection As** saves selected text to DOCX, DOC, XLSX, RTF, XML, HTML, or CSV. (https://helpx.adobe.com/acrobat/using/exporting-pdfs-file-formats.html)
- **Web export limits:** up to 100 MB; OCR converts scans to editable/searchable data; not convertible: password-protected files, permission-restricted files, PDF Portfolios, Illustrator-generated files; converted output preserves layout/quality. (https://helpx.adobe.com/acrobat/web/share-review-and-export/export-and-print/export-pdf-overview.html)
- **Form data interchange:** import/export FDF/XFDF/XML/TXT. (https://helpx.adobe.com/reader/desktop/fill-forms.html, https://helpx.adobe.com/acrobat/using/collecting-pdf-form-data.html)
- **Printing with markup:** a documented preference controls whether comments/annotations are printed (`bprintCommentPopups`) — Rubric's print path needs an explicit "print markup or not" decision. (https://www.adobe.com/devnet-docs/acrobatetk/tools/PrefRef/Windows/Annots.html)
- **Accessibility tags:** tagged-PDF structure, Tags panel, tag-annotations option, and tab-order checks define the accessibility bar for exported marked-up PDFs. (https://helpx.adobe.com/acrobat/using/create-verify-pdf-accessibility.html, https://helpx.adobe.com/acrobat/using/editing-document-structure-content-tags.html)

---

## 8. Automation

- **Action Wizard (Use guided actions)** records batch sequences applied to one PDF or a folder; actions can be edited, renamed, copied, deleted, reordered, imported, and exported as `.sequ` files; input sources include folders on the computer. (https://helpx.adobe.com/acrobat/using/action-wizard-acrobat-pro.html, https://evermap.com/ActionWizardX.asp)
- Community-published actions show the pattern's reach: **Find & Highlight Words** across many documents, an action that detects which documents contain comments and copies them into folders, and one that restricts editing to commenting/filling/signing. (https://acrobatusers.com/actions-exchange, https://pdfautomationstation.substack.com/p/the-acrobat-action-wizard)
- **Licensing stance:** Adobe permits automation of a licensed Acrobat **only** via Action Wizard; scripted/RPA automation is otherwise not permitted, with enterprise scale directed to Document Services APIs. (https://helpx.adobe.com/acrobat/kb/automation-and-document-workflows.html)
- **JavaScript API:** the Doc object exposes `syncAnnotScan` and `getAnnots` to enumerate page annotations — the same review/markup data Acrobat's Comments List shows is script-accessible. (https://developer.adobe.com/acrobat-pdfl-sdk/guides/jsdevguide/js-dev-rma)
- **PDF actions/triggers:** actions attach to events (bookmark selection, page open/close, form fields, media) and can run in sequence, with a Locked option preventing edits — the in-document automation model. (https://helpx.adobe.com/acrobat/desktop/edit-documents/apply-pdf-actions/pdf-actions.html)
- Dynamic stamps themselves are JavaScript-driven (custom calculation scripts), tying automation to the markup layer. (https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/use-stamps/automate-date-time.html)

---

## Out-of-scope capabilities for Rubric

| Capability | Reason |
|---|---|
| PDF content editing (text/image replacement, reflow, redaction) | Rubric is a markup/review app, not an editor (guide II.27 clean-room scope) |
| Cloud storage, Document Cloud, Acrobat Studio/PDF Spaces | Cloud service, not local-first |
| E-signatures (Adobe Sign) | Separate product domain |
| Form authoring/filling, form-data import/export | Forms domain; revisit only for annotation placement UX |
| Conversion/export to Office formats, OCR of scans | Conversion pipeline, not review |
| RichMedia / 3D annotations and 3D measurements | Out of Rubric v1 scope |
| Subscription/licensing, Adobe Sign workflows | Not applicable |

**In-scope inspiration for Rubric (capability seeds):**
1. Full annotation taxonomy: sticky note, highlight, underline, strikethrough, replace-text/insert-text carets, text box/callout, shapes (line/square/circle/polygon/polyline/cloud), ink/pencil + eraser, file attachment, stamp.
2. Comment list with per-annotation popups, replies, resolution, author + timestamp attribution.
3. Measurement: distance/perimeter/area (+ angle via polyline), scale-ratio calibration with unit presets, snap modes (paths/endpoints/midpoints/intersections), ortho lock, measurement-info panel (value/deltas/scale/precision), measurement-as-annotation persistence.
4. Stamps including dynamic (date/time/user) stamps with custom categories.
5. Compare Files UX: old/new swap, text-only mode, document-type presets, summary page, side-by-side change bands.
6. Review distribution: send-for-review consolidation into one annotated PDF, shared links, reviewer tracking.
7. UI patterns: floating quick-action toolbar, right-hand navigation panel, tabbed documents, searchable All-tools view — and the documented failures (non-resizable panels, buried tools) to avoid.
8. Automation: batch action model (.sequ import/export), script-accessible annotation enumeration (`getAnnots`-style API) as a Rubric plugin/scripting surface.

---

## Source URL index

1. https://helpx.adobe.com/reader/desktop.html
2. https://www.adobe.com/acrobat/plans.html
3. https://helpx.adobe.com/acrobat/desktop/whats-new/whats-new-acrobat-desktop.html
4. https://helpx.adobe.com/acrobat/desktop/get-started/access-the-app/install-acrobat.html
5. https://helpx.adobe.com/acrobat/desktop/get-started/access-the-app/download-acrobat.html
6. https://linuxconfig.org/how-to-install-adobe-acrobat-reader-on-ubuntu-20-04-focal-fossa-linux
7. https://www.adobe.com/acrobat/hub/edit-pdf-linux.html
8. https://pypdf.readthedocs.io/en/latest/user/reading-pdf-annotations.html
9. https://www.nutrient.io/guides/document-engine/annotations/introduction-to-annotations/what-are-annotations/
10. https://mapsoft.com/posts/pdf-annotations.html
11. https://www.youtube.com/watch?v=idvanvVK1Ys
12. https://www.uhd.edu/computing/training-lms/adobe-markup.html
13. https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/review-documents/add-comments.html
14. https://services.stthomas.edu/TDClient/1898/ClientPortal/KB/Article/111806/Adobe-Acrobat-Reader-How-to-Annotate-PDFs
15. https://www.adobe.com/acrobat/hub/how-to-mark-up-pdfs-pc.html
16. https://www.adobe.com/uk/acrobat/resources/digital-annotation-tools-for-students.html
17. https://www.loc.gov/preservation/digital/formats/fdd/fdd000474.shtml
18. https://helpx.adobe.com/acrobat/using/create-verify-pdf-accessibility.html
19. https://helpx.adobe.com/acrobat/using/editing-document-structure-content-tags.html
20. https://helpx.adobe.com/acrobat/using/grids-guides-measurements-pdfs.html
21. https://www.fareham.gov.uk/planning/measuring_plans.aspx
22. https://helpx.adobe.com/acrobat/desktop/edit-documents/geospatial-pdfs/change-measurements.html
23. https://acrobatusers.com/forum/architecture-engineering-and-construction/measuring-tool/
24. https://community.adobe.com/questions-9/change-measure-tool-permanently-to-metric-1260141
25. https://community.adobe.com/questions-9/measure-tool-how-do-i-edit-measurement-manually-1259532
26. https://www.youtube.com/watch?v=MXOxAvVp5Ns
27. https://www.youtube.com/watch?v=LBMu5M9GV3I
28. https://community.adobe.com/questions-12/measuring-tool-1507240
29. https://www.pdflib.com/pdf-knowledge-base/pdf-20/new-capabilities/
30. https://www.adobe.com/learn/acrobat/web/gather-feedback-pdf-comments
31. https://helpx.adobe.com/ee/acrobat/how-to/gather-feedback-pdf-comments.html
32. https://www.adobe.com/acrobat/features/share-and-review-pdfs.html
33. https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/manage-reviews/track-reviews.html
34. https://community.adobe.com/questions-9/pdf-workflow-sequential-commenting-by-multiple-reviewers-revision-safe-1305588
35. https://community.adobe.com/questions-9/adobe-acrobat-shared-reviews-permissions-and-commenting-1652120
36. https://resources.ascented.com/ascent-blog/technical-writing-tip-reviewing-comments-in-adobe-acrobat
37. https://helpx.adobe.com/acrobat/using/compare-documents.html
38. https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/use-stamps/add-modify-stamps.html
39. https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/use-stamps/create-custom-stamps.html
40. https://helpx.adobe.com/acrobat/desktop/share-and-review-documents/use-stamps/automate-date-time.html
41. https://controller.vpfa.fsu.edu/sites/default/files/documents/Quality%20Assurance/Setting%20Up%20a%20Dynamic%20Stamp.pdf
42. https://helpx.adobe.com/acrobat/desktop/get-started/learn-the-basics/overview.html
43. https://helpx.adobe.com/acrobat/desktop/work-with-pdf-forms/create-forms/convert-to-forms.html
44. https://helpx.adobe.com/reader/desktop/fill-forms.html
45. https://helpx.adobe.com/acrobat/using/collecting-pdf-form-data.html
46. https://helpx.adobe.com/acrobat/desktop/get-started/learn-the-basics/workspace.html
47. https://svsu.teamdynamix.com/TDClient/1949/Portal/KB/Article/156603/Adobe-Acrobat-New-Experience-Interface
48. https://community.adobe.com/questions-9/acrobat-2023-how-to-revert-to-classic-gui-user-interface-1285215
49. https://pubcom.com/blog/acrobatnew
50. https://helpx.adobe.com/acrobat/using/exporting-pdfs-file-formats.html
51. https://helpx.adobe.com/acrobat/web/share-review-and-export/export-and-print/export-pdf-overview.html
52. https://www.adobe.com/devnet-docs/acrobatetk/tools/PrefRef/Windows/Annots.html
53. https://helpx.adobe.com/acrobat/using/action-wizard-acrobat-pro.html
54. https://evermap.com/ActionWizardX.asp
55. https://acrobatusers.com/actions-exchange
56. https://pdfautomationstation.substack.com/p/the-acrobat-action-wizard
57. https://helpx.adobe.com/acrobat/kb/automation-and-document-workflows.html
58. https://developer.adobe.com/acrobat-pdfl-sdk/guides/jsdevguide/js-dev-rma
59. https://helpx.adobe.com/acrobat/desktop/edit-documents/apply-pdf-actions/pdf-actions.html
60. https://helpx.adobe.com/acrobat/desktop.html
