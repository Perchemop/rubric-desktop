# Profiles (planned feature — draft documentation)

> **Status:** F-606 is `Draft — Pending Review`. This guide describes the
> planned Profile/Workspace System from
> [`docs/spec/profile-workspace-system.spec.md`](../spec/profile-workspace-system.spec.md).
> The Profiles menu, profile files, and profile dialogs are **not shipped** in
> the current release. Existing profile-related UI files are placeholders. Do
> not follow these steps as current product instructions.

Profiles are intended to let one person keep several complete Rubric workspaces
and switch between them without rebuilding a layout each time. A profile will
store panel placement, toolbar configuration, theme override, measurement
defaults, and canvas settings. It will **not** store your PDF, markups, project
file, or recent-file list.

For the on-disk format, see [`docs/profile-schema.md`](../profile-schema.md).

---

## 1. What a profile is for

Use a profile when you repeatedly move between tasks such as reviewing,
measuring, and preparing markup stamps. The planned predefined profiles cover
the main personas; custom profiles cover everything else.

A profile is global to your user account, not to one project. Opening a
different `.rubric.json` project should not change the active profile. The
specification also leaves per-project overrides out of scope for this phase.

> **Screenshot placeholder — Profiles menu**
> After implementation, capture the Profiles menu showing the four predefined
> profiles and at least one custom profile. Replace this block with the reviewed
> image and add it to the PR description; do not commit a generated screenshot
> unless the repository's visual-evidence workflow requires a baseline.

---

## 2. Predefined profiles

Four predefined profiles are planned. They are embedded in the binary, written
to the profile directory on first run if missing, and read-only. Duplicate one
to make an editable copy.

| Profile | Planned layout | Intended use |
|---|---|---|
| **Default** | Balanced layout with all panels visible and standard toolbars. | Everyday starting point and safe fallback. |
| **Review** | Left: Thumbnails + Bookmarks (tabbed). Right: Markups List + Properties (tabbed). Bottom: Status. Main toolbar: standard review tools. | Reading comments, checking statuses, and moving between pages. |
| **Measurement** | Left: Layers + Tool Chest (tabbed). Right: Properties + Measurements (tabbed). Rulers visible. Grid enabled with snap. Measurement toolbar visible. | Calibrated takeoffs and geometry checks. |
| **Markup** | Left: Tool Chest + Stamps (tabbed). Right: Properties + Layers (tabbed). Maximum canvas space and minimal toolbars. | Creating and applying reusable markup styles. |

Predefined profiles cannot be renamed or deleted. Exporting one is planned to
produce a custom, editable copy so the built-in profile stays intact.

---

## 3. Switch profiles

Planned flow:

1. Open the Profiles menu. The exact location is still an open review question:
   a dedicated menu item or an entry under View.
2. Hover a profile to see its preview: name, description, and a short layout
   summary.
3. Select the profile. It is applied immediately, with no confirmation dialog.
4. Rubric should restore the captured panel positions, toolbar order, theme
   override, measurement defaults, and canvas settings. The specification sets
   a 500 ms target and a 200 ms transition that respects reduced-motion
   preferences.

Selecting **Default** should return the balanced factory layout.

> **Screenshot placeholder — profile preview tooltip**
> Capture the hover preview for **Measurement** so the name, description,
> rulers, and grid summary are visible. Replace this block with the reviewed
> image.

---

## 4. Create a profile

Planned flow:

1. Arrange the panels, toolbars, rulers, grid, zoom, and page layout as wanted.
2. Choose **Profiles ▸ Save Current Layout…**.
3. Enter a unique name (1–64 characters). Names are case-insensitive, so
   `Review v2` and `review v2` cannot both exist.
4. Add an optional description (up to 256 characters) for the hover preview.
5. Choose whether the new profile becomes active, then confirm.

The planned file name is a sanitised kebab-case form of the profile name, for
example `My Review` → `my-review.rubric-profile`.

