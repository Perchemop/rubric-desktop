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
        let (jobs, job_rx) = mpsc::channel::<RenderJob>();
        let (result_tx, results) = mpsc::channel::<ThumbPixels>();

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
    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();
    let rgba = rgba8.into_raw();
    ThumbPixels {
        key: job.key.clone(),
        width,
        height,
        rgba,
        cache_path: cache_path.display().to_string(),
        outcome: Ok(()),
    }
}

/// Build a failure payload: all-empty/zero. The payload echoes nothing
/// usable back to the UI (empty pixels, zero width and height, no cache
/// path) plus the render error so the key can be poisoned instead of
/// retried forever.
fn failed_pixels(job: &RenderJob, msg: String) -> ThumbPixels {
    ThumbPixels {
        key: job.key.clone(),
        width: 0,
        height: 0,
        rgba: Vec::new(),
        cache_path: String::new(),
        outcome: Err(msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A zero render width is rejected by `render_page_to_cache` before any
    /// PDFium binding or filesystem access, so this failure path is fully
    /// deterministic and headless. Every field of the returned payload must
    /// be shaped for the UI: the original key echoed back, no pixels, no
    /// cache path, zero width and height and an honest non-empty error
    /// message.
    #[test]
    fn render_thumb_maps_failures_to_shaped_err_payload() {
        let job = RenderJob {
            key: RenderKey {
                pdf_id: "/nonexistent/definitely-missing.pdf".into(),
                page: 0,
                width_px: 0,
            },
            priority: 0,
        };

        let pixels = render_thumb(&job);

        assert!(pixels.outcome.is_err(), "zero width must fail the render");
        let message = pixels.outcome.unwrap_err();
        assert!(!message.is_empty(), "failure payload must carry a message");
        assert_eq!(pixels.key, job.key);
        // The failure payload zeroes both dimensions regardless of the
        // requested width, echoing nothing usable back to the UI.
        assert_eq!(pixels.width, 0);
        assert_eq!(pixels.height, 0);
        assert!(pixels.rgba.is_empty());
        assert!(pixels.cache_path.is_empty());
    }
}
