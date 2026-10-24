# Assumptions and Questions

## Assumptions (made for Phase 0 + Slice 1)
- egui/eframe 0.36 is stable and sufficient for desktop canvas (pan/zoom, draw, select).
- PDF rendering not required in Slice 1: a blank page rectangle is sufficient; PDF will be added later (lopdf or pdfium). The domain is page-space based, so PDF rendering can slot in.
- Single page per "view" initially; document may have multiple pages but UI defaults to first page.
- JSON persistence is adequate for first slice (human-readable, versionable later).
- Measurements: 1 pt = 1/72 in (PDF standard). Scale is user-configurable.
- Undo/redo via full project snapshots is cheap enough for small documents (history capped 128).
- Headless build/link works on Linux (dev libs present). GUI won't run headlessly but `cargo build`/`test`/`clippy`/`fmt` do.
- Timestamps: UTC via chrono.
- No cloud/auth: local-first only.
- Free-only deps: MIT/Apache-2.0 (image, serde, egui, uuid, thiserror, tracing, directories, chrono).

## Scope decisions (Phase 1 — 2026-10-19)
- Feature inventory frozen as `docs/spec/README.md`: 52 feature IDs in seven groups (F-1xx markups/tools, F-2xx measurements/scale, F-3xx review, F-4xx navigation/pages, F-5xx persistence/I/O, F-6xx UI/UX, F-7xx packaging); priorities map to IMPLEMENTATION_PLAN phases (MVP = Phases 2–3 core).
- PDF rendering: attempt a **pure-Rust crate first** (keeps CI, cross-compilation and packaging free of native/system dependencies); the exact crate is decided in Phase 5 (see Questions 17).
- Persistence MVP is **sidecar-only**: all markups/review data live in `.rubric.json`; the source PDF is never opened for writing.
- Export MVP = **rendered image (PNG) + print**; PDF-embedded annotation export is excluded from the MVP and only revisited later (spec non-goal for now).
- Review status set is **Open / In review / Approved / Rejected** (spec F-301); the domain enum was remapped in Phase 4 (2026-10-19), replacing the old variant set (None/Accepted/Rejected/Completed/Cancelled/Alert) — no released files exist, so the rename was free.
- Spec-level requirement IDs **R-S-001…R-S-017** were added to REQUIREMENTS_TRACEABILITY.md, mapping spec areas → F-IDs → planned modules.
- Non-goals recorded in the spec: cloud/collaboration, editing PDF content itself, proprietary/encrypted formats, mobile/web, digital signatures.

## Questions (to answer during Phase 1+)
1. **Page scale UI**: how should users set/lock scale (ratio like 1 in=10 ft, or preset)? Provide presets + custom?
2. **PDF import/export**: which crate (lopdf MIT vs pdfium-render)? Need to preserve markups as sidecar JSON or embed? Prefer sidecar (non-destructive) initially.
3. **Zoom/pan**: Space+drag or middle mouse; wheel zoom centred on cursor (current: button-based; needs mouse wheel + space drag).
4. **Selection**: frontmost (last added) on top? Current hit test picks highest index after reverse — treat "last added on top".
5. **Drawing modes**: polygon/polyline need click-to-add points (finish with Enter/Right-click). Ink needs drag with pressure? egui has no pressure; fallback to distance threshold.
6. **Units rounding**: how many decimals in display? Current formatting shows 3 decimals; area 3 decimals; angle 2 decimals.
7. **Persistence format version**: add a `schema_version` field to Project?
8. **Icon/packaging**: need .desktop file, AppImage? Guide covers later phases.
9. **Theme**: support light/dark? Phase 0 uses light; extend later.
10. **Text editing in-canvas**: egui popup/modal for text/callout content (current text boxes show text but editing not implemented).
11. **Multi-page navigation**: add page list/thumbnails?
12. **Grid/snap**: implement if useful for measure/align?
13. **Stroke dash rendering**: current UI sets dash but canvas rendering draws solid line segments; implement dashed rendering (repeat segments).
14. **Cloud shape**: current draws 4 ovals (loose); could approximate better or use path.
15. **Highlight rendering**: semi-transparent fill + thicker stroke inside — works for egui rect_filled + rect_stroke with StrokeKind::Inside.
16. **Hit testing tolerance**: 3-6 pt default; make configurable?
17. **PDF render crate (Phase 5)**: which pure-Rust renderer — evaluate page rasterisation fidelity, memory use, licence (MIT/Apache-2.0), and font/text handling before committing.
18. **Stamp library content (Phase 4)**: which built-in presets (status marks, check/cross style)? Text-rendered or vector? Do users get custom stamps (own text/image)?
19. **Report export format (F-304)**: plain text, JSON, CSV, or printable view? Which grouping default (status, page, author)?
20. **Image export scope (F-505)**: current page or whole document? Fixed DPI or choice? Where does print fit — OS print dialog only?
21. **Layer model (F-116)**: per-page or per-document layers, and how does it relate to the existing ToolSet presets (expand ToolSet → Layer)?
22. **Page navigation (F-403/F-404)**: thumbnail sidebar vs page dropdown vs both; does navigation arrive with PDF import (Phase 5) or earlier on blank pages?
23. **Q-PACK-1 (signing)**: Should macOS/Windows installers be code-signed (and notarised for Apple)? Signing requires secrets that this repository deliberately does not hold, so the first packaging pass ships **unsigned** installers and documents that trust decision instead of faking a signature.
24. **Q-PACK-2 (icons)**: Only `assets/logo.svg` exists today. Raster icons (PNG at 16/32/48/64/128/256/512 and an `.icns` for macOS) are needed for proper desktop integration; the current workflows ship the SVG and mark this as a follow-up rather than generating a conflicting icon.
25. **Q-PACK-3 (PDFium in installers)**: PDFium is a run-time dependency (`docs/pdfium.md`). Linux/macOS bundles copy it next to the executable; Windows copies `pdfium.dll` beside `rubric.exe` when found. Should official releases bundle it (with its BSD-3-Clause terms restated) or keep shipping runtime-only discovery? Working assumption: bundle, because PDF import is the app's core purpose.

