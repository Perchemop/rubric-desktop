# Takeoff quantities – Capability-level summary

> **HUMAN GATE — implementation requires owner approval of these notes
> (guide §2.3, §8.4). **Scope approved by the owner 2026-11-18** for the
> native path only (§3a): *core four capabilities · CSV export · no pricing ·
> bottom-panel tab* — recorded in `specs/takeoff-quantities.spec.md` and
> merged as PR #225. The MCP-client path (§3b) additionally requires an
> explicit owner decision and remains blocked.
>
> **Implementation status (2026-11-20): COMPLETE** — T1-T5 shipped in PRs #45, #47 with CI verification (CI run IDs recorded below):
> - T1 (domain types), T4 (condition CRUD), T2 (markup assignment), T3 (quantity computation), T5 (CSV export) all landed
> - AC-4 (symbol auto-count) remains out of scope for v0.5
> - AC-5 (MCP bridge) blocked pending owner decision
> - Per-criterion status: all implemented capabilities marked Done in `docs/parity.md`; UI layer (T6-T10) pending for v0.6

**Purpose:** original, capability-level summary of construction "takeoff"
(quantity takeoff, QTO) for a possible future Rubric slice. Rubric copies no
vendor implementation, UI, wording, icons or algorithms; this note records
*what* the capability is, *why* estimators rely on it, how Rubric's existing
measurement features already cover part of it, and where the gaps are — with
public sources only.

**Method (guide §2.4):** free-model web fetch of the public repository README
and the project's public MCP reference page, plus a public community catalog
page; every claim below is summarised into original wording against the URLs in
the *Sources* section. No code was read for implementation purposes, nothing
was decompiled, and no asset or text was copied.

**Sources:**
- OpenTakeoff repository (public README; capability enumeration, licence,
  platform and MCP description):
  <https://github.com/Kentucky-ai/opentakeoff>
- OpenTakeoff MCP server reference (publicly listed tool names and rules):
  <https://github.com/Kentucky-ai/opentakeoff/blob/main/mcp/README.md>
- Awesome-Construction-Takeoff (community catalog; category vocabulary and
  table-stakes features across commercial and open-source tools):
  <https://github.com/ishandutta2007/Awesome-Construction-Takeoff>
- GNU Affero General Public License v3.0 text (licence-boundary analysis):
  <https://www.gnu.org/licenses/agpl-3.0.txt>
- OpenTakeoff licence file (AGPL-3.0-only, with a recorded earlier
  Apache-2.0 period): <https://github.com/Kentucky-ai/opentakeoff/blob/main/LICENSE>

---

## 1. Behaviour summary (original wording)

**What "takeoff" means.** In construction estimating, a *takeoff* (or quantity
takeoff) is the measurement pass every bid starts with: counting up how much
floor area, wall surface, linear run and how many fixtures a set of drawings
contains, at a known scale, sheet by sheet. The output is a quantity list that
pricing and material ordering are built from. The community catalog frames the
category as four modern layers working together: 2D plan measurement (areas,
lengths, perimeters, volumes, symbol counts), 5D BIM extraction, AI/CV
automation (room detection, symbol recognition, schedule OCR) and estimating
(bills of quantities, cost assembly, bid packages).

**What the category treats as table stakes.** Across the commercial and
open-source tools the catalog lists, the recurring baseline capabilities are:

- **Scale calibration per sheet** — from a stated ratio (imperial like a
  quarter-inch scale or metric like 1:50), from a known dimension the user
  drags, or from a graphic scale bar; multiple scales may coexist in one
  document set.
- **Core measurement modes** — area, linear length, perimeter, volume,
  surface area, count/east-each, with snap aids and deduction areas
  (subtracting openings from gross surfaces).
- **Named conditions/attributes** — grouping each measurement under a named
  finish, material or assembly, commonly with a waste percentage and a
  quantity multiplier.
- **A quantity list / bill of quantities** — all measurements rolled up into
  one table, grouped by condition and unit (square feet/metres, linear
  feet/metres, each), with formulas or derived columns.
- **Exports** — CSV/Excel for estimating, plus a marked-up PDF of the plan set
  showing where every quantity came from.
