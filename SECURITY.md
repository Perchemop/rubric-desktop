# Security Policy

## Reporting a vulnerability — privately, please

**Do not open a public issue for a security problem.**

Use GitHub's **private vulnerability reporting** (enabled on this
repository): *Security ▸ Report a vulnerability*. It creates a private
report with a private fork where a fix can be prepared, and lets us open a
draft advisory and publish it on disclosure.

Please include:

- Affected version, platform, and how you run Rubric (portable zip, AppImage, DMG, MSI).
- What an attacker would actually gain (file loss? data corruption? code execution?).
- Reproduction steps or a minimal trigger.
- Whether PDF import is involved (PDFium is the main untrusted input path).

## What to expect

- **Triage within 7 days** of a report.
- A severity decision and fix plan within another **7 days**.
- A fix releases as a patch (semver patch or minor, severity-dependent),
  and the advisory is published with credit to the reporter unless you ask
  to remain anonymous.
- This project is maintained in limited hours; complex findings may take
  longer, and we will keep you informed rather than go silent.

## Scope

**In scope:**

- The `.rubric.json` project format parser (serde deserialization of
  untrusted files) — malformed or hostile sidecars must not crash beyond a
  recoverable error or corrupt other data.
- Anything executed or loaded from a document the user opened: PDF page
  data rendered through PDFium, embedded markup geometry.
- The update-check client (`crates/update-check`): response handling must
  not misinterpret a spoofed / non-GitHub response beyond showing an error.
- The crash-report client (`crates/crash-report`): must never embed or leak
  credentials, must scrub paths from the captured text.

**Out of scope:**

- Vulnerabilities in the PDFium library itself — please report those
  upstream (https://github.com/bblanchon/pdfium-binaries for the binaries'
  acquisition path, and Chromium's PDFium tracker for the code), but do
  tell us so we can ship the fixed binary in bundles.
- Social-engineering of GitHub itself.
- The in-app "report a bug" browser flow landing on GitHub (user leaves the
  app on purpose; the pre-filled text is visible in the dialog before send).

## Threat model (lightweight, maintainer version)

Rubric is a **local-first single-user desktop editor**. It has no server,
no network listener, and inserts no external network traffic by default
except:

1. **update-check** — an HTTP GET to `api.github.com` that the user triggers
   explicitly (Help ▸ Check for Updates) and that sends no user data.
2. **crash-report** — a user-authored issue/ticket draft the user consciously
   sends or opens in a browser; content is previewed and editable before send.

Assets Rubric trusts, in order of exposure:

| Input | Trust | Misuse potential |
|---|---|---|
| PDF files (via PDFium) | Untrusted, sandboxed by library boundaries | Parsing bugs → crash or worse in PDFium; app must not aggregate the risk |
| `.rubric.json` sidecar files | Untrusted when shared by others | Deserialisation must be fail-soft: no panic, no data loss beyond the file |
| Preferences/annotations the user creates | Trusted (user's own) | N/A locally |
| GitHub API responses (update check) | Semi-trusted | Version-compare confusion must fail soft to "Unknown"; no user data sent |

Secrets: the app ships **no tokens**. A GitHub issue-creation flow uses a
user-supplied token only, held in memory, never persisted nor logged. The
distribution server is GitHub Releases, signed by GitHub's TLS only (binary
artefacts carry SHA256 checksums in `SHA256SUMS` for transport-integrity
verification, not signing).

## Incident response

Maintainer procedure when a vulnerability is confirmed:

1. **Triage** — maintainer confirms the report, judges severity
   (Critical: data destruction/RCE; High: session/local exfiltration;
   Medium: data corruption recoverable; Low: DoS/robustness).
2. **Mitigate or disable** — if exploitation is actively plausible, cut a
   hotfix release and, for release-artefact problems, mark affected assets
   obsolete at the release page.
3. **Fix** — prepare the fix on a branch/private fork, verify through CI
   (tests + build), then merge and release.
4. **Disclose** — publish the GitHub advisory within days of the fixed
   release; package maintainers get a heads-up in the release notes.
5. **Notify users** — the release mentions the fixed issue plainly; the
   in-app update check points at the fixed version.

Who does what: the single maintainer (see `GOVERNANCE.md`) triages,
decides mitigation, and coordinates disclosure. If the maintainer is
unreachable for 30 days on a Critical report, users should prefer server-side
 remedies: GitHub support can restrict repository access while the maintainer is absent.

## Security incidents

### 2026-10-28 — Personal data in published git history (resolved)

- **Class**: personal data (guide II.29) — an absolute maintainer home path,
  which embeds the operating-system account name. It is not a credential, so no
  secret was exposed and no token rotation was required.
- **Carriers**: agent configuration (`.opencode/agents/*.md`) and a tracked
  Python bytecode cache (`scripts/__pycache__/*.pyc`) that embedded the absolute
  source path. Both were committed and therefore published.
- **Detection**: a full four-part scan (working tree, history, published surface,
  metadata) run under the new guide II.29 gate.
- **Remediation**: removed from `HEAD` and all current files (PR #37); the
  bytecode cache was untracked and `*.pyc` / `__pycache__/` ignored (PR #38);
  then every branch, tag and commit was rewritten with `git filter-repo` to
  delete the value from all blobs, force-pushed, and branch protection restored.
  The new `Personal data scan` check is **required**, so it cannot recur.
- **Residual risk**: GitHub may serve unreferenced objects and cached views for
  a period after a rewrite, and any pre-existing fork or clone keeps the old
  objects until it re-fetches. The maintainer was informed; a GitHub Support
  purge request remains available if the cached copy must be removed too.
- **The value is deliberately not reproduced here** (guide II.29.1): a document
  may describe the defect but must never quote the value.
