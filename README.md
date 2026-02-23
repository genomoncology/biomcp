# BioMCP

BioMCP is a single-binary CLI and MCP server for querying biomedical databases.
One command grammar, compact markdown output, 12 entities across 15+ data sources.

## Install

### Binary install (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/genomoncology/biomcp/main/install.sh | bash
```

### Install skills

Install guided investigation workflows into your agent directory:

```bash
biomcp skill install ~/.claude --force
```

### For Claude Desktop / Cursor / MCP clients

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "biomcp",
      "args": ["serve"]
    }
  }
}
```

### From source

```bash
cargo build --release --locked
```

## Quick start

```bash
biomcp health --apis-only            # verify API connectivity
biomcp list                          # show all entities and commands
biomcp list gene                     # show gene-specific filters and examples
```

## Command grammar

```
search <entity> [filters]    → discovery
get <entity> <id> [sections] → focused detail
<entity> <helper> <id>       → cross-entity pivots
enrich <GENE1,GENE2,...>     → gene-set enrichment
batch <entity> <id1,id2,...> → parallel gets
```

## Entities and sources

| Entity | Sources | Example |
|--------|---------|---------|
| gene | MyGene.info, UniProt, Reactome, QuickGO, STRING, CIViC | `biomcp get gene BRAF pathways` |
| variant | MyVariant.info, ClinVar, gnomAD, CIViC, OncoKB, cBioPortal, GWAS Catalog, AlphaGenome | `biomcp get variant "BRAF V600E" clinvar` |
| article | PubMed, PubTator3, Europe PMC | `biomcp search article -g BRAF --limit 5` |
| trial | ClinicalTrials.gov, NCI CTS API | `biomcp search trial -c melanoma -s recruiting` |
| drug | MyChem.info, ChEMBL, OpenTargets, Drugs\@FDA, CIViC | `biomcp get drug pembrolizumab targets` |
| disease | Monarch Initiative, MONDO, CIViC, OpenTargets | `biomcp get disease "Lynch syndrome" genes` |
| pathway | Reactome, g:Profiler | `biomcp get pathway R-HSA-5673001 genes` |
| protein | UniProt, InterPro, STRING, PDB/AlphaFold | `biomcp get protein P15056 domains` |
| adverse-event | OpenFDA (FAERS, MAUDE, Recalls) | `biomcp search adverse-event -d pembrolizumab` |
| pgx | CPIC, PharmGKB | `biomcp get pgx CYP2D6 recommendations` |
| gwas | GWAS Catalog | `biomcp search gwas --trait "type 2 diabetes"` |
| phenotype | Monarch Initiative (HPO) | `biomcp search phenotype "HP:0001250"` |

## Cross-entity helpers

Pivot between related entities without rebuilding filters:

```bash
biomcp variant trials "BRAF V600E" --limit 5
biomcp variant articles "BRAF V600E"
biomcp drug adverse-events pembrolizumab
biomcp drug trials pembrolizumab
biomcp disease trials melanoma
biomcp disease drugs melanoma
biomcp disease articles "Lynch syndrome"
biomcp gene articles BRCA1
biomcp gene pathways BRAF
biomcp pathway drugs R-HSA-5673001
biomcp protein structures P15056
biomcp article entities 22663011
```

## Gene-set enrichment

```bash
biomcp enrich BRAF,KRAS,NRAS --limit 10
```

## Sections and progressive disclosure

Every `get` command supports selectable sections for focused output:

```bash
biomcp get gene BRAF                    # summary card
biomcp get gene BRAF pathways           # add pathway section
biomcp get gene BRAF civic interactions # multiple sections
biomcp get gene BRAF all                # everything

biomcp get variant "BRAF V600E" clinvar population conservation
biomcp get drug pembrolizumab label shortage targets indications approvals
biomcp get disease "Lynch syndrome" genes phenotypes variants
biomcp get trial NCT02576665 eligibility locations outcomes
```

## API keys

Most commands work without credentials. Optional keys improve rate limits:

```bash
export NCBI_API_KEY="..."      # PubTator, PMC OA, NCBI ID converter
export OPENFDA_API_KEY="..."   # OpenFDA rate limits
export NCI_API_KEY="..."       # NCI CTS trial search (--source nci)
export ONCOKB_TOKEN="..."      # OncoKB variant helper
export ALPHAGENOME_API_KEY="..." # AlphaGenome variant effect prediction
```

## Multi-worker deployment

BioMCP rate limiting is process-local. For many concurrent workers, run one shared
`biomcp serve-http` endpoint so all workers share a single limiter budget:

```bash
biomcp serve-http --host 0.0.0.0 --port 8080
```

## Skills

14 guided investigation workflows are built in:

```bash
biomcp skill list
biomcp skill show 03
```

| # | Skill | Focus |
|---|-------|-------|
| 01 | variant-to-treatment | Variant annotation to treatment options |
| 02 | drug-investigation | Drug mechanism, safety, alternatives |
| 03 | trial-searching | Trial discovery and patient matching |
| 04 | rare-disease | Rare disease evidence and trial strategy |
| 05 | drug-shortages | Shortage monitoring and alternatives |
| 06 | advanced-therapies | CAR-T and checkpoint therapy workflows |
| 07 | hereditary-cancer | Hereditary cancer syndrome workup |
| 08 | resistance | Resistance mechanisms and next-line options |
| 09 | gene-function-lookup | Gene-centric function and context |
| 10 | gene-set-analysis | Enrichment, pathway, and interaction synthesis |
| 11 | literature-synthesis | Evidence synthesis with cross-entity checks |
| 12 | pharmacogenomics | PGx gene-drug interactions and dosing |
| 13 | phenotype-triage | Symptom-first rare disease workup |
| 14 | protein-pathway | Protein structure and pathway deep dive |

## Ops

```bash
biomcp version          # show version and build info
biomcp health           # check all API connectivity
biomcp update           # self-update to latest release
biomcp update --check   # check for updates without installing
biomcp uninstall        # remove biomcp from ~/.local/bin
```

## Documentation

Full documentation at [biomcp.org](https://biomcp.org/).

- [Getting Started](docs/getting-started/installation.md)
- [Data Sources](docs/reference/data-sources.md)
- [Quick Reference](docs/reference/quick-reference.md)
- [Troubleshooting](docs/troubleshooting.md)

## License

MIT
