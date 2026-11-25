# Rubric Profile File Schema

This document defines the on-disk format of `.rubric-profile` files used by the
Profile/Workspace System (feature F-606). It follows the functional requirements
in [`docs/spec/profile-workspace-system.spec.md`](spec/profile-workspace-system.spec.md),
in particular FR-14–FR-17 and FR-24–FR-26.

- **Target schema version:** `1.2` (draft; review required)
- **File extension:** `.rubric-profile`
- **Encoding:** UTF-8, JSON with 2-space indentation and a trailing newline
- **Status:** Draft — pending review; no shipped release writes `.rubric-profile` files

This is a design document for F-606. The feature specification is still
`Draft — Pending Review`, and its implementation checklist is unfinished. The
prose below describes the intended format and behaviour; it must not be read as
a claim that profile files, the profile store, or the Profiles menu are present
in the current release. The acceptance criteria use schema `1.2` (AC-13/AC-14),
while the implementation TODO says the first `ProfileSchemaVersion` constant is
`1`. Other unresolved contract conflicts are listed below.

## Unresolved contract conflicts

This document records the **documented target contract**; it does not settle
open implementation decisions. The following conflicts must be resolved during
F-606 review before this schema is marked final or implementation begins.

| Topic | Specification / acceptance criteria | Implementation TODO | Current documentation stance |
|---|---|---|---|
| Schema version | AC-13 and AC-14 name current schema `1.2`. | Define `ProfileSchemaVersion` starting at `1`. | Use `1.2` in examples and migration tables because those are the versions the acceptance criteria exercise. Review must choose the shipped constant. |
| Field naming | FR-16 lists `schema_version`, `created_at`, `ui_layout`, and other snake_case keys. | Serialize with `#[serde(rename = "camelCase")]`. | Use snake_case on disk; treat camelCase aliases as legacy `1.0` input (section 5). Review must choose the canonical spelling. |
| Theme override key | `ui_layout.theme.override` is the documented structure. | Apply the override when `theme_override` is set. | Use `theme.override` throughout this document. Review must confirm the exact on-disk key and Rust field name. |
| Migration numbering | AC-13 migrates `1.0` → `1.2`; section 5 defines `1.0` → `1.1` → `1.2`. | Registry uses integer steps `v1→v2`, `v2→v3`. | Keep the `MAJOR.MINOR` chain from the acceptance criteria; map it to the registry only after review. |

Profiles are intended to be global, per-user files. They are not stored inside
a project's `.rubric.json` sidecar. Planned default locations:

