# Token ledger

This file is the source of truth for token-usage metrics (guide §24.1) and
supersedes `docs/tokens.json` as the ledger. One row per release:

| Column | Meaning |
|---|---|
| Version | Release tag |
| Date | Release date |
| Tokens this release | Metered tokens consumed between this release tag and the previous one |
| Tokens cumulative | Running total from the start of development |
| Notes | Honest context for the numbers |

## Source and measured-only rule

Source: the model provider's usage export, aggregated by release-tag boundary.
**Measured only:** every number below comes from this project's own runs. Vendor
marketing figures and estimated savings never appear here or in a release note
(guide §19 rule 3).

## Releases

| Version | Date | Tokens this release | Tokens cumulative | Notes |
|---|---|---:|---:|---|
| v0.3.0 | 2026-11-06 | 0 | 0 | Migrated from `docs/tokens.json`; metering not yet wired — honest zero |
| v0.4.10 | 2026-11-16 | 0 | 0 | Migrated from `docs/tokens.json`; metering not yet wired — honest zero |

Ledger created 2026-11-18 from the existing `docs/tokens.json`
(`total_tokens_since_start: 0`, and `release_tokens: 0` for both recorded
releases). Release dates come from `CHANGELOG.md`. No numbers were invented.

## Relationship to `docs/tokens.json`

`docs/tokens.json` is retained in this slice: the in-app release-notes popup and
the release workflow still read it. It remains the build-time bundle until the
guide §24 build script that generates JSON from this table exists. Do not delete
it before that script lands.
