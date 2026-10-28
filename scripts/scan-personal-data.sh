#!/usr/bin/env bash
# Personal-data scan for tracked files (guide II.29, owner directive
# 2026-10-27): fails when a real-looking personal value is committed. It is a
# required status check ("Personal data scan") so a leak cannot merge.
#
# Dependency notice files that legitimately carry an upstream author's address
# are third-party content (guide II.29.2); if such a file is ever added, the
# scan fails loudly and a human classifies the hit rather than the check
# guessing. This script and the release-asset scanner contain the patterns
# themselves, so both are excluded from their own scan.
#
# Dependency-free (grep + git only): the gate itself adds no supply-chain
# surface.
set -euo pipefail

cd "$(dirname "$0")/.."

EXCLUDES=(
    ':!scripts/scan-personal-data.sh'
    ':!scripts/scan-release-assets.sh'
)

tracked() {
    git ls-files -z -- "${EXCLUDES[@]}" | while IFS= read -r -d '' f; do
        printf '%s\0' "$f"
    done
}

# 1. Absolute home paths. /home/runner is the CI build user and is permitted
#    (guide II.29.6); any other /home/<name> or C:\Users\<name> is a finding.
#    (Home paths are scanned with -a because a build path can be embedded in a
#    committed binary; the literal "/home/" prefix keeps false positives out.)
home_hits="$(
    tracked | while IFS= read -r -d '' f; do
        [ -f "$f" ] || continue
        grep -HnaoE '/home/[A-Za-z0-9_.-]+|C:\\Users\\[A-Za-z0-9_.-]+' "$f" 2>/dev/null || true
    done | grep -vE '/home/runner$' || true
)"

# 2. Email addresses, allowing only obviously synthetic or GitHub-noreply ones.
email_hits="$(
    tracked | while IFS= read -r -d '' f; do
        [ -f "$f" ] || continue
        grep -HInaoE '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' "$f" 2>/dev/null || true
    done | grep -vEi '@(example\.(com|org|net)|users\.noreply\.github\.com|noreply\.github\.com)$' || true
)"

fail=0
if [ -n "${home_hits//[[:space:]]/}" ]; then
    echo "ERROR: tracked file(s) contain a non-CI absolute home path:" >&2
    echo "$home_hits" >&2
    fail=1
fi
if [ -n "${email_hits//[[:space:]]/}" ]; then
    echo "ERROR: tracked file(s) contain a non-synthetic email address:" >&2
    echo "$email_hits" >&2
    fail=1
fi
if [ "$fail" -ne 0 ]; then
    echo "Classify each hit (guide II.29.2): synthetic/CI, or a finding to fix." >&2
    exit 1
fi

echo "personal-data scan: clean (no non-CI home path, no real email)"
