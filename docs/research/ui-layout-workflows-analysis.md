# UI Layouts & Workflows Analysis — OpenTakeoff & Bluebeam

**Clean-room capability-level summary** of UI layouts and user workflows for construction takeoff apps. This informs the **v0.6 UI Parity Revamp** (docs/spec/README.md for feature IDs). No code, icons, fonts, string tables or trade dress copied; original structural patterns only.

---

## Method

Public black-box research from:
- Official documentation pages
- Feature lists and capability descriptions
- Public screenshots (for layout structure only, not pixel values)
- Community tutorials and review analyses

Every claim is an original paraphrase with source URLs. Implementation requires human approval per guide §2.3.

---

## 1. OpenTakeoff — Takeoff Quantities Panel Workflows

### Sources
- https://github.com/Kentucky-ai/opentakeoff
- https://opentakeoff.kentucky-ai.com (hosted canvas)

### 1.1 Quantity Schedule Panel

| Capability | Original Behavioral Pattern | Rubric Inspiration |
|------------|----------------------------|-------------------|
| **Panel Location** | Bottom dockable tab alongside Markups | Bottom-panel tab `Quantities` alongside `Markups List` |
| **Tab Navigation** | Icon strip with `Markups \| Quantities \| Measures` | Segmented control `Markups List \| Quantities` |
| **Empty State** | "No conditions yet. Click + to start." with "New Condition" button | "No takeoff conditions yet. Create one to start the schedule." + New condition button |
| **Condition Rows** | Horizontal row with colour swatch, name, items, total, unit; click selects assigned markups | Same structure: `■ | Name | Kind | Items | Quantity` |
| **Row Interaction** | Click = select contributors + jump to first page; double-click = edit condition | Same: R-17 selection contract |

### 1.2 Condition Management Workflows

| Workflow | OpenTakeoff Pattern | Rubric Mapping |
|----------|---------------------|----------------|
| **Create Condition** | "+" button in panel header, auto-names "Condition 1, 2..." | Takeoff ▸ New condition… (R-1) |
| **Rename Condition** | Inline edit on row double-click, validation rejects duplicates | Inspector edit with duplicate check (R-2) |
| **Delete Condition** | Trash icon, confirmation shows "X measurements will become unassigned" | Delete confirm naming consequence (R-4) |
| **Condition Properties** | Each condition has: name, kind, colour, waste %, height, thickness | TakeoffCondition struct with name/kind/height/colour |

### 1.3 Measurement Assignment Workflows

| Workflow | OpenTakeoff Pattern | Rubric Mapping |
|----------|---------------------|----------------|
| **Assign via Panel** | Row context menu: "Assign to Condition" with sub-items | Right-click markup → Assign to condition (R-7) |
| **Assign via Selection** | Multi-select + right-click → Assign to condition | Batch assign (R-9) |
| **Auto-Unassign** | Deleting condition auto-unassigns all associated measurements | Delete unassigns (R-4) |

### 1.4 Quantity Computation

| Computation | OpenTakeoff Approach | Rubric Domain Pattern |
|-------------|---------------------|----------------------|
| **Linear Quantities** | Sum of lengths in page units, converted to sheet scale | Σ Length measure values |
| **Wall Surface** | Run length × entered height | Perimeter/Length × condition.height |
| **Mixed Units** | Flag row with units involved, display in project default | `mixed_units` flag (R-15) |
| **Unassigned Row** | Dedicated row for unassigned measurements | Trailing "Unassigned" pseudo-row (spec R-16) |

### 1.5 Export Workflows

| Export Type | OpenTakeoff Implementation | Rubric Mapping |
|-------------|----------------------------|----------------|
| **CSV Export** | "Export Report" button, CSV with columns: Item, Category, Measure, Total, Unit | Quantities CSV export (R-19-R-20) |
| **Marked PDF** | Legend cover + annotated sheets as deliverables | Existing flatten-to-PDF, candidate for enhancement |
| **JSON/Export API** | MCP tools `takeoff_summary`, `export_report` | MCP bridge (out of scope for v1) |

---

## 2. Bluebeam — Multi-Panel Docking Architecture

