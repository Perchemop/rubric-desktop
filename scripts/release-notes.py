#!/usr/bin/env python3
"""Release notes for one version, extracted from CHANGELOG.md (guide §6.3).

Usage: release-notes.py <version>        e.g. release-notes.py 0.1.0

Prints the body of the CHANGELOG section whose heading matches the version.
When the tag has no dedicated entry, the `[Unreleased]` section is used with an
explicit disclosure line ("Notes taken from the [Unreleased] section…") so the
release body can never silently misattribute notes. The script exits 1 only
when neither section exists (or is empty), which lets the workflow fall back to
its own honest one-liner.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

CHANGELOG = "CHANGELOG.md"

HEADING = re.compile(r"^##\s+\[([^\]]+)\]")


def sections(text: str) -> dict[str, list[str]]:
    """Map each `## [name]` heading to its body lines."""
    out: dict[str, list[str]] = {}
    current: str | None = None
    for line in text.splitlines():
        match = HEADING.match(line)
        if match:
            current = match.group(1).strip()
            out[current] = []
        elif current is not None:
            out[current].append(line)
    return out


def pick(sections_map: dict[str, list[str]], version: str) -> tuple[str, list[str]] | None:
    """Prefer an exact version entry; fall back to Unreleased."""
    for key, body in sections_map.items():
        if key.lower() == version.lower():
            return key, body
    for key, body in sections_map.items():
        if key.lower() == "unreleased":
            return key, body
    return None


def tidy(body: list[str]) -> list[str]:
    """Trim leading/trailing blank lines."""
    start, end = 0, len(body)
    while start < end and not body[start].strip():
        start += 1
    while end > start and not body[end - 1].strip():
        end -= 1
    return body[start:end]


def main(version: str) -> int:
    path = Path(CHANGELOG)
    if not path.exists():
        print(f"error: {CHANGELOG} not found", file=sys.stderr)
        return 1

    found = pick(sections(path.read_text(encoding="utf-8")), version)
    if found is None:
        print(f"error: no CHANGELOG section for {version}", file=sys.stderr)
        return 1

    heading, body = found
    lines = tidy(body)
    if not lines:
        print(f"error: CHANGELOG section [{heading}] is empty", file=sys.stderr)
        return 1

    # Always tell the reader which section the notes came from: an Unreleased
    # body attached to a tag must not pretend it was written for that tag.
    prefix = "" if heading.lower() == version.lower() else f"> Notes taken from the [{heading}] section of CHANGELOG.md.\n\n"
    sys.stdout.write(prefix + "\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    sys.exit(main(sys.argv[1]))
