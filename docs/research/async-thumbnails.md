# Design note — async page thumbnails (worker thread + on-disk cache)

Status: **approved 2026-11-09 (approver initials `ld`)** — recorded in
`docs/research/README.md`. Internal design written for Rubric; the only
external sources are the public API docs of crates already in the tree
(linked below). No third-party source, layout, icon or algorithm is
reproduced. Per `docs/research/README.md`, a note of this kind needs explicit
owner approval *before* the dependent implementation Task (guide §2.3, §8.4) —
this note is an implementation input **only** because that human gate is
recorded.

Scope: move PDF page-thumbnail rasterisation **and decode** off the UI thread in
the Pages sidebar (spec F-404), reusing the existing on-disk render cache, and
wire the dormant `RenderQueue` scaffolding (UI audit P1-2) into a real worker
pipeline. It also removes a second, identical sync render on the canvas page
background path so the two share one worker.

This is one sub-slice of audit P1-2 ("move heavy work off the UI thread"); the
other P1-2 items (print/export render, bug-report submit, Highlight text
lookups) are out of scope here and only share the plumbing.

---

## 1. Problem statement

Today every page thumbnail is rasterised **synchronously on the UI thread** while
the Pages panel scrolls:

- `ui/pages.rs::load_thumb` calls
  `data::pdf::render_page_to_cache(path, index, 192)`, then `std::fs::read`,
  then `image::load_from_memory` → `to_rgba8`, then
  `ctx.load_texture(...)` — all inline in the frame that first shows the cell
  (`ui/pages.rs:144-186`).
- The canvas page background does the same for the active page
  (`ui/canvas.rs:475-540`), with the comment "No background threads this slice
  (pdfium is not thread-safe)".
- `docs/audit-ui-2026-11-07.md` (P1-2) records the symptom: "page thumbnails
  (render inline while scrolling)" and recommends adopting the app's own
  worker-thread + repaint-signal pattern used by `update-check`
  (`ui/app.rs::begin_update_check`, `ui/app.rs:597-620`).

The cost per thumbnail is one PDF page rasterisation (tens to hundreds of ms for
a text-heavy letter page) **plus** a PNG file read and an image decode. At
~96 px cells a full sidebar can request a dozen pages in one frame, each
blocking the frame — a visible freeze on import and on first scroll.

## 2. Current state (what actually exists)

The P1-2 scaffolding was added but is **entirely unwired**:

| Item | Location | State |
|---|---|---|
| `RenderQueue { in_flight, result, completed_key }` | `ui/app.rs:107-115` | `#[expect(dead_code)]`; nothing reads it |
| `RenderJob { pdf_path, page_index, thumb_width_px, cache_key, result_tx }` | `ui/app.rs:120-131` | dead; `result_tx` (`Option<mpsc::Sender<RenderResult>>`) is never constructed |
| `RenderResult::{Success, Failure{error}}` | `ui/app.rs:135-140` | dead; carries no pixels or path |
| `RepaintSignal` | `ui/app.rs:146` | dead; never used |
| `render: Arc<Mutex<RenderQueue>>` field | `ui/app.rs:477` | `#[allow(dead_code)]` |
| `begin_render_thumb(...)` | `ui/app.rs:631-677` | `#[allow(dead_code)]`; spawns a *detached* thread per call, never called, **never calls `ctx.request_repaint()`**, and stores no result payload |
| `ThumbCache { textures: HashMap<String,TextureHandle>, failed: HashSet<String> }` | `ui/pages.rs:16-20` | synchronous, unbounded |
| On-disk cache `render_cache/<pdf_key>/page_<i>_<w>.png` | `ui/../data/pdf.rs:1249-1331` | shared with canvas; atomic writes; **no eviction** |