| Platform | Profile directory |
|---|---|
| Linux | `~/.config/rubric/profiles/` |
| macOS | `~/.config/rubric/profiles/` |
| Windows | `%APPDATA%\Rubric\profiles\` |

The directory is created automatically on first run. The four predefined
profiles are embedded in the binary and written to this directory only when
missing (FR-17). File names are derived from the profile name with a
kebab-case, sanitised form, for example `My Review` → `my-review.rubric-profile`.
Names are unique case-insensitively; a duplicate name is rejected before a file
is written.

---

## 1. JSON Schema

The following is a JSON Schema (draft 2020-12) for version `1.2`. The keys below
use the documented on-disk spelling from FR-16 (`schema_version`, `created_at`,
`ui_layout`, and so on). Early drafts of the serializer used camelCase aliases
such as `schemaVersion`; treat such files as version `1.0` and migrate them as
described in section 5 rather than editing the version field by hand.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://github.com/Perchemop/rubric-desktop/blob/main/docs/profile-schema.md",
  "title": "Rubric workspace profile",
  "description": "A named, versioned snapshot of Rubric's UI layout and measurement/canvas defaults.",
  "type": "object",
  "required": [
    "schema_version",
    "name",
    "description",
    "created_at",
    "modified_at",
    "is_predefined",
    "ui_layout"
  ],
  "additionalProperties": false,
  "properties": {
    "schema_version": {
      "type": "string",
      "pattern": "^[0-9]+\\.[0-9]+$",
      "description": "Schema version as MAJOR.MINOR. Current version is 1.2."
    },
    "name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 64,
      "description": "Display name; unique across profiles, case-insensitive."
    },
    "description": {
      "type": "string",
      "maxLength": 256,
      "description": "Short summary shown in the Profiles menu preview tooltip."
    },
    "created_at": {
      "type": "string",
      "format": "date-time",
      "description": "RFC 3339 timestamp of first creation."
    },
    "modified_at": {
      "type": "string",
      "format": "date-time",
      "description": "RFC 3339 timestamp of the last write, including migration and auto-save."
    },
    "is_predefined": {
      "type": "boolean",
      "description": "True for the four built-in profiles; those cannot be renamed or deleted."
    },
    "ui_layout": {
      "$ref": "#/$defs/uiLayout"
    }
  },
  "$defs": {
    "uiLayout": {
      "type": "object",
      "description": "Complete captured UI state.",
      "required": ["panels", "toolbars", "theme", "measurement", "canvas"],
      "properties": {
        "panels": { "$ref": "#/$defs/panels" },
        "toolbars": { "$ref": "#/$defs/toolbars" },
        "theme": { "$ref": "#/$defs/theme" },
        "measurement": { "$ref": "#/$defs/measurement" },
        "canvas": { "$ref": "#/$defs/canvas" }
      }
    },
    "panelId": {
      "type": "string",
      "enum": [
        "thumbnails",
        "layers",
        "tool_chest",
        "markups_list",
        "properties",
        "stamps",
        "measurements",
        "bookmarks",
        "status"
      ],
      "description": "Stable identifier for a dockable panel."
    },
    "panels": {
      "type": "object",
      "description": "Dock strips and panel access bar state.",
      "required": [
        "left",
        "right",
        "bottom",
        "strip_sizes",
        "panel_access_bar_auto_hide"
      ],
      "properties": {
        "left": {
          "type": "array",
          "items": { "$ref": "#/$defs/panel" },
          "description": "Panels docked in the left strip, in tab order."
        },
        "right": {
          "type": "array",
          "items": { "$ref": "#/$defs/panel" },
          "description": "Panels docked in the right strip, in tab order."
        },
        "bottom": {
          "type": "array",
          "items": { "$ref": "#/$defs/panel" },
          "description": "Panels docked in the bottom strip, in tab order."
        },
        "strip_sizes": {
          "type": "object",
          "required": ["left", "right", "bottom"],
          "properties": {
            "left": { "type": "number", "minimum": 120 },
            "right": { "type": "number", "minimum": 120 },
            "bottom": { "type": "number", "minimum": 100 }
          }
        },
        "panel_access_bar_auto_hide": {
          "type": "boolean",
          "description": "Whether the left-edge panel access bar collapses until hovered."
        }
      }
    },
    "panel": {
      "type": "object",
      "required": ["id", "visible", "position", "split_state", "tab_order", "floating"],
      "properties": {
        "id": { "$ref": "#/$defs/panelId" },
        "visible": { "type": "boolean" },
        "position": {
          "type": "string",
          "enum": ["left", "right", "bottom", "floating"]
        },
        "split_state": {
          "type": ["object", "null"],
          "description": "Null when the panel occupies its strip alone.",
          "required": ["axis", "ratio", "pane"],
          "properties": {
            "axis": { "type": "string", "enum": ["horizontal", "vertical"] },
            "ratio": { "type": "number", "minimum": 0.05, "maximum": 0.95 },
            "pane": { "type": "integer", "minimum": 0 }
          }
        },
        "tab_order": {
          "type": "array",
          "items": { "$ref": "#/$defs/panelId" },
          "uniqueItems": true,
          "description": "Order of tabs within this panel's group."
        },
        "floating": {
          "type": ["object", "null"],
          "description": "Null when docked; geometry and mode when floating.",
          "required": ["x", "y", "width", "height", "mode"],
          "properties": {
            "x": { "type": "integer" },
            "y": { "type": "integer" },
            "width": { "type": "integer", "minimum": 200 },
            "height": { "type": "integer", "minimum": 150 },
            "mode": { "type": "string", "enum": ["windowed", "overlay"] }
          }
        }
      }
    },
    "toolbars": {
      "type": "object",
      "required": [
        "main",
        "quick_access",
        "properties",
        "custom_toolbars",
        "global_lock"
      ],
      "properties": {
        "main": { "$ref": "#/$defs/toolbar" },
        "quick_access": { "$ref": "#/$defs/toolbar" },
        "properties": { "$ref": "#/$defs/toolbar" },
        "custom_toolbars": {
          "type": "array",
          "maxItems": 20,
          "items": { "$ref": "#/$defs/customToolbar" }
        },
        "global_lock": { "type": "boolean" }
      }
    },
    "toolbar": {
      "type": "object",
      "required": ["visible", "tools_order", "lock_state"],
      "properties": {
        "visible": { "type": "boolean" },
        "tools_order": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 },
          "description": "Stable tool identifiers in display order; dividers are represented by 'divider'."
        },
        "lock_state": { "type": "boolean" }
      }
    },
    "customToolbar": {
      "allOf": [
        { "$ref": "#/$defs/toolbar" },
        {
          "type": "object",
          "required": ["name"],
          "properties": {
            "name": { "type": "string", "minLength": 1, "maxLength": 64 }
          }
        }
      ]
    },
    "theme": {
      "type": "object",
      "required": ["override"],
      "properties": {
        "override": {
          "type": ["string", "null"],
          "enum": ["light", "dark", "high_contrast", null],
          "description": "Null means follow the app-level theme preference."
        }
      }
    },
    "measurement": {
      "type": "object",
      "required": ["units", "precision", "scale_preset", "grid", "rulers"],
      "properties": {
        "units": {
          "type": "string",
          "enum": ["points", "inches", "feet", "millimetres", "centimetres", "metres"]
        },
        "precision": { "type": "integer", "minimum": 0, "maximum": 6 },
        "scale_preset": {
          "type": "string",
          "description": "Named scale preset, or 'custom' when a ratio was entered directly."
        },
        "grid": { "$ref": "#/$defs/grid" },
        "rulers": { "type": "boolean", "description": "Ruler visibility." }
      }
    },
    "grid": {
      "type": "object",
      "required": ["enabled", "snap", "spacing"],
      "properties": {
        "enabled": { "type": "boolean" },
        "snap": { "type": "boolean" },
        "spacing": { "type": "number", "exclusiveMinimum": 0 }
      }
    },
    "canvas": {
      "type": "object",
      "required": [
        "zoom",
        "page_layout",
        "ruler_visibility",
        "guide_visibility"
      ],
      "properties": {
        "zoom": { "type": "number", "exclusiveMinimum": 0 },
        "page_layout": {
          "type": "string",
          "enum": ["single", "continuous", "facing"]
        },
        "ruler_visibility": { "type": "boolean" },
        "guide_visibility": { "type": "boolean" },
        "show_measurements": { "type": "boolean" }
      }
    }
  }
}
```

