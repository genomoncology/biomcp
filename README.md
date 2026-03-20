# BioMCP

BioMCP gives researchers, clinicians, and agents one command grammar across biomedical APIs that usually require separate search habits, identifiers, and output formats. It keeps results compact and evidence-oriented so you can move from discovery to detail without rewriting the workflow for each source. One command grammar, compact markdown output, 12 remote entities across 15+ data sources, plus local study analytics.

## Install

### PyPI tool install

```bash
uv tool install biomcp-cli
# or: pip install biomcp-cli
```

This installs the `biomcp` binary on your PATH.

### Binary install

```bash
curl -fsSL https://biomcp.org/install.sh | bash
```

### Install skills

Install guided investigation workflows into your agent directory:

```bash
biomcp skill install ~/.claude --force
```

### MCP clients

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

For shared or remote deployments:

```bash
biomcp serve-http --host 127.0.0.1 --port 8080
```

Remote clients connect to `http://127.0.0.1:8080/mcp`. Probe routes are
`GET /health`, `GET /readyz`, and `GET /`.

Runnable demo:

```bash
uv run --script demo/streamable_http_client.py
```

See [Remote HTTP Server](https://biomcp.org/getting-started/remote-http/) for
the newcomer guide.

### From source

```bash
cargo build --release --locked
```

## Quick start

First useful query in under 30 seconds:

```bash
uv tool install biomcp-cli
biomcp health --apis-only
biomcp list gene
biomcp search all --gene BRAF --disease melanoma  # unified cross-entity discovery
biomcp get gene BRAF pathways hpa
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

- **Federated article search:** `search article` fans out across PubTator3 and Europe PMC, optionally adds a Semantic Scholar search leg when `S2_API_KEY` is set, merges identifiers across PMID/PMCID/DOI, and ranks relevance directness-first.
- **Cross-entity pivots:** move directly from a gene, variant, drug, disease, pathway, protein, or article into the next built-in view.
- **Study analytics and charting:** downloaded studies support query, cohort, survival, compare, and co-occurrence workflows with native terminal or SVG charts.
- **Citation graphs and article helpers:** `article citations`, `article references`, `article recommendations`, and `article entities` support literature navigation from a known paper.
- **Gene-set enrichment and batch retrieval:** use `biomcp enrich` for top-level g:Profiler enrichment and `biomcp batch` for up to 10 focused `get` calls in one command.

## Entities and sources

| Entity | Upstream providers used by BioMCP | Example |
|--------|-----------------------------------|---------|
| gene | MyGene.info, UniProt, Reactome, QuickGO, STRING, GTEx, Human Protein Atlas, DGIdb, ClinGen | `biomcp get gene BRAF pathways hpa` |
| variant | MyVariant.info, ClinVar, gnomAD fields via MyVariant, CIViC, Cancer Genome Interpreter, OncoKB, cBioPortal, GWAS Catalog, AlphaGenome | `biomcp get variant "BRAF V600E" clinvar` |
| article | PubMed, PubTator3, Europe PMC, PMC OA, NCBI ID Converter, Semantic Scholar (optional with `S2_API_KEY`) | `biomcp search article -g BRAF --limit 5` |
| trial | ClinicalTrials.gov API v2, NCI CTS API | `biomcp search trial -c melanoma -s recruiting` |
| drug | MyChem.info, ChEMBL, OpenTargets, Drugs@FDA, OpenFDA, CIViC | `biomcp get drug pembrolizumab targets` |
| disease | MyDisease.info, Monarch Initiative, MONDO, OpenTargets, Reactome, CIViC | `biomcp get disease "Lynch syndrome" genes` |
| pathway | Reactome, KEGG, g:Profiler, Enrichr-backed enrichment sections | `biomcp get pathway hsa05200 genes` |
| protein | UniProt, InterPro, STRING, ComplexPortal, PDB, AlphaFold | `biomcp get protein P15056 complexes` |
| adverse-event | OpenFDA FAERS, MAUDE, Recalls | `biomcp search adverse-event --drug pembrolizumab` |
| pgx | CPIC, PharmGKB | `biomcp get pgx CYP2D6 recommendations` |
| gwas | GWAS Catalog | `biomcp search gwas --trait "type 2 diabetes"` |
| phenotype | Monarch Initiative (HPO semantic similarity) | `biomcp search phenotype "HP:0001250"` |

## Cross-entity helpers

Pivot between related entities without rebuilding filters.

See the [cross-entity pivot guide](docs/how-to/cross-entity-pivots.md) for when
to use a helper versus a fresh search.

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
biomcp pathway drugs hsa05200
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
other entity views still reference **Enrichr** where that is the backing source.

## Sections and progressive disclosure

Every `get` command supports selectable sections for focused output:

```bash
biomcp get gene BRAF                    # summary card
biomcp get gene BRAF pathways           # add pathway section
biomcp get gene BRAF hpa                # protein tissue expression + localization
biomcp get gene BRAF civic interactions # multiple sections
biomcp get gene BRAF all                # everything

biomcp get variant "BRAF V600E" clinvar population conservation
biomcp get article 22663011 tldr
biomcp get drug pembrolizumab label targets civic approvals
biomcp get disease "Lynch syndrome" genes phenotypes variants
biomcp get trial NCT02576665 eligibility locations outcomes
```

In JSON mode, `get` responses expose `_meta.next_commands` for the next likely
follow-ups and `_meta.section_sources` for section-level provenance. `batch ...
--json` returns per-entity objects with the same metadata shape.

## API keys

Most commands work without credentials. Optional keys improve rate limits or
unlock optional enrichments:

```bash
export NCBI_API_KEY="..."        # PubTator, PMC OA, NCBI ID converter
export S2_API_KEY="..."          # Semantic Scholar search leg, TLDR, citations, references, recommendations
export OPENFDA_API_KEY="..."     # OpenFDA rate limits
export NCI_API_KEY="..."         # NCI CTS trial search (--source nci)
export ONCOKB_TOKEN="..."        # OncoKB variant helper
export ALPHAGENOME_API_KEY="..." # AlphaGenome variant effect prediction
```

`search article` works without `S2_API_KEY`; when the key is present it also
fans out to Semantic Scholar and exposes ranking/support metadata in the search
output. `--source` still remains `all|pubtator|europepmc` in v1, so the S2 leg
is automatic rather than directly selectable.
References and recommendations can be empty for paywalled papers because of
publisher elision in Semantic Scholar upstream coverage.

## Multi-worker deployment

BioMCP rate limiting is process-local. For many concurrent workers, run one shared
Streamable HTTP `biomcp serve-http` endpoint so all workers share a single
limiter budget:

```bash
biomcp serve-http --host 0.0.0.0 --port 8080
```

Remote clients should connect to `http://<host>:8080/mcp`. Lightweight process
probes are available at `GET /health`, `GET /readyz`, and `GET /`.

## Skills

BioMCP ships an embedded agent guide instead of a browsable in-binary catalog.
Use `biomcp skill` to read the embedded BioMCP guide, then install it into
your agent directory when you want local copies of the workflow references:

```bash
biomcp skill
biomcp skill install ~/.claude --force
```

See [Skills](docs/getting-started/skills.md) for supported install targets,
installed files, and legacy compatibility notes.

## Local study analytics

`study` is BioMCP's local analysis family for downloaded cBioPortal-style datasets.
The 12 remote entity commands query upstream APIs for discovery and detail; `study`
commands work on local datasets when you need per-study query, cohort, survival,
comparison, or co-occurrence workflows.

Use `study download` to fetch a dataset into your local study root. Set
`BIOMCP_STUDY_DIR` when you want an explicit dataset location for reproducible
scripts and demos; if it is unset, BioMCP falls back to its default study root.

```bash
export BIOMCP_STUDY_DIR="$HOME/.local/share/biomcp/studies"
biomcp study download msk_impact_2017
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations --chart bar --theme dark --palette wong -o docs/blog/images/tp53-mutation-bar.svg
```

See the [CLI reference](docs/user-guide/cli-reference.md#local-study-analytics)
for the full `study` command family and dataset prerequisites.

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
- [Search All Workflow](docs/how-to/search-all-workflow.md)
- [Cross-Entity Pivot Guide](docs/how-to/cross-entity-pivots.md)
- [Source Licensing and Terms](docs/reference/source-licensing.md)
- [Data Sources](docs/reference/data-sources.md)
- [Quick Reference](docs/reference/quick-reference.md)
- [Troubleshooting](docs/troubleshooting.md)

## Citation

If you use BioMCP in research, cite it via [`CITATION.cff`](CITATION.cff).
GitHub also exposes `Cite this repository` in the repository sidebar when that file is present.

## Data Sources and Licensing

BioMCP is MIT-licensed. It performs on-demand queries against upstream providers instead of vendoring or mirroring their datasets, but upstream terms govern reuse of retrieved results.

Some providers are fully open, some BioMCP features require registration or API keys, and some queryable sources still impose notable reuse limits. The two biggest cautions are KEGG, which distinguishes academic and non-academic use, and COSMIC, which BioMCP keeps indirect-only because its licensing model is incompatible with a direct open integration.

Use [Source Licensing and Terms](docs/reference/source-licensing.md) for the per-source breakdown and [API Keys](docs/getting-started/api-keys.md) for setup steps and registration links.

## License

MIT
