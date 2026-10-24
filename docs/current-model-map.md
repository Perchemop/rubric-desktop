# Current Model Map

Which free model is assigned to which subagent role for the current session.
Updated whenever the available free models change (guide §5.2).

## Provider order (guide §5)

1. **OpenCode free models** (default).
2. **NVIDIA free models** — used automatically when OpenCode free is
   exhausted (rate limits, quota, or unavailable), via the configured API
   key integration.

No local model hosting (Ollama or similar) is used. Only the provider
changes between the two tiers; the process, roles, vertical-slice
discipline, CI-first rule and tracking files stay identical.

## OpenCode free (active)

| Role | Model |
|---|---|
| Orchestrator / lead (plans & delegates, never edits) | `nvidia/z-ai/glm-5.3` — NVIDIA free tier 2, used because `opencode/big-pickle` was retired from the OpenCode free tier |
| Domain dev (pure Rust types/logic/tests) | `opencode/deepseek-v4.1-flash-free#high` (unchanged) |
| UI dev (egui/eframe slices) | `opencode/nemotron-3.5-super-vl-free#high` — vision-capable per guide §5.3 for canvas-touching slices (previously `beam-free#high`) |
| Fixer (build-gate failures) | `opencode/flint-40b-free#high` — newest OpenCode free code-generation model (previously `glm-5.3-flash-free#high`) |
| Reviewer (read-only review) | `opencode/glm-5.3-flash-free#high` (unchanged) |
| Test writer | `opencode/mimo-v2.6-flash-free` (unchanged) |
| Docs writer | `opencode/beam-free#high` (big-pickle retired) |
| Explorer (fast search) | `opencode/mimo-v2.6-flash-free` (unchanged) |
| CI runner | `opencode/beam-free#high` (proven this session) |
| designer (UI/UX) | `opencode/nemotron-3.5-super-vl-free#high` | vision-capable; design + performance review |

Model IDs are not load-bearing: they change over time, so the orchestrator
re-discovers the currently available free models at session start and
reassigns the best fit per role (guide §5.2). `.opencode/agents/*.md`
`model:` values are defaults the orchestrator overrides.

## NVIDIA free (fallback, not currently active)

Discovered via `opencode.models(provider = "nvidia")`. Representative free
models available in the fallback tier (re-checked 2026-10-23; selection
re-runs when the switch happens):

- `nvidia/nvidia/nemotron-3.5-super-vl-120b-a12b` — strong reasoning, vision
- `nvidia/stepfun-ai/step-5-preview`
- `nvidia/deepseek-ai/deepseek-v4.1-flash` — coding
- `nvidia/z-ai/glm-5.3-flash` — coding
- `nvidia/z-ai/glm-5.3` — general reasoning (currently the orchestrator)
- `nvidia/meta/muse-glimmer-30b`
- `nvidia/poolside/laguna-xs-2.1`
- `nvidia/google/diffusiongemma-26b-a4b-it`
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
NVIDIA free → paid API keys and no mid-session switching.

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

No research notes exist yet — Rubric's implemented feature set was designed
from its own spec (`docs/spec/README.md`), not from vendor research. When notes
are added, the method used is recorded in the note itself, and the human gate
in guide §8.4 applies before dependent implementation.

## Model terms record (Part II §19.7 / §2.4)

Every model and API used during the build is accounted for, so the
redistribution terms stay auditable:

- **Providers:** OpenCode (free tier) and NVIDIA (free tier) only — no paid
  generation, no local model hosting. Terms of use are the sites' own
  standard free-tier terms; nothing forbids redistribution of generated
  *project* material in this repository, which is reviewed as original work
  before commit.
- **Record of record:** the licence/terms summary lives in
  `ATTRIBUTION.md` ("AI tooling used during the build"), next to the
  dependency SBOMs — kept together so attribution review sees both humans'
  and machines' inputs in one place.
- **Rule in force:** model outputs whose terms forbid redistribution are
  never committed; anything committed must satisfy the clean-room review
  bar regardless of who (or what) authored the first draft.
