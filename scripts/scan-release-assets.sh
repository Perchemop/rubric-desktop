#!/usr/bin/env bash
# Release-asset personal-data scan (guide II.29.3 / II.29.6). Extracts every
# archive under the given directory and fails on a non-CI absolute home path
# before the assets are attached to a release. The CI build users
# /home/runner, /Users/runner and C:\Users\runner are the only permitted home
# paths.
#
# Usage: bash scripts/scan-release-assets.sh <dist-dir>
set -euo pipefail

root="${1:?usage: scan-release-assets.sh <dist-dir>}"
[ -d "$root" ] || { echo "ERROR: $root is not a directory" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

while IFS= read -r -d '' f; do
    dest="$(mktemp -d "$tmp/asset.XXXXXX")"
    case "$f" in
        *.zip) unzip -q -o "$f" -d "$dest" ;;
        *.tar.gz | *.tgz) tar -xzf "$f" -C "$dest" ;;
    esac
done < <(find "$root" -type f \( -name '*.zip' -o -name '*.tar.gz' -o -name '*.tgz' \) -print0)

# Scan the extracted trees and the raw assets. `-o` keeps each match to just the
# path (so the runner allowance anchors correctly and a binary hit does not dump
# a whole file); `-a` is deliberate because a compiled binary can embed the
# build machine's home path, which `-I` would skip.
HOME_RE='/home/[A-Za-z0-9_.-]+|/Users/[A-Za-z0-9_.-]+|C:\\Users\\[A-Za-z0-9_.-]+'
HOME_ALLOW='(/home/runner|/Users/runner|C:\\Users\\runner)$'
hits="$(
    grep -rHnaoE "$HOME_RE" "$tmp" "$root" 2>/dev/null | grep -vE "$HOME_ALLOW" || true
)"

if [ -n "${hits//[[:space:]]/}" ]; then
    echo "ERROR: release assets contain a non-CI absolute home path:" >&2
    echo "$hits" >&2
    exit 1
fi

echo "release-asset scan: clean (only CI runner home paths permitted)"
