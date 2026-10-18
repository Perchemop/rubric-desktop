#!/usr/bin/env bash
# Generate PNG icons from SVG. Requires rsvg-convert.

set -e

cd "$(dirname "$0")/../assets" 2>/dev/null || cd ../assets

for s in 16 32 48 64 128 256 512 1024; do
  rsvg-convert -w $s -h $s logo.svg -o "icons/icon_${s}.png"
done
ls -la icons/
echo "done"
