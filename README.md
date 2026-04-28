# BioMCP

One binary. One grammar. Evidence from the biomedical sources you already trust.

## Description

BioMCP cuts through the usual biomedical data maze: one query reaches the
sources that normally live behind different APIs, identifiers, and search
habits. Researchers, clinicians, and agents use the same command grammar to
search, focus, and pivot without rebuilding the workflow for each source. You
get compact, evidence-oriented results across live public data plus local study analytics.

## Features

- **Search the literature:** `search article` fans out across PubTator3 and
  Europe PMC, deduplicates PMID/PMCID/DOI identifiers, and can add a Semantic
  Scholar leg when your filters support it.
- **Pivot without rework:** move from a gene, variant, drug, disease, pathway,
  protein, or article straight into the next built-in view instead of
  rebuilding filters by hand.
- **Choose a playbook:** `biomcp suggest "<question>"` routes a biomedical
  question to a shipped worked example and two starter commands.
- **Analyze studies locally:** `study` commands cover local query, cohort, survival,
  compare, and co-occurrence workflows with native terminal, SVG, and PNG
  charts for downloaded cBioPortal-style datasets.
- **Follow the paper trail:** `article citations`, `article references`,
  `article recommendations`, and `article entities` turn one known paper into a
  broader evidence map.
- **Enrich and batch:** use `biomcp enrich` for top-level g:Profiler
  enrichment and `biomcp batch` for up to 10 focused `get` calls in one
  command.

## Installation

### Binary install

```bash
curl -fsSL https://biomcp.org/install.sh | bash
```

### PyPI tool install

```bash
uv tool install biomcp-cli
# or: pip install biomcp-cli
```

This installs the `biomcp` binary on your PATH.

### Claude Desktop extension (.mcpb)

Install BioMCP from the Anthropic Directory in Claude Desktop when that path is
available for your environment. For local/manual setups, use the JSON MCP
config below.

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
uv run --script examples/streamable-http/streamable_http_client.py
```

See [Remote HTTP Server](https://biomcp.org/getting-started/remote-http/) for
the newcomer guide.

### From source

```bash
make install
"$HOME/.local/bin/biomcp" --version
```

For repo-local verification, `make check` now includes the release-critical
Python/docs contract lane (`make test-contracts`), and `make release-gate` is
the full release-readiness command (`make check` plus `make spec-pr`).

## Quick start

First useful query in under 30 seconds:

```bash
uv tool install biomcp-cli
biomcp health --apis-only
biomcp suggest "What drugs treat melanoma?"
biomcp list gene
biomcp search all --gene BRAF --disease melanoma  # unified cross-entity discovery
biomcp get gene BRAF pathways hpa
```

## Command grammar

```text
search <entity> [filters]    → discovery
suggest <question>           → playbook routing for how-to questions
discover <query>             → concept resolution before entity selection
get <entity> <id> [sections] → focused detail
<entity> <helper> <id>       → cross-entity pivots
enrich <GENE1,GENE2,...>     → gene-set enrichment
batch <entity> <id1,id2,...> → parallel gets
search all [slot filters]    → counts-first cross-entity orientation
```

## Entities and sources

The 13-row table below is the public entity surface; individual rows may use
live APIs, local runtime data, or hybrid source overlays.

| Entity | Upstream providers used by BioMCP | Example |
|--------|-----------------------------------|---------|
| gene | MyGene.info, UniProt, Reactome, QuickGO, STRING, GTEx, Human Protein Atlas, DGIdb, ClinGen, NIH Reporter, DisGeNET, GTR-backed diagnostics pivot | `biomcp get gene BRAF pathways hpa` |
| variant | MyVariant.info, ClinVar, gnomAD fields via MyVariant, CIViC, Cancer Genome Interpreter, OncoKB, cBioPortal, GWAS Catalog, AlphaGenome | `biomcp get variant "BRAF V600E" clinvar` |
| article | PubMed, PubTator3, Europe PMC, PMC OA, NCBI ID Converter, Semantic Scholar (optional auth; `S2_API_KEY` recommended) | `biomcp search article -g BRAF --limit 5` |
| trial | ClinicalTrials.gov API v2, NCI CTS API | `biomcp search trial -c melanoma -s recruiting` |
| diagnostic | NCBI Genetic Testing Registry local bulk bundle + WHO IVD local CSV + optional OpenFDA device overlay | `biomcp get diagnostic GTR000006692.3 regulatory` |
| drug | MyChem.info, DDInter local bundle, EMA local batch, WHO Prequalification local exports, ChEMBL, OpenTargets, Drugs@FDA, OpenFDA labels/shortages/approvals/FAERS/MAUDE/recalls, CIViC | `biomcp drug interactions warfarin` |
| disease | MyDisease.info, Monarch Initiative, MONDO, OpenTargets, Reactome, CIViC, SEER Explorer, NIH Reporter, DisGeNET, MedlinePlus `clinical_features`, GTR/WHO IVD diagnostics pivot | `biomcp get disease "Lynch syndrome" genes` |
| pathway | Reactome, KEGG, WikiPathways, g:Profiler, Enrichr-backed enrichment sections | `biomcp get pathway hsa05200 genes` |
| protein | UniProt, InterPro, STRING, ComplexPortal, PDB, AlphaFold | `biomcp get protein P15056 complexes` |
| adverse-event | OpenFDA FAERS/MAUDE/recalls plus CDC WONDER VAERS aggregate vaccine search | `biomcp search adverse-event --drug pembrolizumab` |
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
other entity views still reference **Enrichr** where that is the backing
source.

## Sections and progressive disclosure

Every `get` command supports selectable sections for focused output:

```bash
biomcp get gene BRAF                    # summary card
biomcp get gene BRAF pathways           # add pathway section
biomcp get gene BRCA1 diagnostics       # diagnostic-test pivot from GTR
biomcp get gene BRAF hpa                # protein tissue expression + localization
biomcp get gene BRAF civic interactions # multiple sections
biomcp get gene BRAF all                # standard sections; diagnostics/funding stay opt-in

