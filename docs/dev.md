# Development and branch-protection settings

Recorded 2026-11-18 (guide §9.3, II.26.7).

## Branch protection on `main`

The default branch is protected. Changes land only through a reviewed pull
request merge; direct pushes to `main` are disabled. Settings in force:

- Require a pull request before merging.
- Require status checks to pass. The authoritative checks are the exact names
  in `.ci-required.txt`:
  - `check`
  - `snapshot`
  - `Secret scan`
  - `Personal data scan`
  - `Dependency review`
  - `Cargo deny (licences, advisories)`
  - `SBOM (SPDX + CycloneDX)`
  - `packaging-plan`
- Require linear history.
- Do not allow force-pushes.
- Do not allow deletions.
- `enforce_admins: true` — no admin bypass.
- Require conversation resolution before merging.

`CodeQL (SAST)` is deliberately not an authoritative PR check: it is skipped on
pull requests (it runs on pushes to `main` and on a schedule), so it is reported
but does not decide the PR verdict. Other checks not listed in
`.ci-required.txt` are likewise reported without blocking.

These settings are recorded from the repository's own rules (`.ci-required.txt`,
`AGENTS.md`, guide §9.3); the live GitHub configuration is the owner's
responsibility to confirm.

## Dev identity

Every git action — commit, PR, merge, tag — is authored as
`Rubric Dev <dev@example.com>` (repository-local git config, asserted by
`ci.sh`). The GitHub login of the Dev account is `Perchemop`, owner of the
project remote `https://github.com/Perchemop/rubric-desktop.git`. GitHub
attributes the push/merge event visually to the account holding the credential;
that is a platform limitation recorded in guide II.26.7, not a reason to use a
personal identity in commit metadata.

## Enforcing tool

`~/.config/opencode/scripts/ci.sh` is the enforcing tool and the only supported
path to `main`: it asserts the Dev identity (name, email, and `gh api user`
login), commits the working tree to a branch, pushes it, opens or updates the
pull request, waits for the required checks, and reports the CI verdict. An
identity mismatch or a red authoritative check blocks the merge. No agent
pushes or merges by hand.
