# Panel Docking & Splitting — Feature Specification

**Feature ID:** F-607 (extends F-6xx UI/UX polish)  
**Priority:** P0 — Core workspace flexibility, integrates with Profile System  
**Phase:** 6.3 (3-4 weeks)  
**Status:** Draft — Pending Review

---

## 1. Overview & User Value

### 1.1 Problem Statement
Professional PDF workflows require simultaneous access to multiple panels: a reviewer needs Thumbnails + Markups List + Properties visible simultaneously; an estimator needs Layers + Tool Chest + Measurements; a markup author needs Stamps + Properties + Layers. Rubric's current fixed panel layout forces users to constantly open/close panels via menu, wasting time and breaking focus.

### 1.2 Solution
A **Panel Docking & Splitting System** following Bluebeam Revu's model:
- **Edge Docking:** Drag panel tab to left/right/bottom edge → docks in strip
- **Split Wheel:** Drag panel tab over existing docked panel → 4-zone drop indicator (split left/right/top/bottom)
- **Tab Groups:** Drag panel tab into existing docked panel → adds as tab
- **Floating Panels:** Drag to canvas center → independent window (windowed or overlay mode)
- **Panel Access Bar:** Left-edge icon strip with auto-hide on hover
- **Per-Profile Persistence:** Complete panel layout saved/restored with profiles

### 1.3 User Value
- **Reviewers** → Thumbnails (left) + Markups List (bottom) + Properties (right) simultaneously
- **Estimators** → Layers (left) + Tool Chest (left tab) + Measurements (right) + Properties (right tab)
- **Markup Authors** → Stamps (left) + Properties (right) + Layers (right tab) — maximum canvas
- **Presenters** → Floating Markups List on second monitor; main window clean
- **Power Users** → Recursive splits for 3+ panel grids (e.g., Thumbnails | Layers | Properties)

### 1.4 Success Metrics
| Metric | Target |
|------|--------|
| Dock operation latency | < 100ms visual feedback |
| Split operation latency | < 150ms visual feedback |
| Panel layout restore (profile) | < 500ms |
| Multi-panel adoption | > 50% of active users use ≥2 docked panels |
| Floating panel usage | > 20% of multi-monitor users |
| Support tickets for "lost panel" | < 2% of current volume |

---

## 2. Functional Requirements (EARS Format)

### 2.1 Panel Inventory

**All Panels (Dockable):**
| Panel ID | Title | Default Location | Content |
|----------|-------|------------------|---------|
| `thumbnails` | Page Thumbnails | Left | Page previews, click to jump |
| `layers` | Layers | Left (tab) | Layer visibility, lock, order |
| `tool_chest` | Tool Chest | Left (tab) | Stamp presets, custom tools |
| `markups_list` | Markups List | Bottom | All markups with status, author, page |
| `properties` | Properties | Right | Selected markup properties (color, line weight, etc.) |
| `stamps` | Stamps | Right (tab) | Stamp library, recent stamps |
| `measurements` | Measurements | Right (tab) | Measurement list, calibration, units |

### 2.2 Edge Docking

| ID | Requirement |
|----|-------------|
| FR-01 | WHEN the user drags a panel tab to the **left edge** of the workspace, THE application SHALL dock the panel in the left strip, creating the strip if none exists. |
| FR-02 | WHEN the user drags a panel tab to the **right edge**, THE application SHALL dock the panel in the right strip. |
| FR-03 | WHEN the user drags a panel tab to the **bottom edge**, THE application SHALL dock the panel in the bottom strip. |
| FR-04 | WHEN a panel is docked to an empty strip, THE application SHALL make it the sole panel in that strip (no tab bar if alone). |
| FR-05 | WHEN a panel is docked to a strip with existing panels, THE application SHALL add it as a new tab in the tab group. |
| FR-06 | THE left strip SHALL support vertical stacking (panels top-to-bottom). |
| FR-07 | THE right strip SHALL support vertical stacking. |
| FR-08 | THE bottom strip SHALL support horizontal arrangement (panels left-to-right). |

### 2.3 Split Wheel (Panel Splitting)

