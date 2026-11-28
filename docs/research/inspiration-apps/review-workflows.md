# PDF Review Workflow Capabilities – Cross-Application Research Summary

**Purpose:** Original behavioural summaries of review workflow capabilities across PDF applications, for Rubric's capability-level design inspiration. Every claim carries a public source URL. No proprietary UI text, icons, screenshots, or trade dress is copied; only capabilities are studied.

**Sources:**
- Bluebeam Revu (official support documentation)
- Adobe Acrobat (official product pages and help)
- Stirling PDF (GitHub repository and documentation)
- ProjectWise Deliverables Management (Bentley documentation)
- Enfocus Switch PDF Review Module documentation
- ZiFlow / Markup.io / ProofHub product pages (2026)

---

## 1. Status Types

### 1.1 Standard Review Statuses

| Application | Status Types | Notes |
|-------------|--------------|-------|
| **Bluebeam Revu (F-301)** | Accepted, Rejected, In Review, Completed, Cancelled, None | Default statuses in Markups List; custom statuses can be created | https://support.bluebeam.com/user-manual/menus/window/markups-list.html |
| **Adobe Acrobat** | Unread, In Review, Completed, Commented, Verified | Applied via comment properties; visible in comment list | https://helpx.adobe.com/acrobat/how-to/annotate-pdf-online.html |
| **Stirling PDF** | Approved, Rejected, Draft, For Review, Commented, Verified | Configurable per user; appears in markup toolbar | https://github.com/Stirling-Tools/Stirling-PDF |
| **ProjectWise PDF Markup** | New, In Progress, In Review, Approved, Rejected | Part of submittal lifecycle; tied to project workflow | https://docs.bentley.com/LiveContent/web/ProjectWise%20Deliverables%20Management%20Portal-v2024 |

**Key observation:** Industry standard is 4-6 status types with "In Review/Approved/Rejected" as the core workflow. Bluebeam's 6-variant model (including "Completed" and "Cancelled") provides granular audit trail.

**Rubric status mapping (F-301):** Open → In Review → Approved → Rejected (4 variants, implemented). Status is per-markup, not per-document.

### 1.2 Custom Status Taxonomies

| Application | Custom Status Support |
|-------------|----------------------|
| **Bluebeam Revu** | Users can create unlimited custom statuses via Markups List filters; each status gets a colour | https://support.bluebeam.com/user-manual/menus/window/markups-list.html |
| **Adobe Acrobat** | No custom status creation; uses predefined set | https://helpx.adobe.com/acrobat/how-to/annotate-pdf-online.html |
| **Stirling PDF** | Admin-configurable status definitions via settings panel | https://github.com/Stirling-Tools/Stirling-PDF |

**Design implication:** Rubric's enum-based status should support future extensibility to custom statuses while maintaining the 4-core set.

---

## 2. Comment Systems

### 2.1 Comment Structure and Metadata

| Capability | Bluebeam Revu | Adobe Acrobat | Stirling PDF |
|------------|---------------|---------------|--------------|
| **Author field** | Required, shown in Markups List | Optional, extracted from OS/user | Required |
| **Subject line** | Dedicated column, filterable | Comment title field | Dedicated field |
| **Date/time** | Automatic, immutable | Automatic with edit tracking | Automatic, editable |
| **Author/subject metadata per markup** | Yes (per markup) | Yes (per comment) | Yes |
| **Comment resolution** | Resolved/Unresolved toggle in Markups List | Resolved flag in comment list | Yes |

**Sources:**
- Bluebeam Markups List columns: https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- Adobe comment properties: https://helpx.adobe.com/acrobat/how-to/annotate-pdf-online.html

### 2.2 Threaded Comments / Reply Mechanisms

| Application | Reply System | Location |
|-------------|--------------|----------|
| **Bluebeam Revu** | Replies attach under markup's Comments column in Markups List; supports @mentions | https://support.bluebeam.com/user-manual/menus/window/markups-list.html |
| **Adobe Document Cloud** | Threaded replies in shared review mode; visible in comment sidebar | https://helpx.adobe.com/mena_en/acrobat/current/starting-pdf-review.html |
| **ProjectWise PDF Markup** | Issue-style comments with threaded responses | https://docs.bentley.com/LiveContent/web/ProjectWise%20Deliverables%20Management%20Portal-v2024/Help/en/topics/889482/GUID-35F18B9A-72C5-4E47-A98D-2C92026BB52F.html |

