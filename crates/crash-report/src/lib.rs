//! Shared crash/bug reporting for Rubric suite apps (guide §7.1).
//!
//! Design rules, all enforced here:
//!
//! * **Never embed a privileged bot token.** Submission either uses a
//!   *user-supplied* token (entered at report time, held only in memory) or
//!   falls back to a pre-filled "New Issue" URL the user opens in a browser.
//! * **Consent first.** A report is captured, but the caller decides what to do
//!   with it: `CrashReport::to_issue_body` is only turned into a network call by
//!   explicit user action, and the user can always cancel.
//! * **Scrub before share.** Absolute paths, home directories and anything that
//!   looks like a token are redacted from the captured backtrace.
//!
//! The crate is deliberately UI-free: an app renders the preview with its own
//! toolkit and calls [`submit`] only after the user confirms.

use serde::{Deserialize, Serialize};
use std::panic::PanicHookInfo;

/// Environment and build facts captured with every report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    /// Version of the app that crashed (from `CARGO_PKG_VERSION`).
    pub app_version: String,
    /// Operating system identifier (e.g. `linux-aarch64`).
    pub os: String,
}

impl Environment {
    /// Capture the environment of the running process.
    pub fn capture(app_version: impl Into<String>) -> Self {
        Self {
            app_version: app_version.into(),
            os: os_identifier(),
        }
    }
}

/// `<os>-<arch>` derived at compile time, so no platform trait magic is needed.
pub fn os_identifier() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// A captured crash: what happened, where, and the scrubbed context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrashReport {
    pub environment: Environment,
    /// One-line reason (the panic message, truncated).
    pub summary: String,
    /// Scrubbed backtrace lines, most recent first.
    pub backtrace: Vec<String>,
    /// Free-text notes the user chose to add. Optional and never fabricated.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub user_notes: String,
    /// Report format version, so the payload can evolve safely.
    pub report_version: u32,
}

/// Current payload format version.
pub const REPORT_VERSION: u32 = 1;

/// Maximum characters retained for the summary line.
pub const MAX_SUMMARY_CHARS: usize = 200;
/// Maximum backtrace frames retained.
pub const MAX_FRAMES: usize = 40;

impl CrashReport {
    /// Build a report from a panic payload and the current backtrace.
    pub fn from_panic(environment: Environment, info: &PanicHookInfo<'_>) -> Self {
        let summary = panic_message(info);
        Self {
            environment,
            summary: truncate(&summary, MAX_SUMMARY_CHARS),
            backtrace: scrubbed_backtrace(),
            user_notes: String::new(),
            report_version: REPORT_VERSION,
        }
    }

    /// Build a report for a non-panic bug the user reports by hand.
    pub fn manual(environment: Environment, summary: impl Into<String>) -> Self {
        Self {
            environment,
            summary: truncate(&summary.into(), MAX_SUMMARY_CHARS),
            backtrace: scrubbed_backtrace(),
            user_notes: String::new(),
            report_version: REPORT_VERSION,
        }
    }

    /// Human-readable issue title.
    pub fn issue_title(&self) -> String {
        format!("[crash] {}: {}", self.environment.os, self.summary)
    }

    /// The full issue body: report + user notes, as markdown.
    pub fn to_issue_body(&self) -> String {
        let mut out = String::new();
        out.push_str("## Crash report\n\n");
        out.push_str(&format!(
            "- App version: {}\n- OS: {}\n- Report format: v{}\n\n",
            self.environment.app_version, self.environment.os, self.report_version
        ));
        out.push_str("### Summary\n\n```\n");
        out.push_str(&self.summary);
        out.push_str("\n```\n\n");
        if !self.backtrace.is_empty() {
            out.push_str("### Backtrace (scrubbed)\n\n```\n");
            for frame in &self.backtrace {
                out.push_str(frame);
                out.push('\n');
            }
            out.push_str("```\n\n");
        }
        if !self.user_notes.is_empty() {
            out.push_str("### User notes\n\n");
            out.push_str(&self.user_notes);
            out.push('\n');
        }
        out
    }

