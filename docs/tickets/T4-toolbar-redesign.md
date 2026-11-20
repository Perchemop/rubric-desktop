# T4: Toolbar Redesign - Tools Dropdown

**Epic:** UI-REDIRECT  
**Type:** Task  
**Story Points:** 2  
**Status:** Ready

## Description
Refactor the toolbar from inline icon strip to a cleaner design with active tool chip, zoom display, and page navigation, plus a "Tools" dropdown for all functions.

## Acceptance Criteria

```
Given Rubric is running with no document
When the user views the top toolbar
Then the toolbar shows only: Active tool chip | Zoom % | Page navigation (◀ ▶)
And no inline tool icons
```

```
Given the user clicks the "Tools" dropdown button
When the dropdown opens
Then it shows the top 5 tools with labels
And a "Show All Tools" submenu for remaining tools
```

## Implementation Steps

1. [ ] Refactor `app.rs:top_bar()` to show compact toolbar

2. [ ] Keep existing active tool chip implementation

3. [ ] Add zoom display (e.g., "150%") to right side

4. [ ] Add page navigation arrows (◀ ▶) with page number

5. [ ] Implement Tools dropdown using `draw_tool_dropdown()` pattern

6. [ ] Move status message to bottom panel or tooltip

## Files to Modify
- `apps/rubric/src/ui/app_topbar.rs` - Toolbar refactor
- `apps/rubric/src/ui/app.rs` - Toolbar integration
- `apps/rubric/src/ui/toolbar.rs` - Dropdown logic

## Dependencies
- `draw_tool_dropdown()` function in toolbar.rs

## Risks
- Users may lose quick access to less-frequent tools
- Need to determine "top 5" most used tools
