# T3: Bottom Tab Strip - Unified Markups/Quantities

**Epic:** UI-REDIRECT  
**Type:** Story  
**Story Points:** 4  
**Status:** Ready

## Description
Create a unified bottom panel tab strip that combines Markups List, Quantities, Layers, and Thumbnails into a single tabbed container following Bluebeam's pattern.

## Acceptance Criteria

```
Given a user opens the bottom panel
When the panel is expanded
Then a tab strip shows: Markups List | Quantities | Layers | Thumbnails
And Markups List is the default selected tab
```

```
Given a user has the Quantities tab selected
When conditions exist with measurements
Then the tab shows aggregated quantities by condition with mixed-unit detection
And row click selects that condition's markups on the page
```

## Implementation Steps

1. [ ] Create `BottomPanelTab` enum:
   ```rust
   pub(crate) enum BottomPanelTab { Markups, Quantities, Layers, Thumbnails }
   ```

2. [ ] Create unified bottom panel container with tab strip

3. [ ] Refactor `app_quantities.rs` to work with tabbed container

4. [ ] Wire Markups List panel into tab strip

5. [ ] Add Layers tab panel (placeholder or minimal implementation)

6. [ ] Add Thumbnails tab panel

## Files to Modify
- `apps/rubric/src/ui/app.rs` - Bottom panel container
- `apps/rubric/src/ui/app_quantities.rs` - Quantities integration
- `apps/rubric/src/ui/app_markups_list.rs` - Tab integration
- `apps/rubric/src/ui/app_pages.rs` - Extract thumbnails logic

## Dependencies
- `app_quantities.rs` quantities panel (T6 work)
- `markups_list` module

## Risks
- Tab state persistence across sessions
- Performance with large markups lists
