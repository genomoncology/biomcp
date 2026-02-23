# Quick Reference

This page is the high-signal command and vocabulary reference for day-to-day BioMCP use.
It focuses on frequently used commands, common filter values, and domain constants that are
useful for oncology and translational workflows.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/genomoncology/biomcp/main/install.sh | bash
```

## Core command grammar

```text
biomcp search <entity> [filters]       # discovery
biomcp get <entity> <id> [section...]  # focused detail
biomcp <entity> <helper> <id>          # cross-entity pivots
biomcp enrich <GENE1,GENE2,...>        # gene-set enrichment
biomcp batch <entity> <id1,id2,...>    # parallel gets
```

## Common lookups

```bash
biomcp get gene BRAF
biomcp get gene BRAF pathways
biomcp get variant "BRAF V600E"
biomcp get variant "BRAF V600E" clinvar
biomcp get article 22663011
biomcp get article 22663011 fulltext
biomcp get trial NCT02576665
biomcp get trial NCT02576665 eligibility
biomcp get drug carboplatin shortage
biomcp get disease MONDO:0005105
biomcp get pathway R-HSA-5673001 genes
biomcp get protein P15056 domains
biomcp variant oncokb "BRAF V600E"
```

## Common searches

```bash
biomcp search gene -q BRAF --limit 5
biomcp search variant -g BRCA1 --significance pathogenic --limit 5
biomcp search trial -c melanoma --status recruiting --phase 2 --limit 5
biomcp search article -g BRAF -d melanoma --since 2024-01-01 --limit 5
biomcp search pathway -q "MAPK signaling" --limit 5
biomcp search protein -q kinase --limit 5
biomcp search adverse-event --drug pembrolizumab --serious --limit 5
```

## Output modes and discovery commands

```bash
biomcp --json search gene -q BRAF --limit 3
biomcp search trial -c melanoma --limit 3
biomcp list
biomcp list trial
biomcp health --apis-only
biomcp version
```

## Helper pivots

```bash
biomcp variant trials "BRAF V600E" --limit 3
biomcp drug adverse-events pembrolizumab --limit 3
biomcp disease drugs melanoma --limit 3
biomcp gene pathways BRAF
biomcp pathway drugs R-HSA-5673001 --limit 3
biomcp protein structures P15056
```

## Common gene aliases

Use these aliases in `search` when a clinical report or paper does not use the HGNC symbol.
Follow with `get gene <SYMBOL>` once you identify the canonical symbol.

| Alias in literature | Official symbol |
|---------------------|-----------------|
| HER1 | EGFR |
| HER2 | ERBB2 |
| P53 | TP53 |
| C-KIT | KIT |
| PD-1 | PDCD1 |
| PD-L1 | CD274 |
| MLH-1 | MLH1 |
| MSH-2 | MSH2 |

## Trial geographic search quick coordinates

Use `--lat`, `--lon`, and `--distance` for trial site proximity filtering.
Coordinates below are common starting points for regional searches.

| City | State | Latitude | Longitude |
|------|-------|----------|-----------|
| Boston | MA | 42.3601 | -71.0589 |
| New York | NY | 40.7128 | -74.0060 |
| Chicago | IL | 41.8781 | -87.6298 |
| Houston | TX | 29.7604 | -95.3698 |
| Los Angeles | CA | 34.0522 | -118.2437 |
| San Francisco | CA | 37.7749 | -122.4194 |
| Seattle | WA | 47.6062 | -122.3321 |
| Atlanta | GA | 33.7490 | -84.3880 |

Example:

```bash
biomcp search trial -c melanoma --lat 42.3601 --lon -71.0589 --distance 50 --limit 5
```

## Trial status values

`--status` accepts ClinicalTrials.gov style recruitment states. Common values:

| Status value | Meaning |
|--------------|---------|
| recruiting | Currently enrolling participants |
| not yet recruiting | Opened but enrollment not started |
| active, not recruiting | Ongoing study, enrollment closed |
| completed | Study finished |
| terminated | Stopped early |
| suspended | Temporarily paused |
| withdrawn | Stopped before enrollment |
| unknown status | Last known status is unclear |

## Trial phase values

`--phase` accepts either numeric shorthand or explicit phase labels.

| Input | Interpreted as |
|-------|----------------|
| `1` | `PHASE1` |
| `2` | `PHASE2` |
| `3` | `PHASE3` |
| `4` | `PHASE4` |
| `phase1` | `PHASE1` |
| `phase2` | `PHASE2` |
| `phase3` | `PHASE3` |
| `phase4` | `PHASE4` |

## Clinical significance values (variant search)

Use these with `biomcp search variant --significance <value>`.

| Significance value | Typical interpretation |
|--------------------|------------------------|
| pathogenic | Strong evidence for disease association |
| likely_pathogenic | Evidence leans disease-associated |
| uncertain_significance | Evidence is currently inconclusive |
| likely_benign | Evidence leans non-pathogenic |
| benign | Strong evidence against pathogenicity |
| conflicting_interpretations | Submitters disagree |
| risk_factor | Associated with risk, not deterministic |

## Variant consequence values

Use these with `biomcp search variant --consequence <value>`.

| Consequence value | Description |
|-------------------|-------------|
| missense_variant | Amino acid substitution |
| nonsense_variant | Introduces stop codon |
| synonymous_variant | No amino acid change |
| frameshift_variant | Reading-frame disruption |
| splice_acceptor_variant | Splice acceptor disruption |
| splice_donor_variant | Splice donor disruption |
| inframe_deletion | In-frame codon deletion |
| inframe_insertion | In-frame codon insertion |
| stop_lost | Stop codon removed |
| start_lost | Start codon removed |

## Related references

- [CLI Reference](../user-guide/cli-reference.md)
- [Data Sources](data-sources.md)
- [Troubleshooting](../troubleshooting.md)
