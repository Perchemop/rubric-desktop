# Accessibility Patterns Research for Rubric

**Source:** Clean-room analysis of WCAG 2.1 AA requirements  
**Last updated:** 2026-11-28  
**Research method:** Original behavioral summaries + public URLs only

## 1. Keyboard Navigation (SC 2.1.1, 2.1.2, 2.4.3, 2.5.7/2.5.2)

### Requirements
- **SC 2.1.1**: All functionality must be available via keyboard
- **SC 2.1.2**: No keyboard trap - focus can be moved away from all elements
- **SC 2.4.3**: Focus order should follow meaningful sequence
- **SC 2.5.7**: Dragging movements can be aborted
- **SC 2.5.2**: Pointer cancellation via Up event

### Rubric Today / Could Inspire

| Pattern | Requirement | Rubric Status | Inspiration |
|---------|-------------|---------------|-------------|
| Tool shortcuts | Single-key shortcuts | ✅ Implemented (1, 2, 3 keys) | Adobe Acrobat, Bluebeam Revu |
| Shortcut turn-off | G217/F99 violation | ⚠️ Gap | Windows Settings, macOS System Preferences |
| Dialog navigation | Tab sequence within | ✅ Implemented | egui best practices |

**Gap Identified:** SC 2.1.4 (Single-key shortcuts) - no turn-off or remapping mechanism for power users who need alternative input methods.

**URL References:**
- https://www.w3.org/WAI/WCAG21/Understanding/keyboard.html
- https://www.w3.org/WAI/WCAG21/Understanding/character-key-shortcuts.html

## 2. Screen Reader Semantics (SC 4.1.2, 4.1.3, 1.1.1)

### Requirements
- **SC 4.1.2**: Name, Role, Value for all user interface components
- **SC 4.1.3**: Status messages announced
- **SC 1.1.1**: Non-text content has text alternative

### Rubric Today / Could Inspire

| Pattern | Requirement | Rubric Status | Inspiration |
|---------|-------------|---------------|-------------|
| Widget labels | egui AccessKit `labelled_by` | ✅ Implemented | Bluebeam's accessibility tests |
| State announcements | `WidgetInfo` for icon-only | ✅ Implemented | nvda, JAWS testing |

**Implementation Notes:**
Using egui 0.37.0 AccessKit integration:
- `Response::labelled_by()` for icon-only controls
- `widget_info(WidgetInfo::labeled(...))` for state
- Error messages in `egui::TextStyle::MONOITORY`

**URL References:**
- https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html
- https://www.w3.org/WAI/WCAG21/Understanding/status-messages.html

## 3. Color Contrast (SC 1.4.3, 1.4.11, 1.4.12)

### Requirements
- **1.4.3**: 4.5:1 contrast ratio for normal text, 3:1 for large text
- **1.4.11**: 3:1 for non-text graphics and UI components
- **1.4.12**: 3:1 for textSpacing adjustments

### Rubric Today / Could Inspire

| Pattern | Contrast Ratio | Rubric Status | Notes |
|---------|---------------|---------------|-------|
| Body text | 4.5:1 | ✅ AAA compliant | Uses `weak_text_color()` |
| Large text | 3:1 | ✅ Exceeds |  |
| Focus rings | 3:1 | ✅ Implemented | `focus_ring` token |
| High contrast mode | 7:1 AAA | ✅ Option | System preference aware |

**Implementation:**
- `contrast(a, b) = (L1 + 0.05) / (L2 + 0.05)` where L = 0.2126R + 0.7152G + 0.0722B (linearized sRGB)
- Headless `audit()` tests verify every color token
- Contrast checked for light, dark, and high-contrast modes

**URL References:**
- https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html
- https://www.w3.org/TR/WCAG21/#contrast-enhanced

## 4. Target Size (SC 2.5.5, 2.5.8)

### Requirements
- **2.5.5**: Target size 44×44 CSS pixels (enhanced)
- **2.5.8**: Target size 24×24 with spacing ( AAA )
- Exception: "Essential" elements like canvas handles

### Rubric Today / Could Inspire

| Pattern | Size | Rubric Status | Notes |
|---------|------|---------------|-------|
| Toolbar buttons | 44×44 | ✅ Implemented | Matches Android guidelines |
| Canvas handles | 8px (zoom-scaled) | ✅ Compliant | Uses hit-slop pattern |
| Spacing circles | 24×24 + spacing | ✅ Implemented | Touch target exceeds visible size |

**Implementation:**
- Hit testing uses tolerance (not just bounding box)
- Target radius = `radius + TOUCH_TOLERANCE`
- Canvas handles use `slab_rect().radius()` scaled by zoom

