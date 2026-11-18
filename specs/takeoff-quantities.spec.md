# Specification — Takeoff Quantities (v0.5)

Status: **Draft for owner approval** (research gate: `docs/research/takeoff-quantities.md`).
Scope decided by owner 2026-11-18: **Core four · CSV export · no pricing · bottom-panel tab**.
Inspiration: capability-level only, from `Kentucky-ai/opentakeoff` (AGPL-3.0 — **no code,
assets or text adopted**; recorded in `docs/resource-catalog.md`) and the construction-takeoff
category vocabulary. Everything here is original to Rubric.

---

## 1. Overview and user value

A takeoff is the act of measuring quantities off a drawing to estimate a job. Rubric already
measures lengths, areas, perimeters and counts against a calibrated page scale; what it lacks is
the **quantity schedule**: a named, grouped roll-up of every measurement that an estimator can
export and price elsewhere.

This feature adds:

1. **Conditions** — named measurement categories (e.g. *Carpet*, *Skirting*, *Downlights*) with
   an optional **wall height** for derived wall-surface quantities.
2. **Condition assignment** — any measure or count markup can be assigned to a condition from
   the Inspector or the context menu.
3. **Quantities panel** — a bottom-panel tab that aggregates all assigned measurements per
   condition, with live totals in each page's display unit.
4. **CSV export** — the schedule as a CSV file that opens in Excel/Sheets.

**User value:** an estimator imports a plan set, calibrates each sheet, measures once, assigns
each measurement to a named condition, and walks away with a clean CSV schedule — inside the
app they already mark up with, with no upload of their drawings anywhere (privacy posture
unchanged: nothing leaves the machine).

**Out of scope for v1** (tracked for later): symbol auto-count (v2 stretch), pricing/cost
columns, XLSX/PDF schedule export, cut-out/deduction areas, waste %/multiplier fields, formula
columns beyond wall-surface, revision-compare quantity deltas, MCP bridge to external takeoff
engines (dropped with Option A; may be revisited as a separate owner decision).

---

## 2. Domain model

All types live in `apps/rubric/src/domain/` (pure, serde, headless-testable). No egui imports.

### 2.1 `TakeoffCondition` (new, `domain/takeoff.rs`)

```rust
pub struct TakeoffCondition {
    pub id: Uuid,
    /// User-facing name, non-empty after trim, unique case-insensitively.
    pub name: String,
    /// What the condition aggregates.
    pub kind: ConditionKind,
    /// Wall height for WallSurface conditions, in the page display unit.
    /// Must be `Some(> 0.0)` for WallSurface, `None` otherwise.
    pub height: Option<f64>,
    /// Optional legend colour (domain `Color`), used for the panel swatch
    /// and the assigned-markup tint. Defaults per kind.
    pub colour: Color,
    pub created: chrono::DateTime<Utc>,
    pub modified: chrono::DateTime<Utc>,
}

pub enum ConditionKind {
    /// Sum of lengths of assigned Length/Diameter/Radius measures.
    Linear,
    /// Sum of areas of assigned Area measures.
    Area,
    /// Sum of perimeters of assigned Perimeter measures.
    Perimeter,
    /// Derived: sum of (assigned run length) × height.
    /// Run = the perimeter of a Perimeter measure, or the length of a
    /// Length measure — the estimator chooses which measures to assign.
    WallSurface,
    /// Number of assigned Count markers.
    Count,
}
```

- `ConditionKind` is `Copy`, serde snake_case, has `ALL`, and a `label()` and `unit_suffix()`
  (`m²`/`m`/`each` — always in the *display unit*, area suffixed `²`).
- Stored on `Project` as `takeoff: TakeoffSettings { conditions: Vec<TakeoffCondition> }`
  (project-level, not per-document: a plan set shares one schedule).
- Serde: new field on `ProjectSettings`/`Project` with `#[serde(default)]` — legacy
  `.rubric.json` files load with an empty condition list (roundtrip + legacy tests required).

