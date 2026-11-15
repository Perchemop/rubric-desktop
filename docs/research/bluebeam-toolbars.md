# Bluebeam Revu — Toolbar & Tool Chest Architecture

## Method
Public black-box research. Sources: official Bluebeam Technical Support pages,
the Revu user manual / online help, an official Bluebeam YouTube tutorial, the
official Bluebeam blog, and public third-party training articles. Every claim
carries its source URL. All wording below is an original behavioural summary —
no Bluebeam help text is reproduced. Researched 2026-11-15.

Key structural insight up front: Revu separates **toolbars** (command-launcher
icon strips docked around the workspace) from the **Tool Chest** (a side panel
that stores reusable *saved markups*, organized into named **tool sets**). The
two systems are configurable independently but can be bridged — a tool set can
be pinned to a toolbar.

## Toolbar Architecture

- The Revu interface is driven by a menu bar plus customizable toolbars and
  panels. Bluebeam's interface overview names eight layout regions: Menu Bar,
  Toolbar, Side panels, Panel access bar, Main workspace, Navigation bar,
  Bottom panel, Status bar.
  https://support.bluebeam.com/user-manual/welcome/revu-interface.html
- A toolbar is a group of icons representing functions (markups, measurements,
  form fields, etc.). Bluebeam describes toolbar tools as "more specific
  functions that do not require a special interface" — this is their stated
  contrast with Panels, which are full interfaces (e.g. Tool Chest, Markups
  List). https://support.bluebeam.com/revu/how-to/customize-toolbars.html
- Toolbars live in **toolstrips** — the empty bars to the left, right or top
  of the workspace where toolbar and panel icons sit. Each toolbar shows a
  "handle" (a string of dots at its top or left edge) used to drag it; panels
  and toolbars always occupy separate toolstrips.
  https://support.bluebeam.com/revu/how-to/customize-toolbars.html
- **Named customizable toolbars.** The Revu 21 help lists 18 customizable
  toolbars: Advanced, Advanced Text, Alignment, Appearance, Control Point,
  Digital Signatures, Document, Document Management, File, Font, Full Screen,
  Form, Line Style, Measure, Order, Rotation, Shapes, Sketch.
  https://support.bluebeam.com/se/online-help/revu21/Content/RevuHelp/Menus/Tools/Toolbars/Toolbars--MV.htm
- Bluebeam's "Customize toolbars" reference table lists 22 toolbar names for
  Revu 20/21 including the Properties Toolbar, plus two edition-gated ones:
  Form and Redaction were Revu eXtreme-only in the perpetual-licence era.
  https://support.bluebeam.com/revu/how-to/customize-toolbars.html
- **Three fixed special toolbars** are not part of the customizable set:
  the Status Bar, Navigation Bar and Properties Toolbar — the Revu 21 Tools
  menu describes these three as fixed.
  https://support.bluebeam.com/de/online-help/revu21/Content/RevuHelp/Menus/Tools/Tools-Menu.htm
