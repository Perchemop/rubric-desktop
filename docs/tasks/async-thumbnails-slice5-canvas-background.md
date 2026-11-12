# Task async-thumbnails Slice 5 — canvas page background on the render worker

**Status**: drafted (queued — lands after PR #160 merges)

## Source

- `docs/research/async-thumbnails.md` — §7 Slice 5 ("Route the canvas page
  background through the same worker with a 'background pending' placeholder",
  owner `ui-dev`), §3.1 safety analysis (pdfium-render 0.9.x `thread_safe` is a
  default feature; the canvas background path is currently the second
  sanctioned PDFium caller on the UI thread), §3.3 (texture upload stays on
  the UI thread), §6 acceptance criteria.
- UI audit P1-2 (`docs/audit-ui-2026-11-07.md`): "move heavy work off the UI
  thread" — this is the last synchronous raster in the live editor.
- Spec F-404 / R-S-011 (`REQUIREMENTS_TRACEABILITY.md`).
- Slices 1-3 shipped in PR #160: `apps/rubric/src/domain/thumb_render.rs`
  (domain types), `apps/rubric/src/data/render_worker.rs` (worker),
  `apps/rubric/src/ui/pages.rs` + `apps/rubric/src/ui/app_pages.rs`
  (pages-sidebar wiring).

## Goal

Route the canvas page-background raster — the active page's PDF backing image,
drawn synchronously inside `ui/canvas.rs::draw_canvas` (the `painted_texture`
branch, ~lines 482-563) — through the same `RenderWorker` the Pages sidebar
uses, eliminating the UI-thread PDFium raster. While a background is pending,
the canvas shows an honest paper-white page plus a muted "Rendering page…"
hint instead of hitching the frame; failures keep the existing `bg_error`
surface.

## Current state (verified)

- `draw_canvas` builds the attempt identity
  `attempt_key = format!("{pdf_path}:{page_no}:{target_width}")`
  (canvas.rs:500) — byte-identical to
  `domain::thumb_render::RenderKey`'s Display (`"{pdf_id}:{page}:{width_px}"`).
  `target_width` is quantised to 256-px steps and clamped by
  `data::pdf::MAX_RENDER_WIDTH_PX` (canvas.rs:488-495).
- It then calls `crate::data::pdf::render_page_to_cache` (canvas.rs:502),
  `std::fs::read` (506) and `image::load_from_memory` (507) **on the UI
  thread**, uploads with `ctx.load_texture` into `page_textures` — keyed by
  the cache PNG path (504) — poisons failures in `failed_page_textures` by
  `attempt_key`, and sets `bg_error` + `ui_state.status` to
  `"PDF background unavailable: {e}"` (523-560) without per-frame spam.
- The in-code comment (canvas.rs:476-481) explicitly defers this to Slice 5.
- Worker state today: `RubricApp.render_worker: Option<RenderWorker>`
  (app.rs:428), lazily spawned in `ui/app_pages.rs::draw_pages_panel`
  (app_pages.rs:17-19), torn down with `self.render_worker = None` in
  `install_project_now` (app.rs:1331).

## Scope in

1. **Canvas request path.** Build the `RenderKey` from the existing inputs
   (its Display matches the existing `attempt_key`), submit
   `RenderJob { key, priority: 0 }` through the shared worker; a per-key
   `pending` set (same shape as `ThumbCache.pending` in `ui/pages.rs`) stops
   duplicate submits on every frame.
2. **Canvas drain path.** After painting, drain `worker.try_drain()`:
   `ThumbPixels` are uploaded on the UI thread with `ctx.load_texture` into
   `page_textures` keyed by `ThumbPixels.cache_path` — the existing texture
   cache keying, so the export path's `load_page_background` hits stay valid.
   Failures poison `failed_page_textures` by `attempt_key` and set
   `bg_error` + status exactly as today.
3. **Honest pending state.** While the active page's background render is
   queued or in flight, keep the paper-white fill and add a muted
   "Rendering page…" label centred on the page rect (suppressed when
   `is_export`; markups keep drawing on top; never "No data", never a dead
   control).
4. **Worker ownership hoisted to `RubricApp`.** One `RenderWorker` instance
   shared by pages and canvas: a single accessor
   `RubricApp::ensure_render_worker(&mut self, ctx: &egui::Context)` that
   spawns lazily if `None`; the lazy spawn in `app_pages.rs` is removed;
   `install_project_now` keeps the long-lived worker (it is stateless —
   per-document state lives in the caches) and clears `page_textures`,
   `failed_page_textures`, the canvas pending set and `bg_error`. Drained
   results whose `key.pdf_id` is not the active `source_pdf` are dropped
   (staleness rule, research §6.8).

## Scope out

- **Slice 4 (disk eviction)** — `plan_disk_eviction`, the startup/periodic
  scan, touch-on-serve. Independent slice, may run in parallel.
- **Print/export render path** — `data/render.rs::render_page_to_png_bytes`
  and `load_page_background` stay synchronous (they run on a private
  off-screen `egui::Context::default()`); offloading them is separate P1-2
  follow-up work.
- **Bug-report submit** and **Highlight text lookups** (P1-2 siblings).
- **Texture LRU for `page_textures`** — behaviour parity: the canvas texture
  cache stays unbounded as today. No adoption of `RenderQueueState` either;
  the canvas keeps the lightweight pending-set pattern shipped in Slice 3.
- **No Cargo.toml change** — pdfium-render `thread_safe` is already a default
  feature (research §3.1); `deny.toml`/`ATTRIBUTION.md` unaffected.

## File ownership (parallel partition)

- **Agent A — `ui/canvas.rs` + `ui/app_canvas.rs`:** replace the sync raster
  with the request/drain loop; pending-state painting; keep the poisoning and
  `bg_error` semantics verbatim; remove the "Slice 5 may move this path too"
  comment.
- **Agent B — `ui/app.rs` + `ui/app_pages.rs`:** `ensure_render_worker`
  accessor, canvas pending-set field, install-reset changes, drop-stale-
  results guard shared with the drain call site.
- **Agent C — tests + docs note:** `apps/rubric/tests/render_worker_tests.rs`
  additions, pure unit tests for the pending/staleness helpers,
  `docs/a11y.md` pending-state note.

A and B must agree on the drain-side contract (below) before starting; no
file is touched by two agents.

## Acceptance criteria (DoD)

- [ ] No UI-thread engine work on the live canvas path: `draw_canvas` with
      `is_export == false` no longer calls
      `crate::data::pdf::render_page_to_cache`, `std::fs::read` of the cache
      PNG or `image::load_from_memory`; all raster + decode for the page
      background happen in `data/render_worker.rs` via the shared
      `RenderWorker`. (The off-screen export harness in `data/render.rs` is
      explicitly excluded — see Risks.)
- [ ] Texture upload still happens on the UI thread in the drain:
      `ctx.load_texture` into `page_textures` keyed by `cache_path`; the
      `failed_page_textures` poisoning rule and the `bg_error` + status
      message (`"PDF background unavailable: {e}"`) are preserved verbatim.
- [ ] Honest pending state: paper-white page plus muted "Rendering page…"
      hint while queued/in flight (not in export mode, not blocking markups);
      placeholder copy is honest per the audit guidance.
- [ ] Single worker instance: ownership hoisted to `RubricApp` via
      `ensure_render_worker`; `app_pages.rs` no longer spawns; a review check
      confirms pages and canvas share the one instance.
- [ ] Staleness: `install_project_now` clears the canvas pending set and
      caches, keeps the worker, and drained results for a non-active
      `pdf_id` are dropped (mirrors research §6.8).
- [ ] Behaviour preserved: non-PDF projects keep the flat-white page and the
      empty-state card; the 256-px width quantisation survives so zoom steps
      stay cache-friendly; pan/zoom repaints are not regressed.
- [ ] No undo/snapshot change: `draw_canvas`'s `need_snapshot` return
      contract is unchanged; the background pipeline never mutates the
      project.
- [ ] A11y: the pending-state copy is recorded in `docs/a11y.md` (muted text,
      contrast-checked against paper white).
- [ ] Tests added: a headless integration test rendering a fixture page at a
      canvas-quantised width through the worker; a staleness test proving the
      drain drops results for a non-active `pdf_id`; pure unit tests for the
      pending-set dedup helper.
- [ ] Quality gates green in CI (fmt, `clippy -- -D warnings`, tests, release
      build); PR merged after PR #160.

## Risks

- **Texture upload must stay on the UI thread** (research §3.3):
  `TextureHandle` must not be created on the worker. The drain runs in the
  frame after the worker's `ctx.request_repaint()` (render_worker.rs:51);
  verify no flicker between the pending hint and the painted texture.
- **Repaint scheduling**: the worker's repaint covers completion, but
  page/zoom switches must also repaint (existing behaviour) so the pending
  hint appears and disappears promptly; a missed repaint looks like a hang.
- **Same-instance requirement**: pages and canvas MUST share one
  `RenderWorker` — two workers would serialise against each other through
  PDFium's global mutex and duplicate queue memory. The hoisted accessor is
  the enforcement point.
- **Split borrows**: `draw_canvas_panel` needs `&RenderWorker` beside
  `&mut app.page_textures`, `&mut app.failed_page_textures` and
  `&mut app.bg_error`. `ensure_render_worker` only *ensures* the spawn; the
  call sites then use disjoint field borrows
  (`app.render_worker.as_ref().unwrap()` beside the `&mut` fields) — the
  pattern already proven at `app_pages.rs:32-33`.
- **Export harness**: `render_page_to_png_bytes` drives `draw_canvas` on an
  off-screen `Context::default()` with no worker, and `load_page_background`
  pre-loads the background synchronously there. The signature therefore takes
  `Option<&RenderWorker>`: `None` keeps today's synchronous fallback (out of
  scope), `Some(worker)` is the live path covered by the AC above.
