# Component Specs — Rubric Design System (egui/eframe)

This document describes the UI components for the Rubric PDF markup application, targeting the egui/eframe immediate-mode GUI toolkit.

---

## Color Tokens (WCAG 2.2 AA Compliant)

| Token | Light | Dark | Purpose |
|-------|-------|------|---------|
| `surface_base` | `#f9f9fb` | `#1a1a1a` | Panel backgrounds |
| `surface_raised` | `#ebebee` | `#252525` | Elevated surfaces |
| `border_subtle` | `#d1d5db` | `#404040` | Borders, separators |
| `text_primary` | `#111827` | `#e5e7eb` | Primary text |
| `text_secondary` | `#4b5563` | `#9ca3af` | Secondary text |
| `accent_500` | `#2563eb` | `#3b82f6` | Primary actions, links |
| `accent_on` | `#ffffff` | `#000000` | Text/icons on accent |
| `danger_500` | `#dc2626` | `#f87171` | Destructive actions |
| `success_500` | `#047857` | `#4ade80` | Success messages |
| `warning_500` | `#92400e` | `#fbbf24` | Warnings, cautions |

---

## Spacing Scale (4-point system)

```
[0, 2, 4, 8, 12, 16, 24, 32, 48, 64]
```

Applied consistently:
- Panel padding: `spacing-3` (8px)
- Component gap: `spacing-4` (12px)
- Large separations: `spacing-6` (24px)

---

## Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `radius_sm` | 4px | Small buttons, chips |
| `radius_md` | 8px | Panels, dialogs, inputs |
| `radius_lg` | 12px | Modal dialogs |

---

## Typography (egui Font Definitions)

| Style | Size | Weight | Line Height | Usage |
|-------|------|--------|-------------|-------|
| `heading` | 18.0 | Medium | 24.0 | Panel titles |
| `label` | 13.0 | Regular | 20.0 | Form labels |
| `body` | 13.0 | Regular | 20.0 | Body text |
| `small` | 11.0 | Regular | 16.0 | Captions, hints |
| `button` | 13.0 | Medium | 0.0 | Button labels |

---

## Components

### Button

```rust
egui::Button::new(label)
    .heading()  // 18pt medium for primary
    .text_color(accent_on)
    .background_color(accent_500)
    .min_size(egui::Vec2::new(112.0, 40.0))  // md size
    .radius(radius_md)
```

**Variants:**
- `primary`: accent background, white text
- `secondary`: border accent, white text, transparent bg
- `tertiary`: no border, text only (accent color)
- `danger`: danger background, white text

**States:**
- `hover`: accent_hover background
- `active`: accent_active background + inset shadow
- `disabled`: text_disabled + border_subtle

### Panel (Docked)

```rust
egui::Panel::side(egui::Position::Left)
    .min_width(280.0)
    .default_width(280.0)
    .default_content_height_tab_scrolling()
```

**Features:**
- Draggable header for docking/floating
- Tab strip for multiple items
- Close button with confirmation for last panel

### SplitView (Canvas)

```rust
egui::TOP_BARS::main.show(ctx, |ui| {
    egui::Grid::two_columns("split")
        .spacing([spacing-4, spacing-4])
        .show(ui, |ui| {
            // Column 1 - Primary canvas
            // Column 2 - Secondary canvas (if active)
        });
});
```

**Behavior:**
- Vertical split with draggable divider
- `ratio` controls width proportion (clamped 0.2–0.8)
- Shared selection state across panes
- Independent zoom/pan per pane

### MeasurementLabel

```rust
let label = egui::Label::new(format!("{:.1} m²", area))
    .heading()
    .text_color(text_primary)
    .bg_color(surface_base);
```

**States:**
- `normal`: semi-transparent background
- `selected`: accent background with accent_on text
- `hover`: subtle elevation

### Toolbar

```rust
egui::Toolbar::default()
    .add(egui::Button::new("").text("🔍").tooltip("Search"))
    .add_suggester(search_input)
    .add(egui::Button::new("Save").on_click(save))
```

**Layout:**
- Primary tools left-aligned
- Status/info center
- Document controls right

### PreferencesDialog

```rust
egui::Window::new("Preferences")
    .min_size(egui::Vec2::new(400.0, 300.0))
    .collapsible(false)
    .show(ctx, |ui| {
        ui.tab_layout(...);
    });
```

**Tabs:**
1. General (theme, units, autosave)
2. Shortcuts (key binding display)
3. About (version, release notes)

---

## Visual States

| State | Appearance |
|-------|------------|
| Default | Surface base, primary text |
| Hover | Slight elevation, accent border |
| Active | Accent fill, accent-on text |
| Focus | 2px accent ring, offset 2px |
| Disabled | Text muted, no interaction |
| Error | Danger border, error icon |

---

## Animation Timing

| Type | Duration | Easing |
|------|----------|--------|
| Hover | 120ms | ease-out |
| Modal enter | 240ms | ease-out |
| Modal exit | 200ms | ease-in |

---

## Accessibility Notes

- All interactive elements have a minimum 44px touch target
- Color alone never conveys information
- Focus indicators meet 3:1 contrast ratio
- Screen reader labels provided via `aria_label`/`Role`

---

## Dark Theme Adjustments

Dark theme uses the same structure with adjusted backgrounds:
- Surfaces are darker (`surface-base-dark`)
- Text is lighter (`text-primary-dark`)
- Accent colors maintain the same semantic meaning
- Borders are subtly desaturated for reduced glare