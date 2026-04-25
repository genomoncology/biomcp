use crate::error::BioMcpError;

const LIST_REFERENCE: &str = include_str!("list_reference.md");

pub fn render(entity: Option<&str>) -> Result<String, BioMcpError> {
    match entity.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(list_all()),
        Some(raw) => match raw.to_ascii_lowercase().as_str() {
            "gene" => Ok(list_gene()),
            "variant" => Ok(list_variant()),
            "article" => Ok(list_article()),
            "trial" => Ok(list_trial()),
            "diagnostic" => Ok(list_diagnostic()),
            "drug" => Ok(list_drug()),
            "disease" => Ok(list_disease()),
            "phenotype" => Ok(list_phenotype()),
            "pgx" => Ok(list_pgx()),
            "gwas" => Ok(list_gwas()),
            "pathway" => Ok(list_pathway()),
            "protein" => Ok(list_protein()),
            "study" => Ok(list_study()),
            "adverse-event" | "adverse_event" | "adverseevent" => Ok(list_adverse_event()),
            "search-all" | "search_all" | "searchall" => Ok(list_search_all()),
            "suggest" => Ok(list_suggest()),
            "discover" => Ok(list_discover()),
            "batch" => Ok(list_batch()),
            "enrich" => Ok(list_enrich()),
            "skill" | "skills" => Ok(crate::cli::skill::list_use_cases()?),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown entity: {other}\n\nValid entities:\n- gene\n- variant\n- article\n- trial\n- diagnostic\n- drug\n- disease\n- phenotype\n- pgx\n- gwas\n- pathway\n- protein\n- study\n- adverse-event\n- search-all\n- suggest\n- discover\n- batch\n- enrich\n- skill"
            ))),
        },
    }
}

fn list_all() -> String {
    let has_oncokb = std::env::var("ONCOKB_TOKEN")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    let mut out = LIST_REFERENCE.to_string();

    if has_oncokb {
        out = out.replace(
            "- `variant articles <id>`\n",
            "- `variant articles <id>`\n- `variant oncokb <id>`\n",
        );
    }
    out
}

fn list_discover() -> String {
    r#"# discover

## Commands

- `discover <query>` - resolve a free-text biomedical phrase into a primary concept and suggested BioMCP follow-up commands
- `--json discover <query>` - emit structured concepts plus discover-specific `_meta` metadata for agents

## When to use this surface

- Use `discover` when you only have free text and need BioMCP to resolve the first entity or alias before choosing the next typed command.
- Discover is primarily a single-entity resolver for aliases, brands, symptoms, and close concept names.
- Prefer the first suggested command when the query clearly implies treatment, symptoms, safety, trials, or gene+disease orientation.
- Existing routed exceptions remain supported for symptom-of-disease prompts, HPO symptom bridging, treatment prompts, gene+disease orientation, and unambiguous gene-plus-topic follow-ups.
- Relational or multi-entity questions may redirect to `biomcp search all --keyword "<query>"`.
- Unambiguous gene-plus-topic queries can also surface `biomcp search article -g <symbol> -k <topic> --limit 5` when the remaining topic is meaningful.
- If no biomedical entities resolve, discover suggests `biomcp search article -k <query> --type review --limit 5`.
- If only low-confidence concepts resolve, discover adds a broader-results article-search hint.
"#
    .to_string()
}

fn list_suggest() -> String {
    r#"# suggest

## Commands

- `suggest <question>` - route a biomedical question to one shipped BioMCP worked-example playbook
- `--json suggest <question>` - emit the four-field response for agents

## Output fields

- `matched_skill` - playbook slug, or no match
- `summary` - short routing explanation
- `first_commands` - exactly two starter commands on match; none on no-match
- `full_skill` - `biomcp skill <slug>` for the full playbook, or none

## When to use this surface

- Use `suggest` when you know the biomedical question but not the first command sequence.
- Matched responses are offline and deterministic; they do not call upstream APIs.
- No-match stays successful and reports `No confident BioMCP skill match`.
- Use `discover "<question>"` when you need entity resolution rather than playbook selection.
"#
    .to_string()
}

fn list_gene() -> String {
    r#"# gene

## When to use this surface

- Use `get gene <symbol>` for the default card when you need the canonical summary first.
- Add `protein`, `hpa`, `expression`, `diseases`, `diagnostics`, or `funding` when you need deeper function, localization, disease, diagnostic-test, or NIH grant context.
- Use `gene articles <symbol>` or `search article -g <symbol>` when you need literature tied to one gene.

## Commands

- `get gene <symbol>` - basic gene info (MyGene.info)
- `get gene <symbol> pathways` - pathway section
- `get gene <symbol> ontology` - ontology enrichment section
- `get gene <symbol> diseases` - disease enrichment section
- `get gene <symbol> diagnostics` - diagnostic tests for this gene from GTR
- `get gene <symbol> protein` - UniProt protein summary
- `get gene <symbol> go` - QuickGO terms
- `get gene <symbol> interactions` - STRING interactions
- `get gene <symbol> civic` - CIViC evidence/assertion summary
- `get gene <symbol> expression` - GTEx tissue expression summary
- `get gene <symbol> hpa` - Human Protein Atlas protein tissue expression + localization
- `get gene <symbol> druggability` - DGIdb interactions plus OpenTargets tractability/safety
- `get gene <symbol> clingen` - ClinGen validity + dosage sensitivity
- `get gene <symbol> constraint` - gnomAD gene constraint (pLI, LOEUF, mis_z, syn_z)
- `get gene <symbol> disgenet` - DisGeNET scored gene-disease associations (requires `DISGENET_API_KEY`)
- `get gene <symbol> funding` - NIH Reporter grants mentioning the gene in the most recent 5 NIH fiscal years
- `get gene <symbol> all` - include every standard section (`diagnostics` and `funding` stay opt-in)
- `gene definition <symbol>` - same card as `get gene <symbol>`
- `gene get <symbol>` - alias for `gene definition <symbol>`

## Search filters

- `search gene <query>`
- `search gene -q <query>`
- `search gene -q <query> --type <protein-coding|ncRNA|pseudo>`
- `search gene -q <query> --chromosome <N>`
- `search gene -q <query> --region <chr:start-end>`
- `search gene -q <query> --pathway <id>`
- `search gene -q <query> --go <GO:0000000>`
- `search gene -q <query> --limit <N> --offset <N>`

## Search output

- Includes Coordinates, UniProt, and OMIM in default result rows.

## JSON Output

- Non-empty `search gene --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get gene <symbol>`.
- `biomcp list gene` is always included so agents can inspect the full filter surface.

## Helpers

- `gene trials <symbol>`
- `gene drugs <symbol>`
- `gene articles <symbol>`
- `gene pathways <symbol> --limit <N> --offset <N>`
"#
    .to_string()
}