### 2.2 Markup assignment

`Markup` gains:

```rust
/// The takeoff condition this measurement belongs to, if any.
#[serde(default)]
pub condition_id: Option<Uuid>,
```

- `None` = unassigned (legacy files, or the estimator hasn't scheduled it yet). Legacy
  markups load as unassigned; nothing breaks.
- Assignment is only valid on **measure and count markups** (`Length`, `Area`, `Perimeter`,
  `Diameter`, `Radius`, `Angle`→no, `Count`). The Inspector/context menu offers assignment
  only for those kinds. Assigning to a condition whose `kind` does not match the markup's
  measure family is **prevented in the UI** and rejected in the domain method (e.g. an Area
  measure cannot join a Linear condition) — with one exception: **WallSurface conditions
  accept Perimeter and Length measures** (the "run").
- Deleting a condition does **not** delete measurements: all `condition_id`s pointing at it
  become `None` (the measurements stay; they just leave the schedule). The deletion
  confirmation names this consequence (component-recipe "Danger" rule).

### 2.3 Quantity computation (pure function, `domain/takeoff.rs`)

```rust
pub struct QuantityRow {
    pub condition: TakeoffCondition,     // snapshot of the condition
    pub total: f64,                      // in the condition's unit semantics
    pub unit: MeasureUnit,               // display unit of the contributing pages
    pub item_count: usize,               // how many markups contributed
    pub mixed_units: bool,               // true if contributors use >1 display unit
}

pub fn quantity_rows(project: &Project) -> Vec<QuantityRow>
```

Rules:

- Iterates `takeoff.conditions`; for each, folds over **visible** markups assigned to it
  (hidden layers excluded — quantities match what prints; locked layers still count).
- `Linear`: Σ `measurement.value_points` converted to each contributing page's display unit.
  `Perimeter`: Σ perimeter values. `Area`: Σ area values (`value_points` semantics per
  existing measure kinds — reuse `MeasureKind` value plumbing; do not re-derive geometry).
- `WallSurface`: Σ (run length in display units × `condition.height`). Height is entered in
  the condition's unit and treated as uniform across the project in v1 (documented limit;
  multi-height is a v2 candidate).
- `Count`: number of assigned Count markers (unit "each", `total` = count).
- `mixed_units`: set when contributors span display units; the row shows the **project
  default unit** and the panel flags the row (never silently adds m to ft).