Unknown root properties are rejected: a profile that fails validation is
skipped with a warning and Rubric starts with **Default** (NFR-04). Unknown
properties inside `ui_layout` are ignored by version `1.2` readers so that a
minor-version addition does not make existing profiles unloadable.

---

## 2. Field reference

### 2.1 Root object

| Field | Type | Required | Description |
|---|---|---|---|
| `schema_version` | string `MAJOR.MINOR` | yes | Format version; current value `1.2`. A higher major version is refused. |
| `name` | string (1–64) | yes | Display name in the Profiles menu. Unique case-insensitively. |
| `description` | string (0–256) | yes | Shown in the hover preview tooltip (FR-18). |
| `created_at` | RFC 3339 date-time | yes | Creation timestamp; preserved across migration. |
| `modified_at` | RFC 3339 date-time | yes | Last write, including migration and auto-save. |
| `is_predefined` | boolean | yes | Built-in profiles are read-only; `false` for user profiles and exported copies. |
| `ui_layout` | object | yes | Complete captured UI state (section 2.2). |

### 2.2 `ui_layout`

| Field | Type | Required | Description |
|---|---|---|---|
| `panels` | object | yes | Left, right and bottom dock strips, strip sizes, access-bar state. |
| `toolbars` | object | yes | Main, Quick Access, Properties and custom toolbars plus global lock. |
| `theme` | object | yes | Optional per-profile theme override. |
| `measurement` | object | yes | Units, precision, scale preset, grid and ruler defaults. |
| `canvas` | object | yes | Zoom, page layout and overlay visibility. |

