---
flow: build
priority: 8
---

# Render the author card's follow-ups from the same source JSON uses

Filed from `sdlc/issues/2026-08-27-the-markdown-author-card-drops-the-next-commands-json-carries.md`,
which carries the repro and the verified mechanism — read it first.

The markdown author card hardcodes `next_commands: vec![]`
(`src/render/markdown/author.rs:178`), so it ends at the ORCID line while
the `--json` surface carries the truthful pivot (`author papers`, wired by
1060). The two surfaces disagree; markdown users never see the follow-up.

## Done when

- The markdown author card renders its More/See-also block from the same
  next-commands source the JSON `_meta` uses — one source, two renderings,
  structurally unable to diverge.
- The author card in markdown shows the author-papers pivot for an author
  whose record is available, pinned by a test that renders both surfaces
  and asserts the command sets agree.
- The one-source pattern is checked across the other compact-card
  renderers: any other renderer found hardcoding its own command list
  either migrates to the shared source or is named here with a reason.

Filed as build, not quickfix: green suite, authored proof, and the
fix is structural (shared source) rather than a one-line fill-in.
