# Specification — UI Parity Revamp: Docking, Split Views, Workspaces & Preferences (v0.6)

Status: **Draft for owner approval** (research gate: `docs/research/bluebeam-vs-rubric-audit.md`,
rows G3/G5/G6/D-series/B6/B7 — approved 2026-11-15).
Clean-room note: this spec maps **capability-level** behaviours only. Nothing here copies
Bluebeam Revu's icons, fonts, string tables, exact layouts or trade dress (guide §2.2, II.27).
Every UI element is original Rubric design built on egui tokens (II.30).

Owner decisions already locked: **multi-document tabs shipped (v0.4.11-era PR #227)**;
docking is now a product goal (this spec), which unlocks the II.30.4 `egui_dock` adoption
rule — recorded in `DECISIONS.md` (D-UI-1).

---

## 1. Overview and user value

Bluebeam-class estimators live inside one window all day: panels docked where they want
them, two drawings side by side for revision comparison, a preferences surface that makes
shortcuts discoverable. Rubric today is a **fixed layout**: Pages left, Tools + Tool Chest
centre-left, canvas centre, Inspector right, bottom panel for lists. The audit calls this
the largest structural gap (#12, G3/G5).

This program rebuilds the shell around four capabilities:

1. **Docking** — every panel can be moved between docked regions (left / right / bottom)
   and closed/reopened without losing state.
2. **Floating windows** — any panel can detach into a movable, resizable window (for
   dual-monitor workflows).
3. **Split views** — the canvas area splits into 1 or 2 vertical panes (and a horizontal
   split in v2), each with its own page/zoom/pan but shared selection.
4. **Preferences & shortcuts surface** — a dialog where the user inspects every shortcut,
   edits the ones that are remappable, sets default units/precision/autosave, and sees the
   app version/release-notes entry point (§9.4 gate).

**User value:** an estimator arranges Rubric once — takeoff quantities bottom-left, pages
right, two sheets side by side — and the workspace persists across restarts. Nothing about
the arrangement requires the mouse-only fiddling that fixed layouts force.

**Non-goals (this program):** ribbon/menu-bar redesign, right-click radial menus, custom
theme editor, plugin surface, mobile/touch layouts, multi-window app-level (OS) windows.

---

## 2. Architecture decisions (DECISIONS.md D-UI-1…D-UI-4)

| ID | Decision | Rationale (II.30.4 mapping) |
|----|----------|------------------------------|
| D-UI-1 | **Adopt `egui_dock`** (pinned to the version resolving against egui 0.37 — verify in Cargo.lock before the slice; never bump egui in this program) | II.30.4: adopt only when user-rearrangeable panels are a product goal — they now are. One docking path per app; no second theming/dock system. |
| D-UI-2 | **Split views are Rubric-original** (hand-rolled two-pane canvas state), not a dock feature | egui_dock docks panels; split *views of one document* is app semantics. Keeps dock tree simple. |
| D-UI-3 | **Workspace = serialised dock tree + panel visibility + split state**, stored in `AppPrefs` (`#[serde(default)]`, legacy-safe) | One restart-restore story; no separate store. |
| D-UI-4 | **Shortcuts stay code-defined** (existing `Tool` shortcut table); the preferences dialog is read-mostly with remapping only for the actions the domain already routes through `EditorAction` | Remapping everything is a v2; do not fake it (dead-code rule). |

---

## 3. Domain model (pure, serde, headless)

### 3.1 `domain/workspace.rs` (new)

```rust
pub enum PanelId { Pages, Tools, ToolChest, Inspector, MarkupsList, Quantities }

pub struct WorkspaceState {
    /// Which panels are open.
    pub open: BTreeSet<PanelId>,
    /// Dock region per open panel.
    pub region: BTreeMap<PanelId, Region>,
    pub floating: BTreeMap<PanelId, FloatState>,
    /// Canvas split: 1 = single, 2 = vertical side-by-side.
    pub split: SplitState,
}

pub enum Region { Left, Right, Bottom }
pub struct FloatState { pub rect: [f32; 4] }   // x, y, w, h in points
pub struct SplitState { pub count: u8, pub ratio: f32, pub secondary: SecondaryView }

pub struct SecondaryView {
    pub page: usize,        // independent page per pane
    pub zoom: f32,
    pub pan: [f32; 2],
    pub follow_primary: bool, // v1: default true
}
```

- Every field `#[serde(default)]` → legacy prefs load unchanged (R-P-1).
- Methods are pure: `open_panel`, `close_panel`, `move_panel(id, region)`, `float_panel(id,
  rect)`, `dock_panel(id)`, `set_split(count, ratio)`, `set_secondary_page(p)`.
- Validation: at most one of each panel open; split count ∈ {1, 2}; ratio clamped 0.2–0.8;
  floating rects clamped into a sane viewport box.
- **Selection is shared** across split panes (one `selected` set — matching Bluebeam's
  behaviour where a selection is document-wide); each pane has its own page/zoom/pan.

### 3.2 Constraints the domain enforces

- The canvas is never closable and never floats (it is the app's centre; II.30.11 narrow-
  window rule keeps it last-shrunk).
- At least one side/bottom panel may be closed, but the dock tree must always have ≥1 panel
  open somewhere (the dock area cannot be empty; enforced in `WorkspaceState::validate`).
- `PanelId::Quantities` exists only after the takeoff feature lands (feature-flagged by a
  compile-time `cfg`/registry so this spec composes with `specs/takeoff-quantities.spec.md`
  without ordering coupling).

---

## 4. Functional requirements (EARS)

### Docking (G3)

- **R-D1.** When the user drags a panel's title tab over a dock region indicator (left /
  right / bottom / centre-of-pane), the system shall preview the drop zone with a
  token-coloured overlay and, on drop, persist the new region to `WorkspaceState`.
- **R-D2.** When the user drops a panel onto another panel's header, the system shall stack
  them as tabs within that region (tab strip per region), preserving the hidden panel's
  state.
- **R-D3.** When the user clicks a panel's close control, the system shall remove it from
  `open` (state retained in `WorkspaceState` defaults so reopening restores content).
- **R-D4.** When the user reopens a panel from the **Panels** menu (View ▸ Panels ▸ …), the
  system shall restore it in its last region (or the default region if never moved).
- **R-D5.** When a floating panel's `FloatState` would place it off-screen, the system shall
  clamp it into the viewport on restore.
- **R-D6.** When the user double-clicks a panel's header, the system shall toggle between
  docked and floating (Bluebeam-parity affordance), persisting the float rect.
- **R-D7.** When the workspace loads, the system shall rebuild the dock tree from
  `WorkspaceState` exactly; on parse failure the system shall fall back to the default
  layout and surface a status message (fail-open, matching layer semantics).

### Split views (G5)

- **R-S1.** When the user activates **View ▸ Split vertical**, the system shall render two
  canvas panes sharing one selection, each with independent page/zoom/pan.
- **R-S2.** When the user drags the split divider, the system shall update `ratio` (clamped
  0.2–0.8) and persist it.
- **R-S3.** When `follow_primary` is on and the user changes page/zoom in the primary pane,
  the system shall mirror to the secondary pane; toggling follow off freezes the secondary.
- **R-S4.** When a markup is selected in either pane, the system shall highlight it in both
  panes (shared selection) and scroll the owning pane to keep it visible.
- **R-S5.** When the user deactivates the split, the system shall keep the primary pane's
  view state and discard secondary-only state.
- **R-S6.** When either pane renders, the system shall route ALL painting/hit-testing
  through that pane's own `to_screen_rotated` mapping (the PR #223 rotation architecture) —
  no shared transform state between panes.

### Preferences & shortcuts surface (G6 + §9.4)

- **R-P1.** When the user opens **Edit ▸ Preferences…**, the system shall show a modal
  dialog with tabs: General (theme, autosave, default unit, precision), Tool Chest
  (Max-Recents), Shortcuts, About (version → release-notes popup, §9.4).
- **R-P2.** When the Shortcuts tab opens, the system shall list every registered shortcut
  from the existing code-defined table (tool shortcuts + EditorAction bindings + Ctrl+Tab
  document cycling) with its action label and key chord.
- **R-P3.** When the user edits a remappable action's chord, the system shall validate
  uniqueness (reject chords already bound, showing the conflicting action), persist to
  `AppPrefs.shortcut_overrides: BTreeMap<ActionId, String>` (`#[serde(default)]`), and
  apply on next input dispatch.
- **R-P4.** When the user clicks Reset, the system shall clear overrides for that action
  and restore the code-defined chord (one undo snapshot is NOT pushed — preferences are not
  document state; persistence is immediate via `save_app_prefs`).
- **R-P5.** When a chord conflicts on load (stale override vs new default), the system shall
  drop the override, fall back to the default, and log to `docs/devlog.md` (fail-open).
- **R-P6.** When the General tab changes theme/precision/default unit, the system shall
  reuse the exact existing settings paths (no parallel settings store).

### Tool Chest completion (B6/B7)

- **R-T1.** When a tool set is selected in the filter, the system shall offer **Set Scale…**
  where the estimator enters (or calibrates) a scale that all presets in the set store; on
  placement, the placed markup's page adopts the set's scale when the page is uncalibrated
  (B6 — set scale is a *suggestion* the drawing's calibration overrides; the audit's
  scale-gate semantics).
- **R-T2.** When the user pins a set (row context menu ▸ Pin to toolbar), the system shall
  show that set's presets as a toolbar flyout group (B7 — the documented Toolbar↔Chest
  bridge), persisted in `AppPrefs.pinned_tool_sets: Vec<Uuid>` (`#[serde(default)]`).
