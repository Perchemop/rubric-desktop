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
`CommandCategory::ALL` include an eighth row, `Advanced`. This document preserves
the registry and source state rather than silently resolving that conflict.

| Field | Meaning |
|---|---|
| ID | The key used in `command_registry()` and toolbar entries. |
| Name | The `CommandInfo::name` string intended for display. |
| Icon | The `CommandInfo::icon` key from the `icon` module. |
| Shortcut | The `CommandInfo::shortcut` string recorded in source. An empty cell means the source stores `""`. |
| Category | The `CommandCategory::label()` string recorded in source. |
| Description | The `CommandInfo::description` string recorded in source. |

Shortcut strings are metadata. Duplicate strings are retained exactly as
registered; a value here does not prove an active keybinding in the current
build. Four strings contain a trailing space in source: `Ctrl+Shift+] `,
`Ctrl+Shift+[ `, `Ctrl+] `, and `Ctrl+[ `. `Pan` and `Highlight` both register
`H`.

## 1. Selection & Navigation

| ID | Name | Icon | Shortcut | Category | Description |
|---|---|---|---|---|---|
| `sel` | Select | `sel` | `V` | Selection & Navigation | Select, move and resize markups |
| `pan` | Pan | `pan` | `H` | Selection & Navigation | Pan the canvas |
| `zi` | Zoom In | `zi` | `+` | Selection & Navigation | Zoom in on the canvas |
| `zo` | Zoom Out | `zo` | `-` | Selection & Navigation | Zoom out on the canvas |
| `zf` | Zoom Fit | `zf` | `F` | Selection & Navigation | Fit page to canvas |
| `pf` | First Page | `pf` | `Ctrl+Home` | Selection & Navigation | Go to first page |
| `pp` | Previous Page | `pp` | `Page Up` | Selection & Navigation | Go to previous page |
| `pn` | Next Page | `pn` | `Page Down` | Selection & Navigation | Go to next page |
| `pl` | Last Page | `pl` | `Ctrl+End` | Selection & Navigation | Go to last page |
| `jmp` | Jump to Page | `jmp` | `Ctrl+G` | Selection & Navigation | Jump to a specific page |

## 2. Markup Tools

| ID | Name | Icon | Shortcut | Category | Description |
|---|---|---|---|---|---|
| `hl` | Highlight | `hl` | `H` | Markup Tools | Draw highlight markup |
| `sk` | Strikeout | `sk` | `S` | Markup Tools | Draw strikeout markup |
| `ul` | Underline | `ul` | `U` | Markup Tools | Draw underline markup |
| `sq` | Squiggly | `sq` | `Q` | Markup Tools | Draw squiggly underline markup |
| `txt` | Text Box | `txt` | `T` | Markup Tools | Draw a text box |
| `co` | Callout | `co` | `C` | Markup Tools | Draw a callout box |
| `note` | Sticky Note | `note` | `N` | Markup Tools | Add a sticky note |
| `stamp` | Stamp Presets | `stamp` | `M` | Markup Tools | Apply stamp presets |
| `rect` | Rectangle | `rect` | `R` | Markup Tools | Draw a rectangle |
| `ell` | Ellipse | `ell` | `O` | Markup Tools | Draw an ellipse |
| `cld` | Cloud | `cld` | `D` | Markup Tools | Draw a cloud callout |
| `pline` | Polyline | `pline` | `Y` | Markup Tools | Draw a polyline |
| `poly` | Polygon | `poly` | `P` | Markup Tools | Draw a polygon |
| `line` | Line | `line` | `L` | Markup Tools | Draw a line |
| `arrow` | Arrow | `arrow` | `A` | Markup Tools | Draw an arrow |
| `ink` | Pen | `ink` | `I` | Markup Tools | Freehand ink drawing |

Source note: some comments and tests describe Markup Tools as 15 commands, but
`commands_by_category()` returns 16 entries, including `stamp` / Stamp Presets.

## 3. Measurement Tools

| ID | Name | Icon | Shortcut | Category | Description |
|---|---|---|---|---|---|
| `ln` | Length | `ln` | `1` | Measurement Tools | Measure length |
| `perim` | Perimeter | `perim` | `2` | Measurement Tools | Measure perimeter |
| `ar` | Area | `ar` | `3` | Measurement Tools | Measure area |
| `ang` | Angle | `ang` | `4` | Measurement Tools | Measure angle |
| `dia` | Diameter | `dia` | `5` | Measurement Tools | Measure diameter |
| `rad` | Radius | `rad` | `6` | Measurement Tools | Measure radius |
| `cnt` | Count | `cnt` | `7` | Measurement Tools | Count markups |
| `cal` | Calibrate Scale | `cal` | `K` | Measurement Tools | Calibrate page scale with known distance |
| `unit` | Unit Selector | `unit` |  | Measurement Tools | Change measurement units |

