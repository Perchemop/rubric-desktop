# Bluebeam Revu — UI Layout & Panel Architecture

**Purpose:** original behavioural summary of how Bluebeam Revu organizes its
overall window layout, panels, docking, multi-document navigation and status
bar, for Rubric capability-level inspiration. No proprietary material copied;
every claim carries a public source URL. Human gate applies before dependent
implementation (guide §2.3/§8.4).

## Method

Public black-box research. Sources: official Bluebeam Technical Support pages
(support.bluebeam.com user manual and how-to guides), plus public third-party
tutorials. All findings are original paraphrases.

## Window Layout

Revu's desktop window is organized as a vertical stack of regions around a
central document canvas:

1. **Menu bar** — top. Nine categories: Revu, File, Edit, View, Document,
   Batch, Tools, Window, Help.
2. **Properties toolbar** — directly under the menu bar. Dynamic: with nothing
   selected it shows the current document's properties; when a markup is
   selected it shows that markup's editable appearance properties.
3. **Toolbar areas / toolstrips** — customizable groups of tool icons on
   gray "toolstrips" along the top and down the left and right sides.
4. **Side panel areas (left and right)** — vertical pull-out regions hosting
   panels; a "panel access bar" of icons opens them and can auto-hide.
5. **Main workspace** — the central canvas.
6. **Navigation bar** — between the main workspace and the bottom panel,
   dedicated to moving between pages and views.
7. **Bottom panel** — horizontal pull-out at the bottom hosting the Markups
   List, 3D Model Tree, and a JavaScript console.
8. **Status bar** — the final strip at the very bottom.

Default first-run arrangement: navigation/info/properties/tools panels on the
left, markup and measurement toolbars on the right, Markups List at the bottom.

Sources:
- https://support.bluebeam.com/user-manual/welcome/revu-interface.html
- https://support.bluebeam.com/revu/subscription/navigate-and-customize-the-interface-21.html
- https://support.bluebeam.com/revu/resources/revu-21-starter-kit.html

## Side Panels

Revu ships roughly 17 panels; each is a "feature with a special interface":

- **Tool Chest** — reusable markup library organized into tool sets.
- **Markups List** — tabular log of every markup (author, date, status, comments).
- **Thumbnails** — page previews.
- **Bookmarks** — hierarchical outline.
- **Properties** — file metadata or selected markup's properties.
- **File Access** — recent-files and explorer modes.
- **Layers** — PDF layer visibility and management.
- **Flags** — lightweight position markers.
- **Links** — places/links defined in the document.
- **Search** — text search.
- **Measurements** — measurement tool settings, viewports, snap-to-content.
- **Sets** — multi-file drawing-set management.
- **Spaces** — spatial regions defined on pages.
- **Signatures** — digital signature workflows.
- **Forms** — form-field creation/editing (edition-gated).
- **Studio** — Bluebeam Studio Sessions/Projects collaboration.
- **3D Model Tree** — 3D model views/tree.

Panel toggling is centralized in **Window > Panels**.
Sources:
- https://support.bluebeam.com/revu/how-to/customize-panels.html
- https://support.bluebeam.com/user-manual/menus/window/window-panel.html

## Panel Docking

- **Attach left / right / bottom.** Drag a panel's icon to any edge of the
  workspace, or right-click the icon and choose Attach > Left / Right / Bottom.
- **Float.** Drag a panel icon into the main workspace to detach it into a
  floating panel.
- **Split (stack side-by-side within one dock).** Drag one panel icon onto an
  open panel and a four-arrow "position wheel" appears; dropping on an arrow
  splits that dock region in the chosen direction.
- **Hide.** Right-click a panel icon > Hide.
- **Auto-hide panel access bar** — the icon strip itself can auto-hide.
- **Resizing.** Hover a panel edge and drag to resize.
- **Overlay mode.** The bottom panel can overlay the left/right panels.
- **Persistence.** The workspace arrangement is remembered between sessions
  and can be saved into a named **Profile**.

Sources:
- https://support.bluebeam.com/revu/how-to/customize-panels.html
- https://support.bluebeam.com/revu/how-to/customize-toolbars.html

## Multi-Document Navigation

- **Document tabs.** Open PDFs appear as tabs above the main workspace; tabs
  can be reordered, moved between splits, hovered for a preview.
- **MultiView (split workspace).** Splits the workspace into independent,
  resizable regions — up to **sixteen** splits. Different documents or
  different areas of one document can occupy different splits.
- **MultiView Extended (detached windows).** Drag a document tab off the main
  window to create a stand-alone workspace for multi-monitor work.
- **Synchronized views.** Two sync modes: **Synchronize Document** (same page
  index across documents) and **Synchronize Page** (whatever content is on
  screen stays in step regardless of page index).

Sources:
- https://support.bluebeam.com/user-manual/multiview.html
- https://support.bluebeam.com/revu/how-to/split-sync-and-unsplit-views.html

## Navigation Panels (Thumbnails, Bookmarks)

- **Navigation bar** (toggle F4). Three zones: left = MultiView split controls;
  center = pan/zoom/select mode switching and page/view stepping; right =
  profile menu, Dimmer, document properties, security.
- **Thumbnails panel** (Alt+T). Page previews; clicking a thumbnail jumps to
  that page. Size slider, page labels, page operations toolbar (insert /
  extract / replace / delete / rotate pages). Pages reordered by dragging.
- **Bookmarks panel** (Alt+B). The document's outline as a parent/child tree;
  clicking a bookmark runs its action (default: jump to the saved view).
  Bookmarks can be created from page labels or text capture ("AutoMark").

Sources:
- https://support.bluebeam.com/user-manual/toolbars/navigation-bar.html
- https://support.bluebeam.com/online-help/revu20/Content/RevuHelp/Menus/Window/Panels/Thumbnails/Thumbnails-Tab--MTV.htm
- https://support.bluebeam.com/user-manual/menus/window/bookmarks-panel.html

## Status Bar

Located at the very bottom; toggled with F8. Split into two zones:

- **Left — context-sensitive help text.** Reads "Open PDF to markup" with no
  document, "Ready" when idle, and usage hints for the selected tool.
- **Right — document readouts and toggles.** Page scale, page size (dimensions),
  live cursor coordinates.
- **Toggle buttons** (highlighted blue when active): Show Grid, Snap to Grid,
  Snap to Content, Snap to Markup, Reuse Markup Tools, Sync (Document/Page
  mode dropdown).

Sources:
- https://support.bluebeam.com/user-manual/toolbars/status-bar.html
- https://support.bluebeam.com/revu/how-to/customize-navigation-and-status-bars.html

Clean-room note: all statements above are original behavioural summaries with
URLs. No Bluebeam UI text, icons, screenshots, or trade dress may be copied
into Rubric; capability-level ideas only (guide §2.1/§2.2). Human approval of
this note is required before any dependent implementation task (guide §2.3).