- **R-T3.** When a pinned set's preset is used, it records into Recent Tools (R-#3 path)
  without duplication.

### Cross-cutting

- **R-X1.** When any shell change lands, all new controls shall satisfy II.30.7: ≥24 pt
  targets, AccessKit names, keyboard path, Esc closes floating panes and the preferences
  dialog, focus never trapped.
- **R-X2.** When the Smart-Zone guard would be hit by a slice, the slice is split further
  (§5.6.1) — every T-item below is sized for one session.
- **R-X3.** When egui_dock's API conflicts with the token/theming rules (II.30.6), the
  dock's visuals shall be mapped through `Tokens` (one token source; no scattered hex).
- **R-X4.** When the program lands, the kittest shell baselines SHALL be regenerated once,
  reviewed by the designer role, and committed with the dock slice (artefacts in PR, not
  the repo).

---

## 5. Non-functional requirements

| ID | Requirement |
|----|-------------|
| NFR-1 | Panel drag/drop and split-divider drag stay interactive ≥ 30 fps on the reference machine — dock tree updates are O(panels), not O(markups). |
| NFR-2 | Workspace restore ≤ 50 ms from prefs (pure deserialise + rebuild; no I/O beyond prefs). |
| NFR-3 | No `unwrap`/`expect` in dock/parse paths (§2.6); every parse failure is a fallback, not a panic. |
| NFR-4 | `egui_dock` pinned; a version bump is its own slice with CI green + evidence re-review (II.30.3). |
| NFR-5 | All new state rides `AppPrefs` serde-defaults; legacy prefs load unchanged (NFR-4 of the takeoff spec pattern). |
| NFR-6 | The raw-colour grep gate stays green (II.30.6); dock visuals mapped via `Tokens::visuals()`. |
| NFR-7 | 416+ existing tests stay green at every slice boundary — no behaviour change at rotation 0 / single-pane. |

