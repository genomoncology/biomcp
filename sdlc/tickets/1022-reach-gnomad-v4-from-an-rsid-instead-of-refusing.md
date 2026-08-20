---
flow: build
priority: 18
---
# Reach gnomAD v4 population data from an rsID

`get variant rs334 population` returns population data on the published 0.8.25 release and returns `inapplicable` on the current development build. The message is "Direct gnomAD v4 population data requires a trustworthy GRCh38 coordinate." The same happens for `get variant "BRAF V600E" population`. This is a regression against shipped behavior, and it lands on the headline feature of the next minor release: an rsID is how most people name a variant, so the new gnomAD v4 output is currently unreachable by the most natural route into it.

The refusal itself is correct and must stay. MyVariant.info answers an rsID lookup with a GRCh37 coordinate and reports `"genome_build": "GRCh37", "genome_build_provenance": "MyVariant.info provider default"`. Silently treating that as GRCh38 would report the wrong position's frequencies, which is worse than refusing. Nothing here should guess a coordinate or perform a liftover.

What is missing is that a trustworthy GRCh38 coordinate is already obtainable without guessing. dbSNP returns one directly: for rs334 the SPDI is `NC_000011.10:5227001:T:A`, which is `NC_000011.10:g.5227002T>A` in the form this command already accepts and answers. BioMCP already treats dbSNP as a source and already prints a dbSNP link in this command's own output. So the fix is to resolve the identifier through a source that states the assembly, rather than to relax the check on a source that does not.

`--assembly hg38` does not currently help, because it is rejected for anything that is not a chromosome-prefixed genomic coordinate: `Error: Invalid argument: --assembly only applies to chromosome-prefixed genomic coordinates`. Setting `BIOMCP_DEFAULT_ASSEMBLY=hg38` has no effect on this path either. Decide deliberately whether either of those should apply once an rsID resolves to a real GRCh38 coordinate, and say so in the design.

## Done when

- `get variant rs334 population` returns gnomAD v4 population data, with the same exome and genome separation, allele counts, ancestry rows, FAF95, and filter status that the explicit `NC_` accession form already returns today.
- The values returned for a resolved rsID are identical to the values returned when the equivalent GRCh38 accession is supplied directly.
- The resolved coordinate's assembly is stated in the response along with the source that supplied it, so a reader can tell resolution happened and who is accountable for it.
- An identifier that genuinely cannot be resolved to a GRCh38 coordinate from an assembly-stating source still returns `inapplicable` with a message naming what was tried. No liftover is performed anywhere.
- A GRCh37-only answer from a provider is never treated as GRCh38.

## Existing tests that pin this

Restatement is authorized in these files, for these tests by name, only to the extent they assert that an rsID or a gene-and-change input yields an `inapplicable` population outcome:

- `src/entities/variant/get/tests.rs` — `population_request_requires_a_grch38_genomic_coordinate`
- `src/render/markdown/variant/tests.rs` — `variant_population_markdown_keeps_missing_status_compact`

No other test file is authorized. Do not weaken any assertion that a GRCh37 coordinate is refused as GRCh38 — that is the safety property this ticket preserves, and it must still hold at the end.

If the design stage finds a further shipped assertion that pins the old refusal and is not named above, stop and say so in the design output rather than restating it; that is a ticket amendment, not a design decision.

## Verification note

The reference values were confirmed against the gnomAD GraphQL API for `11-5227002-T-A` in `gnomad_r4` on 2026-08-19: exomes AC 2335, AN 1458356, 31 homozygotes, grpmax FAF95 0.05474387 (afr); genomes AC 1937, AN 152294, grpmax FAF95 0.04188667 (afr). The development build already reproduces every one of these exactly when given the GRCh38 accession, so the data path is correct and only the identifier path is at fault.

## Addendum, 2026-08-20 — the source inventories

Attempt 1 refused at code review on traceability: commit `49d4ba10`, an ordinary `code:` commit, changed `src/cli/health/tests/catalog.rs` and `tests/test_source_licensing_docs_contract.py`. The refusal is correct — a test may only change in a commit whose message says a test-owning stage made it — and the branch was cleared rather than continued, because a commit-history refusal cannot be repaired by adding a later commit. The discarded work is preserved under the tag `attempt/1022-20260820-1`.

The changes themselves were right, and the ticket should have foreseen them. This ticket makes dbSNP a registered source, and both of those files enumerate every source the project has: one lists the names the health inventory must report and what each affects, the other maps source modules to their licensing entries. Adding a source adds a line to each. That is not restating an assertion to match the code — it is an inventory acquiring the entry the work just created.

Restatement is authorized in `src/cli/health/tests/catalog.rs` and `tests/test_source_licensing_docs_contract.py`, bounded strictly to adding the dbSNP entry to a list that enumerates sources — its name in the health inventory, what it affects, and its licensing module mapping. Nothing else in either file is opened. No existing entry may be removed, renamed, or weakened, and no assertion about any other source may change.

These additions belong in a `design:` or `design-review:` commit, like every other test change. That is the whole of what went wrong: the content was correct and its placement was not.

Everything the ticket already said stands. In particular, the two named tests in `src/entities/variant/get/tests.rs` and `src/render/markdown/variant/tests.rs` keep their existing bounds, the GRCh37-is-never-GRCh38 assertions keep full strength, and the instruction above still holds — a further shipped assertion that pins the old refusal, not named here, is a ticket amendment and not a design decision. Nothing in this addendum authorizes touching an assertion about the refusal behavior; it covers source inventories only.
