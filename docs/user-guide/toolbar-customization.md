# Toolbar Customization (planned feature — draft documentation)

> **Status:** F-608 is `Draft — Pending Review`. This guide describes the planned
> Toolbar Customization Dialog from
> [`docs/spec/toolbar-customization.spec.md`](../spec/toolbar-customization.spec.md).
> The dialog, custom toolbar runtime, per-profile toolbar persistence, and
> `Ctrl+Shift+T` shortcut are **not shipped** in the current release. Do not
> follow these steps as current product instructions.

Planned references:

- [Toolbar command reference](../toolbar-command-reference.md)
- [Profiles (planned feature)](profiles.md)
- [F-608 specification](../spec/toolbar-customization.spec.md)

Toolbar customization is intended to let a reviewer keep status and comment
tools close, an estimator keep measurement tools close, and a markup author
keep stamp and shape tools close. The design is a four-pane dialog with a live
preview: changes made in the dialog should appear in the main window
immediately.

> **Screenshot placeholder:** four-pane Customize Toolbars dialog with a
> selected category, commands list, toolbar list, and ordered item list.
> Replace with reviewed evidence after the dialog is implemented and CI or
> hands-on verification is recorded.

## 1. Opening and navigating the dialog

The planned entry points are **Tools > Toolbars > Customize...** and
`Ctrl+Shift+T`.

| Input | Planned result |
|---|---|
| `Ctrl+Shift+T` | Open the modal dialog with Categories focused. |
| `Tab` | Move between Categories, Commands, Toolbars, and Items. |
| Arrow keys | Move within the focused pane. |
| `Enter` | Activate or select the highlighted item. |
| `Delete` | Remove the selected item from the current toolbar. |
| `Ctrl+F` | Filter the Commands pane by command name. |
| `Escape` | Close the dialog. |

The dialog is intended to remember the last selected category and toolbar for
the current session.

```text
+-------------------------------------------------------------------+
| Customize Toolbars                                    [Reset] [X] |
+-------------------+-------------------+----------------+----------+
| Categories        | Commands          | Toolbars       | Items    |
| [x] Selection     | [icon] Select     | Main Toolbar   | Select   |
| [x] Markup        | [icon] Pan        | Quick Access   | Highlight|
| [ ] Measurement   | [icon] Zoom In    | Properties     | Length   |
| ...               | ...               | Custom         | [divider]|
+-------------------+-------------------+----------------+----------+
```

## 2. Pane behaviour

| Pane | Planned contents | Notes |
|---|---|---|
| Categories | Checkable category filters. | Checked categories filter what appears in Commands. |
| Commands | Icon, name, and shortcut for each available command, plus a Divider pseudo-command. | Commands are draggable into Items. `Ctrl+F` filters this list. |
| Toolbars | Main Toolbar, Quick Access Area, Properties Toolbar, and custom toolbars. | Show/hide checkbox and lock state per row; New, Rename, and Delete act on custom toolbars. |
| Items | Ordered tools in the selected toolbar. | Drag to reorder; `Delete` removes a tool or divider. |

The specification says “7 command categories,” but it lists eight. The current
domain source also exposes eight categories in `CommandCategory::ALL`. The
planned Categories pane should be treated as eight rows until review resolves
the conflict:

1. Selection & Navigation
2. Markup Tools
3. Measurement Tools
4. Review & Comment
5. Edit & Modify
6. View & Display
7. File & Project
8. Advanced

See the [command reference](../toolbar-command-reference.md) for the exact IDs,
names, shortcuts, and descriptions currently present in the command registry.

## 3. Adding, removing, and reordering tools

Planned workflow:

1. Open **Tools > Toolbars > Customize...** or press `Ctrl+Shift+T`.
2. Select the destination toolbar in the Toolbars pane.
3. Check one or more categories in Categories.
4. Drag a command from Commands into the Items pane at the desired position.
5. Drag items inside Items to reorder them.
6. Select an item and press `Delete`, or drag it back to Commands, to remove it.
7. Drag the Divider pseudo-command into Items to create a vertical separator.
8. Leave the dialog to apply the live preview to the main UI.

| Action | Planned result |
|---|---|
| Drag Commands → Items | Add the tool at the drop position. |
| Drag within Items | Reorder tools or dividers. |
| `Delete` in Items | Remove the selected tool or divider. |
| Drag Items → Commands | Remove the selected tool from the toolbar. |
| Drop on an invalid target | No change; the tool returns to its source. |

> **Screenshot placeholder:** Commands pane with a tool being dragged into the
> Main Toolbar Items list, showing the insertion marker and live preview.

## 4. Custom toolbars

The planned **New Toolbar** flow asks for a name, creates an empty custom
toolbar, and adds it to the Toolbars pane. Custom toolbars can then be filled
from Commands, renamed, deleted with confirmation, made visible or hidden, and
docked or floated by the planned toolbar runtime.

| Rule | Planned behaviour |
|---|---|
| Name length | 1–64 characters after trimming whitespace. |
| Name uniqueness | Case-insensitive across the active profile. |
| Maximum | 20 custom toolbars per profile. |
| Predefined toolbars | Main Toolbar, Quick Access Area, and Properties Toolbar cannot be renamed or deleted. |
| Delete confirmation | Removing a custom toolbar also removes its tools from that toolbar. |

