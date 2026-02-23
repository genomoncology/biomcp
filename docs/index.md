# BioMCP

**Single-binary CLI and MCP server for querying biomedical databases.**
15 data sources, one command grammar, compact markdown output.

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
biomcp health --apis-only
biomcp search gene -q BRAF
biomcp get gene BRAF pathways
biomcp get variant "BRAF V600E" clinvar
biomcp variant trials "BRAF V600E" --limit 5
```

## Command grammar

```
search <entity> [filters]    → discovery
get <entity> <id> [sections] → focused detail
<entity> <helper> <id>       → cross-entity pivots
```

## Entities and sources

| Entity | Source | Example |
|--------|--------|---------|
| gene | MyGene.info | `biomcp get gene BRAF` |
| variant | MyVariant.info (ClinVar, gnomAD) | `biomcp get variant "BRAF V600E"` |
| trial | ClinicalTrials.gov / NCI CTS | `biomcp search trial -c melanoma` |
| article | PubMed / PubTator3 | `biomcp search article -g BRAF` |
| drug | MyChem.info | `biomcp get drug pembrolizumab` |
| disease | Monarch / MONDO | `biomcp get disease "Lynch syndrome"` |
| pathway | Reactome | `biomcp get pathway R-HSA-5673001` |
| protein | UniProt | `biomcp get protein P15056 domains` |
| adverse-event | OpenFDA FAERS | `biomcp search adverse-event -d pembrolizumab` |
| pgx | PharmGKB / CPIC | `biomcp get pgx CYP2D6 recommendations` |
| gwas | GWAS Catalog | `biomcp search gwas -g BRAF` |
| phenotype | Monarch Initiative | `biomcp search phenotype "HP:0001250"` |
| organization | NCI CTS | `biomcp search organization "Dana-Farber"` |
| intervention | NCI CTS | `biomcp search intervention pembrolizumab` |
| biomarker | NCI CTS | `biomcp search biomarker BRAF` |

## API keys

Most commands run without credentials. Optional keys improve quota headroom:

```bash
export NCBI_API_KEY="..."      # PubTator, PMC OA, NCBI ID converter
export OPENFDA_API_KEY="..."   # OpenFDA rate limits
export NCI_API_KEY="..."       # NCI CTS trial/vocabulary calls
export ONCOKB_TOKEN="..."      # OncoKB helper
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

## License

MIT
