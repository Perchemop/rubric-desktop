//! Data-layer worker thread for off-page-thumbnail rasterisation.
//! Pure orchestration + I/O; domain types live in `src/domain/thumb_render.rs`.

use std::sync::mpsc;

use crate::data::pdf::render_page_to_cache;
use crate::domain::thumb_render::{RenderJob, RenderKey};

// ---- Worker state (owned by the thread) ----

/// Channel ends kept by the UI; the worker thread owns the other halves.
pub struct RenderWorker {
    /// UI -> worker: enqueue a render job.
    jobs: mpsc::Sender<RenderJob>,
    /// Worker -> UI: receive decoded thumbnail pixels.
    results: mpsc::Receiver<ThumbPixels>,
}

/// Decoded thumbnail pixels crossing the purity boundary.
/// The worker sends raw RGBA bytes; the UI uploads them as egui textures.
#[derive(Debug, Clone)]
pub struct ThumbPixels {
    pub key: RenderKey,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,      // straight-alpha, row-major, size = width*height*4
    pub cache_path: String, // path to the on-disk PNG
    pub outcome: Result<(), String>,
}

impl RenderWorker {
    /// Spawn a single long-lived worker thread.
    /// The caller (UI setup) only keeps these channel ends; the worker owns
    /// the matching receiver and sender, so there is no second receiver.
    pub fn spawn(ctx: &egui::Context) -> Self {
        let ctx = ctx.clone();
        let (jobs, job_rx) = mpsc::channel::<RenderJob>(64);
        let (result_tx, results) = mpsc::channel::<ThumbPixels>(64);

        // ---- Worker loop ----
        std::thread::spawn(move || {
            // Blocking receive; the UI only enqueues keys it does not already
            // have pending/loaded, so channel depth stays small. When every
            // sender drops the channel disconnects and the loop ends instead
            // of busy-spinning.
            while let Ok(job) = job_rx.recv() {
                let pixels = render_thumb(&job);
                let _ = result_tx.send(pixels);
                // Wake the UI so it picks up the result (or the failure) on
                // the next frame.
                ctx.request_repaint();
            }
        });

        RenderWorker { jobs, results }
    }

    /// UI -> worker: enqueue a render job.
    pub fn submit(&self, job: RenderJob) {
        let _ = self.jobs.send(job);
    }

    /// Non-blocking drain: returns any completed results since last call.
    /// The UI typically calls this each frame after `draw_pages`.
    pub fn try_drain(&self) -> Vec<ThumbPixels> {
        let mut collected = Vec::new();
        while let Ok(pixel) = self.results.try_recv() {
            collected.push(pixel);
        }
        collected
    }
}

/// Render one job end to end, mapping every failure to an honest `Err`
/// payload so the UI can poison that key instead of retrying forever.
fn render_thumb(job: &RenderJob) -> ThumbPixels {
    // 1. Render the page into the on-disk PNG cache.
    let cache_path = match render_page_to_cache(&job.key.pdf_id, job.key.page, job.key.width_px) {
        Ok(path) => path,
        Err(e) => return failed_pixels(job, format!("cache render error: {}", e)),
    };

    // 2. Read the PNG back and decode it.
    let img = match std::fs::read(&cache_path) {
        Ok(data) => match image::load_from_memory(&data) {
            Ok(img) => img,
            Err(e) => return failed_pixels(job, format!("decode error: {}", e)),
        },
        Err(e) => return failed_pixels(job, format!("cache read error: {}", e)),
    };

    // 3. Convert to straight-alpha RGBA and send back to the UI. Width and
    //    height are the ACTUAL decoded dimensions; the requested width may
    //    have been clamped by the cache layer.
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();
    ThumbPixels {
        key: job.key.clone(),
        width,
        height,
        rgba,
        cache_path: cache_path.display().to_string(),
        outcome: Ok(()),
    }
}

/// Build a failure payload: empty pixels, a zero height and the render error.
/// On failure the usable cache path is unknown, so it stays empty.
fn failed_pixels(job: &RenderJob, msg: String) -> ThumbPixels {
    ThumbPixels {
        key: job.key.clone(),
        width: job.key.width_px,
        height: 0,
        rgba: Vec::new(),
        cache_path: String::new(),
        outcome: Err(msg),
    }
}
