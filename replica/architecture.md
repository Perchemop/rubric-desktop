# Architecture — Rubber PDF Markup Application

## 1. Stack Selection (Rust + egui/eframe)

Following guide.md II.30 and the Rubric project's Rust-first approach:

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Language** | Rust 1.85 | Native performance, memory safety, rich ecosystem |
| **GUI** | egui + eframe + wgpu | Immediate-mode, cross-platform desktop; II.30.4: adopt only when user-rearrangeable panels are a product goal |
| **Windowing** | winit + eframe | eframe provides cross-platform window management |
| **Docking** | egui_dock | D-UI-1 (DECISIONS.md): adopted for user-rearrangeable panels |
| **Persistence** | serde_json | JSON format for human-readable `.rubric.json` files |
| **Data storage** | In-memory with file export | Document-centric app; no database required |
| **PDF rendering** | hayro (pure Rust) or pdfium (bundled) | II.27.12 gap G6: PDF rendering is core functionality |
| **File system** | Native Rust std::fs + directories crate | Cross-platform user directory access |
| **Crash reporting** | Shared `crates/crash-report` crate | Suite infrastructure per guide §7 |
| **Update checking** | Shared `crates/update-check` crate | GitHub Releases API integration |

**One database: NONE** — This is a document-centric desktop app. Data is stored in JSON files, not a database.

**Access rules**: All document operations are user-specific by default (the user's document). No multi-tenancy or row-level security needed.

## 2. Domain Model

### Core Types (in `src/domain/`)

```rust
// Document structure
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub file_path: Option<PathBuf>,          // PDF path if imported
    pub pages: Vec<Page>,
    pub markups: Vec<Markup>,
    pub takeoff: TakeoffSettings,            // D-UI-3
}

pub struct Page {
    pub id: Uuid,
    pub number: usize,
    pub width_points: f64,
    pub height_points: f64,
    pub rotation: PageRotation,
}

pub enum MarkupKind {
    Length(RelativeMarkup),
    Area(AreaMarkup),
    Perimeter(PerimeterMarkup),
    Diameter(DiameterMarkup),
    Radius(RadiusMarkup),
    Count(CountMarkup),
    // ... additional markup types
}

pub struct Markup {
    pub id: Uuid,
    pub kind: MarkupKind,
    pub vertices: Vec<[f32; 2]>,  // Screen coordinates in points
    pub page_id: Uuid,
    pub condition_id: Option<Uuid>, // Takeoff integration
    pub style: StylePreset,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

// Takeoff domain (from specs/takeoff-quantities.spec.md)
pub struct TakeoffSettings {
    pub conditions: Vec<TakeoffCondition>,
}

pub enum ConditionKind { Linear, Area, Perimeter, WallSurface, Count }
```

### Workspace State (from specs/ui-parity-revamp.spec.md)

```rust
pub enum PanelId { Pages, Tools, ToolChest, Inspector, MarkupsList, Quantities }
pub enum Region { Left, Right, Bottom }

pub struct WorkspaceState {
    pub open: BTreeSet<PanelId>,
    pub region: BTreeMap<PanelId, Region>,
    pub floating: BTreeMap<PanelId, FloatState>,
    pub split: SplitState,
}
```

## 3. API (Internal Rust APIs)

### Document Operations

| Function | Input | Output | Flow |
|----------|-------|--------|------|
| `Document::new()` | name | new Document with empty state | New document workflow |
| `Document::load(path)` | Path to .rubric.json | Document | Import/Retrospective |
| `Document::save(&self)` | &self | IO Result | Autosave + manual save |
| `Document::add_markup(&mut self, markup)` | markup | &mut Markup | Creation flow |
| `Document::apply_operator(&mut self, operator)` | Operator enum | Result<()> | Measurement tools |

### Workspace Operations

| Function | Input | Output | Flow |
|----------|-------|--------|------|
| `WorkspaceState::open_panel(&mut self, id)` | PanelId | bool (success) | Docking flow |
| `WorkspaceState::close_panel(&mut self, id)` | PanelId | bool (success) | Docking flow |
| `WorkspaceState::set_split(&mut self, count, ratio)` | u8, f32 | | Split view flow |
| `AppPrefs::load()` | config path | Self | Startup |
| `AppPrefs::save(&self)` | &self | IO Result<()> | Shutdown |

## 4. Components That Bite

- **PDF coordinate systems**: PDF is origin-bottom-left, screen is origin-top-left. Need `to_screen_rotated()` mapping per page.
- **DPI/scaling**: Different screens have different DPI; need to store points in user units while displaying in screen coordinates.
- **Hit testing tolerance**: Rubber-band selections need tolerance; implement distance-to-polyline.
- **Undo/redo**: Implement as snapshots in `History` struct (bounded to 128 entries).
- **Floating window lifecycle**: egui_dock integration; need to handle window close events.
- **Text rendering for measurement labels**: egui's text system needs font configuration.
- **Time zones**: All timestamps stored in UTC; local display only.

## 5. Build Order (Vertical Slices First)

### v0.1.0 Vertical Slice
1. App shell (egui/eframe, basic window)
2. Document model (empty document creation)
3. Simple markup (click-to-create length measurement)
4. Save/load persistence
5. Basic PDF import (single page)

### Next Must-Haves (from spec)
1. **Docking UI** (D-UI-4) - Panels can be docked/floated
2. **Takeoff conditions** (specs/takeoff-quantities) - Measurement grouping
3. **Split views** (G5) - Dual canvas comparison
4. **Preferences dialog** (G6) - Workspace persistence

### Should-Haves
1. Multiple markup types (Area, Perimeter, Count)
2. Layer management
3. Measurement styles and presets
4. Tool chest with recent tools

### Could-Haves
1. Cloud sync
2. Plugin system
3. Batch operations
4. Advanced measurement tools

## 6. Key Files Structure

```
apps/rubric/src/
├── main.rs                    # Entry point
├── app.rs                     # eframe::App implementation
├── ui/
│   ├── app.rs                 # RubricApp struct
│   ├── canvas.rs              # PDF canvas with measurements
│   ├── panels/
│   │   ├── pages.rs           # Pages panel
│   │   ├── tools.rs           # Tools panel
│   │   ├── inspectors.rs      # Inspector panels
│   │   └── markups.rs         # Markups list
│   ├── dock.rs                # egui_dock integration
│   ├── shortcuts.rs           # Keyboard shortcuts
│   └── preferences.rs         # Preferences dialog
├── domain/
│   ├── mod.rs
│   ├── document.rs            # Document model
│   ├── markup.rs              # Markup types
│   ├── page.rs                # Page model
│   ├── workspace.rs           # Workspace state
│   └── takeoff.rs             # Takeoff conditions
├── data/
│   └── prefs.rs               # AppPrefs, crash report
└── ui/
    └── ...                    # Additional UI code
```

## 7. Three Riskiest Parts

1. **PDFium bundling** (G6) - Need to bundle PDFium binaries for all platforms while handling cross-platform build complexity
2. **Docking state persistence** (D-UI-3) - Complex serialization of floating window positions and dock arrangements
3. **Measurement accuracy** - Converting between PDF coordinates, screen coordinates, and user measurement units

## Next Step

**`/replica-design`** — Rebuild the design system as tokens for the egui application:
- Color roles (surface.base, text.primary, accent, etc.)
- Spacing scale (2-4-8-12-16-24-32-48)
- Radius system (2/4/8/12)
- Typography scale
- Component specs for panels, buttons, dialogs