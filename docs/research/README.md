# Research Notes

Public, clean-room research for capabilities not yet implemented.

## The rules (guide §2.1–2.4, §8.4)

1. **Summaries + URLs only.** Every note here is an *original behavioural
   summary* written for Rubric, plus links to the public sources consulted.
   No scraped manual text, no full HTML dumps, no third-party screenshots as
   files, no pixel-perfect UI descriptions.
2. **Human gate.** Research notes require explicit human approval *before*
   any large implementation Task that depends on them. Approval means: original
   wording, URLs present, no substantial copyrighted paste, no instruction to
   clone a vendor's exact UI.
3. **Allowed sources:** official public documentation, published manuals and
   getting-started guides, public tutorials/blogs/talks, public marketing
   screenshots and demo videos (for workflow understanding only), public API
   specifications and open standards.
4. **Forbidden:** decompiling or reverse-engineering binaries; copying
   proprietary source, icons, fonts, exact layouts, string tables, presets, or
   internal algorithms; using RE output as implementation input.

## Method

Recorded per note. Preferred order (guide §2.4):

1. a free model/MCP that can fetch public pages;
2. a CI/script step that fetches **text** from listed public URLs, which the
   agent then summarises into an original note;
3. human-assisted (last resort): the human pastes URLs or short excerpts and
   the agent writes the behavioural summary.

## Index

| Note | Capability | Status |
|---|---|---|
| — | (none yet — Rubric's implemented capabilities were designed from the original spec in `docs/spec/README.md`) | |

When a note is added, link it here with the capability it covers, its
approval date, and the approving human's initials.
