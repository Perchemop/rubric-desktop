# Rubric User Guide

Rubric is a free, cross-platform PDF markup, review, and measurement
application. You import a PDF, draw on it, measure it, review it, and export
marked-up copies — without ever modifying the original PDF.

---

## 1. Getting started

**Open a project.** File ▸ Open Project… (`Ctrl+O`) opens a `.rubric.json`
project file. A project stores your markups, scale, and preferences — not the
PDF itself.

**Import a PDF.** File ▸ Import PDF… (`Ctrl+I`) loads a PDF for marking up.
Rubric never modifies the source PDF; all markups live in the sidecar project
file.

**Start drawing.** Pick a tool from the toolbar or press its single-key
shortcut (for example `R` for Rectangle, `H` for Highlight, `1` for Length
measure). Click or drag on the canvas to place a markup.

**Save.** File ▸ Save Project (`Ctrl+S`) writes your `.rubric.json` sidecar.
Reopen it later to continue reviewing.

---

## 2. Tools (toolbar & shortcuts)

| Tool | Shortcut | What it does |
| --- | --- | --- |
| Select | `V` | Select, move, and resize existing markups |
| Highlight | `H` | Highlight text (snaps to text lines under the cursor) |
| Strikeout | `S` | Strike through text |
| Underline | `U` | Underline text |
| Squiggly | `Q` | Squiggly-line annotation |
| Text Box | `T` | Place a resizable text box |
| Callout | `C` | Text box with a leader tail |
| Sticky Note | `N` | Anchored comment pin with threaded replies |
| Stamp | `M` | Pre-set review stamp (Approved, Rejected, etc.) |
| Rectangle | `R` | Filled or outline rectangle |
| Ellipse | `O` | Filled or outline ellipse |
| Cloud | `D` | Cloud-shaped markup |
| Polygon | `P` | Multi-point closed shape |
| Polyline | `Y` | Multi-point open shape |
| Line | `L` | Straight line with optional arrowhead |
| Arrow | `A` | Line with arrowhead |
| Pen / Ink | `I` | Freehand ink stroke |
| Length | `1` | Measure a distance |
| Perimeter | `2` | Measure around a closed shape |
| Area | `3` | Measure surface area |
| Angle | `4` | Measure a three-point angle |
| Diameter | `5` | Measure a circle diameter |
| Radius | `6` | Measure a circle radius |
| Count | `7` | Place a numbered count marker |
| Calibrate Scale | `K` | Two-click reference segment to set page scale |

**Tip:** Every tool also appears in the context menu (right-click the canvas)
and in the View ▸ Tool Chest panel, where you can save and reuse style
presets.

---

## 3. Measurements & scale

Measurements need a page scale to convert PDF points into real units.

**Set the scale.** Choose View ▸ Calibrate Scale (click two points on canvas)…
or press `K`. Draw a two-click reference segment of a known real-world length,
then enter that length in the dialog. Rubric recomputes all measurements on
the page.

**Change units.** Open the Inspector ▸ Measure tab and pick a display unit
(points, inches, feet, millimetres, centimetres, metres, or a custom ratio).
Measurements recompute live.

**Show measurement labels on the canvas.** View ▸ Show Measurements toggles
the on-screen measurement readout under each measure markup.

---

## 4. Review workflow

Every markup carries a review **status**: Open, In review, Approved, or
Rejected. The status controls the markup's colour and appears in the
Inspector ▸ Status legend and the Markups List.

**Change status.** Select one or more markups, then use the Inspector or the
Markups List status dropdown. To set many markups at once, multi-select them
then right-click ▸ Batch Status.

**Add comments.** Select a markup and open the Inspector ▸ Comments tab to
add threaded replies. Sticky Note markups (`N`) are comment pins you place
directly on the canvas.

**Apply a style.** Right-click a markup ▸ Copy Properties, select other
markups, then right-click ▸ Paste Properties. Or save a look as a preset in
the Tool Chest (View ▸ Show Tool Chest) and apply it to any selection.

---

## 5. Exporting (the source PDF is never modified)

**Flattened PDF.** File ▸ Export PDF… bakes your markups onto a copy of the
source PDF, including any Bates stamps. Choose "Export Page to PDF…" for one
page or "Export All Pages to PDF…" for the whole document.

**Page image.** File ▸ Export Page Image… renders the current page with its
markups to a PNG.

**Print.** File ▸ Print… sends the current page to your system printer.

**Report.** File ▸ Export Report… writes a text or JSON summary of all
markups, grouped by status, author, or subject.

---

## 6. Bates numbering (legal stamping)

View ▸ Tool Chest … no — Bates lives in the Inspector ▸ Settings tab. Enable
Bates numbering, set a prefix/suffix, start number, digit padding, corner
placement, and page range. Rubric shows a ghosted preview on the canvas and
bakes the sequence into every exported/printed page. The source PDF is never
touched.

---

## 7. Canvas navigation

- **Zoom:** wheel up/down, or View ▸ Zoom In / Out / Fit Page
  (`Ctrl++` / `Ctrl+-` / `Ctrl+0`), or the 50 / 100 / 200 % presets.
- **Pan:** hold `Space` and drag, or middle-drag.
- **Pages:** Page Up / Page Down, or the Pages sidebar (View ▸ Show Pages).
- **Full screen:** View ▸ Full Screen; press `Esc` to leave.

---

## 8. Keyboard shortcuts at a glance

Open Help ▸ Keyboard Shortcuts… for the full list. The most used:

- `Ctrl+O` Open · `Ctrl+S` Save · `Ctrl+Shift+S` Save As
- `Ctrl+I` Import PDF · `Ctrl+P` Print
- `Ctrl+Z` Undo · `Ctrl+Y` Redo · `Ctrl+A` Select All · `Del` Delete
- `Ctrl+D` Duplicate · `Ctrl+C` / `Ctrl+V` Copy / Paste Properties
- `Ctrl++` / `Ctrl+-` / `Ctrl+0` Zoom In / Out / Fit Page
- `Esc` Cancel current action / close dialog
- `V H S U Q T C N M R O D P Y L A I 1 2 3 4 5 6 7 K` — pick a tool

---

## 9. Accessibility

Rubric supports light, dark, and high-contrast themes (View ▸ Theme / View ▸
High Contrast). All toolbar buttons and context-menu items are named for
screen readers. Every tool has a single-key shortcut that works from the
keyboard without touching the mouse.

---

## 10. Files & locations

- **Project file:** `.rubric.json` — your markups, scale, and preferences.
- **Source PDF:** referenced by path, never modified.
- **Recents:** stored in your OS config directory (not in the project file),
  so shared projects don't carry local paths.

---

## Need help?

- Help ▸ Check for Updates… — see if a new version is available.
- Help ▸ Report a Bug… — capture diagnostics locally, then send only when you
  choose to (no data leaves your machine without your consent).
- Source & releases: https://github.com/Perchemop/rubric-desktop