- **Stale results after a project switch**: keeping the worker alive across
  installs means old-document jobs can complete and drain later; the
  pdf_id guard at the drain site drops them.

## Dependencies

- Lands **after PR #160** (dependency refresh + god-file splits + async page
  thumbnails Slices 1-3) merges to `main`; rebase this slice on it.
- Research approval already recorded 2026-11-09 in
  `docs/research/README.md` (approver initials `ld`) — no further human gate
  is needed for this slice.
- Slice 4 (disk eviction) shares no files with this brief; `PROGRESS.md` /
  `CHANGELOG.md` / `REQUIREMENTS_TRACEABILITY.md` updates are integrated by
  the parent.

## Files likely touched

- `apps/rubric/src/ui/canvas.rs` — async background branch (request + drain +
  pending hint); comment update.
- `apps/rubric/src/ui/app_canvas.rs` — pass worker + pending state into
  `draw_canvas`.
- `apps/rubric/src/ui/app.rs` — `ensure_render_worker`, canvas pending-set
  field, install reset.
- `apps/rubric/src/ui/app_pages.rs` — use the accessor instead of spawning.
- `apps/rubric/tests/render_worker_tests.rs` — new tests.
- `docs/a11y.md` — pending-state note.

## Files NOT touched (out of scope)

- `apps/rubric/src/data/render_worker.rs` — the worker API is already generic
  over `RenderKey`; no change needed.
- `apps/rubric/src/data/render.rs` — export/print path stays synchronous.
- `apps/rubric/src/data/pdf.rs` — cache layer unchanged.
- `apps/rubric/Cargo.toml` — no dependency change.

## Notes / follow-ups

- If the pdf_id drain guard proves useful, `ThumbCache` in `ui/pages.rs`
  could adopt the same check (small follow-up, not this slice).
- A canvas-side texture LRU should be a separate decision after Slice 4
  lands; the research budget constants (§5.1) apply unchanged.
