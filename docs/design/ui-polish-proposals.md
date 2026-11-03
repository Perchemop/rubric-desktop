# Rubric Desktop UI Polish — Current-State Inventory & Design Proposals

Status: design proposal (no code changes). Authored for the Rubric UI developer stage.
Scope: `apps/rubric/src/ui/*` (egui/eframe 0.36). References Rubric UI elements by
function, not by file path. All colour/contrast claims are against the shipped
palettes in `theme.rs` and `assets/branding.md`.

---

## PART 1 — Current-state inventory (12 layers)

### 1. Branding & Chrome

**What works well**
- Two themes (light default, dark indigo) with the document page always paper-white —
  a strong, trustworthy rule (see `Theme` toggle in the View menu and `apply_theme`).
- Brand accent (Rubric red-orange `#E4572E` / `#FF7A45`) used consistently for selection
  and active tool states; accent 2 (`#FF7A45`) reserved for dark-theme hyperlinks so it
  clears WCAG AA on indigo.
- App icon loaded from `assets/icons/icon_256.png` and raster set 16→1024 (brand.md).
- Window title re-syncs to the open file name ("… — Rubric") via `ViewportCommand::Title`.

**What could be improved**
- Top-level window chrome is mostly egui defaults; no custom title-bar styling or
  wordmark in the menu bar. Brand mark (`assets/logo.svg`) is never shown in-app.
- Panel fills differ slightly between light (`#FAFAFC`) and dark indigo, but there is no
  single "chrome token" set exposed for reuse (radius, elevation, shadow are implicit).
- Dark theme selection text-on-accent only clears AA-large, not AA-normal — acceptable
  for large selected glyphs, but small selected text (e.g. selected tool label) should
  use the dark text colour on the accent fill or a darker accent.

### 2. Top-level Navigation & Menus

**What works well**
- Standard menu bar: File / Edit / View / (implicit Tools via left panel) / Help,
  built with `MenuBar`. Shortcuts shown via `shortcut_text` on action buttons.
- File menu is comprehensive: New, Open, Import PDF, Export Report/Page Image/PDF, Print,
  Recent submenu, Save/Save As, Auto-save submenu, Quit.
- Edit menu exposes Undo/Redo, Select All, selection commands, order, batch status.
- View menu exposes zoom presets, page navigation, grid, theme, panel toggles, full screen.
- Help menu: About, Check for Updates, Report a Bug.

**What could be improved**
- Menu items are mostly plain `ui.button` with mixed ordering; separators are
  inconsistent (some groups separated, others not). No "Tools" top-level menu — tools
  live only in the left panel, so keyboard-only users reach them via context menu Tool
  submenu or shortcut keys.
- Accelerators are only shown on a subset of actions; many (Zoom In/Out, Fit Page, page
  nav) lack shortcut hints in the menu.
- No recent-files keyboard accelerator; "Recent" is a submenu only.
- Menu bar height and padding follow egui defaults; could be tightened to a 28–32 px
  bar with consistent 8 px horizontal padding.

### 3. Tool Palette & Toolbar

**What works well**
- Two surfaces: a horizontal top toolbar (Select, Highlight, Rectangle, Ellipse, Line,
  Arrow, Pen, Text Box, Length) and a grouped left "Tools" panel (collapsing Review /
  Shapes / Paths / Measure sections) — good information architecture.
- Active tool indicated via `selectable_label` (highlighted selection bg).
- Shortcut keys shown on hover tooltip (`on_hover_text` with key).
- Tool Chest panel (saved style presets) is a nice pro feature, armed-preset flow.

**What could be improved**
- Top toolbar only exposes 9 of ~25 tools; the rest require expanding the left panel.
  A pro tool typically has a single dense, icon-led toolbar with overflow.
- No icons — text-only buttons ("Highlight", "Rectangle"). For a paid feel, small
  16 px glyphs + short labels reduce width and scanning time.
- Active state is the generic selection fill; no distinct "armed tool" affordance
  (e.g. filled accent chip).