- **Properties Toolbar:** sits just below the menu bar, cannot be moved (can
  be hidden), and is *dynamic* — it shows the editable properties of whatever
  is currently active (nothing selected → basic PDF info; a markup selected →
  that markup's appearance properties; a Studio tab → project info).
  https://support.bluebeam.com/user-manual/toolbars/properties-toolbar.html
- **Navigation Bar:** sits between the main workspace and the bottom panel;
  page/view movement plus split views, zoom/pan/select tools, page scale
  indicator, profile selector, Dark Mode toggle. Toggle with F4.
  https://support.bluebeam.com/user-manual/toolbars/navigation-bar.html
- **Status Bar:** bottom of the app; snap controls, markup reuse, view
  synchronization, active PDF size, mouse coordinates.
  https://support.bluebeam.com/user-manual/welcome/revu-interface.html
- **Which toolbars show by default depends on the Profile.** Toolbar
  configurations are stored per Profile; customizing toolbars affects only the
  active Profile, and exporting/sharing a Profile shares its toolbar layout.
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Tools/Toolbars/Toolbars--MV.htm
- Example default toolbar contents (per Bluebeam's per-toolbar reference
  pages; the exact set varies by Profile):
  - Shapes toolbar default: Line, Arrow, Arc, Polyline, Callout, Dimension,
    Rectangle, Ellipse, Polygon, Cloud, Cloud+, Image, Crop Image.
    https://support.bluebeam.com/user-manual/toolbars/shapes-toolbar.html
  - Document toolbar default: Headers & Footers, Crop Pages, Flatten, Erase
    Content. https://support.bluebeam.com/user-manual/toolbars/document-toolbar.html

### Adding / removing / reordering toolbar tools

- Show/hide whole toolbars: Tools > Toolbars — each named toolbar has a
  checkmark when visible. https://support.bluebeam.com/revu/how-to/customize-toolbars.html
- Move a toolbar: drag its handle onto any toolstrip; a new toolstrip is
  created automatically if dropped on empty space, and dragging toward the
  opposite side of an existing strip creates a second row.
  https://support.bluebeam.com/revu/how-to/customize-toolbars.html
- Lock toolbars (Tools > Toolbars > Lock) so they cannot be dragged. Locking
  is all-or-nothing (individual toolbars cannot be locked separately), and
  locked toolbars hide their handles, saving a little screen space.
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Tools/Toolbars/Toolbars--MV.htm
- **The Customize Toolbars dialog** (Tools > Toolbars > Customize) has four
  panes: *Categories* (filters the command list), *Commands* (available
  commands), *Toolbar* (list of existing toolbars with show/hide checkboxes),
  and *Items* (the commands currently on the selected toolbar). From it a user
  can: add a command to a toolbar; reorder Items up/down; remove an item;
  hide an item without removing it (uncheck it); insert separators or blank
  spaces after an item; create a brand-new named toolbar; rename a toolbar;
  and delete a toolbar. This covers both built-in and user-created toolbars.
  https://support.bluebeam.com/user-manual/toolbars/toolbars.html and
  https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Tools/Toolbars/Toolbars--MV.htm
- Panel access bars (the icon strips that open panels such as Tool Chest) can
  auto-hide when unused, and their contents can be chosen via a Show menu.
  https://support.bluebeam.com/revu/subscription/navigate-and-customize-the-interface-21.html

## Tool Chest

- The Tool Chest is a panel that stores customized markups so they can be
  re-placed on any PDF without being recreated. Open it via Window > Panels >
  Tool Chest or the Alt+X shortcut.
  https://support.bluebeam.com/revu/features/tool-chest-guide.html and
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- Its contents are organized into **tool sets** — named groups of related
  markups for specific workflows or project types. Which tool sets are enabled
  by default depends on the active Profile; My Tools and Recent Tools exist in
  every Profile. https://support.bluebeam.com/revu/features/tool-chest-guide.html
- The panel has a top dropdown menu listing all available tool sets (checked =
  shown), plus Manage Tool Sets and Tool Chest settings.
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- **Each tool set has its own mini toolbar** with some or all of:
  expand/collapse toggle; a scale on/off toggle (when the set has a scale);
  a lock/checked-out indicator for shared sets; and a properties (gear) menu.
  Collapsed sets expose a flyout arrow that lets the user pick a tool without
  expanding the set. https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- Per-set properties menu (non-Recent sets): Symbol/Detail display modes, Set
  Scale / Remove Scale, generate a Markups **Legend** from the set, Export,
  Save, **Pin** (attach the tool set to a chosen toolbar), Hide.
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- Tool Chest settings include **Comment Persistence** (whether a tool's stored
  comment text is copied onto the target markup when applied — with a default
  mode that spares Text Boxes/Notes) and **Update Tool Set Item on Reuse**
  (property edits made while placing are written back to the stored tool).
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- **Placement modes.** Every stored tool works in one of two modes:
  *Properties Mode* (place a new markup reusing only the saved appearance —
  colour, line width, opacity, style — user redraws shape/text; the default
  for most tools) or *Drawing Mode* (place an exact duplicate of the stored
  markup, including dimensions and text). Double-click the tool's icon to
  toggle; the icon's appearance changes to signal the mode. Grouped markups
  and Stamps are always Drawing Mode. Recent Tools' default mode is
  configurable in its properties menu.
  https://support.bluebeam.com/revu/features/tool-chest-guide.html and
  https://support.bluebeam.com/user-manual/menus/window/use-tool-chest.html
- **Set Scale:** a scale can be assigned to a whole tool set so its tools
  resize automatically when placed in a differently-calibrated drawing —
  aimed at reusing the same set across drawings with different scales.
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- **What cannot be stored as a tool:** Notes, Hyperlinks, File attachments,
  the text-editing markups (Review Text, Underline, Squiggly, Strikethrough,
  Text Highlight), and symbols that already have a Sequence or Action defined.
  https://support.bluebeam.com/revu/how-to/create-and-save-custom-markups-in-tool-chest.html
- **Adding tools to the chest:** right-click a markup on the PDF → Add to
  Tool Chest → pick the target set; or drag a tool from Recent Tools into
  another set. Multiple markups can be grouped (Shift-select → Group) and the
  group is saved as a single reusable tool. Customizing a markup's properties
  and choosing "Set as Default" in the Properties panel changes that *tool's*
  default everywhere.
  https://support.bluebeam.com/revu/how-to/create-and-save-custom-markups-in-tool-chest.html
- **Special Sequences & Actions tool set:** a Sequence makes a text-based
  markup auto-increment (number or text) each time it is placed; an Action
  attaches a hyperlink to a Tool Chest tool.
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html

## Tool Sets

- **Creating:** Tool Chest dropdown → Manage Tool Sets → Add → New; enter a
  title and choose options: *Display* (show it now), *Show In All Profiles*
  (cross-profile availability), and *Relative Path* (store the set's file
  location relative to the current Profile). The set is saved as a `.btx`
  file at a chosen location.
  https://support.bluebeam.com/user-manual/menus/window/manage-tool-sets.html
- **Reordering / show-hide:** inside Manage Tool Sets, tool sets can be
  dragged into a preferred order, and individual sets checked on/off to show
  or hide them in the panel.
  https://support.bluebeam.com/revu/features/tool-chest-guide.html
- **Individual tools within a set** are reordered by dragging them.
  https://support.bluebeam.com/revu/features/tool-chest-guide.html
- **Sharing model — two distinct operations:**
  - *Export* writes a `.btx` file that can be distributed (email, etc.).
  - *Import* copies the `.btx` into the user's default location (local,
    independent copy; edits do not sync back).
  - *Add (Add Existing)* references the file where it lives — intended for a
    shared network drive so all users' copies stay the same; changes made by
    anyone with the set propagate to everyone sharing it. Network sets are
    cached locally for offline use.
  https://support.bluebeam.com/revu/how-to/share-tool-sets.html and
  https://support.bluebeam.com/user-manual/menus/window/manage-tool-sets.html