**URL References:**
- https://www.w3.org/WAI/WCAG21/Understanding/target-size.html
- https://www.w3.org/WAI/WCAG21/Understanding/target-size-enhanced.html
- https://developer.android.com/design/build-blocks/buttons

## 5. Reduced Motion (SC 2.3.3)

### Requirements
- Respect user's reduced motion preference
- Prefer subtle animations or none at all

### Rubric Today / Could Inspire

| Pattern | Implementation | Rubric Status | Notes |
|---------|---------------|---------------|-------|
| Animation duration | ~120ms | ✅ Quick | Not distracting |
| User preference | System setting aware | ⚠️ Partial | Need explicit toggle |
| Canvas render | Direct render | ✅ No animation |  |

**Gap:** No explicit in-app reduced motion toggle for users who haven't configured OS-level preferences.

**URL References:**
- https://www.w3.org/WAI/WCAG21/Understanding/animation.html
- https://developer.mozilla.org/en-US/docs/Web/CSS/prefers-reduced-motion

## 6. Error Handling (SC 3.3.1, 3.3.3, 3.3.4)

### Requirements
- **3.3.1**: Specific error identification
- **3.3.3**: Error suggestion
- **3.3.4**: Error prevention for legal/financial

### Rubric Today / Could Inspire

| Pattern | Implementation | Rubric Status | Notes |
|---------|---------------|---------------|-------|
| Inline validation | Text field validation | ✅ Implemented | "Invalid PDF path" messages |
| Error summary | Modal with all errors | ✅ Implemented | Focus return after close |
| Prevention | Undo/Redo | ✅ Implemented | Command history pattern |

**Inspiration:** WAI-ARIA alert dialogs and in-line validation patterns from Google Forms and Microsoft Forms.

**URL References:**
- https://www.w3.org/WAI/WCAG21/Understanding/error-identification.html
- https://www.w3.org/WAI/WCAG21/Understanding/error-suggestion.html

## 7. Empty States

### Requirements
- Provide clear information about the current state
- Offer actionable next steps
- Maintain consistent visual design

### Rubric Today / Could Inspire

| State | Implementation | Rubric Status | Notes |
|-------|---------------|---------------|-------|
| Markups list | "No markups yet…" | ✅ Implemented | + Clear CTA |
| Canvas (initial) | Placeholder text | ⚠️ Gap | Needs CTA |
| Quantities panel | Empty state message | ⚠️ Gap | Needs design |

**Inspiration:** Empty states from Material Design and Apple Human Interface Guidelines.

**URL References:**
- https://m2.material.io/design/content/placeholder.html
- https://developer.apple.com/design/human-interface-guidelines/foundations/empty-states

## 8. Focus Management

### Requirements
- Visible focus indicator (at least 2px or 3:1 contrast)
- Logical focus order
- Focus containment in modals/dialogs
- Focus return after dialog close

### Rubric Today / Could Inspire

| Pattern | Implementation | Rubric Status | Notes |
|---------|---------------|---------------|-------|
| Focus ring | 3px, 3:1 contrast | ✅ Implemented | `focus_ring` token |
| Tab trap | Dialogs contain focus | ✅ Implemented | egui built-in |
| Focus return | Esc exits modal | ⚠️ Gap | No explicit return after close |

**Gap:** Dialog focus return to triggering element (keyboard navigation pattern).

**URL References:**
- https://www.w3.org/WAI/WCAG21/Understanding/focus-visible.html
- https://www.w3.org/WAI/ARIA/apg/practices/model-dialog/

## Summary Table

| WCAG SC | Requirement | Rubric Status | Priority |
|---------|-------------|---------------|----------|
| 2.1.1 | Keyboard accessible | ✅ Done | Low |
| 2.1.2 | No keyboard trap | ✅ Done | Low |
| 2.1.4 | Keyboard shortcut alternatives | ❌ Gap | Medium |
| 2.4.3 | Focus order | ✅ Done | Low |
| 2.5.2 | Pointer cancellation | ✅ Done | Low |
| 2.5.5 | Target size enhanced | ✅ Done | Low |
| 2.5.8 | Target size AAA | ✅ Done | Low |
| 2.3.3 | Reduced motion | ⚠️ Partial | Low |
| 1.4.3 | Text contrast | ✅ Done | Low |
| 1.4.11 | Non-text contrast | ✅ Done | Low |
| 4.1.2 | Name, role, value | ✅ Done | Low |
| 3.3.1 | Error identification | ✅ Done | Low |
| 3.3.3 | Error suggestion | ✅ Done | Low |

## Next Steps

1. Add preference toggle for reduced motion in Settings
2. Implement focus return after profile rename dialog
3. Add keyboard shortcut customization/remapping
4. Add empty states for canvas (first run) and quantities panel