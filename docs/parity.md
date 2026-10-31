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
| Open & render PDF documents, page navigation | F-401–F-406, F-504 | Done | PDF import + page rendering via PDFium/hayro hybrid; thumbnails (F-404), grid/snap (F-405), rulers (F-406), zoom presets (F-401); pure-Rust hayro renderer available via --features hayro |
| Text & review markup (highlight, strikeout, underline, squiggly, text, callout, note, stamp) | F-101–F-111 | Done | tool set + note popover (F-103) 2026-10-30 |
| Shape & path markup (rectangle, ellipse, cloud, polygon, polyline, line, arrow, ink) | F-101–F-116 | Done | shapes/paths + drag-to-draw; page+layer-scoped z-order and copy properties (F-114/F-115) |
| Selection editing (select, move, delete, copy properties, z-order, layers) | F-113–F-116 | Done | select/move/delete, copy/paste visual style, page+layer-scoped z-order + layer visibility/lock/rename/reorder |
| Measurements with page scale (length, perimeter, area, angle, diameter, radius, count) | F-201–F-206 | Done | measure tools + scale/units + on-canvas labels |
| Calibration & units (presets, custom ratio, unit selection, labels) | F-207–F-209 | Done | inspector scale editor + on-canvas labels |
| Review workflow (status, comments, author/subject, report) | F-301–F-305 | Done | statuses, comments, summary report, status legend |
| Markups list / review console (filter, sort, group) | R-V02-01 | Done | `MarkupList` domain model plus resizable bottom panel with search/status/author/subject/layer filters, all sorts and group modes, row selection + page jump (2026-10-26) |
| Document compare / markup diff (added, removed, changed) | R-V02-02 | Partial (domain done) | `DiffResult` + `diff_documents`/`diff_projects` landed 2026-10-26; UI presentation pending |
| Reusable style presets | R-V02-02 | Partial (domain done) | `StylePreset`/`StylePresetLibrary` landed 2026-10-26; preset editing/apply affordance still pending |
| Persistence (project sidecar, autosave, recents, dialogs, schema version) | F-501–F-503, F-506, F-507 | Done | `.rubric.json` sidecar + recents + native dialogs landed; autosave loop and `schema_version` guard landed 2026-10-26 |
| Export & print (rendered page image, print) | F-505 | Done | image export + print |
| Undo/redo | F-604 | Done | bounded 128 snapshots |
| UI/UX polish (keyboard shortcuts, dark theme, context menus) | F-601–F-603 | Done (2026-10-23) | F-601 per-tool keys, F-602 dark theme, F-603 context menus |
| Accessibility (contrast, keyboard reachability) | F-605 | Done | WCAG contrast audit test + audited body-text colours + labelled controls (2026-10-30, `docs/a11y.md`) |
| Packaging & installers (AppImage, DMG, MSI, CI artifacts) | F-701–F-704 | Done (2026-10-23) | CI release build for 3 targets + Linux AppImage, macOS DMG, Windows MSI; installers unsigned (Q-PACK-1) |
| Web (WASM) target | — | Out of scope (v1) | see `docs/scope.md` |

## Notes

- "Done" here means implemented **and** verified by the GitHub Actions Task
  Runner (fmt, clippy `-D warnings`, full tests, release build).
- Capability-level inspiration only: Rubric copies no proprietary UI, wording or
  algorithms; see `docs/inspiration.md` and the originality attestation in
  `docs/qc/checklist.md`.
