# Toolbar Customization Dialog — Feature Specification

**Feature ID:** F-608 (extends F-6xx UI/UX polish)  
**Priority:** P0 — Core power-user feature, integrates with Profile System  
**Phase:** 6.2 (2-3 weeks)  
**Status:** Draft — Pending Review

---

## 1. Overview & User Value

### 1.1 Problem Statement
Professional users have distinct tool preferences: a reviewer needs status/comment tools prominent; an estimator needs measurement tools front-and-center; a markup author needs stamp/shape tools accessible. Rubric's current fixed toolbar forces all users into one layout, requiring excessive menu navigation or keyboard shortcuts for common actions.

### 1.2 Solution
A **Toolbar Customization Dialog** following the Bluebeam Revu four-pane model (Categories → Commands → Toolbars → Items) that allows users to:
- Show/hide toolbars
- Reorder tools within toolbars
- Add/remove tools from toolbars
- Create custom toolbars
- Configure Quick Access Area (5-12 user-defined tools)
- Lock toolbars globally with per-toolbar override
- All toolbar state persisted per-profile

### 1.3 User Value
- **Reviewers** → Status, Comments, Author tools in Main Toolbar
- **Estimators** → Measurement tools (Length, Area, Calibrate) prominent
- **Markup Authors** → Stamp Presets, Shapes, Ink in Quick Access
- **Power Users** → Custom "Redaction" toolbar with Sanitize, Redact, Bates
- **Teams** → Standardized toolbar layouts via shared profiles

### 1.4 Success Metrics
| Met | Target |
|-----|--------|
| Dialog open-to-first-edit | < 3 seconds |
| Custom toolbar creation | < 30 seconds |
| Toolbar config per-profile switch | < 500ms (with profile) |
| User adoption (custom toolbar) | > 35% of active users |
| Support tickets for "missing tool" | < 5% of current volume |

---

## 2. Functional Requirements (EARS Format)

### 2.1 Dialog Structure & Navigation

| ID | Requirement |
|----|-------------|
| FR-01 | WHEN the user chooses Tools > Toolbars > Customize... (or Ctrl+Shift+T), THE application SHALL open a modal dialog with four panes: **Categories** (left), **Commands** (left-center), **Toolbars** (right-center), **Items** (right). |
| FR-02 | THE **Categories** pane SHALL list all 7 command categories with checkboxes; checking a category filters the Commands pane. |
| FR-03 | THE **Commands** pane SHALL display all commands in the selected category(ies) with icon, name, and keyboard shortcut; commands are draggable. |
| FR-04 | THE **Toolbars** pane SHALL list all toolbars (Main Toolbar, Quick Access Area, Properties Toolbar, Custom Toolbars) with show/hide checkboxes and lock icons. |
| FR-05 | THE **Items** pane SHALL display the tools in the currently selected toolbar in order; items are draggable for reordering. |
| FR-06 | THE dialog SHALL support keyboard navigation: Tab between panes, Arrow keys within pane, Enter to activate, Delete to remove item. |

### 2.2 Command Categories (7 Total)

| Category | Key Commands |
|----------|--------------|
| **Selection & Navigation** | Select (V), Pan (H), Zoom In/Out/Fit, Page First/Prev/Next/Last, Jump to Page |
| **Markup Tools** | Highlight, Strikeout, Underline, Squiggly, Note, Text Box, Callout, Rectangle, Ellipse, Cloud, Polyline, Polygon, Line, Arrow, Ink, Stamp Presets |
| **Measurement Tools** | Length, Perimeter, Area, Angle, Diameter/Radius, Count, Calibrate Scale, Unit Selector |
| **Review & Comment** | Status Dropdown, Add Comment, Show Comments, Author Filter, Summary Report |
| **Edit & Modify** | Copy Properties, Paste Properties, Bring Forward, Send Backward, Bring to Front, Send to Back, Toggle Lock, Layer Visibility |
| **View & Display** | Theme Toggle, Show/Hide Grid, Snap to Grid, Grid Spacing, Show/Hide Rulers, Full Screen, Panel Visibility |
| **File & Project** | New, Open, Save, Save As, Import PDF, Export Image, Print, Recent Files, Project Settings |
| **Advanced** | Bates Numbering Config, Calibration Dialog, Preferences, Keyboard Shortcuts, Profiles Management |