- Left panel tool buttons are full-width text rows with no hover affordance beyond
  egui default; collapsed sections hide discoverability of less-used tools.
- Status hint ("Using Highlight") duplicates toolbar info; could be a contextual
  tooltip instead.

### 4. Buttons, Controls & Hierarchy

**What works well**
- Disabled-state affordances exist (e.g. Export Page to PDF disabled with
  `on_disabled_hover_text` explaining why).
- Primary actions (Save, Apply, Send) are plain buttons but placed consistently at
  the end of dialogs.
- Checkboxes/radio values used for boolean settings; `DragValue` for numeric scale.

**What could be improved**
- No visual hierarchy tokens: primary/secondary/tertiary are all default `ui.button`.
  Save/Apply should be accent-filled; Cancel should be outlined/ghost.
- No defined corner radius, padding, or height tokens (egui defaults vary). A 4 px
  radius + 6/10 px padding + 28 px control height would feel deliberate.
- Hover/focus/pressed states rely on egui defaults; focus ring is not accent-coloured.
- Icon+text vs icon-only vs text-only is inconsistent: top toolbar is text-only, menus
  text-only, dialogs text-only.

### 5. Menus, Dropdowns & Combo Boxes

**What works well**
- `ComboBox` used for units (calibration) and report grouping; `selectable_value`
  for grid spacing and auto-save interval.
- Long lists (stamps, units) handled with `MeasureUnit::ALL` iteration.

**What could be improved**
- ComboBoxes show only the selected text with a small arrow; no search/filter for
  long lists (stamp presets, units). For pro tools, a searchable combo or grouped
  menu is expected.
- Open state styling is the generic `widgets.open.bg_fill`; no elevation/shadow on
  the popup.
- Arrow-key navigation works via egui defaults but there is no visible focus
  indicator beyond the selection fill.
- Combo boxes in dialogs (Calibrate) lack labels aligned to the field.

### 6. Canvas & Markup Interaction

**What works well**
- Selection handles, resize corner/point grips, marquee selection, snap-to-grid
  indicators, measurement labels all implemented (canvas module).
- Review statuses distinguished by hue + text label (status legend in inspector
  with swatch + label + count) — colour is never the only channel.
- Empty project shows no dead pages sidebar (panel hidden when page_count == 0).
- Note popovers, in-canvas text editor, context menu (canvas + inspector share one).

**What could be improved**
- Empty canvas state has only a small hint string ("Click canvas to place shapes…")
  in the left panel; no in-canvas empty-state illustration or "Import a PDF to
  begin" guidance.
- Snap indicators and measurement labels could use a subtle halo/shadow for
  readability over busy PDFs.
- Selection handles are likely small (default egui); pro tools use 8 px square
  handles with a 1 px white border for visibility on any background.
- No visual distinction between locked vs editable markups beyond skip-on-delete;
  a lock glyph or dimmed stroke would help.

### 7. Inspector & Properties

**What works well**
- Right panel (280 px default, 240 min) hosts properties, comments, measurements,
  status legend, scale.
- Inspector modes exist (Selection / Properties / Comments / Measurements /
  Settings) as an enum, though the panel currently shows a single flow.
- Status legend with counts is a nice at-a-glance affordance.
- Calibration dialog (two-click reference segment) is well-scoped.

**What could be improved**
- The inspector is dense and flat: many fields in one scroll with only `collapsing`
  sections. A sectioned card layout (Properties, Style, Review, Measurement, Scale)
  with consistent label/value columns would feel more pro.
- Field density: labels and values on the same row via `ui.horizontal` are not
  aligned across rows; no fixed label column width.
- Colour picker is a popup with a one-frame debounce (`colour_pending`) — works but
  feels laggy; an inline swatch + popover would be smoother.
- Measurements section mixes read-only computed values with editable scale inputs
  without clear grouping.

### 8. Status Bar & Feedback

**What works well**
- Bottom status bar shows hint string, cursor position in PDF points, page count,
  document title.
