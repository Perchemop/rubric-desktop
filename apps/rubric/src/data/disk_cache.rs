//! On-disk render-cache eviction (async-thumbnails Slice 4).
//!
//! Policy (`docs/research/async-thumbnails.md` §5.2): LRU by whole
//! `render_cache/<pdf_key>/` directory, with a byte budget, an age cap and
//! active-document protection. The selection itself is pure domain logic
//! ([`plan_disk_eviction`]); this module owns all the I/O around it — the
//! scan, the directory deletes and the use-refresh "touch". Everything here
//! is best-effort: a cache that cannot be scanned or pruned is never an
//! error the user sees.

use crate::domain::thumb_render::{
    plan_disk_eviction, DiskCacheEntry, MAX_DISK_CACHE_AGE_SECS, MAX_DISK_CACHE_BYTES,
};

/// The per-PDF cache-key directory name for `pdf_path`, derived exactly as
/// the cache writer derives it (`pdf::pdf_cache_key`: hash of canonical
/// path, file length and mtime). `None` when it cannot be derived — a
/// missing or unreadable file simply has no key to protect.
pub(crate) fn active_pdf_key(pdf_path: &std::path::Path) -> Option<String> {
    crate::data::pdf::pdf_cache_key(pdf_path).ok()
}

/// Scan the render-cache root: one [`DiskCacheEntry`] per per-PDF
/// subdirectory. Files directly under the root are skipped (the layout is
/// `render_cache/<pdf_key>/page_<i>_<width>.png`). `last_used_secs` is the
/// directory's mtime as unix seconds — whatever the filesystem reports; std
/// cannot set directory mtimes, so tests cannot control it. Best-effort:
/// entries whose metadata fails are skipped.
pub(crate) fn scan_entries(root: &std::path::Path) -> Vec<DiskCacheEntry> {
    let Ok(read) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    for dir in read.flatten() {
        let path = dir.path();
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if !meta.is_dir() {
            continue;
        }
        entries.push(DiskCacheEntry {
            pdf_key: dir.file_name().to_string_lossy().into_owned(),
            bytes: dir_bytes(&path),
            last_used_secs: mtime_secs(&path),
        });
    }
    entries
}

/// Scan, plan and delete: whole cache directories per the eviction plan.
/// Returns how many directories were actually removed. The active PDF's key
/// — when derivable — is never selected.
pub(crate) fn evict_now(active_pdf: Option<&std::path::Path>) -> usize {
    match crate::data::pdf::render_cache_dir() {
        Some(root) => evict_in(&root, active_pdf),
        None => 0,
    }
}

