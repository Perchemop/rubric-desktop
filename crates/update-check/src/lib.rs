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
//! * When a newer release publishes a checksum manifest, its minisign
//!   signature is verified against the embedded release key and the outcome
//!   is attached to [`UpdateStatus::Available`]; every signature problem
//!   fails soft and never breaks the update check itself.

use serde::Deserialize;
use std::time::{Duration, Instant};

pub mod signature;

pub use signature::SignatureState;

/// How long a successful check stays valid before it may be refreshed.
pub const CACHE_TTL: Duration = Duration::from_secs(60 * 60);

/// GitHub API base; overridable in tests.
pub const RELEASES_API: &str = "https://api.github.com/repos";

/// Release asset holding the checksum manifest the signature covers
/// (published by the release workflow next to `SHA256SUMS`).
const MANIFEST_ASSET: &str = "SHA256SUMS.txt";

/// Release asset holding the minisign signature over [`MANIFEST_ASSET`].
const SIGNATURE_ASSET: &str = "SHA256SUMS.txt.minisig";

/// Outcome of a check. `Unknown` covers every failure mode so the UI can show
/// one honest "couldn't check" state instead of a stack of error variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// Running version is the newest published release.
    UpToDate,
    /// A newer release exists; carries its tag, URL and the state of its
    /// checksum-manifest signature.
    Available {
        latest_version: String,
        url: String,
        signature: SignatureState,
    },
    /// The check could not be completed (offline, rate-limited, bad payload…).
    Unknown,
}

/// One downloadable asset attached to a release (subset of the GitHub
/// payload).
#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseAsset {
    /// File name, e.g. `SHA256SUMS.txt.minisig`.
    #[serde(default)]
    pub name: String,
    /// Direct download URL for the asset.
    #[serde(default)]
    pub browser_download_url: String,
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
    /// Assets attached to the release; empty when the payload omits them.
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
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

/// One network round trip plus the comparison. When a newer release carries
/// the checksum manifest and its signature, they are fetched and the
/// signature verified too; every extra fetch problem fails soft.
fn check_now(repo: &str, current: &str) -> UpdateStatus {
    let url = format!("{RELEASES_API}/{repo}/releases/latest");
    let resp = match ureq::get(&url)
        .header("User-Agent", "rubric-update-check")
        .header("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(r) => r,
        // Offline, DNS failure, timeout, or a non-2xx HTTP status: fail soft.
        Err(_) => return UpdateStatus::Unknown,
    };
    // ureq 3.x treats non-2xx status codes as errors, so a successful call is 2xx.
    let release: Release = match resp.into_body().read_json() {
        Ok(r) => r,
        Err(_) => return UpdateStatus::Unknown,
    };
    // Keep the checksum-manifest asset URLs before `finish` consumes the
    // release; they are only fetched when a newer release is offered, and a
    // fetch failure must never break the update check itself.
    let checksum_urls = checksum_asset_urls(&release.assets);
    let status = finish(repo, current, release);
    match status {
        UpdateStatus::Available {
            latest_version,
            url,
            ..
        } => UpdateStatus::Available {
            latest_version,
            url,
            signature: fetch_and_verify_signature(checksum_urls.as_ref()),
        },
        other => other,
    }
}

/// The browser download URLs of a release's checksum manifest and its
/// minisign signature, when the release carries both assets.
fn checksum_asset_urls(assets: &[ReleaseAsset]) -> Option<(String, String)> {
    let manifest = assets
        .iter()
        .find(|a| a.name == MANIFEST_ASSET && !a.browser_download_url.is_empty())?;
    let signature = assets
        .iter()
        .find(|a| a.name == SIGNATURE_ASSET && !a.browser_download_url.is_empty())?;
    Some((manifest.browser_download_url.clone(), signature.browser_download_url.clone()))
}

/// GET a URL and read the body as text; `None` on any network problem. The
/// signature fetch must never break the update check, so callers treat every
/// failure the same as a missing asset.
fn fetch_text(url: &str) -> Option<String> {
    ureq::get(url)
        .header("User-Agent", "rubric-update-check")
        .call()
        .ok()?
        .into_body()
        .read_to_string()
        .ok()
}

/// Fetch the checksum manifest and its signature and verify the signature.
/// Fail-soft: a missing or unfetchable asset reports
/// [`SignatureState::Missing`]; a present-but-bad signature reports
/// [`SignatureState::Failed`].
fn fetch_and_verify_signature(urls: Option<&(String, String)>) -> SignatureState {
    let Some((manifest_url, sig_url)) = urls else {
        return SignatureState::Missing;
    };
    let Some(manifest) = fetch_text(manifest_url) else {
        return SignatureState::Missing;
    };
    let Some(sig) = fetch_text(sig_url) else {
        return SignatureState::Missing;
    };
    signature::verify_manifest(&manifest, &sig)
}

/// The comparison half of a check, split out so it is testable without a
/// network. `repo` is only used to build the release URL. `finish` never
/// touches the network, so an `Available` result always starts with
/// [`SignatureState::Missing`]; `check_now` upgrades it after fetching.
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
            signature: SignatureState::Missing,
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
            assets: Vec::new(),
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
                signature,
            } => {
                assert_eq!(latest_version, "v0.2.0");
                assert!(url.contains("/tag/v0.2.0"));
                // `finish` is pure (no network), so the signature state can
                // only start as Missing; `check_now` upgrades it.
                assert_eq!(signature, SignatureState::Missing);
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
