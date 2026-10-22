# Project Brief — Rubric

Discovery answers per guide §0. Source of truth for scope questions;
`docs/scope.md` holds the in/out list, `docs/assumptions-and-questions.md`
holds everything unconfirmed.

**Facts / Assumptions / Decisions are marked** `[Fact]`, `[Assumption]`,
`[Decision]` throughout (guide §6).

## A. Product identity

1. **Purpose (one sentence)**: A free, cross-platform desktop application for
   marking up, reviewing, and measuring PDF documents entirely locally — no
   subscriptions, no cloud, no accounts. `[Decision]`
2. **Primary user**: Engineers, architects, contractors, designers, and
   reviewers who redline drawings and documents and need calibrated
   measurements plus a simple review workflow. `[Assumption — not explicitly
   confirmed; inferred from the feature set]`
3. **Problem solved**: Professional markup/review/measurement workflows are
   generally tied to commercial, subscription, or single-platform tools;
   Rubric provides an original, offline, free alternative on Linux, macOS,
   and Windows. `[Decision]`

## B. Inspiration & scope

4. **Feature inspiration**: Capability-level inspiration is recorded in
   `docs/inspiration.md` — publicly observable features only.
   `[Decision]`
5. **Most important capabilities**: non-destructive markup (sidecar JSON),
   calibrated measurement toolset, review statuses/comments/stamps, native
   file dialogs, PDF import/rendering. `[Decision — mirrored in
   docs/spec/README.md]`
6. **Out of scope for v1**: cloud/collaboration, editing PDF content itself,
   proprietary/encrypted formats, mobile/web deployment, digital signatures,
   embedding markups into the PDF. Full list in `docs/scope.md`.
   `[Decision — spec non-goals]`

## C. Technical preferences

7. **Language/framework**: Pure Rust + egui/eframe (0.36). `[Decision]`
8. **Target platforms**: All practical desktop platforms — Windows, macOS,
   Linux (WASM/web evaluated later; mobile only if requested). `[Decision]`
9. **Offline**: Yes — fully local-first; the app must work with no network.
   `[Fact]`
10. **Dependencies/licences**: Free OSS only; MIT/Apache-2.0 strongly
    preferred, AGPL/GPL prohibited without approval (AGENTS.md). Runtime
    PDFium library is BSD-3-Clause and loaded on demand, never linked at
    compile time. `[Decision]` Full list: `ATTRIBUTION.md`.

## D. Quality & process

11. **Polish**: A professional, uncluttered, original visual design is
    desired; original branding lives in `assets/branding.md`.
    `[Decision]`
12. **Constrained machine**: The local host is memory-constrained; **all**
    heavy work (test, clippy, check, release build) runs in GitHub Actions.
    `[Fact — user directive 2026-10-19, AGENTS.md "CI-first"]`
13. **Design system**: Original palette/typography defined in
    `assets/branding.md`; accessibility review is tracked as F-605 (later).
    `[Decision]`

## E. Name

14. **Name**: **Rubric** — invented for this project and locked; see
    `docs/product-name.md`. `[Decision]`
