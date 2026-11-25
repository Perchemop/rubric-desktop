# Inspiration Apps — Deep UI/UX Pattern Research

**Status:** Research Complete  
**Date:** 2026-11-25  
**Purpose:** Granular analysis of settings, sub-settings, toolbars, sizing, resizing, and UI patterns in inspiration apps to inform Rubric's design decisions  
**Apps Analyzed:** Bluebeam Revu, Adobe Acrobat, Stirling PDF, PDF-XChange Editor, General commercial PDF suites

---

## Executive Summary

This document provides exhaustive research into the UI/UX patterns of four major PDF application categories. Each app demonstrates distinct approaches to toolbar management, panel systems, settings organization, workspace customization, and responsive sizing. These patterns will directly inform Rubric's F-6xx (UI/UX polish) features and future phases.

**Key Cross-App Patterns:**
1. **Profile/Workspace system** — All major apps support saving/restoring complete UI layouts
2. **Dual toolbar types** — Static toolbars (always visible) + dynamic properties/context toolbars
3. **Panel docking/splitting** — Panels can dock left/right/bottom, float, or split into tab groups
4. **Customization via dialog** — Dedicated "Customize Toolbars" dialogs with command categorization
5. **Keyboard-driven workflows** — Extensive shortcuts for power users
6. **Theme support** — Light/dark/system themes with customizable colors/fonts
7. **Touch vs. mouse modes** — Separate UI densities for different input methods

---

## 1. Bluebeam Revu — Detailed Analysis

### 1.1 Architecture Overview

**Core UI Components:**
- **Menu Bar** — Top-level application menus
- **Toolbars** — Configurable tool collections with drag handles (......)
- **Properties Toolbar** — Dynamic, context-sensitive toolbar showing properties of active selection
- **Side Panels** — Vertical pull-outs (left/right) for in-depth features
- **Panel Access Bar** — Icon strip for opening panels, can auto-hide
- **Main Workspace** — Document viewing area
- **Navigation Bar** — Page navigation, view controls, profile switching (F4 to toggle)
- **Status Bar** — Context-sensitive info, cursor coords, snap/grid toggles (F8 to toggle)
- **Bottom Panel** — Expandable area for Markups List, 3D Model Tree, JS Console

### 1.2 Toolbar System (Granular Detail)

**Toolbar Characteristics:**
- Each toolbar has a **handle** (...... dots) for dragging
- Toolbars can be moved to any **toolstrip** (left, right, top of workspace)
- Dragging to empty space creates new toolstrip row
- Toolbars and panels use **separate toolstrips** — never mix

**Customization Features:**
| Feature | Implementation |
|---------|----------------|
| Show/Hide | Tools > Toolbars > [checkmark list] |
| Move | Drag handle to any toolstrip |
| Lock All | Tools > Toolbars > Lock (removes handles, saves space) |
| Customize Dialog | Tools > Toolbars > Customize... |
| Save to Profile | Revu > Profiles > Save Profile |

**Customize Toolbars Dialog:**
- **Categories** — Filter commands by category
- **Commands** — Individual draggable commands
- **Toolbar List** — Existing toolbars with show/hide checkboxes
- **Items** — Commands in selected toolbar (reorder with up/down arrows)
- **Operations:** Add tool (drag from Commands → Items), Remove tool, Reorder, Create new toolbar

**Properties Toolbar (Special):**
- **Dynamic** — Changes based on active selection
- **Not customizable** — Automatic behavior
- **Located below Menu Bar** — Always visible when something selected
- **Shows:** Appearance properties (color, line weight, opacity, font, etc.)

### 1.3 Panel System (Granular Detail)

**Panel vs Toolbar Distinction:**
- Panels = in-depth features with special interfaces
- Toolbars = specific functions, no special interface needed
- **Panels have icons in Panel Access Bar** (left side by default)
- **Toolbars have handles (......) and go in toolstrips**

**Panel Customization:**
| Operation | Method |
|-----------|--------|
| Show/Hide | Window > Panels > [dropdown with shortcuts] |
| Move | Drag panel icon to left/right/bottom |
| Float | Drag to workspace center → floating panel |
| Split | Drag panel icon over open panel → 4-arrow wheel → drop on arrow |
| Tab Group | Drag additional panel icons into floating panel |
| Detach | Right-click panel icon > Attach > Detach |
| Hide | Right-click panel icon > Hide |

**Built-in Panels:**
- 3D Model Tree / Properties
- Bookmarks / Search
- File Access / Sets
- Flags / Signatures
- Forms / Spaces
- Layers / Studio
- Links / Thumbnails
- **Markups / Tool Chest** (critical for Rubric)
- Measurements

### 1.4 Profile System (Workspace Persistence)

**Preloaded Profiles:**
| Profile | Purpose |
|---------|---------|
| Essentials | New users, focused introduction |
| Revu | Basic streamlined interface, most users |
| Revu Advanced | Robust interface, power users |
| Quantity Takeoff | Estimators, specifiers, takeoffs |
| Field Issues | Punch process workflows |

