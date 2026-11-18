# Current Model Map

Which free model is assigned to which subagent role for the current session.
Updated whenever the available free models change (guide §5.2).

## Provider order (guide §5)

1. **OpenCode free models** (default, tier 1).
2. **NVIDIA free models** — used automatically when OpenCode free is
   exhausted (rate limits, quota, or unavailable), via the configured API
   key integration (tier 2).
3. **OpenRouter free models** — owner-added tier 3 (deviation from the guide's
   two-tier policy, owner-directed), reached only after tiers 1 and 2; the
   owner-supplied OpenRouter key is live in `~/.config/opencode/opencode.jsonc`
   and the free quota (50 requests/day, 1000 with credits added) is currently
   exhausted.

No local model hosting (Ollama or similar) is used. Only the provider
changes between the tiers; the process, roles, vertical-slice
discipline, CI-first rule and tracking files stay identical.

## OpenCode free (active — discovered 2026-11-18)

Live catalog at session start (final full CLI listing, `opencode models`):
`deepseek-v4.1-flash-free`, `mimo-v2.6-flash-free`,
`flint-40b-free`, `nemotron-3.5-lightning-free`, `longcat-2.5-free`,
`inkling-2-small-free`, `iceman-free`, `hy4-flash-free`, `beam-free` (all
zero-cost), plus the interactive default `opencode/big-pickle` and the
vision-only `nemotron-3.5-super-vl-free`. The earlier note that
`nemotron-3.5-lightning-free` was no longer listed was stale; it is live again.