---

## 6. Acceptance criteria (Given / When / Then)

### Docking

- **AC-D1.** Given the Inspector docked right, when the user drags its tab to the left
  region indicator and drops, then the Inspector renders left, `WorkspaceState.region`
  persists, and restarting the app restores it.
- **AC-D2.** Given two panels in one region, when the user drops a third onto that region's
  header, then the region shows three tabs and each tab retains its own state when
  switched.
- **AC-D3.** Given the Inspector floating, when the user quits and relaunches with the
  float rect off-screen, then the panel is clamped visible.
- **AC-D4.** Given all side panels closed, when the user opens View ▸ Panels ▸ Inspector,
  then it reappears in its last region.
- **AC-D5.** Given a corrupted `workspace` blob in prefs, when the app starts, then the
  default layout loads, a status message names the fallback, and no panic occurs.

### Split views

- **AC-S1.** Given a 3-page document, when the user enables split vertical and sets the
  secondary to page 3, then both panes render independently and the divider drag persists
  after restart.
- **AC-S2.** Given split view with `follow_primary`, when the user zooms in the primary,
  then the secondary mirrors; when follow is toggled off, secondary freezes.
- **AC-S3.** Given a markup selected in the secondary pane, then the primary highlights the
  same markup (shared selection).
- **AC-S4.** Given a rotated page (90°) in the secondary pane, then the pane renders via
  the rotated mapping (PR #223) and `to_page(to_screen(p)) == p` still holds.

### Preferences

- **AC-P1.** Given the Shortcuts tab, then every code-defined shortcut is listed with its
  chord and action label (spot-check ≥ 10 rows against the documented table).
- **AC-P2.** Given the user remaps "Export Quantities" to Ctrl+Shift+Q (unbound), then
  Ctrl+Shift+Q triggers the action after restart and the old chord is freed.
- **AC-P3.** Given the user remaps an action to Ctrl+S (already bound to Save), then the
  dialog rejects with "Conflicts with: Save document" and no override is stored.
- **AC-P4.** Given a stale override for a removed action, when prefs load, then the
  override is dropped and the default applies (no dialog).

### Tool Chest completion

- **AC-T1.** Given a set with Set Scale 1:50, when a preset from that set is placed on an
  uncalibrated page, then the page adopts 1:50 and the placed measure reports in the
  calibrated unit; when the page IS calibrated, the page keeps its own scale.
- **AC-T2.** Given a pinned set, then the toolbar shows its flyout group; clicking a preset
  arms it exactly as the chest row does (mode included) and records a Recent.

---

## 7. Error handling table

| Situation | Detection | Response | User sees |
|---|---|---|---|
| Corrupted workspace blob | serde failure on load | Default layout + status message | "Workspace reset (unreadable layout)" |
| Float rect off-screen | Viewport clamp on restore | Clamped into view | Panel visible (silent) |
| Dock area would be empty | `WorkspaceState::validate` | Refuse close, tooltip reason | "At least one panel stays open" |
| Split ratio out of bounds | Clamp in domain | Clamped 0.2–0.8 | Divider snaps (silent) |
| Shortcut chord conflict | Uniqueness check | Reject override | "Conflicts with: <action>" |
| Stale shortcut override on load | Lookup miss | Drop override, use default | Silent + devlog entry |
| egui_dock API mismatch at build | CI compile | Slice blocked; pin fix | n/a (CI gate) |
| Export/import of workspace (v2) | n/a | Out of scope this program | n/a |

---

## 8. Phased implementation checklist (T1–T12; each = one PR, CI-gated)

**Phase A — foundations (no visible change)**

- [ ] **T1 — Workspace domain.** `domain/workspace.rs` types + validation + serde
      roundtrip + legacy-load tests. Tests: `workspace_roundtrip`,
      `legacy_prefs_load_default_workspace`, `validate_refuses_empty_dock`,
      `ratio_clamped`.
- [ ] **T2 — egui_dock adoption spike (timeboxed).** Add pinned dep; render the existing
      fixed layout INSIDE a dock area with all regions fixed (zero behaviour change);
      verify kittest baselines unchanged at rotation 0 / single pane. CI gate + snapshot
      proof. **Rollback plan:** revert the dep; no state written yet.
- [ ] **T3 — DECISIONS/ATTRIBUTION.** D-UI-1…D-UI-4 entries; egui_dock licence recorded.

**Phase B — docking**

- [ ] **T4 — Panels menu + close/reopen.** R-D3/R-D4 with `WorkspaceState.open` (no drag
      yet). Tests: open/close/reopen domain + headless UI predicates.
- [ ] **T5 — Region moves via menu.** R-D1's menu-driven subset (View ▸ Panels ▸ Move
      Inspector ▸ Left/Right/Float) — drag comes after the state plumbing is proven.
