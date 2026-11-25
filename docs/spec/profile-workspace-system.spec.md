# Profile/Workspace System — Feature Specification

**Feature ID:** F-606 (extends F-6xx UI/UX polish)  
**Priority:** P0 — Foundational for toolbar customization & panel docking  
**Phase:** 6.1 (2-3 weeks)  
**Status:** Draft — Pending Review

---

## 1. Overview & User Value

### 1.1 Problem Statement
Professional PDF reviewers and estimators work in distinct modes: reviewing markups, taking calibrated measurements, creating markup stamps, or presenting to clients. Each mode demands a different UI layout. Currently, Rubric users must manually rearrange panels, toolbars, and settings every time they switch tasks — a friction that accumulates across dozens of daily context switches.

### 1.2 Solution
A **Profile/Workspace System** that captures the complete UI state (panel visibility/positions/splits, toolbar configuration, theme, measurement defaults, canvas settings) into named, versioned profiles. Users switch profiles instantly via a preview-enabled menu. Four predefined profiles ship out-of-the-box for the primary personas.

### 1.3 User Value
- **Reviewers/Engineers** → "Review" profile: Thumbnails + Markups List + Properties in one click
- **Estimators** → "Measurement" profile: Layers + Measurements + Rulers/Grid pre-configured
- **Markup Authors** → "Markup" profile: Tool Chest + Stamps + Properties, maximum canvas
- **All Users** → "Default" profile: Balanced starting point
- **Teams** → Share `.rubric-profile` files for standardized workflows
- **Power Users** → Create custom profiles for niche workflows (e.g., "Bates Stamping", "Redaction Review")

### 1.4 Success Metrics
| Metric | Target |
|--------|--------|
| Profile switch latency | < 500ms (95th percentile) |
| Profile save/load success rate | 100% (no data loss) |
| Predefined profile adoption | > 60% of active users within 30 days |
| Custom profile creation | > 20% of power users |
| Profile import/export usage | Measurable in telemetry (opt-in) |

---

## 2. Functional Requirements (EARS Format)

### 2.1 Profile Management

| ID | Requirement |
|----|-------------|
| FR-01 | WHEN the user selects a profile from the Profiles menu, THE application SHALL apply the complete UI layout (panel visibility, positions, splits, tab groups, toolbar configuration, theme override, measurement defaults, canvas settings) within 500ms. |
| FR-02 | WHEN the user chooses "Save Profile" from the Profiles menu, THE application SHALL serialize current UI state to a `.rubric-profile` JSON file with schema version, prompting for a name if new. |
| FR-03 | WHEN the application starts, THE application SHALL load the last-used profile or "Default" if none previously selected. |
| FR-04 | WHEN the user enables "Auto-save profile on close" and closes the application gracefully, THE application SHALL update the active profile with current UI state. |
| FR-05 | WHEN the user chooses "Manage Profiles", THE application SHALL display a dialog listing all profiles with preview thumbnails, allowing rename, delete, duplicate, import, export, and set-as-default. |
| FR-06 | WHEN the user imports a `.rubric-profile` file, THE application SHALL validate schema version, migrate if needed, and add to profile list. |
| FR-07 | WHEN the user exports a profile, THE application SHALL write a versioned JSON file to the chosen location. |

### 2.2 Predefined Profiles

| ID | Requirement |
|----|-------------|
| FR-08 | THE application SHALL ship with four predefined profiles: **Default**, **Review**, **Measurement**, **Markup**. |
| FR-09 | Predefined profiles SHALL be read-only (user cannot modify/delete) but SHALL be duplicatable to create custom variants. |
| FR-10 | **Default** profile SHALL reflect the current Rubric balanced layout (all panels visible, standard toolbars). |
| FR-11 | **Review** profile SHALL configure: Left panel → Thumbnails + Bookmarks (tabbed); Right panel → Markups List + Properties (tabbed); Bottom panel → Status; Main toolbar → standard tools. |
| FR-12 | **Measurement** profile SHALL configure: Left panel → Layers + Tool Chest (tabbed); Right panel → Properties + Measurements (tabbed); Rulers visible; Grid enabled with snap; Measurement toolbar visible. |
| FR-13 | **Markup** profile SHALL configure: Left panel → Tool Chest + Stamps (tabbed); Right panel → Properties + Layers (tabbed); Maximum canvas space; Minimal toolbars. |