fn list_variant() -> String {
    let has_oncokb = std::env::var("ONCOKB_TOKEN")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    let mut out = r#"# variant

## Commands

- `get variant <id>` - core annotation (MyVariant.info)
- `get variant <id> predict` - AlphaGenome prediction (requires `ALPHAGENOME_API_KEY`)
- `get variant <id> predictions` - expanded dbNSFP model scores (REVEL, AlphaMissense, etc.)
- `get variant <id> clinvar` - ClinVar section details
- `get variant <id> population` - gnomAD population frequencies
- `get variant <id> conservation` - phyloP/phastCons/GERP conservation scores
- `get variant <id> cosmic` - COSMIC context from cached MyVariant payload
- `get variant <id> cgi` - CGI drug-association evidence table
- `get variant <id> civic` - CIViC cached + GraphQL clinical evidence
- `get variant <id> cbioportal` - cBioPortal frequency enrichment (on-demand)
- `get variant <id> gwas` - GWAS trait associations
- `get variant <id> all` - include all sections

## Search filters

- `-g <gene>`
- `--hgvsp <protein_change>`
- `--significance <value>`
- `--max-frequency <0-1>`
- `--min-cadd <score>`
- `--consequence <term>`
- `--review-status <stars>`
- `--population <afr|amr|eas|fin|nfe|sas>`
- `--revel-min <score>`
- `--gerp-min <score>`
- `--tumor-site <site>`
- `--condition <name>`
- `--impact <HIGH|MODERATE|LOW|MODIFIER>`
- `--lof`
- `--has <field>`
- `--missing <field>`
- `--therapy <name>`

## Search output

- Includes ClinVar Stars, REVEL, and GERP in default result rows.

## JSON Output

- Non-empty `search variant --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get variant <id>`.
- `biomcp list variant` is always included so agents can inspect the full filter surface.

## IDs

Supported formats:
- rsID: `rs113488022`
- HGVS genomic: `chr7:g.140453136A>T`
- Gene + protein: `BRAF V600E`, `BRAF p.Val600Glu`

## Helpers

- `variant trials <id> --source <ctgov|nci> --limit <N> --offset <N>`
- `variant articles <id>`
"#
    .to_string();

    if has_oncokb {
        out.push_str("- `variant oncokb <id>` - explicit OncoKB lookup for therapies/levels\n");
    } else {
        out.push_str("\nOncoKB helper: set `ONCOKB_TOKEN`, then use `variant oncokb <id>`.\n");
    }
    out
}

