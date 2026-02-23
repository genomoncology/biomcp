# BioMCP Command Reference

BioMCP connects to PubMed, ClinicalTrials.gov, ClinVar, gnomAD, OncoKB, Reactome,
UniProt, PharmGKB, CPIC, OpenFDA, Monarch Initiative, GWAS Catalog, and more.
One command grammar covers all entities.

## Entities

- gene
- variant
- article
- trial
- organization
- intervention
- biomarker
- drug
- disease
- pgx
- gwas
- pathway
- protein
- adverse-event

## Patterns

- `search <entity> [query|filters]` - find entities
- `search trial [filters]` - trial search is filter-only
- `get <entity> <id> [section...]` - fetch by identifier with optional sections
- `enrich <GENE1,GENE2,...>` - gene-set enrichment via g:Profiler
- `batch <entity> <id1,id2,...>` - parallel get operations

## Filter Highlights

- `search variant ... --review-status --population --revel-min --gerp-min --tumor-site --condition --impact --lof --has --missing --therapy`
- `search adverse-event ... --date-from --date-to --suspect-only --sex --age-min --age-max --reporter --count`
- `search gene ... --region --pathway --go` (search output includes Coordinates/UniProt/OMIM)
- `search protein ... --reviewed --disease --existence` (default reviewed mode)
- `search trial ... --study-type --has-results --date-from --date-to`
- `search article ... --date-from --date-to --journal`

## Helpers

- `variant trials <id> --source <ctgov|nci> --limit <N> --offset <N>`
- `variant articles <id>`
- `drug trials <name>`
- `drug adverse-events <name>`
- `disease trials <name>`
- `disease articles <name>`
- `disease drugs <name>`
- `article entities <pmid> --limit <N>`
- `gene trials|drugs|articles <symbol>`
- `gene pathways <symbol> --limit <N> --offset <N>`
- `pathway drugs|articles|trials <id>`
- `protein structures <accession> --limit <N> --offset <N>`
- `search phenotype \"HP:... HP:...\"`
- `search gwas -g <gene> | --trait <text>`

## Best-Effort Searches

Best-effort helpers search free-text fields (for example, eligibility criteria,
indication text, and abstracts) rather than strict structured identifiers.
Results depend on source document wording and may vary across sources.

## Deployment Notes

- Set `NCBI_API_KEY` to increase NCBI request throughput for article annotation/full-text paths.
- In multi-worker environments, run one shared `biomcp serve-http` process so workers use a single BioMCP SSE server and one limiter budget.

## Ops

- `update [--check]`
- `uninstall`
- `health [--apis-only]`
- `version`

Run `biomcp list <entity>` for entity-specific examples.
