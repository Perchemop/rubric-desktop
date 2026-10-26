# Process Feedback

Append-only log of agent-reported process friction (guide-addendum §N).
Feedback is advisory until the client promotes it into the local
guide/addendum. One entry per session unless work is fully blocked.

---

```text
PROCESS FEEDBACK
Date / session: 2026-10-25 (session "Rubric v0.2-01 markup-list slice CI fixes")
What I was trying to do: push the v0.2-01 domain slice fixes for CI
  verification on the existing feature branch / open PR #30.
Block or friction: ci.sh defaults to CI_BRANCH=opencode/ci and opens/updates a
  PR from that branch, so running it from a feature branch that already had an
  open PR (#30) created a second, duplicate PR (#32) with a partial commit
  state. The two PRs then diverged and both consumed Actions minutes.
Why the current guide/addendum caused it (or did not cover it): guide §4 /
  AGENTS.md describe ci.sh as "commits the working tree to a branch" without
  saying the branch defaults to a fixed opencode/ci branch regardless of the
  checked-out branch, nor that it should be pointed at the feature branch when
  one exists.
Suggested change (one concrete edit): document in guide §4 (and echo in ci.sh
  usage output) that when the repo already has an open PR for the current
  branch, run `CI_BRANCH=<current-branch> ci.sh run …`; or make ci.sh detect
  an existing PR for the checked-out branch and use it by default.
Severity: slows-work
Workaround I used (if any): CI_BRANCH=opencode/v0.2-01-markup-list-domain
  env override; duplicate PR #32 to be closed after #30 goes green.
Confirm: did NOT violate clean-room / v0.1.0 gate / no-secrets /
  CI-before-client-try / no meta-guide push
```
