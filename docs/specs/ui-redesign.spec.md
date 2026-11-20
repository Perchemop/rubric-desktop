# UI Redesign Specification

**Version:** 1.0  
**Status:** Draft  
**Author:** Rubric Dev  
**Date:** 2026-11-21

---

## 1. Overview

### 1.1 Purpose
This specification defines the UI redesign for the Rubric application to eliminate visual clutter and improve usability, inspired by Bluebeam Revu's panel architecture.

### 1.2 User Value
- **Reduce cognitive load** by minimizing simultaneous visible panels
- **Improve discoverability** through consistent panel access patterns  
- **Enhance screen real estate** via auto-hide and edge-trigger panels
- **Align with industry standard** (Bluebeam-style layout)

### 1.3 Target User
PDF markup, review, and measurement professionals using desktop applications on Windows, macOS, and Linux.

---

## 2. Functional Requirements (EARS Format)

### 2.1 Panel Consolidation

**REQ-001:** When the application window is resized to a narrow screen (≤1200px width), the application **shall** collapse all left-side panels (Pages, Tools, Tool Chest) into a single tabbed container.

**REQ-002:** Where panel consolidation is enabled, the application **shall** display a "Panels" tab in the left sidebar showing: Pages, Tools, and Tool Chest as separate tabs within a single panel container.

### 2.2 Properties Panel

**REQ-003:** When a single markup is selected, the application **shall** display its properties in a single right-side panel with context-sensitive sections:
- **Appearance** for shape/text markups (color, opacity, stroke, fill)
- **General** for all markups (subject, author, comments, layer, status)
- **Measurement** for measure markups (length, area, volume, count, height/width, units)

**REQ-004:** Where multiple markups are selected, the application **shall** display only the shared properties (subject, status, layer) in the Properties section.

### 2.3 Bottom Panel Tab Strip

**REQ-005:** When any bottom panel is shown, the application **shall** display a horizontal tab strip containing:
- **Markups List** (default tab) - table of all markups with filter/search
- **Quantities** - takeoff condition schedule aggregated measurements
- **Layers** - document layer visibility and management
- **Thumbnails** - page thumbnails with quick navigation

**REQ-006:** Where the Quantities tab is selected and a condition exists, the application **shall** display aggregated quantities by condition with mixed-unit detection badges.

### 2.4 Toolbar Redesign

**REQ-007:** When the application loads, the toolbar **shall** display:
- Active tool icon chip (current tool indicator)
- Zoom display (e.g., "150%")
- Page navigation (◀ ▶ arrows, page number)

**REQ-008:** When the user clicks the "Tools" dropdown button, the application **shall** show:
- Top 5 most frequently used tools as icon buttons with labels
- "Show All Tools" submenu for remaining tools grouped by family

### 2.5 Panel Access & Visibility

**REQ-009:** The application **shall** display panel access icons on the left edge of left panels and right edge of right/bottom panels.

**REQ-010:** Where the mouse cursor is positioned within 8px of a panel edge, the application **shall** highlight the edge and change cursor to double-arrow for reveal.

**REQ-011:** When the user clicks a highlighted panel edge, the application **shall** slide the panel open (if closed) or close (if open).

**REQ-012:** The application **shall** remember panel visibility state in user preferences (show/hide per panel).

---

## 3. Non-Functional Requirements

### 3.1 Performance
- Panel transitions shall complete within 100ms
- Properties panel update on selection change shall complete within 50ms
- Markups List filter refresh for 1000 markups shall complete within 200ms

### 3.2 Accessibility
- All interactive panel edges shall be keyboard-focusable via Tab navigation
- Panel toggles shall have accessible labels (e.g., aria-label="Show Markups List")
- Contrast ratio ≥ 4.5:1 for panel access bar icons

### 3.3 Visual Design
- Panel container elevation shall follow `theme.rs:ELEVATION_1` (2px offset)
- Spacing between elements shall use 4-point scale tokens: SPACING_4, SPACING_8, SPACING_12, SPACING_16
- Panel minimum width: 180px, maximum width: 40% screen width

---

## 4. Acceptance Criteria

### 4.1 Panel Consolidation AC
```
Given a user opens Rubric with no document
When the user views the left sidebar
Then only a "Panels" tab shows with Pages, Tools, and Tool Chest as tab options
And all three can be toggled within that single panel
```

### 4.2 Properties Panel AC
```
Given a user selects a Rectangle markup
When the Properties panel is visible in the right sidebar
Then only the Appearance and General sections appear
And Measurement section is hidden
```

```
Given a user selects a Perimeter measure
When the Properties panel is visible
Then Measurement section shows Length, Area, Perimeter, Units
And Height field appears for Height mode markups
```

### 4.3 Bottom Panels AC
```
Given a user opens Markups List tab
When the tab is active
Then the Markups List table displays with filter/search toolbar above
And the tab height is 40px header + remaining space for table
```

### 4.4 Panel Edge AC
```
Given no panel is visible
When the user moves cursor to the left screen edge
Then the edge highlights with 2px border
And cursor changes to double-arrow
```

---

## 5. Error Handling

| Error | Cause | Handling | Log Level |
|-------|-------|----------|-----------|
| Panel edge trigger fails | CSS/hidpi scaling issue | Fallback to manual toggle via View menu | Warning |
| Properties panel fails to update | Selection changed during async operation | Cancel pending update, block new until completion | Error |
| Bottom panel tabs fail to render | Missing tab content module | Show placeholder "Content unavailable" | Error |

---

## 6. Implementation Plan

### Phase 1: Panel Consolidation (T1) - 3 SP
- [ ] Create new `LeftPanel` enum (Pages \| Tools \| ToolChest)
- [ ] Refactor `app.rs:left_panel()` to use tabbed container
- [ ] Add panel access bar icons on edge triggers
- [ ] Add auto-hide behavior for left panel

### Phase 2: Properties Panel (T2) - 5 SP
- [ ] Consolidate inspector tabs into single panel
- [ ] Implement context-sensitive sections (Appearance/General/Measurement)
- [ ] Add section toggle logic based on selection type
- [ ] Update `UiState` to track active section

### Phase 3: Bottom Tab Strip (T3) - 4 SP
- [ ] Create unified bottom panel container
- [ ] Implement tab strip: Markups List \| Quantities \| Layers \| Thumbnails
- [ ] Wire Quantities panel into tab strip
- [ ] Add tab state persistence

### Phase 4: Toolbar (T4) - 2 SP
- [ ] Replace inline tools with "Tools" dropdown
- [ ] Add active tool chip (current implementation)
- [ ] Add zoom display + page navigation
- [ ] Consolidate status into bottom panel

### Phase 5: Polish & A11y (T5) - 3 SP
- [ ] Add keyboard shortcuts for panel toggles
- [ ] Implement edge-trigger accessibility labels
- [ ] Fix contrast for panel access bar
- [ ] Add animation for panel transitions

**Total: 17 Story Points**

---

## 7. Dependencies

- `theme.rs:ELEVATION_1` must exist and be usable for panel frames
- `SPACING_*` tokens (4, 8, 12, 16) must be defined and used consistently
- Panel access bar icons need SVG in `assets/icons/`

---

## 8. References

- [Bluebeam Revu Interface](https://support.bluebeam.com/user-manual/welcome/revu-interface.html)
- [Markups List Documentation](https://support.bluebeam.com/user-manual/menus/window/markups-list.html)
- [Properties Panel Documentation](https://support.bluebeam.com/user-manual/menus/window/file-properties.html)
- Design Audit: `docs/design-audit-2026-11-21.md`

*This specification is derived from grill-me decisions in this session.*