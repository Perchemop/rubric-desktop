//! Domain types for async thumbnail rendering.
//! Pure, serialisable, no threads, no I/O — fully headless-testable.

use serde::{Deserialize, Serialize};

/// Identity of one thumbnail render: enough to dedup, cache and poison.
/// Display form is stable and matches the existing UI key
/// `"<pdf path>:<page>:<width px>"`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
        if self.capacity == 0 {
            return;
        }
        if let Some(pos) = self.order.iter().position(|k| k == &key) {
            self.order.remove(pos);
        }
        self.order.push_back(key);
        while self.order.len() > self.capacity {
            let _ = self.order.pop_front();
        }
    }

    pub fn insert(&mut self, key: RenderKey) -> Option<RenderKey> {
        if self.capacity == 0 {
            // Nothing can be retained: report the incoming key as evicted
            // instead of panicking on the empty order queue.
            return Some(key);
        }
        let evicted = if self.order.len() >= self.capacity {
            self.order.pop_front()
        } else {
            None
        };
        self.order.push_back(key);
        evicted
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}

/// Disk render-cache budget: 512 MiB of page PNGs across all cached PDF
/// revisions (async-thumbnails design §5.2; first estimate, a pure input to
/// [`plan_disk_eviction`] so tuning never touches logic).
pub const MAX_DISK_CACHE_BYTES: u64 = 512 * 1024 * 1024;

/// Disk render-cache age cap: 30 days without a served page.
pub const MAX_DISK_CACHE_AGE_SECS: i64 = 30 * 24 * 60 * 60;

/// One per-PDF cache directory, summarised for the eviction planner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskCacheEntry {
    /// The `<pdf_key>` directory name under the render-cache root.
    pub pdf_key: String,
    /// Total bytes of cached files in the directory.
    pub bytes: u64,
    /// Directory mtime as unix seconds; refreshed when a page is served.
    pub last_used_secs: i64,
}