### 2.3 Profile Persistence & Storage

| ID | Requirement |
|----|-------------|
| FR-14 | Profiles SHALL be stored as per-user global files in `~/.config/rubric/profiles/` (Linux/macOS) or `%APPDATA%\Rubric\profiles\` (Windows). |
| FR-15 | Profile files SHALL use `.rubric-profile` extension with JSON format. |
| FR-16 | Profile JSON SHALL include: `schema_version`, `name`, `description`, `created_at`, `modified_at`, `is_predefined`, `ui_layout` (panels, toolbars, theme, measurement, canvas). |
| FR-17 | Predefined profiles SHALL be embedded in the binary and written to the profiles directory on first run if missing. |

### 2.4 Profile Switching & Preview

| ID | Requirement |
|----|-------------|
| FR-18 | WHEN the user hovers a profile in the Profiles menu, THE application SHALL show a tooltip preview with profile name, description, and layout summary. |
| FR-19 | WHEN the user clicks a profile in the Profiles menu, THE application SHALL apply the profile immediately (no confirmation dialog). |
| FR-20 | WHEN a profile is applied, THE application SHALL animate panel/toolbar transitions smoothly (200ms) respecting `prefers-reduced-motion`. |

### 2.5 Auto-Save Behavior

| ID | Requirement |
|----|-------------|
| FR-21 | ON first application close, THE application SHALL prompt: "Update active profile with current layout?" with options: Yes / No / Don't Ask Again. |
| FR-22 | IF user chooses "Don't Ask Again", THE application SHALL remember preference in settings. |
| FR-23 | WHEN auto-save is enabled and application closes gracefully, THE application SHALL serialize current UI state to the active profile file. |

### 2.6 Import/Export & Versioning

| ID | Requirement |
|----|-------------|
| FR-24 | WHEN importing a `.rubric-profile` file with older schema_version, THE application SHALL auto-migrate using registered migration functions and warn user if data loss possible. |
| FR-25 | WHEN importing a `.rubric-profile` file with newer schema_version, THE application SHALL refuse load with clear error: "Profile requires newer Rubric version (X.Y.Z). Please update." |
| FR-26 | WHEN exporting a predefined profile, THE application SHALL create a custom (non-predefined) copy. |

---

## 3. Non-Functional Requirements

| ID | Requirement | Category |
|----|-------------|----------|
| NFR-01 | Profile switch completes in < 500ms (95th percentile) on reference hardware | Performance |
| NFR-02 | Profile JSON files SHALL be human-readable with 2-space indentation | Usability |
| NFR-03 | Profile schema SHALL be documented in `docs/profile-schema.md` with JSON Schema | Documentation |
| NFR-04 | Profile loading SHALL never crash the application; invalid profiles skipped with warning | Reliability |
| NFR-05 | Predefined profiles SHALL be embedded in binary (no external file dependency) | Deployment |
| NFR-06 | Profile directory SHALL be created automatically on first run | Usability |
| NFR-07 | Maximum 50 profiles per user (prevents clutter); warn at 45 | Usability |
| NFR-08 | Profile operations SHALL be fully testable headless (no GUI required) | Testability |
| NFR-09 | Migration functions SHALL be pure, deterministic, and unit-tested | Maintainability |
| NFR-10 | Profile preview tooltip SHALL render in < 100ms | Performance |

---

## 4. Acceptance Criteria (Given/When/Then)

### 4.1 Profile Switching

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-01: Switch to Review | App running, "Review" profile exists | User selects "Review" from Profiles menu | Panels rearrange: Left=Thumbnails+Bookmarks, Right=Markups List+Properties, Bottom=Status; completes < 500ms |
| AC-02: Switch to Measurement | App running, "Measurement" profile exists | User selects "Measurement" | Rulers visible, Grid enabled, Left=Layers+Tool Chest, Right=Properties+Measurements |
| AC-03: Switch to Markup | App running, "Markup" profile exists | User selects "Markup" | Left=Tool Chest+Stamps, Right=Properties+Layers, maximum canvas |
| AC-04: Switch back to Default | Any profile active | User selects "Default" | Returns to balanced factory layout |
| AC-05: Hover preview | Profiles menu open | User hovers "Measurement" | Tooltip shows: name, description, layout summary (panels, rulers, grid) |

### 4.2 Profile Creation & Management

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-06: Save new profile | User has custom panel layout | User chooses Profiles > Save Profile > names "My Workflow" | `.rubric-profile` created in profiles dir, appears in menu |
| AC-07: Duplicate predefined | "Review" profile exists | User chooses Manage Profiles > Duplicate "Review" > names "Review v2" | New custom profile "Review v2" created, editable/deletable |
| AC-08: Delete custom | Custom profile "My Workflow" exists | User chooses Manage Profiles > Delete "My Workflow" | Profile removed from disk and menu; predefined profiles cannot be deleted |
| AC-09: Rename profile | Custom profile "Old Name" exists | User chooses Manage Profiles > Rename > "New Name" | Profile file renamed, menu updated |
| AC-10: Set default | Multiple profiles exist | User chooses Manage Profiles > Set "Review" as default | "Review" loads on next app start |

### 4.3 Import/Export

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-11: Export custom | Custom profile "My Workflow" exists | User chooses Manage Profiles > Export "My Workflow" > saves to Desktop | `.rubric-profile` file on Desktop, valid JSON, includes schema_version |
| AC-12: Import valid | Valid `.rubric-profile` on Desktop | User chooses Manage Profiles > Import > selects file | Profile added to list, appears in menu, loads correctly |
| AC-13: Import older schema | `.rubric-profile` v1.0, current schema v1.2 | User imports file | Auto-migrates to v1.2, warns if any data loss, loads successfully |
| AC-14: Import newer schema | `.rubric-profile` v2.0, current schema v1.2 | User imports file | Refused with error: "Profile requires newer Rubric version (2.0.0). Please update." |
| AC-15: Export predefined | "Review" profile selected | User chooses Export | Creates custom copy "Review (copy)", not read-only |

### 4.4 Auto-Save

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-16: First close prompt | Fresh install, no preference set | User closes app gracefully | Dialog: "Update active profile with current layout?" [Yes] [No] [Don't Ask Again] |
| AC-17: Auto-save enabled | "Don't Ask Again" = Yes, auto-save enabled | User closes app gracefully | Active profile updated with current UI state silently |
| AC-18: Auto-save disabled | "Don't Ask Again" = No | User closes app gracefully | No profile update, no prompt |

### 4.5 Edge Cases

| Scenario | Given | When | Then |
|----------|-------|------|------|
| AC-19: Corrupt profile | Malformed JSON in profile file | App starts | Profile skipped, warning logged, "Default" loads |
| AC-20: Missing profile dir | First run, no `~/.config/rubric/profiles/` | App starts | Directory created, predefined profiles written, "Default" loads |
| AC-21: Profile limit | 49 profiles exist | User creates 50th | Warning: "Maximum 50 profiles reached. Delete unused profiles to create more." |
| AC-22: Rapid switching | User clicks 3 profiles in < 1s | Rapid menu clicks | Each switch completes; no race conditions; final profile applied correctly |

---

## 5. Error Handling Table

| Error Condition | Detection | User Message | Logging | Recovery |
|-----------------|-----------|--------------|---------|----------|
| Malformed profile JSON | JSON parse failure on load | "Could not load profile 'X': invalid format. Using Default." | ERROR with file path | Skip profile, load Default |
| Missing required field | Schema validation failure | "Profile 'X' is incomplete. Using Default." | WARN with missing fields | Skip profile, load Default |
| Schema version newer | `schema_version` > current | "Profile 'X' requires Rubric Y.Y.Y. Please update." | INFO with versions | Refuse load, offer update link |
| Schema version older | `schema_version` < current | "Profile 'X' migrated from vA.B to vC.D. Some settings reset." | INFO with migration path | Auto-migrate, load |
| Disk full on save | Write returns ENOSPC | "Cannot save profile: disk full." | ERROR | Abort save, retain previous |
| Permission denied | Write returns EACCES | "Cannot save profile: permission denied." | ERROR | Abort save, suggest admin/check perms |
| Import file not found | Path doesn't exist | "Profile file not found." | WARN | Abort import |
| Import not a profile | File lacks `schema_version` | "Selected file is not a valid Rubric profile." | WARN | Abort import |
| Duplicate profile name | Name already exists (case-insensitive) | "Profile 'X' already exists. Choose a different name." | INFO | Block creation, focus name field |
| Max profiles reached | Count >= 50 | "Maximum 50 profiles reached. Delete unused profiles first." | INFO | Block creation, open Manage dialog |

---

## 6. Implementation TODO Checklist

### 6.1 Data Model & Schema (`src/domain/profile.rs`)

- [ ] Define `Profile` struct with all fields
- [ ] Define `ProfileSchemaVersion` const (start at 1)
- [ ] Define `UiLayout` sub-struct (panels, toolbars, theme, measurement, canvas)
- [ ] Define `PanelLayout` (visibility, position, split_state, tab_order, floating)
- [ ] Define `ToolbarLayout` (visible, tools_order, lock_state, custom_toolbars)
- [ ] Define `MeasurementDefaults` (units, precision, scale_preset, grid, rulers)
- [ ] Define `CanvasSettings` (zoom, page_layout, ruler_visibility, guide_visibility)
- [ ] Implement `serde` serialization with `#[serde(rename = "camelCase")]`
- [ ] Write JSON Schema to `docs/profile-schema.json`
- [ ] Unit tests: round-trip serialization, schema validation