| ID | Requirement |
|----|-------------|
| FR-09 | WHEN the user drags a panel tab **over an existing docked panel**, THE application SHALL display a **4-zone drop indicator** (split left, split right, split top, split bottom) centered on the target panel. |
| FR-10 | WHEN the user drops on **left zone**, THE application SHALL split the target panel vertically, placing the dragged panel to the left. |
| FR-11 | WHEN the user drops on **right zone**, THE application SHALL split vertically, placing the dragged panel to the right. |
| FR-12 | WHEN the user drops on **top zone**, THE application SHALL split horizontally, placing the dragged panel above. |
| FR-13 | WHEN the user drops on **bottom zone**, THE application SHALL split horizontally, placing the dragged panel below. |
| FR-14 | SPLIT panels SHALL have independent sizing (draggable divider between them). |
| FR-15 | RECURSIVE SPLITS SHALL be supported: a split panel can be split again, creating 3+ panel grids. |
| FR-16 | EACH split pane SHALL maintain its own tab group (can accept additional tabbed panels). |

### 2.4 Tab Groups

| ID | Requirement |
|----|-------------|
| FR-17 | WHEN the user drags a panel tab **into an existing docked panel's tab area**, THE application SHALL add it as a new tab in that panel's tab group. |
| FR-18 | TAB BAR SHALL be visible only when a docked strip contains >1 panel (tabs). |
| FR-19 | TAB ORDER SHALL be reorderable by dragging tabs within the tab bar. |
| FR-20 | RIGHT-CLICK TAB SHALL show context menu: Close, Move to New Float, Duplicate, Rename (custom panels only). |
| FR-21 | KEYBOARD SHORTCUT: Ctrl+Tab / Ctrl+Shift+Tab cycles tabs in focused strip. |

### 2.5 Floating Panels

| ID | Requirement |
|----|-------------|
| FR-22 | WHEN the user drags a panel tab to the **workspace center** (not over edge or panel), THE application SHALL create a **floating panel window**. |
| FR-23 | FLOATING PANELS SHALL support two modes (user choice per panel): **Windowed** (native window, title bar, taskbar entry) or **Overlay** (frameless, always-on-top, no taskbar entry). |
| FR-24 | FLOATING PANELS SHALL be resizable from edges/corners. |
| FR-25 | FLOATING PANELS SHALL remember position and size per session. |
| FR-26 | FLOATING PANELS SHALL support tab groups (drag another panel tab into floating panel → adds as tab). |
| FR-27 | RIGHT-CLICK FLOATING PANEL TITLE BAR SHALL show: Windowed/Overlay toggle, Move to Dock (Left/Right/Bottom), Close. |
| FR-28 | FLOATING PANELS ON SECOND MONITOR SHALL persist position across sessions (if monitor configuration unchanged). |

### 2.6 Panel Access Bar

| ID | Requirement |
|----|-------------|
| FR-29 | THE **Panel Access Bar** SHALL be a vertical icon strip on the **left edge** of the workspace (outside dock strips). |
| FR-30 | EACH DOCKABLE PANEL SHALL have an icon in the Panel Access Bar. |
| FR-31 | CLICKING an icon SHALL toggle the panel's visibility (show if hidden, hide if visible). |
| FR-32 | WHEN **Auto-Hide** is enabled (right-click access bar → Auto-Hide), THE bar SHALL collapse to a thin strip; HOVERING SHALL expand it to full icon width. |
| FR-33 | AUTO-HIDE STATE SHALL be persisted per-profile. |
| FR-34 | PANEL ACCESS BAR ICONS SHALL show active state (highlighted) when panel is visible. |
| FR-35 | TOOLTIP ON HOVER SHALL show panel name and keyboard shortcut. |

### 2.7 Keyboard Shortcuts

| ID | Requirement |
|----|-------------|
| FR-36 | EACH PANEL SHALL have a configurable keyboard shortcut (default: Ctrl+1 Thumbnails, Ctrl+2 Layers, Ctrl+3 Tool Chest, Ctrl+4 Markups List, Ctrl+5 Properties, Ctrl+6 Stamps, Ctrl+7 Measurements). |
| FR-37 | PRESSING SHORTCUT SHALL toggle panel visibility (show if hidden, focus if visible, hide if focused). |
| FR-38 | SHORTCUTS SHALL be customizable in Preferences → Shortcuts. |
| FR-39 | PANEL FOCUS SHALL follow shortcut: hidden→show, visible→focus, focused→hide. |