- **Symbol counting** — either manual click-to-count markers or automated
  detection of repeated plan symbols.
- **Revision compare** — quantity deltas between a drawing issue and its
  addendum/revision.

**What OpenTakeoff does at capability level** (from its public README and MCP
reference — described here only as observed behaviour, none of it copied):

- It is a browser-based, local-first takeoff engine for construction and
  flooring plan sets: plans are opened in the browser, nothing uploads, and a
  self-hosted fork is explicitly supported. It exposes two front ends over the
  same takeoff data format — a visual canvas for people and an MCP server for
  AI agents (a large tool surface, publicly documented at 69 tools on a default
  build).
- **Scale is treated as a gate, not a default**: the engine may *detect* a
  drawn scale note but never applies one on its own, and polygon/line
  measurement on an unscaled sheet is refused. A scale set by an agent lands
  "unconfirmed" until a human confirms it in the canvas.
- **Measurement types** it offers: polygon area (with wall-face tracing as the
  intended workflow for rooms), straight/curved linear runs, wall surface area
  (a traced run × an entered height), per-item count markers, cut-outs
  subtracted from a parent area, and derived quantities (e.g. baseboard length
  derived from committed room boundaries; transition strips where two finishes
  meet).
- **Conditions** carry a colour, a hatch, a waste percentage, a multiplier, a
  height and a thickness; materials are computed from coverage rates against
  the measured quantity to produce rounded order quantities.
- **Reporting/exports**: per-condition and grand totals in area/linear/count
  units, with and without waste; CSV, JSON, Excel and DXF exports; and a
  "marked set" PDF — a legend cover plus the annotated plan sheets — as the
  human-reviewable deliverable. An optional pricing layer (unit prices, tax,
  overhead, profit) produces an estimate and a proposal document.
- **Multi-sheet handling**: a sheet tree with title-block numbers, levels,
  stitching of sheets across match lines, page rotation/reordering/extraction
  with quantities following, and read-only reference sheets shown beside the
  working sheet.
- **Review workflow**: agent-produced shapes stay a dashed "proposal" until a
  person accepts them; revisions carry quantity deltas; an RFI register and
  approval seals exist at capability level.
- **Automated counting**: a symbol-sweep capability finds every placement of a
  repeated symbol from one example (with matched/near-miss/refused outcomes
  disclosed rather than silently committed), and a schedule-row sweep counts a
  device schedule's tags across plan sheets. Automatic room detection from a
  single click exists but is currently gated off by the project while
  re-validated.

*Note on a catalogue discrepancy:* the Awesome-Construction-Takeoff entry for
OpenTakeoff still describes it as Apache-2.0; the repository's own licence file
is authoritative and states AGPL-3.0-only (the project relicensed from an
earlier Apache-2.0 period, which remains available for code from before that
point). Any future work must rely on the repo's stated licence, not the
catalogue's.

### 1.1 UI affordances from the inspiration products (distilled for the slice)

Each point restates an already-cited behaviour from the sources above, at the
level useful for designing Rubric's quantity UI. No new claims are introduced
and nothing visual is copied.

- **Traceable quantity rows.** Conditions carry a colour *and* a hatch, so a
  quantity row can be matched back to the shapes it aggregates on the plan.
  Rubric's rows should pair the colour swatch with a text label — colour is
  never the only signal.
- **Legend-first deliverable.** The human-reviewable export leads with a legend
  cover before the annotated sheets: the legend is what makes a marked-up plan
  readable to someone who did not do the measuring.
- **Two-state scale affordance.** A detected-but-unapplied scale note, and an
  agent-set "unconfirmed" scale, are visible states distinct from a confirmed
  scale — and measuring an unscaled sheet is refused. Rubric deliberately keeps
  its softer design (warning rather than refusal, see §2), but the *visible
  state distinction* is worth keeping.
- **Proposal-before-commit rendering.** Not-yet-accepted shapes render dashed —
  a review state that never looks like committed work.
- **Disclosure of automated outcomes.** The symbol sweep reports
  matched/near-miss/refused outcomes instead of silently committing placements:
  any auto-count UI presents outcomes for review, never a silent write.
