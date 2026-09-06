use super::super::{
    OutputStream, PaginationMeta, alias_suggestion_outcome, extract_json_from_sections,
    render_batch_json, resolve_query_input, search_json, search_json_with_meta,
    search_json_with_meta_and_suggestions, search_meta, search_meta_with_suggestions,
    search_meta_with_workflow,
};
use crate::cli::shared::render_human_clap_error;
use crate::cli::{execute, sanitize_human_diagnostic, try_parse_cli};
use crate::entities::discover::{
    AliasAmbiguity, AliasCandidateSummary, AliasCanonicalMatch, AliasFallbackDecision,
    DiscoverConfidence, DiscoverType, MatchTier,
};

#[tokio::test]
async fn formerly_plain_finite_commands_emit_one_json_document() {
    let rows: &[&[&str]] = &[
        &["biomcp", "--json", "skill"],
        &["biomcp", "--json", "skill", "render"],
        &["biomcp", "--json", "chart"],
        &[
            "biomcp",
            "--json",
            "mcp-config",
            "--client",
            "claude-desktop",
        ],
        &["biomcp", "--json", "cache", "path"],
    ];
    for args in rows {
        let cli = try_parse_cli(*args).expect("matrix command must parse");
        let outcome = crate::cli::run_outcome(cli)
            .await
            .expect("matrix command must execute");
        assert_eq!(outcome.exit_code, 0, "{args:?}");
        serde_json::from_str::<serde_json::Value>(&outcome.text)
            .unwrap_or_else(|error| panic!("{args:?} did not emit one JSON value: {error}"));
    }
}

#[test]
fn long_running_server_json_rejection_is_structured_and_nonzero() {
    let outcome = crate::cli::server_json_rejection();
    assert_ne!(outcome.exit_code, 0);
    let value: serde_json::Value = serde_json::from_str(&outcome.text).unwrap();
    assert_eq!(value["error"]["code"], "invalid_argument");
}

#[test]
fn clap_diagnostics_remove_terminal_controls_from_rejected_arguments() {
    let args = ["biomcp", "--bad\u{9b}31m\u{202e}\nforged-line"];
    let error = try_parse_cli(args).expect_err("unknown argument should fail");
    let rendered = render_human_clap_error(&error, &args.map(std::ffi::OsString::from));

    assert_eq!(error.exit_code(), 2);
    assert!(rendered.contains("--bad forged-line"));
    assert!(!rendered.contains(['\u{9b}', '\u{202e}']));
    assert!(!rendered.contains("--bad\nforged-line"));
}

#[cfg(unix)]
#[test]
fn clap_diagnostics_sanitize_layout_controls_in_non_utf8_arguments() {
    use std::os::unix::ffi::OsStringExt as _;

    let args = vec![
        std::ffi::OsString::from("biomcp"),
        std::ffi::OsString::from_vec(b"--bad\xff\nforged-line".to_vec()),
    ];
    let error = try_parse_cli(args.clone()).expect_err("unknown argument should fail");
    let rendered = render_human_clap_error(&error, &args);

    assert!(!rendered.contains("\nforged-line'"));
}

#[tokio::test]
async fn reflected_study_identifier_is_sanitized_at_the_human_diagnostic_boundary() {
    let error = execute(vec![
        "biomcp".into(),
        "study".into(),
        "download".into(),
        "bad\u{1b}[31m\u{202e}/id".into(),
    ])
    .await
    .expect_err("path-like study ID should fail before network access");
    let rendered = sanitize_human_diagnostic(&error.to_string());

    assert!(rendered.contains("bad/id"));
    assert!(!rendered.contains(['\u{1b}', '\u{202e}']));
}

#[test]
fn extract_json_from_sections_detects_trailing_long_flag() {
    let sections = vec!["all".to_string(), "--json".to_string()];
    let (cleaned, json_override) = extract_json_from_sections(&sections);
    assert_eq!(cleaned, vec!["all".to_string()]);
    assert!(json_override);
}

#[test]
fn extract_json_from_sections_detects_trailing_short_flag() {
    let sections = vec!["clinvar".to_string(), "-j".to_string()];
    let (cleaned, json_override) = extract_json_from_sections(&sections);
    assert_eq!(cleaned, vec!["clinvar".to_string()]);
    assert!(json_override);
}

#[test]
fn extract_json_from_sections_keeps_regular_sections() {
    let sections = vec!["eligibility".to_string(), "locations".to_string()];
    let (cleaned, json_override) = extract_json_from_sections(&sections);
    assert_eq!(cleaned, sections);
    assert!(!json_override);
}

