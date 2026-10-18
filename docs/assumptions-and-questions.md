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

## Decisions so far
- Name: Rubric (original, punchy, thematically apt).
- Stack: Rust + egui/eframe only; free OSS.
- Domain-first over UI-first.
- JSON sidecar persistence (separate from PDF) for safety.
- Scale stored per page; measurements recompute and keep value_points.
- History capped at 128 snapshots.
- Phase 0 implements Slice 1: drawing, selection, move, undo/redo, inspector, save/load, tests.
- Never commit external reference material or the build guide.