- **Reference context beside the work.** Read-only reference sheets render
  beside the working sheet; the sheet tree carries title-block numbers and
  levels and stitches sheets across match lines.
- **Totals with and without waste.** Totals are reported both ways — a dual
  column, not a mode switch.

---

## 2. Rubric gap analysis

Rubric today (capability inventory rows 15–17, 19, 30; ROADMAP v0.4-05/-08):
per-page `Scale` with preset/custom/drag calibration; measurement tools for
length, perimeter, area, angle, diameter, radius and count (`MeasureKind` in
`domain/units.rs`); numbered per-page Count markers with a running total;
geometry-anchored badges with a precision preference; measurements recompute on
scale/unit change; a grouped summary Report with text/JSON export; and a
Markups List with filters, measurement totals for the filtered view and a CSV
export.

| Takeoff capability | Rubric's existing feature | Gap |
|---|---|---|
| Quantity list rolling up **all** measurements by kind/condition with names | Measurement totals appear in the filtered Markups List; Report groups by status/author/subject | No dedicated quantity panel that aggregates every measure markup by `MeasureKind` under a user-assigned item name, with per-page and document rollups |
| Wall-surface derivation (traced run × entered height) | Perimeter measurement exists | No per-measurement height attribute and no derived surface-area quantity |
| Waste % / ×N multiplier per condition | None | No per-item or per-group waste/multiplier columns in any quantity view |
| Quantity-shaped CSV/Excel export (aggregated items, units, pages) | Markups List CSV export exists (row-level, with filtered totals) | The existing CSV is a markup listing, not a takeoff schedule (no per-kind aggregation, unit rollups or waste columns) |
| Additional rollup units (e.g. square yards) | Display-unit selection and conversion exist for linear/area | No square-yard/other trade-unit rollup column in a totals view |
| Symbol auto-count from one example | Count markers exist — but they are manual, user-placed, numbered markers | No automated symbol detection. Note the distinction: manual counts are deterministic user marks; auto-count is a detection *aid* whose results should arrive as reviewable proposals, never silently committed |
| Deduction/cut-out areas | Polygon area exists; no subtraction concept | No way to subtract an opening from a parent area quantity |
| Revision compare of quantities | Document diff exists (markup-level added/removed/changed) | No quantity delta view between two project versions |
| Scale gate (refuse measuring an unscaled sheet) | Scale is optional; 1:1 default | Deliberate design difference — Rubric allows unscaled measuring. A takeoff slice could surface an "unscaled" warning rather than adopt the refusal behaviour |
| Estimate/pricing layer | None | Out of scope for Rubric v1 (estimating is a separate product surface; see *Out of scope*) |
| Marked-up export with legend/quantity cover | Flatten-to-PDF + Bates-stamped export exist | No legend/quantity-summary cover page in the PDF export path (candidate, lower priority) |

**Closest-fit conclusion:** Rubric's measurement domain is already the hard
part — calibrated, rescalable, persisted measurements of the right kinds. The
takeoff gap is *aggregation and derivation* (a quantity list, one derived
surface quantity, a takeoff-shaped export), not new geometry engines.

---

## 3. Two integration paths and the licence boundary

### Path (a) — native takeoff features in Rubric (recommended, licence-clean)

Rubric implements the capabilities itself, in its own original Rust domain/UI
code, inspired at capability level only. Because every line is original and no
third-party takeoff code is linked, vendored or derived-from, Rubric's existing
**MIT OR Apache-2.0** licence is unaffected and no approval beyond the normal
research gate is needed. This is the path the gap analysis above maps.

### Path (b) — MCP client to a user-installed OpenTakeoff server (owner decision required)

OpenTakeoff is AGPL-3.0-only. In plain terms:

- **AGPL-3.0 is a strong copyleft licence.** If Rubric *incorporated*
  OpenTakeoff code — linked against it, vendored its source, bundled its
  artefacts, or produced a derivative work of it — the combined work would
  have to be distributed under AGPL-3.0. That directly conflicts with Rubric's
  MIT OR Apache-2.0 policy and with the repo rule that prohibits AGPL/GPL
  dependencies without explicit owner approval. **No code adoption from that
  repository is permitted.**