### Sources
- https://support.bluebeam.com/user-manual/welcome/revu-interface.html
- https://support.bluebeam.com/revu/how-to/customize-panels.html
- https://support.bluebeam.com/user-manual/multiview.html

### 2.1 Dock Region Structure

**8-region vertical stack** (Bluebeam):
```
+--------------------------------------------------+
| Menu Bar                                         |
+--------------------------------------------------+
| Properties Toolbar (dynamic)                     |
+--------------------------------------------------+
| Toolbar Areas / Toolstrips                       |
+----------+---------------------------------------+
| Tool     |                                       |
| Chest    |   Main Workspace (Canvas)             |
| (left)   |                                       |
+----------+---------------------------------------+
| Navigation Bar (page controls, split)            |
+--------------------------------------------------+
| Bottom Panel (Markups List, 3D Model Tree)       |
+--------------------------------------------------+
| Status Bar                                       |
+--------------------------------------------------+
```

**Key patterns for Rubric**:
- **3 primary dock regions**: Left, Right, Bottom (vs Bluebeam's 8)
- **Status bar** at bottom with context-sensitive help
- **Navigation controls** distributed: top bar + status bar (vs dedicated nav bar)

### 2.2 Dock Behavior Patterns

| Behavior | Bluebeam Pattern | Rubric v0.6 Adaptation |
|----------|------------------|--------------------------|
| **Attach** | Drag to edge, position wheel appears | Drop zone preview when dragging panel |
| **Float** | Drag icon into workspace | Double-click header toggles float |
| **Split** | Drag onto panel header → position wheel | Stack tabs within same region |
| **Auto-hide** | Panel access bar can minimize | Optional future enhancement |
| **Persistence** | Saved in Profiles | Store in WorkspaceState/AppPrefs |

### 2.3 Panel Design Patterns

| Panel Type | Bluebeam Characteristics | Rubric UI Patterns |
|------------|--------------------------|-------------------|
| **Header** | Title + collapse × + pin ⚇ | Panel header with close toggle |
| **Tabs** | Tab strip within dock region | Same region, stacked tabs |
| **Content** | Scrollable, resizable | Match egui patterns |
| **State** | Visible/hidden persisted | `WorkspaceState.open` set |

---

## 3. Cross-App Workflow Patterns

### 3.1 Selection & Navigation

| Pattern | Bluebeam | OpenTakeoff | Rubric Implementation |
|---------|----------|-------------|----------------------|
| **Row Select** | Markups List row → highlight in canvas | Quantity row → highlight markups | R-17 selection contract |
| **Jump to Page** | Thumbnails or Markups List → scroll to page | Quantity row → first contributor | Same behavior expected |
| **Multi-Select** | Shift/Ctrl + click | Same | Existing in canvas.rs |

### 3.2 Keyboard Navigation

| Shortcut | Bluebeam | OpenTakeoff | Rubric |
|----------|----------|-------------|--------|
| **Panel Toggle** | Alt + letter | Menu access keys | Existing shortcuts |
| **Close Panel** | Ctrl+W | Escape | Escape closes dialogs |
| **Split Toggle** | Custom | N/A | Planned for v0.6 |
| **Preferences** | Ctrl+K | N/A | Menu path |

### 3.3 Empty/Error States

| State | Pattern | Rubric |
|-------|---------|--------|
| **Empty Panel** | Descriptive text + CTAs | "No takeoff conditions yet" + New button |
| **Loading** | Spinner overlay | Existing measurement loading states |
| **Error** | Persistent banner | Status bar error (R-21) |
| **Permission Denied** | Grayed UI + tooltip | Locked layers handled per spec |

---

## 4. UI Component Patterns for Takeoff (v0.5 + v0.6)

### 4.1 Condition Row Components

```
┌─────────────────────────────────────────────────┐
│ ■  Carpet (Area)        4    124.50 m²       │
└─────────────────────────────────────────────────┘
```

| Element | Required | Accessibility Notes |
|---------|----------|---------------------|
| Colour swatch | ✓ | AccessKit role: `color_button` |
| Name (editable) | ✓ | Enter commits, Esc cancels |
| Kind label | ✓ | Static, "Wall surface" for derived |
| Item count | ✓ | Tabular alignment, ≥24pt target |
| Total (tabulated) | ✓ | Full precision, unit suffix |
| Mixed units badge | ✓ | Tooltip lists units |

### 4.2 Assignment Context Menu

```
Assign to Condition
├── Linear
│   ├── Skirting (2m)
│   └── Dovetail (3m)
├── Area
│   ├── Carpet (124.50 m²)
│   └── Tiles (89.00 m²)
└── Wall Surface
    └── Drywall (42.00 m²)
Unassign
```

Pattern: Only show kinds compatible with markup type. Group by `ConditionKind`.

### 4.3 Condition Dialog

A tabbed dialog:
- **General** → name, kind (locked after creation)
- **Measurements** → height (WallSurface only), unit suffix displayed
- **Appearance** → colour swatch from palette
- **Notes** → optional text field
- **Actions** → Delete (danger) with "N measurements will become unassigned" warning

---

## 5. Accessibility Compliance Patterns

### 5.1 WCAG 2.2 AA Requirements

| Requirement | Pattern Source | Rubric Implementation |
|-------------|----------------|----------------------|
| **Contrast 4.5:1** | Both apps use high-contrast themes | Token system enforces (II.30.6) |
| **Keyboard Accessible** | Tab order follows reading order | Spec R-30 |
| **Focus Visible** | Focus ring on interactive elements | `focus.ring` token |
| **No Color Alone** | Text labels + colour indicators | Kind labels + swatches |
| **Target Size 24pt** | Button minimums | Row height, click targets |

### 5.2 Reduced Motion

| App | Handling | Rubric |
|-----|----------|--------|
| Bluebeam | Preferences > Performance | Toggle stored in AppPrefs |
| OpenTakeoff | N/A (browser-based) | Default off when unavailable |

---

## 6. Implementation Recommendations

### 6.1 For Takeoff Quantities Panel (T6-T10)

1. **Panel Structure** (bottom dock region — Rubric's panels are hand-rolled egui panels, not egui_dock)
   - Tab strip with Markups List / Quantities
   - Match OpenTakeoff's row structure
   - Empty state per spec R-13

2. **Quantity Row Design**
   - Follow spec's column order
   - Add mixed units badge (R-15)
   - Click selection contract (R-17)

3. **Condition Dialog**
   - Tabbed per section above
   - Height field validation (R-3)
   - Delete confirm naming (R-4)

### 6.2 For Docked Panels (v0.6 Phase B)

1. **Region Management** (WorkspaceState)
   - Left: Pages, Tools (existing)
   - Right: Inspector (existing) → extendable
   - Bottom: Markups List → add Quantities tab

2. **Drag/Drop Zones**
   - Visual feedback on hover
   - Drop zone preview (token coloured overlay)

3. **Persistence**
   - Serialize to AppPrefs
   - Validate on load (fallback to default)

### 6.3 For Split Views (v0.6 Phase C)

1. **Two-Pane Canvas**
   - Shared selection state
   - Independent page/zoom/pan
   - Follow primary/secondary pattern (R-S3)

---

## 7. Source URLs (Clean-Room Attribution)

### OpenTakeoff
- Repository: https://github.com/Kentucky-ai/opentakeoff
- MCP Reference: https://github.com/Kentucky-ai/opentakeoff/blob/main/mcp/README.md
- Hosted Canvas: https://opentakeoff.kentucky-ai.com

### Bluebeam Revu
- Interface Overview: https://support.bluebeam.com/user-manual/welcome/revu-interface.html
- Panel Customization: https://support.bluebeam.com/revu/how-to/customize-panels.html
- MultiView: https://support.bluebeam.com/user-manual/multiview.html
- Status Bar: https://support.bluebeam.com/user-manual/toolbars/status-bar.html

---

## 8. Clean-Room Attestation

All observations in this document are:
- Original behavioral summaries written for Rubric
- Derived from public documentation only
- Not copied from any product's UI, code, icons, fonts, or string tables
- Capability-level patterns suitable for inspiration (guide §2.1/§2.2)

Human approval required before implementation (guide §2.3).