- **Shared-set locking:** a network-shared set is locked (read-only) by
  default; a lock icon shows this. A user with write permission can click the
  lock to check the set out for editing (checked-out icon). Offline users
  cannot check a set out. https://support.bluebeam.com/user-manual/menus/window/manage-tool-sets.html
- **Default tool sets by Profile.** Revu ships with preset profiles; each
  profile enables different tool sets: the default Revu profile includes
  Architect, Contractor, and Engineer Review sets plus measurement and symbol
  sets; the Field Issues profile carries punch-key sets organized by industry;
  the Quantity Takeoff profile carries measurement sets.
  https://www.taradigm.com/9-ways-to-customize-bluebeam-revu-using-profiles/
  https://www.youtube.com/watch?v=cwup-ydP1JA
- Bluebeam also publishes a "Revu Custom Libraries" page of downloadable
  industry-standard tool sets. (Exact count of preloaded sets per profile is
  not published — unverified — no public source found.)
  https://support.bluebeam.com/revu/how-to/tips-and-tricks/add-tool-sets-to-pdf-layers.html
- **Pin to toolbar:** a tool set can be pinned to a specified toolbar, the
  one documented bridge between the Tool Chest and the toolbar system.
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html

## Toolbar vs Tool Chest — the difference

- Toolbars = configurable collections of **command/function icons** docked in
  toolstrips around the workspace; Bluebeam describes their tools as specific
  functions that don't need a special interface.
  https://support.bluebeam.com/revu/how-to/customize-toolbars.html
- Tool Chest = a **panel** (a full interface region on a side edge) that
  stores *saved markups* — reusable tools with their appearance, metadata
  (Subject/Comment/Label), placement mode, and optional scale — organized in
  tool sets. https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- They interoperate: markups are usually created with toolbar/menu tools first,
  then saved into Tool Chest sets; and a tool set can itself be pinned onto a
  toolbar. https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html

## Recent/Custom Tools (My Tools & Recent Tools)