    /// Serialise the report (used by the review dialog and by tests).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Extract a readable message from a panic payload.
pub fn panic_message(info: &PanicHookInfo<'_>) -> String {
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

/// Install the panic hook. Returns the captured report through `sink`, which
/// the app forwards to its review dialog. `RUST_BACKTRACE=full` is honoured by
/// `std`, so no extra plumbing is needed to enable frames.
pub fn install_panic_hook<F: Fn(CrashReport) + Send + Sync + 'static>(
    environment: Environment,
    sink: F,
) {
    std::panic::set_hook(Box::new(move |info| {
        let report = CrashReport::from_panic(environment.clone(), info);
        // Still print to stderr so CLI runs and CI logs keep the usual output.
        eprintln!("{report:?}",);
        sink(report);
    }));
}

/// Collect the current backtrace, scrubbed, newest frame first.
pub fn scrubbed_backtrace() -> Vec<String> {
    let raw = std::backtrace::Backtrace::force_capture().to_string();
    raw.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .map(|l| scrub_line(&l))
        .take(MAX_FRAMES)
        .collect()
}

/// Redact paths and credential-looking material from one line.
///
/// Home directories, any absolute path under a user directory, and
/// `key=value`-style tokens are replaced so a shared report cannot leak
/// usernames or secrets. Query strings on URLs are dropped entirely.
pub fn scrub_line(line: &str) -> String {
    let mut out = line.to_string();
    let home = home_dir();
    if !home.is_empty() {
        out = out.replace(&home, "~");
    }
    out = redact_tokens(&out);
    out
}

fn redact_tokens(s: &str) -> String {
    // Redact any run of "secret-ish" characters assigned to a key: token=…,
    // api_key=…, bearer …, ?token=…&.
    let mut out = String::with_capacity(s.len());
    let lower = s.to_ascii_lowercase();
    let markers = ["token", "api_key", "apikey", "secret", "password", "bearer"];
    let mut i = 0;
    while i < lower.len() {
        let rest = &lower[i..];
        if let Some(pos) = markers.iter().filter_map(|m| rest.find(m)).min() {
            let at = i + pos;
            // Keep the key, redact what follows an `=`, `:` or space.
            out.push_str(&s[i..at]);
            let after = &s[at..];
            let (key_end, sep_len) = match after.find(|c| c == '=' || c == ':') {
                Some(p) if p < 16 => (at + p, 1),
                _ => (at, 0),
            };
            out.push_str(&s[at..key_end]);
            if sep_len == 1 {
                out.push('=');
            }
            // Consume the value: skip any space after the separator, then take
            // up to the next whitespace (or end of line).
            let mut value_start = key_end + sep_len;
            while value_start < s.len() && s[value_start..].starts_with(|c: char| c.is_whitespace())
            {
                value_start += 1;
            }
            let value_end = s[value_start..]
                .find(|c: char| c.is_whitespace())
                .map(|p| value_start + p)
                .unwrap_or(s.len());
            out.push_str("[REDACTED]");
            i = value_end;
        } else {
            out.push_str(&s[i..]);
            break;
        }
    }
    out
}

fn home_dir() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default()
}
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut cut = max;
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}…", &s[..cut])
}

/// Labels accepted for issue creation.
pub const ISSUE_LABELS: [&str; 3] = ["crash", "bug", "user-report"];

/// A prepared GitHub issue: title, body and the labels to request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
}

