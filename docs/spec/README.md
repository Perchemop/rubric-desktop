# Rubric Spec — Requirement Set

## Overview

Rubric is a free, local-first, cross-platform desktop app (Rust + egui/eframe) for marking up, reviewing and measuring PDF documents: users open a PDF, place non-destructive markups (highlights, notes, shapes, stamps) that are stored in a `.rubric.json` sidecar file and never alter the source PDF, run a review workflow with statuses/comments/author metadata, and take calibrated real-world measurements from a per-page scale. Everything runs offline — no cloud, no accounts, no collaboration server.

This requirement set is **original material authored for Rubric**. It is derived solely from general domain knowledge of document review and measurement workflows and publicly documented standards — not from any other product's documentation, UI, or behaviour.

## Conventions

- **ID groups**: F-1xx markups/tools · F-2xx measurements/scale · F-3xx review workflow · F-4xx navigation/pages · F-5xx persistence/I/O · F-6xx UI/UX polish · F-7xx packaging.
- **Priority** maps to `IMPLEMENTATION_PLAN.md`: `MVP` = core of Phases 2–3 (first usable release); `Ph 4` / `Ph 5` / `Ph 6` = scheduled in that phase; `later` = after Phase 6 / optional.
- **Traceability**: `REQUIREMENTS_TRACEABILITY.md` maps each area to R-S-001…R-S-017 and to the planned modules.
- Notes column records current state at adoption (2026-10-19).

## Feature inventory

### F-1xx — Markups and tools

| ID | Feature | Priority | Notes |
|---|---|---|---|
| F-101 | Highlight tool (semi-transparent band over text) | MVP (Ph 2) | core placement done |
| F-102 | Strikeout / underline / squiggly text markups | MVP (Ph 2) | types + rendering done |
| F-103 | Note (anchored comment pin) | MVP (Ph 2) | placement done; popover UX pending |
| F-104 | Text box | MVP (Ph 2) | placement done; in-canvas editing pending |
| F-105 | Callout (text box + pointer tail) | MVP (Ph 2) | placement done; text editing pending |
| F-106 | Rectangle and ellipse shapes | MVP (Ph 2) | done |
| F-107 | Cloud (revision cloud) outline | MVP (Ph 2) | drawn; path quality refine later |
| F-108 | Polyline and polygon (multi-point paths) | Ph 2 | domain types done; multi-point drawing pending |
| F-109 | Line and arrow | MVP (Ph 2) | done |
| F-110 | Ink freehand drawing | Ph 2 | domain type done; stroke accumulation pending |
| F-111 | Stamps (preset library: approved/rejected/draft style marks) | Ph 4 | Stamp kind exists; library content pending |
| F-112 | Drag-to-draw with live preview (bounds/segment/path/angle/ink) | Ph 2 | pending |
| F-113 | Select, move, delete markups | MVP (Ph 2) | done (move/select); delete pending wiring |
| F-114 | Copy properties and apply style to other markups | Ph 4 | pending |
| F-115 | Z-order (bring forward / send back) | Ph 4 | pending |
| F-116 | Layer visibility and lock | Ph 4 | `Markup.layer` field exists; UI pending |

### F-2xx — Measurements and scale

| ID | Feature | Priority | Notes |
|---|---|---|---|
| F-201 | Length measurement | MVP (Ph 2–3) | computation done; label pending |
| F-202 | Perimeter measurement | MVP (Ph 2–3) | computation done; label pending |
| F-203 | Area measurement | MVP (Ph 2–3) | computation + rescale fix done |
| F-204 | Angle measurement (three-point) | Ph 2 | computation done; three-point drawing pending |
| F-205 | Diameter and radius measurement | MVP (Ph 2–3) | computation done |
| F-206 | Count marker measurement | MVP (Ph 2–3) | computation done |
| F-207 | Calibrate / set scale (presets e.g. 1:1, 1 in = 1 ft, 1 in = 10 ft, plus custom ratio) | Ph 3 | domain scale done; editor UI pending |
| F-208 | Unit selection (document/page display unit) | Ph 3 | units domain done; dropdown pending |
| F-209 | Measurement labels on canvas + precision/format preferences | Ph 3 | pending |

