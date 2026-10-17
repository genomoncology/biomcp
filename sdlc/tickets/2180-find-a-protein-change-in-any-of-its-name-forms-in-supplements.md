---
priority: 2
---

# 2180 — Find a protein change in any of its name forms in supplements

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision 5151a59, accept

Landed: 5151a59 (fast-forward: Accepted work reached main in reviewed combined research-fix landing bf9916b; completion records this source batch without another source change.)

## Outcome

`article supplement-search` finds a protein change written in one-letter, three-letter or HGVS protein form, whichever form the query uses, and the help example finds its row.

- Evidence: research's spot check: BioMCP's own help example `biomcp --json article supplement-search 28944238 P245S` (`src/cli/article/mod.rs:430`) finds nothing on live data because sheet ST2 spells the change `p.Pro245Ser`.
- Keeps: exact locators (sheet and row), and no fuzzy matching beyond the defined name forms of the same protein change.
- Changes: claims `src/entities/article/supplement_search.rs` `src/cli/article/mod.rs` `tests/article_asset_terminal.rs` `tests/fixtures/article/supplements/28944238-axin2-st2.xlsx` `docs/user-guide/supplementary-materials.md` `CHANGELOG.md`. Match only the defined equivalent protein substitution forms, reusing the existing normalizer and residue mapping.
- Proof: a recorded supplement whose sheet says `p.Pro245Ser` returns ST2 row 85 for `P245S`, `Pro245Ser` and `p.Pro245Ser`; on current main `P245S` returns nothing. The existing BRCA1 F1761I row still matches.

Coordinator outcome before code: Match only defined equivalent protein name forms in supplement text while retaining exact sheet/row locators, and make the documented example find its row. Reuse existing approved name mappings rather than invent clinical equivalence. Before product edits, reproduce on current main through the build job and report the exact command/output, proposed implementation/test files and the outside-in failing proof. Root records that evidence and scope before implementation. Developers write no ticket or status records.

## Progress

- 2026-10-17 started


## Coordinator reproduction and scope

Qualified CLI source `926b5786d400d3ec8b54c929701c1e46c62b5a95` differs from current main `90c00c371` only by ticket records. CLI SHA256: `e9992fadcb93e290a6c310827a14368444dcd1305784661c659c46311ed9083d`.

Outcome: match the defined equivalent protein substitution forms with exact locators and ordinary literal matching intact. Correct matcher claim to `src/entities/article/supplement_search.rs`. Claims also include `tests/article_asset_terminal.rs` existing supplement case, `tests/fixtures/article/supplements/28944238-axin2-st2.xlsx`, `src/cli/article/mod.rs`, `docs/user-guide/supplementary-materials.md`, and changelog. Reuse existing normalization and residue mapping; no transcript/clinical identity changes.

Job 2180/1792273493204-70820 ran `biomcp --json article supplement-search 28944238 P245S`: hits=[]. Job 2180/1792273666120-86573 corrected recorded workbook C85 to actual p.Pro245Ser and failed the real CLI ST2-row85 assertion: output {"query":"P245S","hits":[]}, exit1. Authorize minimal matcher fix after this RED, three-form regression, preserved BRCA1 control and matching help/docs wording. Shared acceptance-file edits must remain in distinct existing functions.

Coordinator verification: GREEN2180/1792273913427-7868 passed all3 existing supplement-search CLI tests, targeted Clippy and formatting. Mechanical checks2180/1792274041338-54299 passed. Local source331cb34053fd6fa7c6d4ad1a82fa86318f6714bb matches the applied tested patch. All3 name forms reach ST2row85; BRCA1row2780, larger-token refusals and ordinary literal behavior remain covered. Existing runtime comparison rows never call supplement-search and remain attributed to their prior immutable candidate. Fresh review follows.