### 2.3 Toolbar Operations

| ID | Requirement |
|----|-------------|
| FR-07 | WHEN the user drags a command from **Commands** to **Items**, THE application SHALL add the tool to that toolbar at the drop position. |
| FR-08 | WHEN the user drags an item within **Items**, THE application SHALL reorder tools in that toolbar. |
| FR-09 | WHEN the user drags an item from **Items** back to **Commands** or presses Delete, THE application SHALL remove the tool from the toolbar. |
| FR-10 | WHEN the user checks/unchecks a toolbar in **Toolbars**, THE application SHALL immediately show/hide that toolbar in the UI. |
| FR-11 | WHEN the user clicks "New Toolbar", THE application SHALL prompt for a name, create an empty custom toolbar, and add it to **Toolbars** list. |
| FR-12 | WHEN the user selects a custom toolbar and clicks "Rename", THE application SHALL prompt for new name and update. |
| FR-13 | WHEN the user selects a custom toolbar and clicks "Delete", THE application SHALL remove the toolbar and its tools (with confirmation). |
| FR-14 | PREDEFINED toolbars (Main Toolbar, Quick Access Area, Properties Toolbar) SHALL NOT be deletable or renamable. |

### 2.4 Quick Access Area

| ID | Requirement |
|----|-------------|
| FR-15 | THE Quick Access Area SHALL support 5-12 user-defined tools (flexible count). |
| FR-16 | WHEN the user adds tools to Quick Access Area, THE application SHALL grow/shrink the area dynamically. |
| FR-17 | QUICK ACCESS tools SHALL always be visible (never hidden by profile switch unless profile explicitly configures it). |
| FR-18 | QUICK ACCESS SHALL support drag-reorder within the area. |
| FR-19 | QUICK ACCESS SHALL support dividers (vertical separators) for visual grouping. |

### 2.5 Properties/Context Toolbar

| ID | Requirement |
|----|-------------|
| FR-20 | THE Properties Toolbar SHALL have fixed structure (cannot add/remove property fields). |
| FR-21 | THE Properties Toolbar SHALL show/hide automatically based on markup selection. |
| FR-22 | IN the customization dialog, Properties Toolbar SHALL appear in Toolbars list with show/hide checkbox (controls auto-show behavior). |
| FR-23 | PROPERTIES Toolbar SHALL show: Color, Line Weight, Opacity, Fill, Font Family/Size, Line Style, Arrow Style (context-dependent). |

### 2.6 Toolbar Lock

| ID | Requirement |
|----|-------------|
| FR-24 | WHEN the user clicks "Lock Toolbars" (global), THE application SHALL hide all toolbar handles and prevent all toolbar movement. |
| FR-25 | WHEN global lock is active, CUSTOM toolbars SHALL have an independent "Unlock" toggle in the dialog allowing per-toolbar override. |
| FR-26 | LOCKED toolbars SHALL occupy minimal vertical space (no handles visible). |
| FR-27 | LOCK STATE SHALL be persisted per-profile. |

### 2.7 Dividers & Separators

| ID | Requirement |
|----|-------------|
| FR-28 | THE Commands pane SHALL include a "Divider" pseudo-command that can be dragged to any toolbar. |
| FR-29 | DIVIDERS SHALL render as vertical separators between tool groups. |
| FR-30 | DIVIDERS SHALL be reorderable and removable like regular tools. |

### 2.8 Profile Integration

