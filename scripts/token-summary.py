#!/usr/bin/env python3
"""Markdown token-usage block for release notes (guide §12.1).

Usage: token-summary.py <version> <tokens-this-release>

Prints a "Token usage" section for the GitHub Release body: the tokens used by
this release and the cumulative total since project start, both taken from
``docs/tokens.json`` plus the supplied per-release figure. Release notes must
reference the ledger values, which is what this produces.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

LEDGER = Path("docs/tokens.json")


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    version = argv[1].lstrip("v")
    try:
        this_release = int(argv[2])
    except ValueError:
        print(f"error: tokens must be an integer, got {argv[2]!r}", file=sys.stderr)
        return 2

    data = json.loads(LEDGER.read_text(encoding="utf-8")) if LEDGER.exists() else {}
    prior = int(data.get("total_tokens_since_start", 0))
    cumulative = prior + this_release

    print("### Token usage")
    print()
    print(f"- This release ({version}): **{this_release}** tokens")
    print(f"- Cumulative since project start: **{cumulative}** tokens")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
