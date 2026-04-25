# BioMCP Command Reference

BioMCP connects to PubMed, ClinicalTrials.gov, ClinVar, gnomAD, OncoKB, Reactome,
KEGG, UniProt, PharmGKB, CPIC, OpenFDA, Monarch Initiative, GWAS Catalog, and more.
One command grammar covers all entities.

## Quickstart

New to BioMCP? Try:

- `skill install` - install BioMCP skill guidance to your agent
- `suggest "What drugs treat melanoma?"` - choose the worked-example playbook for a question
- `get gene BRAF` - look up a gene
- `get gene BRCA1 diagnostics` - inspect GTR diagnostic tests for a known gene
- `get disease tuberculosis diagnostics` - inspect up to 10 local diagnostic tests for a condition
- `search diagnostic --gene BRCA1 --limit 5` - find genetic tests for a known gene
- `search diagnostic --disease HIV --source who-ivd --limit 5` - find WHO infectious-disease diagnostics
- `get variant "BRAF V600E"` - annotate a variant
- `discover "chest pain"` - resolve a single-entity free-text phrase before choosing a typed command
- `search trial -c melanoma` - find clinical trials
- `search all --gene BRAF --disease melanoma` - cross-entity summary card

## When to Use What

| You want to know... | Start with |
|---|---|
| I have a biomedical question and need the right playbook | `suggest "<question>"` |
| How much NIH funded a disease or gene | `get disease <name_or_id> funding` or `get gene <symbol> funding` |
| What drugs treat a disease | `search drug --indication "<disease>" --limit 5` |
| What diagnostic test exists for a gene or disease | `get gene <symbol> diagnostics`, `get disease <name_or_id> diagnostics`, or `search diagnostic --gene <symbol> --limit 5` |
| What the 5-year survival outlook is for a cancer | `get disease <name_or_id> survival` |
| Symptoms or phenotypes of a disease | `get disease <name_or_id> phenotypes` |
| MedlinePlus clinical-summary rows for a configured disease | `get disease <name_or_id> clinical_features` |
| Which diseases match HPO IDs or symptom text | `search phenotype "<HP:... HP:...>"` or `search phenotype "seizure, developmental delay"` |
| What a gene does | `get gene <symbol>` |
| Tissue expression or localization of a gene product | `get gene <symbol> hpa` or `get gene <symbol> protein` |
| Drug safety or adverse events | `drug adverse-events <name>` or `get drug <name> safety` |
| Which drugs or drug classes interact with a known drug | `drug interactions <name>` or `get drug <name> interactions` |
| Review literature that synthesizes a topic | `search article -k "<query>" --type review --limit 5` |
| Turn a literature question into article filters | `biomcp list article` (known gene/disease/drug anchors go in `-g/-d/--drug`; free-text concepts go in `-k`; PubMed ESearch cleans question-format terms provider-locally; exact whole-keyword entity labels or aliases can trigger typed follow-up suggestions on keyword-only result pages) |
| Follow one article into related evidence | `article citations <id> --limit 5` or `article recommendations <id> --limit 5` |
| I know the entities but not the next pivot | `search all --gene BRAF --disease melanoma` |
| I only have a single-entity biomedical phrase and need routing | `discover "<free text>"` (supported routed exceptions still cover symptom-of-disease, HPO symptom, treatment, gene+disease, and unambiguous gene-plus-topic flows; relational or multi-entity questions may redirect to `search all --keyword "<query>"`) |
| The same sections for several entities | `batch <entity> <id1,id2,...> --sections <s1,s2,...>` |
| Enriched pathways or functions for a gene set | `enrich <GENE1,GENE2,...>` |

## Entities

- gene
- variant
- article
- trial
- diagnostic
- drug
- disease
- phenotype
- pgx
- gwas
- pathway
- protein
- study
- adverse-event

## Patterns

