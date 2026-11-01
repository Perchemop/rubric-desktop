# Free / OSS design tooling for an egui desktop app (research note)

Date: 2026-11-01 · Method: web research, licences verified against upstream
sources. Policy baseline: `deny.toml` gates *dependencies*; running a GPL
*rasteriser* in CI is a tool, not a dependency, and does not affect asset
licensing. Everything below is free and slots into the CI-first workflow
(GitHub Actions is the only runner).

## 1. Design & prototyping

- **Penpot** (MPL-2.0, <https://github.com/penpot/penpot/blob/develop/LICENSE>)
  — Figma-class, SVG-native, design tokens; free hosted tier. SVG/CSS output
  maps cleanly onto the token tables in `docs/design/brand.md`.
- **Excalidraw** (MIT, <https://github.com/excalidraw/excalidraw/blob/master/LICENSE>)
  — low-fi wireframing; the community "Desktop Resolutions" and "Lo-Fi
  wireframing kit" libraries fit desktop-app flows
  (<https://libraries.excalidraw.com/>).
- Avoid **tldraw** — SDK is source-available, not OSS (production needs a
  licence key). Watch **OpenPencil** (MIT, very young).

## 2. Icon sets + egui embedding

- **`egui-phosphor`** (crate MIT OR Apache-2.0; icons MIT,
  <https://crates.io/crates/egui-phosphor>) — bundled icon **font**; the
  lowest-friction route (no SVG rasterisation), with a `subset!` macro to
  compile in only used glyphs. Directly retires the ▲▼◀▶ unicode placeholders.
- **Lucide** (ISC + MIT subset, <https://lucide.dev/license>) ~1,900 icons;
  **Tabler** (MIT, <https://github.com/tabler/tabler-icons/blob/main/LICENSE>)
  6,238 icons. Caveat: egui's `egui_extras` SVG loader is resvg-based and does
  not reliably honour `currentColor`, which both sets use — preprocess to a hex
  colour or wrap (e.g. `egui-lucide`, which is very new/maturity flag).

## 3. Colour / design systems (feed `docs/a11y.md`)

- **Radix Colors** (MIT) — 12-step scales where each step has a semantic role;
  steps 11/12 are text colours with guaranteed contrast
  (<https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale>).
- **Adobe Leonardo** (Apache-2.0) — generates palettes *from target contrast
  ratios* (WCAG 2 + APCA), <https://github.com/adobe/leonardo>. Use to fix the
  documented `#E4572E` ≈ 3.7:1 dark-theme link case.
- **Open Color** (MIT) — neutral UI ramps, colour-blindness tested
  (<https://github.com/yeun/open-color>).
- CI tie-in: keep the `shipped_palettes_meet_wcag_contrast` unit test as the
  gate; these systems are for choosing better values. Manual aids: WebAIM
  contrast checker, WhoCanUse.

## 4. Typography (all OFL-1.1 — already in `deny.toml`'s allow list)

- **Inter** (<https://github.com/rsms/inter/blob/master/LICENSE.txt>) — tall
  x-height, tabular numbers + slashed zero: ideal for measurement readouts.
  Bundle static weights, not the variable font.
- **JetBrains Mono** (OFL for fonts, <https://github.com/JetBrains/JetBrainsMono/blob/master/OFL.txt>)
  — monospace for coordinates/scale fields.
- egui bundling: `fonts.font_data.insert("inter", Arc::new(FontData::from_static(include_bytes!(...))))`
  then insert at position 0 of `FontFamily::Proportional` (official
  `custom_font` example). Duties: ship the OFL text with the binaries
  (extend `ATTRIBUTION.md`); OFL Reserved-Font-Name rules apply if modified.

## 5. Visual regression / screenshot evidence in CI (gap G4)

- **`egui_kittest`** (MIT OR Apache-2.0) — headless; `harness.snapshot("name")`
  writes PNGs to `tests/snapshots/`, failures produce `.new.png`/`.diff.png`,
  `UPDATE_SNAPSHOTS=true` regenerates, per-OS tolerance in `kittest.toml`
  (<https://github.com/emilk/egui/blob/main/crates/egui_kittest/README.md>).
  It also asserts the **AccessKit tree** (`get_by_label`,
  `accesskit_node().toggled()`) — closing the "widget-by-widget a11y audit"
  item in `docs/a11y.md` at the same time. Recent versions ship an **`eframe`
  feature** that can drive an `eframe::App` directly — check first; it may
  shrink the planned `Host`/`Ui` seam refactor.
- Runners are GPU-less: install Mesa **lavapipe** (`mesa-vulkan-drivers`,
  `libvulkan1`, point `VK_ICD_FILENAMES` at `lvp_icd*.json`). Generate
  baselines **on ubuntu-latest**, not locally — cross-system pixel comparison
  is the known failure mode (wgpu-py's own practice note).
- Image-diff crates if bespoke checks are needed: `dify` (MIT — the engine
  kittest already uses), `image_compare` (MIT, SSIM/RMS).

## 6. UX review aids (no lab required)

- **Nielsen's 10 Usability Heuristics**
  (<https://www.nngroup.com/articles/ten-usability-heuristics/>) + the free
  heuristic-evaluation workbook. NN/g content is copyrighted — record findings
  as original text in `docs/`; do not republish the workbook.
- **18F Methods** (**CC0**, <https://github.com/18F/methods>) — redistributable
  usability-test script and worksheets
  (<https://methods.18f.gov/usability-test-script/>); adaptable into `docs/`.
- For an egui app, web checkers (axe/Lighthouse) do not apply; the equivalent
  is kittest's AccessKit assertions + the WCAG contrast unit test.

## 7. Logo / brand-asset pipeline (evaluates `scripts/gen-icons.sh`)

The current script shells out to a **local** `rsvg-convert`, contradicting the
CI-first rule, and produces no `.ico`/`.icns` (Q-PACK-2). Replacement, all in
Actions:

| Tool | Licence | Role |
|---|---|---|
| **resvg** | Apache-2.0 OR MIT (<https://github.com/linebender/resvg>) | SVG→PNG at 16–1024 px; same renderer as egui's SVG loader, so CI output matches in-app rendering |
| **ImageMagick** (`magick`, preinstalled on ubuntu runners) | permissive | multi-size **`.ico`** for the WiX MSI (`-define icon:auto-resize=16,24,32,48,64,256`) |
| **icnsutil** | MIT (<https://github.com/relikd/icnsutil>) | composes the **`.icns`** from the PNGs (alternative: Apple's `iconutil` on `macos-latest`) |
| libicns `png2icns` | LGPL-2.1+ | usable as a CI *tool* but icnsutil is the cleaner pick |
| Inkscape | GPL-3.0 tool | fine for occasional interactive editing; adds nothing over resvg-in-CI |

Window icon in-app: `ViewportBuilder::with_icon` +
`eframe::icon_data::from_png_bytes` on the generated 256 px PNG.

## Prioritised adoption plan

| # | Item | Effort | Why first |
|---|---|---|---|
| 1 | CI icon pipeline (resvg + ImageMagick `.ico` + icnsutil `.icns`, outputs committed) | M | closes Q-PACK-2; removes the last local-only script; zero code risk |
| 2 | `egui-phosphor` icon font (subset) for toolbar/status glyphs | S | biggest visible quality jump per hour; retires the unicode-arrow placeholders |
| 3 | Bundle Inter + JetBrains Mono + OFL texts in `ATTRIBUTION.md` | S | typography consistency; tabular numbers for measurements |
| 4 | G4: `Host`/`Ui` seam + `egui_kittest` snapshot & AccessKit tests (baselines on ubuntu-latest + lavapipe) | L | unlocks the deferred visual-evidence gate *and* the a11y-tree audit in one refactor |
| 5 | Palette refresh pass (Radix semantics + Leonardo) for the sub-AA `#E4572E` dark-theme link | S-M | turns the honest known limitation into a pass |
| 6 | UX pass with 18F CC0 scripts + NN/g heuristic workbook (findings recorded as original text) | S | readiness evidence; findings feed the v0.4 slices |