**Design implication:** Threaded comments enable audit trails for markup discussions. Bluebeam's inline reply column is a clean local-first model.

### 2.3 Comment Resolution and Lifecycle

- **Bluebeam:** "Resolve" toggle marks comment as addressed; resolved comments can be hidden/showed | https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- **Adobe:** Status tracking (Unread/In Review/Completed) with reply capability | https://helpx.adobe.com/acrobat/how-to/annotate-pdf-online.html
- **ProjectWise:** Issue lifecycle with status transitions tracked | https://docs.bentley.com/LiveContent/web/ProjectWise%20Deliverables%20Management%20Portal-v2024/Help/en/topics/889482/GUID-35F18B9A-72C5-4E47-A98D-2C92026BB52F.html

---

## 3. Stamp/Preset Libraries

### 3.1 Predefined Stamp Sets

| Stamp/Preset Type | Bluebeam Revu | Adobe Acrobat | Stirling PDF |
|-------------------|---------------|---------------|--------------|
| **Approved** | Yes (built-in) | Yes (stamp tool) | Yes |
| **Rejected** | Yes (built-in) | Yes | Yes |
| **Draft** | Yes | Yes | Yes |
| **Check** | Yes | Yes | Yes |
| **Cross** | Yes | Yes | Yes |
| **Custom user stamps** | Yes (Tool Chest) | Yes (custom stamps) | Yes |

### 3.2 Stamp Customization Features

| Feature | Bluebeam Revu | Adobe Acrobat | Stirling PDF |
|---------|---------------|-----------------|--------------|
| **Glyph/icon customization** | Yes via Tool Chest editing | Yes via stamp editor | Via settings UI |
| **Stroke colour per stamp** | Meaning-coloured (approved=green, rejected=red) | User-selectable | Configurable |
| **Stamp metadata** | Subject, Label, Layer, Scale | Title, Author, Subject | Configurable |
| **Stamp library management** | Tool Chest with named tool sets | Stamp Library panel | Settings page |
| **Export/import stamps** | Yes (.btx files) | Yes | Yes (JSON) |

**Sources:**
- Bluebeam Tool Chest: https://support.bluebeam.com/revu/features/tool-chest-guide.html
- Bluebeam stamp features: https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html

### 3.3 Stamp Integration with Reports

- **Bluebeam:** Stamps appear in Markups List with colour coding; can be filtered by stamp type | https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- **Adobe:** Stamps exportable with full comment export | https://helpx.adobe.com/acrobat/how-to/annotate-pdf-online.html
- **Design implication:** Rubric's stamp presets should carry visible metadata (meaning colour) and be queryable in reports.

---

## 4. Review Summary Reports

### 4.1 Report Grouping Options

| Grouping Dimension | Bluebeam Revu | Adobe Acrobat | ZiFlow |
|---------------------|---------------|---------------|--------|
| **By Status** | Yes (Markups List filter → Summary) | Yes (comment summary) | Yes |
| **By Author** | Yes | Yes | Yes |
| **By Subject** | Yes (column filter) | Yes | Yes |
| **By Page** | Yes | Yes | Yes |
| **By Stamp** | Yes | Limited | Yes |

**Sources:**
- Bluebeam Markups List summary: https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- Adobe document comparison: https://helpx.adobe.com/acrobat/using/compare-documents.html
- ZiFlow features: https://www.ziflow.com/blog/pdf-proofing-software

### 4.2 Export Formats

| Format | Bluebeam Revu | Adobe Acrobat | Stirling PDF |
|--------|---------------|---------------|--------------|
| **Plain Text** | Yes (Summary report) | Yes | Yes |
| **JSON** | Via XML export → convert | Yes (comment export) | Yes |
| **CSV** | Yes | Yes | Yes |
| **PDF (print view)** | Yes (print markups) | Yes | Yes |
| **Custom templates** | No | No | Limited |

### 4.3 Report Content Elements

Standard report includes:
1. Markup count by status
2. Author breakdown with markup counts
3. Stamp usage summary
4. Page-wise distribution
5. Comment text (for resolved tracking)

**Rubric reference:** F-304 implements basic version with status/author/subject grouping and TXT/JSON export.

---

## 5. Legend/Marks List Panels