| ID | Requirement |
|----|-------------|
| FR-31 | ALL toolbar configuration (visibility, order, custom toolbars, Quick Access, lock state) SHALL be persisted as part of the active profile. |
| FR-32 | WHEN a profile is applied, THE application SHALL restore the complete toolbar configuration. |
| FR-33 | WHEN the user saves a profile, THE application SHALL include current toolbar state. |
| FR-34 | PREDEFINED profiles SHALL define default toolbar configurations for their workflow. |

### 2.9 Persistence & Reset

| ID | Requirement |
|----|-------------|
| FR-35 | TOOLBAR state SHALL be saved to profile on every change (debounced 500ms). |
| FR-36 | WHEN the user clicks "Reset to Defaults" in the dialog, THE application SHALL restore the active profile's factory toolbar configuration. |
| FR-37 | WHEN the user clicks "Reset All Profiles", THE application SHALL restore all profiles to factory defaults (with confirmation). |

---

## 3. Non-Functional Requirements

| ID | Requirement | Category |
|----|-------------|----------|
| NFR-01 | Dialog SHALL open in < 200ms | Performance |
| NFR-02 | Drag-drop operations SHALL be smooth at 60fps | Performance |
| NFR-03 | Dialog SHALL be keyboard accessible (Tab order, Arrow keys, Enter, Delete, Escape) | Accessibility |
| NFR-04 | All toolbar changes SHALL apply immediately to main UI (live preview) | Usability |
| NFR-05 | Custom toolbar names SHALL be unique (case-insensitive), max 64 chars | Data Integrity |
| NFR-06 | Maximum 20 custom toolbars per profile | Usability |
| NFR-06 | Toolbar state SHALL serialize to profile JSON without loss | Reliability |
| NFR-07 | Dialog SHALL remember last selected category/toolbar per session | Usability |
| NFR-08 | Search/filter in Commands pane (Ctrl+F) | Usability |

---

## 4. Acceptance Criteria (Given/When/Then)

### 4.1 Dialog Opening & Navigation

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-01: Open dialog | App running | User presses Ctrl+Shift+T | Dialog opens with 4 panes, Categories focused, all 7 categories listed |
| AC-02: Category filter | Dialog open, "Markup Tools" checked | User unchecks "Markup Tools" | Commands pane hides all markup tools |
| AC-03: Multi-category | Dialog open | User checks "Markup Tools" + "Measurement Tools" | Commands shows both categories' tools |
| AC-04: Keyboard nav | Dialog open | User presses Tab → Right Arrow → Enter | Focus moves: Categories → Commands → Toolbars → Items; Enter selects item |
| AC-05: Search filter | Dialog open | User presses Ctrl+F, types "calib" | Commands filters to "Calibrate Scale" only |

### 4.2 Adding/Removing Tools

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-06: Add tool | Dialog open, Main Toolbar selected in Toolbars | User drags "Area Measurement" from Commands to Items | Area tool appears in Main Toolbar at drop position; Main UI updates live |
| AC-07: Remove tool | Dialog open, "Area Measurement" in Main Toolbar Items | User selects it and presses Delete | Area tool removed from Main Toolbar; Main UI updates live |
| AC-08: Reorder tools | Dialog open, multiple tools in Main Toolbar Items | User drags "Length" above "Perimeter" | Tool order changes in Main Toolbar; Main UI updates live |
| AC-09: Add divider | Dialog open, Main Toolbar selected | User drags "Divider" from Commands to Items between tools | Vertical separator appears in Main Toolbar |
| AC-10: Remove divider | Dialog open, Divider in Main Toolbar Items | User selects divider and presses Delete | Divider removed |