#[test]
fn resolve_query_input_accepts_flag_or_positional() {
    let from_flag = resolve_query_input(Some("BRAF".into()), None, "--query").unwrap();
    assert_eq!(from_flag.as_deref(), Some("BRAF"));

    let from_positional = resolve_query_input(None, Some("melanoma".into()), "--query").unwrap();
    assert_eq!(from_positional.as_deref(), Some("melanoma"));
}

#[test]
fn resolve_query_input_rejects_dual_values() {
    let err = resolve_query_input(Some("BRAF".into()), Some("TP53".into()), "--query").unwrap_err();
    assert!(format!("{err}").contains("Use either positional QUERY or --query, not both"));

    let err_gene =
        resolve_query_input(Some("TP53".into()), Some("BRAF".into()), "--gene").unwrap_err();
    assert!(format!("{err_gene}").contains("Use either positional QUERY or --gene, not both"));
}

#[test]
fn phenotype_search_json_contract_unchanged() {
    let pagination = PaginationMeta::offset(0, 1, 1, Some(1));
    let json = search_json(
        vec![crate::entities::disease::PhenotypeSearchResult {
            disease_id: "MONDO:0100135".to_string(),
            disease_name: "Dravet syndrome".to_string(),
            score: 15.036,
            direct_support: vec![crate::entities::disease::PhenotypeDirectSupport {
                hpo_id: "HP:0001250".into(),
                status: crate::entities::disease::PhenotypeDirectSupportStatus::Supported,
            }],
        }],
        pagination,
    )
    .expect("phenotype search json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["count"], 1);
    assert_eq!(value["results"][0]["disease_id"], "MONDO:0100135");
    assert_eq!(value["results"][0]["disease_name"], "Dravet syndrome");
    assert!(
        value.get("_meta").is_none(),
        "generic search json should not grow entity-style _meta"
    );
}

#[test]
fn search_meta_trims_empty_commands() {
    let meta = search_meta(vec![
        " biomcp get gene BRAF ".to_string(),
        String::new(),
        "   ".to_string(),
        "biomcp list gene".to_string(),
    ])
    .expect("search meta should be present");

    let value = serde_json::to_value(meta).expect("meta json");
    assert_eq!(
        value["next_commands"][0],
        serde_json::Value::String("biomcp get gene BRAF".into())
    );
    assert_eq!(
        value["next_commands"][1],
        serde_json::Value::String("biomcp list gene".into())
    );
    assert!(value.get("suggestions").is_none());
}

#[test]
fn search_meta_with_suggestions_keeps_empty_suggestions_array() {
    let meta = search_meta_with_suggestions(
        vec!["biomcp get article 12345".to_string()],
        Some(vec![String::new(), "   ".to_string()]),
    )
    .expect("search meta should be present");

    let value = serde_json::to_value(meta).expect("meta json");
    assert_eq!(value["suggestions"].as_array().map(Vec::len), Some(0));
}

#[test]
fn search_meta_with_workflow_keeps_meta_without_next_commands() {
    let meta = search_meta_with_workflow(
        Vec::new(),
        None,
        Some(crate::workflow_ladders::WorkflowMeta {
            workflow: "demo-workflow".to_string(),
            rationale: "A deterministic demo workflow.".to_string(),
            playbook: "biomcp skill demo-workflow".to_string(),
        }),
    )
    .expect("workflow metadata should force _meta");

    let value = serde_json::to_value(meta).expect("meta json");
    assert_eq!(value["next_commands"], serde_json::json!([]));
    assert_eq!(value["workflow"], "demo-workflow");
    assert_eq!(
        value["workflow_rationale"],
        "A deterministic demo workflow."
    );
    assert_eq!(value["workflow_playbook"], "biomcp skill demo-workflow");
    assert!(value.get("ladder").is_none());
}

#[test]
fn search_json_with_meta_includes_next_commands() {
    let pagination = PaginationMeta::offset(0, 1, 1, Some(1));
    let json = search_json_with_meta(
        vec![crate::entities::gene::GeneSearchResult {
            symbol: "BRAF".to_string(),
            name: "B-Raf proto-oncogene".to_string(),
            entrez_id: "673".to_string(),
            genomic_coordinates: None,
            uniprot_id: None,
            omim_id: None,
        }],
        pagination,
        vec![
            "biomcp get gene BRAF".to_string(),
            "biomcp list gene".to_string(),
        ],
    )
    .expect("search json with meta");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["count"], 1);
    assert_eq!(
        value["_meta"]["next_commands"][0],
        serde_json::Value::String("biomcp get gene BRAF".into())
    );
    assert_eq!(
        value["_meta"]["next_commands"][1],
        serde_json::Value::String("biomcp list gene".into())
    );
}

