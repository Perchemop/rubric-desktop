//! Shared update checking for Rubric suite apps (guide §7.2).
//!
//! Behaviour contract:
//!
//! * Queries the **GitHub Releases API** for the latest published (non-draft,
//!   non-prerelease) release and compares it with the running version.
//! * **Fails soft offline** — any network or parse problem becomes an
//!   `UpdateStatus::Unknown`, never an error dialog and never a retry loop.
//! * **No API abuse** — results are cached in memory for [`CACHE_TTL`] and the
//!   caller is expected to invoke this on an explicit user action or at most
//!   once per session.
//! * The comparison is *semantic*, not string ordering, so `0.2.0` correctly
//!   beats `0.10.0`.

use serde::Deserialize;
use std::time::{Duration, Instant};

/// How long a successful check stays valid before it may be refreshed.
pub const CACHE_TTL: Duration = Duration::from_secs(60 * 60);

/// GitHub API base; overridable in tests.
pub const RELEASES_API: &str = "https://api.github.com/repos";

/// Outcome of a check. `Unknown` covers every failure mode so the UI can show
/// one honest "couldn't check" state instead of a stack of error variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// Running version is the newest published release.
    UpToDate,
    /// A newer release exists; carries its tag and URL.
    Available { latest_version: String, url: String },
    /// The check could not be completed (offline, rate-limited, bad payload…).
    Unknown,
}

/// The subset of the GitHub release payload this crate needs.
#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    #[serde(default)]
    pub tag_name: String,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
}

/// Parsed semantic version (major.minor.patch, optional pre-release suffix).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    /// Parse `v?MAJOR.MINOR.PATCH(-suffix)?`. Suffixes are ignored for
    /// ordering; unparseable input yields `None` rather than a wrong answer.
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        let raw = raw.strip_prefix('v').unwrap_or(raw);
        let core = raw.split(['-', '+']).next()?;
        let mut parts = core.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

/// Compare two versions: `Ordering::Greater` when `a` is newer than `b`.
///
/// Exposed for tests and for callers that want to show a diff direction.
pub fn compare(a: &str, b: &str) -> Option<std::cmp::Ordering> {
    Some(Version::parse(a)?.cmp(&Version::parse(b)?))
}

/// The record a successful check leaves behind for the cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedCheck {
    pub status: UpdateStatus,
    pub at: Instant,
}

/// Cached, rate-limit-friendly checker.
#[derive(Debug, Default)]
pub struct UpdateChecker {
    cache: Option<CachedCheck>,
}

impl UpdateChecker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check for updates. `repo` is `owner/name`; `current` is the running
    /// version. Returns a cached result while [`CACHE_TTL`] is unexpired.
    ///
    /// Blocking: call from a worker thread, never from the UI loop (the app
    /// spawns it and posts the result back).
    pub fn check(&mut self, repo: &str, current: &str) -> UpdateStatus {
        if let Some(cached) = &self.cache {
            if cached.at.elapsed() < CACHE_TTL {
                return cached.status.clone();
            }
        }
        let status = check_now(repo, current);
        if !matches!(status, UpdateStatus::Unknown) {
            self.cache = Some(CachedCheck {
                status: status.clone(),
                at: Instant::now(),
            });
        }
        status
    }

    /// Test/CLI hook: run one check with no caching.
    pub fn check_uncached(&self, repo: &str, current: &str) -> UpdateStatus {
        check_now(repo, current)
    }

    /// Forget any cached result (used by "check again now").
    pub fn invalidate(&mut self) {
        self.cache = None;
    }

    /// The cached status, if fresh. Pure; never touches the network.
    pub fn cached(&self) -> Option<&UpdateStatus> {
        self.cache
            .as_ref()
            .filter(|c| c.at.elapsed() < CACHE_TTL)
            .map(|c| &c.status)
    }
}

/// One network round trip plus the comparison.
fn check_now(repo: &str, current: &str) -> UpdateStatus {
    let url = format!("{RELEASES_API}/{repo}/releases/latest");
    let resp = match ureq::get(&url)
        .set("User-Agent", "rubric-update-check")
        .set("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(r) => r,
        // Offline, DNS failure, timeout…: fail soft.
        Err(_) => return UpdateStatus::Unknown,
    };
    if resp.status() != 200 {
        // 404 (no releases yet), rate-limited, and other HTTP conditions all
        // mean the same thing to the user: "could not check".
        return UpdateStatus::Unknown;
    }
    let release: Release = match resp.into_json() {
        Ok(r) => r,
        Err(_) => return UpdateStatus::Unknown,
    };
    finish(repo, current, release)
}

/// The comparison half of a check, split out so it is testable without a
/// network. `repo` is only used to build the release URL.
fn finish(_repo: &str, current: &str, release: Release) -> UpdateStatus {
    if release.draft || release.prerelease || release.tag_name.is_empty() {
        // Not a published release: nothing to offer the user.
        return UpdateStatus::UpToDate;
    }
    let Some(latest) = Version::parse(&release.tag_name) else {
        return UpdateStatus::Unknown;
    };
    let Some(running) = Version::parse(current) else {
        return UpdateStatus::Unknown;
    };
    if latest > running {
        UpdateStatus::Available {
            latest_version: release.tag_name.clone(),
            url: release.html_url.clone(),
        }
    } else {
        UpdateStatus::UpToDate
    }
}

