//! Data-layer worker thread for off-page-thumbnail rasterisation.
//! Pure orchestration + I/O; domain types live in `src/domain/thumb_render.rs`.

use std::sync::mpsc;
use std::time::Duration;

use crate::data::pdf::render_page_to_cache;
use crate::domain::thumb_render::{
    RenderJob, RenderKey, RenderOutcome, RenderQueueState, TextureLru,
};
use crate::ui::pages::ThumbCache;
use egui::Context;

// ---- Worker state (owned by the thread) ----

/// Mirror ends of the channel owned by the UI.
pub struct RenderWorker {
    /// UI → worker: enqueue a render job.
    jobs: mpsc::Sender<RenderJob>,
    /// Worker → UI: receive decoded thumbnail pixels.
    results: mpsc::Receiver<ThumbPixels>,
}

/// Decoded thumbnail pixels crossing the purity boundary.
/// The worker sends raw RGBA bytes; the UI uploads them as egui textures.
#[derive(Debug, Clone)]
pub struct ThumbPixels {
    pub key: RenderKey,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,      // straight‑alpha, row‑major, size = width*height*4
    pub cache_path: String, // path to the on‑disk PNG
    pub outcome: Result<(), String>,
}

impl RenderWorker {
    /// Spawn a single long‑lived worker thread.
    /// The caller (UI setup) owns the `Sender`; the worker owns the `Receiver`.
    pub fn spawn(ctx: &egui::Context) -> Self {
        let ctx = ctx.clone();
        let (jobs, job_rx) = mpsc::channel::<RenderJob>(64);
        let (result_tx, results) = mpsc::channel::<ThumbPixels>(64);

        // ---- Worker loop ----
        std::thread::spawn(move || {
            loop {
                // Blocking receive — the UI only enqueues keys it does not already
                // have pending/loaded, so channel depth stays small.
                if let Ok(job) = job_rx.recv() {
                    // 1️⃣ Render the page to the on‑disk cache.
                    let cache_path =
                        render_page_to_cache(&job.key.pdf_id, job.key.page, job.key.width_px);

                    // 2️⃣ Decode the PNG.
                    let rgba = match std::fs::read(&cache_path) {
                        Ok(data) => match image::load_from_memory(&data) {
                            Ok(img) => img.to_bytes(image::ImageOutputFormat::RawRgba8).to_vec(),
                            Err(e) => {
                                // Record failure but still repaint next frame.
                                let _ = result_tx.send(ThumbPixels {
                                    key: job.key,
                                    width: job.key.width_px,
                                    height: 0,
                                    rgba: vec![],
                                    cache_path,
                                    outcome: Err(e.to_string()),
                                });
                                continue;
                            }
                        },
                        Err(e) => {
                            // Send failure payload and repaint.
                            let _ = result_tx.send(ThumbPixels {
                                key: job.key,
                                width: job.key.width_px,
                                height: 0,
                                rgba: vec![],
                                cache_path,
                                outcome: Err(format!("cache read error: {}", e)),
                            });
                            continue;
                        }
                    };

                    // 3️⃣ Send pixels back to the UI and request a repaint.
                    let _ = result_tx.send(ThumbPixels {
                        key: job.key,
                        width: job.key.width_px,
                        height: rgba.len() as u32 / (4 * job.key.width_px as u32),
                        rgba,
                        cache_path,
                        outcome: Ok(()),
                    });

                    // Request repaint so the UI can display the new texture next frame.
                    ctx.request_repaint();
                }
            }
        });

        RenderWorker { jobs, results }
    }

    /// UI → worker: enqueue a render job.
    pub fn submit(&self, job: RenderJob) {
        let _ = self.jobs.send(job);
    }

    /// Non‑blocking drain: returns any completed results since last call.
    /// The UI typically calls this each frame after `draw_pages`.
    pub fn try_drain(&self) -> Vec<ThumbPixels> {
        let mut collected = Vec::new();
        while let Ok(pixel) = self.results.try_recv() {
            collected.push(pixel);
        }
        collected
    }
}

// ---- UI‑side handle (owned by the app) ----

/// Handle stored by the UI; the worker owns the mirror ends.
pub struct RenderWorkerHandle {
    /// Sender end (owned by UI).
    jobs: mpsc::Sender<RenderJob>,
    /// Receiver end (mirror, owned by worker).
    _results: mpsc::Receiver<ThumbPixels>,
}

impl RenderWorkerHandle {
    /// Submit a render job for a given key. Idempotent — the domain state
    /// machine in `RenderQueueState` deduplicates across pending/in‑flight/ready/failed.
    pub fn submit(&self, key: RenderKey, priority: i32) {
        let job = RenderJob { key, priority };
        let _ = self.jobs.send(job);
    }
}