- Unassigned measurements appear in a fixed trailing pseudo-row **"Unassigned"** only when at
  least one exists (so the estimator can see what's left to schedule). It is display-only:
  never exported, never counted in condition totals.
- Precision/rounding is a **display** concern (existing `precision` setting); `total` is
  unrounded. CSV writes full precision.
- Complexity: O(markups). Recomputed on render of the panel; no caching in v1 (pages have
  ≤10⁴ markups; measure at 60 fps is not required for a panel redraw — but recompute must
  not allocate per frame beyond the returned Vec).

---

## 3. Functional requirements (EARS)

### Conditions (CRUD)

- **R-1.** When the user chooses **Takeoff ▸ New condition…** (Inspector Settings tab), the
  system shall create a condition with a unique default name (`Condition 1`, `2`, …), kind
  `Linear`, and the next palette colour, and push one undo snapshot.
- **R-2.** When the user edits a condition's name, the system shall accept it only after
  trim, reject empty and case-insensitively-duplicate names (with an inline message), and
  push one undo snapshot on commit (lost-focus/Enter, not per keystroke — P1-4 pattern).
- **R-3.** When the user sets a condition's kind to `WallSurface`, the system shall require a
  height > 0 in the current default unit before the condition is saved, showing the unit
  suffix on the field.
- **R-4.** When the user deletes a condition that has assigned markups, the system shall show
  a confirmation naming the consequence ("N measurements will become unassigned"), and on
  confirm unassign them and push one undo snapshot.
- **R-5.** The system shall cap conditions at 128 per project; when the cap is reached, the
  New-condition action shall be disabled with a tooltip stating the cap.
- **R-6.** The system shall keep condition CRUD out of the markup undo snapshots except where
  R-1/R-2/R-4 state one (condition edits snapshot only themselves; assigning a markup
  snapshots with the markup change).

### Assignment

- **R-7.** When the user right-clicks a measure/count markup, the context menu shall offer
  **Assign to condition ▸** listing conditions whose `kind` accepts that markup's measure
  family, plus **Unassign** when one is assigned.
- **R-8.** When the user assigns or unassigns via the Inspector (a combo in the Properties
  tab, shown only for assignable kinds), the system shall set `condition_id`, touch the
  markup, and push one undo snapshot per commit.
- **R-9.** When a selection contains multiple assignable markups, the context menu shall
  assign/unassign the whole selection in one snapshot (batch pattern, matching Batch
  Status).
- **R-10.** When a markup is duplicated, its duplicate shall carry the same `condition_id`
  (it is a copy of the measurement); when a markup is deleted, the quantity list simply
  stops counting it (no condition change).
- **R-11.** When a condition is renamed or recoloured, the panel and any assigned markups'
  legend tints shall update without a markup snapshot (view derives from the condition).

### Quantities panel

- **R-12.** Where the bottom panel is open, the system shall provide a **Quantities** tab
  beside **Markups List**, remembered in `AppPrefs` (`show_quantities` +
  `quantities_tab_selected`, both `#[serde(default)]`).
- **R-13.** When the panel opens with zero conditions, the empty state shall read:
  "No takeoff conditions yet. Conditions name what you measure — create one to start the
  schedule." with a **New condition** button (II.30.11 empty-state rule: what/why/next).
- **R-14.** When conditions exist, the panel shall render one row per condition: colour
  swatch, name, kind label, item count, and the total with unit suffix, using tabular
  number alignment.
- **R-15.** When a row's contributors span more than one display unit, the system shall
  show the total with a "mixed units" badge and a tooltip listing the units involved; the
  total shall be labelled with the project default unit.
- **R-16.** When at least one measurable markup is unassigned, the system shall show the
  trailing **Unassigned** row with its aggregate and a hint to assign from the context menu.
- **R-17.** When the user clicks a row, the system shall select all markups assigned to that
  condition and jump to the first contributor's page (same selection contract as the
  Markups List rows).
- **R-18.** When measurements, assignments or scales change, the panel totals shall update
  on the next frame without a manual refresh.

### CSV export

- **R-19.** When the user chooses **File ▸ Export ▸ Quantities (CSV)…**, the system shall
  open a save dialog (default `<project stem>-quantities.csv`), and on confirm write the
  schedule on a worker thread, never blocking the UI loop.
- **R-20.** The CSV shall contain one row per condition (excluding Unassigned) with columns:
  `Condition,Kind,Unit,Quantity,Items,Height,Notes` — `Quantity` full precision with `.`,
  decimal separator; `Notes` = the condition's notes field, quoted per RFC 4180 (commas,
  quotes, newlines escaped).
- **R-21.** When the export write fails (permission, disk full, path in use), the system
  shall show a status-bar error naming the path and keep the dialog's chosen path in the
  recent list; no partial file is left behind (write to temp, rename on success — same
  atomic pattern as `save_project`).
- **R-22.** When the project has no conditions, the export action shall be disabled with a
  disabled-hover reason ("No takeoff conditions to export").
- **R-23.** When an export completes, the status bar shall confirm
  "Quantities exported to <file name>" (file name, not full path — personal-data rule).

### Scale & unit coupling

- **R-24.** When a page's scale is recalibrated, the system shall recompute affected
  quantities through the existing measurement rescaling path — no separate takeoff
  rescale logic (single source of truth: `value_points`).
- **R-25.** When the display unit changes for a page or the app default, the panel and CSV
  shall reflect the new unit on the next frame/export (conversion, not re-measurement).

