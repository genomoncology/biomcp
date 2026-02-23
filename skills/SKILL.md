---
name: biomcp
description: Search and retrieve biomedical data — genes, variants, clinical trials, articles, drugs, diseases, pathways, proteins, adverse events, pharmacogenomics, and phenotype-disease matching. 15 sources including PubMed, ClinicalTrials.gov, ClinVar, OncoKB, Reactome, UniProt, PharmGKB, OpenFDA, Monarch. Use when asked about gene function, variant pathogenicity, trial matching, drug safety, resistance mechanisms, hereditary syndromes, or literature evidence.
---

# BioMCP CLI

BioMCP is a biomedical command-line tool plus an embedded workflow catalog for agents. The CLI supports direct lookup/search tasks, while skills provide reusable multi-step investigation patterns.

## Quick Start

Run these three commands first in every new session:

```bash
biomcp health
biomcp list
biomcp list gene
```

What each command gives you:
- `biomcp health`: verifies source/API connectivity.
- `biomcp list`: shows all entities, patterns, and helper families.
- `biomcp list <entity>`: shows detailed per-entity filters, IDs, and examples.

## Command Model

BioMCP follows a small command grammar:

- `biomcp search <entity> [filters]`
- `biomcp get <entity> <id> [section...]`
- `biomcp <entity-family> <helper> <id-or-name>`

Examples:

```bash
biomcp get gene BRAF pathways
biomcp get variant rs113488022 clinvar
biomcp get article 22663011 annotations
biomcp get trial NCT02576665 eligibility
biomcp get drug carboplatin shortage
```

## Search Patterns

Free-text searches:
- `gene`, `disease`, `drug`, `pathway`, `protein` support `-q` (and positional query input).
- `article` supports `-k/--keyword` and `-q/--query` aliases (and positional query input).

Structured/filter-first searches:
- `variant` uses structured filters (for example: gene, significance, consequence, protein change).
- `trial` uses structured filters (for example: condition, intervention, mutation, phase, status).

Query examples:

```bash
biomcp search disease "lung cancer"
biomcp search gene -q BRAF --limit 5
biomcp search article -q "immunotherapy resistance" --limit 5
biomcp search variant -g BRAF --significance pathogenic --limit 5
biomcp search trial -c melanoma --mutation "BRAF V600E" --status recruiting --limit 5
```

## Cross-Entity Helpers

Helpers let you pivot quickly between related entities without manually rebuilding filters.

```bash
biomcp variant trials "BRAF V600E"
biomcp variant articles "BRAF V600E"
biomcp drug adverse-events pembrolizumab
biomcp disease trials melanoma
biomcp gene articles BRCA1
biomcp pathway drugs R-HSA-5673001
biomcp protein structures P15056 --limit 25
```

## Per-Entity Guides

Use `biomcp list <entity>` for the full reference page of an entity.

Each page includes:
- supported command forms (`search`, `get`, helpers)
- ID formats and section names
- filter vocabulary
- practical examples

Common entries:

```bash
biomcp list gene
biomcp list variant
biomcp list article
biomcp list trial
biomcp list protein
```

## Skills

Skills are step-by-step investigation workflows. Each skill chains multiple BioMCP commands into a validated research pattern.

Use skills when you need a structured investigation, not just a single lookup.

```bash
biomcp skill list
biomcp skill 03
biomcp skill gene-set-analysis
```

| # | Slug | Focus |
|---|------|-------|
| 01 | `variant-to-treatment` | Variant to treatment/evidence workflow |
| 02 | `drug-investigation` | Drug mechanism, safety, alternatives |
| 03 | `trial-searching` | Trial discovery + patient matching |
| 04 | `rare-disease` | Rare disease evidence and trial strategy |
| 05 | `drug-shortages` | Shortage monitoring and alternatives |
| 06 | `advanced-therapies` | CAR-T/checkpoint therapy workflows |
| 07 | `hereditary-cancer` | Hereditary syndrome workflows |
| 08 | `resistance` | Resistance and next-line options |
| 09 | `gene-function-lookup` | Gene-centric function and context lookup |
| 10 | `gene-set-analysis` | Enrichment + pathway + interaction synthesis |
| 11 | `literature-synthesis` | Evidence synthesis with cross-entity checks |
| 12 | `pharmacogenomics` | PGx gene-drug interactions and dosing |
| 13 | `phenotype-triage` | Symptom-first rare disease workup |
| 14 | `protein-pathway` | Protein structure and pathway deep dive |

## Common Pitfalls

- Variant IDs with shell metacharacters must be quoted:

```bash
biomcp get variant "chr7:g.140453136A>T"
```

- Variant `search` vs `get`:
  - `search variant` is filter-based.
  - `get variant` accepts rsID, HGVS, or `GENE CHANGE` formats.

- Best-effort helpers search free-text fields (for example, eligibility criteria or abstracts). Results depend on source document wording.

- If zero results:
  - broaden to a higher-level entity (for example, `gene` before `variant`),
  - try alternate wording/synonyms,
  - try a different source option where supported (for example, `--source nci` for trial workflows).
