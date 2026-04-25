# BioMCP

One binary. One grammar. Evidence from the biomedical sources you already trust.

BioMCP replaces the usual routine of jumping between database sites, auth
schemes, and API docs with one command surface for researchers, clinicians, and
agents. Search, drill into detail, pivot to the next entity, and keep the
output compact enough to stay useful. When public APIs are not enough, the same
binary also runs local study analytics on downloaded cBioPortal datasets.

## Install

### Binary install

```bash
curl -fsSL https://biomcp.org/install.sh | bash
```

### PyPI tool install

```bash
uv tool install biomcp-cli
# or, inside an active Python environment:
# pip install biomcp-cli
```

Install the `biomcp-cli` package, then use `biomcp` for the commands shown
throughout the docs.

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
`examples/streamable-http/streamable_http_client.py` for a runnable client example.

### From source

```bash
make install
"$HOME/.local/bin/biomcp" --version
```

## Quick start

Install to first result in under 30 seconds:

```bash
uv tool install biomcp-cli
biomcp health --apis-only
biomcp suggest "What drugs treat melanoma?"
biomcp discover "chest pain"
biomcp list gene
biomcp search all --gene BRAF --disease melanoma  # unified cross-entity discovery
biomcp get gene BRAF pathways hpa
```

## Command grammar

```text
search <entity> [filters]    → discovery
suggest <question>           → playbook routing for how-to questions
discover <query>             → single-entity concept resolution before entity selection
get <entity> <id> [sections] → focused detail
<entity> <helper> <id>       → cross-entity pivots
enrich <GENE1,GENE2,...>     → gene-set enrichment
batch <entity> <id1,id2,...> → parallel gets
search all [slot filters]    → counts-first cross-entity orientation
```

## Feature highlights

- **Search the literature:** `search article` fans out across PubTator3 and
  Europe PMC, deduplicates PMID/PMCID/DOI identifiers, and can add a Semantic
  Scholar leg when your filters support it.
- **Resolve messy queries:** `biomcp discover` turns aliases, brands, symptoms,
  and close concept names into the right entity before you commit to a typed
  command. Relational or multi-entity questions may redirect to
  `biomcp search all --keyword "<query>"`.
- **Choose the workflow:** `biomcp suggest "<question>"` returns the matching
  worked-example playbook and two starter commands.
- **Pivot across entities:** move from a known gene, variant, drug, disease,
  pathway, protein, or article into trials, articles, drugs, pathways,
  structures, or article graph helpers without rebuilding context.
- **Analyze studies locally:** `study` commands cover local query, cohort, survival,
  compare, and co-occurrence workflows with native terminal, SVG, and PNG
  charts for downloaded cBioPortal-style datasets.
- **Follow the paper trail:** `article citations`, `article references`,
  `article recommendations`, and `article entities` expand one paper into its
  surrounding evidence with optional Semantic Scholar auth.
- **Enrich and batch:** `biomcp enrich` uses g:Profiler, and `biomcp batch`
  runs up to 10 focused `get` calls with shared JSON metadata.

## Entities and sources

The 13-row table below is the public entity surface; individual rows may use
live APIs, local runtime data, or hybrid source overlays.

| Entity | Upstream providers used by BioMCP | Example |
|--------|-----------------------------------|---------|
| gene | MyGene.info, UniProt, Reactome, QuickGO, STRING, GTEx, Human Protein Atlas, DGIdb, ClinGen, NIH Reporter, DisGeNET, GTR-backed diagnostics pivot | `biomcp get gene ERBB2 funding` |
| variant | MyVariant.info, ClinVar, gnomAD fields via MyVariant, CIViC, Cancer Genome Interpreter, OncoKB, cBioPortal, GWAS Catalog, AlphaGenome | `biomcp get variant "BRAF V600E" clinvar` |
| article | PubMed, PubTator3, Europe PMC, PMC OA, NCBI ID Converter, Semantic Scholar (optional auth; `S2_API_KEY` recommended) | `biomcp search article -g BRAF --limit 5` |
| trial | ClinicalTrials.gov API v2, NCI CTS API | `biomcp search trial -c melanoma -s recruiting` |
| diagnostic | NCBI Genetic Testing Registry local bulk bundle + WHO IVD local CSV + optional OpenFDA device overlay | `biomcp get diagnostic GTR000006692.3 regulatory` |
| drug | MyChem.info, DDInter local bundle, EMA local batch, WHO Prequalification local exports, ChEMBL, OpenTargets, Drugs@FDA, OpenFDA labels/shortages/approvals/FAERS/MAUDE/recalls, CIViC | `biomcp drug interactions warfarin` |
| disease | MyDisease.info, Monarch Initiative, MONDO, OpenTargets, Reactome, CIViC, SEER Explorer, NIH Reporter, DisGeNET, MedlinePlus `clinical_features`, GTR/WHO IVD diagnostics pivot | `biomcp get disease "chronic myeloid leukemia" funding` |
| pathway | Reactome, KEGG, WikiPathways, g:Profiler, Enrichr-backed enrichment sections | `biomcp get pathway hsa05200 genes` |
| protein | UniProt, InterPro, STRING, ComplexPortal, PDB, AlphaFold | `biomcp get protein P15056 complexes` |
| adverse-event | OpenFDA FAERS/MAUDE/recalls plus CDC WONDER VAERS aggregate vaccine search | `biomcp search adverse-event --drug pembrolizumab` |
| pgx | CPIC, PharmGKB | `biomcp get pgx CYP2D6 recommendations` |
| gwas | GWAS Catalog | `biomcp search gwas --trait "type 2 diabetes"` |
| phenotype | Monarch Initiative (HPO semantic similarity) | `biomcp search phenotype "HP:0001250"` |

