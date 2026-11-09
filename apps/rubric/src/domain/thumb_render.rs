//! Domain types for async thumbnail rendering.
//! Pure, serialisable, no threads, no I/O — fully headless-testable.

use serde::{Deserialize, Serialize};

/// Identity of one thumbnail render: enough to dedup, cache and poison.
/// Display form is stable and matches the existing UI key
/// `"<pdf path>:<page>:<width px>"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderKey {
    pub pdf_id: String,
    pub page: usize,
    pub width_px: u32,
}

impl std::fmt::Display for RenderKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.pdf_id, self.page, self.width_px)
    }
}

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
    Ready {
        key: RenderKey,
        cache_path: String,
        width: u32,
        height: u32,
    },
    Failed {
        key: RenderKey,
        message: String,
    },
}

/// Queue + poison state for one active document. No threads, no I/O.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderQueueState {
    /// Keys enqueued and not yet completed, in the order they must run.
    pending: Vec<RenderJob>,
    /// Keys currently being rendered by the worker (0 or 1 today).
    in_flight: std::collections::BTreeSet<RenderKey>,
    /// Keys that failed this session: never retried (poison).
    failed: std::collections::BTreeSet<RenderKey>,
    /// Keys already resident in the texture cache.
    ready: std::collections::BTreeSet<RenderKey>,
}

impl RenderQueueState {
    /// Enqueue unless already pending, ready, failed or in flight. Returns true
    /// when a job was added. Idempotent — the per-frame UI loop calls it freely.
    pub fn request(&mut self, job: RenderJob) -> bool {
        if self.is_known(&job.key) {
            return false;
        }
        self.pending.push(job);
        true
    }

    /// Next job for the worker: highest `priority`, then FIFO. Moves it to
    /// `in_flight`.
    pub fn take_next(&mut self) -> Option<RenderJob> {
        if self.pending.is_empty() {
            return None;
        }
        let best_idx = self
            .pending
            .iter()
            .enumerate()
            .max_by_key(|(_, j)| j.priority)
            .map(|(i, _)| i)
            .unwrap();
        let job = self.pending.swap_remove(best_idx);
        self.in_flight.insert(job.key.clone());
        Some(job)
    }

    /// Record a completed render (moves key in_flight → ready).
    pub fn complete(&mut self, key: &RenderKey) {
        self.in_flight.remove(key);
        self.ready.insert(key.clone());
    }

    /// Record a failure (moves key in_flight → failed).
    pub fn fail(&mut self, key: &RenderKey) {
        self.in_flight.remove(key);
        self.failed.insert(key.clone());
    }

    /// Drop every queued/in-flight job whose `pdf_id` is not `active` — called
    /// when the project/document switches. Returns the dropped keys.
    pub fn retain_document(&mut self, active: &str) -> Vec<RenderKey> {
        let mut dropped = Vec::new();
        self.pending.retain(|j| {
            if j.key.pdf_id == active {
                true
            } else {
                dropped.push(j.key.clone());
                false
            }
        });
        self.in_flight.retain(|k| k.pdf_id == active);
        self.failed.retain(|k| k.pdf_id == active);
        self.ready.retain(|k| k.pdf_id == active);
        dropped
    }

    /// A key is "known" when it is pending, in flight, ready or failed; the UI
    /// uses this to decide between texture, placeholder and "do not enqueue".
    pub fn is_known(&self, key: &RenderKey) -> bool {
        self.pending.iter().any(|j| j.key == *key)
            || self.in_flight.contains(key)
            || self.ready.contains(key)
            || self.failed.contains(key)
    }
}

/// In-memory texture cache LRU with capacity limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureLru {
    /// Order of use; most-recently touched at the back.
    order: std::collections::VecDeque<RenderKey>,
    /// Capacity hard limit.
    capacity: usize,
}

impl TextureLru {
    pub const CAPACITY: usize = 64;

    pub fn new(capacity: usize) -> Self {
        TextureLru {
            order: std::collections::VecDeque::new(),
            capacity,
        }
    }

    pub fn touch(&mut self, key: RenderKey) {
        if let Some(pos) = self.order.iter().position(|k| k == &key) {
            self.order.remove(pos);
        }
        self.order.push_back(key);
    }

