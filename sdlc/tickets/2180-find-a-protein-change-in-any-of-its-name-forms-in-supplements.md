---
priority: 2
---

# 2180 — Find a protein change in any of its name forms in supplements

Status: OPEN.

Milestone: 0.9.3

## Outcome

`article supplement-search` finds a protein change written in one-letter, three-letter or HGVS protein form, whichever form the query uses, and the help example finds its row.

- Evidence: research's spot check: BioMCP's own help example `biomcp --json article supplement-search 28944238 P245S` (`src/cli/article/mod.rs:430`) finds nothing on live data because sheet ST2 spells the change `p.Pro245Ser`.
- Keeps: exact locators (sheet and row), and no fuzzy matching beyond the defined name forms of the same protein change.
- Changes: claims the supplement search matcher under `src/transform/article/supplement_text/` and `src/cli/article/`, its tests, `CHANGELOG.md`. Expand a protein change query to its one-letter, three-letter and `p.` forms before matching.
- Proof: a recorded supplement whose sheet says `p.Pro245Ser` returns ST2 row 85 for `P245S`, `Pro245Ser` and `p.Pro245Ser`; on current main `P245S` returns nothing. The existing BRCA1 F1761I row still matches.

Coordinator outcome before code: Match only defined equivalent protein name forms in supplement text while retaining exact sheet/row locators, and make the documented example find its row. Reuse existing approved name mappings rather than invent clinical equivalence. Before product edits, reproduce on current main through the build job and report the exact command/output, proposed implementation/test files and the outside-in failing proof. Root records that evidence and scope before implementation. Developers write no ticket or status records.

## Progress

- 2026-10-17 started