`begin_render_thumb`'s own doc comment already sketches the intended contract
("the worker renders the page to a cached PNG using the existing
`pdf::render_page_to_cache` … the UI reads the queue each frame and, once the job
completes, creates the egui texture"), but the implementation is a stub: one job
at a time, no payload, no repaint, no reader. This note replaces it with a real
pipeline.

Reusable pieces (keep):
- `data::pdf::render_page_to_cache` — shared, content-keyed, atomic on-disk
  cache; a page the canvas already rendered is a cache hit (`data/pdf.rs:420-449`).
- The `update-check` worker pattern (`ui/app.rs:597-620`): clone `egui::Context`,
  move it into the thread, call `ctx.request_repaint()` when done.
- The canvas poisoning rule: a failed render is remembered for the session so it
  is not retried every frame (`ui/canvas.rs:488-493`, `ui/pages.rs:154-156`).

## 3. Constraints

### 3.1 PDFium is not thread-safe (the decisive constraint)

`pdfium-render` states plainly that PDFium "makes no guarantees about thread
safety and should be assumed *not* to be thread safe", and that the library
authors recommend *parallel processing*, not multi-threading, to process
multiple documents. The crate exposes a `thread_safe` feature that wraps the
global bindings behind a mutex so every call is serialised; without it `Pdfium`
is neither `Send` nor `Sync`.
Source: <https://docs.rs/pdfium-render/latest/pdfium_render/> (public API docs).

Rubric's `apps/rubric/Cargo.toml` currently pulls `pdfium-render = "0.9"` with
**default features and no `thread_safe`** (line 37). Consequence: the current
"no background threads" caution on the canvas is correct, and a naive
worker-thread render would be a data race against any UI-thread PDFium call
(import/probe paths also initialise the global bindings).

Two safe options, both adopted in the design:

1. **One dedicated render worker thread**, long-lived, owning *all*
   `render_page_to_cache` calls. Even without the `thread_safe` feature, a
   single thread that never overlaps another PDFium caller is safe. This is what
   "parallel processing, not multi-threading" means: one serialised renderer.
2. **Enable `pdfium-render`'s `thread_safe` feature** in `apps/rubric/Cargo.toml`
   as defence-in-depth, so an accidental call from another thread cannot crash.
   It gives no speed-up (it mutex-serialises), but the goal here is *off the UI
   thread*, not parallel throughput — so that is acceptable. This is a one-line
   Cargo change and a `deny.toml`/`ATTRIBUTION.md` no-op (same crate, same
   version, feature only).

### 3.2 hayro is already thread-safe

When the `hayro` feature is enabled, `render_page_to_cache` dispatches to
`hayro_pages::render_page_hayro` — pure Rust, no global mutable bindings, `Send`
friendly (`data/pdf.rs:438-448`). The worker design is therefore correct with
either engine; the `thread_safe` change is only needed for the PDFium default.

### 3.3 egui threading

- `egui::Context` is `Clone + Send + Sync` (an `Arc` internally); the
  update-check thread already clones and moves one (`ui/app.rs:608-618`), so
  that pattern is proven in-tree.
- `Context::load_texture` is callable from any thread, but texture *uploads* are
  applied at frame start. The design creates textures in the UI-thread poll for
  deterministic ordering and to keep the worker free of egui types.
- `TextureHandle` must not be created on the worker; the worker produces raw RGBA.

### 3.4 Domain purity

Per `AGENTS.md`, `src/domain/` stays pure: no threads, channels, `std::fs`, or
egui. The queue **state machine**, the **key**, the **staleness rule** and the
**eviction policies** are pure and live in domain; the thread, channels, file
I/O and texture upload live in `data/` and `ui/`.

## 4. Design

### 4.1 Architecture

```
 UI thread (ui/pages.rs, each frame)              Worker thread (data/render_worker.rs)
 ─────────────────────────────────                ──────────────────────────────────────
 draw_pages()
   for each visible cell:
     cache.wanted(key)  ──► enqueue RenderJob ──► job_rx.recv() (blocking)
     if let Some(tex) = cache.textures.get(key)   render_page_to_cache(path,i,w)
        paint it                                 fs::read(png) → image::decode → RGBA
     else paint numbered placeholder              result_tx.send(ThumbResult{key,rgba,dims})
   after the panel:                               ctx.request_repaint()
     for each ThumbResult in result_rx.try_recv():
       ctx.load_texture → cache.insert
     if worker idle and backlog non-empty: nothing (worker pulls)
```

- **One long-lived worker**, spawned once in `RubricApp::new_headless` (or
  lazily on first request). It owns the job receiver and the result sender.
- **Channels**: `std::sync::mpsc` (already imported in `app.rs`; no new
  dependency). UI owns `Sender<RenderJob>` and `Receiver<ThumbResult>`; worker
  owns the mirror ends. Bounded work is achieved by the UI only enqueuing keys
  it does not already have pending/loaded, so the channel depth stays small
  (a few dozen at most).
- **Backpressure / ordering**: UI enqueues *deduplicated* keys. Priority is
  FIFO with the current page's neighbours first (see 4.4). No unbounded growth.
- **Repaint**: worker calls `ctx.request_repaint()` after each `send`, so a
  completed thumbnail appears next frame without polling latency (fixes the stub
  bug noted in §2).

Why a dedicated thread rather than `std::thread::spawn` per job (the current
stub): PDFium serialisation (§3.1), bounded resource use, and a single place to
own the engine. `thread_safe` makes accidental overlap safe; the single thread
makes overlap not happen.

### 4.2 Domain types (`src/domain/thumb_render.rs`, pure, serde)

```rust
/// Identity of one thumbnail render: enough to dedup, cache and poison.
/// Display form is stable and matches the existing UI key
/// `"<pdf path>:<page>:<width px>"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderKey {
    pub pdf_id: String,   // canonical-ish path display, or a per-doc id
    pub page: usize,
    pub width_px: u32,
}
impl std::fmt::Display for RenderKey { /* "{pdf_id}:{page}:{width_px}" */ }

/// A queued request. Pure data; the worker adds the engine call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderJob {
    pub key: RenderKey,
    /// Higher runs first; the UI derives this from distance to `current_page`.
    pub priority: i32,
}