### Persistence & compatibility

- **R-26.** The system shall serialise conditions and `condition_id`s in the project JSON;
  files saved by this version load identically (roundtrip test).
- **R-27.** When a legacy project (no takeoff data) loads, the system shall present an empty
  condition list and all markups unassigned, with no migration prompt.
- **R-28.** When a project file from a *newer* schema loads, existing behaviour (refusal)
  is unchanged.
- **R-29.** When autosave runs, takeoff state persists with the rest of the project (no
  separate store).

### Accessibility (II.21/II.30 — binding)

- **R-30.** Every panel control shall be keyboard-reachable in reading order; Esc closes
  any open condition dialog; Enter commits the focused name/height field; focus is visible
  and never trapped.
- **R-31.** Icon-only controls (row swatch, export) shall carry AccessKit labels
  (`labelled_by`), not just tooltips.
- **R-32.** Interactive targets shall be ≥ 24 logical points; the panel shall not clip at
  200% zoom / narrow width (table scrolls, rows never overlap).

---

## 4. Non-functional requirements

| ID | Requirement |
|----|-------------|
| NFR-1 | Quantity recompute for 10 000 markups ≤ 1 ms (pure fold, no per-frame allocation beyond the row Vec; measured by a `#[test]` with a wall-clock budget or op-count assertion). |
| NFR-2 | CSV export of 1 000 rows ≤ 100 ms on a worker thread; UI never blocks (pattern: `begin_update_download`). |
| NFR-3 | No network access, no telemetry — takeoff data never leaves the machine (privacy posture unchanged). |
| NFR-4 | All new types serde-roundtrip; legacy file load is a tested invariant, not a hope. |
| NFR-5 | New UI passes the a11y smoke checklist (keyboard-only walkthrough, Esc, contrast in both themes + high contrast). |
| NFR-6 | Domain code has zero egui imports; panel code zero business logic (K.3 layering — reviewer-enforced). |
| NFR-7 | Colours for new UI come from theme tokens; the raw-colour grep gate stays green. |

---

## 5. Acceptance criteria (Given / When / Then)

### Conditions

- **AC-1.** Given a project with no conditions, when the user creates one named "Carpet"
  (kind Area), then the panel shows a "Carpet" row with 0 items and total `0.00 m²`, and
  Ctrl+Z removes it.
- **AC-2.** Given a condition named "carpet" exists, when the user tries to create "Carpet",
  then the name field shows the duplicate message and no condition is created.
- **AC-3.** Given a WallSurface condition without a height, when the user blurs the height
  field empty or ≤ 0, then the condition is not saved and the field shows the error state.
- **AC-4.** Given a condition with 5 assigned markups, when the user deletes it and
  confirms, then the 5 markups remain (unassigned) and one Ctrl+Z restores both the
  condition and the assignments.

### Assignment

- **AC-5.** Given an Area measure on a locked layer, when the user right-clicks it, then
  Assign is offered and works (locked affects *editing*, not scheduling).
- **AC-6.** Given a Linear condition and an Area measure, when the context menu opens, then
  that condition is absent from the Assign submenu.
- **AC-7.** Given 3 selected Length measures, when the user assigns them to "Skirting" via
  the context menu, then one Ctrl+Z unassigns all three.

### Aggregation

- **AC-8.** Given two Length measures (2.0 m on a 1:50 page, 3.4 m on a 1:100 page, both
  displaying metres) assigned to "Skirting", then the panel shows 5.400 m with 2 items.
- **AC-9.** Given a WallSurface condition height 2.7 m with an assigned Perimeter of
  12.0 m, then the row total is 32.400 m² and the kind label reads "Wall surface".
- **AC-10.** Given contributors in metres and feet on one condition, then the row shows the
  mixed-units badge and the tooltip lists both units.
- **AC-11.** Given a Count condition with 4 assigned count markers, then the row shows
  "4 each".
