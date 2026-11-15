# Bluebeam Revu — Markup Tools Organization

## Method

Public black-box research. Sources: official Bluebeam documentation (Revu 20
online help, Revu 21 user manual, Bluebeam Technical Support articles) and
public tutorial pages. Researched 2026-11-15. All wording below is an
original behavioural summary; tool names and shortcut keys are factual data
from the cited pages.

## Tool Categories

- Bluebeam's own documentation treats "markup" and "annotation" as
  interchangeable terms: markups are annotations that sit on top of the PDF
  content and can be moved and edited independently.
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Unsorted/Markups-Introduction--M.htm
- The **Tools menu** is the canonical inventory, subdivided into named
  sub-menus: **Markup**, **Image**, **PDF Content**, **Stamp**, **Measure**,
  **Sketch to Scale**, **Form**, and **Signature** (plus Hyperlink and File
  Attachment entries).
  https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html
- **Markup sub-menu tools** (drawing/annotation): Text Box, Typewriter, Note,
  Callout, Pen, Highlight, Eraser, Line, Arrow, Arc, Polyline, Dimension,
  Rectangle, Ellipse, Polygon, Cloud, Cloud+.
  https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html
- **Image sub-menu**: place image from file / camera / scanner, Crop Image, Flag.
  https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html
- **PDF Content sub-menu**: Review Text, Underline, Squiggly, Strikethrough,
  auto-creation of hyperlinks from URLs.
  https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html
- **Measure sub-menu**: Set Scale, Length, Polylength, Area, Perimeter,
  Diameter, Center Radius, 3-Point Radius, Angle, Volume, Polygon Cutout,
  Ellipse Cutout, Count, Dynamic Fill.
  https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html
- **Sketch to Scale sub-menu**: Polygon, Rectangle, Ellipse and Polyline
  variants that place a shape at an exact scale entered by the user.
  https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html

## Tool Access Patterns

- The same markup tools are reachable four ways: the **Tools menu**, the
  **Text** and **Shapes toolbars**, the **Tool Chest panel**, and keyboard
  shortcuts. https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Unsorted/Markups-Introduction--M.htm
- Every time a markup tool is used it is automatically stored in the
  **Recent Tools** section of the Tool Chest for easy reuse.
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Unsorted/Markups-Introduction--M.htm
- **Tool Chest** (Alt+X) stores frequently used markups in named **tool
  sets**; **My Tools** is permanent across sessions while **Recent Tools** is
  cleared when Revu closes.
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- A placed markup can be saved as a reusable tool by right-clicking it and
  choosing **Add to Tool Chest**.
  https://support.bluebeam.com/user-manual/menus/window/use-tool-chest.html
- Custom tool sets can be created, imported, exported, shared over a network,
  and pinned to a toolbar; tool sets can show **Symbol** (icon grid) or
  **Detail** (table) modes.
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- **Hot keys vs keyboard shortcuts**: Tool Chest tools get numeric hot keys
  (numbered by order stored; My Tools is the default hot-key set). Pressing
  the number activates the tool *with its saved properties*, whereas keyboard
  shortcuts always invoke the tool's default properties.
  https://support.bluebeam.com/revu/features/hot-keys-vs-keyboard-shortcuts.html
- **Recent Tools** can default to *Properties Mode* (only appearance saved) or
  *Drawing Mode* (appearance plus geometry/text saved as a stamp-like tool).
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Window/Panels/Tool-Chest/Working-with-the-Tool-Chest--MTV.htm
- A **Reuse** toggle makes the last-used tool stay active after placing.
  https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html
- Revu has a **View/Markup mode split**: markup tools are disabled until the
  user switches to Revu > Markup Mode.
  https://support.bluebeam.com/revu/troubleshooting/tools-and-menu-items-are-disabled.html
- While a markup tool is active, holding **spacebar** pans the document.
  https://support.bluebeam.com/revu/how-to/tips-and-tricks/bluebeam-subscription-tips-and-tricks-guide.html

## Keyboard Shortcuts

Full published list:
https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html

| Tool | Shortcut | Tool | Shortcut |
|---|---|---|---|
| Arrow | A | Line | L |
| Callout | Q | Note | N |
| Cloud | C | Pen | P |
| Cloud+ | K | Polygon | Shift+P |
| Dimension | Shift+L | Polyline | Shift+N |
| Ellipse | E | Rectangle | R |
| Eraser | Shift+E | Stamp | S |
| Highlight | H | Text Box | T |
| Image | I | Typewriter | W |

- Manipulation/editing: Group Ctrl+G, Ungroup Ctrl+Shift+G, Lock Ctrl+Shift+L.
- **Measure shortcuts are deliberately separated** from markup ones:
  Length Shift+Alt+L, Polylength Shift+Alt+Q, Area Shift+Alt+A,
  Perimeter Shift+Alt+P, Volume Shift+Alt+V, Diameter Shift+Alt+D,
  Radius Shift+Alt+U, Angle Shift+Alt+G, Count Shift+Alt+C, Dynamic Fill J,
  Measure Tool M. https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html