### 2.8 Panel State Persistence (Per-Profile)

| ID | Requirement |
|----|-------------|
| FR-40 | COMPLETE PANEL LAYOUT SHALL be serialized to active profile: dock strips (left/right/bottom), panel assignments, tab groups, split tree, floating panels (position, size, mode), Panel Access Bar auto-hide state. |
| FR-41 | WHEN a profile is applied, THE application SHALL restore the complete panel layout within 500ms. |
| FR-42 | PANEL VISIBILITY SHALL be persisted (hidden panels stay hidden on profile load). |
| FR-43 | PANEL SIZES (split ratios, strip widths) SHALL be persisted and restored. |
| FR-44 | FLOATING PANEL POSITION/SIZE/MODE SHALL be persisted per-profile (with monitor-aware restoration). |

### 2.9 Predefined Profile Panel Layouts

| Profile | Left Strip | Right Strip | Bottom Strip | Floating |
|---------|------------|-------------|--------------|----------|
| **Default** | Thumbnails + Layers (tabs) | Properties + Stamps + Measurements (tabs) | Markups List | — |
| **Review** | Thumbnails + Bookmarks (tabs) | Properties + Markups List (tabs) | — | — |
| **Measurement** | Layers + Tool Chest (tabs) | Properties + Measurements (tabs) | — | — |
| **Markup** | Tool Chest + Stamps (tabs) | Properties + Layers (tabs) | — | — |
| **Minimal** | — (all hidden) | — | — (Status only) | — |
| **Presentation** | — | — | — | Markups List (overlay, 2nd monitor) |

---

## 3. Non-Functional Requirements

| ID | Requirement | Category |
|----|-------------|----------|
| NFR-01 | Dock/split visual feedback (drop zones) SHALL appear in < 50ms | Performance |
| NFR-02 | Panel layout restore SHALL complete in < 500ms (95th percentile) | Performance |
| NFR-03 | Split divider drag SHALL be smooth at 60fps | Performance |
| NFR-04 | Floating panel creation SHALL complete in < 200ms | Performance |
| NFR-05 | All panel operations SHALL be keyboard accessible | Accessibility |
| NFR-06 | Panel Access Bar auto-hide animation SHALL be 150ms ease-out | Polish |
| NFR-07 | Maximum recursive split depth: 4 levels (prevents absurd grids) | Usability |
| NFR-08 | Minimum panel size: 200px width / 150px height | Usability |
| NFR-09 | Floating panel position restoration SHALL handle monitor reconnection gracefully | Reliability |
| NFR-10 | Panel state SHALL serialize to profile JSON without loss | Reliability |

---

## 4. Acceptance Criteria (Given/When/Then)

### 4.1 Edge Docking

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-01: Dock left | Thumbnails panel floating | User drags Thumbnails tab to left edge | Thumbnails docks in left strip; strip created if needed |
| AC-02: Dock right | Properties panel floating | User drags Properties tab to right edge | Properties docks in right strip |
| AC-03: Dock bottom | Markups List floating | User drags Markups List tab to bottom edge | Markups List docks in bottom strip |
| AC-04: Tab group | Layers docked left, Tool Chest floating | User drags Tool Chest tab into Layers panel tab area | Tool Chest becomes tab in Layers panel; tab bar appears |
| AC-05: Tab reorder | Thumbnails + Layers tabbed left | User drags Layers tab left of Thumbnails | Tab order changes: Layers, Thumbnails |
| AC-06: Tab close | Tool Chest tabbed with Layers | User right-clicks Tool Chest tab → Close | Tool Chest hidden; Layers remains |

### 4.2 Split Wheel

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-07: Split left | Properties docked right | User drags Measurements tab over Properties → drops on left zone | Vertical split: Measurements left, Properties right; divider draggable |
| AC-08: Split right | Properties docked right | User drops on right zone | Vertical split: Properties left, Measurements right |
| AC-09: Split top | Markups List docked bottom | User drops on top zone | Horizontal split: Measurements above, Markups List below |
| AC-10: Split bottom | Markups List docked bottom | User drops on bottom zone | Horizontal split: Markups List above, Measurements below |
| AC-11: Recursive split | Properties | Measurements split vertically | User drags Stamps over Measurements → drops on right zone | Three panels: Properties | Measurements | Stamps (two vertical dividers) |
| AC-12: Split divider drag | Vertical split exists | User drags divider | Panels resize smoothly; minimum 200px enforced |

