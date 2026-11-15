# Bluebeam Revu — Settings & Preferences Architecture

## Method
Public black-box research. Sources: official Bluebeam Technical Support user manual
(Revu 21 and Revu 20 online help), official how-to/troubleshooting articles, and
public community/tutorial pages. All claims carry a source URL. Behavioural
summaries are original wording — no Bluebeam help text is reproduced verbatim.

## Preferences Categories

### Dialog shell
- The Preferences dialog is opened from the menu path **Revu > Preferences**
  (Ctrl+K).
  Source: https://support.bluebeam.com/user-manual/menus/revu/revu-preferences.html
- The dialog contains a **search bar** for finding a specific setting; picking a
  result from the dropdown jumps to the Preferences page containing it.
  Source: https://support.bluebeam.com/user-manual/menus/revu/revu-preferences.html
- Settings are grouped into named categories shown as a list/sidebar.
  Source: https://support.bluebeam.com/user-manual/menus/revu/revu-preferences.html

### Category inventory (Revu 21)
| Category | Notes |
|---|---|
| Administrator | Deployment/IT-level settings; includes Net Config |
| Camera | Camera capture resolutions |
| Document | Part of the General page |
| File Access | Recents, DMS/SharePoint (Interface page) |
| Forms | PDF form field behaviour (Tools page) |
| General | Options/Startup, Document, Navigation, Grid & Snap, Spelling tabs |
| Grid & Snap | Grid, snapping, measurement units system |
| Import/Export | Export-to-Office/image options and import image handling |
| Integrations | Connections to external products |
| JavaScript | Advanced page subsection |
| Markup | Markup appearance/behaviour defaults (Tools page) |
| Markups List | Markups panel behaviour (Interface page) |
| Measure | Measurement tool behaviour incl. Dynamic Fill (Tools page) |
| Navigation | Mouse/scroll behaviour (General tab) |
| Net Config | Network configuration (Administrator page) |
| PDF/A | Advanced page subsection |
| Presentation | Full-screen presentation behaviour (Window page) |
| Rendering | PDF render quality/performance (Advanced page) |
| Sets | File-set category behaviour |
| Signature | Digital signature settings (Tools page) |
| Sketch Tools | Sketch tool input modes (Tools page) |
| Snapshot | Snapshot capture behaviour (grouped under Markup on Tools page) |
| Spelling | Spell check and auto-complete (General tab) |
| Startup | Startup mode/files (General > Options) |
| Studio | Bluebeam Studio session behaviour |
| Tablet | Tablet/Surface Dial behaviour (Window page) |
| WebTab | Embedded web browser tab behaviour (Window page) |

Sources: https://support.bluebeam.com/user-manual/menus/revu/revu-preferences.html and linked per-category pages.

### Organization within categories
- Some categories are single pages; others use **tabs inside the category** —
  General contains five tabs: Options, Document, Navigation, Grid & Snap, Spelling.
  Source: https://support.bluebeam.com/user-manual/menus/revu/general.html
- Control types observed: checkboxes for toggles, dropdowns for enumerated
  choices, sliders for opacity, colour-picker boxes, numeric fields, and path
  fields with a Browse button.
  Sources: https://support.bluebeam.com/user-manual/menus/revu/general.html ,
  https://support.bluebeam.com/user-manual/menus/revu/tools-preferences.html
- Notable behavioural settings in General > Options: user name, interface
  language, Dark/Light theme, tab-name truncation side, full-screen crosshair,
  startup mode, reopen last session, usage-data telemetry opt-in, reset
  hidden messages.
  Source: https://support.bluebeam.com/user-manual/menus/revu/general.html

## Markup Settings

### Tools > Markup category (Revu 21)
All from: https://support.bluebeam.com/user-manual/menus/revu/tools-preferences.html
- **Dynamic default properties**: a toggle makes Revu remember the last-used
  appearance properties per markup type.
- **Reuse Tools**: after placing a markup the tool stays active for repeated
  placement; also toggleable from the Status Bar.
- **Autosize text boxes/callouts** to fit their text after edits.
- **Scale appearance properties for grouped markups**: line width/text size
  scale proportionally when a group is resized.
- **Retain layer information** when copying markups.
- **Embed fonts** used by markups into the PDF.
- **Popup notes**: show author/date in pop-ups; print pop-ups; Pop-Up Opacity slider.
- **Copy highlighted text into the markup comment**.
- **Default image encoding** format.
- **Drag behaviour for shapes**: rectangle-drag vs Bézier control-point mode.
- **Snapshot settings**: raster DPI, vector snapshot, markups included/excluded.

### Default tool properties (three mechanisms)
1. **Set As Default button** — the Properties panel has a Set As Default action.
   Sources: https://support.bluebeam.com/user-manual/menus/tools/create-edit-markups.html ,
   https://support.bluebeam.com/revu/how-to/create-and-save-custom-markups-in-tool-chest.html
2. **Dynamic defaults preference** — the Tools > Markup toggle.
   Source: https://support.bluebeam.com/user-manual/menus/revu/tools-preferences.html
3. **Tool Chest** — users save a customized markup as a reusable tool.
   Source: https://support.bluebeam.com/revu/how-to/create-and-save-custom-markups-in-tool-chest.html

## Measurement Settings

### Tools > Measure category
All from: https://support.bluebeam.com/user-manual/menus/revu/tools-preferences.html
- **Automatically Split Count Measurements by Space**.
- **Dynamic Fill extraction controls**: analysis DPI, hide markups during fill,
  fill cursor size/speed/colour, boundary cursor size/colour, edge sensitivity.