## Cross-entity pivots

See the [cross-entity pivot guide](how-to/cross-entity-pivots.md) for when to
use a helper, a sectioned diagnostic pivot, or a fresh search.

```bash
biomcp variant trials "BRAF V600E" --limit 5
biomcp variant articles "BRAF V600E"
biomcp drug adverse-events pembrolizumab
biomcp drug trials pembrolizumab
biomcp disease trials melanoma
biomcp disease drugs melanoma
biomcp disease articles "Lynch syndrome"
biomcp get disease tuberculosis diagnostics
biomcp gene trials BRAF
biomcp gene drugs BRAF
biomcp gene articles BRCA1
biomcp gene pathways BRAF
biomcp get gene BRCA1 diagnostics
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
other entity pages still describe **Enrichr** where that is the source.

## API keys

Most commands work without credentials. Optional keys improve rate limits:

```bash
export NCBI_API_KEY="..."        # PubTator, PubMed/efetch, PMC OA, NCBI ID converter
export S2_API_KEY="..."          # Optional Semantic Scholar auth; dedicated quota at 1 req/sec
export OPENFDA_API_KEY="..."     # OpenFDA rate limits
export NCI_API_KEY="..."         # NCI CTS trial search (--source nci)
export DISGENET_API_KEY="..."    # Scored DisGeNET gene/disease sections
export ONCOKB_TOKEN="..."        # OncoKB variant helper
export UMLS_API_KEY="..."        # discover crosswalk enrichment
export ALPHAGENOME_API_KEY="..." # AlphaGenome variant effect prediction
```

## Data Sources and Licensing

BioMCP is MIT-licensed. It performs on-demand queries against upstream providers instead of vendoring or mirroring their datasets, but upstream terms govern reuse of retrieved results.

Some providers are fully open, some BioMCP features require registration or API keys, and some queryable sources still impose notable reuse limits. The two biggest cautions are KEGG, which distinguishes academic and non-academic use, and COSMIC, which BioMCP keeps indirect-only because its licensing model is incompatible with a direct open integration.

Use [Source Licensing and Terms](reference/source-licensing.md) for the per-source breakdown and [API Keys](getting-started/api-keys.md) for setup steps and registration links.

## Skills

BioMCP ships an embedded guide for agent workflows rather than a built-in
catalog. Read it with `biomcp skill`, install it with
`biomcp skill install ~/.claude --force`, and see
[Skills](getting-started/skills.md) for the current workflow and legacy notes.

## Local study analytics

`study` is BioMCP's local analysis family for downloaded cBioPortal-style datasets.
The public entity surface handles API-backed, local-runtime, and hybrid
discovery/detail; `study` commands cover local query, cohort, survival,
compare, and co-occurrence workflows.

```bash
export BIOMCP_STUDY_DIR="$HOME/.local/share/biomcp/studies"
biomcp study download msk_impact_2017
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations --chart bar --theme dark --palette wong -o docs/blog/images/tp53-mutation-bar.svg
```

## Documentation

- [Installation](getting-started/installation.md)
- [First Query](getting-started/first-query.md)
- [Search All Workflow](how-to/search-all-workflow.md)
- [BioASQ Benchmark](reference/bioasq-benchmark.md)
- [Discover](user-guide/discover.md)
- [Source Licensing and Terms](reference/source-licensing.md)
- [Data Sources](reference/data-sources.md)
- [Quick Reference](reference/quick-reference.md)
- [Troubleshooting](troubleshooting.md)

## Citation

If you use BioMCP in research, cite it via [`CITATION.cff`](https://github.com/genomoncology/biomcp/blob/main/CITATION.cff).
GitHub also exposes `Cite this repository` in the repository sidebar when that file is present.

## License

MIT