### 4.3 Floating Panels

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-13: Create float | Markups List docked bottom | User drags Markups List tab to canvas center | Floating window created with Markups List; title bar shows "Markups List" |
| AC-14: Float mode toggle | Floating Markups List (Windowed) | User right-clicks title bar → Overlay | Panel becomes frameless, always-on-top, no taskbar entry |
| AC-15: Float tab group | Floating Markups List | User drags Properties tab into floating window | Floating window now has two tabs: Markups List, Properties |
| AC-16: Float to dock | Floating Measurements | User right-clicks title bar → Move to Dock → Right | Measurements docks in right strip; floating window closes |
| AC-17: Float position persist | Floating Markups List on 2nd monitor | User closes app, reopens | Markups List floating on 2nd monitor at same position (if monitor present) |

### 4.4 Panel Access Bar

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-18: Toggle via icon | Thumbnails hidden | User clicks Thumbnails icon in Panel Access Bar | Thumbnails appears in left strip (last known position) |
| AC-19: Hide via icon | Thumbnails visible | User clicks Thumbnails icon | Thumbnails hides |
| AC-20: Auto-hide enable | Panel Access Bar visible | User right-clicks bar → Auto-Hide | Bar collapses to thin strip; expands on hover |
| AC-21: Auto-hide disable | Panel Access Bar auto-hidden | User hovers bar → expands → right-clicks → Auto-Hide (uncheck) | Bar stays expanded permanently |
| AC-22: Active state | Thumbnails visible | User views Panel Access Bar | Thumbnails icon highlighted/blue |

### 4.5 Keyboard Shortcuts

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-23: Shortcut toggle hidden | Thumbnails hidden | User presses Ctrl+1 | Thumbnails shows in left strip |
| AC-24: Shortcut toggle visible | Thumbnails visible, not focused | User presses Ctrl+1 | Thumbnails focused (keyboard navigation works) |
| AC-25: Shortcut toggle focused | Thumbnails focused | User presses Ctrl+1 | Thumbnails hides |
| AC-26: Shortcut customize | Default Ctrl+1 for Thumbnails | User changes to Ctrl+Shift+T in Preferences | Ctrl+Shift+T now toggles Thumbnails |

### 4.6 Profile Integration

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-27: Profile saves layout | User has custom 3-panel split layout | User saves profile "My Workflow" | Profile includes left/right/bottom strips, splits, tabs, floating panels, access bar state |
| AC-28: Profile restores layout | "Review" profile applied | Profile applied | Left: Thumbnails+Bookmarks tabs; Right: Properties+Markups List tabs; no bottom |
| AC-29: Float position restore | "Presentation" profile has floating Markups List on 2nd monitor | Profile applied on dual-monitor setup | Markups List floating on 2nd monitor at saved position |
| AC-30: Monitor missing | "Presentation" profile has 2nd monitor float | Profile applied on single monitor | Floating panel appears on primary monitor at relative position |

### 4.7 Predefined Profiles

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-31: Default profile | Fresh install | App starts | Left: Thumbnails+Layers tabs; Right: Properties+Stamps+Measurements tabs; Bottom: Markups List |
| AC-32: Review profile | User selects "Review" | Profile applied | Left: Thumbnails+Bookmarks; Right: Properties+Markups List; No bottom |
| AC-33: Measurement profile | User selects "Measurement" | Profile applied | Left: Layers+Tool Chest; Right: Properties+Measurements; Rulers/Grid on |
| AC-34: Markup profile | User selects "Markup" | Profile applied | Left: Tool Chest+Stamps; Right: Properties+Layers; Max canvas |
| AC-35: Minimal profile | User selects "Minimal" | Profile applied | All panels hidden; only top toolbar + status bar visible |
| AC-36: Presentation profile | User selects "Presentation" on dual monitor | Profile applied | Primary: clean canvas; Secondary: Markups List floating overlay |

---

## 5. Error Handling Table

