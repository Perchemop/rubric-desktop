# Capability Inventory (guide II.27.3)

**What this is.** A single, reviewable inventory of every capability Rubric
delivers: the *capability* (not any vendor's implementation), the public
source that showed the capability is useful, the acceptance criteria that
define "done", and the **evidence** that it is done. It is the design/parity
view over `docs/spec/README.md` (the 52 feature IDs),
`REQUIREMENTS_TRACEABILITY.md` (requirement → feature → module) and
`docs/parity.md` (inspiration-level parity).

**Originality.** Rubric copies no proprietary UI, wording, icons, algorithms or
layouts. Inspiration is capability-level only; see `docs/inspiration.md`,
`docs/research/*` (summaries + URLs) and the originality attestation in
`docs/qc/checklist.md`.

**Evidence conventions.** "Code" is the module that owns the logic; "Test" is a
named `cargo test` case; "CI" is a GitHub Actions run ID. Status is **Done**
only when implemented *and* CI-verified.

---

## Capabilities

| # | Capability | Source (why it is useful) | Feature IDs | Acceptance criteria | Evidence |
|---|---|---|---|---|---|
| 1 | Open a PDF read-only and render pages; navigate by page | Adobe annotate page <https://www.adobe.com/acrobat/how-to/annotate-pdf-online.html>; `docs/research/adobe-acrobat-capabilities.md` | F-503, F-504, F-403 | Import a PDF without modifying it; render the active page; move between pages via arrows, Page Up/Down and the thumbnail sidebar | Code `data/pdf.rs`, `ui/canvas.rs`, `ui/pages.rs`; test `tests/domain_tests.rs` PDF round-trip; CI `357...`/v0.1.0 release run |
| 2 | Non-destructive markup stored in a sidecar | General commercial PDF suites; `docs/inspiration.md` | F-501, F-507 | Markups never touch the source PDF; save/load `.rubric.json` with UUIDs + UTC timestamps; refuse a newer `schema_version` | Code `data/store.rs`, `domain/project.rs`; test `load_project_refuses_a_newer_schema_version`, `roundtrip`; CI v0.2-04 |
| 3 | Highlight / strikeout / underline / squiggly text markup | Adobe annotate page | F-101, F-102 | Place each text-markup kind; hit-test stroke/interior per kind | Code `domain/markup.rs`; test `hit testing v2` suite; CI v0.1.0 |
| 4 | Sticky note (anchored comment pin) with a floating popover | Adobe sticky notes; Bluebeam markups page | F-103 | Place a note pin; open a floating popover showing the threaded comments; add a comment (blank refused) | Code `domain/markup.rs` (`add_comment`), `ui/canvas.rs`; test `add_comment_appends_and_refuses_blank`; CI `40942594880` |
| 5 | Text box and callout editing on the canvas | Adobe callouts | F-104, F-105 | Draw text/callout; edit in place; commit on focus loss, Esc cancels | Code `ui/canvas.rs` (`text_edit`); test `ui/canvas.rs` commit path; CI v0.1.0 |
| 6 | Shape markup (rectangle, ellipse, cloud) | Adobe annotate page | F-106–F-108 | Draw bounds shapes; filled vs stroke-only hit-testing | Code `domain/markup.rs`, `ui/canvas.rs`; tests `filled shapes hit interior`, `ellipse edge`, `polygon even-odd`; CI v0.1.0 |
| 7 | Path markup (polyline, polygon) and freehand ink | Bluebeam markups page | F-108–F-110 | Click-to-append vertices with rubber-band preview; commit on double-click/right-click/Enter; ink stroke merging | Code `ui/canvas.rs` (`DrawState`); tests `path_points_ok`, polygon area; CI v0.1.0 |
| 8 | Line / arrow markup | Adobe annotate page | F-108 | Drag to draw; commit a valid segment only | Code `ui/canvas.rs`; test segment-commit guards; CI `40317456119` |
| 9 | Stamp library (approved/rejected/draft/check/cross) | Bluebeam stamps; `docs/research/bluebeam-studio-capabilities.md` | F-111, F-304 | Pick a preset; stamp glyph + meaning-coloured stroke; include in summary report | Code `domain/markup.rs` (`StampPreset`), `ui/canvas.rs`; test `markup_stamp_serde_roundtrip_preserves_preset`; CI v0.1.0 |
| 10 | Drag-to-draw with live preview for every geometry | General PDF suites | F-112 | Bounds/segment/path/angle/ink all preview while drawing and commit on release/commit gesture | Code `ui/canvas.rs`; test draft-commit guards; CI v0.1.0 |
| 11 | Selection editing: select, move, delete | Bluebeam markups page | F-113 | Pick frontmost markup; move a selection with one snapshot per drag; delete selection | Code `ui/canvas.rs`, `ui/app.rs`; test `z_order_stays_within_the_page_and_stops_at_edges`; CI `40317456119` |
| 12 | Copy/paste properties (visual style) | Bluebeam markups page | F-114 | Copy a markup's visual style; apply to selected editable markups; locked targets skipped; one snapshot | Code `domain/markup.rs` (`apply_style`), `ui/app.rs`; test `apply_style_to_skips_locked_and_missing_markups`; CI `40317456119` |
| 13 | Z-order (forward, backward, to front, to back) | Bluebeam markups page | F-115 | Reorder within the markup's page **and layer**; boundary no-ops | Code `domain/document.rs`; test `z_order_is_scoped_to_layer_and_page`; CI `40849531047` |
| 14 | Layers: visibility, lock, rename, reorder (real stacking) | Bluebeam layers | F-004, F-116 | Hide/lock a layer gates paint + editing; rename; reorder changes paint order; default layer pinned | Code `domain/document.rs`, `ui/inspector.rs`; test `markups_in_paint_order_respects_layer_order`, `move_layer_up_and_down_respects_default_pin_and_updates_z`; CI `40849531047` |
| 15 | Measurements: length, perimeter, area, angle, diameter, radius, count | Bluebeam measurement; `docs/research/bluebeam-studio-capabilities.md` | F-201–F-206 | Place each measure kind; compute from page scale; recompute on scale/unit change | Code `domain/markup.rs`, `domain/units.rs`; tests `length_measurement_uses_scale`, `area_measurement_is_squared`; CI v0.1.0 |
| 16 | Calibration & units (presets, custom ratio, unit selection) | Bluebeam measurement | F-207, F-208 | Set 1:1 or a custom paper:real ratio; pick a display unit; preserve physical measurement | Code `domain/units.rs`, `ui/inspector.rs`; tests `Scale::with_display_unit`; CI v0.1.0 |
| 17 | Measurement labels on canvas + precision preference | Bluebeam measurement | F-209 | Geometry-anchored labels; 0–6 dp precision, persisted | Code `domain/units.rs` (`format_value_precise`), `ui/canvas.rs`; tests precision matrix; CI v0.1.0 |
| 18 | Review workflow: statuses, comments, author/subject | Bluebeam status workflow; `docs/research/bluebeam-studio-capabilities.md` | F-301–F-303 | Open/In review/Approved/Rejected with colours; threaded comments; author/subject fields | Code `domain/markup.rs` (`ReviewStatus`, `Comment`); tests status remap; CI v0.1.0 |
| 19 | Summary report export | Bluebeam markups-and-data | F-304 | Group markups by status/author/subject; export text/JSON | Code `domain/report.rs`, `data/export.rs`; test report grouping; CI v0.1.0 |
| 20 | Status colours and legend | Bluebeam status workflow | F-305 | Distinct colour per status; live legend counts; colour never the only signal | Code `ui/inspector.rs`, `domain/markup.rs`; tests `ReviewStatus::color`; CI v0.1.0 |
| 21 | Markups list / review console (filter, sort, group) | Bluebeam markups list | R-V02-01, F-116 | Resizable panel; search + status/author/subject/layer filters; all sorts/groups; row → page jump | Code `domain/markup_list.rs`, `ui/markups_list.rs`; test `markup_list_filter_and_sort_from_document`; PR #34 |
| 22 | Document compare / markup diff | Adobe compare files <https://helpx.adobe.com/acrobat/using/compare-documents.html> | R-V02-02 | Diff by stable id → added/removed/changed with geometry/style-aware change detection | Code `domain/diff.rs`; test `diff_documents_reports_add_remove_change`; CI `40304244261` |
| 23 | Reusable style presets | Bluebeam tool sets | R-V02-02 | Capture/apply a style; 64-entry library; upsert dedupe | Code `domain/style_preset.rs`; test `style_preset_apply_and_roundtrip`; CI `40304244261` |
| 24 | Persistence: recents, native dialogs, autosave | General suites | F-501–F-503, F-506, F-507 | Native Open/Save; recents self-prune; autosave only while dirty, ≤ once per interval, beside the project | Code `data/store.rs`, `data/recent.rs`, `ui/app.rs`; test `autosave_due_respects_the_interval_and_clamps_tiny_values`; CI v0.2-04 |
| 25 | Export rendered page image + print | Bluebeam markups-and-data | F-505 | Render the current page to PNG; send to the OS print command with honest status | Code `data/render.rs`, `data/print.rs`; tests render/print; CI v0.1.0 |
| 26 | Undo / redo (bounded) | General suites | F-604 | Project snapshots, capped at 128; Ctrl+Z / Ctrl+Y | Code `domain/project.rs` (`History`); tests history bounds; CI v0.1.0 |
| 27 | Keyboard shortcuts, dark theme, context menus | General suites | F-601–F-603 | Per-tool keys; persisted light/dark theme; shared context menu through one executor | Code `domain/project.rs`, `domain/theme.rs`, `ui/menu.rs`; tests shortcut table, theme serde; CI v0.1.0 |
| 28 | Accessibility: readable contrast, keyboard reachability, labels | General suites | F-605 | Body text meets WCAG AA; accent fill meets AA-large; icon-only controls labelled | Code `domain/a11y.rs`, `ui/theme.rs`; test `shipped_palettes_meet_wcag_contrast`; CI `40963721115`; `docs/a11y.md` |
| 29 | Packaging & installers (AppImage, DMG, MSI, CI artifacts) | General desktop software | F-701–F-704 | Release build for Linux/macOS/Windows; native installers; checksums | Code `.github/workflows/release.yml`, `package.yml`, `packaging/wix/`; CI release run `40049997043` |
| 30 | Grid, snap-to-grid, rulers | Bluebeam measurement aids | F-405, F-406 | Optional grid with snapping; labelled ruler ticks that follow pan/zoom | Code `domain/grid.rs`, `ui/canvas.rs`; test `ruler_ticks` ladder; CI v0.1.0 |
| 31 | Page thumbnails sidebar | Adobe page organisation | F-404 | One cell per page; PDF-backed preview reuses the page cache; click to jump | Code `domain/thumbs.rs`, `ui/pages.rs`; test `thumb_layout`; CI v0.1.0 |

## Deliberately out of scope

| Capability | Reason | Record |
|---|---|---|
| Web (WASM) target | Not required for the desktop product in v1 | `docs/scope.md` |
| PDF redaction, signatures, page organisation | Research-only; no clean-room note approved for implementation | `docs/research/stirling-pdf-capabilities.md` |
| Real-time collaboration / cloud | Contradicts the local-first, no-account design | `docs/scope.md`, `docs/inspiration.md` |

## Open process gaps (not product capabilities)

- **G3** — this file (closed by this slice).
- **G4** — automated `egui_kittest` visual evidence needs a `Host`/plain-`Ui`
  seam in `RubricApp`; deferred to v0.3 (`docs/guide-compliance.md`). No
  fabricated harness is used in the meantime.
