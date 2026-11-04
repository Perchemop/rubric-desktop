#!/usr/bin/env python3
"""Record one release's token metrics in the CI-managed ledger (guide §12.1).

Usage: update-token-ledger.py <version> <tokens>

The ledger at ``docs/tokens.json`` carries:

    {
      "total_tokens_since_start": <int>,
      "releases": [
        {"release_version": "<v>", "release_tokens": <int>},
        ...
      ]
    }

Running this script for a version that is already present updates that entry
instead of duplicating it, and recomputes ``total_tokens_since_start`` from the
entries — so it is safe to re-run (idempotent), which matters when a release
workflow is retried.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

LEDGER = Path("docs/tokens.json")


def load(path: Path = LEDGER) -> dict:
    if not path.exists():
        return {"total_tokens_since_start": 0, "releases": []}
    data = json.loads(path.read_text(encoding="utf-8"))
    data.setdefault("total_tokens_since_start", 0)
    data.setdefault("releases", [])
    return data


def record(data: dict, version: str, tokens: int) -> dict:
    releases = [r for r in data.get("releases", []) if r.get("release_version") != version]
    releases.append({"release_version": version, "release_tokens": tokens})
    releases.sort(key=lambda r: r.get("release_version", ""))
    data["releases"] = releases
    data["total_tokens_since_start"] = sum(
        int(r.get("release_tokens", 0)) for r in releases
    )
    return data


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    version = argv[1].lstrip("v")
    try:
        tokens = int(argv[2])
    except ValueError:
        print(f"error: tokens must be an integer, got {argv[2]!r}", file=sys.stderr)
        return 2
    if tokens < 0:
        print("error: tokens must not be negative", file=sys.stderr)
        return 2

    data = record(load(), version, tokens)
    LEDGER.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    print(
        f"ledger: {version} -> {tokens} tokens; "
        f"cumulative {data['total_tokens_since_start']}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
