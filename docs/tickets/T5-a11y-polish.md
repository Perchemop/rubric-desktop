# T5: Polish & Accessibility - A11y and Keyboard

**Epic:** UI-REDIRECT  
**Type:** Task  
**Story Points:** 3  
**Status:** Ready

## Description
Implement accessibility improvements and keyboard navigation for the redesigned UI, ensuring WCAG compliance and efficient keyboard workflows.

## Acceptance Criteria

```
Given a user does not have a mouse
When they press Tab through the interface
Then all panel toggle shortcuts are reachable
And focus moves logically between elements
```

```
Given a user with color vision deficiency
When viewing the interface
Then all UI elements have sufficient contrast (≥4.5:1)
And panel access bar icons are distinguishable without color alone
```

## Implementation Steps

1. [ ] Add keyboard shortcuts for panel toggles:
   - `Ctrl+1` - Toggle left panel
   - `Ctrl+2` - Toggle right panel
   - `Ctrl+3` - Toggle bottom panel

2. [ ] Implement edge-trigger accessibility labels:
   - `aria-label="Show Left Panels"`
   - `aria-label="Show Properties Panel"`

3. [ ] Audit and fix contrast for panel access bar icons

4. [ ] Add panel transition animations (smooth slide)

5. [ ] Implement focus trapping for modal dialogs spawned from panels

## Files to Modify
- `apps/rubric/src/ui/app.rs` - Keyboard shortcuts
- `apps/rubric/src/ui/theme.rs` - Contrast checks
- `apps/rubric/src/ui/mod.rs` - Accessibility imports

## Dependencies
- `theme.rs` elevation tokens
- `SPACING_*` tokens

## Risks
- Animation may cause issues for users with motion sensitivity
- Need to test with screen readers