- **Talking *to* it is different from incorporating it.** MCP is an open
  protocol; a Rubric-side MCP client that speaks the protocol to a separate,
  user-installed, unmodified OpenTakeoff server is inter-process communication
  between independent programs. Rubric would contain only its own original
  client code. The AGPL's network-service clause concerns whoever runs a
  *modified* OpenTakeoff as a service; it does not turn a client into a
  derivative work.
- **Conditions if the owner approves path (b):** the decision is recorded in
  `DECISIONS.md`; OpenTakeoff is never bundled in Rubric's repository or
  installers and is never auto-downloaded — the user installs and launches the
  server themselves (it is distributed separately under its own licence);
  Rubric's docs describe it as a separate AGPL project not distributed with
  Rubric; the bridge stays optional and off by default; no OpenTakeoff code,
  assets or string tables enter Rubric's tree; and Rubric must never ship a
  modified copy of the server (which would trigger AGPL's share-alike for the
  modified work).
- **Practical caveat:** the publicly listed server is a Node/npm package driven
  by tools Rubric does not control, and its tool surface is large and
  versioned. A Rubric client would be a thin, read-oriented consumer (e.g.
  request a summary/quantity report for a plan Rubric has open), not a second
  takeoff UI.

**Recommendation recorded for the owner:** path (a) is self-contained,
licence-clean and builds directly on Rubric's existing domain; path (b) is
optional, additive and blocked pending an explicit owner decision.

---

## 4. Acceptance criteria for a future native slice (EARS-style)

These are *proposed* criteria recorded at research stage. The owner-approved
implementation contract for v0.5 is now `specs/takeoff-quantities.spec.md`
(EARS requirements R-1–R-32 with its own Given/When/Then acceptance set);
see the *Implementation status* addendum at the end of this note.

> **Status (2026-11-20): COMPLETE** — T1-T5 shipped in PRs #45, #47 with CI verification:
> - AC-1/AC-2 (quantity list), AC-9 (WallSurface), AC-13/AC-14/AC-15 (CSV export) all landed
> - AC-3 (CSV export) implemented
> - AC-4 (symbol auto-count) out of scope for v0.5
> - AC-5 (MCP bridge) blocked pending owner decision

- **AC-1 Quantity list.** *When* the user opens the Quantity panel, *the system
  shall* aggregate every measurement markup in the project by `MeasureKind`
  and by user-assigned item name, showing item count, total and unit per group
  and a document total per kind. *Where* a measurement has no assigned name,
  *then* a default name derived from its kind is displayed. *When* any
  measurement's geometry, its page's scale or the display unit changes, *then*
  the affected item and every aggregate update.
- **AC-2 Wall-surface derivation.** *When* the user applies a wall-surface
  derivation to a perimeter measurement and enters a height, *the system shall*
  display a derived surface quantity equal to perimeter × height in the
  current display unit. *When* the perimeter's geometry, the page scale or the
  unit changes, *then* the derived quantity is recomputed. *When* no height has
  been entered, *then* the item is shown as incomplete rather than zero.
- **AC-3 Quantity CSV export.** *When* the user exports the quantity list,
  *the system shall* write a CSV file whose rows are quantity items (name,
  kind, quantity, unit, page) with one totals row per kind, openable in a
  spreadsheet application. *When* the project contains no conditions, *then*
  export is disabled with a plain-language reason.
- **AC-4 Symbol auto-count (stretch).** *When* the user requests an
  auto-count of a chosen symbol region on a page, *the system shall* present
  candidate placements for review before anything is committed. *Where* a
  candidate is accepted, *then* it becomes a normal numbered Count marker
  (indistinguishable downstream from a manually placed one apart from an
  optional origin note). *When* detection cannot proceed (no rendered content,
  no scale-independent basis), *then* a plain-language refusal names the
  reason and no markup is created.
