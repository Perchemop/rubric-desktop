# Governance

## Decision model

Rubric currently follows a **benevolent-dictator-for-now (BDFL)** model,
because it is exactly one maintainer. Decisions that carry the project
forward — spec, priority, releases, dependencies — rest with the
maintainer (see `CONTRIBUTORS.md`). This is recorded deliberately rather
than left implicit (opensource.guide, leadership-and-governance):

- The default decision process is **consensus-seeking** — proposals are
  discussed until serious concerns are heard, then resolved; a cold vote is
  avoided. The maintainer is the tiebreaker, and that role is named here
  **before** it is needed.
- If participation grows to the point of being genuinely multi-maintainer,
  the maintainer commits to re-convening publicly (an issue opened for
  governance) to choose one of the three canonical models and rewrite this
  file. Nothing about "one maintainer" is permanent by design.

## Roles

| Role | Meaning here |
|---|---|
| **Maintainer** | Feels responsible for the whole: direction, releases, triage. Currently one person plus the agent build pipeline, which is a tool of the maintainer, not a role in governance. |
| **Contributor** | Anyone whose work was merged — code, docs, tests, issue triage, accessibility feedback. Recorded in `CONTRIBUTORS.md`. |
| **Committer** | Commit access is held by the maintainer and by the agent pipeline's narrow write token (below). |

### How to grow into a governance role (written path)

- Sustained, trusted contributions (merged work in code, docs, triage, or
  a11y/security work) plus a track record of good issue conduct.
- A maintainer path opens when the project's arms grow past the one
  maintainer's honest capacity: the existing maintainer nominates the new
  maintainer publicly (issue + commit to this file), and the nomination
  follows the consensus-seeking rule above. Self-nomination is welcome —
  sustained work is the qualification, not seniority.
- Commit access may be granted earlier than full maintainer status, only
  paired with branch protection so nothing bypasses review (§20.2).

## Agent automation (least privilege, documented)

The Rubric build is agent-assisted (see `AGENTS.md`). Governance-relevant
rules for those agents:

- Agent credentials are scoped: a fine-grained token with push access to
  this repository only — not an owner account, not an org-wide key. Agent
  identities commit as `Rubric Dev <dev@example.com>` (repository-local
  config), keeping human and agent contributions distinguishable.
- Agents do not have permissions beyond commit/push/PR; release, admin, and
  secret-management duties stay with the maintainer.
- Branch protection (required CI check, no force pushes, admin-enforced)
  applies to agent credentials exactly as to the maintainer.
- If the repository ever moves to an Organization, **a second admin must be
  added** (bus-factor rule) before any agent or automation is granted more
  than write access. Today the single maintainer keeps admin; that is
  accepted and monitored, not ignored.

## Conflicts

- Moderator, not combatant: the maintainer de-escalates, re-anchors on
  `VISION.md`, and closes threads that stop converging (with a reason).
- A lone loud voice is not the community; silent users count.
- Repeatedly requested-and-declined topics get written into the docs once,
  instead of re-litigated forever.

## Commercial participation

None currently: no company funds Rubric, no sponsorships, no paid
prioritisation. If that ever changes, this section is updated first and the
change is judged by the same criteria as any other contribution.

## Funding

No funds are collected; there is no legal entity. If donations are ever
enabled (GitHub Sponsors is the lightest path), they go to the maintainer
directly and this file states that plainly. Nothing about Rubric's licence
or roadmap is for sale.
