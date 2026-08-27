---
flow: build
priority: 9
deps: ["1069"]
---

# Markdown and JSON must agree — the cross-surface command contract test

The markdown author card ends at the ORCID line while the same request in
`--json` carries the author-papers pivot (1069 fixes that instance by
rendering both surfaces from one source). The bug class is surfaces
drifting: two renderings, each maintained by hand, disagreeing about what
the card suggests next. Nothing today fails a build when they diverge.

## Done when

- An offline contract test renders the same fixture entity both ways —
  markdown and `--json` — for every card family that carries
  `_meta.next_commands`, extracts the command set each surface presents
  (the markdown More/See-also blocks; the JSON `_meta.next_commands`), and
  asserts the two sets agree, per the one-source policy 1069 establishes.
- Where a surface legitimately presents more than the other (a documented
  asymmetry), the test encodes that asymmetry explicitly with the reason,
  so any new asymmetry is a deliberate, reviewed diff rather than drift.
- The test fails on pre-1069 code for the author card (proving teeth) and
  passes after 1069 lands (the dep ordering enforces this).
- Coverage includes at least: gene, disease, drug, trial, article, author,
  adverse-event.

This makes "the two surfaces disagree" a build failure instead of a
support ticket.
