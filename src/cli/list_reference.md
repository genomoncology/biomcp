# BioMCP Command Reference

BioMCP connects to PubMed, ClinicalTrials.gov, ClinVar, gnomAD, OncoKB, Reactome,
KEGG, UniProt, PharmGKB, CPIC, OpenFDA, Monarch Initiative, GWAS Catalog, and more.
One command grammar covers all entities.

## Quickstart

New to BioMCP? Try:

- `skill install` - install BioMCP skill guidance to your agent
- `skill status` - compare installed guidance with this BioMCP binary
- `mcp-config --client claude-desktop` - print local stdio MCP client config
- `skill list` - choose a worked-example playbook for a question
- `get gene BRAF` - look up a gene
- `get gene BRCA1 diagnostics` - inspect GTR diagnostic tests for a known gene
- `get disease tuberculosis diagnostics` - inspect up to 10 local diagnostic tests for a condition
- `search diagnostic --gene BRCA1 --limit 5` - find genetic tests for a known gene
- `search diagnostic --disease HIV --source who-ivd --limit 5`
  - find WHO infectious-disease diagnostics
- `get variant "BRAF V600E"` - annotate a variant
- `discover "chest pain"` - resolve a single-entity free-text phrase before choosing a typed command
- `search trial -c melanoma` - find clinical trials
- `search all --gene BRAF --disease melanoma` - cross-entity summary card

## When to Use What

| You want to know... | Start with |
|---|---|
| I have a biomedical question and need the right playbook | `skill list` then `skill <name>` |
| How much NIH funded a disease or gene | `get disease <name_or_id> funding` or `get gene <symbol> funding` |
| What drugs treat a disease | `search drug --indication "<disease>" --limit 5` |
| Diagnostic test for a gene or disease | `get gene <symbol> diagnostics`; `get disease <id> diagnostics`; or `search diagnostic --gene <symbol>` |
| What the 5-year survival outlook is for a cancer | `get disease <name_or_id> survival` |
| Symptoms or phenotypes of a disease | `get disease <name_or_id> phenotypes` |
| Monarch/HPO phenotype rows framed as clinical features | `get disease <name_or_id> clinical_features` |
| Which diseases match HPO IDs or symptom text | `search phenotype "<HP:... HP:...>"` or `search phenotype "seizure, developmental delay"` |
| What a gene does | `get gene <symbol>` |
| Tissue expression or localization of a gene product | `get gene <symbol> hpa` or `get gene <symbol> protein` |
| Drug safety or adverse events | `drug adverse-events <name>` or `get drug <name> safety` |
| Which drugs or drug classes interact with a known drug | `drug interactions <name>` or `get drug <name> interactions` |
| Review literature that synthesizes a topic | `search article -k "<query>" --type review --limit 5` |
| Turn a literature question into article filters | `biomcp list article` |
| Follow one article into related evidence | `article citations <id> --limit 5` or `article recommendations <id> --limit 5` |
| I know the entities but not the next pivot | `search all --gene BRAF --disease melanoma` |
| I only have a biomedical phrase and need routing | `discover "<free text>"`; fallback: `search all --keyword "<query>"` |
| The same sections for several entities | `batch <entity> <id1,id2,...> --sections <s1,s2,...>`; adverse-event batches do not support sections |
| Enriched pathways or functions for a gene set | `enrich <GENE1,GENE2,...>` |

{{ENTITY_CATALOG}}

## Patterns

- `search <entity> [query|filters]` - find entities
- `discover <query>` - resolve a single-entity free-text phrase into typed follow-up commands; relational questions may redirect to `search all --keyword`
- `search all [slot filters]` - curated multi-entity orientation (`--gene/--variant/--disease/--drug/--keyword`)
- `search trial [filters]` - trial search is filter-only
- `get <entity> <id> [section...]` - fetch by identifier with optional sections
- `get gene <symbol> diagnostics` - opt-in diagnostic-test pivot from a gene card
- `get disease <name_or_id> diagnostics`
  - opt-in diagnostic-test pivot capped at 10 rows
- `get disease <name_or_id> clinical_features`
  - opt-in Monarch/HPO phenotype rows framed as clinical features