### 2.3 `panels` and `panel` entries

| Field | Type | Required | Description |
|---|---|---|---|
| `panels.left` | array of panel entries | yes | Left dock strip in tab order. |
| `panels.right` | array of panel entries | yes | Right dock strip in tab order. |
| `panels.bottom` | array of panel entries | yes | Bottom dock strip in tab order. |
| `panels.strip_sizes` | object | yes | Pixel sizes for left, right and bottom strips. |
| `panels.strip_sizes.left` | number ≥ 120 | yes | Width of the left strip in pixels. |
| `panels.strip_sizes.right` | number ≥ 120 | yes | Width of the right strip in pixels. |
| `panels.strip_sizes.bottom` | number ≥ 100 | yes | Height of the bottom strip in pixels. |
| `panels.panel_access_bar_auto_hide` | boolean | yes | Collapse-until-hover state of the left-edge bar. |
| `panel.id` | panel ID enum | yes | Stable panel identifier. |
| `panel.visible` | boolean | yes | Hidden panels remain hidden when the profile is applied. |
| `panel.position` | `left` \| `right` \| `bottom` \| `floating` | yes | Dock strip or floating state. |
| `panel.split_state` | object or `null` | yes | Split geometry; `null` when the panel occupies its strip alone. |
| `panel.split_state.axis` | `horizontal` \| `vertical` | when split | Direction of the split. |
| `panel.split_state.ratio` | number 0.05–0.95 | when split | Fraction of the split occupied by the first pane. |
| `panel.split_state.pane` | integer ≥ 0 | when split | Index of the pane this panel belongs to. |
| `panel.tab_order` | array of panel IDs | yes | Order of tabs in this panel's group. |
| `panel.floating` | object or `null` | yes | `null` when docked; otherwise geometry and mode. |
| `panel.floating.x` | integer | when floating | Horizontal position of the floating panel. |
| `panel.floating.y` | integer | when floating | Vertical position of the floating panel. |
| `panel.floating.width` | integer ≥ 200 | when floating | Width of the floating panel. |
| `panel.floating.height` | integer ≥ 150 | when floating | Height of the floating panel. |
| `panel.floating.mode` | `windowed` \| `overlay` | when floating | Whether the panel is a window or an in-app overlay. |

Panel IDs: `thumbnails`, `layers`, `tool_chest`, `markups_list`, `properties`,
`stamps`, `measurements`, `bookmarks`, `status`. The last two appear in the
predefined Review profile (FR-11).

### 2.4 `toolbars`

| Field | Type | Required | Description |
|---|---|---|---|
| `toolbars.main` | toolbar object | yes | Main toolbar state. |
| `toolbars.quick_access` | toolbar object | yes | Quick Access Area state. |
| `toolbars.properties` | toolbar object | yes | Context-sensitive Properties toolbar auto-show state. |
| `toolbars.custom_toolbars` | array of custom toolbar objects | yes | Up to 20 user-defined toolbars, each with a unique name. |
| `toolbars.global_lock` | boolean | yes | Locks all toolbars; custom toolbars may override with `lock_state`. |
| `toolbar.visible` | boolean | yes | Show/hide the toolbar. |
| `toolbar.tools_order` | array of strings | yes | Stable tool IDs in display order; `divider` inserts a separator. |
| `toolbar.lock_state` | boolean | yes | Per-toolbar lock; meaningful for custom toolbars. |
| `custom_toolbar.name` | string (1–64) | yes | Unique, case-insensitive custom toolbar name. |

### 2.5 `theme`, `measurement` and `canvas`

