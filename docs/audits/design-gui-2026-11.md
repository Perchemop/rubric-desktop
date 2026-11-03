# Rubric Design & GUI Audit

**Date:** 2026-11-03  
**Scope:** Design system, visual language, information architecture, interaction design, accessibility surface, and polish of the egui/eframe desktop UI  
**Tree:** `main` @ ~942a667 (post v0.4 gaps 1–8)  
**Method:** Read-only review of `apps/rubric/src/ui/*`, `docs/design/`, `docs/a11y.md`, `ACCESSIBILITY.md`, `theme.rs`, brand assets, prior full-app audits, CHANGELOG, and ROADMAP. No local binary run; interaction conclusions are code- and doc-derived.

**Headline:** Architecture, honesty, and workflow completeness are strong. The remaining distance to a finished professional product is primarily **visual craft, systemisation, discoverability, and empty-state design** — not missing core capabilities.

---

## 1. Product context

**Rubric** is a free, local-first PDF markup, review, and measurement tool (Rust + egui/eframe). It targets reviewers who currently print PDFs or pay for heavyweight commercial tools.

**Design constraints that matter:**

- Immediate-mode GUI (egui) — no retained widget tree, limited native chrome control.
- Domain-first architecture — UI must not own business truth.
- Paper-white document surface in both themes (trust rule).
- No telemetry, no accounts, no cloud UI.
- Keyboard operation and WCAG-intent accessibility treated as requirements.

**Reference artefacts:**

| Artefact | Role |
|---|---|
| `docs/design/brand.md` | Brand intent (mark, palette, usage rules) |
| `apps/rubric/src/ui/theme.rs` | Live chrome palette + WCAG unit tests |
| `docs/a11y.md` / `ACCESSIBILITY.md` | A11y commitment and limits |
| `docs/audits/v0.3-full-app-audit.md` | Prior functional/UX defect register |
| `docs/research/v0.4-capability-gap-analysis.md` | Workflow parity gaps (now closed in code) |
| `docs/research/design-tooling-2026-11.md` | Icon/font/tooling adoption plan |

---

## 2. Executive summary

### What is already excellent

1. **Domain/UI separation** — geometry, markups, measurements, history, and persistence are pure; the UI is a thin, honest layer.
2. **Brand honesty** — original mark, no competitor trade dress, paper-always-white rule, intentional indigo dark chrome.
3. **Interaction integrity** — dirty-state handling, recovery vs real save, focus-gated shortcuts, shared context-menu action path, measurement recompute on move, degenerate-drag rejection.
4. **Workflow completeness (v0.4)** — multi-select, resize + vertex edit, drag calibration, tool chest, batch status, flatten-to-PDF, duplicate, editable Markups List.
5. **Accessibility posture** — contrast maths + CI palette tests, non-colour-only status, tooltips on icon-only controls, documented limits.

### What still holds the product back

1. **Visual system is under-specified** — emoji/text glyphs, default typography, limited elevation tokens.
2. **Layout density and persistence** — many panels; workspace state not saved.
3. **Discoverability** — multiple tool-entry points, no global shortcut legend, sparse empty states.
4. **Automated visual evidence** — no `egui_kittest` Host seam yet (G4).
5. **Polish consistency** — ellipsis rules, handle sizing vs zoom, note popover constraints, title dirty indicator.

**Overall design/GUI grade: B+ (approaching A-)**  
Engineering and interaction honesty exceed typical early egui apps. Visual refinement and systemisation are the next lever.

---

## 3. Brand & visual identity

### 3.1 Brand mark

From `docs/design/brand.md`:

- Geometric "sheet" (page) + bold diagonal stroke ending in a tick.
- Reads as page + reviewer mark at 16 px through 1024 px.
- Flat vector, single-weight strokes, no gradients/bevels/shadows.

**Verdict:** Clear, original, and appropriate. Construction rules are documented. No change required to the concept.