/// [`evict_now`] against an explicit root — the seam the tests use instead
/// of touching the real per-user cache directory.
fn evict_in(root: &std::path::Path, active_pdf: Option<&std::path::Path>) -> usize {
    let entries = scan_entries(root);
    let protected: Vec<String> = active_pdf.and_then(active_pdf_key).into_iter().collect();
    let now_secs = chrono::Utc::now().timestamp();
    let selected = plan_disk_eviction(
        &entries,
        MAX_DISK_CACHE_BYTES,
        MAX_DISK_CACHE_AGE_SECS,
        now_secs,
        &protected,
    );
    let mut removed = 0;
    for key in &selected {
        if std::fs::remove_dir_all(root.join(key)).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Refresh the per-PDF cache directory's mtime so the LRU reflects use.
///
/// std cannot set mtimes, so the refresh recreates a marker file inside the
/// directory: creating (or replacing) an entry updates the parent
/// directory's mtime on every supported platform. Best-effort — every error
/// is ignored; a failed touch only leaves an older last-used time.
pub(crate) fn touch_served(cache_png: &std::path::Path) {
    let Some(dir) = cache_png.parent() else {
        return;
    };
    let marker = dir.join(".touched");
    let _ = std::fs::remove_file(&marker);
    let _ = std::fs::File::create(&marker);
}

/// Total size of every file under `dir`, recursively (the layout is flat
/// today, but the recursion costs nothing and survives layout changes).
fn dir_bytes(dir: &std::path::Path) -> u64 {
    let Ok(read) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut total = 0;
    for item in read.flatten() {
        let Ok(meta) = item.metadata() else {
            continue;
        };
        if meta.is_dir() {
            total = total.saturating_add(dir_bytes(&item.path()));
        } else if meta.is_file() {
            total = total.saturating_add(meta.len());
        }
    }
    total
}

/// Directory mtime as unix seconds, or 0 when unavailable — a 0 sorts as
/// "ancient" in the eviction planner, which is the honest fallback for a
/// directory whose age cannot be read.
fn mtime_secs(path: &std::path::Path) -> i64 {
    std::fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temp render-cache-like root that removes itself (best-effort) when
    /// the test ends. Unique per test run via the process id, so concurrent
    /// test binaries do not collide.
    struct TempRoot(std::path::PathBuf);

    impl TempRoot {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "rubric_disk_cache_{}_{}",
                std::process::id(),
                name
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("create temp cache root");
            TempRoot(path)
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    // NOTE: std cannot set directory mtimes, so `last_used_secs` values in
    // these tests are whatever the filesystem reports. The *selection*
    // semantics (expiry ordering, LRU, budget, protection) are therefore
    // covered with synthetic entries in the domain tests
    // (`domain/thumb_render.rs`), per the Slice 4 brief.

    #[test]
    fn scan_entries_enumerates_dirs_and_byte_totals() {
        let root = TempRoot::new("scan");
        let doc = root.0.join("doc_a");
        std::fs::create_dir_all(&doc).expect("create per-pdf dir");
        std::fs::write(doc.join("page_0_192.png"), vec![0u8; 10]).expect("write page");
        std::fs::write(doc.join("page_1_192.png"), vec![0u8; 20]).expect("write page");
        // A stray file directly under the root is not a document directory.
        std::fs::write(root.0.join("stray"), b"x").expect("write stray");

        let entries = scan_entries(&root.0);
        assert_eq!(entries.len(), 1, "only directories are entries");
        assert_eq!(entries[0].pdf_key, "doc_a");
        assert_eq!(entries[0].bytes, 30);
        // mtime is filesystem-reported (uncontrollable from std); only
        // sanity-check it is a real unix-seconds value, not the fallback.
        assert!(entries[0].last_used_secs > 0);
    }

    #[test]
    fn evict_now_on_empty_tree_returns_zero_and_removes_nothing() {
        let root = TempRoot::new("evict_empty");
        assert_eq!(evict_in(&root.0, None), 0);
        assert!(scan_entries(&root.0).is_empty(), "nothing was removed");
    }

    #[test]
    fn evict_now_keeps_fresh_small_documents() {
        let root = TempRoot::new("evict_fresh");
        let doc = root.0.join("doc_b");
        std::fs::create_dir_all(&doc).expect("create per-pdf dir");
        std::fs::write(doc.join("page_0_192.png"), vec![0u8; 16]).expect("write page");
        // A freshly written directory is not expired (its mtime is "now")
        // and 16 bytes is nowhere near the 512 MiB budget, so nothing is
        // evicted — deterministically, without mtime control.
        assert_eq!(evict_in(&root.0, None), 0);
        assert!(doc.is_dir(), "the document directory survived");
        assert_eq!(scan_entries(&root.0).len(), 1);
    }

    #[test]
    fn touch_served_recreates_the_marker_file() {
        let root = TempRoot::new("touch");
        let doc = root.0.join("doc_c");
        std::fs::create_dir_all(&doc).expect("create per-pdf dir");
        let png = doc.join("page_0_192.png");
        std::fs::write(&png, b"png").expect("write page");

        touch_served(&png);
        assert!(doc.join(".touched").is_file(), "marker recreated");
        // Touching twice must not fail or pile up markers.
        touch_served(&png);
        assert!(doc.join(".touched").is_file());
    }

    #[test]
    fn active_pdf_key_derives_for_a_real_file() {
        let root = TempRoot::new("key");
        let pdf = root.0.join("doc.pdf");
        std::fs::write(&pdf, b"%PDF-1.4 not really").expect("write pdf");
        let key = active_pdf_key(&pdf).expect("key derivable for a real file");
        assert_eq!(key.len(), 16, "hash format is {:016x}");
        let again = active_pdf_key(&pdf).expect("stable across calls");
        assert_eq!(key, again, "same file, same key");
        assert!(
            active_pdf_key(&root.0.join("missing.pdf")).is_none(),
            "a missing file has no derivable key"
        );
    }
}