/// Result payload crossing back to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderOutcome {
    Ready { key: RenderKey, cache_path: String, width: u32, height: u32 },
    Failed { key: RenderKey, message: String },
}
```

Note: `RenderOutcome::Ready` carries the **cache path + dimensions**, not the
RGBA pixels, because domain must not own a `Vec<u8>` blob that cannot be
diffed/serialised cheaply in tests. The worker sends the path; the UI reads and
uploads. To also move the *decode* off the UI thread (recommended, §4.3) the
worker additionally sends the decoded bytes over a **separate** `data`-layer
message (`data::render_worker::ThumbPixels`), which is not a domain type.

Pure state machine (the heart, fully headless-testable):

```rust
/// Queue + poison state for one active document. No threads, no I/O.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderQueueState {
    /// Keys enqueued and not yet completed, in the order they must run.
    pending: Vec<RenderJob>,
    /// Keys currently being rendered by the worker (0 or 1 today, a set so the
    /// type does not encode the single-worker assumption).
    in_flight: std::collections::BTreeSet<RenderKey>,
    /// Keys that failed this session: never retried (poison), mirroring the
    /// canvas/thumbnail rule.
    failed: std::collections::BTreeSet<RenderKey>,
    /// Keys already resident in the texture cache.
    ready: std::collections::BTreeSet<RenderKey>,
}

impl RenderQueueState {
    /// Enqueue unless already pending, ready, failed or in flight. Returns true
    /// when a job was added. Idempotent — the per-frame UI loop calls it freely.
    pub fn request(&mut self, job: RenderJob) -> bool;
    /// Next job for the worker: highest `priority`, then FIFO. Moves it to
    /// `in_flight`.
    pub fn take_next(&mut self) -> Option<RenderJob>;
    /// Record a completed render (moves key in_flight → ready).
    pub fn complete(&mut self, key: &RenderKey);
    /// Record a failure (moves key in_flight → failed).
    pub fn fail(&mut self, key: &RenderKey);
    /// Drop every queued/in-flight job whose `pdf_id` is not `active` — called
    /// when the project/document switches. Returns the dropped keys.
    pub fn retain_document(&mut self, active: &str) -> Vec<RenderKey>;
    /// A key is "known" when it is pending, in flight, ready or failed; the UI
    /// uses this to decide between texture, placeholder and "do not enqueue".
    pub fn is_known(&self, key: &RenderKey) -> bool;
}
```

Every behavioural rule above gets a unit test in-module (dedup, priority+FIFO,
complete/fail transitions, poison-not-retried, document switch drop, serde
roundtrip — see §6).

### 4.3 Data layer (`src/data/render_worker.rs`, I/O + threads)

```rust
/// Handle owned by the UI; the worker owns the mirror ends.
pub(crate) struct RenderWorker {
    jobs: mpsc::Sender<RenderJob>,
    results: mpsc::Receiver<ThumbPixels>,
}

/// Decoded thumbnail pixels for one key (not a domain type: raw bytes).
pub(crate) struct ThumbPixels {
    pub key: RenderKey,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,          // straight-alpha, row-major
    pub cache_path: String,
    pub outcome: Result<(), String>, // Err ⇒ Failed; rgba empty
}