### 4.3 Toolbar Visibility & Creation

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-11: Hide toolbar | Dialog open, "Quick Access Area" checked | User unchecks "Quick Access Area" | Quick Access Area disappears from main UI immediately |
| AC-12: Show toolbar | Dialog open, "Properties Toolbar" unchecked | User checks "Properties Toolbar" | Properties Toolbar appears (auto-show on selection) |
| AC-13: Create custom toolbar | Dialog open | User clicks "New Toolbar" → names "Redaction Tools" → adds Redact/Sanitize/Bates | "Redaction Tools" appears in Toolbars list, can be docked/floated |
| AC-14: Rename custom toolbar | Custom toolbar "Redaction Tools" exists | User selects it → clicks Rename → "Redaction v2" | Toolbar renamed in list and UI |
| AC-15: Delete custom toolbar | Custom toolbar "Redaction Tools" exists | User selects it → clicks Delete → confirms | Toolbar removed from list and UI; tools return to Commands |
| AC-16: Cannot delete predefined | Dialog open | User selects "Main Toolbar" → clicks Delete | Delete button disabled; no action |

### 4.4 Quick Access Area

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-17: Add to Quick Access | Dialog open, Quick Access Area selected | User drags "Stamp Presets" to Items | Stamp Presets appears in Quick Access Area (top-right) |
| AC-18: Quick Access grows | 3 tools in Quick Access | User adds 4th tool | Area expands horizontally to fit |
| AC-19: Quick Access reorder | 5 tools in Quick Access | User drags tool to new position | Order changes in Quick Access Area |
| AC-20: Quick Access divider | Quick Access has tools | User drags Divider between tools | Visual separator appears in Quick Access Area |