- **Recent Tools** automatically records every markup as it is placed on a
  PDF, so any recent markup can be re-placed immediately. It is temporary:
  cleared automatically when Revu closes. When more recents exist than fit, an
  overflow arrow at the right reveals the rest.
  https://support.bluebeam.com/user-manual/menus/window/use-tool-chest.html
- Recent Tools settings (gear menu): configurable **Maximum Recents** count,
  manual **Clear Recents**, a default Properties/Drawing Mode choice, Hide,
  and **Add to My Tools** for the selected tool.
  https://support.bluebeam.com/revu/features/tool-chest-guide.html and
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- **Promoting a recent tool to permanent:** drag it from Recent Tools into any
  other tool set, or right-click → "Add Item to My Tools".
  https://support.bluebeam.com/user-manual/menus/window/use-tool-chest.html
- **My Tools** is the permanent, personal set of frequently used tools. It
  exists in all Profiles by default, survives restarts, and can be hidden but
  never fully removed. https://support.bluebeam.com/user-manual/menus/window/use-tool-chest.html
- **My Tools auto-hotkeys:** each tool in My Tools is automatically assigned a
  numeric hot key according to its *position* in the set; the number is shown
  in the upper-right corner of the tool's icon, and reordering the set
  reassigns the hot keys. Number shortcuts can be enabled via Settings >
  Enable Number Shortcuts, and a different tool set can be assigned to the
  hot keys via Settings > Edit.
  https://support.bluebeam.com/user-manual/menus/window/use-tool-chest.html and
  https://support.bluebeam.com/revu/features/hot-keys-vs-keyboard-shortcuts.html

## Icon Display (grid/list, labels)

- Two display modes per tool set: **Symbol view** (tools shown as their visual
  icons only — the icon-grid look) and **Detail view** (a table listing each
  tool with its Subject, Comment, and Label text — the labelled-list look).
  Chosen per set from the set's properties/gear menu.
  https://support.bluebeam.com/revu/features/tool-chest-guide.html and
  https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- An **icon-size slider at the bottom of the panel** scales the tool icons
  larger or smaller. https://support.bluebeam.com/revu/features/tool-chest-guide.html

## Source URLs

Official Bluebeam Technical Support / Revu help:
- https://support.bluebeam.com/user-manual/welcome/revu-interface.html
- https://support.bluebeam.com/revu/how-to/customize-toolbars.html
- https://support.bluebeam.com/user-manual/toolbars/toolbars.html
- https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Tools/Toolbars/Toolbars--MV.htm
- https://support.bluebeam.com/user-manual/toolbars/properties-toolbar.html
- https://support.bluebeam.com/user-manual/toolbars/navigation-bar.html
- https://support.bluebeam.com/user-manual/toolbars/shapes-toolbar.html
- https://support.bluebeam.com/user-manual/toolbars/document-toolbar.html
- https://support.bluebeam.com/revu/subscription/navigate-and-customize-the-interface-21.html
- https://support.bluebeam.com/revu/features/tool-chest-guide.html
- https://support.bluebeam.com/user-manual/menus/window/tool-chest-panel.html
- https://support.bluebeam.com/user-manual/menus/window/use-tool-chest.html
- https://support.bluebeam.com/user-manual/menus/window/manage-tool-sets.html
- https://support.bluebeam.com/revu/how-to/share-tool-sets.html
- https://support.bluebeam.com/revu/how-to/create-and-save-custom-markups-in-tool-chest.html
- https://support.bluebeam.com/revu/features/hot-keys-vs-keyboard-shortcuts.html

Official Bluebeam media/blog:
- https://www.youtube.com/watch?v=cwup-ydP1JA (official Bluebeam Tool Chest tutorial video)

Public third-party training (behaviour only):
- https://www.taradigm.com/9-ways-to-customize-bluebeam-revu-using-profiles/
- https://www.taradigm.com/how-to-create-custom-tool-sets-in-bluebeam-revu/
- https://facilities.gatech.edu/sites/default/files/revu-basics-guide_rev1.pdf

Clean-room note: all statements above are original behavioural summaries with
URLs. No Bluebeam UI text, icons, screenshots, or trade dress may be copied
into Rubric; capability-level ideas only (guide §2.1/§2.2). Human approval of
this note is required before any dependent implementation task (guide §2.3).