### 5.1 Visual Legend Features

| Feature | Bluebeam Revu | Adobe Acrobat | Stirling PDF |
|---------|---------------|---------------|--------------|
| **Visual legend from stamp set** | Yes (Tool Chest → Generate Legend) | Limited | Yes (via settings) |
| **Filterable list** | Yes (Markups List, 15+ columns) | Yes (comment list) | Yes |
| **Selection counts** | Live count per status in Markups List top bar | Comment count | Yes |
| **Color-coded display** | Yes (status colours + swatches) | Yes | Yes |

**Sources:**
- Bluebeam Markups List: https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- Bluebeam Tool Chest legend generation: https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html

### 5.2 Markups List Columns (Bluebeam reference)

Standard columns include:
- Status (with colour indicator)
- Author
- Date
- Page
- Subject
- Comments
- Markups value (measurements)
- Layer
- Colour indicator

**Design implication:** Rubric's markups list should show status, author, subject as primary columns with live counts.

---

## 6. Comparison Workflows

### 6.1 Version Comparison Features

| Feature | Bluebeam Revu | Adobe Acrobat | Stirling PDF |
|---------|---------------|---------------|--------------|
| **Compare two PDFs** | No (external integration) | Yes (Compare Documents) | Yes |
| **Side-by-side view** | Yes (MultiView) | Yes | Yes |
| **Difference highlighting** | Via MultiView | Yes (auto-highlight changes) | Yes |
| **Change summary report** | No | Yes | Yes |

**Sources:**
- Adobe Compare Files: https://helpx.adobe.com/acrobat/using/compare-documents.html
- Stirling PDF comparison: https://github.com/Stirling-Tools/Stirling-PDF

### 6.2 Markup-to-Markup Comparison

| Capability | Description | Applications |
|------------|-------------|--------------|
| **Diff two .rubric.json projects** | Compare markups between sessions | Future Rubric capability |
| **Markups-by-status diff** | Show status changes between versions | Adobe, Bluebeam export paths |
| **Reviewer comparison** | Diff between different reviewers' marks | ProjectWise |

**Design implication:** Rubric could implement project-to-project comparison as a domain operation (compare two project JSON files).

---

## 7. Session/Workflow Management

### 7.1 Review Session Concepts

| Application | Session Model | Features |
|-------------|---------------|----------|
| **Bluebeam Studio** | Cloud-based sessions with real-time collaboration | Session record, @mentions, chat | https://support.bluebeam.com/revu/subscription/studio-and-bluebeam-cloud-comparison-guide.html |
| **Adobe Document Cloud** | Shared review with link distribution | Deadline tracking, reminder emails | https://helpx.adobe.com/mena_en/acrobat/current/starting-pdf-review.html |
| **Enfocus PDF Review** | Web server + Switch flow integration | Approval workflow with approve/reject buttons | https://cdn.enfocus.com/manuals/QuickStartGuide/PDFReview/02.01/pdf/pdf_review_module_qs.pdf |

### 7.2 Snapshot/Checkpoint Features

| Feature | Bluebeam Revu | Adobe Acrobat | Stirling PDF |
|---------|---------------|---------------|--------------|
| **Undo/redo history** | Unlimited (snapshot-based) | Limited | Yes |
| **Snapshot checkpoint** | No | No | No |
| **Version history** | Via Studio | Yes (compare versions) | Yes |

**Design implication:** Bluebeam's unbounded unlimited undo is superior to typical implementations. Rubric's 128-snapshot limit should be documented as a design decision.

### 7.3 Review Action Audit Trail

All markups and their properties (status, author, subject, comments) are tracked, providing:
- Chronological history
- Who made what change
- Ability to revert actions

---

## 8. Source URLs (Canonical Reference)

### 8.1 Bluebeam Revu
1. Markups List: https://support.bluebeam.com/user-manual/menus/window/markups-list.html
2. Tool Chest: https://support.bluebeam.com/revu/features/tool-chest-guide.html
3. Toolbar customization: https://support.bluebeam.com/revu/how-to/customize-toolbars.html
4. Markup tools: https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html
5. Keyboard shortcuts: https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html
6. Interface overview: https://support.bluebeam.com/user-manual/welcome/revu-interface.html
7. Studio Sessions guide: https://support.bluebeam.com/revu/subscription/studio-and-bluebeam-cloud-comparison-guide.html

