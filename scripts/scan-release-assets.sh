#!/usr/bin/env bash
# Release-asset personal-data scan (guide II.29.3 / II.29.6). Extracts every
# archive under the given directory and fails on a non-CI absolute home path
# before the assets are attached to a release. /home/runner (the CI build user)
# is the only permitted home path.
#
# Usage: bash scripts/scan-release-assets.sh <dist-dir>
set -euo pipefail

root="${1:?usage: scan-release-assets.sh <dist-dir>}"
[ -d "$root" ] || { echo "ERROR: $root is not a directory" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

while IFS= read -r -d '' f; do
    dest="$tmp/$(basename "$f")"
    mkdir -p "$dest"
    case "$f" in
        *.zip) unzip -q -o "$f" -d "$dest" ;;
        *.tar.gz | *.tgz) tar -xzf "$f" -C "$dest" ;;
    esac
done < <(find "$root" -type f \( -name '*.zip' -o -name '*.tar.gz' -o -name '*.tgz' \) -print0)

# Scan the extracted trees and the raw assets. `-a` is deliberate: a compiled
# binary can embed the build machine's home path, which `-I` would skip.
hits="$(
    grep -rnaE '/home/[A-Za-z0-9_.-]+|C:\\Users\\[A-Za-z0-9_.-]+' "$tmp" "$root" 2>/dev/null \
        | grep -vE '/home/runner$' || true
)"

if [ -n "${hits//[[:space:]]/}" ]; then
    echo "ERROR: release assets contain a non-CI absolute home path:" >&2
    echo "$hits" >&2
    exit 1
fi

echo "release-asset scan: clean (only /home/runner permitted)"
