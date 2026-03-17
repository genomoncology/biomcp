# BioMCP

BioMCP gives researchers, clinicians, and agents one command grammar across biomedical APIs that normally require separate identifiers, search forms, and output conventions. It keeps results compact and evidence-oriented so you can move quickly from orientation to detail. When public APIs are not enough, the same binary also runs local study analytics on downloaded cBioPortal datasets.

## Install

### PyPI tool install

```bash
uv tool install biomcp-cli
# or, inside an active Python environment:
# pip install biomcp-cli
```

Install the `biomcp-cli` package, then use `biomcp` for the commands shown
throughout the docs.

### Binary install

```bash
curl -fsSL https://biomcp.org/install.sh | bash
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

### Remote HTTP server

For shared or remote deployments, start BioMCP over Streamable HTTP instead of
stdio:

```bash
biomcp serve-http --host 127.0.0.1 --port 8080
```

Remote clients connect to `http://127.0.0.1:8080/mcp`. Probe routes are
`/health`, `/readyz`, and `/`. See
[Remote HTTP Server](getting-started/remote-http.md) for setup details,
`getting-started/remote-http.md` in the docs tree, and
`demo/streamable_http_client.py` for a runnable client example.

### From source

```bash
cargo build --release --locked
```

## Quick start

```bash
uv tool install biomcp-cli
biomcp health --apis-only
biomcp list gene
biomcp search all --gene BRAF --disease melanoma  # unified cross-entity discovery
biomcp get gene BRAF pathways druggability
```

## Command grammar

```text
search <entity> [filters]    → discovery
get <entity> <id> [sections] → focused detail
<entity> <helper> <id>       → cross-entity pivots
enrich <GENE1,GENE2,...>     → gene-set enrichment
batch <entity> <id1,id2,...> → parallel gets
search all [slot filters]    → counts-first cross-entity orientation
```

## Feature highlights

- **Federated article search:** PubTator3 and Europe PMC run together for `search article`, then deduplicate by PMID.
- **Cross-entity pivots:** move directly from a known entity into trials, articles, drugs, pathways, structures, or article graph helpers.
- **Study analytics + charts:** `study` commands support local cohort analytics plus native terminal, SVG, and PNG chart output.
- **Citation graph helpers:** `article citations`, `article references`, and `article recommendations` add literature navigation from a known paper when `S2_API_KEY` is configured.
- **Gene-set enrichment and batch retrieval:** `biomcp enrich` uses g:Profiler, and `biomcp batch` runs up to 10 focused `get` calls with shared JSON metadata.

## Entities and sources

| Entity | Upstream providers used by BioMCP | Example |
|--------|-----------------------------------|---------|
| gene | MyGene.info, UniProt, Reactome, QuickGO, STRING, GTEx, DGIdb, ClinGen | `biomcp get gene BRAF pathways` |
| variant | MyVariant.info, ClinVar, gnomAD fields via MyVariant, CIViC, Cancer Genome Interpreter, OncoKB, cBioPortal, GWAS Catalog, AlphaGenome | `biomcp get variant "BRAF V600E" clinvar` |
| article | PubMed, PubTator3, Europe PMC, PMC OA, NCBI ID Converter, Semantic Scholar (optional with `S2_API_KEY`) | `biomcp search article -g BRAF --limit 5` |
| trial | ClinicalTrials.gov API v2, NCI CTS API | `biomcp search trial -c melanoma -s recruiting` |
| drug | MyChem.info, ChEMBL, OpenTargets, Drugs@FDA, OpenFDA, CIViC | `biomcp get drug pembrolizumab targets` |
| disease | MyDisease.info, Monarch Initiative, MONDO, OpenTargets, Reactome, CIViC | `biomcp get disease "Lynch syndrome" genes` |
| pathway | Reactome, g:Profiler, Enrichr-backed enrichment sections | `biomcp get pathway R-HSA-5673001 genes` |
| protein | UniProt, InterPro, STRING, PDB, AlphaFold | `biomcp get protein P15056 domains` |
| adverse-event | OpenFDA FAERS, MAUDE, Recalls | `biomcp search adverse-event --drug pembrolizumab` |
| pgx | CPIC, PharmGKB | `biomcp get pgx CYP2D6 recommendations` |
| gwas | GWAS Catalog | `biomcp search gwas --trait "type 2 diabetes"` |
| phenotype | Monarch Initiative (HPO semantic similarity) | `biomcp search phenotype "HP:0001250"` |

## Cross-entity helpers

See the [cross-entity pivot guide](how-to/cross-entity-pivots.md) for when to
use a helper versus a fresh search.

```bash
biomcp variant trials "BRAF V600E" --limit 5
biomcp variant articles "BRAF V600E"
biomcp drug adverse-events pembrolizumab
biomcp drug trials pembrolizumab
biomcp disease trials melanoma
biomcp disease drugs melanoma
biomcp disease articles "Lynch syndrome"
biomcp gene trials BRAF
biomcp gene drugs BRAF
biomcp gene articles BRCA1
biomcp gene pathways BRAF
biomcp pathway drugs R-HSA-5673001
biomcp pathway articles R-HSA-5673001
biomcp pathway trials R-HSA-5673001
biomcp protein structures P15056
biomcp article entities 22663011
biomcp article citations 22663011 --limit 3
biomcp article references 22663011 --limit 3
biomcp article recommendations 22663011 --limit 3
```

## Gene-set enrichment

```bash
biomcp enrich BRAF,KRAS,NRAS --limit 10
```

Top-level `biomcp enrich` uses **g:Profiler**. Gene enrichment sections inside
other entity pages still describe **Enrichr** where that is the source.

## API keys

Most commands work without credentials. Optional keys improve rate limits:

```bash
export NCBI_API_KEY="..."        # PubTator, PMC OA, NCBI ID converter
export S2_API_KEY="..."          # Semantic Scholar TLDR, citations, references, recommendations
export OPENFDA_API_KEY="..."     # OpenFDA rate limits
export NCI_API_KEY="..."         # NCI CTS trial search (--source nci)
export ONCOKB_TOKEN="..."        # OncoKB variant helper
export ALPHAGENOME_API_KEY="..." # AlphaGenome variant effect prediction
```

## Skills

BioMCP ships an embedded guide for agent workflows rather than a built-in
catalog. Read it with `biomcp skill`, install it with
`biomcp skill install ~/.claude --force`, and see
[Skills](getting-started/skills.md) for the current workflow and legacy notes.

## Local study analytics

`study` is BioMCP's local analysis family for downloaded cBioPortal-style datasets.
The 12 remote entity commands handle live API-backed discovery and detail; `study`
commands cover local query, cohort, survival, compare, and co-occurrence workflows.

```bash
export BIOMCP_STUDY_DIR="$HOME/.local/share/biomcp/studies"
biomcp study download msk_impact_2017
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations --chart bar --theme dark --palette wong -o docs/blog/images/tp53-mutation-bar.svg
```

## Documentation

- [Installation](getting-started/installation.md)
- [First Query](getting-started/first-query.md)
- [Search All Workflow](how-to/search-all-workflow.md)
- [Data Sources](reference/data-sources.md)
- [Quick Reference](reference/quick-reference.md)
- [Troubleshooting](troubleshooting.md)

## Citation

If you use BioMCP in research, cite it via [`CITATION.cff`](https://github.com/genomoncology/biomcp/blob/main/CITATION.cff).
GitHub also exposes `Cite this repository` in the repository sidebar when that file is present.

## License

MIT