#[test]
fn search_json_with_meta_and_suggestions_includes_zero_result_suggestions() {
    let pagination = PaginationMeta::offset(0, 5, 0, Some(0));
    let json = search_json_with_meta_and_suggestions::<serde_json::Value>(
        Vec::new(),
        pagination,
        Vec::new(),
        Some(vec!["biomcp list diagnostic".to_string()]),
    )
    .expect("search json with zero-result suggestions");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["count"], 0);
    assert_eq!(value["results"], serde_json::json!([]));
    assert_eq!(value["_meta"]["next_commands"], serde_json::json!([]));
    assert_eq!(
        value["_meta"]["suggestions"],
        serde_json::json!(["biomcp list diagnostic"])
    );
}

#[test]
fn search_json_with_meta_omits_meta_when_empty() {
    let pagination = PaginationMeta::offset(0, 1, 1, Some(1));
    let json = search_json_with_meta(
        vec![crate::entities::gene::GeneSearchResult {
            symbol: "BRAF".to_string(),
            name: "B-Raf proto-oncogene".to_string(),
            entrez_id: "673".to_string(),
            genomic_coordinates: None,
            uniprot_id: None,
            omim_id: None,
        }],
        pagination,
        vec![String::new(), "   ".to_string()],
    )
    .expect("search json with empty meta");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert!(value.get("_meta").is_none());
}

fn canonical_alias_decision(
    requested_entity: DiscoverType,
    query: &str,
    canonical: &str,
    canonical_id: &str,
    command: &str,
) -> AliasFallbackDecision {
    AliasFallbackDecision::Canonical(AliasCanonicalMatch {
        requested_entity,
        query: query.to_string(),
        canonical: canonical.to_string(),
        canonical_id: canonical_id.to_string(),
        confidence: DiscoverConfidence::CanonicalId,
        match_tier: MatchTier::Exact,
        sources: vec!["OLS4".to_string()],
        next_commands: vec![command.to_string()],
    })
}

fn ambiguous_gene_decision() -> AliasFallbackDecision {
    AliasFallbackDecision::Ambiguous(AliasAmbiguity {
        requested_entity: DiscoverType::Gene,
        query: "V600E".to_string(),
        candidates: vec![AliasCandidateSummary {
            label: "V600E".to_string(),
            primary_type: DiscoverType::Variant,
            primary_id: Some("SO:0001583".to_string()),
            confidence: DiscoverConfidence::CanonicalId,
            match_tier: MatchTier::Exact,
        }],
        next_commands: vec![
            "biomcp discover V600E".to_string(),
            "biomcp search gene -q V600E".to_string(),
        ],
    })
}

#[test]
fn gene_alias_fallback_returns_exit_1_markdown_suggestion() {
    let decision = canonical_alias_decision(
        DiscoverType::Gene,
        "ERBB1",
        "EGFR",
        "HGNC:3236",
        "biomcp get gene EGFR",
    );
    let outcome = alias_suggestion_outcome("ERBB1", DiscoverType::Gene, &decision, false)
        .expect("alias outcome");

    assert_eq!(outcome.stream, OutputStream::Stderr);
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.text.contains("Error: gene 'ERBB1' not found."));
    assert!(
        outcome
            .text
            .contains("Did you mean: `biomcp get gene EGFR`")
    );
}

#[test]
fn gene_alias_fallback_json_writes_stdout_and_exit_1() {
    let decision = canonical_alias_decision(
        DiscoverType::Gene,
        "ERBB1",
        "EGFR",
        "HGNC:3236",
        "biomcp get gene EGFR",
    );
    let outcome = alias_suggestion_outcome("ERBB1", DiscoverType::Gene, &decision, true)
        .expect("alias json outcome");

    assert_eq!(outcome.stream, OutputStream::Stdout);
    assert_eq!(outcome.exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&outcome.text).expect("valid alias json");
    assert_eq!(
        value["_meta"]["alias_resolution"]["canonical"], "EGFR",
        "json={value}"
    );
    assert_eq!(value["_meta"]["next_commands"][0], "biomcp get gene EGFR");
}

fn gene_fixture(symbol: &str, name: &str, entrez_id: &str) -> crate::entities::gene::Gene {
    crate::entities::gene::Gene {
        section_outcomes: Default::default(),
        symbol: symbol.to_string(),
        name: name.to_string(),
        entrez_id: entrez_id.to_string(),
        ensembl_id: None,
        location: None,
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: None,
        summary: None,
        gene_type: None,
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: None,
        go: None,
        interactions: None,
        civic: None,
        expression: None,
        hpa: None,
        druggability: None,
        clingen: None,
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    }
}

