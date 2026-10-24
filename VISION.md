# Vision

**Rubric** is a free, cross-platform PDF markup and measurement tool: a
reviewer opens a document, marks it up (annotations, measurements, notes),
and exports a clean record of what was reviewed and why.

## What Rubric will be

- **Complete, not endless.** Every capability in `docs/scope.md` is either
  implemented and verified or tracked openly in `ROADMAP.md` — no hidden
  gaps between "demo works" and "product works".
- **Local-first.** Documents, markings, and preferences live on the user's
  machine. Optional networking exists only for two user-triggered features
  (update check, optional crash/bug report) and sends only what the user
  has seen and approved.
- **Free and open.** MIT OR Apache-2.0 code, no telemetry, no account, no
  subscription. Heavy building happens in public CI; anyone can rebuild
  every artefact.
- **Honest.** Roadmap percentages move only from verified evidence; release
  notes say what does not work yet; limitations are documented next to the
  features they affect.

## What Rubric will not be

- It will not become a commercial document-management suite — no cloud
  storage backend, no account system, no tracking.
- It will not copy any commercial product's look or name (see
  `docs/inspiration.md` for the capability-level inspirations that remain
  allowed, and the clean-room rules in our contributing guide).
- It will not bundle privileged credentials or embed a bot token.
- It will not grow a component the spec has not asked for (scope creep is
  answered with `docs/scope.md`, not with taste arguments).

## Vision statement (one paragraph)

Millions of review workflows still happen by printing a PDF and drawing on
it with a pen, or by annotating in heavyweight commercial tools that cost
more than the work being reviewed. Rubric makes that review loop free,
keyboard-friendly, and measurable on every platform — an open tool whose
output is a precise, portable record of the review.

## Where the plan lives

The staged roadmap with readiness percentages lives in `ROADMAP.md` (which
this document points at as the roadmap-of-record; the two are updated in
the same commits). The research-to-plan pipeline and its human gate are
described in `docs/research/README.md`.
