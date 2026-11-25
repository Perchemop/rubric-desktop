# Toolbar Command Reference (planned toolbar dialog — draft documentation)

> **Status:** F-608 is `Draft — Pending Review`. This reference records the
> command registry currently present in
> [`apps/rubric/src/domain/commands.rs`](../apps/rubric/src/domain/commands.rs).
> It is source-of-truth metadata for the planned Toolbar Customization Dialog,
> not proof that the dialog, drag-and-drop editing, or these keybindings are
> shipped. See the [F-608 specification](spec/toolbar-customization.spec.md) and
> the [planned toolbar guide](user-guide/toolbar-customization.md).

The registry currently defines **70 commands** across **eight categories**. The
F-608 specification says “7 command categories,” but its own category table and
`CommandCategory::ALL` include a eighth row, `Advanced`. This document preserves
the registry and source state rather than silently resolving that conflict.

| Field | Meaning |
|---|---|
| ID | The key used in `command_registry()` and toolbar entries. |
| Name | The `CommandInfo::name` string intended for display. |
| Icon | The `CommandInfo::icon` key from the `icon` module. |
| Shortcut | The `CommandInfo::shortcut` string recorded in source. An empty cell means the source stores `""`. |
| Description | The `CommandInfo::description` string recorded in source. |

Shortcut strings are metadata. Duplicate strings are retained exactly as
registered; a value here does not prove an active keybinding in the current
build. Four strings contain a trailing space in source: `Ctrl+Shift+] `,
`Ctrl+Shift+[ `, `Ctrl+] `, and `Ctrl+[ `. `Pan` and `Highlight` both register
`H`.

## 1. Selection & Navigation

| ID | Name | Icon | Shortcut | Description |
|---|---|---|---|---|
| `sel` | Select | `sel` | `V` | Select, move and resize markups |
| `pan` | Pan | `pan` | `H` | Pan the canvas |
| `zi` | Zoom In | `zi` | `+` | Zoom in on the canvas |
| `zo` | Zoom Out | `zo` | `-` | Zoom out on the canvas |
| `zf` | Zoom Fit | `zf` | `F` | Fit page to canvas |
| `pf` | First Page | `pf` | `Ctrl+Home` | Go to first page |
| `pp` | Previous Page | `pp` | `Page Up` | Go to previous page |
| `pn` | Next Page | `pn` | `Page Down` | Go to next page |
| `pl` | Last Page | `pl` | `Ctrl+End` | Go to last page |
| `jmp` | Jump to Page | `jmp` | `Ctrl+G` | Jump to a specific page |

## 2. Markup Tools

| ID | Name | Icon | Shortcut | Description |
|---|---|---|---|---|
| `hl` | Highlight | `hl` | `H` | Draw highlight markup |
| `sk` | Strikeout | `sk` | `S` | Draw strikeout markup |
| `ul` | Underline | `ul` | `U` | Draw underline markup |
| `sq` | Squiggly | `sq` | `Q` | Draw squiggly underline markup |
| `txt` | Text Box | `txt` | `T` | Draw a text box |
| `co` | Callout | `co` | `C` | Draw a callout box |
| `note` | Sticky Note | `note` | `N` | Add a sticky note |
| `stamp` | Stamp Presets | `stamp` | `M` | Apply stamp presets |
| `rect` | Rectangle | `rect` | `R` | Draw a rectangle |
| `ell` | Ellipse | `ell` | `O` | Draw an ellipse |
| `cld` | Cloud | `cld` | `D` | Draw a cloud callout |
| `pline` | Polyline | `pline` | `Y` | Draw a polyline |
| `poly` | Polygon | `poly` | `P` | Draw a polygon |
| `line` | Line | `line` | `L` | Draw a line |
| `arrow` | Arrow | `arrow` | `A` | Draw an arrow |
| `ink` | Pen | `ink` | `I` | Freehand ink drawing |

Source note: some comments and tests describe Markup Tools as 15 commands, but
`commands_by_category()` returns 16 entries, including `stamp` / Stamp Presets.

## 3. Measurement Tools

| ID | Name | Icon | Shortcut | Description |
|---|---|---|---|---|
| `ln` | Length | `ln` | `1` | Measure length |
| `perim` | Perimeter | `perim` | `2` | Measure perimeter |
| `ar` | Area | `ar` | `3` | Measure area |
| `ang` | Angle | `ang` | `4` | Measure angle |
| `dia` | Diameter | `dia` | `5` | Measure diameter |
| `rad` | Radius | `rad` | `6` | Measure radius |
| `cnt` | Count | `cnt` | `7` | Count markups |
| `cal` | Calibrate Scale | `cal` | `K` | Calibrate page scale with known distance |
| `unit` | Unit Selector | `unit` |  | Change measurement units |