- `get drug <name> regulatory [--region <us|eu|ema|who|all>]` - region-aware U.S./EU/WHO regulatory context
- `get drug <name> safety|shortage [--region <us|eu|ema|all>]` - region-aware U.S./EU drug safety and shortage context
- `get drug <name> all [--region <us|eu|ema|who|all>]` - include all sections plus region-aware regulatory context
- `ema` is accepted as an input alias for the canonical `eu` drug region value
- Omitting `--region` on `get drug <name> regulatory` checks combined regions.
- Other no-flag `get drug` shapes stay on the default U.S. path.
- `get trial <nct_id> --offset <N> --limit <N> locations` - page trial locations
- `enrich <GENE1,GENE2,...>` - gene-set enrichment via g:Profiler
- `batch <entity> <id1,id2,...>` - parallel get operations
- `mcp-config [--client <client>] [--absolute-path]` - print local stdio MCP client config
- `study list|download|top-mutated|filter|query|co-occurrence|cohort|survival|compare` - local cBioPortal study analytics

## Filter Highlights

- `search variant` filter highlights:
  - `--review-status <0-4|N_star|N_stars|none|expert_panel|criteria_provided>`
  - `--has` / `--missing`: `cadd|revel|gerp|clinvar|gnomad|dbsnp|snpeff|civic|cosmic`
  - `--population --revel-min --gerp-min --tumor-site --condition --impact --lof --therapy`
- Variant consequences:
  - `missense_variant|synonymous_variant|frameshift_variant|nonsense_variant|stop_gained`
  - `stop_lost|start_lost|splice_acceptor_variant|splice_donor_variant`
  - `inframe_insertion|inframe_deletion|intron_variant`
  - `upstream_gene_variant|downstream_gene_variant|non_coding_transcript_variant`
- `search adverse-event ... --source <faers, vaers, all> --date-from --date-to --suspect-only --sex --age-min --age-max --reporter --count`
- `search diagnostic ... --source <gtr|who-ivd|all>`
  - filters: `--gene`, `--disease`, `--type`, `--manufacturer`
  - `--disease` requires at least 3 alphanumeric characters
- `search gene ... --region --pathway --go` (use GO IDs like `GO:0004672`; search output includes Coordinates (GRCh38)/UniProt/OMIM)
- `search protein ... --reviewed|--include-unreviewed --all-species --disease --existence` (human and reviewed are independent defaults)
- `search trial ... --mutation --criteria --study-type --has-results --date-from --date-to`
  - Broad discovery of simple mutation text gets a registry eligibility check.
  - It removes exclusion-only matches; absent terms remain discoverable.
  - Use boolean expressions for discovery-only searches.
  - CTGov intervention workers are quoted literals.
  - Expansion uses plausible trade names and investigational codes while excluding systematic chemical synonyms.
  - A rejected expanded alias preserves successful requested-name results and makes the exact total unknown.
  - `--no-alias-expand` performs one literal request.
- `search author -q <name> --source semanticscholar --limit <N> --offset <N>`
- `get author semanticscholar:<id>`
- `search article ... -a <author> --date-from --date-to --year-min --year-max --journal`
  - JSON rows are compact by default; add `--full` for abstracts, full provenance, and ranking diagnostics
  - `--sort date` replaces relevance ranking and emits an in-band warning
  - add `--source <all, pubtator, europepmc, pubmed, semanticscholar, litsense2>`
  - add `--max-per-source <N>` or `--session <token>` when needed
- known gene/disease/drug anchors go in `-g/-d/--drug`; free-text concepts go in `-k`
- known authors go in `-a/--author`; default candidate search uses Europe PMC + compatible PubMed
- For article search, keep known gene/disease/drug anchors in `-g/-d/--drug`.
- Put mechanisms, phenotypes, outcomes, and datasets in provider-neutral `-k/--keyword`; use `--author` or `--journal` instead of provider field syntax.
- PubMed ESearch cleans question-format terms provider-locally.
- Direct and compatible federated PubMed ESearch cleans question-format terms
  provider-locally; non-PubMed sources keep the original wording.
- Keyword-only article result pages can suggest typed entity follow-ups when
  the whole keyword exactly matches a gene, drug, or disease label or alias.
- Multi-concept keyword phrases and searches that already use `-g`, `-d`, or `--drug` do not get direct entity suggestions
- Article result pages can also suggest year-refinement follow-ups when visible rows expose publication years and the current search has no explicit date bounds
- `--session <token>` is a local non-secret caller label for JSON loop-breakers.
- Same-session article searches can suggest prior `article batch`, `discover`,
  and date-narrowing follow-ups.