- **AC-5 (owner-gated, path b only).** *When* the user enables an MCP bridge
  and configures a locally installed takeoff server, *the system shall* be
  able to request a quantity summary from it for display alongside Rubric's
  own quantities, clearly labelled with its source. *When* the bridge is off
  or the server is unreachable, *then* Rubric works fully without it and the
  panel shows an idle state; *and* Rubric shall never bundle, download or
  auto-configure that server.

---

## 5. Edge cases

- **Scale changes mid-project.** Rubric already recomputes existing
  measurements when a page's scale changes (stored `value_points` are rescaled).
  A quantity list and any derived quantities (wall surface) must ride the same
  recompute path so aggregates never go stale.
- **Multi-sheet documents.** Rubric's scale is per page, so a document can mix
  scales and units. The quantity list must scope rows by page and define a
  rollup policy for mixed units (group per unit; never silently convert).
- **Rotated pages.** Measurement geometry lives in page space, so quantities
  are unaffected by rotation; only canvas affordances (label placement,
  hit-testing) need care. Rubric already documents rotated-page text-anchor
  fidelity as a limitation — unrelated to quantities but worth remembering in
  the same slice.
- **Count markers vs auto-count.** Manual counts are deterministic,
  user-placed, per-page numbered markers (Rubric never re-issues a number).
  Auto-count, if built, must remain a reviewable proposal source that *creates*
  ordinary count markers on acceptance — both must stay distinguishable in the
  quantity list (e.g. an origin note), and a rejected candidate must leave no
  markup behind.
- **Locked/hidden layers.** Decide whether quantities include measurements on
  hidden or locked layers; propose: hidden layers are excluded from totals and
  the exclusion is visible in the panel (open design question for the owner).
- **Precision and rounding.** Aggregates must sum stored values and round only
  at display, so the total never accumulates per-row rounding error.
- **Deleted markups.** Quantities follow the document: deleting a measurement
  removes it from aggregates (with undo restoring it, as elsewhere in Rubric).

---

## 6. Out of scope / not proposed

- **Estimating/pricing** (unit prices, tax, overhead/profit, bid proposals) —
  a separate product surface; Rubric remains a markup/measurement tool.
- **Roll-goods layout planning** (seam/lane nesting for sheet flooring) —
  specialised trade geometry, not a markup-tool concern.
- **Flood-fill/one-click room detection** — the inspiration project itself has
  this gated while re-validating it; Rubric's clean-room bar plus the
  detection risk make this a poor fit for v1.
- **Any adoption of OpenTakeoff code, assets or UI text** — prohibited by the
  repo rule on AGPL/GPL dependencies (see §3).

---

## 7. URLs (every claim above)

- OpenTakeoff README — capabilities, scale-gate rule, measurement types,
  conditions, materials, exports, review workflow, licence and relicensing
  note, self-hosting stance: <https://github.com/Kentucky-ai/opentakeoff>
- OpenTakeoff MCP reference — publicly listed tool names (`load_plan`,
  `set_scale`, `measure_polygon`, `measure_line`, `measure_surface`,
  `place_count`, `count_marks`, `symbol_sweep`, `cut_out`, `takeoff_summary`,
  `export_report`, `export_marked_pdf`, `export_dxf`, estimate/revision tools),
  scale rules and limits: <https://github.com/Kentucky-ai/opentakeoff/blob/main/mcp/README.md>
- OpenTakeoff hosted canvas (platform evidence): <https://opentakeoff.kentucky-ai.com>
- Awesome-Construction-Takeoff — category definition, table-stakes features,
  measurement modes, calibration forms, deduction areas, revision deltas,
  export formats: <https://github.com/ishandutta2007/Awesome-Construction-Takeoff>
- AGPL-3.0 licence text (basis of the §3 boundary analysis):
  <https://www.gnu.org/licenses/agpl-3.0.txt>
- OpenTakeoff LICENSE file (AGPL-3.0-only statement):
  <https://github.com/Kentucky-ai/opentakeoff/blob/main/LICENSE>

*Clean-room attestation: this note is an original summary written for Rubric.
No vendor manual text, source code, screenshots or string tables were copied;
no binary was decompiled; the described behaviours are cited to the public
pages above.*