impl RenderWorker {
    pub fn spawn(ctx: egui::Context) -> Self;
    pub fn submit(&self, job: RenderJob);           // UI → worker
    pub fn try_drain(&self) -> Vec<ThumbPixels>;    // non-blocking worker → UI
}
```

Worker loop, per job:
1. `data::pdf::render_page_to_cache(path, page, width)` → `cache_path`.
2. `std::fs::read(&cache_path)` → `image::load_from_memory` → `to_rgba8`.
3. Send `ThumbPixels` with dims + raw bytes + `Ok(())`.
4. `ctx.request_repaint()`.
On any error: send `ThumbPixels { outcome: Err(msg), rgba: vec![], .. }` and
still repaint. The worker never panics on a bad page (mirrors the `Result`
policy in `data/pdf.rs`).

Doing the **decode on the worker** moves the last blocking step off the UI
thread: the UI only builds a `ColorImage::from_rgba_unmultiplied` and calls
`load_texture` (an upload, not a decode). This is the main reason to prefer
pixels-over-channel rather than path-over-channel; the payload is a single
192 px thumbnail (~190 KB RGBA worst case), trivial for a channel.

The worker is also the natural future home for the canvas page-background render
(`ui/canvas.rs`) — same engine, same single-thread rule. That migration is a
**follow-up slice** (it needs a canvas-side "background pending" placeholder);
this note only reserves the seam by keeping the worker generic over `RenderKey`.

### 4.4 UI layer (`src/ui/pages.rs`, `src/ui/app.rs`, `src/ui/app_pages.rs`)

`ThumbCache` grows from a texture map into an async-aware cache:

```rust
pub(crate) struct ThumbCache {
    textures: HashMap<RenderKey, TextureHandle>,
    queue: RenderQueueState,       // domain state machine
    lru: TextureLru,               // in-memory eviction (§5.1)
}
```

Per frame in `draw_pages`:
1. Capture `current_page` and the visible row range (already computed from
   `thumb_layout`).
2. For each visible cell, build its `RenderKey`. If `textures` has it → paint.
   Else if `queue.is_known` and failed → numbered placeholder. Else paint the
   placeholder **and** `queue.request(RenderJob{ key, priority })` and
   `worker.submit(...)` when `request` returned `true`.
3. Priority: `0` for pages in view, `-distance_to_current` otherwise, so the
   page the user is looking at renders first; scrolling pre-fetches a small
   look-ahead window (`current_page ± 1 row`).
4. After the panel, drain results: for each `ThumbPixels`, on `Ok` decode-to-
   `ColorImage` + `load_texture` + `textures.insert` + `lru.touch`; on `Err`
   `queue.fail`, leave the placeholder (no status spam — the canvas already
   owns background error messaging).

`app_pages.rs` passes the worker handle into `draw_pages` (a `&RenderWorker`
borrow beside the `&mut ThumbCache`), and `app.rs` owns the `RenderWorker`.
`RubricApp::install_project` / document switch calls
`queue.retain_document(new_pdf_id)` so stale jobs are dropped (fixes the case
where switching PDFs while a render is queued paints the wrong page).

`begin_render_thumb`, the dead `RenderJob`/`RenderResult`/`RepaintSignal` types
and the `render: Arc<Mutex<RenderQueue>>` field are **removed** and replaced by
this worker + `RenderQueueState`. Nothing else references them (§2 table), so
this is a safe deletion.

### 4.5 Integration points with the existing P1-2 scaffolding

- The singleton `RenderQueue` becomes `RenderQueueState` in **domain** (pure,
  tested) plus the `RenderWorker` handle in **data** (threads). The current
  `in_flight: bool` becomes the `in_flight` set; `completed_key: Option<String>`
  becomes the `ready` set; `result: Option<RenderResult>` is removed (results
  travel on the channel).
- Keep the `update-check` pattern verbatim: clone `Context`, `request_repaint`
  on completion — the one thing the stub forgot.
- Keep `render_page_to_cache` and its cache layout unchanged; the worker only
  consumes its public API, so the `hayro`/PDFium engine split stays invisible.

## 5. Cache eviction policy

There are **two** caches to bound; the on-disk one is shared with the canvas and
the export/print paths, so its policy must not evict a page an export is about
to read (exports run synchronously and read the cache immediately, so a
protected "active document" rule is sufficient).

### 5.1 In-memory texture cache (`ThumbCache.textures`)

Unbounded today. A 192 px-wide letter thumbnail is ~192×249×4 ≈ 190 KB of GPU
texture; 300 pages ≈ 57 MB. Policy:

- **Capacity**: `MAX_THUMB_TEXTURES = 64` (≈12 MB), configurable const.
- **Recency**: `TextureLru { order: VecDeque<RenderKey> }`; `touch` moves a key
  to the back, `insert` pushes back and evicts from the front while
  `len > cap`. Evicted keys are removed from `textures` and **removed from
  `queue.ready`**, so a re-scroll re-enqueues cheaply (the PNG is still on disk,
  so it is a fast re-decode, not a re-render).
- **Never evict** a key in the current visible row set (the UI touches all
  visible keys each frame before eviction, which naturally protects them).

Pure, testable type: `TextureLru` (`touch`, `insert`, `evicted()`), unit-tested
for capacity and LRU order.

### 5.2 On-disk render cache (`render_cache/<pdf_key>/…`)

Unbounded today and never pruned. Because the directory key already changes when
a PDF's content changes (`data/pdf.rs:1315-1331`), the cache is naturally
"one directory per distinct PDF revision", and deleting whole directories is
consistent (no orphaned pages) and cheap.

Policy: **LRU by directory, whole-directory deletion**, with an age cap.

- Budgets: `MAX_DISK_CACHE_BYTES = 512 MiB`, `MAX_DISK_CACHE_AGE = 30 days`.
- `last_used` = the directory's mtime, refreshed by touching the directory when
  a page is served from it (best-effort; a failure to touch is ignored).
- **Protected**: the active document's `pdf_key` is never evicted.
- Algorithm (pure, in domain):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskCacheEntry { pub pdf_key: String, pub bytes: u64, pub last_used_secs: i64 }

/// Select directories to delete. Never returns a protected key. Expired
/// entries go first (oldest first), then least-recently-used until the total
/// of the survivors is within `budget_bytes`. Deterministic (stable sort).
pub fn plan_disk_eviction(
    entries: &[DiskCacheEntry],
    budget_bytes: u64,
    max_age_secs: i64,
    now_secs: i64,
    protected: &[String],
) -> Vec<String>;
```