- **AC-12.** Given a markup on a hidden layer assigned to "Carpet", then its area is
  excluded from the row; when the layer is shown, it is included without any re-assignment.

### CSV

- **AC-13.** Given the AC-8 state, when the user exports, then the CSV contains the header
  `Condition,Kind,Unit,Quantity,Items,Height,Notes` and the row
  `Skirting,Linear,m,5.4,2,,,` (no trailing spaces; CRLF or LF consistent with platform).
- **AC-14.** Given a condition named `Carpet, red` (comma in name), then the exported CSV
  quotes it: `"Carpet, red"`.
- **AC-15.** Given the export target is a read-only directory, when the user confirms the
  save, then an error names the path, no `.csv` or `.tmp` file remains, and the app is
  usable.

### Compatibility

- **AC-16.** Given a v0.4 project file, when loaded, then it renders identically to today
  and the Quantities panel shows the empty state (R-13).
- **AC-17.** Given a takeoff project saved and reloaded, then conditions, names, kinds,
  heights, colours and all `condition_id` assignments are identical (roundtrip test).

---

## 6. Error handling table

| Situation | Detection | Response | User sees |
|---|---|---|---|
| Duplicate condition name | Case-insensitive compare on commit | Reject save, keep dialog open | Inline "A condition with this name already exists" |
| WallSurface height ≤ 0 / empty | Field validation | Reject save, error state on field | "Enter a height greater than 0" |
| Condition cap (128) reached | Count before create | Disable action | Disabled button + tooltip "Condition limit (128) reached" |
| Condition deleted with assignments | Count before delete | Confirm dialog → unassign | "N measurements will become unassigned" |
| CSV write fails | `Result` from worker | Temp-file discarded, no rename | Status error with file name; dialog path retained |
| Export with 0 conditions | Gate before dialog | Action disabled | Hover reason "No takeoff conditions to export" |
| Mixed display units in one condition | Unit compare during fold | Row flagged, total in project default unit | "mixed units" badge + tooltip |
| Orphaned `condition_id` (hand-edited file) | Lookup miss during fold | Treat as unassigned (fail-open, matching layer semantics) | Markup appears in Unassigned |
| Condition edit on read-only store / IO error | `Result` | Status-bar error, no snapshot pushed | "Could not save: <reason>" |

---

## 7. UI specification (Quantities panel)

- **Location:** bottom panel, tab strip `Markups List | Quantities` (Inspector-style
  segmented control; selection persisted in `AppPrefs`).
- **Layout:** header row (heading + item count + `New condition` + `Export CSV` +
  `Clear filters` none — v1 has no filters), then the table:
  `■ | Name | Kind | Items | Quantity`, right-aligned tabular numbers.
- **Row interaction:** click = select contributors + jump to first page (R-17);
  double-click / ⏎ = edit condition dialog; context menu = Rename, Change kind, Set height
  (WallSurface), Recolour, Delete.
- **Condition dialog:** name, kind combo (locked after creation if assignments exist —
  changing kind with assignments would orphan them; the dialog states this and disables the
  combo, with "Unassign all first" hint), height (WallSurface only, unit suffix), colour
  swatch (palette), notes. Esc = cancel, ⏎ = commit.
- **States:** empty (R-13); normal; error (CSV failure surfaces in status bar, panel
  unchanged); narrow window (table scrolls horizontally, name column truncates with `…`
  and full name in tooltip); high contrast + reduced motion honoured via tokens.
- **Keyboard:** Tab order header → table → buttons; ⏎ opens edit; Del on a selected row
  opens the delete confirm; Esc closes dialog; focus ring from `focus.ring`.

---

## 8. Persistence & compatibility summary

- `Project.takeoff: TakeoffSettings` (serde default), `Markup.condition_id` (serde default).
- Schema version **unchanged** in v1: new fields are additive with defaults, so
  `load_project`'s version gate is unaffected (R-26/27 verified by tests).