### 6.2 Profile Storage (`src/data/profile_store.rs`)

- [ ] `ProfileStore` struct with `profiles_dir: PathBuf`
- [ ] `initialize()` — create dir, write predefined profiles if missing
- [ ] `list_profiles()` → `Vec<ProfileSummary>` (name, predefined, modified)
- [ ] `load_profile(name)` → `Result<Profile>`
- [ ] `save_profile(profile)` → `Result<()>`
- [ ] `delete_profile(name)` → `Result<()>`
- [ ] `export_profile(name, path)` → `Result<()>`
- [ ] `import_profile(path)` → `Result<Profile>`
- [ ] `get_active_profile()` / `set_active_profile(name)` (persisted in app settings)
- [ ] File naming: sanitize name → `kebab-case.rubric-profile`
- [ ] Atomic writes (write to temp, rename)
- [ ] Unit tests: CRUD, import/export, atomic write, concurrent access

### 6.3 Migration System (`src/data/profile_migration.rs`)

- [ ] `MigrationFn` type: `fn(Value) -> Result<Value>`
- [ ] `MIGRATIONS: &[(u32, MigrationFn)]` registry (from_version → fn)
- [ ] `migrate(value, from_version, to_version)` — apply chain
- [ ] Migration v1→v2: add new fields with defaults
- [ ] Migration v2→v3: rename field, transform data
- [ ] Unit tests: each migration, full chain, idempotency