- **Legacy persist behaviour** for measurement Subject/Label.

### Units, precision, and scale
- **Default measurement system (units)** is a Grid & Snap preference.
  Source: https://support.bluebeam.com/user-manual/menus/revu/general.html
- **Precision and per-document units** live on the Measurements panel.
  Source: https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Window/Panels/Measurements/Measure-Tab--MV.htm
- **Default units for new measurements** captured with Set as Default.
  Source: https://support.bluebeam.com/revu/troubleshooting/measurement-markups-dont-appear-in-correct-units.html
- **System-level default units** follow the Windows regional measurement setting.
  Source: https://support.bluebeam.com/revu/how-to/follow-best-practices-for-measurement-markups.html
- **Scale** set per document by calibrating against a known length.
  Sources: https://www.dot.ga.gov/PartnerSmart/Training/Documents/Bluebeam/QRG%20-%20Bluebeam%20Revu%20-%20Scale%20and%20Measurement%20Tools.pdf

## Keyboard Shortcuts

- Shortcut management is a **separate dialog, not a Preferences category**:
  **Revu > Keyboard Shortcuts**.
  Sources: https://support.bluebeam.com/revu/how-to/customize-keyboard-shortcuts.html ,
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Revu/Keyboard-Shortcuts/Keyboard-Shortcuts--MT.htm
- The dialog divides shortcuts into two groups, **Tools** and **Commands**.
  Source: https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Revu/Keyboard-Shortcuts/Keyboard-Shortcuts--MT.htm
- Users can edit defaults, **Restore Defaults** reverts to factory shortcut set.
  Source: https://support.bluebeam.com/revu/how-to/customize-keyboard-shortcuts.html
- **Export/import of shortcut files** supported.
  Sources: https://support.bluebeam.com/revu/how-to/customize-keyboard-shortcuts.html ,
  https://novedge.com/blogs/design-news/bluebeam-tip-configure-and-deploy-bluebeam-revu-keyboard-shortcuts
- **Single-key shortcut suppression in forms**: a Forms preference controls
  whether single-letter tool shortcuts fire while filling PDF form fields.
  Source: https://support.bluebeam.com/user-manual/menus/revu/tools-preferences.html

## Settings Export/Import

- **Whole-preferences export/import**: the Preferences dialog offers **Export**
  and **Import** actions; settings serialize to an XML file.
  Source: https://support.bluebeam.com/user-manual/menus/revu/revu-preferences.html
- **Keyboard shortcuts** have their own export/import.
  Source: https://support.bluebeam.com/revu/how-to/customize-keyboard-shortcuts.html
- **Spelling auto-complete lists** can be imported/exported.
  Source: https://support.bluebeam.com/user-manual/menus/revu/general.html

## Notes for Rubric (original design implications — no Bluebeam copying)
- A searchable preferences dialog with a flat, well-named category list is the
  core UX pattern; search jumps directly to the owning page.
- Tool defaults are layered: per-type dynamic defaults (toggle), explicit
  per-type "set as default", and named reusable tool presets (Tool Chest
  analogue).
- Measurement units split three ways: app-default system (preferences),
  per-document panel state (precision + units), and OS regional defaults —
  Rubric should model these layers explicitly rather than one global unit.
- Export/import of the entire settings tree (XML), plus separate export of
  shortcut profiles, is a documented enterprise workflow worth a capability row.
- Keyboard shortcut management is deliberately outside Preferences with its own
  Tools/Commands split, restore-defaults, and file export/import.

## Source URLs
1. https://support.bluebeam.com/user-manual/menus/revu/revu-preferences.html
2. https://support.bluebeam.com/user-manual/menus/revu/general.html
3. https://support.bluebeam.com/user-manual/menus/revu/tools-preferences.html
4. https://support.bluebeam.com/user-manual/menus/revu/interface-preferences.html
5. https://support.bluebeam.com/user-manual/menus/revu/import-export-preferences.html
6. https://support.bluebeam.com/user-manual/menus/revu/advanced-preferences.html
7. https://support.bluebeam.com/user-manual/menus/revu/sets-preferences.html
8. https://support.bluebeam.com/user-manual/menus/revu/studio-preferences.html
9. https://support.bluebeam.com/user-manual/menus/revu/admin-preferences.html
10. https://support.bluebeam.com/user-manual/menus/revu/window-preferences.html
11. https://support.bluebeam.com/user-manual/menus/revu/integrations-preferences.html
12. https://support.bluebeam.com/revu/how-to/customize-keyboard-shortcuts.html
13. https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html
14. https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Revu/Keyboard-Shortcuts/Keyboard-Shortcuts--MT.htm
15. https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Window/Panels/Measurements/Measure-Tab--MV.htm
16. https://support.bluebeam.com/revu/how-to/follow-best-practices-for-measurement-markups.html
17. https://support.bluebeam.com/revu/how-to/create-and-save-custom-markups-in-tool-chest.html
18. https://support.bluebeam.com/user-manual/menus/tools/create-edit-markups.html
19. https://support.bluebeam.com/revu/troubleshooting/measurement-markups-dont-appear-in-correct-units.html
20. https://support.bluebeam.com/revu/troubleshooting/single-key-keyboard-shortcuts-arent-working.html

Clean-room note: all statements above are original behavioural summaries with
URLs. No Bluebeam UI text, icons, screenshots, or trade dress may be copied
into Rubric; capability-level ideas only (guide §2.1/§2.2). Human approval of
this note is required before any dependent implementation task (guide §2.3).
