#!/usr/bin/env bash
# Secret scan for tracked files (guide Part I §13, Part II §20.1/§20.7):
# Fails the build when a recognised credential pattern appears in git's
# tracked content. Deliberately dependency-free (no external scanner) so the
# security check itself adds no supply-chain surface. Gitleaks remains a
# recommended deeper alternative for local pre-commit use.
set -euo pipefail

cd "$(dirname "$0")/.."

# The script references the patterns it scans for; exclude itself (and a
# checked-in expected-matches fixture, if one is ever added) from matching.
matches="$({
    git ls-files -z -- ':!*scan-secrets.sh'
} | while IFS= read -r -d '' f; do
    [ -f "$f" ] || continue
    # Binary files cannot hold meaningful token patterns: skip (-I).
    grep -InE \
        -e 'github_pat_[A-Za-z0-9_]{20,}' \
        -e 'gh[pousr]_[A-Za-z0-9]{36}' \
        -e 'AKIA[0-9A-Z]{16}' \
        -e 'AIza[0-9A-Za-z_-]{35}' \
        -e 'xox[baprs]-[A-Za-z0-9-]{10,}' \
        -e 'sk-(live|test)-[A-Za-z0-9]{20,}' \
        -e '-----BEGIN [A-Z ]*PRIVATE KEY-----' \
        "$f" 2>/dev/null || true
done)"

# Explicitly reject committing whole dotenv files, regardless of content.
tracked_env="$(git ls-files -z | tr '\0' '\n' | grep -E '(^|/)\.env(\.|$)' || true)"

if [ -n "$matches" ]; then
    echo "ERROR: recognised credential pattern(s) present in tracked files:" >&2
    echo "$matches" >&2
    exit 1
fi
if [ -n "$tracked_env" ]; then
    echo "ERROR: dotenv file(s) tracked in git (secrets belong in CI-only storage):" >&2
    echo "$tracked_env" >&2
    exit 1
fi
echo "secrets scan: clean (no credential patterns, no .env files tracked)"