**2026-11-18 rotation refresh.** `opencode/glm-5.3-flash-free` was **removed
from the catalog** and from every failover chain (probe + plugin log showed
`provider.no-route`); the GLM second opinion remains reachable as
`nvidia/z-ai/glm-5.3-flash` (tier 2). All 37 chained models were live-probed
through the OpenCode server: 26 ok, 6 rate-limited (alive, 429), 5 nvidia
timeouts (loaded endpoints, not removals). Full table:
`~/.config/opencode/model-rotation.md` (section "Scan + live probe —
2026-11-18"). A weekly systemd user timer (`model-rotation.timer`) now re-runs
scan + probe + rebuild every Monday 06:30.

| Role | Model |
|---|---|
| Orchestrator / lead (plans & delegates, never edits) | runs in-session on the best available free model; `deepseek-v4.1-flash-free#high` is the strongest current coder fallback for the role |
| Domain dev (pure Rust types/logic/tests) | `opencode/deepseek-v4.1-flash-free#high` |
| UI dev (egui/eframe slices) | `opencode/nemotron-3.5-super-vl-free#high` — vision-capable per guide §5.3 for canvas-touching slices |
| Fixer (build-gate failures) | `opencode/flint-40b-free#high` — newest OpenCode free code-generation model |
| Reviewer (read-only review) | `opencode/flint-40b-free#high` (glm-5.3-flash-free removed from catalog 2026-11-18; second opinion preserved — reviewer never starts on the coder's model) |
| Test writer | `opencode/mimo-v2.6-flash-free` |
| Docs writer | `opencode/beam-free#high` |
| CI runner | `opencode/beam-free#high` |
| designer (UI/UX) | `opencode/nemotron-3.5-super-vl-free#high` — vision-capable; design + performance review |

Model IDs are not load-bearing: they change over time, so the orchestrator
re-discovers the currently available free models at session start and
reassigns the best fit per role (guide §5.2). `.opencode/agents/*.md`
`model:` values are defaults the orchestrator overrides.

## NVIDIA free (fallback, not currently active)

Discovered via `opencode.models(provider = "nvidia")`. Representative free
models available in the fallback tier (re-checked 2026-10-27; selection
re-runs when the switch happens):

- `nvidia/nvidia/nemotron-3.5-super-vl-120b-a12b` — strong reasoning, vision
- `nvidia/nvidia/nemotron-3-ultra-550b-a55b` — strong reasoning
- `nvidia/stepfun-ai/step-5-preview`
- `nvidia/deepseek-ai/deepseek-v4.1-flash` — coding
- `nvidia/z-ai/glm-5.3-flash` — coding
- `nvidia/z-ai/glm-5.3` — general reasoning
- `nvidia/minimaxai/minimax-m3.1-flash`
- `nvidia/nvidia/nemotron-3.5-lightning-30b-a3b`
- `nvidia/meta/muse-glimmer-30b`
- `nvidia/poolside/laguna-xs-2.1`
- `nvidia/google/diffusiongemma-26b-a4b-it`
- `nvidia/google/gemma-4-31b-it`
- `nvidia/openai/gpt-oss-20b`

Guard/vision-only models (llama-guard, paligemma) are excluded from roles.

When OpenCode free is exhausted, the orchestrator notes it in `PROGRESS.md`,
switches to NVIDIA free, re-discovers the available NVIDIA models, and
reassigns each role to the best fit — then continues the same slice pipeline.

## Verified availability (re-checked 2026-10-23)

Live probes of the NVIDIA free tier, which is tier 2 in the provider order:

| Model | Result |
| --- | --- |
| `nvidia/z-ai/glm-5.3-flash` | reachable |
| `nvidia/stepfun-ai/step-5-preview` | reachable |
| `nvidia/openai/gpt-oss-20b` | reachable, intermittently |
| `nvidia/z-ai/glm-5.3` | reachable — active orchestrator model this session (earlier "not reachable" note was stale) |

`nvidia/meta/llama-guard-4-12b` and `nvidia/google/google-paligemma` are guard
and vision models, not coding models, so they are not used for roles.

## Availability notes (2026-10-23)

The free catalog churns within a single session: `opencode/big-pickle` was absent
at session-start discovery (the orchestrator therefore runs on NVIDIA free tier 2)
but reappeared in later listings; `opencode/minimax-m3.1-flash-free` was listed,
returned "not available" when used, and vanished from the catalog the same day.
Assignments are re-verified on first use, with failover chain OpenCode free →
NVIDIA free → OpenRouter free and no mid-session switching. No paid API tier is
ever selected.

The router in `~/.config/opencode/failover.json` (`fallbacks` +
`fallbacksByAgent`) carries this order. It never re-chooses a model that has
already failed in the same session, and a model that stops connecting is
dropped after two attempts, so a stale ID degrades into the next tier instead
of stalling the session.

## Research method (guide §2.4)

Public research for `docs/research/*.md` uses the least-human-input ladder:

1. **Preferred:** a browse-capable free model or MCP fetches public pages; the
   Orchestrator assigns it to research Tasks.
2. **CI / script assist:** an Action or script fetches **text** from URLs listed
   in the note; the agent then writes an original behavioural summary (no bulk
   copyright dump in git).
3. **Human assist (last resort):** the human pastes URLs or short excerpts; the
   agent writes behavioural notes only.

Rubric's core feature set was designed from its own spec
(`docs/spec/README.md`), not from vendor research. Capability-level research
notes now exist for the review workflow and PDF tooling
(`docs/research/bluebeam-studio-capabilities.md`,
`docs/research/adobe-acrobat-capabilities.md` and the client-directed
`docs/research/stirling-pdf-capabilities.md`); each records its sources as URLs,
uses original wording, and was approved at the guide §8.4 human gate before any
dependent implementation. The method used is recorded in each note, and the
human gate in guide §8.4 still applies before dependent implementation.

## MCP and tooling status (2026-10-27)

Context7 MCP is available for library/framework documentation. GitHub
operations (PRs, runs, checks, releases) run through the authenticated `gh`
CLI rather than a GitHub MCP server, which is not present in this environment's
tool catalog; the effect the addendum §D.1 asks for (agent-driven PRs, issues,
Actions and Releases) is met by `gh`. Recorded in `docs/guide-compliance.md` as
an adaptation rather than assumed.

## Model terms record (Part II §19.7 / §2.4)

Every model and API used during the build is accounted for, so the
redistribution terms stay auditable:

- **Providers:** OpenCode (free tier) and NVIDIA (free tier) and OpenRouter
  (free tier, owner-added) only — no paid generation, no local model hosting.
  Terms of use are the sites' own standard free-tier terms; nothing forbids
  redistribution of generated *project* material in this repository, which is
  reviewed as original work before commit. OpenRouter is metered (50 free
  requests/day, 1000 with credits added); the key is stored in the OpenCode
  config with mode 600 and nothing key-derived is committed.
- **Record of record:** the licence/terms summary lives in
  `ATTRIBUTION.md` ("AI tooling used during the build"), next to the
  dependency SBOMs — kept together so attribution review sees both humans'
  and machines' inputs in one place.
- **Rule in force:** model outputs whose terms forbid redistribution are
  never committed; anything committed must satisfy the clean-room review
  bar regardless of who (or what) authored the first draft.

## Guide synchronization record (§12.3)

- **2026-11-16:** guide.md (dated 2026-11-16, 1557 lines, 114397 bytes) read in
  full — Part I §0–18, Part II II.18–II.30, §19 (skills + dead-code rule),
  §20 (design anti-patterns), §21 (high-quality UI patterns), §22 (pstack —
  new). Global ruleset (`~/.config/opencode/AGENTS.md`) synchronized: §22
  pstack added verbatim with the four binding usage rules (rigor mode for
  substantial tasks; verification posture complementary to clean-room /
  never-crash / CI-only; HARD BUILD RULE unaffected; stricter rule wins on
  conflict), design-guidelines canonical source re-pointed from the deprecated
  standalone file to guide §20/§21, dead-code rule pointer added to §19.
  No Rubric-repo rule changes required (house-rules block unchanged).

- **2026-11-17:** guide.md (1677 lines, Nov 17 17:33) read in full; global
  ruleset re-synced — §5.2 three-tier model rotation (OpenRouter free tier as
  tier 3, owner-supplied key verified live), 14 house rules (Dev-identity
  authorship, guide II.26.7 in force), §19 token-efficiency skills +
  measured-only rule, §20 scope note, §23 config appendices, §24
  token/contributor schemas (`docs/tokens.md` supersedes `docs/tokens.json`
  as the ledger; measured-only), §25 six external catalogs + use rules,
  II.29 fifth scan surface (crash payload), §7.1 mandatory crash scrub,
  §7.2 signing-key compromise procedure, §9.1–9.4 gates, §17 binding Rust
  summary. OpenRouter scan: 355 models, 13 free + coding-capable; free
  quota shared (50/day) so the tier sits last in every chain.

- **2026-11-18 (2nd sync):** guide.md grown to 1961 lines; re-read in full.
  New: **Authority preamble** (stricter-rule-wins; guide beats skills/plugins/
  commands; no waiving/weakening; 4-step blocker protocol), **§5.6 OpenCode
  Workflow Protocol** (Smart Zone <150k tokens + `/new` between phases; Case
  1–5 selection matrix; seven commands /grill-me·/to-spec·/to-tickets·/tdd·/
  implement·/implement-spec·/code-review; TDD verified via `ci.sh` with
  recorded red-run IDs; batching ≤5 tests per red run), **§5.7 Multi-Agent
  Architecture** (teams only for Case 4; one Orchestrator; 2–5 agents;
  `docs/team-log/` audit trail; allowlist-only pre-approved commands — build
  commands forbidden; plan approval before code; graceful shutdown; docs/dev.md
  records the allowlist), §11 adds docs/specs·tickets·team-log·reference,
  §13 long-session row, §15 fallback prompt now includes the §5.6/§5.7
  workflow, §17 binding Rust summary when the addendum is absent, §18 private
  maintenance-repo + binding §5.6/§5.7 reminders. Global ruleset synced with
  all of the above (Authority preamble + §5.6 verified complete + §5.7 added +
  §4 ci.sh merge/login-assertion/bootstrap-fallback + §11 + §15 + §17 + §18).