/// Choose the newest release from a pre-fetched list (test helper and offline
/// fallback for CI-driven checks).
pub fn pick_latest(releases: &[Release]) -> Option<Release> {
    releases
        .iter()
        .filter(|r| !r.draft && !r.prerelease)
        .filter(|r| Version::parse(&r.tag_name).is_some())
        .max_by(|a, b| {
            Version::parse(&a.tag_name)
                .unwrap()
                .cmp(&Version::parse(&b.tag_name).unwrap())
        })
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    fn rel(tag: &str) -> Release {
        Release {
            tag_name: tag.to_string(),
            html_url: format!("https://github.com/x/y/releases/tag/{tag}"),
            draft: false,
            prerelease: false,
        }
    }

    #[test]
    fn version_parses_plain_and_prefixed() {
        assert_eq!(
            Version::parse("1.2.3"),
            Some(Version {
                major: 1,
                minor: 2,
                patch: 3
            })
        );
        assert_eq!(
            Version::parse("v0.1.0"),
            Some(Version {
                major: 0,
                minor: 1,
                patch: 0
            })
        );
        assert_eq!(Version::parse("  v2.10.0 "), Version::parse("2.10.0"));
    }

    #[test]
    fn version_handles_prerelease_and_build_suffixes() {
        assert_eq!(Version::parse("1.2.3-rc.1"), Version::parse("1.2.3"));
        assert_eq!(Version::parse("1.2.3+build.9"), Version::parse("1.2.3"));
    }

    #[test]
    fn version_rejects_garbage() {
        assert_eq!(Version::parse(""), None);
        assert_eq!(Version::parse("abc"), None);
        assert_eq!(Version::parse("1.2"), None);
        assert_eq!(Version::parse("1.2.3.4"), None);
        assert_eq!(Version::parse("1.2.x"), None);
    }

    #[test]
    fn compare_is_semantic_not_lexicographic() {
        assert_eq!(compare("0.2.0", "0.10.0"), Some(Ordering::Less));
        assert_eq!(compare("0.10.0", "0.2.0"), Some(Ordering::Greater));
        assert_eq!(compare("1.0.0", "1.0.0"), Some(Ordering::Equal));
        assert_eq!(compare("0.1.1", "0.1.0"), Some(Ordering::Greater));
        assert_eq!(compare("nope", "0.1.0"), None);
    }

    #[test]
    fn pick_latest_ignores_drafts_prereleases_and_bad_tags() {
        let mut bad = rel("1.4.0");
        bad.draft = true;
        let mut pre = rel("9.9.9");
        pre.prerelease = true;
        let releases = [rel("0.1.0"), bad, pre, rel("0.2.0"), rel("garbage")];
        assert_eq!(pick_latest(&releases).unwrap().tag_name, "0.2.0");
    }

    #[test]
    fn pick_latest_empty_is_none() {
        assert!(pick_latest(&[]).is_none());
    }

    #[test]
    fn release_payload_deserialises_from_github_shape() {
        let json = r#"{
            "tag_name": "v0.2.0",
            "html_url": "https://github.com/x/y/releases/tag/v0.2.0",
            "draft": false,
            "prerelease": false,
            "name": "Rubric 0.2.0",
            "id": 123
        }"#;
        let r: Release = serde_json::from_str(json).expect("deserialise");
        assert_eq!(r.tag_name, "v0.2.0");
        assert!(!r.draft);
    }

    #[test]
    fn release_payload_missing_fields_defaults_safely() {
        let r: Release = serde_json::from_str("{}").expect("deserialise");
        assert_eq!(r.tag_name, "");
        assert!(!r.draft);
    }

    #[test]
    fn finish_reports_newer_release() {
        let status = finish("x/y", "0.1.0", rel("v0.2.0"));
        match status {
            UpdateStatus::Available {
                latest_version,
                url,
            } => {
                assert_eq!(latest_version, "v0.2.0");
                assert!(url.contains("/tag/v0.2.0"));
            }
            other => panic!("expected Available, got {other:?}"),
        }
    }

    #[test]
    fn finish_reports_up_to_date_when_running_is_newest() {
        assert_eq!(
            finish("x/y", "0.2.0", rel("v0.2.0")),
            UpdateStatus::UpToDate
        );
        assert_eq!(
            finish("x/y", "0.3.0", rel("v0.2.0")),
            UpdateStatus::UpToDate
        );
        // Semantic, not lexicographic: running 0.10.0 beats published 0.2.0.
        assert_eq!(
            finish("x/y", "0.10.0", rel("v0.2.0")),
            UpdateStatus::UpToDate
        );
    }

    #[test]
    fn finish_ignores_drafts_prereleases_and_empty_tags() {
        let mut draft = rel("v9.0.0");
        draft.draft = true;
        assert_eq!(finish("x/y", "0.1.0", draft), UpdateStatus::UpToDate);
        let mut pre = rel("v9.0.0");
        pre.prerelease = true;
        assert_eq!(finish("x/y", "0.1.0", pre), UpdateStatus::UpToDate);
        assert_eq!(finish("x/y", "0.1.0", rel("")), UpdateStatus::UpToDate);
    }

    #[test]
    fn finish_is_unknown_for_unparseable_tags_or_versions() {
        assert_eq!(
            finish("x/y", "0.1.0", rel("garbage")),
            UpdateStatus::Unknown
        );
        assert_eq!(
            finish("x/y", "dev-build", rel("v0.2.0")),
            UpdateStatus::Unknown
        );
    }

    #[test]
    fn checker_caches_within_ttl_and_respects_invalidate() {
        let mut checker = UpdateChecker::new();
        assert!(checker.cached().is_none());
        // Seed the cache directly: no network in unit tests.
        checker.cache = Some(CachedCheck {
            status: UpdateStatus::UpToDate,
            at: Instant::now(),
        });
        assert_eq!(checker.cached(), Some(&UpdateStatus::UpToDate));
        checker.invalidate();
        assert!(checker.cached().is_none());
    }
}