**Profile Management:**
- **Change:** Revu > Profiles > [select]
- **Manage:** Revu > Profiles > Manage Profiles (rename, add, import, export, delete)
- **Create Custom:** Arrange UI → Manage Profiles > Add > Name
- **Save Changes:** Profiles > Save Profile (also auto-saves on close)
- **Shared Profiles:** Network-shared, local modifications saved as copies
- **File Format:** `.bpx` files, double-click to import

**What Profiles Store:**
- Toolbar visibility, position, content
- Panel visibility, position, split state
- Menu customizations
- Display settings
- Tool sets (Tool Chest contents)

### 1.5 Settings/Preferences Organization

**Preferences Dialog Categories:**
- General
- Documents
- Full Screen
- Page Display
- 3D & Multimedia
- Measurement (rulers, grids, guides)
- Performance (memory, threads)
- Registration
- Customize UI (fonts, colors, backgrounds, overlay shadows)

**Keyboard Shortcuts:**
- F4 = Toggle Navigation Bar
- F8 = Toggle Status Bar
- F11 = Full Screen
- Ctrl+; = Show/Hide Guides
- Panel-specific shortcuts shown in Window > Panels menu

### 1.6 Sizing/Resizing Behaviors

**Panel Resizing:**
- Hover over panel edge → cursor highlights blue → click+drag to resize
- Side and bottom panels independently resizable
- Split panels have independent sizing

**Toolbar Density:**
- **Lock Toolbars** removes handles, compacts space
- **Touch Mode** (tablet): Larger icons, spaced apart
- **Classic Toolbars** option for legacy layout

**Workspace Maximization:**
- Auto-hide Panel Access Bar (right-click > Auto-Hide)
- Full Screen mode (F11)
- MultiView / MultiView Extended for multi-doc

---

## 2. Adobe Acrobat — Detailed Analysis

### 2.1 Architecture Overview

**Core UI Components (Modern Acrobat):**
- **Hamburger Menu** (Windows) / App Menu (macOS) — Top-left, all app functions
- **Quick Action Toolbar** — Top, customizable frequent tools
- **Common Tools Toolbar** — Navigation, zoom, page display, select & zoom
- **Right Side Panel** — Collapsible, customizable tool panels
- **Left Navigation Pane** — Bookmarks, thumbnails, attachments, etc.
- **Document Pane** — Main viewing area
- **Properties/Format Bar** — Context-sensitive (appears when editing)

### 2.2 Toolbar System (Granular Detail)

**Toolbar Types:**
1. **Quick Action Toolbar** — User's most-used tools, top-right
2. **Common Tools Toolbar** — Navigation, zoom, view controls
3. **Quick Tools** — Subset of Common Tools, right side
4. **Custom Tool Sets** — User-created tool collections

**Quick Action Toolbar Customization:**
| Action | Method |
|--------|--------|
| Open Customize | Right-click toolbar > Customize Quick Tools / View > Show/Hide > Customize Toolbar |
| Add Tool | Select from right pane > Add to Toolbar |
| Remove Tool | Select from toolbar list > Remove from Toolbar |
| Reorder | Select tool > Up/Down arrows |
| Group | Add Divider to Toolbar |
| Save | Select Save in dialog |

**Common Tools Customization:**
- Right-click empty toolbar space > Choose tools to show
- Categories: Page Navigation, Page Display, Select & Zoom, Rotate View, File, Edit
- Each category has "Show All" and individual tool toggles
- Reset option per category

**Custom Tool Sets (Advanced):**
- **Tools > Create Custom Tool**
- **Toolbar Tab:** Add/remove/reorder tools, add dividers
- **Custom Set Tab:** Create named panels, add instructions, add dividers
- **Save/Share:** Export tool sets, share with colleagues
- **Switch:** Customize button > Choose tool set

### 2.3 Panel System (Granular Detail)

**Side Panels (Right):**
- Right-click panel > Customization menu
- Check/uncheck panels to show/hide
- Panels include: AI Assistant, Generative Summary, Comment, etc.
- **Reset Side Panels** to restore defaults

**Left Navigation Pane:**
- Bookmarks, Page Thumbnails, Attachments, Search, etc.
- Collapsible/expandable
- View > Show/Hide > Navigation Panes

**Panel Behavior:**
- Persistent per-document (different PDFs can have different panel states)
- State persists across sessions
- Width resizable by dragging edge

### 2.4 Settings/Preferences Organization

**Preferences Dialog (Edit > Preferences / Acrobat > Preferences):**
| Category | Key Settings |
|----------|--------------|
| General | Touch mode, basic tools, startup behavior |
| Documents | Open settings, save settings, PDF/A view mode |
| Full Screen | Navigation, transitions, escape key behavior |
| Page Display | Default layout, zoom, rendering, page transitions |
| 3D & Multimedia | 3D settings, multimedia playback |
| **Measurement** | **Units, precision, scale, grids, guides, rulers** |
| **Accessibility** | **Screen reader, high contrast, color replacement** |
| **Security** | **Enhanced security, trusted locations, certificates** |
| **Forms** | **Auto-complete, highlight fields, tab order** |
| **Signatures** | **Creation, validation, trusted identities** |
| **Identity** | **User info for comments, digital IDs** |
| **Comments** | **Font, author name, pop-up behavior** |
| **Spelling** | **Dictionary, check as you type** |
| **Units & Guides** | **Measurement units, grid spacing, guide color** |
| **Page Display** | **Default layout, zoom, rendering** |
| **Full Screen** | **Loop, escape exits, navigation** |
| **Reading** | **Thread reading order, article threads** |
| **Multimedia** | **Preferred players, accessibility** |
| **3D & Multimedia** | **Renderer, lighting, animation** |
| **JavaScript** | **Enable, console, debugger** |
| **Web Capture** | **Conversion settings** |
| **Updater** | **Auto-update, notifications** |
| **Color Management** | **Working spaces, policies, profiles** |
| **Advanced** | **Print, PDF generation, font embedding** |

