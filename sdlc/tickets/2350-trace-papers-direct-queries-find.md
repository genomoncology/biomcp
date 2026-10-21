# 2350 — Trace papers direct provider queries find

Status: OPEN.

Milestone: 0.9.4

Depends on: 2353
Depends on: 2354
Depends on: 2355

Reviews: revision d13912251a9a5631f2e58bad269e2ff10895c6fb, accept

## Outcome

For each of research's 42 direct-only candidates, starting with BRCA1 PMIDs 10486320 and 31788995, record whether an equivalent BioMCP query should find it, and why BioMCP missed it: fields, aliases, dates, page size or ranking. File one fix ticket per cause that applies to more than one paper. Fix every cause that applies to more than one paper in 0.9.4; Ian moved this in on 2026-10-21.

- Changes: `sdlc/records/2350-direct-only-paper-causes.md`.

- Proof: a record lists each candidate with its cause class, and an outside-in test per fixed cause recovers its papers.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 1: direct Europe PMC queries recovered 2 of BRCA1's expert papers that no BioMCP search returned, plus 40 other direct-only candidates. Direct queries often differ in fields, aliases, dates or page size.
- Primary expert-paper recall rose from 12 of 43 to 24 of 43 with uncapped search and direct calls.

- Coordinator restatement: account for each stable paper and each research observation, distinguish equivalent query failures from source/harness limitations, and assign confirmed multi-paper causes to bounded fixes without asserting every lexical candidate is the supplied allele.
- Fresh independent reviewer accepts investigation record d13912251a9a5631f2e58bad269e2ff10895c6fb: all56traces,42globalpapers,43case/papercontexts and27observationIDs verified against research561cb47. Actual archived j4307/j4335/j4346 support field comparisons and failures; invalidlimit/embedded-provider-DSL attempts remain excluded. Clinical statements match the cited specialist ruling. No additional provider request or product change in this record review.
- Child2353 owns persistent dated FANCL work,2354 owns exact10811118 managed conversion reads,2355 owns missing all-field compatible-alias discovery. Parent remains OPEN until each child's actual outside-in proof and disposition are recorded; this record acceptance does not claim those fixes landed or that final release qualification passed.