- `AppPrefs`: `show_quantities: bool`, `quantities_tab_selected: bool` (serde defaults).
- Autosave: nothing new — takeoff rides the existing project snapshot path (R-29).

---

## 9. Implementation TODO checklist

Ordered; each item is one PR-sized slice with its tests.

- [ ] **T1 — Domain types.** `domain/takeoff.rs`: `TakeoffCondition`, `ConditionKind`
      (serde, ALL, labels, unit suffixes), `TakeoffSettings`; `Project.takeoff` field;
      legacy-load + roundtrip tests. *Tests: `takeoff_settings_serde_roundtrip`,
      `legacy_project_loads_empty_takeoff`.*
- [ ] **T2 — Markup assignment.** `Markup.condition_id` (serde default); domain methods
      `assign_condition(&mut self, ids, condition_id) -> usize` (kind-match validation,
      WallSurface accepts Perimeter+Length, touch + snapshot policy documented);
      duplicate-carries-assignment; orphan fail-open. *Tests: kind-reject, batch assign,
      duplicate carry, orphan lookup.*
- [ ] **T3 — Quantity computation.** `quantity_rows(project)` pure fold (visible-only,
      mixed-unit flag, Unassigned row data, WallSurface derivation, NFR-1 budget test).
      *Tests: AC-8, AC-9, AC-11, AC-12 as headless domain tests.*
- [ ] **T4 — Condition CRUD.** Project methods (create with default name/colour, rename
      uniqueness, kind/height validation, delete-with-unassign); 128 cap. *Tests: AC-1–AC-4
      domain halves, cap test.*
- [ ] **T5 — CSV export.** `data/export.rs`: RFC 4180 writer, temp+rename atomic, worker
      off-thread, disabled-with-reason gate. *Tests: header/content (AC-13), quoting
      (AC-14), failure leaves no file (AC-15).*
- [ ] **T6 — Panel UI.** Bottom-panel tab strip + Quantities table (tokens, states, row
      click-select, empty state, mixed-unit badge, narrow-window behaviour, AccessKit
      labels, keyboard path). *Tests: filter-from-state style unit tests + kittest
      AccessKit assertions where the seam allows; snapshot baseline update (Settings/
      bottom panel change).* 
- [ ] **T7 — Assignment UI.** Inspector combo (assignable kinds only) + context menu
      submenu with batch assign/unassign; undo snapshot wiring. *Tests: action-enabled
      predicates (domain/action.rs pattern).*
- [ ] **T8 — Condition dialog.** Create/edit dialog with validation states (name, kind
      lock, height), Esc/⏎, delete confirm naming consequence. *Tests: validation
      predicates; kittest Esc assertion.*
- [ ] **T9 — Prefs wiring.** `show_quantities` + tab persistence; AppPrefs roundtrip
      extension. *Tests: prefs roundtrip.*
- [ ] **T10 — Docs & release.** `docs/research/takeoff-quantities.md` approval note
      resolved, capability-inventory rows → implemented with CI evidence, CHANGELOG
      `[0.5.0]` entry, README feature paragraph, ROADMAP checkboxes ticked with CI run IDs.

Each slice: branch → `ci.sh run` → green → PR (template) → merge; no slice bumps egui or
adds dependencies (CSV is hand-rolled RFC 4180 — no new crates).

---

## 10. Open questions (owner, non-blocking for T1–T5)

1. **Condition colours** — fixed palette (like ReviewStatus) vs free colour picker?
   Spec assumes fixed palette for v1 consistency.
2. **Angle measures** — excluded from assignment in v1 (an angle is not a quantity);
   confirm.
3. **WallSurface height unit** — assumed to follow the project default display unit;
   alternative is a per-condition fixed unit. Spec assumes default-unit.
4. **Unassigned row export** — excluded from CSV in v1; confirm.

---

*Research basis: `docs/research/takeoff-quantities.md` (human-gated). This spec is the
implementation contract for v0.5; deviations require a `DECISIONS.md` entry.*