**Recommendations:**

- Audit committed raster icon set (`assets/icons/`) against the SVG at every target size (16/32/48/256/512).
- Ensure mono variant exists for dark/busy backgrounds as the brand doc requires.
- Close packaging item Q-PACK-2 (CI icon pipeline: resvg → ico/icns) so generated icons stay in sync with the SVG.

### 3.2 Colour system

| Token | Role | Notes |
|---|---|---|
| Rubric orange `#E4572E` | Selection / emphasis fill | ~3.7:1 on white — AA large only; correctly not used as body text |
| Accent 2 `#FF7A45` | Dark-theme links | ≥ 4.5:1 on indigo panel |
| Ink neutrals | Body text, rulers, chrome lines | Explicit `override_text_color` per theme |
| Paper `#FFFFFF` | Document page | **Always light** in both themes |
| Light chrome | Warm near-white panels | Audited body contrast |
| Dark chrome `INK_900` / `INK_700` | Window surfaces | Distinctive; page stays paper |

**Strengths**

- Paper-vs-chrome rule is the single most important visual trust signal for a measurement tool.
- Body text colours are explicit and CI-tested (`shipped_palettes_meet_wcag_contrast`).
- Status colours are paired with text labels everywhere they appear.

**Gaps**

| Severity | Finding | Recommendation |
|---|---|---|
| Minor | Accent is strong as fill but the overall light theme reads slightly flat | Introduce 2–3 elevation steps (panel / raised / overlay) with consistent borders |
| Minor | Dark indigo is distinctive but elevation tokens are thin | Same elevation scale; avoid pure black |
| Polish | Status amber historically failed AA as list text (fixed with darkened `text_color` on light theme) | Keep domain test; never use swatch colour alone for body text |

### 3.3 Typography

Current state: egui default proportional font; no bundled UI or measurement face.

| Severity | Finding | Recommendation |
|---|---|---|
| Major | Measurement values and dense tables lack tabular figures | Bundle a monospace with tabular nums (e.g. JetBrains Mono OFL) for measurement readouts, scale fields, and list columns |
| Major | UI lacks a consistent type scale | Define caption / body / section / heading sizes and use them via egui `RichText` or style overrides |
| Minor | No brand-aligned UI sans | Bundle Inter (or equivalent OFL) for chrome; keep paper/document rendering independent |

**Adoption path** (already researched in `docs/research/design-tooling-2026-11.md`): load fonts once at startup; fall back gracefully if missing.

### 3.4 Iconography

Current state: emoji and text glyphs (eye/lock arrows, labelled selectable rows). Functional, not professional.

| Severity | Finding | Recommendation |
|---|---|---|
| Major | Emoji icons vary by OS font and look inconsistent | Adopt an icon font subset (egui-phosphor or Lucide/Tabler SVGs with `currentColor`) |
| Major | Tool kinds lack distinct silhouettes in the palette | Kind-specific glyphs for Review / Shapes / Paths / Measure |
| Minor | Layer and page controls rely on hover tips alone for meaning | Keep tips; add text labels in wider layouts or a "Show labels" preference |
| Polish | Tool Chest apply-to-selection control is cryptic | Prefer a short text button ("Apply") or a standard paint glyph + tip |

---

## 4. Information architecture and layout

### 4.1 Shell structure

Primary layout: menu bar; left Tools / Tool Chest; centre Canvas; right Inspector / Layers / Scale / Status legend; optional Markups List and Pages sidebar; status/hint bar.

**Strengths**

- Clear primary surface (canvas) with secondary panels.
- View menu toggles for Markups List and Tool Chest.
- Shared context menu between canvas and inspector (single action path).
- Pages sidebar with current-page highlight and honest placeholders when no PDF.

### 4.2 Layout issues

