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

```text
PROCESS FEEDBACK (follow-up)
Date / session: 2026-10-26 (same slice, final PR hygiene)
Resolution of the duplicate-PR friction above: #30 was closed as superseded
  and #32 (opencode/ci) was retitled/re-bodied as the authoritative slice PR,
  because it already carried the reviewer fixes and docs on top of the slice.
  This is a workaround, not a fix for the underlying ci.sh default.
Still recommended: make ci.sh detect an open PR for the checked-out branch and
  push to that branch by default, or print the `CI_BRANCH=<branch>` hint when a
  second PR would be created. Until then, the workaround is to pass
  `CI_BRANCH` explicitly and keep one PR per slice.
Severity: slows-work (open)
Confirm: no clean-room, gate or secrets issue; no force-push; human still merges.
```

```
PROCESS FEEDBACK
Date / session: 2026-11-16 (Rubric v0.4.10 release)
What I was trying to do: tag v0.4.10 and let CI append the release row to
  docs/tokens.json per suite standard §12.1 ("CI updates the ledger on each
  semver tag and commits it").
Block or friction: no post-tag ledger commit appeared on main after the
  v0.4.10 Release/Package workflows succeeded (git log v0.4.10..origin/main
  empty); the release body's token section therefore reported the stale
  ledger.
Why the current guide/addendum caused it (or did not cover it): the ledger
  commit step is not wired (or is silent-failing) in this repo's
  release.yml; §12.1 assumes it exists.
Suggested change (one concrete edit): add a ledger-update job to
  release.yml that appends {release_version, release_tokens} on tag builds
  and commits docs/tokens.json to main via PR, or drop the §12.1 ledger
  claim from the suite standard for repos where it is not wired.
Severity: slows-work (open)
Workaround I used (if any): appended the 0.4.10 row by hand in this PR with
  the ledger's honest values (0 tokens — no metering wired); release body
  already reports the same zeros, so no published claim changed.
Confirm: did NOT violate clean-room / v0.1.0 gate / no-secrets /
  CI-before-client-try / no meta-guide push.
```