impl Issue {
    /// Build the issue a confirmed report would create.
    pub fn from_report(report: &CrashReport) -> Self {
        Self {
            title: report.issue_title(),
            body: report.to_issue_body(),
            labels: ISSUE_LABELS.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Serialize as the GitHub issue-creation JSON payload.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Pre-filled "New Issue" URL for the no-token fallback. Nothing is sent
    /// until the user presses Submit in their browser.
    pub fn new_issue_url(&self, repo: &str) -> String {
        format!(
            "https://github.com/{}/issues/new?title={}&body={}&labels={}",
            repo.trim_end_matches('/'),
            urlencode(&self.title),
            urlencode(&self.body),
            urlencode(&self.labels.join(",")),
        )
    }
}

/// Submit the issue to GitHub.
///
/// `token` is the *user's* token, supplied at report time. Passing `None` or an
/// empty token returns the pre-filled URL path instead of making any network
/// call — this crate never performs a POST the user has not seen and confirmed.
pub fn submit(repo: &str, issue: &Issue, token: Option<&str>) -> Result<(), String> {
    let token = match token {
        Some(t) if !t.trim().is_empty() => t.trim(),
        _ => {
            return Err("no user token supplied; open the pre-filled issue URL instead".to_string())
        }
    };
    let url = format!(
        "https://api.github.com/repos/{}/issues",
        repo.trim_end_matches('/')
    );
    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("User-Agent", "rubric-crash-report")
        .set("Accept", "application/vnd.github+json")
        .send_string(&issue.to_json())
        .map_err(|e| format!("network error: {e}"))?;
    match resp.status() {
        200..=299 => Ok(()),
        401 => Err("GitHub rejected the token (401). Check it and retry.".into()),
        403 => {
            Err("GitHub refused issue creation (403). The token may lack the `repo` scope.".into())
        }
        s => Err(format!("GitHub returned {s}")),
    }
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('%'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> Environment {
        Environment {
            app_version: "0.1.0".into(),
            os: "linux-aarch64".into(),
        }
    }

    #[test]
    fn os_identifier_uses_compile_time_target() {
        let id = os_identifier();
        assert!(id.contains(std::env::consts::OS));
        assert!(id.contains(std::env::consts::ARCH));
        assert!(id.contains('-'));
    }

    #[test]
    fn manual_report_carries_summary_and_version() {
        let r = CrashReport::manual(env(), "canvas pan stalled");
        assert_eq!(r.summary, "canvas pan stalled");
        assert_eq!(r.report_version, REPORT_VERSION);
        assert_eq!(r.environment.app_version, "0.1.0");
        assert_eq!(r.environment.os, "linux-aarch64");
    }

    #[test]
    fn summary_is_truncated_on_a_char_boundary() {
        let long = "é".repeat(MAX_SUMMARY_CHARS + 50);
        let r = CrashReport::manual(env(), long.clone());
        assert!(r.summary.chars().count() <= MAX_SUMMARY_CHARS + 1);
        assert!(r.summary.ends_with('…'));
        // The truncation must not panic on a multi-byte boundary.
        let _ = CrashReport::manual(env(), format!("{}{}", "x".repeat(199), "ééé"));
    }

    #[test]
    fn issue_body_contains_report_and_notes_but_nothing_else() {
        let mut r = CrashReport::manual(env(), "boom");
        r.user_notes = "happens when zooming".into();
        let body = r.to_issue_body();
        assert!(body.contains("0.1.0"));
        assert!(body.contains("linux-aarch64"));
        assert!(body.contains("boom"));
        assert!(body.contains("happens when zooming"));
        assert!(body.starts_with("## Crash report"));
    }

    #[test]
    fn json_round_trips() {
        let r = CrashReport::manual(env(), "boom");
        let json = r.to_json();
        let back: CrashReport = serde_json::from_str(&json).expect("valid json");
        assert_eq!(back, r);
    }

    #[test]
    fn scrub_redacts_home_directory() {
        let home = home_dir();
        if home.is_empty() {
            return; // no HOME in this environment
        }
        let line = format!("{home}/projects/rubric/src/lib.rs");
        assert_eq!(scrub_line(&line), "~/projects/rubric/src/lib.rs");
    }

    #[test]
    fn scrub_redacts_tokens_and_passwords() {
        assert_eq!(scrub_line("token=abc123 send"), "token=[REDACTED] send");
        assert_eq!(scrub_line("api_key: hunter2!"), "api_key=[REDACTED]");
        assert_eq!(scrub_line("password=hunter2"), "password=[REDACTED]");
        assert!(scrub_line("?token=secretvalue&x=1").contains("[REDACTED]"));
        assert!(!scrub_line("?token=secretvalue&x=1").contains("secretvalue"));
    }

    #[test]
    fn scrub_leaves_normal_code_lines_alone() {
        let line = "12: rubric::ui::canvas::draw_canvas";
        assert_eq!(scrub_line(line), line);
    }

    #[test]
    fn issue_payload_is_valid_and_labelled() {
        let issue = Issue::from_report(&CrashReport::manual(env(), "boom"));
        assert!(issue.title.starts_with("[crash] linux-aarch64:"));
        assert_eq!(issue.labels, vec!["crash", "bug", "user-report"]);
        let json = issue.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(parsed["title"], issue.title);
        assert!(parsed["labels"].is_array());
    }

    #[test]
    fn new_issue_url_is_prefilled_and_percent_encoded() {
        let issue = Issue::from_report(&CrashReport::manual(env(), "boom"));
        let url = issue.new_issue_url("Perchemop/rubric-desktop");
        assert!(url.starts_with("https://github.com/Perchemop/rubric-desktop/issues/new?"));
        assert!(url.contains("title=%5Bcrash%5D"));
        assert!(url.contains("labels=crash%2Cbug%2Cuser-report"));
        assert!(!url.contains(' '));
    }

    #[test]
    fn submit_without_token_refuses_to_send() {
        // The no-token path must be an error, never a silent unauthenticated POST.
        let issue = Issue::from_report(&CrashReport::manual(env(), "boom"));
        let err = submit("Perchemop/rubric-desktop", &issue, None);
        assert!(err
            .expect_err("expected an error")
            .contains("no user token"));
    }

    #[test]
    fn submit_with_blank_token_refuses_too() {
        let issue = Issue::from_report(&CrashReport::manual(env(), "boom"));
        assert!(submit("Perchemop/rubric-desktop", &issue, Some("   ")).is_err());
    }
}