### 6.4 Predefined Profiles (`src/domain/predefined_profiles.rs`)

- [ ] `DEFAULT_PROFILE` — balanced layout
- [ ] `REVIEW_PROFILE` — thumbnails+bookmarks | markups+properties | status
- [ ] `MEASUREMENT_PROFILE` — layers+toolchest | properties+measurements | rulers+grid
- [ ] `MARKUP_PROFILE` — toolchest+stamps | properties+layers | max canvas
- [ ] `PREDEFINED: &[(&str, Profile)]` registry
- [ ] Embed JSON in binary via `include_str!`
- [ ] Write to disk on first run if missing

### 6.5 Profile Application (`src/ui/profile_applicator.rs`)

- [ ] `apply_profile(app: &mut RubricApp, profile: &Profile)` → `Result<()>`
- [ ] Panel application: visibility, dock position, split ratios, tab order, floating state
- [ ] Toolbar application: visibility, tool order, lock state, custom toolbars
- [ ] Theme application: override if `theme_override` set
- [ ] Measurement defaults: units, precision, scale, grid, rulers
- [ ] Canvas settings: zoom, page layout, ruler/guide visibility
- [ ] Animated transitions (200ms, respects `prefers-reduced-motion`)
- [ ] Error handling: partial apply rolls back, logs warnings
- [ ] Integration tests: apply each predefined, verify UI state

### 6.6 UI — Profiles Menu (`src/ui/app_menu.rs`)