- Selection/navigation: Select V, Lasso Shift+O, Pan Shift+V.
- Panel toggles: Markups list Alt+L, Measurements panel Alt+U, Tool Chest
  Alt+X, Properties Alt+P.
- Shortcuts are **user-remappable** via Revu > Keyboard Shortcuts.
  https://support.bluebeam.com/revu/how-to/customize-keyboard-shortcuts.html
- **Modifier-key drawing behaviours**: Shift constrains drawing to
  horizontal/vertical/45°; Shift makes Rectangle/Ellipse draw squares/circles.
  https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html

## Properties Panel

- Selecting any placed markup and right-clicking > **Properties** (or
  Alt+P) opens the Properties panel showing that markup's data.
  https://support.bluebeam.com/online-help/revu21/Content/RevuHelp/Menus/Window/Panels/Properties/Properties--MV.htm
- The markup Properties panel has consistent sections: **General**
  (Author, Subject, Label, Comments, Lock), **Appearance** (style settings
  that vary per tool), **Custom**, **Layout** (numeric position fields), and
  **Options**.
  https://support.bluebeam.com/online-help/revu21/Content/RevuHelp/Menus/Window/Panels/Properties/Properties--MV.htm
- **Style IS editable after drawing**: the Appearance section exposes
  colour, opacity, fill, line width/style/ends per tool.
  https://support.bluebeam.com/online-help/revu21/Content/RevuHelp/Menus/Tools/Markup/Rectangle-Tool--MTV.htm
- **Options** section holds two key actions: **Add to Tool Chest** and
  **Set As Default**.
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Unsorted/Markups-Introduction--M.htm
- The right-click context menu additionally offers Apply to All Pages,
  Change Colors for multi-selection recolouring, Format Painter, and Lock.
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Unsorted/Markups-Introduction--M.htm
- **Measurements get an extra dedicated surface**: a separate **Measurements
  panel** where Subject/Label/Depth are entered during placement.
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Unsorted/Takeoffs-and-Measuring--T.htm

## Measurement vs Annotation Organisation

- Measurements are documented and grouped as their own family: a dedicated
  **Measure** sub-menu, a dedicated **Measure toolbar**, a dedicated
  **Measurements panel**, their own shortcut block, and their own help-guide
  chapter — separate from the Markup family.
  https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html
- They converge at the data layer: measurements **can persist as markups**.
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Unsorted/Takeoffs-and-Measuring--T.htm

## Markup List

- The **Markups list** (bottom panel by default; Alt+L) automatically tracks
  every markup placed on the PDF with author, date, colour and comments.
  https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- It is a sortable table; selecting a row jumps the canvas to that markup.
  https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- **Columns are extensive and toggleable**: Subject, Page, Lock, Status,
  Checkmark, Author, Date, Colour, X/Y coordinates, Comments, measurement
  value columns, Label, Sequence, Layer, Space, Legend.
  https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- **Status workflow lives here**: default statuses are Accepted, Rejected,
  Completed, Cancelled and None; custom statuses can be created.
  https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- **Reply threads**: replies attach under a markup's row in the Comments column.
  https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- Toolbar features: **Search**, **Filter List** with saveable named filter
  sets, **Import/Export** of markups (PDF, XML, BAX, FDF), **Summary** report
  publishing.
  https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- A running **measurement total** per measurement type is shown at the top
  of the Measurement column.
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Unsorted/Takeoffs-and-Measuring--T.htm

## Rubric relevance (behavioural takeaways, capability-level only)

- Category-first tool organisation (Markup vs Measure vs Sketch-to-Scale)
  with a persistent, searchable list of placed markups.
- Two-speed re-invocation: hot keys replay a tool *with saved styling*,
  keyboard shortcuts replay defaults — a clean model for Rubric's Tool
  Chest/inspection design.
- Properties editing is uniformly post-draw (select → panel/toolbar), with
  "Set as default" and "Add to tool chest" as the two styling-persistence
  actions.
- Measurements share the annotation data model but get a dedicated panel,
  shortcut block and totals surface.

## Source URLs

- https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Unsorted/Markups-Introduction--M.htm
- https://support.bluebeam.com/user-manual/menus/tools/tools-menu.html
- https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html
- https://support.bluebeam.com/user-manual/menus/window/markups-list.html
- https://support.bluebeam.com/online-help/revu21/Content/RevuHelp/Menus/Window/Panels/Properties/Properties--MV.htm
- https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- https://support.bluebeam.com/user-manual/menus/window/use-tool-chest.html
- https://support.bluebeam.com/revu/features/hot-keys-vs-keyboard-shortcuts.html
- https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Unsorted/Takeoffs-and-Measuring--T.htm

Clean-room note: all statements above are original behavioural summaries with
URLs. No Bluebeam UI text, icons, screenshots, or trade dress may be copied
into Rubric; capability-level ideas only (guide §2.1/§2.2). Human approval of
this note is required before any dependent implementation task (guide §2.3).
