//! Literature and study command-reference pages for `biomcp list`.
pub(super) fn list_article() -> String {
    r#"# article

## When to use this surface

- Use keyword search to scan a topic before you know the entities.
- Add `-g/--gene` when you already know the molecular anchor.
- Prefer `--type review` for synthesis questions, broad mechanism questions, or list-style answers.
- Refine the query before paginating when the first page is noisy; paginate when the first page is relevant.
- Use `article citations <id>`, `article references <id>`, and `article recommendations <id>` to deepen one strong paper into a multi-hop evidence trail.

## Commands

- `get article <id>` - get by PMID/PMCID/DOI
- `get article <id> tldr` - Semantic Scholar TLDR/influence section (optional auth; shared pool without `S2_API_KEY`)
- `get article <id> annotations` - PubTator entity mentions
- `get article <id> fulltext` - download/cache full text via XML -> PMC HTML
- `get article <id> fulltext --pdf` - allow Semantic Scholar PDF after XML and PMC HTML miss
- `get article <id> all` - include all article sections
- `article entities <pmid> --limit <N>` - annotated entities with next commands
- `article batch <id> [<id>...]` - compact multi-article summary cards
- `article citations <id> --limit <N>` - citation graph with contexts/intents (optional auth; shared pool without `S2_API_KEY`)
- `article references <id> --limit <N>` - reference graph with contexts/intents (optional auth; shared pool without `S2_API_KEY`)
- `article recommendations <id> [<id>...] [--negative <id>...] --limit <N>` - related papers (optional auth; shared pool without `S2_API_KEY`)

## Search

- `search article -g <gene>` - gene filter (PubTator autocomplete)
- `search article -d <disease>` - disease filter (PubTator autocomplete)
- `search article --drug <name>` - chemical/drug filter (PubTator autocomplete)
- `search article <query>` - positional free text keyword
- `search article -k <keyword>` (or `-q <keyword>`) - free text keyword
- `search article --type <review|research|case-reports|meta-analysis>`
- `search article --date-from <YYYY|YYYY-MM|YYYY-MM-DD> --date-to <YYYY|YYYY-MM|YYYY-MM-DD>`
- `search article --since <YYYY|YYYY-MM|YYYY-MM-DD>` - alias for `--date-from`
- `search article --year-min <YYYY> --year-max <YYYY>` - exact year aliases for article date bounds
- `search article --journal <name>`
- `search article --open-access`
- `search article --exclude-retracted`
- `search article --include-retracted`
- `search article --sort <date|citations|relevance>`
- `search article --ranking-mode <lexical|semantic|hybrid>`
- `search article --weight-semantic <float>`
- `search article --weight-lexical <float>`
- `search article --weight-citations <float>`
- `search article --weight-position <float>`
- `search article --source <all, pubtator, europepmc, pubmed, litsense2>`
- `search article --max-per-source <N>`
- `search article --session <token>` - local caller label for JSON loop-breaker suggestions across consecutive article keyword searches
- `search article --debug-plan` - include executed planner/routing metadata in markdown or JSON
- `search article ... --limit <N> --offset <N>`

## Query formulation

| Question shape | How to map it |
|---|---|
| Known gene/disease/drug already identified | Put the anchor in `-g/--gene`, `-d/--disease`, or `--drug` |
| Known anchor plus mechanism, phenotype, process, or outcome | Keep the anchor typed and put the free-text concept in `-k/--keyword` |
| Keyword-only topic, dataset, or method question | Use `-k/--keyword`; add `--type review` for synthesis or survey questions |
| Unknown gene/disease/drug; identify it from symptoms, mechanisms, or evidence first | Do not invent `-g/-d/--drug`; stay keyword-first or start with `discover` |

Result-page follow-ups:

- Keyword-only result pages can suggest typed `get gene`, `get drug`, or `get disease` follow-ups when the whole `-k/--keyword` exactly matches a gene, drug, or disease vocabulary label or alias.
- Multi-concept keyword phrases and searches that already use `-g/--gene`, `-d/--disease`, or `--drug` do not get direct entity suggestions.
- Visible dated result pages with no existing date bounds can also suggest year-refinement next commands such as `biomcp search article ... --year-min <YYYY> --year-max <YYYY> --limit 5`.
- With `--json --session <token>`, consecutive keyword searches in the same local session can add loop-breaker `_meta.suggestions[]` after 60% post-stopword term overlap. Use a short non-identifying token such as `lit-review-1`; do not put credentials, PHI, or user identifiers in it.

Entity-only quick start:

- `biomcp search article -g BRAF --limit 5`

Routing note:

- On the default `search article --source all` route, typed gene/disease/drug anchors participate in PubTator3 + Europe PMC + PubMed when the filter set is compatible; Semantic Scholar is still automatic on compatible queries.
- Add `-k/--keyword` for mechanisms, phenotypes, datasets, and other free-text concepts; that also brings LitSense2 into compatible federated searches and makes the default relevance mode hybrid instead of lexical.
- Direct and compatible federated PubMed ESearch cleans question-format gene/disease/drug/keyword terms provider-locally; query echoes and non-PubMed sources keep the original wording.
- Cap each federated source's contribution after deduplication and before ranking.
- Default: 40% of `--limit` on federated pools with at least three surviving primary sources.
- `0` uses the default cap; setting it equal to `--limit` disables capping.
- Rows count against their primary source after deduplication.
- `--type`, `--open-access`, and `--no-preprints` can narrow the compatible default source set instead of acting as universal article filters across every backend.

Worked examples:

- Disease-identification question: `biomcp search article -k '"cafe-au-lait spots" neurofibromas disease' --type review --limit 5` keeps the search keyword-first because the disease is the unknown answer.
- Known-gene question: `biomcp search article -g TP53 -k "apoptosis gene regulation" --limit 5` keeps TP53 typed and the regulatory process in free text.
- Known-drug question: `biomcp search article --drug amiodarone -k "photosensitivity mechanism" --limit 5` keeps the drug typed and the adverse-effect mechanism in free text.
- Method/dataset question: `biomcp search article -k "TCGA mutation analysis dataset" --type review --limit 5` stays keyword-only because the question is about a dataset, not a typed biomedical entity.
- Historical literature slice: `biomcp search article -k "BRAF melanoma" --year-min 2000 --year-max 2013 --limit 5` narrows the shared article date filter to a publication-year window.

## JSON Output

- Non-empty `search article --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get article <pmid>`.
- `biomcp list article` is always included so agents can inspect the full filter surface.
- Keyword-only exact entity matches can also add `biomcp get gene <symbol>`, `biomcp get drug <name>`, or `biomcp get disease <name>` to `_meta.next_commands`.
- Article search `_meta.suggestions` is an optional array of objects with `command` and `reason`. Exact entity suggestions include `sections`; loop-breaker suggestions from `--session` omit `sections`.
- Multi-concept keyword phrases and typed-filter searches omit direct entity suggestion objects.
- Loop-breaker suggestions, when emitted, are ordered as prior `biomcp article batch ...`, `biomcp discover <topic>`, then a date-narrowed `biomcp search article ... --year-min ... --year-max ...` retry when available.
- When no explicit article date bounds are present, visible dated rows can also add a year-refinement next command that rebuilds the current search with `--year-min <YYYY> --year-max <YYYY> --limit 5`.
- Each result may include `first_index_date` as `YYYY-MM-DD` when the upstream record exposes when it was first indexed. Europe PMC and PubMed provide it today; PubTator3, LitSense2, and Semantic Scholar do not.

## Notes

- Set `NCBI_API_KEY` to increase throughput for NCBI-backed article enrichment.
- Set `S2_API_KEY` to send authenticated Semantic Scholar requests at 1 req/sec. Without it, BioMCP uses the shared pool at 1 req/2sec.
- `get article <id> fulltext` tries XML first, then PMC HTML, and never falls back to PDF.
- Add `--pdf` only with `fulltext` to extend that ladder with Semantic Scholar PDF as the last resort.
- `--pdf` requires the `fulltext` section and is rejected for other article requests.
- On the default `search article --source all` route, typed gene/disease/drug anchors participate in PubTator3 + Europe PMC + PubMed when the filter set is compatible; Semantic Scholar is still automatic on compatible queries.
- Add `-k/--keyword` for mechanisms, phenotypes, datasets, and other free-text concepts; that also brings LitSense2 into compatible federated searches and makes the default relevance mode hybrid instead of lexical.
- Direct and compatible federated PubMed ESearch cleans question-format gene/disease/drug/keyword terms provider-locally; query echoes and non-PubMed sources keep the original wording.
- `search article --source litsense2` requires `-k/--keyword` (or a positional query) and does not support `--type` or `--open-access`.
- `--type`, `--open-access`, and `--no-preprints` can narrow the compatible default source set instead of acting as universal article filters across every backend.
- `search article --type ...` on `--source all` uses Europe PMC + PubMed when PubMed-compatible filters are selected, and collapses to Europe PMC-only when `--open-access` or `--no-preprints` makes PubMed ineligible.
- `search article --sort relevance` accepts `--ranking-mode lexical|semantic|hybrid`.
- When `--ranking-mode` is omitted, keyword-bearing article queries default to hybrid ranking and entity-only queries default to lexical ranking.
- `--ranking-mode semantic` sorts by the LitSense2-derived semantic signal and falls back to lexical ties.
- Default hybrid scoring is `0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position`; `--weight-*` flags retune those components.
- Hybrid ranking uses the same LitSense2-derived semantic signal, and rows without LitSense2 provenance contribute `semantic=0`.
- Weight flags are part of the hybrid contract and pair with `--sort relevance`.
- Markdown search output adds `Newest indexed: YYYY-MM-DD (N days ago)` immediately after the result table when any returned row has `first_index_date`.
"#
    .to_string()
}