fn list_article() -> String {
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

fn list_trial() -> String {
    r#"# trial

## Commands

- `get trial <nct_id>` - protocol card by NCT ID
- `get trial <nct_id> eligibility` - show eligibility criteria inline
- `get trial <nct_id> locations` - site locations section
- `get trial <nct_id> locations --offset <N> --limit <N>` - paged location slice
- `get trial <nct_id> outcomes` - primary/secondary outcomes
- `get trial <nct_id> arms` - arm/intervention details
- `get trial <nct_id> references` - trial publication references
- `get trial <nct_id> all` - include every section
- `search trial [filters]` - search ClinicalTrials.gov (default) or NCI CTS (`--source nci`)

## Useful filters (ctgov)

- `--condition <name>` (or `-c`)
- `--intervention <name>` (or `-i`)
- `--no-alias-expand`
- `--status <status>` (or `-s`)
- `--phase <NA|1|1/2|2|3|4>` (or `-p`)
- `--facility <name>`
- `--age <years>` (decimals accepted, e.g. `0.5`)
- `--sex <female|male|all>`
- `--mutation <text>`
- `--criteria <text>`
- `--biomarker <text>`
- `--sponsor-type <nih|industry|fed|other>`
- `--prior-therapies <text>`
- `--progression-on <drug>`
- `--line-of-therapy <1L|2L|3L+>`
- `--lat <N>` + `--lon <N>` + `--distance <miles>`
- `--results-available`
- `--has-results` (alias)
- `--study-type <interventional|observational|...>`
- `--date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- `--count-only`
- `--limit <N> --offset <N>`

## CTGov alias expansion

- `--intervention` auto-expands known aliases from the shared drug identity surface on the default CTGov path.
- Expanded rows add `Matched Intervention` in markdown and `matched_intervention_label` in JSON when an alternate alias matched first.
- `--no-alias-expand` forces literal matching.
- `--next-page` is not supported once alias expansion fans out to multiple queries; use `--offset` or `--no-alias-expand`.

## NCI source notes

- `--source nci --condition <name>` first tries to ground the name to an NCI disease ID and falls back to CTS `keyword`; there is no separate NCI keyword flag.
- `--source nci --status <status>` accepts one normalized status at a time and maps it to CTS recruitment or lifecycle filters.
- `--source nci --phase 1/2` maps to CTS `I_II`; `--phase early_phase1` is not supported.
- `--source nci --lat/--lon/--distance` uses direct `sites.org_coordinates_*` CTS filters and serializes distance with the required `mi` suffix.

## JSON Output

- Non-empty `search trial --json` responses include `_meta.next_commands`.
- Alias-expanded trial rows may include `matched_intervention_label`.
- The first follow-up drills the top result with `biomcp get trial <nct_id>`.
- `biomcp list trial` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

fn list_diagnostic() -> String {
    r#"# diagnostic

## When to use this surface

- Use `search diagnostic` when you need source-native diagnostic inventory from the local GTR and WHO IVD bundles.
- Start with `--gene` for GTR genetic-test questions, or `--disease --source who-ivd` for WHO infectious-disease diagnostics; add `--type` or `--manufacturer` only when narrowing a real result set.
- Use `get diagnostic <id>` for the base summary card, then add `genes`, `conditions`, `methods`, or `regulatory` when you need progressive disclosure.

## Commands

- `get diagnostic <gtr_accession>` - summary card from the local GTR bundle
- `get diagnostic "<who_ivd_product_code>"` - summary card from the local WHO IVD CSV
- `get diagnostic <gtr_accession> genes` - joined gene list from GTR detail data
- `get diagnostic <gtr_accession> conditions` - joined condition list from GTR detail data
- `get diagnostic <gtr_accession> methods` - GTR methods list
- `get diagnostic <id> regulatory` - optional live FDA device 510(k)/PMA overlay matched from source-native diagnostic names
- `get diagnostic "<who_ivd_product_code>" conditions` - WHO target/marker section
- `get diagnostic <id> all` - include every section supported by the resolved source
- `search diagnostic --gene <symbol>` - case-insensitive exact gene match
- `search diagnostic --disease <name> --source who-ivd` - minimum-length word/phrase match over WHO pathogen/disease/marker
- `search diagnostic --disease <name> --source gtr` - minimum-length word/phrase match over GTR condition names
- `search diagnostic --type <test_type> --source <gtr|who-ivd|all>` - case-insensitive exact type filter
- `search diagnostic --manufacturer <name> --source <gtr|who-ivd|all>` - case-insensitive substring over manufacturer/lab labels
- `search diagnostic ... --limit <N> --offset <N>` - offset pagination with `1..=50` result limits

## Search rules

- At least one of `--gene`, `--disease`, `--type`, or `--manufacturer` is required.
- All provided filters are conjunctive.
- `--disease` must contain at least 3 alphanumeric characters and matches full words or phrases at boundaries; short noisy tokens are rejected.
- `--source` accepts `gtr`, `who-ivd`, or `all` (default).
- Explicit `--source who-ivd --gene ...` is invalid; use `--source gtr` or omit `--source` for gene-first workflows.
- Use `--limit` and `--offset` to page broader diagnostic result sets beyond capped disease cards.
- Result ordering is deterministic: normalized test name ascending, then accession ascending.
- `summary` is always part of `get diagnostic`; supported public section tokens are `genes`, `conditions`, `methods`, `regulatory`, and `all`.
- Source-aware section support: GTR supports `genes`, `conditions`, `methods`, and `regulatory`; WHO IVD supports `conditions` and `regulatory`.
- `all` stays source-aware but intentionally excludes `regulatory` because the FDA overlay is live and opt-in.

## JSON Output

- Non-empty `search diagnostic --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get diagnostic <id>` and quotes WHO product codes that contain spaces.
- Non-empty next commands include `biomcp list diagnostic` so agents can inspect the full filter surface.
- True zero-result `search diagnostic --json` responses keep `count: 0`, `results: []`, and truthful pagination while adding `_meta.suggestions`.
- Zero-result suggestions include `biomcp list diagnostic` so agents can inspect source-aware diagnostic filters and local GTR/WHO IVD usage.
- `get diagnostic --json` keeps section-aware follow-ups and `_meta.section_sources`.
- `get diagnostic --json ... regulatory` adds a top-level `regulatory` field; omitting the section omits the field, and no FDA match serializes `regulatory: []`.

## Local data

- BioMCP auto-downloads GTR local data on first diagnostic use into `BIOMCP_GTR_DIR` or the default platform data directory.
- BioMCP auto-downloads WHO IVD local data on first WHO diagnostic use into `BIOMCP_WHO_IVD_DIR` or the default platform data directory.
- Full `biomcp health` reports `GTR local data (<resolved_root>)` and `WHO IVD local data (<resolved_root>)`; `biomcp health --apis-only` intentionally omits them.
- Use `biomcp gtr sync` to force-refresh the local GTR bundle.
- Use `biomcp who-ivd sync` to force-refresh the local WHO IVD CSV.
"#
    .to_string()
}

fn list_drug() -> String {
    r#"# drug

## When to use this surface

- Use the positional name lookup when you already know the drug or brand name.
- Use `--indication`, `--target`, or `--mechanism` when the question is structured.
- Use `get drug <name>` for label, regulatory, safety, target, or indication detail after you have the normalized drug name.

## Commands

- `get drug <name>` - get by name (MyChem.info aggregation)
- `get drug <name> label [--raw]` - compact FDA approved-indications summary by default; add `--raw` for the truncated FDA label text
- `get drug <name> regulatory [--region <us|eu|who|all>]` - regional regulatory summary (Drugs@FDA, EMA, and/or WHO Prequalification)
- `get drug <name> safety [--region <us|eu|all>]` - regional safety context (OpenFDA and/or EMA)
- `get drug <name> shortage [--region <us|eu|all>]` - query current shortage status
- `get drug <name> targets` - generic targets from ChEMBL/OpenTargets plus additive CIViC variant-target annotations when available
- `get drug <name> indications` - enrich with OpenTargets indications
- `get drug <name> interactions` - OpenFDA label interaction text when available; otherwise a truthful public-data fallback
- `get drug <name> civic` - CIViC therapy evidence/assertion summary
- `get drug <name> approvals` - Drugs@FDA approval/application details (US-only legacy section)
- `get drug <name> all [--region <us|eu|who|all>]` - include all sections

## Search

- `search drug <query>`
- `search drug -q <query>`
- `search drug <query> --region <us|eu|who|all>`
- `search drug <query> --region who --product-type <finished_pharma|api|vaccine>`
- `search drug --target <gene>`
- `search drug --indication <disease>`
- `search drug --indication <disease> --region who --product-type <finished_pharma|api>`
- `search drug --mechanism <text>`
- `search drug --atc <code>`
- `search drug --pharm-class <class>`
- `search drug --interactions <drug>` - unavailable from current public data sources
- `search drug ... --limit <N> --offset <N>`

## Helpers

- `drug trials <name> [--no-alias-expand]`
- `drug adverse-events <name>` - checks FAERS first, distinguishes FAERS 404 from FAERS 200+empty results, and falls back to ClinicalTrials.gov trial-reported adverse events only on FAERS 404

## JSON Output

- `search drug --json` responses use a region-aware envelope: top-level `region`, top-level `regions`, and optional top-level `_meta`.
- Single-region searches expose one bucket under `regions.us`, `regions.eu`, or `regions.who`.
- Omitted `--region` on plain name/alias lookup and explicit `--region all` expose `regions.us`, `regions.eu`, and `regions.who`.
- Each region bucket keeps `pagination`, `count`, and `results`.
- Non-empty `search drug --json` responses include `_meta.next_commands`.
- Structured indication searches with matching results can also include `_meta.workflow` and `_meta.ladder[]` for the `treatment-lookup` workflow.
- Non-vaccine searches keep `biomcp get drug <name>` as the preferred follow-up; WHO vaccine-only results stay search-only and omit broken `get drug` guidance.
- `biomcp list drug` is always included so agents can inspect the full filter surface.
- `biomcp --json drug adverse-events <name>` keeps the FAERS `summary` / `results` / `count` fields, adds `faers_not_found`, and includes `trial_adverse_events` only when the ClinicalTrials.gov fallback returns posted trial adverse-event terms.

## Notes

- Omitting `--region` searches U.S., EU, and WHO data for plain name/alias lookups.
- Structured filters remain U.S.-only when `--region` is omitted.
- Explicit `--region who` filters structured U.S. hits through WHO prequalification.
- `--product-type <finished_pharma|api|vaccine>` is WHO-only and requires explicit `--region who`.
- WHO vaccine search is plain name/brand only; structured WHO filters reject `--product-type vaccine`.
- Default WHO search excludes vaccines unless you explicitly request `--product-type vaccine`.
- Explicit `--region eu|all` is still invalid with structured filters.
- `ema` is accepted as an input alias for the canonical `eu` drug region value.
- Omitting `--region` on `get drug <name> regulatory` is the one implicit combined-region get path; other no-flag `get drug` shapes stay on the default U.S. path.
- WHO vaccine support in this ticket is search-only; `get drug <name> regulatory --region who|all` remains finished-pharma/API only.
- `drug trials <name>` inherits CTGov intervention alias expansion, adds `Matched Intervention` / `matched_intervention_label` when an alternate alias matched first, and accepts `--no-alias-expand` for literal matching.
- `drug adverse-events <name>` explains when a drug is absent from FAERS versus present with no matching FAERS events; only the FAERS-404 branch queries ClinicalTrials.gov.
- EU regional commands auto-download the EMA human-medicines JSON feeds into `BIOMCP_EMA_DIR` or the default data directory on first use.
- Default/EU vaccine brand lookups and explicit WHO vaccine name/brand searches can also auto-download the CDC CVX/MVX bundle into `BIOMCP_CVX_DIR` or the default data directory on first use.
- WHO regional commands auto-download the WHO finished-pharma, API, and vaccine CSV exports into `BIOMCP_WHO_DIR` or the default data directory on first use (`who_pq.csv`, `who_api.csv`, and `who_vaccines.csv`).
- Run `biomcp ema sync`, `biomcp cvx sync`, `biomcp who sync`, `biomcp gtr sync`, or `biomcp who-ivd sync` to force-refresh the local runtime data.
"#
    .to_string()
}

fn list_disease() -> String {
    r#"# disease

## When to use this surface

- Use `get disease <name_or_id>` when you want the normalized disease card with genes, pathways, and phenotypes.
- Use `get disease <name_or_id> diagnostics` when you need a capped diagnostic-test card from local GTR and WHO IVD data.
- Use `get disease <name_or_id> funding` when the question is about NIH grant support for a disease.
- Use `get disease <name_or_id> survival` when the question is specifically about cancer survival outcomes.
- Use `get disease <name_or_id> phenotypes` for symptom-style questions.
- Use `get disease <name_or_id> clinical_features` only when you need MedlinePlus clinical-summary rows for configured diseases; unsupported diseases omit fabricated rows, and the section stays opt-in.
- Use `search article -d <disease>` when you need broader review literature or want to supplement sparse structured data.

## Commands

- `get disease <name_or_id>` - resolve MONDO/DOID or best match by name with OpenTargets gene scores
- `get disease <name_or_id> genes` - Monarch rows plus additive CIViC/OpenTargets disease-gene associations with merged OpenTargets scores
- `get disease <name_or_id> pathways` - Reactome pathways from associated genes
- `get disease <name_or_id> phenotypes` - HPO phenotypes with resolved names
- `get disease <name_or_id> diagnostics` - up to 10 diagnostic tests for this condition from GTR and WHO IVD
- `get disease <name_or_id> variants` - CIViC disease-associated molecular profiles
- `get disease <name_or_id> models` - Monarch model-organism evidence
- `get disease <name_or_id> prevalence` - OpenTargets prevalence-like evidence
- `get disease <name_or_id> survival` - SEER Explorer 5-year relative survival by sex for mapped cancers
- `get disease <name_or_id> civic` - CIViC disease-context evidence
- `get disease <name_or_id> disgenet` - DisGeNET scored disease-gene associations (requires `DISGENET_API_KEY`)
- `get disease <name_or_id> funding` - NIH Reporter grants for the requested disease phrase, or the resolved canonical name for identifier lookups, over the most recent 5 NIH fiscal years
- `get disease <name_or_id> clinical_features` - MedlinePlus clinical-summary rows for configured diseases; unsupported diseases omit fabricated rows
- `get disease <name_or_id> all` - include all standard disease sections (`diagnostics`, `disgenet`, `funding`, and `clinical_features` stay opt-in)
- `search disease <query>` - positional search by name
- `search disease -q <query>` - search by name
- `search phenotype "<HP terms or symptom phrases>"` - HPO IDs or resolved symptom text to ranked diseases
- `search disease -q <query> --source <mondo|doid|mesh>` - constrain ontology source
- `search disease -q <query> --inheritance <pattern>`
- `search disease -q <query> --phenotype <HP:...>`
- `search disease -q <query> --onset <period>`
- `search disease -q <query> --no-fallback` - skip discover recovery and keep the direct zero-result response
- `search disease ... --limit <N> --offset <N>`

Disease diagnostic cards are capped at 10 rows. When rows exist, the card
prints a `See also:` command such as
`biomcp search diagnostic --disease <query> --source all --limit 50`; continue
with `--offset` on `search diagnostic` for later pages.

## Helpers

- `disease trials <name>`
- `disease articles <name>`
- `disease drugs <name>`

## JSON Output

- Non-empty `search disease --json` responses include `_meta.next_commands`.
- Disease search JSON emits at most one workflow ladder; `mutation-catalog` wins over `trial-recruitment` when both bounded probes match.
- The first follow-up drills the top result with `biomcp get disease <id>`.
- `biomcp list disease` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

fn list_phenotype() -> String {
    r#"# phenotype

## Commands

- `search phenotype "<HP:... HP:...>"` - rank diseases by phenotype similarity
- `search phenotype "<symptom phrase[, symptom phrase]>"` - resolve symptom text to HPO IDs, then rank diseases
- `search phenotype "<HP:...>" --limit <N> --offset <N>` - page ranked disease matches

## Examples

- `search phenotype "HP:0001250 HP:0001263"`
- `search phenotype "HP:0001250" --limit <N> --offset <N>`
- `search phenotype "HP:0001250,HP:0001263" --limit 10`
- `search phenotype "seizure, developmental delay" --limit 10`

## Workflow tips

- Start with 2-5 high-confidence HPO terms when you have them; otherwise use one symptom phrase or comma-separated symptom phrases.
- Use specific neurologic/cancer phenotype terms before broad umbrella terms.
- Run `discover "<symptom text>"` first when you want BioMCP to surface candidate `HP:` terms before ranking diseases.
- Follow with `get disease <id> all` to inspect phenotypes, genes, and pathways.

## Related

- `search disease -q <query> --phenotype <HP:...>`
- `disease trials <name>`
- `disease articles <name>`
"#
    .to_string()
}

fn list_pgx() -> String {
    r#"# pgx

## Commands

- `get pgx <gene_or_drug>` - CPIC-based PGx card by gene or drug
- `get pgx <gene_or_drug> recommendations` - dosing recommendation section
- `get pgx <gene_or_drug> frequencies` - population frequency section
- `get pgx <gene_or_drug> guidelines` - guideline metadata section
- `get pgx <gene_or_drug> annotations` - PharmGKB enrichment section
- `get pgx <gene_or_drug> all` - include all PGx sections
- `search pgx -g <gene>` - interactions by gene
- `search pgx -d <drug>` - interactions by drug
- `search pgx --cpic-level <A|B|C|D>`
- `search pgx --pgx-testing <value>`
- `search pgx --evidence <level>`
- `search gwas -g <gene>` - GWAS-linked variants by gene
- `search gwas --trait <text>` - GWAS-linked variants by disease trait

## Examples

- `get pgx CYP2D6`
- `get pgx codeine recommendations`
- `search pgx -g CYP2D6 --limit 5`
- `search gwas --trait "type 2 diabetes" --limit 5`

## JSON Output

- Non-empty `search pgx --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get pgx <gene_or_drug>`.
- `biomcp list pgx` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

fn list_gwas() -> String {
    r#"# gwas

## Commands

- `search gwas -g <gene>` - GWAS-linked variants by gene
- `search gwas --trait <text>` - GWAS-linked variants by disease trait
- `search gwas --region <chr:start-end>`
- `search gwas --p-value <threshold>`
- `search gwas ... --limit <N> --offset <N>`

## Examples

- `search gwas -g TCF7L2 --limit 5`
- `search gwas --trait "type 2 diabetes" --limit 5`
- `search gwas --region 7:55000000-55200000 --p-value 5e-8 --limit 10`

## Workflow tips

- Use `--trait` for phenotype-first discovery and `-g` for gene-first review.
- Tighten noisy results with `--p-value` and locus-focused `--region`.
- Pivot high-interest hits into `get variant <id>` and `variant trials <id>`.

## Related

- `list pgx` - pharmacogenomics command family
- `search trial --mutation <text>`
- `search trial --criteria <text>`
- `search article -g <gene>`

## JSON Output

- Non-empty `search gwas --json` responses include `_meta.next_commands`.
- The first follow-up drills the top hit with `biomcp get variant <rsid>`.
- `biomcp list gwas` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

fn list_batch() -> String {
    r#"# batch

## When to use this surface

- Use batch when you already have a short list of IDs and want the same `get` call repeated consistently.
- Batch is better than sequential `get` calls when you are comparing a few entities side by side.

## Command

- `batch <entity> <id1,id2,...>` - parallel `get` operations for up to 10 IDs

## Options

- `--sections <s1,s2,...>` - request specific sections on each entity
- `--source <ctgov|nci>` - trial source when `entity=trial` (default: `ctgov`)

## Supported entities

- `gene`, `variant`, `article`, `trial`, `drug`, `disease`, `pgx`, `pathway`, `protein`, `adverse-event`

## Examples

- `batch gene BRAF,TP53 --sections pathways,ontology`
- `batch trial NCT04280705,NCT04639219 --source nci --sections locations`
"#
    .to_string()
}

fn list_enrich() -> String {
    r#"# enrich

## When to use this surface

- Use enrich when you already have a gene set and need pathways, GO terms, or broader functional categories.
- Start using enrichment once you have 3 or more genes; smaller lists are often better handled by direct `get gene` review.

## Command

- `enrich <GENE1,GENE2,...>` - gene-set enrichment using g:Profiler

## Options

- `--limit <N>` - max number of returned terms (must be 1-50; default 10)

## Examples

- `enrich BRAF,KRAS,NRAS`
- `enrich EGFR,ALK,ROS1 --limit 20`
"#
    .to_string()
}

fn list_search_all() -> String {
    r#"# search-all

## Command

- `search all` - cross-entity summary card with curated section fan-out

## Slots

- `--gene` (or `-g`)
- `--variant` (or `-v`)
- `--disease` (or `-d`)
- `--drug`
- `--keyword` (or `-k`)

## Output controls

- `--since <YYYY|YYYY-MM|YYYY-MM-DD>` - applies to date-capable sections
- `--limit <N>` - rows per section (default: 3)
- `--counts-only` - markdown keeps section counts and follow-up links without row tables; `--json` omits per-section results and links
- `--debug-plan` - include executed leg/routing metadata in markdown or JSON
- `--json` - machine-readable sections; in `--counts-only` mode sections carry metadata and counts only

## Notes

- At least one typed slot is required.
- Unanchored keyword-only dispatch is article-only.
- Keyword is pushed into drug search only when `--gene` and/or `--disease` is present.

## Understanding the Output

- Section order follows anchor priority: gene, disease, drug, variant, then keyword-only.
- `get.top` links open the top row as a detailed card.
- `cross.*` links pivot to a related entity search.
- `filter.hint` links show useful next filters for narrowing.
- `search.retry` links appear when a section errors or times out.
- In `--json --counts-only`, per-section follow-up links are omitted; markdown counts-only keeps them.
- Typical workflow: `search all` -> `search <entity>` -> `get <entity> <id>` -> helper commands.
"#
    .to_string()
}

fn list_pathway() -> String {
    r#"# pathway

## Commands

- `search pathway <query>` - positional pathway search (Reactome + KEGG)
- `search pathway -q <query>` - pathway search (Reactome + KEGG)
- `search pathway -q <query> --type pathway`
- `search pathway --top-level`
- `search pathway -q <query> --limit <N> --offset <N>`
- `get pathway <id>` - base pathway card
- `get pathway <id> genes` - pathway participant genes
- `get pathway <id> events` - contained events (Reactome only)
- `get pathway <id> enrichment` - g:Profiler enrichment from pathway genes (Reactome only)
- `get pathway <id> all` - include all sections supported by that pathway source

## Search filters

- `search pathway <query>`
- `search pathway -q <query>`
- `--type pathway`
- `--top-level`
- `--limit <N> --offset <N>`

## Helpers

- `pathway drugs <id>`
- `pathway articles <id>`
- `pathway trials <id>`

## Workflow examples

- To find pathways for an altered gene, run `biomcp search pathway "<gene or process>" --limit 5`.
- To inspect pathway composition, run `biomcp get pathway <id> genes`.
- For Reactome pathways, events are also available: `biomcp get pathway R-HSA-5673001 events`.
- To pivot to clinical context, run `biomcp pathway trials <id>` and `biomcp pathway articles <id>`.

## JSON Output

- Non-empty `search pathway --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get pathway <id>`.
- `biomcp list pathway` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

fn list_study() -> String {
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

fn list_protein() -> String {
    r#"# protein

## Commands

- `search protein -q <query>` - protein search (UniProt, human-only by default)
- `search protein <query>` - positional query form
- `search protein -q <query> --all-species`
- `search protein -q <query> --reviewed`
- `search protein -q <query> --disease <name>`
- `search protein -q <query> --existence <1-5>`
- `search protein ... --limit <N> --offset <N>`
- `get protein <accession_or_symbol>` - base protein card
- `get protein <accession> domains` - InterPro domains
- `get protein <accession> interactions` - STRING interactions
- `get protein <accession> complexes` - ComplexPortal protein complexes
- `get protein <accession> structures` - structure IDs (PDB/AlphaFold)
- `get protein <accession> all` - include all sections

## Search filters

- `search protein <query>`
- `search protein -q <query>`
- `--all-species`
- `--reviewed` (default behavior uses reviewed=true for safer results)
- `--disease <name>`
- `--existence <1-5>`
- `--limit <N> --offset <N>`
- `--next-page <token>` (cursor compatibility alias; `--offset` is preferred UX)

## Helpers

- `protein structures <accession> --limit <N> --offset <N>`

## Workflow examples

- To find a target protein from a gene symbol, run `biomcp search protein BRAF --limit 5`.
- To inspect complex membership, run `biomcp get protein <accession> complexes`.
- To inspect structural context, run `biomcp get protein <accession> structures`.
- To continue result browsing, run `biomcp search protein <query> --limit <N> --offset <N>`.
"#
    .to_string()
}

fn list_adverse_event() -> String {
    r#"# adverse-event

## Commands

- `search adverse-event --drug <name> --source <faers|vaers|all>` - FAERS by default for ordinary drug queries; vaccine-resolved searches can add CDC VAERS
- `search adverse-event <vaccine query> --source vaers` - aggregate CDC WONDER VAERS summary
- `search adverse-event --drug <name> --outcome <death|hospitalization|disability>`
- `search adverse-event --drug <name> --serious <type>`
- `search adverse-event --drug <name> --date-from <YYYY|YYYY-MM-DD> --date-to <YYYY|YYYY-MM-DD>`
- `search adverse-event --drug <name> --suspect-only --sex <m|f> --age-min <N> --age-max <N>`
- `search adverse-event --drug <name> --reporter <type>`
- `search adverse-event --drug <name> --count <field>` - OpenFDA FAERS aggregation mode
- `search adverse-event ... --limit <N> --offset <N>`
- `get adverse-event <report_id>` - retrieve report by ID

## Source behavior

- default `--source all` always runs OpenFDA FAERS and adds CDC VAERS only when the query resolves to a vaccine and the active filters are VAERS-compatible
- `--source vaers` is aggregate-only and supports plain vaccine query text from `--drug` or the positional query
- VAERS intentionally does not support --reaction, --outcome, --serious, --date-from, --date-to, --suspect-only, --sex, --age-min, --age-max, --reporter, --count, or --offset > 0
- `--source` only applies to `--type faers`; recall and device searches keep their existing source-specific paths

## Other query types

- `search adverse-event --type recall --drug <name>` - enforcement/recalls
- `search adverse-event --type device --device <name>` - MAUDE device events
- `search adverse-event --type device --manufacturer <name>` - MAUDE by manufacturer
- `search adverse-event --type device --product-code <code>` - MAUDE by product code

## JSON Output

- Non-empty `search adverse-event --json` responses include `_meta.next_commands`.
- `--source all` keeps the FAERS envelope and adds a truthful `vaers` status block.
- `--source vaers` returns a VAERS-first envelope with `source`, `query`, `vaers`, and `_meta`.
- FAERS and device searches drill the top result with `biomcp get adverse-event <report_id>`.
- Recall searches currently return `biomcp list adverse-event` without a recall-specific `get` command.
- `biomcp list adverse-event` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::{list_drug, list_gene, render};

    #[test]
    fn list_root_includes_routing_table_and_quickstart() {
        let out = render(None).expect("list root should render");
        assert!(out.contains("## Quickstart"));
        assert!(out.contains("## When to Use What"));
        assert_eq!(out.matches("## When to Use What").count(), 1);
        assert!(out.contains("search all --gene BRAF --disease melanoma"));
        assert!(out.contains("suggest \"What drugs treat melanoma?\""));
        assert!(out.contains("discover \"<free text>\""));
        assert!(out.contains("single-entity biomedical phrase"));
        assert!(out.contains("search all --keyword \"<query>\""));
        assert!(out.contains("article citations <id>"));
        assert!(out.contains("enrich <GENE1,GENE2,...>"));
        assert!(out.contains("Turn a literature question into article filters"));
        assert!(out.contains("`skill install` - install BioMCP skill guidance to your agent"));
        assert!(out.contains("`suggest <question>`"));
        assert!(out.contains("`discover <query>`"));
        assert!(out.contains("`cache path`"));
        assert!(out.contains("`cache stats`"));
        assert!(
            out.contains("`cache clean [--max-age <duration>] [--max-size <size>] [--dry-run]`")
        );
        assert!(!out.contains("## Query formulation"));
        assert!(!out.contains("photosensitivity mechanism"));
    }

    #[test]
    fn list_discover_page_exists() {
        let out = render(Some("discover")).expect("list discover should render");
        assert!(out.contains("# discover"));
        assert!(out.contains("discover <query>"));
        assert!(out.contains("--json discover <query>"));
        assert!(out.contains("single-entity resolver"));
        assert!(out.contains("search all --keyword \"<query>\""));
        assert!(out.contains("gene-plus-topic queries"));
        assert!(out.contains("biomcp search article -g <symbol> -k <topic> --limit 5"));
        assert!(out.contains("biomcp search article -k <query> --type review --limit 5"));
    }

    #[test]
    fn list_suggest_page_exists() {
        let out = render(Some("suggest")).expect("list suggest should render");
        assert!(out.contains("# suggest"));
        assert!(out.contains("suggest <question>"));
        assert!(out.contains("--json suggest <question>"));
        assert!(out.contains("matched_skill"));
        assert!(out.contains("first_commands"));
        assert!(out.contains("full_skill"));
        assert!(out.contains("No confident BioMCP skill match"));
        assert!(out.contains("discover \"<question>\""));
    }

    #[test]
    fn list_diagnostic_page_exists() {
        let out = render(Some("diagnostic")).expect("list diagnostic should render");
        assert!(out.contains("# diagnostic"));
        assert!(out.contains("search diagnostic --gene <symbol>"));
        assert!(out.contains("get diagnostic <id> regulatory"));
        assert!(out.contains("get diagnostic \"<who_ivd_product_code>\" conditions"));
        assert!(out.contains("minimum-length word/phrase match over WHO"));
        assert!(out.contains("must contain at least 3 alphanumeric characters"));
        assert!(
            out.contains("Use `--limit` and `--offset` to page broader diagnostic result sets")
        );
        assert!(out.contains("supported public section tokens are `genes`, `conditions`, `methods`, `regulatory`, and `all`."));
        assert!(out.contains("intentionally excludes `regulatory`"));
        assert!(out.contains("True zero-result `search diagnostic --json` responses keep"));
        assert!(out.contains("Zero-result suggestions include `biomcp list diagnostic`"));
        assert!(out.contains("source-aware diagnostic filters and local GTR/WHO IVD usage"));
        assert!(out.contains("biomcp gtr sync"));
        assert!(out.contains("biomcp who-ivd sync"));
        assert!(out.contains("WHO IVD local data (<resolved_root>)"));
    }

    #[test]
    fn list_discover_page_mentions_gene_topic_article_followup() {
        let out = render(Some("discover")).expect("list discover should render");
        assert!(out.contains(
            "Unambiguous gene-plus-topic queries can also surface `biomcp search article -g <symbol> -k <topic> --limit 5` when the remaining topic is meaningful."
        ));
    }

    #[test]
    fn list_discover_page_mentions_empty_and_low_confidence_article_fallbacks() {
        let out = render(Some("discover")).expect("list discover should render");
        assert!(out.contains(
            "If no biomedical entities resolve, discover suggests `biomcp search article -k <query> --type review --limit 5`."
        ));
        assert!(out.contains(
            "If only low-confidence concepts resolve, discover adds a broader-results article-search hint."
        ));
    }

    #[test]
    fn list_discover_page_mentions_relational_redirect_and_supported_exceptions() {
        let out = render(Some("discover")).expect("list discover should render");
        assert!(out.contains(
            "Existing routed exceptions remain supported for symptom-of-disease prompts, HPO symptom bridging, treatment prompts, gene+disease orientation, and unambiguous gene-plus-topic follow-ups."
        ));
        assert!(out.contains(
            "Relational or multi-entity questions may redirect to `biomcp search all --keyword \"<query>\"`."
        ));
    }

    #[test]
    fn list_search_all_page_mentions_counts_only_json_contract() {
        let out = render(Some("search-all")).expect("list search-all should render");
        assert!(out.contains("markdown keeps section counts and follow-up links"));
        assert!(out.contains("`--json` omits per-section results and links"));
        assert!(out.contains("metadata and counts only"));
    }

    #[test]
    fn list_entity_pages_drop_stale_skill_sections() {
        for entity in ["gene", "variant", "drug"] {
            let out = render(Some(entity)).expect("entity page should render");
            assert!(
                !out.contains("## Recommended skills"),
                "{entity} page should not advertise removed use-case skills"
            );
            assert!(
                !out.contains("## Skills"),
                "{entity} page should not append the generic skills section"
            );
            assert!(
                !out.contains("biomcp skill "),
                "{entity} page should not reference stale skill commands"
            );
        }
    }

    #[test]
    fn list_skill_alias_routes_to_skill_listing() {
        let out = render(Some("skill")).expect("list skill should render");
        assert!(out.contains("# BioMCP Worked Examples"));
        assert!(out.contains("01 treatment-lookup"));
        assert!(out.contains("04 article-follow-up"));
        assert!(out.contains("15 negative-evidence"));
    }

    #[test]
    fn list_batch_and_enrich_pages_exist() {
        let batch = render(Some("batch")).expect("list batch should render");
        assert!(batch.contains("# batch"));
        assert!(batch.contains("batch <entity> <id1,id2,...>"));
        assert!(batch.contains("## When to use this surface"));
        assert_eq!(batch.matches("## When to use this surface").count(), 1);
        assert!(batch.contains("Use batch when you already have a short list of IDs"));

        let enrich = render(Some("enrich")).expect("list enrich should render");
        assert!(enrich.contains("# enrich"));
        assert!(enrich.contains("enrich <GENE1,GENE2,...>"));
        assert!(enrich.contains("## When to use this surface"));
        assert_eq!(enrich.matches("## When to use this surface").count(), 1);
        assert!(enrich.contains("Use enrich when you already have a gene set"));
    }

    #[test]
    fn list_study_page_exists() {
        let out = render(Some("study")).expect("list study should render");
        assert!(out.contains("# study"));
        assert!(out.contains("study download [--list] [<study_id>]"));
        assert!(out.contains("study top-mutated --study <id> [--limit <N>]"));
        assert!(out.contains(
            "study filter --study <id> [--mutated <symbol>] [--amplified <symbol>] [--deleted <symbol>]"
        ));
        assert!(out.contains(
            "study query --study <id> --gene <symbol> --type <mutations|cna|expression>"
        ));
        assert!(out.contains("study cohort --study <id> --gene <symbol>"));
        assert!(
            out.contains(
                "study survival --study <id> --gene <symbol> [--endpoint <os|dfs|pfs|dss>]"
            )
        );
        assert!(out.contains(
            "study compare --study <id> --gene <symbol> --type <expression|mutations> --target <symbol>"
        ));
    }

    #[test]
    fn list_gene_mentions_new_gene_sections() {
        let out = list_gene();
        assert!(out.contains("## When to use this surface"));
        assert!(out.contains("Use `get gene <symbol>` for the default card"));
        assert!(out.contains("`expression`, `diseases`, `diagnostics`, or `funding`"));
        assert!(out.contains("get gene <symbol> expression"));
        assert!(out.contains("get gene <symbol> hpa"));
        assert!(out.contains("get gene <symbol> druggability"));
        assert!(out.contains("get gene <symbol> clingen"));
        assert!(out.contains("get gene <symbol> constraint"));
        assert!(out.contains("get gene <symbol> diagnostics"));
        assert!(out.contains("get gene <symbol> disgenet"));
        assert!(out.contains("get gene <symbol> funding"));
        assert!(out.contains("`diagnostics` and `funding` stay opt-in"));
    }

    #[test]
    fn list_drug_describes_omitted_region_behavior() {
        let out = list_drug();
        assert!(out.contains("## When to use this surface"));
        assert_eq!(out.matches("## When to use this surface").count(), 1);
        assert!(out.contains(
            "Use the positional name lookup when you already know the drug or brand name."
        ));
        assert!(out.contains(
            "Use `--indication`, `--target`, or `--mechanism` when the question is structured."
        ));
        assert!(out.contains(
            "Omitting `--region` searches U.S., EU, and WHO data for plain name/alias lookups."
        ));
        assert!(out.contains("Structured filters remain U.S.-only when `--region` is omitted."));
        assert!(out.contains(
            "Explicit `--region who` filters structured U.S. hits through WHO prequalification."
        ));
        assert!(
            out.contains("`--product-type <finished_pharma|api|vaccine>` is WHO-only and requires explicit `--region who`.")
        );
        assert!(out.contains(
            "WHO vaccine search is plain name/brand only; structured WHO filters reject `--product-type vaccine`."
        ));
        assert!(out.contains(
            "Default WHO search excludes vaccines unless you explicitly request `--product-type vaccine`."
        ));
        assert!(
            out.contains("Explicit `--region eu|all` is still invalid with structured filters.")
        );
        assert!(out.contains(
            "`ema` is accepted as an input alias for the canonical `eu` drug region value."
        ));
        assert!(out.contains(
            "Omitting `--region` on `get drug <name> regulatory` is the one implicit combined-region get path; other no-flag `get drug` shapes stay on the default U.S. path."
        ));
        assert!(out.contains(
            "`search drug --json` responses use a region-aware envelope: top-level `region`, top-level `regions`, and optional top-level `_meta`."
        ));
        assert!(out.contains(
            "Single-region searches expose one bucket under `regions.us`, `regions.eu`, or `regions.who`."
        ));
        assert!(out.contains(
            "Omitted `--region` on plain name/alias lookup and explicit `--region all` expose `regions.us`, `regions.eu`, and `regions.who`."
        ));
        assert!(out.contains("Each region bucket keeps `pagination`, `count`, and `results`."));
        assert!(out.contains("drug trials <name> [--no-alias-expand]"));
        assert!(out.contains("inherits CTGov intervention alias expansion"));
        assert!(out.contains("Matched Intervention"));
        assert!(out.contains("matched_intervention_label"));
        assert!(out.contains("auto-download the EMA human-medicines JSON feeds"));
        assert!(out.contains(
            "search drug <query> --region who --product-type <finished_pharma|api|vaccine>"
        ));
        assert!(out.contains("who_pq.csv"));
        assert!(out.contains("who_api.csv"));
        assert!(out.contains("who_vaccines.csv"));
        assert!(out.contains("CDC CVX/MVX"));
        assert!(out.contains("biomcp cvx sync"));
        assert!(out.contains("biomcp ema sync"));
        assert!(out.contains("biomcp gtr sync"));
        assert!(out.contains("biomcp who-ivd sync"));
        assert!(out.contains("biomcp who sync"));
    }

    #[test]
    fn list_drug_documents_raw_label_mode() {
        let out = list_drug();
        assert!(out.contains("get drug <name> label [--raw]"));
    }

    #[test]
    fn list_disease_mentions_opt_in_sections() {
        let out = render(Some("disease")).expect("list disease should render");
        assert!(out.contains("## When to use this surface"));
        assert_eq!(out.matches("## When to use this surface").count(), 1);
        assert!(
            out.contains(
                "Use `get disease <name_or_id>` when you want the normalized disease card"
            )
        );
        assert!(out.contains("get disease <name_or_id> funding"));
        assert!(out.contains("get disease <name_or_id> survival"));
        assert!(out.contains("get disease <name_or_id> diagnostics"));
        assert!(out.contains("Disease diagnostic cards are capped at 10 rows"));
        assert!(out.contains("biomcp search diagnostic --disease <query> --source all --limit 50"));
        assert!(out.contains("Use `search article -d <disease>` when you need broader review"));
        assert!(out.contains("get disease <name_or_id> disgenet"));
        assert!(out.contains("get disease <name_or_id> clinical_features"));
        assert!(
            out.contains(
                "`diagnostics`, `disgenet`, `funding`, and `clinical_features` stay opt-in"
            )
        );
        assert!(out.contains("MedlinePlus clinical-summary rows for configured diseases"));
        assert!(out.contains("unsupported diseases omit fabricated rows"));
    }

    #[test]
    fn list_disease_mentions_phenotype_search_supports_symptom_phrases() {
        let out = render(Some("disease")).expect("list disease should render");
        assert!(out.contains("search phenotype \"<HP terms or symptom phrases>\""));
        assert!(out.contains("HPO IDs or resolved symptom text to ranked diseases"));
    }

    #[test]
    fn list_trial_and_article_include_missing_flags() {
        let trial = render(Some("trial")).expect("list trial should render");
        assert!(trial.contains("--biomarker <text>"));
        assert!(trial.contains("--no-alias-expand"));
        assert!(trial.contains("## CTGov alias expansion"));
        assert!(trial.contains("auto-expands known aliases"));
        assert!(trial.contains("Matched Intervention"));
        assert!(trial.contains("matched_intervention_label"));
        assert!(trial.contains("use `--offset` or `--no-alias-expand`"));
        assert!(trial.contains("## NCI source notes"));
        assert!(trial.contains("## JSON Output"));
        assert!(trial.contains("`_meta.next_commands`"));
        assert!(trial.contains("biomcp list trial"));
        assert!(trial.contains("NCI disease ID"));
        assert!(trial.contains("one normalized status at a time"));
        assert!(trial.contains("I_II"));
        assert!(trial.contains("early_phase1"));
        assert!(trial.contains("sites.org_coordinates"));
        assert!(trial.contains("no separate NCI keyword flag"));

        let article = render(Some("article")).expect("list article should render");
        assert!(article.contains("## When to use this surface"));
        assert!(
            article.contains("Use keyword search to scan a topic before you know the entities.")
        );
        assert!(article.contains("Add `-g/--gene` when you already know the molecular anchor."));
        assert!(article.contains("Prefer `--type review`"));
        assert!(article.contains("article citations <id>"));
        assert!(article.contains("article recommendations <id>"));
        assert!(article.contains("--date-from <YYYY|YYYY-MM|YYYY-MM-DD>"));
        assert!(article.contains("--date-to <YYYY|YYYY-MM|YYYY-MM-DD>"));
        assert!(article.contains("--since <YYYY|YYYY-MM|YYYY-MM-DD>"));
        assert!(article.contains("--year-min <YYYY>"));
        assert!(article.contains("--year-max <YYYY>"));
        assert!(article.contains("--ranking-mode <lexical|semantic|hybrid>"));
        assert!(article.contains("--weight-semantic <float>"));
        assert!(article.contains("--weight-lexical <float>"));
        assert!(article.contains("--weight-citations <float>"));
        assert!(article.contains("--weight-position <float>"));
        assert!(article.contains("--source <all, pubtator, europepmc, pubmed, litsense2>"));
        assert!(article.contains("--max-per-source <N>"));
        assert!(article.contains("--session <token>"));
        assert!(article.contains("search article --source litsense2"));
        assert!(article.contains(
            "Direct and compatible federated PubMed ESearch cleans question-format gene/disease/drug/keyword terms provider-locally"
        ));
        assert!(article.contains("60% post-stopword term overlap"));
        assert!(article.contains("Use a short non-identifying token such as `lit-review-1`"));
        assert!(article.contains("keyword-bearing article queries default to hybrid"));
        assert!(article.contains("entity-only queries default to lexical"));
        assert!(article.contains("get article <id> fulltext --pdf"));
        assert!(article.contains(
            "get article <id> fulltext` tries XML first, then PMC HTML, and never falls back to PDF."
        ));
        assert!(article.contains(
            "Add `--pdf` only with `fulltext` to extend that ladder with Semantic Scholar PDF as the last resort."
        ));
        assert!(article.contains(
            "`--pdf` requires the `fulltext` section and is rejected for other article requests."
        ));
        assert!(
            article.contains("LitSense2-derived semantic signal and falls back to lexical ties")
        );
        assert!(article.contains("0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position"));
        assert!(article.contains("rows without LitSense2 provenance contribute `semantic=0`"));
        assert!(article.contains(
            "Cap each federated source's contribution after deduplication and before ranking."
        ));
        assert!(article.contains(
            "Default: 40% of `--limit` on federated pools with at least three surviving primary sources."
        ));
        assert!(
            article.contains(
                "`0` uses the default cap; setting it equal to `--limit` disables capping."
            )
        );
        assert!(article.contains("Rows count against their primary source after deduplication."));
        assert!(article.contains("article batch <id> [<id>...]"));
        assert!(article.contains("## Query formulation"));
        assert!(article.contains("## JSON Output"));
        assert!(article.contains("`_meta.next_commands`"));
        assert!(article.contains("biomcp list article"));
        assert!(article.contains("first_index_date"));
        assert!(article.contains("Newest indexed: YYYY-MM-DD (N days ago)"));
        assert!(article.contains("Known gene/disease/drug already identified"));
        assert!(article.contains("Keyword-only topic, dataset, or method question"));
        assert!(
            article.contains(
                "Do not invent `-g/-d/--drug`; stay keyword-first or start with `discover`"
            )
        );
        assert!(article.contains("biomcp search article -g BRAF --limit 5"));
        assert!(
            article.contains(
                "biomcp search article -g TP53 -k \"apoptosis gene regulation\" --limit 5"
            )
        );
        assert!(article.contains(
            "Keyword-only result pages can suggest typed `get gene`, `get drug`, or `get disease` follow-ups when the whole `-k/--keyword` exactly matches a gene, drug, or disease vocabulary label or alias."
        ));
        assert!(article.contains(
            "Multi-concept keyword phrases and searches that already use `-g/--gene`, `-d/--disease`, or `--drug` do not get direct entity suggestions."
        ));
        assert!(article.contains(
            "Visible dated result pages with no existing date bounds can also suggest year-refinement next commands"
        ));
        assert!(article.contains(
            "biomcp search article --drug amiodarone -k \"photosensitivity mechanism\" --limit 5"
        ));
        assert!(article.contains(
            "biomcp search article -k '\"cafe-au-lait spots\" neurofibromas disease' --type review --limit 5"
        ));
        assert!(article.contains(
            "biomcp search article -k \"TCGA mutation analysis dataset\" --type review --limit 5"
        ));
        assert!(article.contains(
            "biomcp search article -k \"BRAF melanoma\" --year-min 2000 --year-max 2013 --limit 5"
        ));
        assert!(article.contains(
            "typed gene/disease/drug anchors participate in PubTator3 + Europe PMC + PubMed"
        ));
        assert!(article.contains(
            "Keyword-only exact entity matches can also add `biomcp get gene <symbol>`, `biomcp get drug <name>`, or `biomcp get disease <name>` to `_meta.next_commands`."
        ));
        assert!(article.contains(
            "Article search `_meta.suggestions` is an optional array of objects with `command` and `reason`."
        ));
        assert!(article.contains(
            "Exact entity suggestions include `sections`; loop-breaker suggestions from `--session` omit `sections`."
        ));
        assert!(article.contains("prior `biomcp article batch ...`, `biomcp discover <topic>`"));
        assert!(article.contains("visible dated rows can also add a year-refinement next command"));
    }

    #[test]
    fn list_article_page_mentions_entity_aware_followups() {
        let article = render(Some("article")).expect("list article should render");
        assert!(article.contains(
            "Keyword-only result pages can suggest typed `get gene`, `get drug`, or `get disease` follow-ups"
        ));
        assert!(article.contains(
            "Article search `_meta.suggestions` is an optional array of objects with `command` and `reason`."
        ));
        assert!(article.contains(
            "Exact entity suggestions include `sections`; loop-breaker suggestions from `--session` omit `sections`."
        ));
    }

    #[test]
    fn list_pathway_describes_source_aware_sections() {
        let out = render(Some("pathway")).expect("list pathway should render");
        assert!(out.contains("get pathway <id> events` - contained events (Reactome only)"));
        assert!(out.contains(
            "get pathway <id> enrichment` - g:Profiler enrichment from pathway genes (Reactome only)"
        ));
        assert!(out.contains(
            "get pathway <id> all` - include all sections supported by that pathway source"
        ));
        assert!(out.contains("biomcp get pathway <id> genes"));
        assert!(out.contains("Reactome pathways, events are also available"));
    }

    #[test]
    fn phenotype_and_gwas_include_workflow_tips() {
        let phenotype = render(Some("phenotype")).expect("list phenotype should render");
        assert!(phenotype.contains("## Workflow tips"));
        assert!(phenotype.contains("2-5 high-confidence HPO terms"));

        let gwas = render(Some("gwas")).expect("list gwas should render");
        assert!(gwas.contains("## JSON Output"));
        assert!(gwas.contains("`_meta.next_commands`"));
        assert!(gwas.contains("biomcp list gwas"));
        assert!(gwas.contains("## Workflow tips"));
        assert!(gwas.contains("--p-value"));
    }

    #[test]
    fn list_search_pages_document_search_json_next_commands() {
        for (entity, expected_list_command) in [
            ("gene", "biomcp list gene"),
            ("variant", "biomcp list variant"),
            ("drug", "biomcp list drug"),
            ("disease", "biomcp list disease"),
            ("pgx", "biomcp list pgx"),
            ("pathway", "biomcp list pathway"),
            ("adverse-event", "biomcp list adverse-event"),
        ] {
            let out = render(Some(entity)).expect("list page should render");
            assert!(
                out.contains("## JSON Output"),
                "{entity}: missing JSON Output section"
            );
            assert!(
                out.contains("`_meta.next_commands`"),
                "{entity}: missing _meta.next_commands docs"
            );
            assert!(
                out.contains(expected_list_command),
                "{entity}: missing list follow-up doc"
            );
        }
    }

    #[test]
    fn phenotype_list_mentions_hpo_ids_and_symptom_phrases() {
        let phenotype = render(Some("phenotype")).expect("list phenotype should render");
        assert!(phenotype.contains("search phenotype \"HP:0001250 HP:0001263\""));
        assert!(phenotype.contains("search phenotype \"<symptom phrase[, symptom phrase]>\""));
        assert!(phenotype.contains("search phenotype \"seizure, developmental delay\" --limit 10"));
        assert!(phenotype.contains("resolve symptom text to HPO IDs"));
        assert!(phenotype.contains("Run `discover \"<symptom text>\"` first"));
    }

    #[test]
    fn unknown_entity_lists_new_valid_entities() {
        let err = render(Some("unknown")).expect_err("unknown entity should fail");
        let msg = err.to_string();
        assert!(msg.contains("- skill"));
        assert!(msg.contains("- enrich"));
        assert!(msg.contains("- batch"));
        assert!(msg.contains("- study"));
        assert!(msg.contains("- suggest"));
        assert!(msg.contains("- discover"));
    }
}
