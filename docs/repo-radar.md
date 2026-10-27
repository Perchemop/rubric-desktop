# Repository radar

Periodic build-vs-adopt pass over public GitHub projects (guide.md §4A). One
section per pass; verdicts are `adopt`, `pilot`, `watch` or `reject`, and a
rejected candidate is not re-proposed unless its reason has changed.

## 2026-10-25 — first pass

Scope: PDF import/rendering, measurement geometry, crash reporting, and release,
testing and supply-chain infrastructure.

Already covered in this repository (no action, do not re-propose): `cargo-deny`,
the in-repo secret scan (deliberately not a third-party action), SPDX +
CycloneDX SBOM, dependency review, Dependabot, SHA-pinned actions.

| Candidate | Verdict | Reason |
| --- | --- | --- |
| `axodotdev/cargo-dist` | **adopt** | Declarative targets/installers instead of the hand-maintained release matrix. Adopted incrementally: `dist-workspace.toml` + `dist plan` on every pull request; the release workflow stays hand-written and SHA-pinned (`allow-dirty = ["ci"]`). |
| `sourcefrog/cargo-mutants` | **adopt** | No mutation testing today; wrong geometry/unit arithmetic silently ships bad measurements into a review. Weekly, scoped to `geom.rs`, `units.rs` and `markup.rs`. |
| `sentry-rust` | **reject** | `crates/crash-report` already files a GitHub issue from a crash without a vendor account or a network dependency at start-up; adding a DSN would be a second, overlapping path. Revisit only if hosted aggregation becomes a requirement. |
| `LaurenzV/hayro` | **watch** | A pure-Rust PDF renderer would remove the PDFium runtime fetch from CI and the packaging problem that goes with it. It is not a drop-in for the annotation/coordinate work already built on `pdfium-render`; re-check when a renderer change is scoped, or if PDFium packaging becomes a blocker. |

Re-check date for the radar as a whole: 2027-01-25.

## 2026-10-26 — inspiration addendum (Stirling PDF)

Scope: client added Stirling PDF as a capability-level inspiration product.
Radar relevance checked even though this is inspiration-only, not a dependency:

| Candidate | Verdict | Reason |
| --- | --- | --- |
| `Stirling-Tools/Stirling-PDF` (as dependency) | **reject** | Java/Spring server platform — wrong stack (§4A.3 Fit) and a server runtime Rubric does not want. As *inspiration only* it is adopted (client-directed): capability analysis in `docs/research/stirling-pdf-capabilities.md`; core is MIT, proprietary directories not consulted. |
| `Stirling-Tools/Stirling-PDF` (compare/redaction/signature capabilities) | **watch** | Compare validates the v0.2-01 `DiffResult` model and may seed a UI slice; redaction/signatures/page-ops need PDF *writing*, which contradicts the read-only PDF decision until that scope decision is revisited. |

## 2026-10-26 — v0.2-02 Markups List UI

**No radar change.** The Markups List/review console capability is already
covered by the Bluebeam/Acrobat inspiration rows and this repository's own
`MarkupList` domain model. This slice introduced no external product,
dependency or copied material.

## 2026-10-26 — v0.2-03 editing commands

**No radar change.** Copy Properties and page-scoped z-order are spec features
(F-114/F-115) implemented from the repository's own markup/document model. No
external product, dependency or copied material was introduced.

## 2026-10-26 — v0.2-04 persistence hardening

**No radar change.** Autosave scheduling and the `schema_version` guard are
internal persistence work (F-502/F-507) built from the repository's own store
and data model. No external product, dependency or copied material was
introduced.