pub(super) fn list_study() -> String {
    r#"# study

## Commands

- `study list` - list locally available cBioPortal studies from `BIOMCP_STUDY_DIR`
- `study download [--list] [<study_id>]` - list downloadable study IDs or install a study into `BIOMCP_STUDY_DIR`
- `study top-mutated --study <id> [--limit <N>]` - rank the most frequently mutated genes in a study
- `study filter --study <id> [--mutated <symbol>] [--amplified <symbol>] [--deleted <symbol>] [--expression-above <gene:threshold>] [--expression-below <gene:threshold>] [--cancer-type <type>]` - intersect sample filters across mutation, CNA, expression, and clinical data
- `study query --study <id> --gene <symbol> --type <mutations|cna|expression>` - run per-study gene query
- `study cohort --study <id> --gene <symbol>` - split the cohort into `<gene>-mutant` vs `<gene>-wildtype`
- `study survival --study <id> --gene <symbol> [--endpoint <os|dfs|pfs|dss>]` - summarize KM survival and log-rank statistics by mutation group
- `study compare --study <id> --gene <symbol> --type <expression|mutations> --target <symbol>` - compare expression or mutation rate across mutation groups
- `study co-occurrence --study <id> --genes <g1,g2,...>` - pairwise mutation co-occurrence (2-10 genes)

## Setup

- `BIOMCP_STUDY_DIR` should point to a directory containing per-study folders (for example `msk_impact_2017/`).
- Use `study download --list` to browse remote IDs and `study download <study_id>` to install a study into that directory.
- `study cohort`, `study survival`, and `study compare` require `data_mutations.txt` and `data_clinical_sample.txt`.
- `study survival` also requires `data_clinical_patient.txt` with canonical `{ENDPOINT}_STATUS` and `{ENDPOINT}_MONTHS` columns.
- Expression comparison also requires a supported expression matrix file.

## Examples

- `study list`
- `study download --list`
- `study download msk_impact_2017`
- `study top-mutated --study msk_impact_2017 --limit 10`
- `study filter --study brca_tcga_pan_can_atlas_2018 --mutated TP53 --amplified ERBB2 --expression-above ERBB2:1.5`
- `study query --study msk_impact_2017 --gene TP53 --type mutations`
- `study query --study brca_tcga_pan_can_atlas_2018 --gene ERBB2 --type cna`
- `study query --study paad_qcmg_uq_2016 --gene KRAS --type expression`
- `study cohort --study brca_tcga_pan_can_atlas_2018 --gene TP53`
- `study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53 --endpoint os`
- `study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type expression --target ERBB2`
- `study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type mutations --target PIK3CA`
- `study co-occurrence --study msk_impact_2017 --genes TP53,KRAS`
"#
    .to_string()
}