| Error Condition | Detection | User Message | Recovery |
|-----------------|-----------|--------------|----------|
| Panel below minimum size | Split drag < 200px | Divider stops at minimum; tooltip "Minimum size reached" | Auto-snap to minimum |
| Recursive split depth exceeded | Attempt 5th level split | "Maximum split depth (4) reached" | Drop rejected, panel returns |
| Floating panel monitor lost | Profile has 2nd monitor float, monitor disconnected | "Saved monitor not found. Panel placed on primary monitor." | Auto-place on primary |
| Panel Access Bar icon missing | Panel hidden, no icon | Icon hidden from access bar | Reappears when panel shown |
| Shortcut conflict | Two panels assigned same shortcut | "Shortcut Ctrl+X already used by Y. Change one." | Highlight both in Shortcuts dialog |
| Profile layout corrupt | Deserialization failure | "Panel layout in profile 'X' invalid. Using Default." | Reset to Default layout |

---

## 6. Implementation TODO Checklist

### 6.1 Data Model (`src/domain/panel.rs`)

- [ ] `PanelId` enum: all 7 panels + extensible for plugins
- [ ] `PanelState` — visibility, dock_location, tab_group, split_tree, floating
- [ ] `DockLocation` enum: `Left`, `Right`, `Bottom`, `Floating`
- [ ] `SplitTree` — recursive structure: `Leaf(PanelId)` or `Split { orientation: Horizontal/Vertical, ratio: f32, children: [SplitTree; 2] }`
- [ ] `TabGroup` — `Vec<PanelId>` with active index
- [ ] `FloatingPanel` — `PanelId`, `position: Pos2`, `size: Vec2`, `mode: Windowed/Overlay`, `monitor: Option<MonitorId>`
- [ ] `PanelAccessBarState` — `auto_hide: bool`, `visible_icons: Vec<PanelId>`
- [ ] `PanelLayout` — complete layout serialization for profiles
- [ ] Serde with schema version
- [ ] Unit tests: split tree serialization, floating panel restore

### 6.2 Panel Manager (`src/ui/panel_manager.rs`)

- [ ] `PanelManager` — owns all panel state, handles dock/split/float operations
- [ ] `dock_panel(panel, edge)` — creates/updates dock strips
- [ ] `split_panel(target, dragged, zone)` — updates split tree
- [ ] `tab_panel(target, dragged)` — updates tab groups
- [ ] `float_panel(panel, mode)` — creates floating window
- [ ] `toggle_panel(panel)` — show/hide via access bar or shortcut
- [ ] `apply_layout(layout)` — full layout restore from profile
- [ ] `serialize_layout()` — current layout to `PanelLayout`
- [ ] Event system for UI components to react

### 6.3 Dock Strips & Split Rendering (`src/ui/dock_strip.rs`)

- [ ] `DockStrip` component — renders left/right/bottom strips
- [ ] Vertical strip (left/right): panels stacked top-to-bottom
- [ ] Horizontal strip (bottom): panels arranged left-to-right
- [ ] Tab bar rendering (only when >1 panel in strip)
- [ ] Split divider rendering (draggable, minimum size enforcement)
- [ ] Recursive split rendering (tree traversal)
- [ ] Drop zone detection for edge docking

### 6.4 Split Wheel Overlay (`src/ui/split_wheel.rs`)

- [ ] `SplitWheelOverlay` — appears on drag over docked panel
- [ ] 4-zone rendering (left, right, top, bottom) with icons
- [ ] Highlight target panel, show drop preview
- [ ] Animated appear/disappear (100ms)
- [ ] Keyboard cancel (Escape)
- [ ] Zone hit-testing for drop

### 6.5 Floating Panel Windows (`src/ui/floating_panel.rs`)

- [ ] `FloatingPanelWindow` — egui `Window` or native window (platform-specific)
- [ ] Windowed mode: native decorations, taskbar entry
- [ ] Overlay mode: frameless, always-on-top, click-through for main window
- [ ] Title bar with: panel name, mode toggle, move-to-dock menu, close
- [ ] Tab support within floating window
- [ ] Resize handles (all edges/corners)
- [ ] Position/size persistence
- [ ] Monitor-aware restoration

### 6.6 Panel Access Bar (`src/ui/panel_access_bar.rs`)

