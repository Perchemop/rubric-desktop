# Brand — Rubric

**Original work.** Everything described here was authored for Rubric; no
competitor mark, trade dress, or colour scheme is referenced or imitated.

## Name

**Rubric** — the marking guide a reviewer uses when assessing work. The name
was chosen for the domain (reviewing and marking up documents) and checked
against common PDF-tool naming so it does not collide with an existing
product's identity.

## Mark

`assets/logo.svg` — an original geometric composition:

- **Shape language:** a rounded square "sheet" (the page under review) with a
  single bold diagonal stroke crossing it — the reviewer's marking action.
  The stroke terminates in a small tick, reading as both a check-mark and a
  drawn markup segment.
- **Why it reads:** page + mark = the product's whole job in two shapes. No
  text in the mark, so it survives small sizes (16 px tray icon to 1024 px
  store listing).
- **Construction:** flat vector, single-weight strokes, generous negative
  space; no gradients, bevels, or drop shadows.

## Colour palette

Defined in code as the single source of truth (`src/domain/style.rs`,
`src/domain/theme.rs`); this document records the intent.

| Token | Value | Use |
|---|---|---|
| Rubric orange | `#FF7A45` | Primary accent — the marking stroke, active tool, selection ring |
| Ink | near-black neutrals | Text, rulers, chrome lines |
| Paper | `#FFFFFF` | Document page (always light, in both themes) |
| Paper chrome (light) | warm light neutral | Light-theme window background |
| Indigo dark surfaces | deep indigo | Dark-theme window chrome; the page itself stays light |

The rule that matters most for trust: **the document page is always
paper-white** in both themes, so what you measure and export matches what you
see. Only the surrounding chrome changes with the theme.

## Review-status colours

Fixed, colour-blind-considered assignments on `ReviewStatus` (see
`src/domain/markup.rs`): Open, In review, Approved, Rejected — each with a
distinct hue and a text label everywhere a colour appears (Inspector legend,
canvas dots), so status never relies on colour alone.

## Usage rules

1. Never recolour or distort the mark; use the provided sizes.
2. Keep clear space around the mark of at least the stroke width.
3. Do not place the mark on photographs or busy backgrounds; use the mono
   variant.
4. Platform icons are generated from the brand assets (`assets/icons/`);
   packaging uses those plus the SVG — see `docs/design/` follow-ups in
   `ROADMAP.md` (raster set audit, Q-PACK-2).