| Platform | Planned profile directory |
|---|---|
| Linux | `~/.config/rubric/profiles/` |
| macOS | `~/.config/rubric/profiles/` |
| Windows | `%APPDATA%\Rubric\profiles\` |

The directory is created automatically on first run. A maximum of 50 profiles
is planned, with a warning at 45.

> **Screenshot placeholder — Save Profile dialog**
> Capture the dialog with a valid name, description, layout summary, and the
> “Set as active profile” option visible. Replace this block with the reviewed
> image.

---

## 5. Manage profiles

The planned **Manage Profiles…** dialog lists each profile with its type,
last-modified time, and available actions.

| Action | Planned behaviour |
|---|---|
| New | Create a profile from the current layout. |
| Duplicate | Copy a profile under a new name; this is how a predefined profile becomes editable. |
| Rename | Change a custom profile's display name and file name. Predefined profiles are excluded. |
| Delete | Remove a custom profile from disk and from the menu. Predefined profiles are excluded. |
| Import | Add a `.rubric-profile` file after validation and any needed migration. |
| Export | Write a `.rubric-profile` file to a chosen location. |
| Set default | Choose which profile loads on the next start. |

The dialog is also expected to provide a preview area, search/filter by name,
and keyboard navigation. Those controls are part of the draft acceptance work,
not existing product behaviour.

> **Screenshot placeholder — Manage Profiles dialog**
> Capture the list, preview pane, and action buttons with at least one
> predefined and one custom profile. Replace this block with the reviewed
> image.

---

## 6. Import and export

### Export

1. Open **Manage Profiles…**.
2. Select a profile and choose **Export**.
3. Choose a destination and file name with the `.rubric-profile` extension.

Exporting a predefined profile should create a custom copy, so the exported
file can be edited and imported elsewhere.

### Import

1. Open **Manage Profiles…**.
2. Choose **Import** and select a `.rubric-profile` file.
3. Review any warning about migrated or reset settings.
4. The profile should appear in the list and menu if validation succeeds.

Planned validation outcomes:

| File | Planned result |
|---|---|
| Current schema, valid JSON | Added as-is. |
| Older schema | Migrated in memory; a warning names reset fields; the original file is not rewritten. |
| Newer schema | Refused with a message to update Rubric. |
| Missing schema version or malformed JSON | Refused as not a valid Rubric profile. |
| Duplicate name | Refused; choose another name. |
| 50 profiles already present | Refused; delete an unused profile first. |

> **Screenshot placeholder — import migration warning**
> Capture the warning shown for an older profile with settings that had to be
> reset. Replace this block with the reviewed image.

---

## 7. Auto-save on close

The planned first-close prompt is:

> Update active profile with current layout?
> **Yes** · **No** · **Don't Ask Again**

- **Yes** updates the active profile when the application closes gracefully.
- **No** leaves the profile unchanged.
- **Don't Ask Again** records the choice in app-level settings so later closes
  do not prompt.

Auto-save is intended for graceful shutdown only; an abnormal exit should not
write a possibly incomplete profile. The exact persisted setting values and how
auto-save treats a predefined active profile remain review questions because
predefined profiles are read-only. The safe expectation is to duplicate a
predefined profile before relying on auto-save for it.

> **Screenshot placeholder — first-close auto-save prompt**
> Capture the three-button prompt. Replace this block with the reviewed image.

---

## 8. Keyboard shortcuts

No profile keyboard shortcut is shipped. The specification asks reviewers to
decide whether `Ctrl+1` through `Ctrl+4` should select the first four profiles.
Those combinations are **not safe to reuse as-is** because the current UI
already documents `Ctrl+1` / `Ctrl+2` / `Ctrl+3` for panel toggles in the
existing keyboard-shortcut notes.

Until a decision is made, assume only generic menu navigation:

| Key | Planned behaviour |
|---|---|
| Arrow keys | Move through profiles in the menu or dialog list. |
| `Enter` | Apply or confirm the selected action. |
| `Esc` | Close the menu or dialog without applying a pending change. |

Confirm the final shortcuts in the main
[`USER_GUIDE.md`](../../USER_GUIDE.md) after F-606 implementation.

---

## 9. Troubleshooting

| Symptom | What to check |
|---|---|
| There is no Profiles menu. | Expected today: F-606 is not shipped. This guide is planning documentation only. |
| A profile does not appear after import. | Check that the file has the `.rubric-profile` extension, a `schema_version`, valid JSON, and a unique name. |
| Import says a newer Rubric version is required. | Use the profile with a newer Rubric build. Do not lower `schema_version` by hand; unknown fields would be ignored. |
| Rubric starts with Default and logs a warning. | The selected profile was invalid or incomplete and was skipped; validate it against `docs/profile-schema.md`. |
| A layout is only partly restored. | Panel docking (F-607) and toolbar customisation (F-608) are also draft specs. Full persistence depends on their reviewed implementation. |
| A profile rename or delete is unavailable. | Predefined profiles are read-only; duplicate one first. |
| Auto-save did not run. | Auto-save is planned for graceful close only. Check whether the active profile was custom and whether a preference was saved. |
| Saving fails with “disk full” or “permission denied”. | Free space or fix the profile directory permissions, then retry. Rubric should keep the previous file. |
| The profile limit warning appears. | Delete or merge unused custom profiles; the planned cap is 50. |

---

## 10. Implementation and review status

- Feature spec: `docs/spec/profile-workspace-system.spec.md` — Draft — Pending
  Review.
- File format: `docs/profile-schema.md` — draft; unresolved contract conflicts
  are listed near the top.
- Dependencies: F-607 (panel docking/splitting) and F-608 (toolbar
  customisation) are also draft specs.
- Implementation: not complete. No CI evidence supports a shipped Profiles
  menu, `.rubric-profile` files, migrations, or auto-save.
- This page contains screenshot placeholders to be replaced with reviewed
  evidence after implementation.
