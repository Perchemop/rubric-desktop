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
