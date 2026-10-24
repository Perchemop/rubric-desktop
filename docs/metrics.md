# Metrics — what Rubric tracks and why

opensource.guide (metrics) asks projects to be deliberate about measurement:
record which metrics are tracked, and why — not to chase popularity, but to
answer the questions that matter for this project. Rubric's goals are
transparency and usefulness, not growth, so the measuring set is small and
cheap; nothing here uses third-party analytics.

## Tracked

| Metric | Question it answers | Source |
|---|---|---|
| Release asset download counts | Are the CI-built artefacts actually usable/used? Which platforms matter for packaging effort? | GitHub Releases API (public) |
| Maintenance traffic views/clones | Is activity reaching people at all (transparency, not promotion)? | GitHub traffic API (maintainer-only) |
| Issue and PR response times | Responsiveness commitment: are reports/PRs being triaged near the 7-day claim in `SECURITY.md`/`CONTRIBUTING.md`? | Issue/PR timestamps (public) |
| Contribution mix (code/docs/triage/a11y tests) | Are low-barrier paths working (is the contributor funnel getting past first contact)? | Merged PR labels |

## Explicitly not tracked

- No telemetry or analytics in the app itself. Update-check requests send no
  payload and are not logged server-side beyond GitHub's API access logs.
- No star/watch/lead reporting; popularity is optional and not a goal
  (metrics guide: interpret navigation spikes carefully, use them to spot
  wrong-audience traffic rather than success).
- No user counts: the application is local-first and inserts no
  identifiers, so unique-user counts are unknowable by design — the honest
  metric for "is it used" is the release asset counts plus the issue flow.

## Cadence

Reviewed by the maintainer around each tagged release (see `GOVERNANCE.md`
for the decision model and the data-model of decisions). A metric that has
not informed a decision is dropped from this file rather than tracked
inertly.