The source reports an error when a 21st custom toolbar is attempted and when a
name duplicates an existing toolbar. The planned messages are listed in
[section 9](#9-errors-and-recovery).

> **Screenshot placeholder:** custom toolbar named “Redaction Tools” with
> New, Rename, and Delete controls visible and Main Toolbar controls disabled.

## 5. Quick Access Area

The Quick Access Area is planned as a flexible top-right strip for the tools a
person uses most often. The specification calls for **5–12 tools** and vertical
dividers for grouping. The domain source currently defines `MAX_TOOLS = 12`,
`MIN_TOOLS = 0`, and validates only the upper bound. This difference is left
visible rather than silently changing either document.

```text
Quick Access: [Select] [Highlight] [|] [Length] [Area] [Save]
```

| Requirement | Planned behaviour | Current source note |
|---|---|---|
| Tool count | 5–12 user-defined tools. | `QuickAccessConfig::is_valid()` rejects only more than 12. |
| Growth | Area grows and shrinks horizontally as tools are added or removed. | Runtime is planned. |
| Reorder | Drag inside the area to change order. | Runtime is planned. |
| Dividers | Divider pseudo-command creates a vertical separator. | `ToolbarItem::Divider` is defined in the domain. |
| Visibility | Always visible unless the active profile explicitly hides it. | Profile integration is planned. |

> **Screenshot placeholder:** Quick Access Area with five tools, one divider,
> and the toolbar growing after a sixth tool is added.

## 6. Properties Toolbar

The Properties Toolbar is planned as a fixed-structure context toolbar. The
customization dialog may show or hide it, but tools cannot be added to or
removed from it.

| Behaviour | Planned result |
|---|---|
| No markup selected | Toolbar hidden. |
| Markup selected | Toolbar appears with relevant property controls. |
| Empty canvas selected | Toolbar hides again. |
| Dialog checkbox unchecked | Auto-show is disabled; the toolbar stays hidden. |
| Drag a command onto its Items list | Drop rejected; structure remains fixed. |

Planned controls are Color, Line Weight, Opacity, Fill, Font Family, Font Size,
Line Style, and Arrow Style, shown as appropriate for the selected markup.

> **Screenshot placeholder:** Properties Toolbar visible below the main
> toolbar while a rectangle is selected, then hidden when the selection is
> cleared.

## 7. Toolbar lock and per-toolbar override

The planned global **Lock Toolbars** toggle hides drag handles and prevents
movement. When global lock is active, custom toolbars are intended to have an
independent override so one custom toolbar can remain movable while the rest
stay locked. Lock state is intended to be saved with the active profile.

| State | Intended UI result |
|---|---|
| Global lock off | Handles visible; toolbars can be moved. |
| Global lock on | Handles hidden; toolbars cannot be moved. |
| Global lock on plus custom override off | That custom toolbar shows a handle and can be moved. |
| Profile switch | Restore the lock state stored with the profile. |

Source note: `ToolbarLockState::is_locked()` currently returns `false`
immediately when `global_locked` is false, so a `true` per-toolbar override
does not lock a toolbar while global lock is off, despite the field comment.
The planned override behaviour above applies while global lock is active, as
described by the specification.

> **Screenshot placeholder:** globally locked main toolbar with hidden handles
> and one unlocked custom toolbar showing a drag handle.

## 8. Profiles and reset

Toolbar visibility, item order, custom toolbars, Quick Access contents, and
lock state are intended to be part of the active profile. Applying a profile
should restore that complete toolbar configuration; saving a profile should
include the current toolbar state. Predefined profiles are intended to carry
workflow-specific defaults, such as measurement tools prominent in the
Measurement profile.

| Reset action | Planned scope |
|---|---|
| Reset to Defaults | Restore the active profile’s factory toolbar configuration after confirmation. |
| Reset All Profiles | Restore every profile’s toolbar configuration after confirmation. |

Profile storage itself is F-606 and remains a draft. See
[Profiles (planned feature)](profiles.md) and
[`docs/profile-schema.md`](../profile-schema.md) for the current contract and
its unresolved conflicts.

## 9. Errors and recovery

These are the planned user-facing messages from the F-608 error table. They are
not current release behaviour.

| Condition | Planned message | Recovery |
|---|---|---|
| Duplicate toolbar name | `Toolbar 'X' already exists. Choose a different name.` | Focus and highlight the name field. |
| 20 custom toolbars already exist | `Maximum 20 custom toolbars reached. Delete unused toolbars first.` | Disable New Toolbar or block creation. |
| Empty, over-long, or reserved name | `Invalid toolbar name. Use 1-64 characters.` | Focus the name field. |
| Invalid drop target | No message. | Visual reject; tool returns to source. |
| Corrupt profile toolbar config | `Toolbar config in profile 'X' invalid. Resetting to defaults.` | Reset the profile toolbar state. |
| More than 12 Quick Access tools | `Quick Access limited to 12 tools. Remove tools to add more.` | Block the addition and highlight the area. |

## 10. Known source/spec differences

These differences are documented for reviewers and implementers:

- The specification says seven categories but lists eight; the domain enum has
  eight entries including `Advanced`.
- Some source comments say Markup Tools has 15 commands, while the registry
  contains 16 including Stamp Presets.
- Registry IDs are short strings such as `sel`, `hl`, and `ln`. Tests and the UI
  fallback list use longer IDs such as `select`, `highlight`, and `length`.
- Four registry shortcut strings contain a trailing space:
  `Ctrl+Shift+] `, `Ctrl+Shift+[ `, `Ctrl+] `, and `Ctrl+[ `.
- `Pan` and `Highlight` both register `H`; the shortcut strings are metadata
  and are not proof that an active keybinding exists.
- The Quick Access specification minimum is 5, while the source soft minimum
  is 0.
- Per-toolbar lock overrides currently take effect only under global lock, as
  described in [section 7](#7-toolbar-lock-and-per-toolbar-override).

Documentation was drafted on 2026-11-25. It does not mark F-608 implemented or
shipped.