### 4.5 Properties Toolbar

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-21: Properties fixed | Dialog open, Properties Toolbar selected | User tries to drag command to Properties Items | Drop rejected; Properties structure fixed |
| AC-22: Properties auto-show | No selection, Properties Toolbar hidden | User clicks a rectangle markup | Properties Toolbar appears showing color, line weight, opacity, fill |
| AC-23: Properties hide | Markup selected, Properties visible | User clicks empty canvas | Properties Toolbar hides |
| AC-24: Properties toggle | Dialog open | User unchecks Properties Toolbar | Properties Toolbar disabled (won't auto-show) |

### 4.5 Toolbar Lock

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-25: Global lock | Toolbars unlocked | User clicks "Lock Toolbars" | All handles hidden; toolbars cannot be moved; "Lock Toolbars" changes to "Unlock Toolbars" |
| AC-26: Custom override | Global lock active, custom toolbar "Redaction Tools" | User unchecks "Lock" for "Redaction Tools" in dialog | "Redaction Tools" shows handle, can be moved; others remain locked |
| AC-27: Lock per-profile | "Review" profile has lock enabled | User switches to "Markup" profile (lock disabled) | Toolbar lock state changes with profile |

### 4.6 Profile Integration

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-28: Profile saves toolbar | User customized Main Toolbar | User saves profile "My Review" | Profile includes toolbar visibility, order, Quick Access, lock state |
| AC-29: Profile restores toolbar | "My Review" profile has custom toolbar config | User applies "My Review" profile | Main Toolbar, Quick Access, custom toolbars, lock state all restored |
| AC-30: Predefined profile toolbar | "Measurement" profile applied | Profile applied | Measurement tools prominent in Main Toolbar; Calibrate in Quick Access |

### 4.7 Reset & Edge Cases

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-31: Reset to defaults | User modified toolbars | User clicks "Reset to Defaults" → confirms | Active profile's toolbar config restored to factory |
| AC-32: Max custom toolbars | 20 custom toolbars exist | User tries to create 21st | Error: "Maximum 20 custom toolbars reached" |
| AC-33: Duplicate toolbar name | "Redaction Tools" exists | User creates new toolbar named "redaction tools" | Error: "Toolbar name already exists (case-insensitive)" |
| AC-34: Drag to empty space | Dialog open | User drags tool to empty toolbar area → creates new toolbar | Prompts for name, creates custom toolbar with that tool |

---

## 5. Error Handling Table

| Error Condition | Detection | User Message | Recovery |
|-----------------|-----------|--------------|----------|
| Duplicate toolbar name | Case-insensitive name match | "Toolbar 'X' already exists. Choose a different name." | Focus name field, highlight |
| Max custom toolbars (20) | Count >= 20 | "Maximum 20 custom toolbars reached. Delete unused toolbars first." | Disable New Toolbar button |
| Invalid toolbar name | Empty, >64 chars, reserved words | "Invalid toolbar name. Use 1-64 characters." | Focus name field |
| Drag-drop invalid target | Drop on non-toolbar area | No action (visual reject) | Tool returns to source |
| Profile toolbar corrupt | Deserialization failure | "Toolbar config in profile 'X' invalid. Resetting to defaults." | Reset profile toolbar to factory |
| Quick Access overflow | >12 tools added | "Quick Access limited to 12 tools. Remove tools to add more." | Block addition, highlight area |

---

## 6. Implementation TODO Checklist

### 6.1 Data Model (`src/domain/toolbar.rs`)

- [ ] `ToolbarConfig` — visibility, lock_state, tools: `Vec<ToolbarItem>`
- [ ] `ToolbarItem` enum: `Tool(CommandId)`, `Divider`, `Spacer`
- [ ] `CommandId` — unique identifier for each command (string enum)
- [ ] `CommandCategory` — 7 categories with `Vec<CommandId>`
- [ ] `COMMAND_REGISTRY` — static map: `CommandId` → `CommandInfo` (icon, name, shortcut, category)
- [ ] `QuickAccessConfig` — `Vec<CommandId>` (5-12 items)
- [ ] `ToolbarLockState` — `global_locked: bool`, `custom_overrides: HashMap<String, bool>`
- [ ] Serde serialization for profile integration
- [ ] Unit tests: serialization, validation, registry lookup

### 6.2 Command Registry (`src/domain/commands.rs`)

- [ ] Define all ~60 commands with IDs, names, icons, shortcuts, categories
- [ ] `CommandInfo` struct: `id`, `name`, `icon`, `shortcut`, `category`, `description`
- [ ] `COMMANDS_BY_CATEGORY: HashMap<Category, Vec<CommandId>>`
- [ ] `COMMAND_REGISTRY: HashMap<CommandId, CommandInfo>`
- [ ] Search index for Ctrl+F filtering
- [ ] Unit tests: registry completeness, category membership, shortcut uniqueness

### 6.3 Customization Dialog (`src/ui/toolbar_customize_dialog.rs`)

- [ ] Modal dialog component with 4-pane layout (CSS Grid / egui layout)
- [ ] **Categories Pane**: Checkbox list, multi-select, scrollable
- [ ] **Commands Pane**: Virtualized list (egui `ScrollArea`), draggable items, search filter (Ctrl+F)
- [ ] **Toolbars Pane**: List with checkbox (visibility), lock icon, type badge (Predefined/Custom), buttons (New, Rename, Delete)
- [ ] **Items Pane**: Reorderable list (egui `DragAndDrop`), shows tool icon + name, Delete key support
- [ ] Drag-drop between Commands → Items, Items → Items (reorder), Items → Commands (remove)
- [ ] Divider pseudo-command in Commands (always available)
- [ ] Live preview: changes apply immediately to main UI
- [ ] Keyboard navigation: Tab panes, Arrows within, Enter activate, Delete remove, Escape close
- [ ] "Reset to Defaults" button with confirmation
- [ ] "Lock Toolbars" global toggle + per-custom-toolbar override checkboxes

### 6.4 Toolbar Runtime (`src/ui/toolbar_runtime.rs`)

- [ ] `ToolbarManager` — builds egui toolbars from `ToolbarConfig`
- [ ] Main Toolbar rendering (top, horizontal, handles when unlocked)
- [ ] Quick Access Area rendering (top-right, flexible width, dividers)
- [ ] Properties Toolbar rendering (below main, auto-show/hide)
- [ ] Custom Toolbar rendering (dockable: top/left/right/bottom, floatable)
- [ ] Handle rendering (......) when unlocked
- [ ] Global lock: hide handles, disable drag
- [ ] Per-custom-toolbar lock override
- [ ] Toolbar state persistence hook (debounced save to profile)

### 6.5 Profile Integration (`src/data/profile_store.rs` + `toolbar`)

- [ ] Add `toolbars: ToolbarProfileConfig` to `Profile` struct
- [ ] `ToolbarProfileConfig` — Main Toolbar, Quick Access, Properties, Custom Toolbars, Lock State
- [ ] Serialize/deserialize with schema version
- [ ] Migration: v1→v2 adds Quick Access config, v2→v3 adds custom toolbar lock overrides
- [ ] Apply profile: `ToolbarManager::apply_profile_config(&config)`

### 6.6 Predefined Profile Toolbar Configs

- [ ] **Default** — Balanced: Selection, Markup, Measurement, Review, Edit groups
- [ ] **Review** — Status, Comments, Author Filter prominent; Measurement hidden
- [ ] **Measurement** — Length, Area, Perimeter, Calibrate, Units prominent; Markup hidden
- [ ] **Markup** — Stamps, Shapes, Ink, Text Box prominent; Measurement hidden
- [ ] Embedded in binary, written to disk on first run

### 6.7 Settings Integration

- [ ] Add `ToolbarConfig` to `AppSettings` (active profile's toolbar state)
- [ ] Persist in `settings.json` for immediate restore on startup
- [ ] Sync with profile on profile switch

### 6.8 Tests

- [ ] Unit: Command registry completeness, category membership
- [ ] Unit: ToolbarConfig serialization, validation
- [ ] Unit: Drag-drop logic (add, remove, reorder, divider)
- [ ] Integration: Dialog opens, all panes populated, live preview works
- [ ] Integration: Profile switch restores toolbar config
- [ ] Integration: Global lock + custom override
- [ ] Integration: Quick Access 5-12 tools, dividers
- [ ] E2E: User creates custom toolbar, adds tools, saves profile, restarts, profile loads
- [ ] Accessibility: Keyboard nav, screen reader labels, focus management
- [ ] Performance: Dialog open < 200ms, drag 60fps

### 6.9 Documentation

- [ ] `docs/user-guide/toolbar-customization.md` — User guide with screenshots
- [ ] `docs/toolbar-command-reference.md` — Complete command list with IDs
- [ ] Update `CHANGELOG.md`
- [ ] Update `ROADMAP.md` — mark F-608 complete

---

## 7. Dependencies & Risks

### 7.1 Dependencies
| Dependency | Status |
|------------|--------|
| Profile System (F-606) | Required for per-profile toolbar persistence |
| Panel Docking (F-607) | Custom toolbars need dockable panel infrastructure |
| egui DragAndDrop API | Required for dialog drag-drop |
| Animation system | Smooth toolbar transitions |

### 7.2 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| egui DragAndDrop limitations | Medium | High | Prototype early; fallback to button-based reorder |
| Command registry maintenance | Low | Medium | Code-gen from single source; tests validate completeness |
| Live preview performance | Low | Medium | Debounce UI updates; virtualize Commands list |
| Profile toolbar migration complexity | Medium | Medium | Versioned schema; comprehensive migration tests |

---

## 8. Open Questions for Review

1. **Dialog placement:** Modal centered vs docked right panel?
2. **Command icons:** Use existing markup tool icons or dedicated toolbar icons?
3. **Toolbar density:** Compact/Comfortable/Touch modes affect toolbar rendering?
4. **Context menu:** Right-click toolbar → Customize... (Bluebeam pattern)?
5. **Toolbar export:** "Export Toolbar Layout" as separate .rubric-toolbar file?

---

## 9. Approval

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Product Owner | | | |
| Tech Lead | | | |
| UX Designer | | | |

---

*Specification version 1.0 — Created 2026-11-25 using feature-forge workflow*