| ID | Severity | Finding | Recommendation |
|---|---|---|---|
| L1 | Major | Many simultaneous panels compete on laptop widths | Persist panel visibility and widths; add Focus/Reading mode |
| L2 | Major | Workspace state is session-only | Persist panels, filters, zoom, page in prefs |
| L3 | Minor | Inspector is a long vertical stack | Segmented tabs: Selection / Properties / Comments / Scale / Layers |
| L4 | Minor | Active tool can disappear when group is collapsed | Persistent active-tool chip |
| L5 | Minor | Multiple tool-selection surfaces can diverge | Single tool metadata table drives all UIs |
| L6 | Polish | Window title does not emphasise dirty state | Asterisk or Unsaved in title |

### 4.3 Navigation and hierarchy

- Global shortcuts in README and hint bar.
- Page Up/Down, Fit Page, zoom presets present.
- Missing: Help Keyboard Shortcuts window; stronger first-run empty guidance; clearer Tool Chest vs main palette hierarchy.

---

## 5. Interaction design

### 5.1 Strengths (post v0.3 + v0.4)

Multi-select, resize + vertex edit, drag calibration, duplicate, batch status, copy/paste properties, z-order, focus-gated shortcuts, unsaved confirms, measurement recompute on move, screen-space hit tolerance, degenerate drag rejection, shared context menu action path, Esc cancel.

### 5.2 Remaining findings

| ID | Severity | Finding | Recommendation |
|---|---|---|---|
| I1 | Major | Text markups are geometry bands, not text-anchored | Accept for v1 or schedule text-extraction path |
| I2 | Major | Canvas is pointer-primary | Document keyboard paths; consider nudge + numeric offset |
| I3 | Minor | Draft vs committed style can diverge | Unify draft rendering |
| I4 | Minor | Note popover fixed size | Dynamic size, scroll, edge flip |
| I5 | Minor | Handles/status dots scale inconsistently with zoom | Constant screen-space size |
| I6 | Polish | Rulers primarily in points | Optional dual-unit rulers |
| I7 | Polish | Select All feedback is thin | Status: N markups selected |

### 5.3 Gesture model

Idle leads to tool drawing, marquee multi-select, handle resize, body translate (with recompute), or context menu actions. Mode visibility (active tool + armed preset) is the main residual risk.

---

## 6. Component-level review

**Tool palette:** collapsing groups match domain; dual horizontal/vertical paths risk drift — single metadata source preferred.

**Tool Chest:** arm / save / apply / rename / delete workflow is clear; library is session-only — persist in prefs; richer empty state; labelled actions.

**Inspector:** strong legend, layers, scale; long stack — segment; monitor colour debounce.

**Markups List:** real console with filters, inline status, totals, CSV — virtualise; clarify empty filtered vs empty document.

**Pages sidebar:** good placeholders and cache poisoning; async thumbnails recommended.

**Canvas chrome:** export correctly suppresses editor chrome; screen-space handles; optional minimap later.

**Menus and dialogs:** shortcuts improved; standardise ellipsis; Help Keyboard Shortcuts; Save and Quit present.

---

## 7. Accessibility (GUI surface)

In place: WCAG contrast tests, explicit body colours, keyboard reachability, tooltips, non-colour-only status, a11y issue template.

Accepted limits: canvas drawing surface; incomplete AccessKit bridge; no pure serial command mode.

Gaps: A1 automated AccessKit/snapshots (G4); A2 focus ring audit; A3 UI scale / high-contrast pref; A4 text labels when narrow panels use icons only.

---

## 8. Design system formalisation

Token layers: Brand, Chrome, Elevation (0–3), Spacing (4–24), Radii, Type scale, minimal Motion.

Component recipes: ToolButton, Swatch, StatusDot, PanelHeader, EmptyState, ConfirmDialog, HintBar — small pure `&mut Ui` helpers.

---

## 9. Empty, error, and loading states