### 2.5 Sizing/Resizing Behaviors

**Toolbar Density:**
- **Touch Mode** — Auto-enabled on touch devices, larger buttons, spaced apart
- **Reduce Large Icons** — Option in Customize Toolbars > Options
- **Classic UI** — Legacy toolbar layout option

**Panel Resizing:**
- Drag panel edge to resize
- Collapse to icons only (chevron toggle)
- Auto-hide not explicitly documented but implied by collapse

**Document View:**
- Single Page / Continuous / Two Page / Two Page Continuous
- Fit Width / Fit Page / Fit Visible / Custom zoom %
- Marquee Zoom (drag rectangle to zoom)

**Full Screen:** F8 to toggle, hides all UI chrome

### 2.6 Keyboard Shortcuts (Key for Power Users)

| Shortcut | Action |
|----------|--------|
| F8 | Toggle toolbars |
| F9 | Toggle menu bar (macOS: Shift+Cmd+M) |
| Ctrl+; | Show/Hide Grids |
| Ctrl+Shift+P | Print |
| Ctrl+Shift+S | Save As |
| Custom shortcuts | Via Customize Toolbars > Properties |

---

## 3. Stirling PDF — Detailed Analysis

### 3.1 Architecture Overview

**Web-based, Self-hosted Application:**
- **Left Sidebar** — File access, workspace, tool categories
- **Right Panel** — Tool options, settings, processing controls
- **Top Navigation** — Tool picker, search, user menu, settings
- **Main Workspace** — Document preview/editor area
- **Bottom Toolbar (Viewer)** — Page navigation, zoom controls (overlays content - known issue)

### 3.2 Settings System (Three Configuration Methods)

**Priority Order (Highest → Lowest):**
1. **Environment Variables** (Docker/System)
2. **Settings File** (`settings.yml`)
3. **In-App Settings** (UI, requires admin login)

**In-App Settings (Admin Only):**
- **Workspace → Usage & Billing** — License, plan, seats
- **Workspace → Users** — Invite, roles, teams, ownership
- **Server → System** — Branding, User Preference Defaults, Endpoint Management, File Storage, Folder Access, Custom Paths, Custom Metadata, Certificate Signing, AI Form Detection
- **Server → Sign-in & Security** — Login methods, lockout, SSO, PDF permissions, audit logging

**Branding & Appearance Settings:**
| Setting | Key | Default | Purpose |
|---------|-----|---------|---------|
| Navbar Brand | `ui.appNameNavbar` | "Stirling PDF" | Browser tab title, TOTP issuer |
| Show Update | `system.showUpdate` | true | Update notifications |
| Show Updates Admin Only | `system.showUpdateOnlyAdmin` | true | Restrict to admins |
| Hide Google Drive | `ui.hideDisabledTools.googleDrive` | false | Hide disabled tool buttons |
| Hide Mobile Scanner | `ui.hideDisabledTools.mobileQRScanner` | false | Hide disabled tool buttons |

**Logo Replacement:**
- Mount `customFiles/static/modern-logo/` with exact filenames:
  - `favicon.ico`, `logo192.png`, `logo512.png`
  - `StirlingPDFLogoNoTextLight.svg`, `StirlingPDFLogoNoTextDark.svg`
  - `StirlingPDFLogoBlackText.svg`, `StirlingPDFLogoWhiteText.svg`, `StirlingPDFLogoGreyText.svg`
  - `logo-tooltip.svg`

**Language & Localization:**
- Priority: User selection (localStorage) > System default > Browser auto-detect > en-US
- Configurable available languages: `ui.languages` array

### 3.3 Tool/Endpoint Management (Granular)

**Endpoint Control (Disable Features):**
```yaml
endpoints:
  toRemove:
    - sign
    - add-watermark
    - compare
    - merge-pdfs
  groupsToRemove:
    - LibreOffice
    - OCRmyPDF
```

**Available Endpoint Groups:**
| Group | Disables |
|-------|----------|
| LibreOffice | Office doc conversions (DOCX, XLSX, PPTX) |
| Python | Scan extraction, some conversions |
| OpenCV | Advanced image processing |
| OCRmyPDF | OCR features |
| Weasyprint | HTML to PDF |
| Calibre | E-book conversions |
| qpdf | PDF operations |
| DeveloperDocs | In-app dev links |

**Individual Tools (50+):**
Organized in tool picker with search, categories:
- Quick Access (top): PDF Text Editor, Page Editor, Merge, Compare, Compress, Convert, OCR, Redact
- Signing: Sign, Certify
- Document Security: Add Password, Watermark, Stamp, Sanitize, Flatten, Permissions
- Verification: Get Info, Validate Signature
- Page Formatting: Crop, Rotate, Split, Reorganize, Scale, Page Numbers, Multi-Page Layout, Booklet
- Extraction: Various extract tools