### F-3xx — Review workflow

| ID | Feature | Priority | Notes |
|---|---|---|---|
| F-301 | Review statuses: Open / In review / Approved / Rejected | Ph 4 | status enum exists with a different variant set; remap planned |
| F-302 | Comments (thread per markup) | MVP (Ph 2) | done |
| F-303 | Author and subject metadata per markup/project | MVP (Ph 2) | done |
| F-304 | Summary report (markups grouped by status/author/subject, exportable) | Ph 5 | pending |
| F-305 | Status colours and legend | Ph 4 | defaults exist; legend pending |

### F-4xx — Navigation and pages

| ID | Feature | Priority | Notes |
|---|---|---|---|
| F-401 | Zoom (mouse wheel centred on cursor, fit, presets) | Ph 2 | button zoom done; wheel pending |
| F-402 | Pan (Space-drag / middle-drag) | Ph 2 | pending |
| F-403 | Page navigation (first/prev/next/jump to page) | Ph 5 | multi-page model exists; UI pending |
| F-404 | Page thumbnails sidebar | Ph 5 | pending |
| F-405 | Grid and snap-to-grid | Ph 6 | pending |
| F-406 | Rulers | Ph 6 | pending |

### F-5xx — Persistence and I/O

| ID | Feature | Priority | Notes |
|---|---|---|---|
| F-501 | `.rubric.json` sidecar save/load (pretty JSON, UUIDs, timestamps) | MVP | done |
| F-502 | Autosave | Ph 5 | settings/hook stub exists |
| F-503 | Recent files list | Ph 5 | pending |
| F-504 | PDF import and page rendering (non-destructive, sidecar kept) | Ph 5 | pending; pure-Rust crate first (see assumptions) |
| F-505 | Export (rendered image) and print | Ph 5 | pending |
| F-506 | Native open/save file dialogs (PDF + `.rubric.json`) | Ph 5 | pending |
| F-507 | Project schema versioning (`schema_version`) | later | pending |

### F-6xx — UI/UX polish

| ID | Feature | Priority | Notes |
|---|---|---|---|
| F-601 | Keyboard and tool shortcuts (Ctrl+S/Z/Shift+Z/Esc/Del, per-tool keys) | Ph 6 | core shortcuts done; tool keys pending |
| F-602 | Dark theme (light/dark toggle) | Ph 6 | done (View > Theme light/dark toggle, persisted preference) |
| F-603 | Context menus (canvas + inspector) | Ph 6 | pending |
| F-604 | Undo/redo (bounded history, 128 snapshots) | MVP | done |
| F-605 | Accessibility (readable contrast, keyboard-reachable controls) | later | pending review |

### F-7xx — Packaging

| ID | Feature | Priority | Notes |
|---|---|---|---|
| F-701 | Linux AppImage + desktop entry/icon | Ph 6 | pending |
| F-702 | macOS DMG | Ph 6 | pending |
| F-703 | Windows MSI | Ph 6 | pending |
| F-704 | CI release artifacts (tagged builds in GitHub Actions) | Ph 6 | CI skeleton exists |

## Non-goals

- Cloud sync, accounts, authentication, or any network service.
- Real-time collaboration or multi-user review sessions.
- Editing the content of the PDF itself (text/graphics stay untouched; Rubric only overlays markups stored outside the PDF).
- PDF-embedded annotation export in the MVP (sidecar is canonical; embedding is at most a later option).
- Proprietary or encrypted document formats.
- Mobile/web versions — desktop only.
- Digital signatures or certificate-based approval.

## Links

- Priorities → `IMPLEMENTATION_PLAN.md` phases.
- Requirement mapping → `REQUIREMENTS_TRACEABILITY.md` (R-S-001…R-S-017).
- Scope decisions/open questions → `docs/assumptions-and-questions.md`; decision log → `DECISIONS.md`.
