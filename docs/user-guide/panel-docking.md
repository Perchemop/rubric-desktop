# Panel Docking and Splitting (planned F-607 — draft)

> **Status:** This guide describes the intended behaviour from the
> [`F-607 panel docking and splitting specification`](../spec/panel-docking-splitting.spec.md).
> The specification is `Draft — Pending Review`. This is design documentation,
> not a claim that edge docking, split panes, floating panels, panel shortcuts,
> or a Panel Access Bar are shipped in the current release.

Rubric panels are dockable views such as Thumbnails, Layers, Tool Chest,
Markups List, Properties, Stamps, and Measurements. The intended design lets a
user keep frequently used views visible at the edge of the canvas, split a
dock so two panels share the same space, group panels as tabs, detach panels
into floating windows or overlays, and restore a complete arrangement from a
predefined profile layout.

All examples below are planned user workflows. The source currently contains
the `PanelLayout` data model and shortcut helpers; availability in the product
depends on the unfinished implementation checklist in the specification.

## The panel areas

| Area | Role | Typical panels |
|---|---|---|
| Left dock | Page navigation and document structure. | Thumbnails, Layers, Tool Chest |
| Right dock | Context for the current selection or markup. | Properties, Stamps, Measurements |
| Bottom dock | Longer lists that benefit from a wide strip. | Markups List |
| Canvas | The PDF page workspace. Panels never become page content. | N/A |
| Panel Access Bar | Compact strip for opening or restoring hidden panels. | Configurable icon set |

Panel IDs are stable strings. The source inventory currently defines
`thumbnails`, `layers`, `tool_chest`, `markups_list`, `properties`, `stamps`,
and `measurements`. The Review layout also uses `bookmarks`, which appears as
`PanelId::Other("bookmarks")` and is not part of that seven-panel inventory.
This discrepancy is preserved here rather than resolved by inventing a new
inventory entry.

## Edge docking

A panel can be attached to the left, right, or bottom edge of the workspace.

1. Drag a panel tab toward an edge. A dock target highlights the strip that
   will receive it.
2. Drop on the left or right edge to create a vertical strip. Drop on the
   bottom edge to create a horizontal strip.
3. Drop onto an existing tab bar to add the panel to that bar as another tab.
4. Drag a tab out of the dock to return it to a floating state.

The visible arrangement is stored in `PanelLayout.left`, `PanelLayout.right`,
and `PanelLayout.bottom`. Each dock keeps its own tab order. Panel values in a
dock also carry `visible`, `dock_location`, and an optional `split_tree`.

### Resizing dock strips

Drag the divider between a dock and the canvas to change its size. The source
defaults are:

| Strip | Default width or height |
|---|---:|
| Left | `200.0` |
| Right | `260.0` |
| Bottom | `200.0` |

The intended UI rejects a drag below the minimum panel size with the message
`Minimum size reached`.

> **Animated GIF placeholder — edge docking:** drag a panel from the right edge
> into the bottom dock, then resize the strip. Replace with a real screen
> recording when F-607 is implemented and reviewed.

## Split wheel

The split wheel is the planned control for dividing one dock into two panes.

1. Hover over a panel header to reveal the split control.
2. Select left, right, top, or bottom to choose the split orientation.
3. Drag the divider to set the ratio.
4. Repeat a split inside a child pane to build a nested arrangement.

A split is represented by `SplitTree::Split { orientation, ratio, children }`.
The ratio is clamped between `0.05` and `0.95`, so no child can be collapsed
entirely through a ratio alone. `SplitTree::Leaf(PanelId)` represents the panel
at the end of a branch. The intended maximum split depth is four; attempting a
deeper split reports `Maximum split depth (4) reached`.

```rust
// Source-shaped example, not an applied setting.
DockedPanel {
    id: PanelId::Layers,
    visible: true,
    dock_location: DockLocation::Left,
    split_tree: Some(SplitTree::Split {
        orientation: SplitOrientation::Vertical,
        ratio: 0.55,
        children: [
            SplitTree::Leaf(PanelId::Thumbnails),
            SplitTree::Leaf(PanelId::Layers),
        ],
    }),
    tab_order: vec![PanelId::Layers],
    floating: None,
}
```

> **Animated GIF placeholder — split wheel:** open the split wheel, choose a
> direction, move the divider, and then nest one more split. Replace with a real
> screen recording after implementation.

## Tab groups

Several panels in the same dock share a tab bar. Selecting a tab brings that
panel to the front while the others remain open. Dragging a tab to another tab
bar moves it there; dragging it to an empty edge creates a new dock.

`DockedPanel.tab_order` records the order of panels in a group. The default for
a newly constructed `DockedPanel` is `[id]`, meaning that the panel is the only
tab until the user adds more.

Closing a tab hides that panel. The layout keeps its identity so a custom
shortcut or the Panel Access Bar can restore it without rebuilding the panel
from scratch.

> **Animated GIF placeholder — tab groups:** drag Markups List beside
> Properties, switch tabs, then close and restore one tab. Replace with a real
> screen recording after implementation.

## Floating panels

A panel can be detached from a dock. `FloatingPanel` records:

| Field | Meaning | Default in source |
|---|---|---|
| `panel_id` | The panel being floated. | Required |
| `position` | `[x, y]` position. | `[100.0, 100.0]` |
| `size` | `[width, height]`. | `[400.0, 300.0]` |
| `mode` | Floating presentation mode. | `windowed` |
| `monitor` | Optional monitor identifier. | `None` |

The source `FloatingMode` variants are `Windowed` and `Overlay`. A windowed
panel behaves as a separate floating window. An overlay stays above the canvas
without owning a full window. The F-607 specification also describes a
presentation workflow with a review panel on a second monitor; that workflow is
not a third `FloatingMode` variant in the current source.