#[test]
fn batch_gene_json_includes_meta_per_item() {
    let genes = vec![
        gene_fixture("BRAF", "B-Raf proto-oncogene", "673"),
        gene_fixture("TP53", "tumor protein p53", "7157"),
    ];
    let output = render_batch_json(&genes, |gene| {
        crate::render::json::to_entity_json_value(
            gene,
            vec![(
                "MyGene.info",
                format!("https://mygene.info/v3/gene/{}", gene.entrez_id),
            )],
            vec![format!("biomcp get gene {}", gene.symbol)],
            vec![crate::render::provenance::SectionSource {
                outcome: crate::entities::section_outcome::SectionOutcomeState::Data,
                key: "identity".to_string(),
                label: "Identity".to_string(),
                sources: vec!["NCBI Gene / MyGene.info".to_string()],
            }],
        )
    })
    .expect("batch json");
    let value: serde_json::Value = serde_json::from_str(&output).expect("valid batch json");
    let items = value.as_array().expect("batch root should stay an array");
    assert_eq!(items.len(), 2, "json={value}");
    assert_eq!(items[0]["symbol"], "BRAF", "json={value}");
    assert_eq!(items[1]["symbol"], "TP53", "json={value}");
    assert!(
        items.iter().all(|item| item["_meta"]["evidence_urls"]
            .as_array()
            .is_some_and(|urls| !urls.is_empty())),
        "each batch item should include non-empty _meta.evidence_urls: {value}"
    );
    assert!(
        items.iter().all(|item| item["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|cmds| !cmds.is_empty())),
        "each batch item should include non-empty _meta.next_commands: {value}"
    );
    assert!(
        items.iter().all(|item| item["_meta"]["section_sources"]
            .as_array()
            .is_some_and(|sources| !sources.is_empty())),
        "each batch item should include non-empty _meta.section_sources: {value}"
    );
}

#[test]
fn ambiguous_gene_miss_points_to_discover() {
    let decision = ambiguous_gene_decision();
    let outcome = alias_suggestion_outcome("V600E", DiscoverType::Gene, &decision, false)
        .expect("ambiguous outcome");

    assert_eq!(outcome.stream, OutputStream::Stderr);
    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome
            .text
            .contains("BioMCP could not map 'V600E' to a single gene.")
    );
    assert!(outcome.text.contains("1. biomcp discover V600E"));
    assert!(outcome.text.contains("2. biomcp search gene -q V600E"));
}

#[test]
fn alias_fallback_ols_failure_preserves_original_not_found() {
    let err = crate::error::BioMcpError::NotFound {
        entity: "gene".to_string(),
        id: "ERBB1".to_string(),
        suggestion: "Try searching: biomcp search gene -q ERBB1".to_string(),
    };
    let rendered = err.to_string();

    assert!(rendered.contains("gene 'ERBB1' not found"));
    assert!(rendered.contains("Try searching: biomcp search gene -q ERBB1"));
}

#[test]
fn drug_alias_fallback_returns_exit_1_markdown_suggestion() {
    let decision = canonical_alias_decision(
        DiscoverType::Drug,
        "Keytruda",
        "pembrolizumab",
        "MESH:C582435",
        "biomcp get drug pembrolizumab",
    );
    let outcome = alias_suggestion_outcome("Keytruda", DiscoverType::Drug, &decision, false)
        .expect("drug alias outcome");

    assert_eq!(outcome.stream, OutputStream::Stderr);
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.text.contains("Error: drug 'Keytruda' not found."));
    assert!(
        outcome
            .text
            .contains("Did you mean: `biomcp get drug pembrolizumab`")
    );
}

#[test]
fn drug_alias_fallback_json_writes_stdout_and_exit_1() {
    let decision = canonical_alias_decision(
        DiscoverType::Drug,
        "Keytruda",
        "pembrolizumab",
        "MESH:C582435",
        "biomcp get drug pembrolizumab",
    );
    let outcome = alias_suggestion_outcome("Keytruda", DiscoverType::Drug, &decision, true)
        .expect("drug alias json outcome");

    assert_eq!(outcome.stream, OutputStream::Stdout);
    assert_eq!(outcome.exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&outcome.text).expect("valid alias json");
    assert_eq!(
        value["_meta"]["alias_resolution"]["canonical"],
        "pembrolizumab"
    );
    assert_eq!(
        value["_meta"]["next_commands"][0],
        "biomcp get drug pembrolizumab"
    );
}

#[test]
fn mcp_alias_suggestion_json_stays_structured() {
    let decision = canonical_alias_decision(
        DiscoverType::Gene,
        "ERBB1",
        "EGFR",
        "HGNC:3236",
        "biomcp get gene EGFR",
    );
    let outcome = alias_suggestion_outcome("ERBB1", DiscoverType::Gene, &decision, true)
        .expect("mcp alias outcome");

    let value: serde_json::Value =
        serde_json::from_str(&outcome.text).expect("valid mcp alias json");
    assert_eq!(value["_meta"]["alias_resolution"]["kind"], "canonical");
    assert_eq!(value["_meta"]["alias_resolution"]["canonical"], "EGFR");
}