- [ ] **T6 — Drag & drop.** egui_dock drag with drop-zone preview (R-D1, R-D2 tab
      stacking). Tests: domain state transitions; kittest drag is manual-QC'd (documented
      limitation).
- [ ] **T7 — Floating.** R-D5/R-D6 float toggle + clamps. Tests: clamp logic headless.
- [ ] **T8 — Persistence.** Workspace in `AppPrefs`; restart-restore tests; corrupted-blob
      fallback test (AC-D5).

**Phase C — split views**

- [ ] **T9 — Split state + two-pane rendering.** R-S1/R-S6 with independent per-pane view
      state; rotation-mapping reuse. Tests: AC-S4 roundtrip at 90°.
- [ ] **T10 — Divider + follow mode.** R-S2/R-S3. Tests: ratio clamp; follow semantics.
- [ ] **T11 — Shared selection.** R-S4. Tests: selection visible in both panes (domain
      selection is shared by construction — UI test asserts highlight path).

**Phase D — preferences & tool chest**

- [ ] **T12 — Preferences dialog shell.** R-P1 tabs General/Shortcuts/About; General tab
      reuses existing settings controls (move, don't duplicate).
- [ ] **T13 — Shortcuts tab.** R-P2 read-only listing first (table from the code-defined
      registry), then R-P3/P4 remap+persist for EditorAction-bound actions.
- [ ] **T14 — Tool set scale + pinning.** R-T1/R-T2 (B6/B7).
- [ ] **T15 — Program closeout.** Baselines regenerated once (R-X4), `docs/qc/shortcuts.md`
      updated, capability-inventory rows → implemented with CI evidence, CHANGELOG
      `[0.6.0]`, ROADMAP ticks with run IDs.

**Dependency order:** T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8 → T9 → T10 → T11 → T12 → T13 →
T14 → T15. T12/T13 can start after T8 (independent of C). Parallel pairs (file-disjoint):
(T4,T5) with (T9) after T2; (T12) with (T6) after T8.

---

## 9. Risks and mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| egui_dock vs egui 0.37 API drift | Medium | T2 spike pins and proves compile before any state work (II.30.3) |
| Dock drag breaks kittest labelled-widget queries | Medium | Tabs keep AccessKit names; baseline regen is a reviewed, single event (R-X4) |
| Split panes double the paint cost | Medium | Panes render the same cached page textures; measure in NFR-1 test |
| Shortcut remap breaks muscle-memory docs | Low | `docs/qc/shortcuts.md` regenerated with T13 |
| Scope creep into theme editor / ribbons | High (mimic-request gravity) | Non-goals frozen in §1; new asks go to ROADMAP via the gate |
| AGPL-adjacent docking code | None | egui_dock is MIT/Apache — verified in T3 before adoption (§25 rule 3) |

---

## 10. Explicit non-goals (frozen)

Ribbon/menu-bar redesign · custom theme editor · plugin system · OS-level multi-window ·
touch layouts · per-monitor workspaces · shortcut chords with mouse buttons · docking the
canvas itself.

---

## 11. Gates

- Research basis: approved audit rows G3/G5/G6/B6/B7 (§2.3 gate satisfied for mapping; this
  spec is the implementation contract).
- Per-slice gates: `ci.sh run` green (check + snapshot + scanners), PR template, merge via
  Dev identity (§9.3).
- Program gate: §9.1/§9.2 hygiene + AC-D1…AC-T2 verified + baselines reviewed (R-X4) before
  the v0.6.0 tag.
