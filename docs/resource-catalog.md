# External resource catalog adoptions

Recorded under guide §25 use rule 2. The guide §25 catalogs are discovery
surfaces, not implementation sources; every adoption is recorded here with its
catalog, source, licence, pinned version/date and the decision that approved it
before use.

| Catalog | Source | Licence | Version-or-date | Decision (DECISIONS.md ref) |
|---|---|---|---|---|
| GitHub search / Awesome-Construction-Takeoff (`ishandutta2007/Awesome-Construction-Takeoff`) | [`Kentucky-ai/opentakeoff`](https://github.com/Kentucky-ai/opentakeoff) | AGPL-3.0-only (repo `LICENSE` is authoritative; a community catalog entry still says Apache-2.0 from before the relicence) | 2026-11-18 | Discovery only — NOT adopted (AGPL-3.0 incompatible with MIT OR Apache-2.0 dependency policy; capability mapped for native implementation in `docs/research/takeoff-quantities.md`; MCP-client path pending owner decision). **Note**: Domain types (`TakeoffCondition`, `ConditionKind`, `TakeoffSettings`, `TakeoffError`) and computation logic (`quantity_rows()`) implemented native in `apps/rubric/src/domain/takeoff.rs` per DECISIONS.md 2026-11-20 (takeoff-quantities domain layer v0.5) |

No code, asset or dependency has been adopted from any §25 catalog. The row
above records a discovery-only mapping (research gate applies, guide §2.3); the
native Rubric implementation it notes is original code written for Rubric —
capability-level inspiration only — tracked on PR #230 (open at the time of
writing).

## Research documentation

| Research doc | Source | Purpose | Status |
|--------------|--------|---------|--------|
| `docs/research/takeoff-quantities.md` | OpenTakeoff, Awesome-Construction-Takeoff | Capability-level summary for takeoff quantities feature | Native path owner-approved 2026-11-18 (`specs/takeoff-quantities.spec.md`, PR #225); implementation open as PR #230; MCP-client path still owner-gated |
| `docs/research/bluebeam-vs-rubric-audit.md` | Bluebeam Revu | UI/UX parity analysis | Approved for implementation guidance |
| `docs/research/ui-layout-workflows-analysis.md` | OpenTakeoff, Bluebeam | UI pattern recommendations for v0.6 | Approved for implementation guidance |

## Tooling already in force (not adopted from the §25 catalogs)

These two components predate the catalogs and are recorded here for
completeness. Licences and versions come from `ATTRIBUTION.md` (the PDFium
binary's pinned build is confirmed in the CI/packaging workflows).

| Component | Source | Licence | Version-or-date | Decision (DECISIONS.md ref) |
|---|---|---|---|---|
| PDFium (bundled runtime library) | `bblanchon/pdfium-binaries` prebuilt binaries; fetch pinned to `chromium/7881` in CI and packaging | BSD-3-Clause (Chromium project) | Chromium build 7881 (`pdfium_7881` bindings) | `DECISIONS.md` 2026-10-20 (runtime-binding choice); `pdfium-render` binding pin 0.9.x |
| hayro (pure-Rust PDF renderer, used for aarch64 and other platforms where PDFium binaries are hard to distribute) | `hayro` / `hayro-syntax` crates (`LaurenzV/hayro`) | MIT OR Apache-2.0 | 0.9 (`ATTRIBUTION.md`) | `DECISIONS.md` 2026-10-31 (adopt a pure-Rust PDF engine) |
