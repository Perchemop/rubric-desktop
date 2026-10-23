# Parity Tracker (guide §6)

Rubric is an **original** implementation. This file tracks **feature parity at
capability level**: what the reference products described in
`docs/inspiration.md` do, and where Rubric stands.

It is a summary view only. `REQUIREMENTS_TRACEABILITY.md` is the authoritative
per-requirement source of truth, and `docs/spec/README.md` holds the feature IDs
(F-1xx…F-7xx). Keep this table in step whenever the traceability file changes
status.

**Legend:** **Done** = implemented and CI-verified · **Partial** = partly
implemented · **Todo** = not started · **Out of scope** = deliberately excluded
(`docs/scope.md`).

| Capability (inspiration level) | Feature IDs | Status | Notes |
|---|---|---|---|
| Open & render PDF documents, page navigation | F-401–F-406, F-504 | Partial | PDF import + page rendering (Phase 5); grid + snap-to-grid (F-405) and rulers (F-406) done 2026-10-23; page thumbnails (F-404) pending |
| Text & review markup (highlight, strikeout, underline, squiggly, text, callout, note, stamp) | F-101–F-111 | Partial | tool set landed; note popover (F-103) pending |
| Shape & path markup (rectangle, ellipse, cloud, polygon, polyline, line, arrow, ink) | F-101–F-116 | Partial | shapes/paths + drag-to-draw landed; z-order / copy-properties (F-113–F-115) pending |
| Selection editing (select, move, delete, layers) | F-113–F-116 | Partial | select/move/delete + layer visibility/lock (F-116) done |
| Measurements with page scale (length, perimeter, area, angle, diameter, radius, count) | F-201–F-206 | Partial | measure tools + scale/units landed |
| Calibration & units (presets, custom ratio, unit selection, labels) | F-207–F-209 | Done | inspector scale editor + on-canvas labels |
| Review workflow (status, comments, author/subject, report) | F-301–F-305 | Done | statuses, comments, summary report, status legend |
| Persistence (project sidecar, autosave, recents, dialogs) | F-501–F-503, F-506, F-507 | Partial | `.rubric.json` sidecar + recents landed; autosave pending |
| Export & print (rendered page image, print) | F-505 | Done | image export + print |
| Undo/redo | F-604 | Done | bounded 128 snapshots |
| UI/UX polish (keyboard shortcuts, dark theme, context menus) | F-601–F-603 | Done (2026-10-23) | F-601 per-tool keys, F-602 dark theme, F-603 context menus |
| Accessibility (contrast, keyboard reachability) | F-605 | Todo | reviewed later |
| Packaging & installers (AppImage, DMG, MSI, CI artifacts) | F-701–F-704 | Todo | Phase 7 |
| Web (WASM) target | — | Out of scope (v1) | see `docs/scope.md` |

## Notes

- "Done" here means implemented **and** verified by the GitHub Actions Task
  Runner (fmt, clippy `-D warnings`, full tests, release build).
- Capability-level inspiration only: Rubric copies no proprietary UI, wording or
  algorithms; see `docs/inspiration.md` and the originality attestation in
  `docs/qc/checklist.md`.