- [ ] Profiles submenu in View menu (or dedicated Profiles menu)
- [ ] List profiles with icons (★ for predefined, 👤 for custom)
- [ ] Hover → tooltip with preview (name, description, layout summary)
- [ ] Click → immediate apply (FR-19)
- [ ] Separator → "Manage Profiles…" opens dialog
- [ ] Separator → "Save Current Layout…" opens save dialog
- [ ] Keyboard navigation (arrow keys, Enter to apply)

### 6.7 UI — Manage Profiles Dialog (`src/ui/profile_manager_dialog.rs`)

- [ ] List view: Name | Type (Predefined/Custom) | Modified | Actions
- [ ] Preview thumbnail area (async render or cached SVG)
- [ ] Buttons: New, Duplicate, Rename, Delete, Import, Export, Set Default
- [ ] Predefined profiles: Disable Delete/Rename, allow Duplicate
- [ ] Drag-to-reorder (optional, for menu order)
- [ ] Search/filter by name
- [ ] Keyboard shortcuts throughout

### 6.8 UI — Save Profile Dialog (`src/ui/save_profile_dialog.rs`)

- [ ] Name input (validation: unique, max 64 chars)
- [ ] Description input (optional, max 256 chars)
- [ ] Checkbox: "Set as active profile"
- [ ] Preview of current layout (summary text)
- [ ] OK / Cancel

### 6.9 Auto-Save Logic (`src/ui/auto_save_profile.rs`)

- [ ] Setting: `auto_save_profile: Option<bool>` (None = unprompted)
- [ ] First-close prompt dialog with three buttons
- [ ] On graceful shutdown: if enabled, `save_profile(active)`
- [ ] Handle panic: don't auto-save (use panic hook to disable)

### 6.10 Settings Integration (`src/domain/settings.rs`)

- [ ] Add `active_profile: String` to `AppSettings`
- [ ] Add `auto_save_profile: Option<bool>` to `AppSettings`
- [ ] Persist in `settings.json` (app-level, not profile-level)

### 6.11 Tests

- [ ] Unit: `profile_store` CRUD, migration, import/export
- [ ] Unit: `profile_applicator` applies each predefined correctly
- [ ] Integration: Full profile switch cycle (load → apply → verify UI)
- [ ] Integration: Auto-save on close
- [ ] Integration: Import/export round-trip
- [ ] Integration: Migration v1→v2→v3
- [ ] E2E: User creates profile, switches, restarts, profile persists
- [ ] Performance: Profile switch < 500ms benchmark
- [ ] Accessibility: Manage dialog keyboard nav, screen reader labels

### 6.12 Documentation

- [ ] `docs/profile-schema.md` — JSON Schema with examples
- [ ] `docs/user-guide/profiles.md` — User-facing guide
- [ ] Update `CHANGELOG.md` with feature entry
- [ ] Update `ROADMAP.md` — mark F-606 complete

---

## 7. Dependencies & Risks

### 7.1 Dependencies
| Dependency | Status |
|------------|--------|
| Panel docking/splitting (F-607) | Required for full panel layout persistence |
| Toolbar customization (F-608) | Required for toolbar layout persistence |
| Settings dialog framework | Required for Manage Profiles dialog |
| Animation system | Required for smooth transitions |

### 7.2 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Panel state serialization complex | High | Medium | Design `PanelLayout` carefully; test each panel type |
| Animation conflicts during rapid switch | Medium | Low | Debounce rapid switches; queue transitions |
| Migration breaks user profiles | Low | High | Comprehensive migration tests; backup before migrate |
| Profile limit too restrictive | Low | Medium | 50 is generous; configurable via hidden setting |
| Preview tooltip performance | Low | Low | Cache rendered previews; lazy load |

---

## 8. Open Questions for Review

1. **Profile menu location:** View menu (current) vs dedicated Profiles menu bar item?
2. **Keyboard shortcut for profile switch:** `Ctrl+1`/`Ctrl+2`/`Ctrl+3`/`Ctrl+4` for first 4 profiles?
3. **Per-project profile override:** Future phase? (User asked for global only now)
4. **Cloud sync of profiles:** Out of scope (no network), but design JSON to be sync-friendly?
5. **Profile sharing UI:** "Share Profile" button → copies JSON to clipboard?

---

## 9. Approval

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Product Owner | | | |
| Tech Lead | | | |
| UX Designer | | | |

---

*Specification version 1.0 — Created 2026-11-25 using feature-forge workflow*