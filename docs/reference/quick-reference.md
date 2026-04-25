# Quick Reference

This page is the high-signal command and vocabulary reference for day-to-day BioMCP use.
It focuses on frequently used commands, common filter values, and domain constants that are
useful for oncology and translational workflows.

## Install

**Binary installer (recommended):**

```bash
curl -fsSL https://biomcp.org/install.sh | bash
```

**PyPI tool install:**

```bash
uv tool install biomcp-cli
# or inside an active Python environment:
# pip install biomcp-cli
```

Install the `biomcp-cli` package, then use the `biomcp` command for the
commands below.

See [Installation](../getting-started/installation.md) for source-build and
platform-specific notes.

## Core command grammar

```text
biomcp search <entity> [filters]       # discovery
biomcp suggest <question>              # choose a worked-example playbook
biomcp discover <query>                # single-entity concept resolution before typed search/get
biomcp get <entity> <id> [section...]  # focused detail
biomcp <entity> <helper> <id>          # cross-entity pivots
biomcp enrich <GENE1,GENE2,...>        # gene-set enrichment
biomcp batch <entity> <id1,id2,...>    # parallel gets
```

## Common lookups

```bash
biomcp get gene BRAF
biomcp get gene BRAF pathways
biomcp get gene BRCA1 diagnostics
biomcp get variant "BRAF V600E"
biomcp get variant "BRAF V600E" clinvar
biomcp get article 22663011
biomcp get article 22663011 tldr
biomcp get article 22663011 fulltext
biomcp get trial NCT02576665
biomcp get trial NCT02576665 eligibility
biomcp get drug carboplatin shortage
biomcp get disease MONDO:0005105
biomcp get disease tuberculosis diagnostics
biomcp get pathway hsa05200 genes
biomcp get protein P15056 domains
biomcp get protein P15056 complexes
biomcp variant oncokb "BRAF V600E"
```

## Common searches

```bash
biomcp search gene BRAF --limit 5
biomcp search variant -g BRCA1 --significance pathogenic --limit 5
biomcp search trial -c melanoma --status recruiting --phase 2 --limit 5
biomcp search article -g BRAF -d melanoma --since 2024-01-01 --limit 5
biomcp search pathway -q "MAPK signaling" --limit 5
biomcp search pathway -q "Pathways in cancer" --limit 5
biomcp search protein -q kinase --limit 5
biomcp search adverse-event --drug pembrolizumab --serious --limit 5
biomcp search adverse-event "COVID-19 vaccine" --source all --limit 5
biomcp search adverse-event "MMR vaccine" --source vaers --limit 5
biomcp search diagnostic --gene BRCA1 --limit 5
biomcp search diagnostic --disease tuberculosis --source all --limit 5
biomcp search all --gene BRAF --disease melanoma
biomcp search all --keyword resistance --counts-only
biomcp suggest "What drugs treat melanoma?"
biomcp discover "chest pain"           # relational or multi-entity questions may redirect to search all --keyword
```

See also: [Search All Workflow](../how-to/search-all-workflow.md)

## Output modes and discovery commands

```bash
biomcp --json search gene -q BRAF --limit 3
biomcp --json suggest "When was imatinib approved?"
biomcp --json discover Keytruda
biomcp search trial -c melanoma --limit 3
biomcp list
biomcp list trial
biomcp health --apis-only
biomcp version
```

## Helper pivots

```bash
biomcp variant trials "BRAF V600E" --limit 3
biomcp variant articles "BRAF V600E"
biomcp drug adverse-events pembrolizumab --limit 3
biomcp drug trials pembrolizumab --limit 3
biomcp disease trials melanoma --limit 3
biomcp disease drugs melanoma --limit 3
biomcp disease articles "Lynch syndrome" --limit 3
biomcp gene trials BRAF --limit 3
biomcp gene drugs BRAF --limit 3
biomcp gene articles BRCA1 --limit 3
biomcp gene pathways BRAF
biomcp pathway drugs R-HSA-5673001 --limit 3
biomcp pathway drugs hsa05200 --limit 3
biomcp pathway articles R-HSA-5673001 --limit 3
biomcp pathway trials R-HSA-5673001 --limit 3
biomcp protein structures P15056
biomcp article entities 22663011
biomcp article citations 22663011 --limit 3
biomcp article references 22663011 --limit 3
biomcp article recommendations 22663011 --limit 3
```

## Study commands

`study` works on local downloaded cBioPortal-style datasets rather than the
remote entity APIs.

Set `BIOMCP_STUDY_DIR` when you want an explicit dataset root for downloads and
analysis; if it is unset, BioMCP uses its default study root.

| Command | Purpose |
|---------|---------|
| `biomcp study list` | List locally available studies |
| `biomcp study download [--list] [<study_id>]` | List downloadable study IDs or install a study locally |
| `biomcp study filter --study <id> [--mutated <symbol>] [--amplified <symbol>] [--deleted <symbol>] [--expression-above <gene:threshold>] [--expression-below <gene:threshold>] [--cancer-type <type>]` | Intersect mutation, CNA, expression, and clinical sample filters |
| `biomcp study query --study <id> --gene <symbol> --type <mutations|cna|expression>` | Summarize one gene within one study |
| `biomcp study cohort --study <id> --gene <symbol>` | Split a cohort into mutant vs wildtype groups |
| `biomcp study survival --study <id> --gene <symbol> [--endpoint <os|dfs|pfs|dss>]` | Compare mutation-defined groups on survival endpoints |
| `biomcp study compare --study <id> --gene <symbol> --type <expression|mutations> --target <symbol>` | Compare a target gene across mutation-defined groups |
| `biomcp study co-occurrence --study <id> --genes <g1,g2,...>` | Compute pairwise mutation co-occurrence across genes |

Examples:

```bash
biomcp study download --list
biomcp study download msk_impact_2017
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations
biomcp study filter --study brca_tcga_pan_can_atlas_2018 --mutated TP53 --amplified ERBB2 --expression-above ERBB2:1.5
biomcp study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type expression --target ERBB2
biomcp study co-occurrence --study msk_impact_2017 --genes TP53,KRAS
```

`study cohort`, `study survival`, and `study compare` require
`data_mutations.txt` and `data_clinical_sample.txt`. `study survival` also
requires `data_clinical_patient.txt` with endpoint columns, and expression
workflows require a supported expression matrix.

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

## NCI CTS trial notes

`search trial --source nci` keeps the shared BioMCP flags but translates them
to NCI CTS semantics:

- `--condition` remains the entry point. BioMCP prefers
  `diseases.nci_thesaurus_concept_id=<C-code>` when grounding succeeds and
  falls back to CTS `keyword=<text>` when it does not. There is no separate
  NCI keyword flag.
- `--status` accepts one normalized value at a time and maps it to CTS
  recruitment or lifecycle filters instead of sending raw CTGov status values.
- `--phase 1/2` becomes CTS `I_II`; `early_phase1` is unsupported on NCI.
- `--lat/--lon/--distance` becomes `sites.org_coordinates_lat`,
  `sites.org_coordinates_lon`, and `sites.org_coordinates_dist=<N>mi`.

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

- [Cross-Entity Pivot Guide](../how-to/cross-entity-pivots.md)
- [CLI Reference](../user-guide/cli-reference.md)
- [Data Sources](data-sources.md)
- [Troubleshooting](../troubleshooting.md)
