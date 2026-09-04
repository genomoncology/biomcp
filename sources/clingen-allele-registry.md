---
title: "ClinGen Allele Registry normalization | BioMCP"
description: "Normalize supported versioned RefSeq HGVS values to ClinGen CAids with bounded source aliases."
---

# ClinGen Allele Registry

ClinGen Allele Registry (CAR) provides canonical CAids for supported versioned
RefSeq HGVS identities. BioMCP preserves source facts and bounded aliases; it
does not infer equivalence, register alleles, or assign clinical meaning.

CAR accepts only versioned `NM_...:c.` and `NC_...:g.` identifiers. Its output keeps
provider cardinality visible when aliases are bounded. External IDs retain the combined
pre-cap distinct dbSNP and ClinVar count, and report truncation when either source exceeds
its eight-ID rendering cap. A malformed provider response is reported as incomplete rather
than a conclusive lookup result.

## What BioMCP exposes

- `variant normalize car <HGVS>` looks up one versioned RefSeq transcript coding
  or genomic HGVS value.
- `variant normalize car --input <path|->` accepts a JSON array of 1-50 values.
- Typed MCP `variant_normalize_car` accepts the same bounded values in memory.

## Example commands

```bash
biomcp --json variant normalize car 'NM_000546.6:c.215C>G'
```

Retrieve a source-provided CAid and bounded aliases for one transcript HGVS.

```bash
biomcp --json variant normalize car --input car-hgvs.json
```

Retrieve an ordered bounded batch from a bare JSON array.

```bash
printf '["NM_000546.6:c.215C>G"]' | biomcp --json variant normalize car --input -
```

Read the same bounded JSON array from standard input.

## API access

No BioMCP API key required.

## Official source

- <https://reg.genome.network>

## Related docs

- [Variant guide](../user-guide/variant.md)
- [Data sources](../reference/data-sources.md)
- [Source licensing](../reference/source-licensing.md)
