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
| Orchestrator / lead (plans & delegates, never edits) | `opencode/big-pickle` |
| Domain dev (pure Rust types/logic/tests) | `opencode/deepseek-v4.1-flash-free#high` |
| UI dev (egui/eframe slices) | `opencode/beam-free#high` |
| Fixer (build-gate failures) | `opencode/glm-5.3-flash-free#high` |
| Reviewer (read-only review) | `opencode/glm-5.3-flash-free#high` |
| Test writer | `opencode/mimo-v2.6-flash-free` |
| Docs writer | `opencode/big-pickle` |
| Explorer (fast search) | `opencode/mimo-v2.6-flash-free` |

Model IDs are not load-bearing: they change over time, so the orchestrator
re-discovers the currently available free models at session start and
reassigns the best fit per role (guide §5.2). `.opencode/agents/*.md`
`model:` values are defaults the orchestrator overrides.

## NVIDIA free (fallback, not currently active)

Discovered via `opencode.models(provider = "nvidia")`. Representative free
models available in the fallback tier (selection re-runs when the switch
happens):

- `nvidia/nvidia/nemotron-3.5-super-vl-120b-a12b` — strong reasoning/multi-file
- `nvidia/deepseek-ai/deepseek-v4.1-flash` — coding
- `nvidia/z-ai/glm-5.3-flash` — coding / fixer
- `nvidia/openai/gpt-oss-20b` — general / review
- `nvidia/meta/llama-guard-4-12b` — light/quick tasks

When OpenCode free is exhausted, the orchestrator notes it in `PROGRESS.md`,
switches to NVIDIA free, re-discovers the available NVIDIA models, and
reassigns each role to the best fit — then continues the same slice pipeline.

## Verified availability (re-checked 2026-10-22)

Live probes of the NVIDIA free tier, which is tier 2 in the provider order:

| Model | Result |
| --- | --- |
| `nvidia/z-ai/glm-5.3-flash` | reachable |
| `nvidia/stepfun-ai/step-5-preview` | reachable |
| `nvidia/openai/gpt-oss-20b` | reachable, intermittently |
| `nvidia/z-ai/glm-5.3` | **not reachable** — `socket connection was closed`; removed from the routing chains |

`nvidia/meta/llama-guard-4-12b` and `nvidia/google/google-paligemma` are guard
and vision models, not coding models, so they are not used for roles.

The router in `~/.config/opencode/failover.json` (`fallbacks` +
`fallbacksByAgent`) carries this order. It never re-chooses a model that has
already failed in the same session, and a model that stops connecting is
dropped after two attempts, so a stale ID degrades into the next tier instead
of stalling the session.
