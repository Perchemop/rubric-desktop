# Scope — Rubric v1

Source of truth for "is this feature in the first major version?". Feature
IDs refer to the original requirement set in `docs/spec/README.md`.
Statuses live in `REQUIREMENTS_TRACEABILITY.md`.

## In scope

| Area | What ships | Spec IDs |
|------|------------|----------|
| Markup tools | Highlight, strikeout, underline, squiggly, text box, callout, note, stamp, rectangle, ellipse, cloud, polygon, polyline, line, arrow, ink | F-101–F-112 |
| Selection & editing | Select, move, delete, copy properties, z-order, in-canvas text editing | F-104/F-105, F-113–F-115 |
| Measurements | Length, perimeter, area, angle, diameter, radius, count with per-page scale calibration and unit selection | F-201–F-209 |
| Review workflow | Statuses (Open/In review/Approved/Rejected), subject, author, comments, legend | F-301–F-305 |
| Document structure | Pages, layers (visibility/lock), multi-document projects | F-401–F-404, F-116 |
| Persistence | `.rubric.json` sidecar (pretty JSON, UUIDs, timestamps), autosave, `schema_version` guard | F-501/F-502/F-507 |
| I/O | Native open/save dialogs, recent files, PDF import + page rendering, image export + print | F-503–F-506 |
| UI/UX polish | Shortcuts, page navigation, dark theme, context menus, icons/branding | F-601–F-605 |
| Packaging | Cross-platform release builds | F-7xx |

## Out of scope (non-goals) — v1

- Cloud storage, sync, accounts, or any collaboration server.
- Editing PDF *content* itself (Rubric only overlays non-destructive markups).
- Proprietary or encrypted document formats.
- Mobile and web targets (WASM is a later evaluation, not a v1 commitment).
- Digital signatures.
- Embedding markups back into the PDF file (MVP exports rendered images +
  print; PDF-embedded annotation export is excluded for now).
- Any network feature: the app is fully offline. `[Fact]`

## MVP definition

The MVP = core of Phases 2–3 per `IMPLEMENTATION_PLAN.md`: drag-to-draw
markups, selection/move/undo, measurements with scale, review statuses, and
`.rubric.json` persistence — all verified by CI before release (guide §9).