biomcp get variant "BRAF V600E" clinvar population conservation
biomcp get article 22663011 tldr
biomcp get drug pembrolizumab label targets civic approvals
biomcp get drug trastuzumab regulatory --region who
biomcp get disease "Lynch syndrome" genes phenotypes variants
biomcp get disease tuberculosis diagnostics
biomcp get diagnostic GTR000006692.3 regulatory
biomcp get trial NCT02576665 eligibility locations outcomes
```

In JSON mode, `get` responses expose `_meta.next_commands` for the next likely
follow-ups and `_meta.section_sources` for section-level provenance. `batch ...
--json` returns per-entity objects with the same metadata shape.

## API keys

Most commands work without credentials. Optional keys improve rate limits or
unlock optional enrichments:

```bash
export NCBI_API_KEY="..."        # PubTator, PubMed/efetch, PMC OA, NCBI ID converter
export S2_API_KEY="..."          # Optional Semantic Scholar auth; dedicated quota at 1 req/sec
export OPENFDA_API_KEY="..."     # OpenFDA rate limits
export NCI_API_KEY="..."         # NCI CTS trial search (--source nci)
export ONCOKB_TOKEN="..."        # OncoKB variant helper
export ALPHAGENOME_API_KEY="..." # AlphaGenome variant effect prediction
```

`search article`, `get article`, `article batch`, `get article ... tldr`, and
the explicit Semantic Scholar helpers all work without `S2_API_KEY`. With the
key, BioMCP sends authenticated requests and uses a dedicated rate limit at
1 req/sec. Without it, BioMCP uses the shared unauthenticated pool at 1 req/2sec.
`search article --source` supports `all`, `pubtator`, `europepmc`, and
`pubmed`. The
default compatible article federation uses PubTator3, Europe PMC, and PubMed,
while the S2 leg remains automatic rather than directly selectable. References
and recommendations can be empty for paywalled papers because of publisher
elision in Semantic Scholar upstream coverage.

## Configuration

### Claude Desktop extension settings

The directory bundle exposes only the optional settings needed for the first
reviewer-facing build:

| Claude Desktop field | Runtime env var | Purpose |
|----------------------|-----------------|---------|
| OncoKB Token | `ONCOKB_TOKEN` | Enables `biomcp variant oncokb "<gene> <variant>"` therapy and level evidence |
| DisGeNET API Key | `DISGENET_API_KEY` | Enables scored DisGeNET sections on gene and disease lookups |
| Semantic Scholar API Key | `S2_API_KEY` | Improves reliability for article TLDR, citation, reference, and recommendation helpers |

The first directory build exposes only those three optional settings. Advanced
CLI-only env vars remain documented in
[API Keys](docs/getting-started/api-keys.md) for the general BioMCP CLI path.

## Usage Examples

### Public cross-entity overview

**User prompt:** Give me a low-noise overview of BRAF in melanoma.

**Expected tool call:** `biomcp search all --gene BRAF --disease melanoma --counts-only`

**Expected behavior:** Returns a cross-entity counts summary that orients the
next command instead of dumping long detail tables.

**Expected output:** Counts-first summary with suggested next commands for the
highest-yield entity follow-ups.

### Public variant evidence

**User prompt:** Summarize ClinVar significance and population frequency for BRAF V600E.

**Expected tool call:** `biomcp get variant "BRAF V600E" clinvar population`

**Expected behavior:** Retrieves the focused variant card, ClinVar section, and
population-frequency data in one read-only call.

**Expected output:** Variant summary, ClinVar significance details, and gnomAD
population frequencies.

### Credentialed OncoKB example

**User prompt:** Show OncoKB therapy evidence for BRAF V600E.

**Expected tool call:** `biomcp variant oncokb "BRAF V600E"`

**Expected behavior:** Uses `ONCOKB_TOKEN` when configured and otherwise
returns helpful guidance about the missing credential.

**Expected output:** Therapy and level evidence when `ONCOKB_TOKEN` is set, or
a clear setup hint when it is not.

### Credentialed DisGeNET example

**User prompt:** Show scored DisGeNET associations for TP53.

**Expected tool call:** `biomcp get gene TP53 disgenet`

**Expected behavior:** Uses `DISGENET_API_KEY` to retrieve the scored
gene-disease association section.

**Expected output:** Ranked disease-association table with evidence counts and
scores when `DISGENET_API_KEY` is configured.

## Privacy Policy

BioMCP does not add telemetry, analytics, or remote log upload. Review the
full privacy statement at <https://biomcp.org/policies/>.

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
Use `biomcp suggest "<question>"` when you need the right worked example,
then use `biomcp skill` to read the embedded BioMCP guide or install it into
your agent directory when you want local copies of the workflow references:

```bash
biomcp suggest "Is variant rs113488022 pathogenic in melanoma?"
biomcp skill
biomcp skill install ~/.claude --force
```

See [Skills](docs/getting-started/skills.md) for supported install targets,
installed files, and legacy compatibility notes.

## Local study analytics

`study` is BioMCP's local analysis family for downloaded cBioPortal-style datasets.
The public entity surface handles API-backed, local-runtime, and hybrid
discovery/detail; `study` commands work on local datasets when you need
per-study query, cohort, survival, comparison, or co-occurrence workflows.

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
biomcp version                            # show version and build info
biomcp health                             # inspect API connectivity plus local DDInter/EMA/cache readiness
biomcp update                             # self-update with release SHA256 checksum verification
biomcp update --check                     # check for updates without installing
biomcp update --allow-missing-checksum    # UNSAFE: install when a release checksum sidecar is missing
biomcp uninstall                          # remove biomcp from ~/.local/bin
```

## Support

- GitHub issues: <https://github.com/genomoncology/biomcp/issues>
- Troubleshooting: [docs/troubleshooting.md](docs/troubleshooting.md)
- Full documentation: <https://biomcp.org/>

## Documentation

- [Getting Started](docs/getting-started/installation.md)
- [Search All Workflow](docs/how-to/search-all-workflow.md)
- [BioASQ Benchmark](docs/reference/bioasq-benchmark.md)
- [Cross-Entity Pivot Guide](docs/how-to/cross-entity-pivots.md)
- [Privacy Policy](docs/policies.md)
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