- **Trigger**: run once shortly after startup (a few seconds in, so it never
  competes with first paint) and then at most every 10 minutes, from the UI
  tick; the scan itself is small (one `read_dir` + `metadata` per PDF).
- The scan (`data` layer) enumerates `render_cache_dir()`, builds
  `Vec<DiskCacheEntry>`, calls `plan_disk_eviction`, then `fs::remove_dir_all`
  for each returned key. All I/O stays in `data/`; the selection is pure.

Why whole-directory LRU rather than per-file: (a) the cache key already bundles
a PDF revision, so per-file granularity only helps within one PDF, where pages
are all similarly useful; (b) it avoids partially-rendered documents; (c) it is
a single deterministic, testable decision.

## 6. Acceptance criteria

Implementation is accepted when **all** hold, verified by CI (never locally)
plus review:

1. **No UI-thread rasterisation or decode.** The Pages path no longer calls
   `render_page_to_cache`, `fs::read` or `image::load_from_memory` on the UI
   thread; all such calls live in `data/render_worker.rs`. (Review + a grep
   guard in CI is optional; the module split enforces it by construction.)
2. **`RenderQueueState` semantics.** `request` is idempotent (dedup across
   pending/in-flight/ready/failed); `take_next` returns highest priority then
   FIFO and moves the key to in-flight; `complete`/`fail` transition correctly;
   a failed key is never re-enqueued; `retain_document` drops non-active keys.
   Unit-tested.
3. **`TextureLru`.** Never exceeds capacity; evicts least-recently-touched;
   repeated `touch` reorders without growing. Unit-tested.
4. **`plan_disk_eviction`.** Never returns a protected key; evicts expired before
   fresh; survivors fit `budget_bytes`; deterministic ordering. Unit-tested.
5. **Serde roundtrip** for every new domain type (`RenderKey`, `RenderJob`,
   `RenderOutcome`, `RenderQueueState`, `DiskCacheEntry`, `TextureLru`).
6. **Behaviour preserved.** Non-PDF projects and render failures show the
   numbered placeholder; clicking a cell jumps; the current page is highlighted;
   a failed render is not retried per frame.
7. **No stall.** Worker calls `ctx.request_repaint()` on every completion, so a
   finished thumbnail appears on the next frame without a manual interaction.
8. **Staleness.** Switching the open document while renders are queued drops the
   stale jobs (`retain_document`), so no wrong-page image is painted.