/// Select whole cache directories to delete. Never returns a protected key.
/// Expired entries go first (oldest first), then least-recently-used until
/// the total of the survivors is within `budget_bytes`. Deterministic
/// (stable sort; ties break by `pdf_key` ascending so identical input yields
/// identical output).
///
/// A key is *expired* when `now_secs - last_used_secs > max_age_secs` (an age
/// exactly at the cap is not expired). Protected keys are never selected,
/// however old or however large the cache.
pub fn plan_disk_eviction(
    entries: &[DiskCacheEntry],
    budget_bytes: u64,
    max_age_secs: i64,
    now_secs: i64,
    protected: &[String],
) -> Vec<String> {
    let is_protected = |key: &str| protected.iter().any(|p| p == key);

    // Oldest first, ties by pdf_key — one comparator drives both the expired
    // phase (age order) and the LRU phase (recency order).
    let oldest_first = |a: &DiskCacheEntry, b: &DiskCacheEntry| {
        (a.last_used_secs, a.pdf_key.as_str()).cmp(&(b.last_used_secs, b.pdf_key.as_str()))
    };

    let mut expired: Vec<&DiskCacheEntry> = entries
        .iter()
        .filter(|e| {
            !is_protected(&e.pdf_key) && now_secs.saturating_sub(e.last_used_secs) > max_age_secs
        })
        .collect();
    expired.sort_by(oldest_first);

    // Every key that will survive regardless of what is deleted below
    // (protected keys and fresh keys) seeds the survivor total.
    let mut survivors: u64 = entries
        .iter()
        .filter(|e| {
            is_protected(&e.pdf_key) || now_secs.saturating_sub(e.last_used_secs) <= max_age_secs
        })
        .map(|e| e.bytes)
        .sum();

    let mut fresh: Vec<&DiskCacheEntry> = entries
        .iter()
        .filter(|e| {
            !is_protected(&e.pdf_key) && now_secs.saturating_sub(e.last_used_secs) <= max_age_secs
        })
        .collect();
    fresh.sort_by(oldest_first);

    let mut selected: Vec<String> = expired.into_iter().map(|e| e.pdf_key.clone()).collect();

    // Least-recently-used fresh keys are deleted while the survivors still
    // exceed the budget; once the total fits, the remaining (more recent)
    // keys are all kept.
    for entry in fresh {
        if survivors > budget_bytes {
            survivors = survivors.saturating_sub(entry.bytes);
            selected.push(entry.pdf_key.clone());
        }
    }
    selected
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
    fn texture_lru_zero_capacity_never_panics() {
        let mut lru = TextureLru::new(0);
        let evicted = lru.insert(RenderKey {
            pdf_id: "a".into(),
            page: 0,
            width_px: 64,
        });
        assert!(evicted.is_some(), "a zero-capacity LRU evicts everything");
        assert_eq!(lru.len(), 0);
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
    fn serde_roundtrip_texture_lru_zero_capacity() {
        let mut lru = TextureLru::new(0);
        let evicted = lru.insert(RenderKey {
            pdf_id: "a".into(),
            page: 0,
            width_px: 64,
        });
        assert!(evicted.is_some(), "a zero-capacity LRU evicts everything");

        let json = serde_json::to_string(&lru).unwrap();
        let back: TextureLru = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 0);
        assert_eq!(lru, back);
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

    #[test]
    fn serde_roundtrip_render_queue_state() {
        let mut sq = RenderQueueState::default();
        sq.request(RenderJob {
            key: RenderKey {
                pdf_id: "p".into(),
                page: 0,
                width_px: 128,
            },
            priority: 0,
        });
        sq.request(RenderJob {
            key: RenderKey {
                pdf_id: "q".into(),
                page: 0,
                width_px: 128,
            },
            priority: 1,
        });
        // Higher priority wins: "q" moves to in_flight, "p" stays pending.
        let taken = sq.take_next().unwrap();
        assert_eq!(taken.key.pdf_id, "q");

        let json = serde_json::to_string(&sq).unwrap();
        let back: RenderQueueState = serde_json::from_str(&json).unwrap();
        assert_eq!(sq, back);
    }

    // --- Disk-cache eviction (async-thumbnails Slice 4) --------------------

    /// Synthetic entry helper: real `DiskCacheEntry` values without any I/O,
    /// so expiry/LRU/budget semantics are fully controlled by the test.
    fn entry(pdf_key: &str, bytes: u64, last_used_secs: i64) -> DiskCacheEntry {
        DiskCacheEntry {
            pdf_key: pdf_key.into(),
            bytes,
            last_used_secs,
        }
    }

    #[test]
    fn serde_roundtrip_disk_cache_entry() {
        let made = entry("abc123", 4096, 1_700_000_000);
        let json = serde_json::to_string(&made).unwrap();
        let back: DiskCacheEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(made, back);
    }

    #[test]
    fn plan_disk_eviction_empty_input_returns_empty() {
        let out = plan_disk_eviction(&[], 512, 100, 1_000, &[]);
        assert!(out.is_empty());
    }

    #[test]
    fn plan_disk_eviction_expired_before_fresh() {
        // "old" is expired (1000 - 800 > 100), "fresh" is not (1000 - 900 =
        // 100 is exactly the cap, which is NOT expired). The age cap acts
        // independently of the budget: "old" goes even though nothing else
        // exists and the budget is far from exhausted.
        let entries = vec![entry("fresh", 10, 900), entry("old", 10, 800)];
        let out = plan_disk_eviction(&entries, 1_000, 100, 1_000, &[]);
        assert_eq!(out, vec!["old".to_string()]);
    }

    #[test]
    fn plan_disk_eviction_expired_oldest_first_with_tie_break() {
        // All three are expired; they are returned oldest first and equal
        // ages break by pdf_key ascending ("a" before "b").
        let entries = vec![
            entry("b", 10, 700),
            entry("a", 10, 700),
            entry("c", 10, 600),
        ];
        let out = plan_disk_eviction(&entries, 1_000, 100, 1_000, &[]);
        assert_eq!(out, vec!["c".to_string(), "a".to_string(), "b".to_string()]);
    }

    #[test]
    fn plan_disk_eviction_budget_enforced() {
        // 3 x 10 = 30 bytes total against a 15-byte budget: the two
        // least-recently-used keys are dropped, the most recent survives and
        // the survivors' total fits the budget.
        let entries = vec![
            entry("recent", 10, 900),
            entry("mid", 10, 800),
            entry("old", 10, 700),
        ];
        let out = plan_disk_eviction(&entries, 15, 10_000, 1_000, &[]);
        assert_eq!(out, vec!["old".to_string(), "mid".to_string()]);
        let survivors: u64 = entries
            .iter()
            .filter(|e| !out.iter().any(|k| k == &e.pdf_key))
            .map(|e| e.bytes)
            .sum();
        assert!(survivors <= 15);
        assert_eq!(survivors, 10);
    }

    #[test]
    fn plan_disk_eviction_budget_exact_boundary() {
        // Survivors exactly at the budget fit: nothing is deleted when the
        // total equals the budget; one byte over forces the least-recently-
        // used deletion.
        let entries = vec![entry("a", 10, 800), entry("b", 10, 900)];
        assert!(plan_disk_eviction(&entries, 20, 10_000, 1_000, &[]).is_empty());
        let out = plan_disk_eviction(&entries, 19, 10_000, 1_000, &[]);
        assert_eq!(out, vec!["a".to_string()]);
    }

    #[test]
    fn plan_disk_eviction_protected_never_returned() {
        // "active" is expired AND alone exceeds the budget, yet it must never
        // be selected; "old" is still removed.
        let entries = vec![entry("active", 1_000_000, 100), entry("old", 10, 100)];
        let out = plan_disk_eviction(&entries, 1, 100, 1_000, &["active".to_string()]);
        assert_eq!(out, vec!["old".to_string()]);
        assert!(!out.iter().any(|k| k.as_str() == "active"));
    }

    #[test]
    fn plan_disk_eviction_all_expired_returns_all_except_protected() {
        // "keep_me" is the oldest of all but is protected; the other two are
        // returned oldest first.
        let entries = vec![
            entry("keep_me", 10, 100),
            entry("gone1", 10, 200),
            entry("gone2", 10, 300),
        ];
        let out = plan_disk_eviction(&entries, 1_000, 100, 1_000, &["keep_me".to_string()]);
        assert_eq!(out, vec!["gone1".to_string(), "gone2".to_string()]);
    }

    #[test]
    fn plan_disk_eviction_tie_breaks_by_pdf_key_deterministically() {
        // Equal last_used_secs: the tie breaks by pdf_key ascending, so
        // identical input always yields identical output.
        let entries = vec![entry("b", 10, 700), entry("a", 10, 700)];
        let first = plan_disk_eviction(&entries, 10, 10_000, 1_000, &[]);
        let second = plan_disk_eviction(&entries, 10, 10_000, 1_000, &[]);
        assert_eq!(first, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(first, second);
    }
}