- `search <entity> [query|filters]` - find entities
- `suggest <question>` - route a biomedical question to one worked-example playbook and two starter commands
- `discover <query>` - resolve a single-entity free-text phrase into typed follow-up commands; relational questions may redirect to `search all --keyword`
- `search all [slot filters]` - curated multi-entity orientation (`--gene/--variant/--disease/--drug/--keyword`)
- `search trial [filters]` - trial search is filter-only
- `get <entity> <id> [section...]` - fetch by identifier with optional sections
- `get gene <symbol> diagnostics` - opt-in diagnostic-test pivot from a gene card
- `get disease <name_or_id> diagnostics` - opt-in diagnostic-test pivot from a disease card, capped at 10 rows with a `search diagnostic` follow-up for broader pages
- `get disease <name_or_id> clinical_features` - opt-in MedlinePlus clinical-summary rows for configured diseases; unsupported diseases omit fabricated rows and `all` still excludes this section
- `get drug <name> regulatory [--region <us|eu|who|all>]` - region-aware U.S./EU/WHO regulatory context
- `get drug <name> safety|shortage [--region <us|eu|all>]` - region-aware U.S./EU drug safety and shortage context
- `get drug <name> all [--region <us|eu|who|all>]` - include all sections plus region-aware regulatory context
- `ema` is accepted as an input alias for the canonical `eu` drug region value
- Omitting `--region` on `get drug <name> regulatory` is the one implicit combined-region get path; other no-flag `get drug` shapes stay on the default U.S. path
- `get trial <nct_id> locations --offset <N> --limit <N>` - page trial locations
- `enrich <GENE1,GENE2,...>` - gene-set enrichment via g:Profiler
- `batch <entity> <id1,id2,...>` - parallel get operations
- `study list|download|top-mutated|filter|query|co-occurrence|cohort|survival|compare` - local cBioPortal study analytics

## Filter Highlights

- `search variant ... --review-status --population --revel-min --gerp-min --tumor-site --condition --impact --lof --has --missing --therapy`
- `search adverse-event ... --source <faers, vaers, all> --date-from --date-to --suspect-only --sex --age-min --age-max --reporter --count`
- `search diagnostic ... --source <gtr|who-ivd|all> --gene --disease --type --manufacturer` (`--disease` requires at least 3 alphanumeric characters and matches disease words/phrases at boundaries; use `--limit`/`--offset` for broader pages)
- `search gene ... --region --pathway --go` (use GO IDs like `GO:0004672`; search output includes Coordinates/UniProt/OMIM)
- `search protein ... --reviewed --disease --existence` (default reviewed mode)
- `search trial ... --mutation --criteria --study-type --has-results --date-from --date-to`
- `search article ... --date-from --date-to --year-min --year-max --journal --source <all, pubtator, europepmc, pubmed, litsense2> --max-per-source <N> --session <token>`
- For article search, keep known gene/disease/drug anchors in `-g/-d/--drug` and put mechanisms, phenotypes, outcomes, and datasets in `-k/--keyword`; run `biomcp list article` for worked decomposition examples
- Direct and compatible federated PubMed ESearch cleans question-format gene/disease/drug/keyword terms provider-locally; query echoes and non-PubMed sources keep the original wording
- Keyword-only article result pages can suggest typed `get gene`, `get drug`, or `get disease` follow-ups when the whole keyword exactly matches a gene, drug, or disease vocabulary label or alias
- Multi-concept keyword phrases and searches that already use `-g`, `-d`, or `--drug` do not get direct entity suggestions
- Article result pages can also suggest year-refinement follow-ups when visible rows expose publication years and the current search has no explicit date bounds
- `--session <token>` is a local non-secret caller label for JSON loop-breaker suggestions across consecutive article keyword searches; overlapping same-session searches can suggest prior `article batch`, `discover`, and date narrowing
- `search drug ... --region <us|eu|who|all>` (omitting `--region` checks U.S., EU, and WHO for plain name/alias lookups; omitted structured filters stay U.S.-only; explicit `who` filters structured U.S. hits through WHO prequalification for finished-pharma/API searches; `--product-type <finished_pharma|api|vaccine>` is WHO-only and requires explicit `--region who`; WHO vaccine search is plain name/brand only and rejects structured filters; default WHO search excludes vaccines unless `--product-type vaccine` is explicit; explicit `eu|all` with structured filters errors; `ema` is accepted as an alias for `eu`; omitted `--region` on plain-name vaccine lookups, explicit `eu|all` vaccine lookups, and explicit WHO vaccine name/brand searches can also use the CDC CVX/MVX bridge after MyChem identity misses, while pure `--region us` search does not use the CVX root)

## Helpers

