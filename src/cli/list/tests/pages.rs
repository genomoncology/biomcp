//! Page content tests for `biomcp list` command-reference output.

use super::super::render;

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
    assert!(out.contains("`cache clean [--max-age <duration>] [--max-size <size>] [--dry-run]`"));
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
    assert!(out.contains("Use `--limit` and `--offset` to page broader diagnostic result sets"));
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
    assert!(
        out.contains("study query --study <id> --gene <symbol> --type <mutations|cna|expression>")
    );
    assert!(out.contains("study cohort --study <id> --gene <symbol>"));
    assert!(
        out.contains("study survival --study <id> --gene <symbol> [--endpoint <os|dfs|pfs|dss>]")
    );
    assert!(out.contains(
        "study compare --study <id> --gene <symbol> --type <expression|mutations> --target <symbol>"
    ));
}

#[test]
fn list_gene_mentions_new_gene_sections() {
    let out = render(Some("gene")).expect("list gene should render");
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
    let out = render(Some("drug")).expect("list drug should render");
    assert!(out.contains("## When to use this surface"));
    assert_eq!(out.matches("## When to use this surface").count(), 1);
    assert!(
        out.contains(
            "Use the positional name lookup when you already know the drug or brand name."
        )
    );
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
    assert!(out.contains("Explicit `--region eu|all` is still invalid with structured filters."));
    assert!(
        out.contains(
            "`ema` is accepted as an input alias for the canonical `eu` drug region value."
        )
    );
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
    assert!(
        out.contains(
            "search drug <query> --region who --product-type <finished_pharma|api|vaccine>"
        )
    );
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
    let out = render(Some("drug")).expect("list drug should render");
    assert!(out.contains("get drug <name> label [--raw]"));
}

#[test]
fn list_disease_mentions_opt_in_sections() {
    let out = render(Some("disease")).expect("list disease should render");
    assert!(out.contains("## When to use this surface"));
    assert_eq!(out.matches("## When to use this surface").count(), 1);
    assert!(
        out.contains("Use `get disease <name_or_id>` when you want the normalized disease card")
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
        out.contains("`diagnostics`, `disgenet`, `funding`, and `clinical_features` stay opt-in")
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
    assert!(article.contains("Use keyword search to scan a topic before you know the entities."));
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
    assert!(article.contains("LitSense2-derived semantic signal and falls back to lexical ties"));
    assert!(article.contains("0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position"));
    assert!(article.contains("rows without LitSense2 provenance contribute `semantic=0`"));
    assert!(article.contains(
        "Cap each federated source's contribution after deduplication and before ranking."
    ));
    assert!(article.contains(
        "Default: 40% of `--limit` on federated pools with at least three surviving primary sources."
    ));
    assert!(
        article
            .contains("`0` uses the default cap; setting it equal to `--limit` disables capping.")
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
        article
            .contains("Do not invent `-g/-d/--drug`; stay keyword-first or start with `discover`")
    );
    assert!(article.contains("biomcp search article -g BRAF --limit 5"));
    assert!(
        article
            .contains("biomcp search article -g TP53 -k \"apoptosis gene regulation\" --limit 5")
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
    assert!(
        out.contains(
            "get pathway <id> all` - include all sections supported by that pathway source"
        )
    );
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
