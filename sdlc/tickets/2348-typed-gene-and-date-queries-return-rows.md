# 2348 — Return rows for typed gene and date queries

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 88bcf385a17ec951ec55e7bc816b5299db2930e8, accept

Landed: 05ac371

## Outcome

Article searches with a typed gene or date filter return the rows that the equivalent literal or long-name query finds, or explain in-band why a typed filter excluded them. Typed filters never silently return zero rows where the literal query finds matches.

- Proof: outside-in replay tests compare the study's typed and literal queries and show the typed query returns the expected rows or an in-band reason.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 7: typed gene and date queries return zero rows where literal or long-name queries return candidates. Exact commands are in 0041-tool-issues.tsv.

- Coordinator restatement: preserve typed identity/date and explain annotation-index scope through the existing note schema. Gator j4152 RED on ff4e1c04e: FANCL --date-from 2026-09-01 --source europepmc --limit 50 --debug-plan returned complete true/count 0 without a reason; literal FANCL OR long-name replay returns rows including 42850567. Both preserve FIRST_PDATE. Outside-in assertion failed on missing GENE_PROTEIN explanation. This proves annotation-index versus literal scope, not a date defect. Approved shared planner note and existing CLI callsites must cover JSON and Markdown without gene-token fallback or schema expansion. Query generation remains owned by 2346.

## Changes

- Share article search limitation notes across JSON, Markdown, discovery and debug plans. Explain Europe PMC's GENE_PROTEIN annotation-index restriction and the literal --keyword alternative without changing typed identity or date filtering.
- Compare the study's FANCL/date typed and literal queries through the CLI using the recorded literal response and a controlled empty annotation-index overlay; require an in-band explanation in JSON and Markdown.

- Final affected proof j4204 passed on published base 735678cb7982abf7664d89a04413cd405b70d44f plus exact patch SHA256 79b622f1d5e838f0ec7477575def92da71b27c96035402e3cc4e89b7133a5e36, matching candidate 88bcf385a17ec951ec55e7bc816b5299db2930e8: 19 Python CLI cases, 49 targeted Rust cases, formatting, Ruff and all-target Clippy. The explanation appears only on empty anchored Europe PMC pages; typed-positive output remains unchanged. Replay evidence distinguishes captured literal rows from the controlled empty annotation overlay and makes no live comparison claim. Initial formatting-only j4175 and prior j4179 results remain historical.
- Fresh independent reviewer accepts 88bcf385a17ec951ec55e7bc816b5299db2930e8 after whole-diff/caller review, actual RED, JSON/Markdown/debug behavior and positive control; no fixes. Yellow ticket-evidence/2348/developer-proof.tar.gz SHA256 e9be34af4323b31ac8fa3053668a3134b27c585b2ec85e09f872fbc2df86e151 retains raw proof. Batch landing gate remains required.

- Shared landing candidate aa3feea1b797e8c96dbb885c6e03c4f6251b767d passed the direct declared M5 make landing-gate in j4354 (exit 0; scheduler 824773 ms): 4295 Rust cases with 36 skipped, 1997 Python cases with 19 skipped, strict documentation, 11 spec golden cases and 7 static specifications. Fresh incremental combined-source review accepted this exact candidate after both bounded fixture repairs. Darwin does not supply Linux network-isolation proof. Yellow release-evidence/0.9.4/landing-research-2346-2348-2349-2351 retains source bindings and all gate attempts; SHA256SUMS SHA256 8e0e7742d920eeee8f8e36ee66c7948c97e9e9d4890acfb43bb68b718e705965. Subsequent ticket-only records preserve every non-SDLC mode and blob; no repeat gate follows publication.
