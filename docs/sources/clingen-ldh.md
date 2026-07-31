---
title: "ClinGen LDH article identity observations | BioMCP"
description: "Use bounded ClinGen Linked Data Hub article annotations as optional auditable variant identity observations."
---

# ClinGen Linked Data Hub

ClinGen Linked Data Hub (LDH) links CAids, genes, PMC articles, and exact text or
table selectors. BioMCP uses it only as an optional post-retrieval identity
observation for `variant articles --verify-identity`; it never discovers, removes,
or ranks article candidates.

It is a source of auditable linkage facts, not a measure of literature coverage or
scientific evidence strength.

## What BioMCP exposes

- `variant articles --verify-identity` can add a typed `clingen_ldh_annotation` linkage to an
  article identity observation.
- LDH requires an applicable CAR CAid and an already-known matching PMCID.
- Missing coverage, malformed LDH data, outages, or bounded-work exhaustion leave
  the candidate available and unverified; they do not prove absence.
- BioMCP bounds each item to one medium lookup, five matching PMC candidates, two
  annotations per candidate, and ten direct annotation requests.

## Example commands

```bash
biomcp variant articles 'BRAF V600E' --verify-identity
```

Verify already retrieved article candidates for one named variant.

```bash
biomcp --json variant articles 'ATM c.1066-6T>G' --verify-identity
```

Inspect typed identity observations and their linkage provenance.

```bash
biomcp --json variant articles --input variants.json --verify-identity
```

Verify each of up to ten structured variant identities from a JSON array.

The JSON identity projection retains the source, selector, CAid, gene, PMCID, and
provenance for a confirmed LDH annotation. Ordinary verification covers only the
requested visible page. `--confirmed-only` verifies up to 50 ranked candidates
before filtering and pagination; it reports incomplete when that bound leaves
candidates unverified.

## API access

No BioMCP API key required.

## Official source

- <https://ldh.genome.network>

## Related docs

- [Variant user guide](../user-guide/variant.md)
- [Article user guide](../user-guide/article.md)
- [Data sources](../reference/data-sources.md)
- [Source licensing](../reference/source-licensing.md)