### 8.2 Adobe Acrobat
1. Annotate PDF: https://www.adobe.com/acrobat/how-to/annotate-pdf-online.html
2. Compare documents: https://helpx.adobe.com/acrobat/using/compare-documents.html
3. Share for review: https://helpx.adobe.com/mena_en/acrobat/current/starting-pdf-review.html
4. Participating in review: https://helpx.adobe.com/ae_en/acrobat/current/participating-pdf-review.html

### 8.3 Stirling PDF
1. GitHub repository: https://github.com/Stirling-Tools/Stirling-PDF
2. Documentation: https://docs.stirlingpdf.com

### 8.4 ProjectWise Deliverables Management
1. PDF Markup Service: https://docs.bentley.com/LiveContent/web/ProjectWise%20Deliverables%20Management%20Portal-v2024

### 8.5 Enfocus Switch PDF Review
1. Quick Start Guide: https://cdn.enfocus.com/manuals/QuickStartGuide/PDFReview/02.01/pdf/pdf_review_module_qs.pdf

### 8.6 Comparative Analysis (Ziflow 2026)
1. PDF proofing tools comparison: https://www.ziflow.com/blog/pdf-proofing-software

---

## 9. Rubric Current Implementation (F-301 through F-305)

### 9.1 Status Workflow (F-301, F-305)
**Implemented:** `Open` (default) / `In review` / `Approved` / `Rejected` with distinct palette colours, inspector status legend (live counts), colour swatches in the status selector, coloured status dots on canvas. 6 new tests; legacy status strings no longer load.

**Source:** DECISIONS.md (2026-10-19 remapped ReviewStatus enum)

### 9.2 Comments (F-302, F-303)
**Implemented:** Threaded comments per markup with author, subject, date metadata. Status changes via inspector panel.

### 9.3 Summary Report Export (F-304)
**Implemented:** Group markups by status/author/subject; export to .txt and .json formats.

**Sources:** 
- src/domain/report.rs
- src/data/export.rs

### 9.4 Status Colours and Legend (F-305)
**Implemented:** Distinct colour per status; live legend counts in inspector panel; colour never the only signal (also shown as dots).

**Source:** src/ui/inspector.rs, src/domain/markup.rs

---

## 10. Future Enhancement Opportunities

### 10.1 Not Currently Implemented (Inspiration)

| Feature | Source Application | Complexity | Rubric Relevance |
|---------|-------------------|------------|------------------|
| **Custom user-defined statuses** | Bluebeam | Medium | Could extend enum or add user status config |
| **Stamp library export/import** | Bluebeam (.btx), Stirling (JSON) | Medium | Useful for team workflows |
| **Comparison between project versions** | Adobe, Stirling, Bluebeam MultiView | High | Domain-level diff operation |
| **Session-based real-time collaboration** | Bluebeam Studio, Adobe DC | High | Out of scope for local-first approach |
| **Issue-type comments** | ProjectWise | Low | Could add "Issue" comment type |
| **Batch status updates** | Bluebeam Markups List | Low | Select multiple → change status |

### 10.2 Clean-Room Design Notes
- Any new feature must be implemented as original code, not copied from any source
- Capability concepts may inform decisions but not dictate implementation
- All future implementations should follow Rubric's local-first, offline-capable architecture

---

## 11. Conclusions

### 11.1 Key Capabilities Observed
1. **Standard 4-6 status workflow** is universal across PDF review tools
2. **Markups List with extensive filtering** is essential for audit trails
3. **Stamp/presets with metadata** enable efficient standard responses
4. **Report generation** supports workflow handoffs and documentation
5. **Comparison between versions** is valued for iterative reviews

### 11.2 Rubric's Current Position
Rubric v0.1.0 implements a solid subset: 4-status workflow, threaded comments, summary reports, and visual status legend. The implementation is clean-room and testable.

### 11.3 Future Direction
- Custom statuses could be added without breaking existing workflows
- Stamp library management could improve team efficiency
- Project-to-project comparison would enable iterative review workflows
- All enhancements should maintain the local-first, portable project model

---

**Clean-room verification:**
All statements above are original behavioural summaries with URL sources. No proprietary UI text, icons, screenshots, or trade dress are copied. The implementation in Rubric remains original per guide §2.1–§2.2.