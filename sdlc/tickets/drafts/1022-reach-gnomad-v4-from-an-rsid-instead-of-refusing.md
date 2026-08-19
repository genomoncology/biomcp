---
flow: build
priority: 8
hold: draft for review; do not promote until Ian releases this
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

Restatement is authorized for the files below by name, to the extent they assert that an rsID input yields an `inapplicable` population outcome. Do not weaken any assertion that a GRCh37 coordinate is refused as GRCh38 — that is the safety property this ticket preserves.

- Any test asserting `inapplicable` for the population section on rsID input under `tests/` — the design stage must name the exact files it touches in its commit message.
- Spec pages under `spec/` covering `get variant ... population`.

## Verification note

The reference values were confirmed against the gnomAD GraphQL API for `11-5227002-T-A` in `gnomad_r4` on 2026-08-19: exomes AC 2335, AN 1458356, 31 homozygotes, grpmax FAF95 0.05474387 (afr); genomes AC 1937, AN 152294, grpmax FAF95 0.04188667 (afr). The development build already reproduces every one of these exactly when given the GRCh38 accession, so the data path is correct and only the identifier path is at fault.