## 4. Review & Comment

| ID | Name | Icon | Shortcut | Description |
|---|---|---|---|---|
| `status` | Status Dropdown | `status` |  | Set markup review status |
| `add_cm` | Add Comment | `add_cm` |  | Add a comment to selection |
| `show_cm` | Show Comments | `show_cm` |  | Toggle marks list visibility |
| `author` | Author Filter | `author` |  | Filter markups by author |
| `report` | Summary Report | `report` |  | Generate markup summary report |
| `review` | Review Status | `review` |  | Show review status filter |

## 5. Edit & Modify

| ID | Name | Icon | Shortcut | Description |
|---|---|---|---|---|
| `copy_prop` | Copy Properties | `copy_prop` | `Ctrl+Shift+C` | Copy markup properties |
| `paste_prop` | Paste Properties | `paste_prop` | `Ctrl+Shift+V` | Paste markup properties |
| `bring_fwd` | Bring Forward | `bring_fwd` | `Ctrl+Shift+] ` | Bring selected markups forward |
| `send_back` | Send Backward | `send_back` | `Ctrl+Shift+[ ` | Send selected markups backward |
| `bring_front` | Bring to Front | `bring_front` | `Ctrl+] ` | Bring selected markups to front |
| `send_back_bt` | Send to Back | `send_back_bt` | `Ctrl+[ ` | Send selected markups to back |
| `toggle_lock` | Toggle Lock | `toggle_lock` | `Ctrl+L` | Lock or unlock selected markups |
| `layer` | Layer Visibility | `layer` |  | Toggle layer visibility |

## 6. View & Display

| ID | Name | Icon | Shortcut | Description |
|---|---|---|---|---|
| `theme` | Theme Toggle | `theme` |  | Toggle between light and dark theme |
| `grid` | Show/Hide Grid | `grid` |  | Toggle grid visibility |
| `snap` | Snap to Grid | `snap` |  | Toggle snapping to grid |
| `grid_spc` | Grid Spacing | `grid_spc` |  | Adjust grid spacing |
| `ru` | Show/Hide Rulers | `ru` | `Ctrl+R` | Toggle rulers visibility |
| `fs` | Full Screen | `fs` | `F11` | Toggle full screen mode |
| `panel` | Panel Visibility | `panel` |  | Show or hide side panels |

## 7. File & Project

| ID | Name | Icon | Shortcut | Description |
|---|---|---|---|---|
| `new` | New | `new` | `Ctrl+N` | Create a new project |
| `open` | Open | `open` | `Ctrl+O` | Open an existing project |
| `save` | Save | `save` | `Ctrl+S` | Save the current project |
| `save_as` | Save As | `save_as` | `Ctrl+Shift+S` | Save the current project as a new file |
| `import` | Import PDF | `import` |  | Import a PDF file |
| `export` | Export Image | `export` | `Ctrl+E` | Export markups as image |
| `print` | Print | `print` | `Ctrl+P` | Print the current view |
| `recent` | Recent Files | `recent` |  | Open a recently used file |
| `settings` | Project Settings | `settings` |  | Configure project settings |

## 8. Advanced

| ID | Name | Icon | Shortcut | Description |
|---|---|---|---|---|
| `bates` | Bates Numbering Config | `bates` |  | Configure Bates numbering for markups |
| `cal_dlg` | Calibration Dialog | `cal_dlg` |  | Open calibration dialog |
| `prefs` | Preferences | `prefs` | `Ctrl+,` | Open preferences dialog |
| `kb` | Keyboard Shortcuts | `kb` | `Ctrl+K` | Show keyboard shortcuts reference |
| `profiles` | Profiles Management | `profiles` |  | Manage user profiles |

## Source differences preserved

- Tests query `select`, `highlight`, and `length`, while the registry keys are
  `sel`, `hl`, and `ln`. The UI fallback list in
  [`apps/rubric/src/ui/toolbar_customize_dialog.rs`](../apps/rubric/src/ui/toolbar_customize_dialog.rs)
  uses another subset of IDs. This reference follows `commands.rs`.
- `CommandId` values equal the corresponding icon keys in the current registry.
  That equality is observed source state, not a guaranteed naming rule.
- `command_count()` returns `70`; `get_all_command_ids()` also returns 70 IDs.
- Empty shortcut cells are intentional because the source stores empty strings
  for those commands.

Documentation was drafted on 2026-11-25. It does not mark F-608 implemented or
shipped.
