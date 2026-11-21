# T1: Panel Consolidation - Tabbed Left Panel

**Epic:** UI-REDIRECT  
**Type:** Story  
**Story Points:** 3  
**Status:** Ready

## Description
Consolidate the three left-side panels (Pages, Tools, Tool Chest) into a single tabbed container to reduce visual clutter and improve screen real estate utilization.

## Acceptance Criteria

```
Given a user opens Rubric with no document
When the user views the left sidebar
Then only a "Panels" tab shows with Pages, Tools, and Tool Chest as tab options
And all three can be toggled within that single panel
```

```
Given a user has the Pages panel open
When the user clicks the "Tool Chest" tab
Then the Panel content changes to show Tool Chest
And the Pages content is hidden but not destroyed
```

```
Given a user resizes the window from wide to narrow (≤1200px)
When the left panel is visible
Then Pages, Tools, and Tool Chest merge into a single tabbed container
```

## Implementation Steps

1. [ ] Create `LeftPanel` enum in `app.rs`:
   ```rust
   pub(crate) enum LeftPanel { Pages, Tools, ToolChest }
   ```

2. [ ] Refactor `app.rs:left_panel()` to use tabbed container with `PickNth` strategy

3. [ ] Add panel access bar icons on edge triggers for consolidated panel

4. [ ] Add auto-hide behavior for left panel (hide all when `focus_mode` active)

5. [ ] Update `UiState` to track active left panel tab

## Files to Modify
- `apps/rubric/src/ui/app.rs` - Main panel container
- `apps/rubric/src/ui/app_pages.rs` - Pages panel content
- `apps/rubric/src/ui/app_tool_chest.rs` - Tool Chest content
- `apps/rubric/src/ui/mod.rs` - Module declarations

## Dependencies
- `theme.rs:ELEVATION_1` panel frame styling

## Risks
- Tab switching may cause state reset in children
- Edge trigger may conflict with existing click targets