- [ ] `PanelAccessBar` component — left-edge vertical strip
- [ ] Icon buttons for each dockable panel
- [ ] Active state highlighting
- [ ] Tooltip with name + shortcut
- [ ] Auto-hide: collapsed strip (32px) → expanded on hover (48px)
- [ ] Animation: 150ms ease-out
- [ ] Right-click context menu: Auto-Hide toggle
- [ ] Keyboard navigation (Up/Down arrows, Enter to toggle)

### 6.7 Keyboard Shortcuts (`src/ui/panel_shortcuts.rs`)

- [ ] Default shortcuts: Ctrl+1..7 for 7 panels
- [ ] Shortcut registry with conflict detection
- [ ] Toggle behavior: hidden→show, visible→focus, focused→hide
- [ ] Customization in Preferences → Shortcuts
- [ ] Conflict resolution UI

### 6.8 Profile Integration

- [ ] Add `PanelLayout` to `Profile` struct
- [ ] Serialize: dock strips, split tree, tab groups, floating panels, access bar
- [ ] Deserialize: `PanelManager::apply_layout()`
- [ ] Migration: v1→v2 adds floating panel mode, v2→v3 adds recursive splits
- [ ] Predefined profile layouts embedded in binary

### 6.9 Predefined Profile Layouts

- [ ] **Default** — Left: Thumbnails+Layers; Right: Properties+Stamps+Measurements; Bottom: Markups List
- [ ] **Review** — Left: Thumbnails+Bookmarks; Right: Properties+Markups List
- [ ] **Measurement** — Left: Layers+Tool Chest; Right: Properties+Measurements; Rulers/Grid on
- [ ] **Markup** — Left: Tool Chest+Stamps; Right: Properties+Layers
- [ ] **Minimal** — All hidden
- [ ] **Presentation** — Primary clean; Secondary: Markups List floating overlay

### 6.10 Tests

- [ ] Unit: SplitTree serialization, recursive split logic
- [ ] Unit: Dock/split/float state transitions
- [ ] Unit: PanelLayout profile round-trip
- [ ] Integration: Drag-drop docking, split wheel, tab grouping
- [ ] Integration: Floating panel creation, mode toggle, dock return
- [ ] Integration: Panel Access Bar auto-hide, shortcuts
- [ ] Integration: Profile switch restores complete layout
- [ ] Integration: Monitor loss graceful handling
- [ ] E2E: User creates 3-panel split, saves profile, restarts, layout restored
- [ ] Accessibility: Keyboard nav, screen reader labels, focus management
- [ ] Performance: Layout restore < 500ms, split drag 60fps

### 6.11 Documentation

- [ ] `docs/user-guide/panel-docking.md` — User guide with animated GIFs
- [ ] `docs/panel-layout-schema.md` — JSON schema for panel layout
- [ ] Update `CHANGELOG.md`
- [ ] Update `ROADMAP.md` — mark F-607 complete

---

## 7. Dependencies & Risks

### 7.1 Dependencies
| Dependency | Status |
|------------|--------|
| Profile System (F-606) | Required for per-profile panel persistence |
| Toolbar Customization (F-608) | Toolbar state also in profile |
| egui Window/Windowed support | Required for floating panels |
| Multi-monitor detection | Required for presentation profile |

### 7.2 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| egui Window limitations for floating panels | High | High | Prototype early; consider native window for overlay mode |
| Split tree complexity | Medium | High | Comprehensive tests; visual debug overlay for dev |
| Monitor configuration changes | Medium | Medium | Store monitor EDID/hash; fallback to primary |
| Recursive split UI complexity | Low | Medium | Limit depth to 4; clear visual feedback |
| Auto-hide animation performance | Low | Low | CSS-like transition; GPU accelerated |

---

## 8. Open Questions for Review

1. **Top edge docking:** Should top edge be supported (for toolbar-like panels)?
2. **Panel collapse:** Should docked panels have a "collapse to icon" mode (like VS Code)?
3. **Split ratio persistence:** Store as percentage (0.5) or pixels? Percentage better for resize.
4. **Floating panel always-on-top:** Overlay mode should float over main window only, not system-wide?
5. **Panel Access Bar position:** Left edge only, or configurable (left/right)?

---

## 9. Approval

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Product Owner | | | |
| Tech Lead | | | |
| UX Designer | | | |

---

*Specification version 1.0 — Created 2026-11-25 using feature-forge workflow*