| Field | Type | Required | Description |
|---|---|---|---|
| `theme.override` | `light` \| `dark` \| `high_contrast` \| `null` | yes | `null` follows the app-level preference. |
| `measurement.units` | unit enum | yes | Display unit for measurements. |
| `measurement.precision` | integer 0–6 | yes | Decimal places in measurement readouts. |
| `measurement.scale_preset` | string | yes | Named scale preset, or `custom`. |
| `measurement.grid.enabled` | boolean | yes | Grid visibility. |
| `measurement.grid.snap` | boolean | yes | Snap-to-grid state. |
| `measurement.grid.spacing` | number > 0 | yes | Grid spacing in document units. |
| `measurement.rulers` | boolean | yes | Ruler visibility shortcut used by the Measurement profile. |
| `canvas.zoom` | number > 0 | yes | Zoom factor applied on profile switch. |
| `canvas.page_layout` | `single` \| `continuous` \| `facing` | yes | Page arrangement. |
| `canvas.ruler_visibility` | boolean | yes | Rulers on the canvas. |
| `canvas.guide_visibility` | boolean | yes | Alignment guides on the canvas. |
| `canvas.show_measurements` | boolean | no | Optional readout toggle; defaults to the app preference when absent. |

---

## 3. Example: custom profile

A complete, valid `1.2` profile for a reviewer who keeps thumbnails on the
left, properties on the right and the Markups List at the bottom:

```json
{
  "schema_version": "1.2",
  "name": "My Review",
  "description": "Thumbnails and properties with the markups console visible.",
  "created_at": "2026-11-25T09:00:00Z",
  "modified_at": "2026-11-25T09:42:11Z",
  "is_predefined": false,
  "ui_layout": {
    "panels": {
      "left": [
        {
          "id": "thumbnails",
          "visible": true,
          "position": "left",
          "split_state": null,
          "tab_order": ["thumbnails", "layers"],
          "floating": null
        },
        {
          "id": "layers",
          "visible": true,
          "position": "left",
          "split_state": null,
          "tab_order": ["thumbnails", "layers"],
          "floating": null
        }
      ],
      "right": [
        {
          "id": "properties",
          "visible": true,
          "position": "right",
          "split_state": null,
          "tab_order": ["properties"],
          "floating": null
        }
      ],
      "bottom": [
        {
          "id": "markups_list",
          "visible": true,
          "position": "bottom",
          "split_state": null,
          "tab_order": ["markups_list"],
          "floating": null
        }
      ],
      "strip_sizes": { "left": 210, "right": 260, "bottom": 220 },
      "panel_access_bar_auto_hide": false
    },
    "toolbars": {
      "main": {
        "visible": true,
        "tools_order": ["select", "highlight", "note", "divider", "length", "calibrate"],
        "lock_state": false
      },
      "quick_access": {
        "visible": true,
        "tools_order": ["save", "undo", "redo"],
        "lock_state": false
      },
      "properties": {
        "visible": true,
        "tools_order": [],
        "lock_state": true
      },
      "custom_toolbars": [],
      "global_lock": false
    },
    "theme": { "override": null },
    "measurement": {
      "units": "millimetres",
      "precision": 2,
      "scale_preset": "custom",
      "grid": { "enabled": false, "snap": false, "spacing": 10 },
      "rulers": true
    },
    "canvas": {
      "zoom": 1.0,
      "page_layout": "single",
      "ruler_visibility": true,
      "guide_visibility": false,
      "show_measurements": true
    }
  }
}
```

## 4. Example: predefined profile

Predefined profiles use the same structure with `is_predefined` set to `true`.
They are embedded in the binary, written to disk only when missing, and cannot
be renamed or deleted. Exporting one produces an editable copy with
`is_predefined: false` (FR-09, FR-26).