- `search drug ... --region <us|eu|ema|who|all>`
  - omitted `--region` checks U.S., EU, and WHO for plain name/alias lookups
  - omitted structured filters stay U.S.-only
  - `who` filters structured U.S. hits through WHO prequalification
  - `--product-type <finished_pharma|api|vaccine>` requires `--region who`
  - WHO vaccine search is plain name/brand only
  - `ema` is accepted as an alias for `eu`
  - vaccine name/brand searches may use the CDC CVX/MVX bridge

## Helpers

- `variant trials <id> --source <ctgov|nci> --limit <N> --offset <N>`
- `variant articles <id> [--strategy <union|annotation|lexical>]`
- `--json variant articles --input <path|-> [--debug-plan]`
- `variant erepo <CAid> [--detail --assertion <UUID> --version <exact-docVersion>]`
- `variant erepo --gene <symbol> [--limit <1-100>] [--offset <N>]`
  - structured input accepts 1-10 variants and returns ordered compact item envelopes
  - defaults to an exact-route union with provenance before one final pagination pass
  - unresolved union results are explicitly best-effort; diagnostic exact routes stay empty
  - debug plans expose normalized routes, providers, ranking inputs, and bounded work only when requested
- `drug trials <name>` - uses the same trial-safe CTGov literal and alias policy as `search trial`
- `drug interactions <name> [--limit <N>] [--offset <N>]` - bounded local DDInter-backed drug-drug interactions
  - defaults to 25 rows, caps pages at 50, and reports fresh/stale bundle state
- `drug adverse-events <name>` - FAERS-first adverse-event lookup
  - FAERS 404 falls back to ClinicalTrials.gov trial-reported adverse events
- `disease trials <name>`
- `disease articles <name>`
- `disease drugs <name>`
- `article entities <pmid> --limit <N>`
- `article batch <id> [<id>...]` - up to 20 compact cards in request order
  - bare-array JSON includes all source-supplied authors plus returned count
  - completeness is `complete`, `source_limited`, or `unavailable`
  - source is `pubtator` or `europepmc`
- Article detail carries the same authorship contract.
- `get article <id> indexing` adds PubMed citation authors with associated affiliations/ORCID and structured MeSH headings; `all` includes it.
- Indexing status distinguishes available-empty metadata from unavailable PubMed citation metadata.
- Europe PMC display-string authorship is source-limited.
- `article citations <id> --limit <N>` (optional auth; shared pool without `S2_API_KEY`)
- `article references <id> --limit <N>` (optional auth; shared pool without `S2_API_KEY`)
- `article recommendations <id> [<id>...] [--negative <id>...] --limit <N>` (optional auth; shared pool without `S2_API_KEY`)
- `gene trials|drugs|articles <symbol>`
- `gene pathways <symbol> --limit <N> --offset <N>`
- `pathway drugs|articles|trials <id>`
- `protein structures <accession> --limit <N> --offset <N>`
- `search drug --interactions <drug>` remains unavailable from current public data sources; use `drug interactions <name>` when you already know the anchor drug
- `study list`
- `study download [--list] [<study_id>]`
- `study top-mutated --study <id> [--limit <N>]`
- `study filter --study <id> [--mutated <symbol>] [--amplified <symbol>]`
  - add CNA, expression, and cancer-type filters as needed
- `study query --study <id> --gene <symbol> --type <mutations|cna|expression|sv>`
- `study cohort --study <id> --gene <symbol>`
- `study survival --study <id> --gene <symbol> [--endpoint <os|dfs|pfs|dss>]`
- `study compare --study <id> --gene <symbol> --type <expression|mutations> --target <symbol>`
- `study co-occurrence --study <id> --genes <g1,g2,...>`
- `search phenotype \"HP:... HP:...\"` or `search phenotype \"seizure, developmental delay\"`
- `search gwas -g <gene> | --trait <text>`

## Best-Effort Searches

Best-effort helpers search free-text fields (for example, eligibility criteria,
indication text, and abstracts) rather than strict structured identifiers.
Results depend on source document wording and may vary across sources.

## Deployment Notes

- Use `biomcp mcp-config --client <client>` to print copy-paste local stdio MCP config.
  Supported clients: codex, claude-desktop, claude-code, cursor, cline, vscode, json.
  Output uses `biomcp serve` by default; add `--absolute-path` when the client cannot see your shell `PATH`.