Presentation floating panels are stored on the profile, not inside
`PanelLayout`. The `predefined_layouts::presentation()` function returns a clean
primary `PanelLayout` and leaves the floating panel to the profile layer. See
[`profile-schema.md`](../profile-schema.md) for the profile surface and
[`panel-layout-schema.md`](../panel-layout-schema.md) for the dock layout
contract. If a saved monitor is unavailable, the intended message is
`Saved monitor not found. Panel placed on primary monitor.`

> **Animated GIF placeholder — floating panels:** detach Properties, move and
> resize it, then place a Markups List overlay on a second monitor. Replace with
> a real screen recording after implementation.

## Panel Access Bar

The Panel Access Bar is a compact strip of icons for panels that are hidden or
floating. Its state is stored in `PanelLayout.access_bar`:

| Source field | Meaning | Default |
|---|---|---|
| `auto_hide` | Whether the bar hides until requested. | `false` |
| `visible_icons` | Panel IDs shown as icons. | Empty list |

A user can pin a frequently used panel to the bar, then click its icon to dock
or reveal it. When the bar is set to auto-hide, it appears near the relevant
edge and retreats after the panel is opened. The exact gesture and hover region
remain subject to specification review.

> **Animated GIF placeholder — Panel Access Bar:** pin Layers to the access
> bar, hide it, then restore it with one click. Replace with a real screen
> recording after implementation.

## Keyboard shortcuts

`PanelId::default_shortcut()` in the source maps the seven inventory panels as
follows:

| Shortcut | Panel | Notes |
|---|---|---|
| `Ctrl+1` | Thumbnails | Default mapping in `PanelId`. |
| `Ctrl+2` | Layers | Default mapping in `PanelId`. |
| `Ctrl+3` | Tool Chest | Default mapping in `PanelId`. |
| `Ctrl+4` | Markups List | Default mapping in `PanelId`. |
| `Ctrl+5` | Properties | Default mapping in `PanelId`. |
| `Ctrl+6` | Stamps | Default mapping in `PanelId`. |
| `Ctrl+7` | Measurements | Default mapping in `PanelId`. |

`panel_shortcuts.rs` is another source of truth for shortcut handling. Its
`ctrl_digit()` helper accepts digits `1` through `7`, but
`PanelShortcutRegistry::default()` currently registers only `Ctrl+1` through
`Ctrl+6`, mapped to Pages, Tools, Tool Chest, Inspector, Markups List, and
Quantities, and does not register Stamps. Names and coverage therefore differ
between these two source locations. This guide preserves both instead of
silently choosing one.

The intended custom-shortcut editor allows reassignment with a conflict check.
The planned error text is
`Shortcut Ctrl+X already used by Y. Change one.` Invalid profile-level panel
layout data is reported as `Panel layout in profile 'X' invalid. Using Default.`

## Predefined layouts

The predefined profile code currently exposes four layouts:

| Profile | Left dock | Right dock | Bottom dock | Floating |
|---|---|---|---|---|
| Default | Thumbnails, Layers | Properties, Stamps, Measurements | Markups List | None |
| Review | Thumbnails, Bookmarks | Properties, Markups List | Not listed | None |
| Measurement | Layers, Tool Chest | Properties, Measurements | Not listed | None |
| Markup | Tool Chest, Stamps | Properties, Layers | Not listed | None |
| Minimal | All hidden by default | All hidden by default | All hidden by default | None |
| Presentation | Clean primary arrangement | Clean primary arrangement | Clean primary arrangement | Secondary Markups List overlay |

There is a source/specification mismatch to keep visible: the specification
describes a Presentation profile layout, while `predefined_profiles()` currently
returns only Default, Review, Measurement, and Markup. Minimal is present in
`panel.rs` predefined layouts but is not yet one of the four profiles returned
by the profile factory. A user should not assume that every row above is
selectable in the current build.

Selecting a predefined layout is intended to replace the current arrangement,
not merge with it. Save a custom profile first if an arrangement should be kept
for later. Profile behavior and file format are documented separately in
[`profile-schema.md`](../profile-schema.md).

> **Animated GIF placeholder — predefined layout picker:** switch from Default
> to Review, then back to a saved custom layout. Replace with a real screen
> recording after implementation.

## Saving and restoring a layout

`PanelLayout` includes `schema_version: 1` (`PANEL_SCHEMA_VERSION = 1`). The
complete field list and JSON Schema are in
[`panel-layout-schema.md`](../panel-layout-schema.md). A profile embeds a
`PanelLayout` as its `panelLayout` value, so saving a profile is the intended
way to persist an arrangement across sessions.

The default empty layout has no docked panels, `access_bar.auto_hide = false`,
and an empty `visible_icons` list. The predefined layouts supply the common
starting arrangements listed above.

When a saved layout cannot be applied, the intended behavior is to fall back to
the Default layout and report the problem without discarding the user's profile
file. That report is a design intent, not a shipped guarantee.

## Status and open review points

- The feature specification remains `Draft — Pending Review`.
- The seven-panel inventory does not include `bookmarks`, although Review uses
  it.
- `PanelId::default_shortcut()` and `PanelShortcutRegistry::default()` disagree
  on names and on `Ctrl+7` / Stamps coverage.
- The specification mentions Presentation and Minimal arrangements that are not
  both exposed through `predefined_profiles()`.
- Floating presentation panels live on `Profile`; they are not part of the
  `PanelLayout` JSON object.
- No implementation, test pass, or release claim is made by this document.

Documentation was drafted on 2026-11-25 from the F-607 specification and the
current domain source.
