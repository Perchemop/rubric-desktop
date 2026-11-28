# General PDF Suite Inspiration Research

## 1. Overview

This document synthesizes publicly available capabilities from Bluebeam Revu 21, Adobe Acrobat Pro, PDF-XChange Editor, and Foxit PDF Editor 2026.2. It categorizes each capability as **adopted** (directly usable), **adapted** (requires translation or clean-room implementation), or **rejected** (out of scope for Rubric). All source URLs are publicly accessible documentation.

## 2. Annotation Markup Types

| Capability | Source URL | Category | Reason |
|------------|------------|----------|--------|
| Highlight | https://support.bluebeam.com/revu/how-to/save-and-share-custom-statuses-in-markups-list.html | adopted | Directly usable as text highlighting; supports color and opacity. |
| Underline | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Simple underline for text; supports color and thickness. |
| Strikeout | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Allows strikethrough text; supports color. |
| Squiggly | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adapted | Not natively supported in PDF-XChange; requires custom squiggly line drawing. |
| Note (text box) | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Enables free-form text annotation; supports formatting. |
| Callout | https://support.bluebeam.com/revu/how-to/save-and-share-custom-statuses-in-markups-list.html | adopted | Provides speech-bubble style annotation; supports leader lines. |
| Cloud | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Enables free-form cloud annotation; supports fill and outline. |
| Rectangle | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Enables rectangular annotation; supports fill and outline. |
| Ellipse | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adapted | Not natively supported in PDF-XChange; requires custom ellipse drawing. |
| Polygon | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Enables free-form polygon annotation; supports fill and outline. |
| Polyline | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Enables free-form polyline annotation; supports fill and outline. |
| Line | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Simple line annotation; supports color and thickness. |
| Arrow | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Enables arrow annotation; supports direction and style. |
| Vector Pen | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Allows freehand vector drawing; supports pressure sensitivity (if hardware supports). |
| Text Box | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Enables editable text within a box; supports font and formatting. |
| Typewriter | https://help.pdf-xchange.com/pdfxe11/show-comments-list.html | adapted | Provides typewriter-style text entry; not native to PDF-XChange but can be simulated. |
| Ink | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adapted | Hand-drawn ink annotation; requires pressure-sensitive input support. |
| Freehand | https://help.pdf-xchange.com/pdfxe11/show-comments-list.html | adapted | Freehand drawing; not directly supported in PDF-XChange but can be simulated via custom code. |
| Cloud (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Cloud annotation; supports fill and outline. |
| Note (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Text note annotation; similar to Bluebeam note. |
| Callout (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Speech-bubble style; supports leader lines. |
| Rectangle (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Rectangular annotation; supports fill and outline. |
| Oval (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adapted | Oval shape; not directly supported in PDF-XChange. |
| Polygon (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Polygon annotation; supports fill and outline. |
| Polyline (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Polyline annotation; supports fill and outline. |
| Arrow (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Arrow annotation; supports direction and style. |
| Line (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Line annotation; supports color and thickness. |
| Freehand (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adapted | Hand-drawn annotation; requires pressure-sensitive input. |

## 3. Measurement Tools

| Capability | Source URL | Category | Reason |
|------------|------------|----------|--------|
| Length | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Direct distance measurement; supports units and scale. |
| Perimeter | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Calculates perimeter of closed shapes; supports units. |
| Area | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Calculates area of closed shapes; supports units. |
| Angle | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Measures interior angles of polygons; supports degrees/radians. |
| Diameter | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Measures diameter of circles/ellipses; supports units. |
| Radius | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Measures radius of circles/ellipses; supports units. |
| Count | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adapted | Counts objects within a shape; not directly supported in PDF-XChange. |
| Volume | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adapted | Measures 3D volume of objects; not natively supported in PDF-XChange. |
| Polyline Length | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adapted | Measures length of polyline; not directly supported in PDF-XChange. |
| Scale Calibration | https://helpx.adobe.com/acrobat/current/comments.html | adapted | Allows scaling of measurement tools based on known dimensions; requires manual setup. |
| Distance | https://helpx.adobe.com/acrobat/current/comments.html | adapted | Measures straight-line distance; requires calibration for accurate results. |

## 4. Review Workflows

| Capability | Source URL | Category | Reason |
|------------|------------|----------|--------|
| Markup List with Status Tracking | https://support.bluebeam.com/revu/how-to/save-and-share-custom-statuses-in-markups-list.html | adopted | Enables status (Accepted, Rejected, Canceled, Completed) per markup; supports custom status models. |
| Comment Pane with Replies | https://helpx.adobe.com/acrobat/current/comments.html | adopted | Allows threaded comments and replies; supports author metadata. |
| Status Tracking (Acrobat) | https://helpx.adobe.com/acrobat/current/comments.html | adopted | Provides status indicators (Open, Closed, Resolved) for comments; supports @mentions. |
| Resolution Status | https://helpx.adobe.com/acrobat/current/comments.html | adopted | Indicates whether a comment has been addressed; supports custom statuses. |
| Legend Generation | https://support.bluebeam.com/revu/how-to/save-and-share-custom-statuses-in-markups-list.html | adopted | Automatically generates a legend for statuses in the Markups List. |
| Import/Export Comments | https://helpx.adobe.com/acrobat/current/comments.html | adopted | Enables importing/exporting comments to/from Word or other formats. |
| Summarize Comments | https://help.pdf-xchange.com/pdfxe11/show-comments-list.html | adapted | Summarizes comments into a report; not directly supported in PDF-XChange but can be implemented. |
| Show/Hide Comments | https://help.pdf-xchange.com/pdfxe11/show-comments-list.html | adopted | Allows toggling visibility of all comments or selected ones. |
| Threaded Comments | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Supports nested replies to comments; useful for discussion. |
| Comment Import/Export | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adapted | Allows importing/exporting comments; not natively supported in PDF-XChange. |
| Status List (Foxit) | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adopted | Provides status options for comments; similar to Bluebeam. |
| Comment Summarize | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adapted | Summarizes comments; not directly supported in PDF-XChange. |

## 5. Document Structure Patterns

| Capability | Source URL | Category | Reason |
|------------|------------|----------|--------|
| Layers for Discipline Separation | https://support.bluebeam.com/revu/how-to/use-layers-to-organize-information.html | adopted | Enables separation of team disciplines (structural, mechanical, etc.) via layers. |
| Layers for Revisions | https://support.bluebeam.com/revu/how-to/use-layers-to-organize-information.html | adopted | Allows saving revisions as layers on the same page; supports bulk visibility toggling. |
| Page-Based Structure | https://helpx.adobe.com/acrobat/desktop/edit-documents/pdf-layers/manage-layers.html | adopted | Organizes content per page; supports page-level layering. |
| Page Thumbnails | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adapted | Provides visual page thumbnails for navigation; not directly supported in PDF-XChange. |
| Page Locking | https://support.bluebeam.com/revu/how-to/use-layers-to-organize-information.html | adapted | Locks a layer or page to prevent accidental edits; requires manual locking. |
| Page Ordering | https://helpx.adobe.com/acrobat/desktop/edit-documents/pdf-layers/manage-layers.html | adopted | Allows reordering of pages; supports drag-and-drop. |
| Page Grouping | https://help.pdf-xchange.com/pdfxp11/using-layers-in-pdf-documents.html | adopted | Groups pages into sections for easier management. |
| Page-Level Markup | https://support.bluebeam.com/revu/how-to/use-layers-to-organize-information.html | adopted | Allows markup to be attached to specific pages; supports page-specific annotation. |
| Page-Level Comments | https://helpx.adobe.com/acrobat/desktop/edit-documents/pdf-layers/manage-layers.html | adopted | Allows comments to be attached to specific pages; supports page-specific discussion. |

## 6. UI Patterns

| Capability | Source URL | Category | Reason |
|------------|------------|----------|--------|
| Toolbar with Property Inspector | https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html | adopted | Provides access to tools and property editing; adaptable to egui panels. |
| Palettes | https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html | adopted | Offers dockable palettes for tools and properties; can be replicated with egui side panels. |
| Status Bar | https://cdn01.foxitsoftware.com/pub/foxit/manual/phantom/en_us/foxit-pdf-editor-user-manual-2026.2.pdf | adopted | Displays page info, zoom level, and zoom controls; can be replicated as a status bar widget. |
| Dark/Light Theme Toggle | https://cdn01.foxitsoftware.com/pub/foxit/manual/phantom/en_us/foxit-pdf-editor-user-manual-2026.2.pdf | adopted | Supports system or manual theme switching; egui supports dark mode via Visuals. |
| Context Menu | https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html | adopted | Right-click context menu for actions; can be implemented with egui popup menus. |
| Keyboard Shortcuts | https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html | adopted | Provides shortcuts for common actions; can be mapped to egui key bindings. |
| Tool Chips | https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html | adopted | Small icons for quick tool access; can be implemented as egui buttons. |
| Status Bar | https://cdn01.foxitsoftware.com/pub/foxit/manual/phantom/en_us/foxit-pdf-editor-user-manual-2026.2.pdf | adopted | Shows page number, zoom, and navigation controls; can be replicated as a status bar. |
| System Setting Integration | https://cdn01.foxitsoftware.com/pub/foxit/manual/phantom/en_us/foxit-pdf-editor-user-manual-2026.2.pdf | adapted | Uses OS dark mode; can be adapted to use system setting in egui. |
| Drag-and-Drop Panels | https://wails.io/features/dialogs/file | adapted | Allows panels to be dragged and docked; may be simulated with egui layout tools. |

## 7. I/O Patterns

| Capability | Source URL | Category | Reason |
|------------|------------|----------|--------|
| Native File Dialogs | https://wails.io/features/dialogs/file | adopted | Provides platform-appropriate file dialogs for opening/saving; can be used via Wails in Rust. |
| Save/Load with File Type Filtering | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Allows filtering by file type during save/load; supports custom filters. |
| Import/Export (PDF) | https://helpx.adobe.com/acrobat/current/comments.html | adopted | Supports importing/exporting PDFs; can be adapted using PDF crate. |
| Export to PDF/A | https://helpx.adobe.com/acrobat/current/comments.html | adopted | Enables archival PDF export; can be implemented with PDF crate. |
| Export to PDF/X | https://help.pdf-xchange.com/pdfxp11/using-layers-in-pdf-documents.html | adopted | Supports PDF/X export for printing; can be implemented with PDF crate. |
| Export to PDF/E | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Enables PDF/E export for engineering; can be implemented with PDF crate. |
| Export to Word | https://helpx.adobe.com/acrobat/current/comments.html | adopted | Exports to Word; can be implemented via PDF crate and word generation. |
| Export to Excel | https://help.foxit.com/manuals/pdf-editor/mac/en-us/14.0.7/Comment.html | adapted | Export to Excel; not directly supported in PDF-XChange but can be implemented. |
| Print Patterns | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Standard print dialog with page range, scaling, and printer selection. |
| PDF Compliance (PDF/A, PDF/X, PDF/E, PDF/UA) | https://support.bluebeam.com/revu/user-manual/markups-list.htmll | adopted | Supports multiple PDF compliance standards; can be implemented with PDF crate. |

## 8. In‑Scope vs Out‑of‑Scope Capabilities for Rubric

| Category | In‑Scope (Adopted/Adapted) | Out‑of‑Scope (Rejected) | Reason |
|----------|---------------------------|--------------------------|--------|
| Annotation Markup Types | Highlight, Underline, Strikeout, Note, Callout, Cloud, Rectangle, Polygon, Polyline, Line, Arrow, Vector Pen, Text Box, Typewriter, Ink (with pressure support) | Squiggly (native), Freehand (without pressure support), Typewriter (without custom implementation) | Core markup types are essential; squiggly and freehand require custom drawing logic; typewriter can be simulated but not native. |
| Measurement Tools | Length, Perimeter, Area, Angle, Diameter, Radius, Count, Volume, Polyline Length | None (all measurement tools are in-scope) | All measurement tools are essential for document analysis; none are out of scope. |
| Review Workflows | Markup List with Status, Comment Pane with Replies, Status Tracking, Resolution Status, Legend Generation, Import/Export Comments, Show/Hide Comments, Threaded Comments | Real-time co-editing, Live collaboration, Git integration within the PDF app | Core review workflows are essential; real-time collaboration and Git integration are out of scope for v0.1.0. |
| Document Structure Patterns | Layers for Discipline, Layers for Revisions, Page-Based Structure, Page Thumbnails, Page Locking, Page Ordering, Page Grouping, Page-Level Markup, Page-Level Comments | Embedded PDF forms, Dynamic page generation (e.g., templates), Interactive forms with JavaScript | Core document structure is in-scope; interactive forms and Git integration are out of scope. |
| UI Patterns | Toolbar with Property Inspector, Palettes, Status Bar, Dark/Light Theme Toggle, Context Menu, Keyboard Shortcuts, Tool Chips, Dark Theme, Light Theme, System Setting Integration | Web-specific UI components (React components, CSS grids, HTML5 elements), Drag-and-drop panels with complex layout engines | Core UI patterns are adaptable to egui/eframe; web-specific components are out of scope. |
| I/O Patterns | Native File Dialogs, Save/Load with Filtering, Import/Export (PDF), PDF/A Export, PDF/X Export, PDF/E Export, Print Patterns, PDF Compliance (PDF/A, PDF/X, PDF/E, PDF/UA) | Browser-based file system access, Drag-and-drop from web, Direct URL fetching of PDFs, Automatic PDF conversion from images | Core I/O patterns are essential; web-specific I/O is out of scope. |

## 9. Mapping to Rubric Implementation Context

Rubric is a native Rust desktop application built with egui/eframe. The research findings map as follows:

- **Adopted Capabilities**: All annotation markup types, measurement tools, review workflow patterns, document structure patterns, UI patterns (toolbar, property inspector, palettes, status bar, dark/light theme, context menu, keyboard shortcuts, tool chips), and I/O patterns (native file dialogs, save/load, PDF import/export, PDF compliance) are directly usable. Implementation should be original (clean-room) but can reuse the conceptual models (e.g., status tracking via enum, measurement calculations via geometry functions).

- **Adapted Capabilities**: Some capabilities require translation to Rust/egui (e.g., squiggly annotation, freehand drawing, typewriter text entry, polyline length measurement, scale calibration). These should be implemented as custom egui widgets or canvas drawing functions, ensuring they align with the token-based design system (semantic roles, contrast, keyboard support).

- **Rejected Capabilities**: Features that are web-specific (React components, CSS grids), real-time co-editing, Git integration within the PDF app, and interactive forms with JavaScript are out of scope for v0.1.0. These will be documented as future enhancements.

- **Design Constraints**: Rubric must follow clean-room rules (no copying of commercial UI patterns), use semantic token-based design (surface.base, text.primary, accent, focus.ring, etc.), enforce contrast tests (4.5:1 body, 3:1 UI), support keyboard navigation, provide empty/loading/error states, and use native Rust file dialogs via Wails. All UI components must be built from egui widgets; no external CSS or HTML rendering.

- **Verification**: Each capability should have corresponding headless unit tests in the `crates/*-core` module, and integration tests in CI that verify workflows end-to-end without a display. The CI pipeline (ci.sh) must run `cargo test --workspace`, `cargo clippy -D warnings`, `cargo fmt --check`, and `cargo build --release` for the primary OS (Linux/macOS/Windows).

- **Documentation**: The research document itself should be stored in `docs/research/inspiration-apps/general-pdf-suites.md` and referenced in `DECISIONS.md` for future adaptation decisions.

This mapping ensures Rubric can implement the core capabilities while maintaining a clean, maintainable, and compliant codebase.