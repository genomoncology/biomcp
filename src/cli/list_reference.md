# BioMCP Command Reference

BioMCP connects to PubMed, ClinicalTrials.gov, ClinVar, gnomAD, OncoKB, Reactome,
KEGG, UniProt, PharmGKB, CPIC, OpenFDA, Monarch Initiative, GWAS Catalog, and more.
One command grammar covers all entities.

## Quickstart

New to BioMCP? Try:

- `skill install` - install BioMCP skill guidance to your agent
- `get gene BRAF` - look up a gene
- `get variant "BRAF V600E"` - annotate a variant
- `discover "chest pain"` - resolve free text before choosing an entity
- `search trial -c melanoma` - find clinical trials
- `search all --gene BRAF --disease melanoma` - cross-entity summary card

## Entities

- gene
- variant
- article
- trial
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
- `discover <query>` - resolve free-text concepts into typed follow-up commands
- `search all [slot filters]` - curated multi-entity orientation (`--gene/--variant/--disease/--drug/--keyword`)
- `search trial [filters]` - trial search is filter-only
- `get <entity> <id> [section...]` - fetch by identifier with optional sections
- `get trial <nct_id> locations --offset <N> --limit <N>` - page trial locations
- `enrich <GENE1,GENE2,...>` - gene-set enrichment via g:Profiler
- `batch <entity> <id1,id2,...>` - parallel get operations
- `study list|download|query|co-occurrence|cohort|survival|compare` - local cBioPortal study analytics

## Filter Highlights

- `search variant ... --review-status --population --revel-min --gerp-min --tumor-site --condition --impact --lof --has --missing --therapy`
- `search adverse-event ... --date-from --date-to --suspect-only --sex --age-min --age-max --reporter --count`
- `search gene ... --region --pathway --go` (use GO IDs like `GO:0004672`; search output includes Coordinates/UniProt/OMIM)
- `search protein ... --reviewed --disease --existence` (default reviewed mode)
- `search trial ... --mutation --criteria --study-type --has-results --date-from --date-to`
- `search article ... --date-from --date-to --journal --source <all|pubtator|europepmc>`

## Helpers

- `variant trials <id> --source <ctgov|nci> --limit <N> --offset <N>`
- `variant articles <id>`
- `drug trials <name>`
- `drug adverse-events <name>`
- `disease trials <name>`
- `disease articles <name>`
- `disease drugs <name>`
- `article entities <pmid> --limit <N>`
- `article citations <id> --limit <N>` (`S2_API_KEY`)
- `article references <id> --limit <N>` (`S2_API_KEY`)
- `article recommendations <id> [<id>...] [--negative <id>...] --limit <N>` (`S2_API_KEY`)
- `gene trials|drugs|articles <symbol>`
- `gene pathways <symbol> --limit <N> --offset <N>`
- `pathway drugs|articles|trials <id>`
- `protein structures <accession> --limit <N> --offset <N>`
- `study list`
- `study download [--list] [<study_id>]`
- `study filter --study <id> [--mutated <symbol>] [--amplified <symbol>] [--deleted <symbol>] [--expression-above <gene:threshold>] [--expression-below <gene:threshold>] [--cancer-type <type>]`
- `study query --study <id> --gene <symbol> --type <mutations|cna|expression>`
- `study cohort --study <id> --gene <symbol>`
- `study survival --study <id> --gene <symbol> [--endpoint <os|dfs|pfs|dss>]`
- `study compare --study <id> --gene <symbol> --type <expression|mutations> --target <symbol>`
- `study co-occurrence --study <id> --genes <g1,g2,...>`
- `search phenotype \"HP:... HP:...\"`
- `search gwas -g <gene> | --trait <text>`

## Best-Effort Searches

Best-effort helpers search free-text fields (for example, eligibility criteria,
indication text, and abstracts) rather than strict structured identifiers.
Results depend on source document wording and may vary across sources.

## Deployment Notes

- Set `NCBI_API_KEY` to increase NCBI request throughput for article annotation/full-text paths.
- Set `S2_API_KEY` to unlock Semantic Scholar TLDR, citation graph, and recommendation paths.
- In multi-worker environments, run one shared `biomcp serve-http` process so workers share one Streamable HTTP `/mcp` endpoint and one limiter budget.

## Ops

- `update [--check]`
- `uninstall`
- `health [--apis-only]`
- `version`

Run `biomcp list <entity>` for entity-specific examples.
