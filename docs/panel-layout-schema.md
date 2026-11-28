# Panel Layout JSON Schema (planned F-607 — draft)

> **Status:** F-607 is `Draft — Pending Review`. This document describes the
> current `PanelLayout` data model in
> [`apps/rubric/src/domain/panel.rs`](../apps/rubric/src/domain/panel.rs). It is
> not a claim that the panel UI, drag-and-drop operations, or profile-level
> restoration are shipped.

## Scope

`PanelLayout` is the dock-layout object embedded in a profile as the camelCase
`panelLayout` field. The nested object itself has no `serde(rename_all)` attribute,
so its own fields are **snake_case**: `strip_size_left`, `access_bar`,
`schema_version`, and so on. This differs from some draft profile examples that
use `stripSizes` or `panels`; those examples are not the source contract.

The source constant is `PANEL_SCHEMA_VERSION = 1`. This schema documents that
version only.

Field defaults are declared with `#[serde(default)]`, so a deserializer accepts
an object that omits them. The schema below marks the minimum fields required by
deserialization and describes the serialized default values in prose. The
Rust constructor `DockedPanel::new()` has its own defaults, noted below; they
are not the same as the serde defaults for every field.

## JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://rubric.example/schema/panel-layout-1.schema.json",
  "title": "Rubric PanelLayout",
  "description": "Dock strips, split trees, floating state and panel access bar state for F-607. Draft schema version 1.",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "left": {
      "type": "array",
      "description": "Panels docked in the left strip, in tab order. Defaults to an empty array.",
      "items": { "$ref": "#/$defs/dockedPanel" }
    },
    "right": {
      "type": "array",
      "description": "Panels docked in the right strip, in tab order. Defaults to an empty array.",
      "items": { "$ref": "#/$defs/dockedPanel" }
    },
    "bottom": {
      "type": "array",
      "description": "Panels docked in the bottom strip, in tab order. Defaults to an empty array.",
      "items": { "$ref": "#/$defs/dockedPanel" }
    },
    "strip_size_left": {
      "type": "number",
      "description": "Width of the left dock strip in PDF points. Default 200.0.",
      "minimum": 0
    },
    "strip_size_right": {
      "type": "number",
      "description": "Width of the right dock strip in PDF points. Default 260.0.",
      "minimum": 0
    },
    "strip_size_bottom": {
      "type": "number",
      "description": "Height of the bottom dock strip in PDF points. Default 200.0.",
      "minimum": 0
    },
    "access_bar": {
      "$ref": "#/$defs/panelAccessBarState"
    },
    "schema_version": {
      "type": "integer",
      "const": 1,
      "description": "PANEL_SCHEMA_VERSION from panel.rs. Defaults to 1."
    }
  },
  "$defs": {
    "panelId": {
      "description": "Stable panel identifier. The Rust type also has Other(String) for plugin or future panels.",
      "anyOf": [
        {
          "type": "string",
          "enum": [
            "thumbnails",
            "layers",
            "tool_chest",
            "markups_list",
            "properties",
            "stamps",
            "measurements",
            "quantities",
            "pages",
            "tools",
            "inspector"
          ]
        },
        {
          "type": "string",
          "minLength": 1,
          "description": "Extension identifier, represented as PanelId::Other(String) in source."
        }
      ]
    },
    "dockLocation": {
      "type": "string",
      "description": "Dock strip containing the panel. Defaults to left when omitted.",
      "enum": ["left", "right", "bottom", "floating"]
    },
    "splitOrientation": {
      "type": "string",
      "enum": ["horizontal", "vertical"]
    },
    "splitTree": {
      "description": "Recursive split tree. Uses serde(tag = \"type\", content = \"content\").",
      "oneOf": [
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["type", "content"],
          "properties": {
            "type": { "const": "Leaf" },
            "content": { "$ref": "#/$defs/panelId" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["type", "content"],
          "properties": {
            "type": { "const": "Split" },
            "content": {
              "type": "object",
              "additionalProperties": false,
              "required": ["orientation", "ratio", "children"],
              "properties": {
                "orientation": { "$ref": "#/$defs/splitOrientation" },
                "ratio": {
                  "type": "number",
                  "minimum": 0.05,
                  "maximum": 0.95,
                  "description": "Split position. split_with_ratio clamps this range; 0.5 is centered."
                },
                "children": {
                  "type": "array",
                  "minItems": 2,
                  "maxItems": 2,
                  "items": { "$ref": "#/$defs/splitTree" }
                }
              }
            }
          }
        }
      ]
    },
    "floatingMode": {
      "type": "string",
      "description": "Current source variants only. There is no Presentation variant in FloatingMode.",
      "enum": ["windowed", "overlay"]
    },
    "floatingPanel": {
      "type": "object",
      "additionalProperties": false,
      "required": ["panel_id", "position", "size", "mode"],
      "properties": {
        "panel_id": { "$ref": "#/$defs/panelId" },
        "position": {
          "type": "array",
          "description": "[x, y] in PDF points from the top-left of the workspace.",
          "minItems": 2,
          "maxItems": 2,
          "items": { "type": "number" }
        },
        "size": {
          "type": "array",
          "description": "[width, height] in PDF points.",
          "minItems": 2,
          "maxItems": 2,
          "items": { "type": "number" }
        },
        "mode": { "$ref": "#/$defs/floatingMode" },
        "monitor": {
          "type": ["integer", "null"],
          "description": "Optional monitor identifier. Defaults to null."
        }
      }
    },
    "panelAccessBarState": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "auto_hide": {
          "type": "boolean",
          "description": "Whether the bar collapses until hovered. Defaults to false.",
          "default": false
        },
        "visible_icons": {
          "type": "array",
          "description": "Panel IDs shown in the access bar. Defaults to an empty array.",
          "items": { "$ref": "#/$defs/panelId" },
          "default": []
        }
      }
    },
    "dockedPanel": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id"],
      "properties": {
        "id": { "$ref": "#/$defs/panelId" },
        "visible": {
          "type": "boolean",
          "description": "Serde default is false; DockedPanel::new() sets true.",
          "default": false
        },
        "dock_location": {
          "$ref": "#/$defs/dockLocation"
        },
        "split_tree": {
          "anyOf": [
            { "$ref": "#/$defs/splitTree" },
            { "type": "null" }
          ],
          "description": "Omitted or null when there is no split. Skipped by the serializer when None."
        },
        "tab_order": {
          "type": "array",
          "description": "Panel IDs in tab order. Serde default is empty; DockedPanel::new() sets [id]. Skipped by the serializer when empty.",
          "items": { "$ref": "#/$defs/panelId" },
          "default": []
        },
        "floating": {
          "anyOf": [
            { "$ref": "#/$defs/floatingPanel" },
            { "type": "null" }
          ],
          "description": "Omitted or null when docked. Skipped by the serializer when None."
        }
      }
    }
  }
}
```

## Field reference

| Path | Type | Required to deserialize | Source default |
|---|---|---|---|
| `left` | array of `dockedPanel` | no | empty |
| `right` | array of `dockedPanel` | no | empty |
| `bottom` | array of `dockedPanel` | no | empty |
| `strip_size_left` | number | no | `200.0` |
| `strip_size_right` | number | no | `260.0` |
| `strip_size_bottom` | number | no | `200.0` |
| `access_bar` | `panelAccessBarState` | no | `auto_hide: false`, `visible_icons: []` |
| `schema_version` | integer | no | `1` |
| `dockedPanel.id` | `panelId` | yes | none |
| `dockedPanel.visible` | boolean | no | serde `false`; constructor `true` |
| `dockedPanel.dock_location` | `dockLocation` | no | `left` |
| `dockedPanel.split_tree` | `splitTree` or null | no | none |
| `dockedPanel.tab_order` | array of `panelId` | no | serde empty; constructor `[id]` |
| `dockedPanel.floating` | `floatingPanel` or null | no | none |
| `floatingPanel.position` | `[number, number]` | yes | constructor `[100.0, 100.0]` |
| `floatingPanel.size` | `[number, number]` | yes | constructor `[400.0, 300.0]` |
| `floatingPanel.mode` | `floatingMode` | yes | constructor `windowed` |
| `floatingPanel.monitor` | integer or null | no | `null` |

`TabGroup` also exists in `panel.rs`, with `tabs` and `active_index`, but it is
not a field of `PanelLayout`. Group membership is represented there by
`tab_order`; `active_index` is not persisted by this schema.

## Example

```json
{
  "left": [
    {
      "id": "thumbnails",
      "visible": true,
      "dock_location": "left",
      "tab_order": ["thumbnails", "layers"]
    },
    {
      "id": "layers",
      "visible": true,
      "dock_location": "left",
      "tab_order": ["layers"]
    }
  ],
  "right": [
    {
      "id": "properties",
      "visible": true,
      "dock_location": "right",
      "split_tree": {
        "type": "Split",
        "content": {
          "orientation": "vertical",
          "ratio": 0.6,
          "children": [
            { "type": "Leaf", "content": "properties" },
            { "type": "Leaf", "content": "stamps" }
          ]
        }
      },
      "tab_order": ["properties"]
    }
  ],
  "bottom": [],
  "strip_size_left": 200.0,
  "strip_size_right": 260.0,
  "strip_size_bottom": 200.0,
  "access_bar": {
    "auto_hide": false,
    "visible_icons": ["thumbnails", "layers", "properties", "stamps"]
  },
  "schema_version": 1
}
```

## Predefined source layouts

`predefined_layouts` in `panel.rs` currently supplies six `PanelLayout`
functions:

| Function | Notable values |
|---|---|
| `default()` | Left Thumbnails + Layers tabs; right Properties + Stamps + Measurements tabs; bottom Markups List; strip sizes `200/260/200`. |
| `review()` | Left Thumbnails with `Other("bookmarks")` in tab order; right Markups List + Properties tabs; right strip `280`, bottom strip `0`. |
| `measurement()` | Left Layers + Tool Chest tabs; right Properties + Measurements tabs; left strip `180`, bottom strip `0`. |
| `markup()` | Left Tool Chest + Stamps tabs; right Properties + Layers tabs; bottom strip `0`. |
| `minimal()` | No docked panels; all strip sizes `0`; access bar auto-hide enabled. |
| `presentation()` | No docked panels; all strip sizes `0`; access bar auto-hide enabled. Floating presentation panels belong to `Profile`, not this object. |

`predefined_profiles()` currently exposes only Default, Review, Measurement,
and Markup. The Minimal and Presentation `PanelLayout` functions exist but are
not yet represented by that profile factory. This mismatch is recorded rather
than hidden.

## Open review points

- `bookmarks` is used by Review as `PanelId::Other("bookmarks")`, but it is not
  one of the seven named inventory variants.
- The F-607 specification describes a presentation workflow, while
  `FloatingMode` has only `windowed` and `overlay`.
- Serde defaults and `DockedPanel::new()` defaults differ for `visible` and
  `tab_order`.
- The specification requires keyboard customization and access-bar toggling;
  this document only defines serialized panel state and does not prove those
  behaviours are implemented.

Documentation was drafted on 2026-11-25 from the F-607 specification and the
current domain source.