### 3.4 Viewer/Editor UI Issues (Documented)

**Known UX Issues (GitHub #6054):**
1. **Bottom toolbar overlays PDF** — `position: fixed; bottom: 0` covers footer content
2. **Top controls overlay PDF** — `position: absolute; top: 0` with 3.5rem viewport gap
3. **Missing Fit Width/Fit Page buttons** — Only +/- zoom, no preset buttons despite ZoomMode support

**Proposed Fix:**
- Move toolbars to flex layout children (not fixed/absolute)
- Add Fit Width/Fit Page buttons using existing ZoomMode
- Reduce viewportGap from 3.5rem to 0.5rem

### 3.5 Theme/CSS Customization

**Theme Settings:**
- In-App: Settings → Server → System → Branding & Appearance
- Light/Dark theme support
- Custom CSS injection via custom `index.html` (no simple CSS file drop)

**Static File Overrides:**
- Mount `customFiles:/customFiles` in Docker
- Replace any static file by matching path structure
- Fonts: `.ttf`/`.otf` in `customFiles/static/fonts/` for PDF text editor

---

## 4. PDF-XChange Editor — Detailed Analysis

### 4.1 Architecture Overview

**Dual UI Modes:**
1. **Ribbon UI (Modern)** — Tabbed ribbon interface (File, Home, View, etc.)
2. **Classic Toolbars** — Traditional toolbar layout
3. **Touch-Optimized UI** — Larger icons for tablets

**Core Components:**
- **Ribbon Tabs** — File, Home, Convert, View, Edit, Forms, etc.
- **Quick Access Toolbar** — Top-left, customizable
- **Document Options Toolbar** — Bottom-left, page layout/view options
- **Page Layout Toolbar** — Bottom-right, page display modes
- **Format Tab** — Replaces Properties toolbar, minimizes with ribbon
- **Navigation Pane** — Left side, bookmarks/thumbnails/layers/etc.
- **Main Document Area** — Center workspace

### 4.2 Toolbar/Ribbon Customization (Granular)

**Customize Toolbars Dialog:**
- **Toolbars Tab:** Show/hide ribbon tabs, Rename/Reset/Delete custom tabs, Make Default
- **Ribbon Tabs Checkboxes:** Expand to show groups, show/hide individual groups
- **Commands Tab:** All commands by category, search, drag-to-add, Properties (tooltip, shortcuts), Export to CSV
- **Options Tab:** Toolbar display/locking options

**Creating Custom Ribbon Tabs:**
1. New > New Ribbon Tab > Name
2. Drag commands from Commands tab to new ribbon
3. Create groups, add dividers, reorder

**Toolbar Operations:**
| Action | Method |
|--------|--------|
| Show/Hide | View > Toolbars > [checkboxes] |
| Move | Click drag handle (grid dots) on toolbar right side |
| Dock | Drag to compatible area → dock indicator |
| Float | Drag away from dock zones |
| Compress/Expand/Hide | Click arrow/pin on toolbar |
| Minimize Icons | Right-click toolbar > Customize > Options > "Reduce large icons" |

**Classic vs Ribbon:**
- Classic: Traditional toolbars, Properties toolbar visible
- Ribbon: Format tab replaces Properties, minimizes when unpinned
- Switch via UI Options dropdown

### 4.3 UI Appearance Settings (Preferences > Customize UI)

**Customize UI Tab:**
| Section | Options |
|---------|---------|
| Theme | Dropdown: Light/Dark/System, "Allow Dark Mode from system" |
| Fonts | Font style/size for menus, toolbars, dialogs |
| Colors | UI color customization |
| Backgrounds | Type (menu, toolbar, etc.), Style/Pattern, Show Overlay Shadows, Turn off textured backgrounds |
| Reset | Reset to Defaults |

**Other Preference Categories:**
- **Measurement** — Rulers, grids, guides, units, precision, scale
- **Performance** — Memory usage, threads
- **Page Display** — Default layout, rendering, smoothing
- **Page Layout** — Single/Continuous/Two-page modes

### 4.4 Panel System

**Navigation Pane (Left):**
- Bookmarks, Thumbnails, Layers, Comments, Attachments, Search, Signatures, etc.
- Show/Hide via View > Navigation Panes
- Tabbed interface within pane
- Width resizable

**Document Options Toolbar (Bottom-Left):**
- Page layout: Single/Continuous/Two Page/Two Page Continuous
- Dropdown for additional options

**Page Layout Toolbar (Bottom-Right):**
- Zoom controls, page navigation

### 4.5 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| F11 | Full Screen |
| Ctrl+; | Show/Hide Guides |
| Ctrl+Shift+P | Print |
| Double-click ribbon header | Expand/collapse (unpinned view) |
| Custom | Via Customize Toolbars > Commands > Properties |

### 4.6 Sizing/Resizing Behaviors

**Ribbon States:**
- **Pinned (Default)** — Full ribbon visible
- **Unpinned** — Tab headers only, click to expand temporarily
- **Touch-Optimized** — Larger icons, more spacing
- **Classic** — Traditional toolbar layout

**Toolbar Compression:**
- "Reduce large icons in toolbars" option
- Unpinned view = thinner toolbar
- Format tab minimizes with other headers

**Panel Resizing:**
- Drag navigation pane edge
- Bottom toolbars fixed height

---

## 5. Cross-App Pattern Synthesis

### 5.1 Toolbar Architecture Comparison

| Aspect | Bluebeam Revu | Adobe Acrobat | Stirling PDF | PDF-XChange |
|--------|--------------|---------------|--------------|-------------|
| **Primary Toolbar** | Multiple movable toolbars | Quick Action + Common Tools | Tool picker (sidebar) | Ribbon tabs + Quick Access |
| **Context Toolbar** | Properties Toolbar (dynamic) | Format bar (when editing) | Tool options panel | Format Tab |
| **Customization** | Customize Dialog (Categories/Commands) | Customize Quick Tools dialog | Endpoint control + Settings | Customize Toolbars (Ribbon/Commands) |
| **Density Control** | Lock (removes handles) | Touch Mode, Reduce Icons | N/A (web) | Reduce Icons, Unpin Ribbon |
| **Persistence** | Profiles (.bpx files) | Per-document + Tool Sets | Settings.yml + Env vars | Default + Custom Ribbon Tabs |
| **Creation** | New toolbar in Customize dialog | Create Custom Tool Set | N/A (admin disables) | New Ribbon Tab |

### 5.2 Panel System Comparison

| Aspect | Bluebeam Revu | Adobe Acrobat | Stirling PDF | PDF-XChange |
|--------|--------------|---------------|--------------|-------------|
| **Location** | Left/Right/Bottom dock | Left nav + Right panels | Left sidebar + Right panel | Left Navigation Pane |
| **Docking** | Drag to edges, split, float | Right-click customize | Fixed layout | Fixed left pane |
| **Tab Groups** | Drag panels together | N/A | Tool categories | Tabbed within pane |
| **Auto-hide** | Panel Access Bar auto-hide | Collapse to icons | N/A | N/A |
| **Persistence** | Saved in Profiles | Per-document | User preferences | Workspace layout |

### 5.3 Settings Organization Comparison

| Aspect | Bluebeam Revu | Adobe Acrobat | Stirling PDF | PDF-XChange |
|--------|--------------|---------------|--------------|-------------|
| **Access** | Revu > Preferences | Edit > Preferences | Settings (admin) + settings.yml | Edit > Preferences |
| **Categories** | ~10 top-level | 25+ categories | Server/Workspace/System | ~15 categories |
| **UI Customization** | Customize UI tab (fonts/colors) | Themes, Touch mode | Branding, Themes, CSS | Customize UI tab (theme/fonts/colors) |
| **Measurement** | Dedicated category | Units & Guides | Tool-specific | Measurement category |
| **Profiles/Workspaces** | Full Profile system | Tool Sets + Per-doc | Settings.yml + Env vars | Default + Custom Ribbon |

### 5.4 Workspace Persistence Patterns

| Pattern | Apps Using | Implementation |
|---------|------------|----------------|
| **Full Profile/Workspace** | Bluebeam, PDF-XChange | Save all UI state (toolbars, panels, menus) to named profile |
| **Tool Sets** | Adobe Acrobat | Save toolbar + panel config as named set, switchable |
| **Config Files** | Stirling PDF | YAML + Env vars, admin UI, file-based |
| **Per-Document State** | Adobe Acrobat | Each PDF remembers its panel/toolbar config |
| **Auto-Save on Close** | Bluebeam, PDF-XChange | Profile auto-updated on app exit |

---

## 6. Implications for Rubric Design

### 6.1 Current Rubric State (from specs)

**Implemented (F-6xx):**
- F-601: Keyboard shortcuts (Ctrl+S/Z/Y/Esc/Del, per-tool keys)
- F-602: Dark theme (light/dark toggle, persisted)
- F-603: Context menus (canvas + inspector)
- F-604: Undo/redo (128 snapshots)
- F-605: Accessibility (WCAG contrast, keyboard reachable, tooltips)

**UI Components (from code):**
- Top toolbar (app_topbar.rs) — Tools, zoom, page nav
- Left panel (app_menu.rs) — Layers, Tool Chest, Thumbnails
- Right panel (Inspector) — Properties, Markups List, Layers
- Bottom panel — Status bar, measurement readouts
- Canvas — Main workspace with rulers (F-406), grid (F-405)

### 6.2 Recommended Enhancements (Priority Order)

#### P0 — Critical (Missing Core Patterns)

**1. Profile/Workspace System**
- Save/restore complete UI layout (panel visibility, positions, splits, toolbar config)
- Predefined profiles: "Review", "Measurement", "Markup", "Minimal"
- Import/export profiles (.rubric-profile JSON)
- Auto-save on close option

**2. Toolbar Customization Dialog**
- Categories → Commands → Toolbar Items (like Bluebeam/PDF-XChange)
- Show/hide toolbars, reorder tools, add dividers
- Create custom toolbars
- Lock toolbars option

**3. Panel Docking/Splitting**
- Drag panel tabs to left/right/bottom edges
- Split panels (horizontal/vertical) with drag-wheel indicator
- Float panels as independent windows
- Panel access bar with auto-hide

#### P1 — High (Power User Features)

**4. Properties/Context Toolbar**
- Dynamic toolbar showing properties of selected markup
- Appears below main toolbar when markup selected
- Color, line weight, opacity, font, fill, etc.

**5. Custom Tool Sets**
- User-defined tool collections (like Acrobat Custom Tools)
- Save/load/share tool sets
- Quick-switch between sets

**6. Advanced Settings Dialog**
- Organized categories (General, Display, Measurement, Performance, Shortcuts)
- Theme: Light/Dark/System + custom accent color
- Fonts: UI font family/size
- Measurement: Units, precision, scale defaults, grid/ruler settings
- Performance: Render cache size, thread count

#### P2 — Medium (Polish)

**7. Touch/Mouse Density Modes**
- Compact (default), Comfortable, Touch (larger targets)
- Reduce toolbar icon size option
- Auto-detect touch device

**8. Keyboard Shortcut Customization**
- View/edit all shortcuts in Settings
- Export/import shortcut schemes
- Conflict detection

**9. Status Bar Customization**
- Show/hide: Page info, Cursor coords, Zoom %, Measurement readout, Snap/Grid toggles
- F8 toggle (Bluebeam pattern)

**10. Ruler/Guide/Grid Settings**
- Grid: Spacing, color, snap tolerance, show/hide
- Guides: Color, magnetic snap, lock guides
- Rulers: Units, origin, tick interval

### 6.3 Specific UI/UX Decisions for Rubric

#### Toolbar Architecture Decision
**Recommendation:** Hybrid approach
- **Main Toolbar** (top) — Primary tools, fixed position, customizable via dialog
- **Properties Toolbar** (below main, auto-show) — Context-sensitive markup properties
- **Quick Access Area** (top-right) — User's 5-8 most-used tools, drag-reorder
- **No Ribbon** — Too heavy for egui, stick to toolbar paradigm

#### Panel System Decision
**Recommendation:** Bluebeam-inspired
- **Left Panel** — Navigation (Thumbnails, Layers, Bookmarks)
- **Right Panel** — Inspector (Properties, Markups List, Tool Chest, Stamps)
- **Bottom Panel** — Status, Measurements, Log
- **All panels:** Dock left/right/bottom, split, float, tab-group
- **Panel Access Bar** — Left edge icons, auto-hide option

#### Settings Organization Decision
**Recommendation:** Acrobat-style categorized preferences dialog
```
Preferences Dialog:
├── General
│   ├── Startup (new doc, recent, template)
│   ├── Autosave interval
│   ├── Language
│   └── Touch/Mouse density
├── Display
│   ├── Theme (Light/Dark/System/Custom)
│   ├── UI Font (family, size)
│   ├── Accent color
│   └── Icon style (outline/filled)
├── Measurement
│   ├── Default units (mm/cm/in/ft)
│   ├── Precision (decimals)
│   ├── Default scale presets
│   ├── Grid (spacing, color, snap)
│   ├── Guides (color, magnetic)
│   └── Rulers (units, origin)
├── Performance
│   ├── Render cache size
│   ├── Thread count
│   ├── Thumbnail quality
│   └── PDFium vs Hayro preference
├── Shortcuts
│   ├── View all / search
│   ├── Edit (with conflict detection)
│   ├── Export/Import scheme
│   └── Reset to defaults
└── Advanced
    ├── Profile management
    ├── Toolbar customization
    ├── Panel layout reset
    └── Debug logging
```

#### Profile System Decision
**Recommendation:** Bluebeam-style Profiles
- **File format:** `.rubric-profile` (JSON with schema version)
- **Contains:** Panel visibility/positions/splits, Toolbar config, Shortcut overrides, Theme override, Measurement defaults
- **Predefined:** Default, Review, Measurement, Markup, Minimal, Presentation
- **Management:** Preferences > Profiles (Save, Manage, Import, Export)
- **Auto-save:** Optional "Update profile on close"

---

## 7. Detailed Feature Specifications (EARS Format)

### 7.1 Profile System

```
WHEN the user selects a profile from the Profiles menu,
THE application SHALL apply the complete UI layout (panel visibility, positions, splits, toolbar configuration, shortcut overrides, theme override, measurement defaults) within 500ms.

WHEN the user chooses "Save Profile",
THE application SHALL serialize current UI state to a .rubric-profile file with schema version.

WHEN the application starts,
THE application SHALL load the last-used profile or "Default" if none selected.

WHEN the user enables "Auto-save profile on close",
THE application SHALL update the active profile with current UI state on graceful shutdown.
```

### 7.2 Toolbar Customization

```
WHEN the user opens the Customize Toolbars dialog,
THE application SHALL display all commands categorized by function, all existing toolbars with show/hide checkboxes, and the items in the selected toolbar.

WHEN the user drags a command from the Commands list to a toolbar's Items list,
THE application SHALL add the tool to that toolbar at the drop position.

WHEN the user clicks "Lock Toolbars",
THE application SHALL hide all toolbar handles and prevent toolbar movement until unlocked.

WHEN the user creates a new toolbar,
THE application SHALL add it to the toolbar list and allow naming and population.
```

### 7.3 Panel Docking/Splitting

```
WHEN the user drags a panel tab to a workspace edge (left/right/bottom),
THE application SHALL dock the panel to that edge, creating a new panel strip if none exists.

WHEN the user drags a panel tab over an existing docked panel,
THE application SHALL display a four-zone drop indicator (split left/right/top/bottom) and split the panel on drop.

WHEN the user drags a panel tab to the workspace center,
THE application SHALL create a floating panel window with its own title bar and close button.

WHEN the user drags a panel tab into an existing floating panel,
THE application SHALL add it as a new tab in that floating panel's tab group.
```

### 7.4 Properties Toolbar

```
WHEN a markup is selected,
THE application SHALL show the Properties Toolbar below the main toolbar within 100ms.

WHEN the selection changes,
THE application SHALL update the Properties Toolbar to show properties of the new selection.

WHEN no markup is selected,
THE application SHALL hide the Properties Toolbar.

THE Properties Toolbar SHALL show: color, line weight, opacity, fill, font family/size, line style, arrow style (context-dependent).
```

### 7.5 Settings Dialog

```
WHEN the user opens Preferences,
THE application SHALL display a categorized dialog with searchable settings.

WHEN the user changes a setting,
THE application SHALL apply it immediately (for UI settings) or on next restart (for performance settings).

WHEN the user clicks "Reset to Defaults" in a category,
THE application SHALL restore all settings in that category to built-in defaults.

THE application SHALL persist all settings to user config file on change.
```

---

## 8. Acceptance Criteria (Given/When/Then)

### 8.1 Profile System

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Switch Profile | App running, "Review" profile exists | User selects "Review" from Profiles menu | Panels rearrange to Review layout, toolbars update, theme may change |
| Save Profile | User has custom panel layout | User chooses Profiles > Save Profile > "My Layout" | .rubric-profile created, appears in Profiles menu |
| Export Profile | "My Layout" profile exists | User chooses Manage Profiles > Export > "My Layout" | .rubric-profile file saved to chosen location |
| Import Profile | Valid .rubric-profile file exists | User chooses Manage Profiles > Import > selects file | Profile appears in list, can be activated |
| Auto-save | "Auto-save on close" enabled | User closes app gracefully | Active profile updated with current UI state |

### 8.2 Toolbar Customization

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Add Tool | Customize dialog open, "Stamp" tool in Commands | User drags "Stamp" to Main Toolbar Items | Stamp tool appears in Main Toolbar at drop position |
| Remove Tool | Customize dialog open, "Stamp" in Main Toolbar Items | User selects "Stamp" and clicks Remove | Stamp removed from Main Toolbar |
| Reorder Tools | Customize dialog open, multiple tools in Items | User drags tool up/down in Items list | Tool order changes in actual toolbar |
| Create Toolbar | Customize dialog open | User clicks New Toolbar > names "My Tools" > adds tools | New toolbar appears in UI, can be docked |
| Lock Toolbars | Toolbars unlocked | User chooses Tools > Toolbars > Lock | Handles hidden, toolbars cannot be moved |

### 8.3 Panel Docking/Splitting

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Dock Left | Markups List panel floating | User drags Markups List tab to left edge | Panel docks to left strip, shows vertically |
| Split Horizontal | Layers panel docked left | User drags Thumbnails tab over Layers > drops on bottom zone | Layers top, Thumbnails bottom, splitter between |
| Float Panel | Properties panel docked right | User drags Properties tab to canvas center | Floating window with Properties tab, closable |
| Tab Group | Floating panel with Markups List | User drags Thumbnails tab into floating panel | Floating panel now has two tabs: Markups List, Thumbnails |
| Auto-hide | Panel Access Bar visible | User right-clicks Panel Access Bar > Auto-Hide | Bar hides, reappears on hover |

### 8.4 Properties Toolbar

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Show on Select | No selection, Properties Toolbar hidden | User clicks a rectangle markup | Properties Toolbar appears showing color, line weight, fill, opacity |
| Update on Re-select | Rectangle selected, Properties showing | User clicks a text markup | Properties updates to show font, size, color, alignment |
| Hide on Deselect | Markup selected, Properties visible | User clicks empty canvas | Properties Toolbar hides |
| Multi-select | Two rectangles selected | User Shift-clicks second rectangle | Properties shows common properties, mixed values indicated |

---

## 9. Implementation Roadmap

### Phase 6.1 — Profile System (2-3 weeks)
- [ ] Design `.rubric-profile` JSON schema
- [ ] Implement profile serialization/deserialization
- [ ] Create Profiles menu with Save/Manage/Import/Export
- [ ] Build predefined profiles (Default, Review, Measurement, Markup, Minimal)
- [ ] Add auto-save on close preference
- [ ] Unit tests for profile round-trip

### Phase 6.2 — Toolbar Customization (2-3 weeks)
- [ ] Design command registry (categories, commands, toolbars)
- [ ] Build Customize Toolbars dialog (Categories/Commands/Toolbars/Items)
- [ ] Implement drag-drop reordering
- [ ] Add toolbar lock/unlock
- [ ] Persist toolbar config in profile
- [ ] Keyboard shortcut for Customize dialog

### Phase 6.3 — Panel Docking/Splitting (3-4 weeks)
- [ ] Implement panel drag-drop with drop zones
- [ ] Add split panel support (horizontal/vertical)
- [ ] Implement floating panels
- [ ] Add tab grouping in panels
- [ ] Panel Access Bar with auto-hide
- [ ] Persist panel layout in profile

### Phase 6.4 — Properties Toolbar (1-2 weeks)
- [ ] Design context-sensitive property definitions per markup type
- [ ] Build dynamic Properties Toolbar component
- [ ] Implement show/hide logic based on selection
- [ ] Add property editors (color picker, line weight, etc.)

### Phase 6.5 — Settings Dialog (2-3 weeks)
- [ ] Build categorized Preferences dialog
- [ ] Implement live preview for UI settings (theme, font, density)
- [ ] Add measurement/grid/guide/ruler settings
- [ ] Performance settings with validation
- [ ] Shortcut editor with conflict detection

### Phase 6.6 — Touch/Mouse Density & Polish (1-2 weeks)
- [ ] Three density modes (Compact/Comfortable/Touch)
- [ ] Auto-detect touch device
- [ ] Reduce icon size option
- [ ] Status bar customization
- [ ] Final accessibility audit

---

## 10. Open Questions & Decisions Needed

| Question | Options | Recommendation |
|----------|---------|----------------|
| Profile file location? | App config dir / Project dir / User choice | App config dir (`~/.config/rubric/profiles/`) |
| Max profiles? | Unlimited / 20 / 50 | 50 (prevents clutter) |
| Toolbar customization granularity? | Per-tool / Per-group only | Per-tool (max flexibility) |
| Panel split state in profile? | Yes / No | Yes (critical for workflow) |
| Properties toolbar always visible? | Auto-show only / Always / User choice | Auto-show (saves space) |
| Settings dialog: modal or panel? | Modal dialog / Right panel | Modal dialog (standard pattern) |
| Shortcut conflicts: allow or prevent? | Allow with warning / Prevent | Prevent with clear error |
| Density mode persistence? | Per-profile / Global | Per-profile (workspace-specific) |

---

## 11. Appendix: Raw Research Notes

### 11.1 Bluebeam Revu Key URLs
- Customize Toolbars: https://support.bluebeam.com/revu/how-to/customize-toolbars.html
- Customize Panels: https://support.bluebeam.com/revu/how-to/customize-panels.html
- Customize Revu Intro: https://support.bluebeam.com/revu/how-to/customize-revu-introduction.html
- Work with Profiles: https://support.bluebeam.com/user-manual/menus/revu/profiles.html
- Revu Interface: https://support.bluebeam.com/user-manual/welcome/revu-interface.html
- Tools and Toolbars: https://support.bluebeam.com/user-manual/toolbars/toolbars.html
- Navigation/Status Bars: https://support.bluebeam.com/revu/how-to/customize-navigation-and-status-bars.html

### 11.2 Adobe Acrobat Key URLs
- Viewing Preferences: https://helpx.adobe.com/acrobat/using/viewing-pdfs-viewing-preferences.html
- Customize Side Panels: https://helpx.adobe.com/acrobat/desktop/get-started/preferences-and-settings/customize-side-panels.html
- Workspace Basics: https://helpx.adobe.com/acrobat/current/workspace-basics.html
- Customize Quick Action Toolbar: https://helpx.adobe.com/acrobat/desktop/get-started/preferences-and-settings/customize-toolbar.html
- Learn: Customize Toolbar: https://www.adobe.com/learn/acrobat/web/customize-toolbar.html

### 11.3 Stirling PDF Key URLs
- UI Customisation: https://docs.stirlingpdf.com/Configuration/Customisation/UI%20Customisation/
- Branding: https://docs.stirlingpdf.com/Server/Customisation/Branding/
- Configuration Guide: https://docs.stirlingpdf.com/Configuration/
- Endpoint Customisation: https://docs.stirlingpdf.com/Configuration/Customisation/Endpoint%20or%20Feature%20Customisation/
- All Tools: https://docs.stirlingpdf.com/Tools/
- Viewer Toolbar Issue: https://github.com/Stirling-Tools/Stirling-PDF/issues/6054

### 11.4 PDF-XChange Editor Key URLs
- Customize Workspace: https://help.pdf-xchange.com/pdfxp11/customizing-the-workspace.html
- Customize Toolbars: https://help.pdf-xchange.com/pdfxp10/customize-toolbars_ed.html
- Customize UI: https://help.pdf-xchange.com/pdfxp7/customize-ui_ed.html
- Operations Guide: https://help.pdf-xchange.com/pdfxe8c/customize-the-workspace_ed.html

---

## 12. Next Steps

1. **Review this document** with stakeholders
2. **Prioritize phases** based on user feedback and resource availability
3. **Create detailed specs** for each phase using feature-forge workflow
4. **Begin implementation** with Profile System (foundational for other features)
5. **Regular design reviews** using hallmark/impeccable criteria

---

*Document prepared for Rubric project UI/UX enhancement planning. All research based on publicly available documentation and capability-level analysis per clean-room policy.*