    pub fn insert(&mut self, key: RenderKey) -> Option<RenderKey> {
        if self.order.len() >= self.capacity {
            let evicted = self.order.pop_front().unwrap();
            self.order.push_back(key);
            Some(evicted)
        } else {
            self.order.push_back(key);
            None
        }
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_render_key() {
        let key = RenderKey {
            pdf_id: "/path/to.pdf".into(),
            page: 3,
            width_px: 192,
        };
        let json = serde_json::to_string(&key).unwrap();
        let back: RenderKey = serde_json::from_str(&json).unwrap();
        assert_eq!(key, back);
    }

    #[test]
    fn serde_roundtrip_render_job() {
        let job = RenderJob {
            key: RenderKey {
                pdf_id: "p".into(),
                page: 0,
                width_px: 128,
            },
            priority: 0,
        };
        let json = serde_json::to_string(&job).unwrap();
        let back: RenderJob = serde_json::from_str(&json).unwrap();
        assert_eq!(job, back);
    }

    #[test]
    fn serde_roundtrip_render_outcome_ready() {
        let ready = RenderOutcome::Ready {
            key: RenderKey {
                pdf_id: "p".into(),
                page: 0,
                width_px: 128,
            },
            cache_path: "/cache".into(),
            width: 128,
            height: 128,
        };
        let json = serde_json::to_string(&ready).unwrap();
        let back: RenderOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(ready, back);
    }

    #[test]
    fn serde_roundtrip_render_outcome_failed() {
        let failed = RenderOutcome::Failed {
            key: RenderKey {
                pdf_id: "p".into(),
                page: 0,
                width_px: 128,
            },
            message: "page not found".into(),
        };
        let json = serde_json::to_string(&failed).unwrap();
        let back: RenderOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(failed, back);
    }

    #[test]
    fn texture_lru_capacity_never_exceeds() {
        let mut lru = TextureLru::new(2);
        lru.touch(RenderKey {
            pdf_id: "a".into(),
            page: 0,
            width_px: 64,
        });
        lru.touch(RenderKey {
            pdf_id: "b".into(),
            page: 0,
            width_px: 64,
        });
        lru.touch(RenderKey {
            pdf_id: "c".into(),
            page: 0,
            width_px: 64,
        });
        assert_eq!(lru.len(), 2);
        assert!(!lru.order.iter().any(|k| k.pdf_id == "a"));
        assert!(lru.order.iter().any(|k| k.pdf_id == "b"));
        assert!(lru.order.iter().any(|k| k.pdf_id == "c"));
    }

    #[test]
    fn texture_lru_touch_reorders_without_growing() {
        let mut lru = TextureLru::new(2);
        lru.touch(RenderKey {
            pdf_id: "a".into(),
            page: 0,
            width_px: 64,
        });
        lru.touch(RenderKey {
            pdf_id: "b".into(),
            page: 0,
            width_px: 64,
        });
        lru.touch(RenderKey {
            pdf_id: "a".into(),
            page: 0,
            width_px: 64,
        });
        assert_eq!(lru.order.front().map(|k| k.pdf_id.as_str()), Some("b"));
        assert_eq!(lru.order.back().map(|k| k.pdf_id.as_str()), Some("a"));
    }

    #[test]
    fn render_queue_state_request_idempotent() {
        let mut sq = RenderQueueState::default();
        let job1 = RenderJob {
            key: RenderKey {
                pdf_id: "p".into(),
                page: 0,
                width_px: 128,
            },
            priority: 0,
        };
        let job2 = job1.clone();

        assert!(sq.request(job1)); // added
        assert!(!sq.request(job2)); // dedup, not added
    }

    #[test]
    fn render_queue_state_take_next_priority_fifo() {
        let mut sq = RenderQueueState::default();
        let job_high = RenderJob {
            key: RenderKey {
                pdf_id: "a".into(),
                page: 0,
                width_px: 64,
            },
            priority: 10,
        };
        let job_low = RenderJob {
            key: RenderKey {
                pdf_id: "b".into(),
                page: 0,
                width_px: 64,
            },
            priority: -5,
        };

        sq.request(job_low);
        sq.request(job_high);

        let taken = sq.take_next();
        assert!(taken.is_some());
        assert_eq!(taken.unwrap().key.pdf_id, "a"); // higher priority first
        let remaining = sq.take_next();
        assert!(remaining.is_some());
        assert_eq!(remaining.unwrap().key.pdf_id, "b"); // lower priority next
    }

    #[test]
    fn render_queue_state_complete_fail_transitions() {
        let mut sq = RenderQueueState::default();
        let job = RenderJob {
            key: RenderKey {
                pdf_id: "p".into(),
                page: 0,
                width_px: 128,
            },
            priority: 0,
        };
        sq.request(job);

        let taken = sq.take_next().unwrap();
        sq.complete(&taken.key);
        assert!(sq.is_known(&taken.key));
        assert!(sq.ready.contains(&taken.key));

        let job2 = RenderJob {
            key: RenderKey {
                pdf_id: "f".into(),
                page: 0,
                width_px: 64,
            },
            priority: 0,
        };
        sq.request(job2);
        let taken2 = sq.take_next().unwrap();
        sq.fail(&taken2.key);
        assert!(sq.is_known(&taken2.key));
        assert!(sq.failed.contains(&taken2.key));
    }

    #[test]
    fn render_queue_state_retain_document() {
        let mut sq = RenderQueueState::default();
        sq.request(RenderJob {
            key: RenderKey {
                pdf_id: "doc1".into(),
                page: 0,
                width_px: 128,
            },
            priority: 0,
        });
        sq.request(RenderJob {
            key: RenderKey {
                pdf_id: "doc2".into(),
                page: 0,
                width_px: 128,
            },
            priority: 0,
        });

        let dropped = sq.retain_document("doc1");
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].pdf_id, "doc2");
        assert!(!sq.is_known(&RenderKey {
            pdf_id: "doc2".into(),
            page: 0,
            width_px: 128,
        }));
    }
}