- Status line (top toolbar right) shows the current operation result ("Snapshot:
  …", "Undo", etc.).
- Update check and bug report dialogs give honest states.

**What could be improved**
- Status bar is text-only and low contrast (`ui.small`). A pro status bar shows
  zoom %, scale, units, page, selection count, and a save indicator (dirty dot).
- No success/error colour coding on the status line; errors look identical to
  info.
- Autosave status is not surfaced ("Saved" / "Saving…").
- Cursor readout uses 1 decimal; could show units-aware coordinates when a scale
  is set.

### 9. Dialogs & Transient UI

**What works well**
- Dialogs are modal-ish windows: About, Unsaved Changes, Replace Unsaved Project,
  Calibrate Page Scale, Report a Bug, Export Report, Check for Updates.
- Button placement is mostly Cancel-then-Apply/Save (left-to-right), consistent.
- Disabled actions explain themselves via hover text.

**What could be improved**
- Window chrome is egui default; no consistent dialog width tokens (Calibrate is
  resizable, Export Report resizable, About fixed). Standardise on 420 px for
  simple dialogs, 560 px for forms.
- Button order in some dialogs is Cancel then primary (good, macOS-style) but
  inconsistent labels ("Apply" vs "Save" vs "Send").
- No keyboard shortcuts in dialogs (Enter to confirm, Esc to cancel) — egui
  defaults may handle Esc for windows but Enter is not wired.
- Calibration dialog uses a `DragValue` with speed 0.1; could be a numeric text
  field with unit combo for faster entry.

### 10. Icons & Visual Affordances

**What works well**
- Status colours (Open/In review/Approved/Rejected) are distinct hues with text
  labels everywhere (canvas dots, inspector legend) — satisfies "colour alone"
  rule.
- Hover tooltips on toolbar buttons and disabled actions.
- Brand mark is a simple two-shape composition that survives small sizes.

**What could be improved**
- No icons anywhere in the UI: menus, toolbar, inspector, status bar are all text.
  A minimal 16 px line-icon set (original, not copied from any product) would
  dramatically improve scan speed and the paid feel.
- The app icon is loaded from `icon_256.png`; verify packaging uses the full
  raster set and the SVG for high-DPI.
- Status legend swatches are 12×12 circles; could be 10 px rounded squares with
  a 1 px border for consistency with markup handles.

### 11. Typography, Spacing & Tokens

**What works well**
- Body text colour audited for WCAG AA (F-605): light `#202230` on `#FAFAFC`,
  dark `#E8EAF2` on indigo. Contrast test exists in `theme.rs`.
- Accent selection fill with AA-large text contrast verified.
- Hyperlink colour adjusted for dark theme (accent 2).

**What could be improved**
- No documented type scale or spacing grid in code. egui defaults (14 px body)
  are used; a deliberate scale (11/12/13/14/16/20) and 4 px spacing grid would
  tighten density.
- `ui.small` is used for secondary text with no consistent size; could be a
  `small()` helper at 11 px.
- Panel default sizes (pages 168, tools 200, tool chest 210, inspector 280,
  markups list 190) are reasonable but not derived from a token; min/max bounds
  exist and should be kept.
- No monospace token for numeric readouts (cursor coords, measurements, zoom).

### 12. Keyboard & Discoverability

**What works well**
- Core shortcuts: Ctrl+S save, Ctrl+Z undo, Ctrl+Y redo, Esc cancel, Del delete,
  Ctrl+A select all, Ctrl+O open, Ctrl+D duplicate, PageUp/PageDown.
- Tool shortcut keys exist (`Tool::shortcut_key`) and are shown in the context
  menu Tool submenu and on toolbar hover.
- Context menu is reachable from canvas and inspector (shared implementation).
- Full-screen toggle, zoom presets, grid toggles are menu-driven.

**What could be improved**
- Many major tools have no documented shortcut in the menu bar (only on hover);
  a "Keyboard Shortcuts" dialog would aid discoverability.
- Pointer-only paths remain: stamp preset selection, layer rename, colour picker,
  tool chest save/rename — all require mouse.
- No visible focus ring styling; keyboard navigation through menus works via
  egui but focus is hard to see.
- Esc cancels an in-progress draw but its availability is not surfaced; the hint
  string mentions it, which is good.

---

## PART 2 — Design proposals (prioritised by impact vs effort)

### 1. Status bar becomes a real status bar
- **Elements**: bottom status bar (`bottom_bar`), top toolbar status label, page nav.
- **Change**: Replace the hint+cursor+page-count row with a structured bar:
  document title · page X/Y · zoom % · scale (units/pt) · units · selection count ·
  dirty/saved indicator. Use monospace for numerics; muted colour for labels. Add
  success/error colour to the top status label (green for saved, red for failures).
- **Impact**: High · **Effort**: Low
- **A11y**: status text must clear AA-normal on both themes (already audited);
  colour is supplementary to a text glyph (●/✓) so not colour-only.
- **Tokens**: bar height 22 px; 4 px horizontal gaps; 11 px secondary text;
  12 px monospace numerics.

### 2. Primary/secondary/tertiary button hierarchy
- **Elements**: all `ui.button` calls in menus, dialogs, inspector.
- **Change**: Introduce three button styles: primary (accent fill, white text),
  secondary (neutral fill), tertiary (ghost/text). Apply primary to Save/Apply/Send,
  secondary to Cancel/Close, tertiary to inline actions. Add 4 px corner radius and
  28 px height.
- **Impact**: High · **Effort**: Low
- **A11y**: primary text on accent must clear AA-large (verified in theme test);
  ensure focus ring is visible (2 px accent outline on `:focus`).
- **Tokens**: radius 4 px; padding 6/10; height 28; primary fill `ACCENT`.

### 3. Icon set (original, 16 px line icons) for toolbar + menus
- **Elements**: top toolbar, left Tools panel, menu bar, status bar.
- **Change**: Add a minimal original line-icon set (16 px, single stroke weight)
  supplementing text labels. Never replace labels. Start with tool glyphs (select,
  highlight, rectangle, ellipse, line, arrow, pen, text, measure) and file actions.
- **Impact**: High · **Effort**: Medium
- **A11y**: icons must have `on_hover_text`/AccessibleName (AccessKit labelling);
  decorative icons marked as such.
- **Tokens**: 16 px glyphs; 2 px gap between icon and label; toolbar button height 30.

### 4. Active tool affordance
- **Elements**: top toolbar `selectable_label`, left panel tool buttons.
- **Change**: Make the armed tool a filled accent chip (accent fill + dark text)
  instead of the generic selection fill; add a 2 px left accent bar on the left
  panel's active row. Show the shortcut key inline (right-aligned, muted) rather
  than only on hover.
- **Impact**: Medium · **Effort**: Low
- **A11y**: active state conveyed by fill + label weight + check glyph, not colour
  alone.
- **Tokens**: chip radius 4 px; 4 px vertical padding.

### 5. Inspector sectioned card layout
- **Elements**: right panel inspector (properties/comments/measurements/scale).
- **Change**: Group fields into titled sections (Properties, Style, Review,
  Measurement, Scale) using a consistent 2-column label/value grid with a fixed
  96 px label column; 8 px section gaps; collapsible headers. Align colour swatch
  inline with its label.
- **Impact**: High · **Effort**: Medium
- **A11y**: section headers as headings for screen readers; labels associated with
  controls via AccessKit.
- **Tokens**: label column 96 px; row height 22; section gap 12; card padding 8.

### 6. In-canvas empty state
- **Elements**: central canvas when no PDF imported / no markups.
- **Change**: Show a centred empty-state block: "Import a PDF to begin" with a
  primary "Import PDF…" button and a muted hint listing supported actions. Keep
  the pages sidebar hidden (already). Avoid dead whitespace.
- **Impact**: Medium · **Effort**: Low
- **A11y**: button keyboard-focusable; text contrast AA-normal.
- **Tokens**: 16 px heading, 12 px body, 24 px vertical spacing.

### 7. Selection handles & locked-markup affordance
- **Elements**: canvas selection handles, resize grips, markup strokes.
- **Change**: Enlarge handles to 8 px squares with 1 px white border; add a
  small lock glyph (or 40% opacity) for locked markups; add a subtle drop shadow
  to measurement labels for readability over busy pages.
- **Impact**: Medium · **Effort**: Medium
- **A11y**: handle size ≥ 8 px improves motor accessibility; colour not the only
  signal (shape + opacity).
- **Tokens**: handle 8 px; border 1 px; shadow blur 2 px.

### 8. Searchable combo boxes for long lists
- **Elements**: unit combo (calibration), stamp preset selection, layer filter.
- **Change**: Replace plain `ComboBox` with a searchable popover (filter-as-you-type)
  for lists > 8 items; group units by family (length/area). Add elevation + 8 px
  radius on the popup.
- **Impact**: Medium · **Effort**: Medium
- **A11y**: arrow-key navigation + Enter to select + Esc to close; focus trap
  within popover.
- **Tokens**: popup radius 8; max height 240; item height 24.

### 9. Dialog standardisation (widths, keyboard, button labels)
- **Elements**: About, Calibrate, Export Report, Bug Report, Unsaved/Replace.
- **Change**: Standardise widths (simple 420, form 560); wire Enter=confirm and
  Esc=cancel; align labels; use consistent primary/secondary button styles from
  proposal 2; rename "Apply"→"Calibrate" in the calibration dialog for clarity.
- **Impact**: Medium · **Effort**: Low
- **A11y**: modal focus management; `aria-labelledby` via window title.
- **Tokens**: widths 420/560; button gap 8.

### 10. Keyboard shortcuts dialog + menu accelerator completeness
- **Elements**: Help menu, menu bar items, toolbar tooltips.
- **Change**: Add "Keyboard Shortcuts…" dialog listing all shortcuts grouped by
  category (File, Edit, View, Tools, Canvas). Add missing shortcut hints to menu
  items (Zoom In/Out, Fit Page, page nav, grid toggles). Ensure every major tool
  is reachable via a single key and documented.
- **Impact**: Medium · **Effort**: Low
- **A11y**: dialog fully keyboard-navigable; printable list for reference.
- **Tokens**: dialog width 560; two-column layout.

### 11. Typography & spacing token pass
- **Elements**: all UI text.
- **Change**: Define a type scale (11 secondary, 12 body-small, 13 body, 14
  subheading, 16 heading, 20 title) and a 4 px spacing grid; expose as constants
  in `theme.rs`. Apply monospace token to numeric readouts. Audit every `ui.small`
  to the 11 px token.
- **Impact**: Medium · **Effort**: Medium
- **A11y**: maintain AA contrast for all sizes; 11 px text only on high-contrast
  pairs.
- **Tokens**: scale 11/12/13/14/16/20; spacing 4/8/12/16/24.

### 12. Theme-aware focus rings & hover states
- **Elements**: all interactive controls.
- **Change**: Set `visuals.widgets.*.rounding` to 4 px; define a 2 px accent focus
  stroke; ensure hovered/active/open fills are distinct but subtle (already
  close). Add a faint elevation (1 px border) to panels.
- **Impact**: Low · **Effort**: Low
- **A11y**: focus visible in both themes; never rely on colour alone (shape).
- **Tokens**: rounding 4; focus stroke 2 px accent.

---

## Notes for the domain-dev agent

- No domain changes are required for these proposals; all are UI-layer. However,
  exposing a "locked" flag glyph and a units-aware cursor coordinate may require
  small domain helpers (e.g. `Markup::is_locked()` already exists via
  `is_markup_editable`; cursor unit conversion needs `Scale` + `MeasureUnit`
  conversion already in `domain::units`).
- Icon assets must be original SVGs authored for Rubric (see `docs/design/brand.md`
  usage rules); do not import third-party icon sets.
- Contrast audit tests in `theme.rs` should be extended to cover any new
  text/background pairs introduced (status bar success/error colours, focus rings).