## 4. Review & Comment

| ID | Name | Icon | Shortcut | Category | Description |
|---|---|---|---|---|---|
| `status` | Status Dropdown | `status` |  | Review & Comment | Set markup review status |
| `add_cm` | Add Comment | `add_cm` |  | Review & Comment | Add a comment to selection |
| `show_cm` | Show Comments | `show_cm` |  | Review & Comment | Toggle marks list visibility |
| `author` | Author Filter | `author` |  | Review & Comment | Filter markups by author |
| `report` | Summary Report | `report` |  | Review & Comment | Generate markup summary report |
| `review` | Review Status | `review` |  | Review & Comment | Show review status filter |

## 5. Edit & Modify

| ID | Name | Icon | Shortcut | Category | Description |
|---|---|---|---|---|---|
| `copy_prop` | Copy Properties | `copy_prop` | `Ctrl+Shift+C` | Edit & Modify | Copy markup properties |
| `paste_prop` | Paste Properties | `paste_prop` | `Ctrl+Shift+V` | Edit & Modify | Paste markup properties |
| `bring_fwd` | Bring Forward | `bring_fwd` | `Ctrl+Shift+] ` | Edit & Modify | Bring selected markups forward |
| `send_back` | Send Backward | `send_back` | `Ctrl+Shift+[ ` | Edit & Modify | Send selected markups backward |
| `bring_front` | Bring to Front | `bring_front` | `Ctrl+] ` | Edit & Modify | Bring selected markups to front |
| `send_back_bt` | Send to Back | `send_back_bt` | `Ctrl+[ ` | Edit & Modify | Send selected markups to back |
| `toggle_lock` | Toggle Lock | `toggle_lock` | `Ctrl+L` | Edit & Modify | Lock or unlock selected markups |
| `layer` | Layer Visibility | `layer` |  | Edit & Modify | Toggle layer visibility |

## 6. View & Display

| ID | Name | Icon | Shortcut | Category | Description |
|---|---|---|---|---|---|
| `theme` | Theme Toggle | `theme` |  | View & Display | Toggle between light and dark theme |
| `grid` | Show/Hide Grid | `grid` |  | View & Display | Toggle grid visibility |
| `snap` | Snap to Grid | `snap` |  | View & Display | Toggle snapping to grid |
| `grid_spc` | Grid Spacing | `grid_spc` |  | View & Display | Adjust grid spacing |
| `ru` | Show/Hide Rulers | `ru` | `Ctrl+R` | View & Display | Toggle rulers visibility |
| `fs` | Full Screen | `fs` | `F11` | View & Display | Toggle full screen mode |
| `panel` | Panel Visibility | `panel` |  | View & Display | Show or hide side panels |

## 7. File & Project

| ID | Name | Icon | Shortcut | Category | Description |
|---|---|---|---|---|---|
| `new` | New | `new` | `Ctrl+N` | File & Project | Create a new project |
| `open` | Open | `open` | `Ctrl+O` | File & Project | Open an existing project |
| `save` | Save | `save` | `Ctrl+S` | File & Project | Save the current project |
| `save_as` | Save As | `save_as` | `Ctrl+Shift+S` | File & Project | Save the current project as a new file |
| `import` | Import PDF | `import` |  | File & Project | Import a PDF file |
| `export` | Export Image | `export` | `Ctrl+E` | File & Project | Export markups as image |
| `print` | Print | `print` | `Ctrl+P` | File & Project | Print the current view |
| `recent` | Recent Files | `recent` |  | File & Project | Open a recently used file |
| `settings` | Project Settings | `settings` |  | File & Project | Configure project settings |

## 8. Advanced

| ID | Name | Icon | Shortcut | Category | Description |
|---|---|---|---|---|---|
| `bates` | Bates Numbering Config | `bates` |  | Advanced | Configure Bates numbering for markups |
| `cal_dlg` | Calibration Dialog | `cal_dlg` |  | Advanced | Open calibration dialog |
| `prefs` | Preferences | `prefs` | `Ctrl+,` | Advanced | Open preferences dialog |
| `kb` | Keyboard Shortcuts | `kb` | `Ctrl+K` | Advanced | Show keyboard shortcuts reference |
| `profiles` | Profiles Management | `profiles` |  | Advanced | Manage user profiles |

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
