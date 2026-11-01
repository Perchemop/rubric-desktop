# Accessibility Statement

## Commitment

Rubric is a desktop PDF markup and measurement tool. Its users read
documents, mark them up, and take measurements — some of them with
disabilities. Accessibility is treated as an engineering requirement, not a
future nicety: **nothing about disabled users without disabled users**
(accessibility reports count as expertise, not complaints).

## Target standard

Desktop GUI applications have **no direct WCAG equivalent**, but the
*intent* of **WCAG 2.2 AA** is adopted wherever egui allows it. Each item
below maps an AA principle to the desktop context, and each is tested by
URLs or manual checks, not by assertion.

## Priorities (in order)

1. **Full keyboard operation** — every panel, tool, dialog, and menu is
   usable without a mouse. The README documents the global shortcuts
   (Ctrl/Cmd combos, Esc, Delete, page navigation); the per-tool single
   keys are shown on each tool's tooltip and in the context menu.
   Shortcuts are single-key when safe (Esc, Delete, Space) and Ctrl/Cmd-prefixed
   when they collide with text entry (Save, Undo, Redo).
2. **Visible focus and cancel** — the active tool, hovered surfaces, and
   in-progress input always have a visible state; Esc is a managed escape
   from every modal state (drag-in-progress, dialogs, popups) — it never
   commits the gesture it cancels.
3. **Non-colour-only information** — semantic differences are carried by
   shape/text (tool icons differ in silhouette; error states add text, not
   just red flashes; grid/snap/ruler states toggle visible controls).
4. **Contrast** — UI text is kept comfortably above contrast requirements in
   both the light and dark (F-602) themes; report anything
   that reads badly and it will be fixed in the theme tokens, not locally.
5. **Screen-reader semantics** — egui exposes widgets' roles to OS
   accessibility on supported platforms; text labels are real text, images
   have alternatives, and counting on hover hints alone is avoided.

## Known limitations

Recorded honestly, in changelog-visible form:

- Interaction is heavily pointing-device-and-shortcut based; there is no
  "tap-free" pure-serial-command mode. Keyboard paths exist, so workarounds
  may always be found — please open an issue for any that do not.
- egui's OS-level accessibility bridge is incomplete upstream; a screen
  reader may not get rich tree semantics for the canvas. The canvas interior
  is a drawing surface (like a PDF viewer); its contents are conveyed by
  the document and measurements readouts in text panes.
- Measurement values are colour-coded per tool *and* textually labelled —
  no auxiliary depth-only rendering (no reliance on stereo/3D cues).

## Supported environments

Desktop, all bundled platforms: Linux x86_64/arm64, macOS arm64,
Windows x86_64. Web/WASM is **not** currently targeted, so web-only
guidance does not apply. Minimum window size and responsive panel layout
are verified per release on the primary (Linux) platform.

## Reporting an accessibility barrier

Use the **Accessibility report** issue template (it asks for impact,
reproduction, environment/AT already at expected severity, and what you
expected to happen), or label an issue `a11y` / `accessibility` if writing
free-form. Accessibility impairment can block use entirely, so these
reports are triaged with the same priority as High-severity bugs.
The PR checklist requires any UI change to pass the accessibility review
items (`.github/PULL_REQUEST_TEMPLATE.md`).