9. **Thread safety.** All engine calls are serialised on the single worker; if
   kept on PDFium, the `thread_safe` cargo feature is enabled (or the async path
   is gated to the `hayro` feature). Recorded in `DECISIONS.md`.
10. **Quality gates green:** fmt, `clippy -- -D warnings`, tests, release build.

## 7. Implementation plan (vertical slices)

Following `AGENTS.md`, one small slice per PR, domain first, file ownership
partitioned so parallel work never collides.

- **Slice 1 — domain (owner: `domain-dev`).** New `src/domain/thumb_render.rs`:
  `RenderKey`, `RenderJob`, `RenderOutcome`, `RenderQueueState`, `TextureLru`,
  `DiskCacheEntry`, `plan_disk_eviction`. No UI, no I/O. All unit tests + serde
  roundtrips from §6.2–6.5. Registers the module in `domain/mod.rs`.
- **Slice 2 — worker (owner: `fixer`/tooling).** New
  `src/data/render_worker.rs`; enable `pdfium-render` `thread_safe` feature in
  `apps/rubric/Cargo.toml` (or gate to `hayro`); unit-test the pure parts and an
  integration test that renders one page of a tiny fixture PDF through the
  worker and asserts non-empty pixels (headless, no display).
- **Slice 3 — UI wiring (owner: `ui-dev`).** Rewrite `ui/pages.rs::load_thumb`
  into the async request/drain loop; extend `ThumbCache`; thread the
  `RenderWorker` through `app_pages.rs`; remove the dead `RenderQueue`/`RenderJob`
  /`RenderResult`/`RepaintSignal` and `begin_render_thumb`. Preserve the existing
  tests/behaviour (placeholder, click-to-jump, highlight).
- **Slice 4 — disk eviction (owner: `fixer`/tooling).** Startup + periodic scan
  in `data/`, calling `plan_disk_eviction`; touch-on-serve. Unit test the scan
  against a temp dir.
- **Slice 5 — canvas background (follow-up, owner: `ui-dev`).** Route the canvas
  page background through the same worker with a "background pending"
  placeholder. Separate slice because it is a distinct visible behaviour.
- **Slice 6 — QA/review (owners: `test-writer`, `reviewer`).** Edge cases: rapid
  scrolling, PDF switch mid-render, failed page, 500-page document (only the
  visible window holds textures), empty/non-PDF project; accessibility note for
  the pending state (`docs/a11y.md`).

## 8. Risks and open questions

- **PDFium `thread_safe` enablement** changes a dependency feature; CI
  `cargo-deny`/SBOM should be unaffected (same crate/version) but the reviewer
  must confirm. Alternative: gate async thumbnails to the `hayro` feature until
  hayro becomes the default engine — safe but leaves the PDFium default
  synchronous.
- **`render_page_to_cache` binds PDFium per call** (`data/pdf.rs:356`); on the
  dedicated worker that is one bind on the worker thread, which is exactly the
  serialised model. If the UI thread has already initialised the global
  bindings (import path), the worker's `bind_pdfium()` takes the
  `PdfiumLibraryBindingsAlreadyInitialized` branch and reuses them — hence the
  `thread_safe` recommendation as the hard guarantee.
- **Immediate vs delayed first paint.** Keeping the very first thumbnail
  synchronous would avoid a blank placeholder on open, but reintroduces the
  freeze. Recommendation: always async, with an honest numbered placeholder
  (already the non-PDF fallback), consistent with the audit's "keep placeholder"
  guidance (`docs/audits/design-gui-2026-11.md`).
- **Cache budget values** (64 textures, 512 MiB, 30 days) are first estimates;
  they are constants, easy to tune after hands-on use. They are pure inputs to
  the tested selection function, so tuning does not touch logic.

## 9. Sources (public)

- `pdfium-render` public API docs — thread-safety statement and the `thread_safe`
  feature: <https://docs.rs/pdfium-render/latest/pdfium_render/>.
- `egui` / `eframe` 0.37 public docs (Context is `Clone + Send + Sync`;
  `load_texture`): <https://docs.rs/egui/0.37>.
- `hayro` (pure-Rust renderer, already an optional engine):
  <https://docs.rs/hayro>.
- In-repo evidence: `ui/pages.rs`, `ui/canvas.rs`, `ui/app.rs`,
  `data/pdf.rs`, `data/render.rs`, `domain/thumbs.rs`,
  `docs/audit-ui-2026-11-07.md` (P1-2), `docs/audits/design-gui-2026-11.md`.
