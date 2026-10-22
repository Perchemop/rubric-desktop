# Inspiration — Capability-Level

## What this file is

Rubric's implementation is original. Inspiration for Rubric is limited to
*publicly observable, capability-level features* of existing tools — the
kind of thing anyone learns from using a product or from general domain
knowledge. Every implementation — data models, interaction patterns, visual
design — is derived from public standards (e.g. PDF page geometry
conventions), publicly documented algorithms (e.g. shoelace polygon area),
and Rubric's own spec (`docs/spec/README.md`).

## Products referenced as feature inspiration

Mentioning a product here means only: *"this product demonstrates that
capability X is useful."* The capability, not the implementation, is
carried into Rubric.

| Product | Capability observed as useful | How Rubric implements it (originally) |
|---|---|---|
| Bluebeam Revu | Status-driven review workflows; stamp libraries; calibrated measurement toolsets on drawings | Own `ReviewStatus` enum + legend, own `StampPreset` five-glyph set drawn in egui, own `Scale`/`units` domain model with paper-space calibration |
| Adobe Acrobat | Standard annotation taxonomy (highlight/strikeout/underline, sticky notes, callouts) | Own `MarkupKind`/`MarkupGeometry` types, own hit-testing with tolerance, own overlay painter |
| General commercial PDF suites | Non-destructive editing (markups never touch the source PDF); sidecar/annotation storage | `.rubric.json` sidecar with UUIDs and UTC timestamps; the PDF is opened read-only |

## Recording future inspiration

Record *what capability* inspired us, never *how* another product works
internally. Anything beyond capability names belongs outside this
repository.
