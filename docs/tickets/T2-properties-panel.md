# T2: Properties Panel - Context-Sensitive Sections

**Epic:** UI-REDIRECT  
**Type:** Story  
**Story Points:** 5  
**Status:** Ready

## Description
Refactor the inspector tabs into a single context-sensitive Properties panel that shows different sections based on selection type, following Bluebeam Revu's pattern.

## Acceptance Criteria

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

```
Given a user selects multiple markups
When the Properties panel is visible
Then only shared properties (subject, status, layer) are displayed
```

## Implementation Steps

1. [ ] Consolidate inspector tabs (Selection/Properties/Comments/Measure/Settings) into single panel

2. [ ] Create `PropertySection` enum:
   ```rust
   pub(crate) enum PropertySection {
       Appearance, // colors, stroke, fill, opacity
       General,    // subject, author, comments, layer, status
       Measurement // length, area, count, units
   }
   ```

3. [ ] Implement section selector logic based on `UiState.selected` and markup types

4. [ ] Add `UiState.active_property_section` field

5. [ ] Create section renderers:
   - `draw_appearance_section()` - shape/text properties
   - `draw_general_section()` - metadata
   - `draw_measurement_section()` - measure properties

## Files to Modify
- `apps/rubric/src/ui/app_inspector.rs` - Major refactor
- `apps/rubric/src/ui/app.rs` - Right panel container
- `apps/rubric/src/ui/mod.rs` - Module structure

## Dependencies
- `SPACING_*` tokens (4, 8, 12, 16) used consistently

## Risks
- Complex conditional logic for section visibility
- Need to handle mixed selection types