- `variant trials <id> --source <ctgov|nci> --limit <N> --offset <N>`
- `variant articles <id>`
- `drug trials <name>`
- `drug interactions <name>` - DDInter-backed structured drug-drug interactions; `--json` adds class summaries and exact helper follow-ups for `biomcp get drug <canonical> safety` plus `biomcp search article --drug <canonical> --limit 5`
- `drug adverse-events <name>` - FAERS-first; FAERS 404 falls back to ClinicalTrials.gov trial-reported adverse events and `--json` adds `faers_not_found` plus optional `trial_adverse_events`, while FAERS 200+empty stays on FAERS
- `disease trials <name>`
- `disease articles <name>`
- `disease drugs <name>`
- `article entities <pmid> --limit <N>`
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
- `study filter --study <id> [--mutated <symbol>] [--amplified <symbol>] [--deleted <symbol>] [--expression-above <gene:threshold>] [--expression-below <gene:threshold>] [--cancer-type <type>]`
- `study query --study <id> --gene <symbol> --type <mutations|cna|expression>`
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

- Set `NCBI_API_KEY` to increase NCBI request throughput for article annotation/full-text paths.
- Set `S2_API_KEY` for authenticated Semantic Scholar requests at 1 req/sec; without it, BioMCP uses the shared pool at 1 req/2sec.
- On the default `search article --source all` route, typed gene/disease/drug anchors participate in PubTator3 + Europe PMC + PubMed when the filter set is compatible, and Semantic Scholar is still automatic on compatible queries.
- Add `-k/--keyword` for mechanisms, phenotypes, datasets, and other free-text concepts; that also brings LitSense2 into compatible federated searches and makes the default relevance mode hybrid instead of lexical.
- Cap each federated source's contribution after deduplication and before ranking.
- Default: 40% of `--limit` on federated pools with at least three surviving primary sources.
- `0` uses the default cap, and setting it equal to `--limit` disables capping.
- Rows count against their primary source after deduplication.
- `--ranking-mode semantic` sorts by the LitSense2-derived semantic signal and falls back to lexical ties.
- Hybrid ranking uses the same LitSense2-derived semantic signal, and rows without LitSense2 provenance contribute `semantic=0`.
- `search article --source litsense2` requires `-k/--keyword` (or a positional query) and does not support `--type` or `--open-access`.
- `--type`, `--open-access`, and `--no-preprints` can narrow the compatible default source set instead of acting as universal article filters across every backend.
- EU drug commands auto-download the EMA human-medicines JSON feeds on first use into the default data dir or `BIOMCP_EMA_DIR`, then refresh stale files after 72 hours.
- WHO regional commands auto-download the WHO finished-pharma, API, and vaccine CSV exports on first use into the default data dir or `BIOMCP_WHO_DIR`, then refresh stale files after 72 hours.
- Default/EU vaccine brand lookups and explicit WHO vaccine name/brand searches can auto-download the CDC CVX/MVX bundle on first use into the default data dir or `BIOMCP_CVX_DIR`, then refresh stale files after 30 days.
- Diagnostic commands auto-download the NCBI GTR bundle on first use into the default data dir or `BIOMCP_GTR_DIR`, then refresh stale files after 7 days.
- Diagnostic WHO IVD commands auto-download `who_ivd.csv` on first use into the default data dir or `BIOMCP_WHO_IVD_DIR`, then refresh stale files after 72 hours.
- Run `ema sync`, `who sync`, `cvx sync`, `gtr sync`, or `who-ivd sync` to force-refresh the local runtime data.
- Use `biomcp health --apis-only` for upstream/API checks and full `biomcp health` for local EMA/WHO/CVX/GTR/cache readiness plus cache-limit warnings.
- In multi-worker environments, run one shared `biomcp serve-http` process so workers share one Streamable HTTP `/mcp` endpoint and one limiter budget.

## Ops

- `cache path` - print the managed HTTP cache directory `<resolved cache_root>/http`; output stays plain text and ignores `--json`
- `cache stats` - show HTTP cache statistics (total blob inventory, referenced blob bytes, age range, resolved limits including min disk free); supports `--json` for machine-readable output
- `cache clean [--max-age <duration>] [--max-size <size>] [--dry-run]` - remove orphan blobs and optionally age- or size-evict the HTTP cache; supports `--json` for machine-readable output
- `cache clear [--yes]` - destructively wipe `<resolved cache_root>/http`; never touches `downloads/`; supports `--json` on success and requires a TTY unless `--yes` is passed
- `ema sync`
- `who sync`
- `cvx sync`
- `gtr sync`
- `who-ivd sync`
- `update [--check]`
- `uninstall`
- `health [--apis-only]`
- `version`

Run `biomcp list <entity>` for entity-specific examples.