```json
{
  "schema_version": "1.2",
  "name": "Review",
  "description": "Thumbnails and bookmarks on the left, markups and properties on the right.",
  "created_at": "2026-11-25T00:00:00Z",
  "modified_at": "2026-11-25T00:00:00Z",
  "is_predefined": true,
  "ui_layout": {
    "panels": {
      "left": [
        {
          "id": "thumbnails",
          "visible": true,
          "position": "left",
          "split_state": null,
          "tab_order": ["thumbnails", "bookmarks"],
          "floating": null
        },
        {
          "id": "bookmarks",
          "visible": true,
          "position": "left",
          "split_state": null,
          "tab_order": ["thumbnails", "bookmarks"],
          "floating": null
        }
      ],
      "right": [
        {
          "id": "markups_list",
          "visible": true,
          "position": "right",
          "split_state": null,
          "tab_order": ["markups_list", "properties"],
          "floating": null
        },
        {
          "id": "properties",
          "visible": true,
          "position": "right",
          "split_state": null,
          "tab_order": ["markups_list", "properties"],
          "floating": null
        }
      ],
      "bottom": [
        {
          "id": "status",
          "visible": true,
          "position": "bottom",
          "split_state": null,
          "tab_order": ["status"],
          "floating": null
        }
      ],
      "strip_sizes": { "left": 200, "right": 280, "bottom": 40 },
      "panel_access_bar_auto_hide": false
    },
    "toolbars": {
      "main": {
        "visible": true,
        "tools_order": ["select", "highlight", "note", "status", "comment"],
        "lock_state": false
      },
      "quick_access": {
        "visible": true,
        "tools_order": ["open", "save", "undo", "redo"],
        "lock_state": false
      },
      "properties": {
        "visible": true,
        "tools_order": [],
        "lock_state": true
      },
      "custom_toolbars": [],
      "global_lock": false
    },
    "theme": { "override": null },
    "measurement": {
      "units": "points",
      "precision": 2,
      "scale_preset": "1:1",
      "grid": { "enabled": false, "snap": false, "spacing": 10 },
      "rulers": false
    },
    "canvas": {
      "zoom": 1.0,
      "page_layout": "single",
      "ruler_visibility": false,
      "guide_visibility": false
    }
  }
}
```

---

## 5. Schema version migration

Versions are `MAJOR.MINOR`. A higher **major** version is incompatible and is
refused; a higher **minor** version may add optional fields that version `1.2`
readers ignore. Imports of older profiles are migrated in memory, and the
original file chosen by the user is never rewritten (FR-24). The migrated copy
written into the profile directory carries `schema_version: "1.2"` and a fresh
`modified_at`. Migration functions are pure and deterministic (NFR-09).

| From | To | Change | Migration action | Possible data loss |
|---|---|---|---|---|
| `1.0` | `1.1` | Toolbar state introduced. | Insert default `main`, `quick_access` and `properties` toolbar objects; preserve the old ordered tool list as `main.tools_order`; set `lock_state` and `global_lock` to `false`. | None for fields present in `1.0`. |
| `1.1` | `1.2` | Flat panel arrays replaced by dock strips with `split_state`, `tab_order` and `floating`; theme, measurement and canvas defaults added; old `toolbar_order` renamed to `toolbars.tools_order`; timestamps normalised to RFC 3339. | Map each panel to its strip by `position`, rebuild `tab_order`, fill missing sections from the **Default** profile, transform the old toolbar list into `toolbars.main.tools_order`, and keep the original `created_at` when parseable. | Floating-window geometry and custom toolbar names not representable in `1.1` are dropped; the warning names the fields that were reset. |

### 5.1 Import outcomes

| Situation | Result |
|---|---|
| `schema_version` equals `1.2` and validates | Profile added to the list and available in the menu. |
| `schema_version` is `1.0` or `1.1` | Migrated through the chain, warning shown if settings were reset, then added. |
| `schema_version` is newer than the app supports | Import refused: `Profile requires newer Rubric version (X.Y.Z). Please update.` The file is left untouched. |
| `schema_version` is missing | Import refused: `Selected file is not a valid Rubric profile.` |
| Required fields missing or types wrong | Profile skipped with a warning; Rubric uses **Default** (NFR-04). |

Do not lower `schema_version` by hand to bypass a refusal. The loader would
treat newer fields as unknown and the layout would be silently reset. Update
Rubric instead, then import the same file again.