- Set `NCBI_API_KEY` to increase NCBI request throughput for article annotation, indexing, and full-text paths.
- Set `S2_API_KEY` for authenticated Semantic Scholar requests at 1 req/sec; without it, BioMCP uses the shared pool at 1 req/2sec.
- `search article --json` preserves article source status, warnings, pagination, retraction state, and next commands in compact and `--full` output.
- `--debug-plan` exposes executed routing plus redacted source auth/availability.
- On default `search article --source all`, `-a/--author` limits candidate search
  to Europe PMC + compatible PubMed; stricter filters may narrow further. Explicit
  PubTator3, Semantic Scholar, or LitSense2 author searches are rejected instead
  of becoming free text.
- Other compatible typed anchors remain federated and Semantic Scholar remains automatic.
- Add provider-neutral `-k/--keyword` for mechanisms, phenotypes, datasets, and
  free-text concepts; use `--author` or `--journal` instead of provider field syntax.
  Ordinary biomedical bracket and colon notation remains literal. The compatible
  default source set stays PubTator3 + Europe PMC + PubMed + Semantic Scholar.
- Use `--source semanticscholar` explicitly to query Semantic Scholar alone.
- Use `--source litsense2` explicitly to query LitSense2.
- Keyword-bearing article queries default to hybrid ranking.
- Cap each federated source's contribution after deduplication and before ranking.
- Default: 40% of `--limit` on federated pools with at least three surviving primary sources.
- `0` uses the default cap, and setting it equal to `--limit` disables capping.
- Rows count against their primary source after deduplication.
- `--ranking-mode semantic` sorts by the LitSense2-derived semantic signal and falls back to lexical ties.
- Hybrid ranking uses the same LitSense2-derived semantic signal, and rows without LitSense2 provenance contribute `semantic=0`.
- `search article --source semanticscholar` uses the same Semantic Scholar route as federation.
- It does not support `--type` or `--open-access`.
- `search article --source litsense2` requires `-k/--keyword` (or a positional query) and does not support `--type` or `--open-access`.
- `--type`, `--open-access`, and `--no-preprints` narrow compatible article
  sources instead of acting as universal backend filters.
- EU drug commands auto-download EMA human-medicines JSON feeds into the default
  data dir or `BIOMCP_EMA_DIR`, then refresh stale files after 72 hours.
- WHO regional commands auto-download finished-pharma, API, and vaccine CSV
  exports into the default data dir or `BIOMCP_WHO_DIR`.
- Vaccine brand lookups can auto-download the CDC CVX/MVX bundle into the
  default data dir or `BIOMCP_CVX_DIR`, then refresh after 30 days.
- Diagnostic commands auto-download the NCBI GTR bundle on first use into the default data dir or `BIOMCP_GTR_DIR`, then refresh stale files after 7 days.
- Diagnostic WHO IVD commands auto-download `who_ivd.csv` into the default data
  dir or `BIOMCP_WHO_IVD_DIR`, then refresh stale files after 72 hours.
- Run `ema sync`, `who sync`, `cvx sync`, `gtr sync`, or `who-ivd sync` to force-refresh the local runtime data.
- Use `biomcp health --apis-only` for upstream/API checks. Repeat `--api` for exact providers and add `--fail-on-error` for automation.
- Full `biomcp health` includes local EMA/WHO/CVX/GTR/cache readiness plus cache-limit warnings.
- In multi-worker environments, run one shared `biomcp serve-http` process so workers share one Streamable HTTP `/mcp` endpoint and one limiter budget.

## Ops

- `cache path` - print the managed HTTP cache directory `<resolved cache_root>/http`; output stays plain text and ignores `--json`
- `cache stats` - show HTTP cache inventory, age range, and resolved limits
- `cache clean [--max-age <duration>] [--max-size <size>] [--dry-run]`
  - remove orphan blobs and optionally age- or size-evict the HTTP cache
- `cache clear [--yes]` - wipe `<resolved cache_root>/http`; never `downloads/`
- `ema sync`
- `who sync`
- `cvx sync`
- `gtr sync`
- `who-ivd sync`
- `update [--check]` - atomically update a standalone-installer-owned Unix binary with release SHA256 verification
- `uninstall`
- `health [--apis-only] [--api <canonical-name>]... [--fail-on-error]`
- `version`

Run `biomcp list <entity>` for entity-specific examples.