Target intentional empty states for no document, no markups, empty Tool Chest, filtered list with no matches, thumbnail failure (keep placeholder), export failures (status + dialog when destructive).

---

## 10. Consistency checklist

Standardise menu ellipsis; complete shortcut display; dirty title indicator; always-visible active tool; screen-space handles; unify draft vs final stroke; keep status-after-action discipline; re-test light/dark after icon/font work; icon + label policy.

---

## 11. Priority backlog (design / GUI only)

### P0 — High impact

1. Icon font/SVG set for tools, layers, Tool Chest, page nav (M)
2. Bundle UI + tabular mono fonts (S–M)
3. Persist workspace layout (M)
4. Global Keyboard Shortcuts dialog + active-tool chip (S)
5. Empty states for primary panels (S)

### P1 — Quality

6. Elevation + spacing + radius tokens (S)
7. Segmented Inspector (M)
8. Virtualised Markups List + async thumbnails (M)
9. Single tool metadata source (S–M)
10. Persist Tool Chest library in prefs (S)
11. Host/Ui seam + egui_kittest visual/a11y evidence G4 (L)

### P2 — Later

12. Text-anchored highlight/strikeout
13. Focus / reading mode
14. Dual-unit rulers
15. UI scale + high-contrast preference
16. Optional minimap

---

## 12. Relationship to prior audits

Functional blockers and majors from `docs/audits/v0.3-full-app-audit.md` are largely fixed; v0.4 gaps 1–8 are closed. Design-tooling research exists; icon/font/G4 adoption is partial. This audit does not re-litigate domain/data defects.

---

## 13. Suggested acceptance criteria for a design-complete milestone

1. Consistent icon set (no emoji dependency for core chrome).
2. Tabular mono for measurement and table numerals.
3. Workspace layout survives restart.
4. Intentional empty state on every primary panel.
5. Help lists every shortcut; active tool always visible.
6. Contrast + at least one visual snapshot suite in CI (G4).
7. Brand icon pipeline is CI-owned and matches `assets/logo.svg`.
8. Manual keyboard-only full review workflow succeeds without a pointer.

---

## 14. Conclusion

Rubric product design principles — local-first, honest state, paper-white truth, keyboard reach, original brand — are coherent and already visible in the code.

The GUI absorbed a large functional audit and a full capability-gap cycle. What remains is visual craft: a small design system, professional icons and type, persistent layout, empty states, and automated visual evidence.

None of that requires abandoning egui or the domain architecture. It requires treating chrome with the same seriousness already applied to measurements and undo.

---

## Appendix A — File map (UI)

| Path | Role |
|---|---|
| `apps/rubric/src/ui/app.rs` | Shell, UiState, menus, dialogs, orchestration |
| `apps/rubric/src/ui/canvas.rs` | Page render, drawing, hit-test, handles, popovers |
| `apps/rubric/src/ui/inspector.rs` | Properties, layers, scale, legend |
| `apps/rubric/src/ui/tool_chest.rs` | My Tools / style presets |
| `apps/rubric/src/ui/toolbar.rs` | Tool groups / simplified toolbar |
| `apps/rubric/src/ui/markups_list.rs` | Filterable markup console |
| `apps/rubric/src/ui/pages.rs` | Thumbnail sidebar |
| `apps/rubric/src/ui/menu.rs` | Shared context menu |
| `apps/rubric/src/ui/theme.rs` | Chrome palette + WCAG tests |
| `apps/rubric/src/ui/icons.rs` | Icon helpers (thin today) |
| `docs/design/brand.md` | Brand specification |
| `docs/a11y.md` | Technical a11y notes |

## Appendix B — Severity legend

| Severity | Meaning |
|---|---|
| **Major** | Blocks professional perception or causes repeated user confusion / layout failure |
| **Minor** | Noticeable quality gap; workaround exists |
| **Polish** | Consistency or delight; safe to batch |

---

*End of design and GUI audit — 2026-11-03*