## Suite questions (guide v4, answered 2026-10-24)

- **Q4 (Is this a multi-app suite?)**: Rubric is currently a **single app**, but the repository is laid out as a suite from now on: `apps/rubric` plus shared crates, so a second app inherits crash reporting and update checking without duplication.
- **Q18 (Shared crash-report + update-check crates?)**: **Yes — implemented 2026-10-24** as `crates/crash-report` (panic hook, scrubbed capture, review-and-cancel dialog, user-token or pre-filled-URL submission) and `crates/update-check` (GitHub Releases API, semantic version compare, fail-soft offline, 1-hour cache). See DECISIONS.md "Suite-ready restructure".
- **Q26 (Crash-report privacy)**: reports are captured locally, shown to the user in full, and only transmitted after an explicit Send. Delivery is browser-based by default (no token in the app); a user-supplied token is used once in memory and never persisted. Documented in `README.md`.
- **Q27 (Research method)**: recorded per note in `docs/research/`; preferred ladder is browse-capable model → CI text fetch → human assist (guide §2.4). No research notes exist yet — the implemented feature set was designed from the original spec, not from vendor research.

## Packaging (Phase 7, 2026-10-23) — architecture as built

- `.github/workflows/ci.yml` (existing) is the quality gate: fmt, clippy `-D warnings`, tests, release build. It runs on every push to `main` and every PR.
- `.github/workflows/release.yml` (new, spec F-701) builds the release binary on three targets (Linux x86_64, macOS arm64, Windows x86_64), packages each as a portable archive, and attaches them to a GitHub release. Tags `v*` publish as real releases; `workflow_dispatch` publishes as a pre-release so a manual run never masquerades as one.
- `.github/workflows/package.yml` (new, spec F-702/F-703/F-704) builds the native installers: a Linux **AppImage** (linuxdeploy over an AppDir containing a `.desktop` entry and appstream metadata), a macOS **`.app`/`.dmg`** bundle, and a Windows **MSI** (WiX v4, product definition kept in-repo at `packaging/wix/Product.wxs`), each with a portable zip/tarball fallback.
- Both workflows honour the CI-first rule: they only build and package, never re-run the test suite, and the desktop machine never runs any of them.

## Docs (Phase 7, 2026-10-23)

- `docs/pdfium.md` (new) documents the PDFium run-time contract: discovery order (`RUBRIC_PDFIUM_PATH` → executable directory → system path), where to obtain pre-built binaries, what a failed import looks like, and which installer bundles the library.
- The packaging workflows deliberately avoid `--locked` surprises by not bumping the lockfile; if `Cargo.lock` is absent from the tree they build against the registry ranges (see Questions 31 on versioning).

## Decisions so far
- Name: Rubric (original, punchy, thematically apt).
- Stack: Rust + egui/eframe only; free OSS.
- Domain-first over UI-first.
- JSON sidecar persistence (separate from PDF) for safety.
- Scale stored per page; measurements recompute and keep value_points.
- History capped at 128 snapshots.
- Phase 0 implements Slice 1: drawing, selection, move, undo/redo, inspector, save/load, tests.
- Never commit external reference material or the build guide.

## Runtime notes (Phase 5, 2026-10-20)
- Native file dialogs use `rfd`'s pure-Rust xdg-portal backend on Linux. A running
  `xdg-desktop-portal` service is required; on minimal window-manager setups the
  Open/Save dialogs may not appear until one is installed (no code workaround —
  documented limitation).
- Recent files persist app-level in the OS config dir (`~/.config/rubric/recent.json`
  on Linux), not in `.rubric.json`, so review documents never carry local paths.
