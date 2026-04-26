//! Search-all formatting and refinement tests.

use serde_json::json;

use crate::cli::debug_plan::{DebugPlan, DebugPlanLeg};
use crate::entities::article::{
    ArticleRankingMetadata, ArticleRankingMode, ArticleSearchResult, ArticleSource,
};
use crate::entities::drug::DrugSearchResult;
use crate::entities::variant::VariantGwasAssociation;

use super::super::format::{
    dedupe_gwas_rows, format_search_all_p_value, refine_drug_results, to_json_array,
    variant_significance_rank,
};
use super::super::{SearchAllLink, SearchAllResults, SearchAllSection, counts_only_json};

#[test]
fn to_json_array_preserves_article_source_and_ranking_metadata() {
    let rows = to_json_array(vec![ArticleSearchResult {
        pmid: "22663011".into(),
        pmcid: Some("PMC9984800".into()),
        doi: Some("10.1056/NEJMoa1203421".into()),
        title: "BRAF melanoma review".into(),
        journal: Some("Journal".into()),
        date: Some("2025-01-01".into()),
        first_index_date: None,
        citation_count: Some(12),
        influential_citation_count: Some(4),
        source: ArticleSource::EuropePmc,
        matched_sources: vec![ArticleSource::EuropePmc, ArticleSource::SemanticScholar],
        score: None,
        is_retracted: Some(false),
        abstract_snippet: Some("Abstract".into()),
        ranking: Some(ArticleRankingMetadata {
            directness_tier: 3,
            anchor_count: 2,
            title_anchor_hits: 2,
            abstract_anchor_hits: 0,
            combined_anchor_hits: 2,
            all_anchors_in_title: true,
            all_anchors_in_text: true,
            study_or_review_cue: true,
            pubmed_rescue: false,
            pubmed_rescue_kind: None,
            pubmed_source_position: None,
            mode: Some(ArticleRankingMode::Lexical),
            semantic_score: None,
            lexical_score: None,
            citation_score: None,
            position_score: None,
            composite_score: None,
            avg_source_rank: None,
        }),
        normalized_title: "braf melanoma review".into(),
        normalized_abstract: "abstract".into(),
        publication_type: Some("Review".into()),
        source_local_position: 0,
    }])
    .expect("article rows should serialize");

    assert_eq!(
        rows[0]["source"],
        serde_json::Value::String("europepmc".into())
    );
    assert_eq!(
        rows[0]["matched_sources"][1],
        serde_json::Value::String("semanticscholar".into())
    );
    assert_eq!(rows[0]["ranking"]["study_or_review_cue"], true);
    assert_eq!(rows[0]["ranking"]["pubmed_rescue"], false);
    assert!(rows[0]["ranking"]["pubmed_rescue_kind"].is_null());
    assert!(rows[0]["ranking"]["pubmed_source_position"].is_null());
}

#[test]
fn counts_only_json_projection_omits_results_links_and_total() {
    let results = SearchAllResults {
        query: "gene=BRAF".to_string(),
        sections: vec![SearchAllSection {
            entity: "gene".to_string(),
            label: "Genes".to_string(),
            count: 1,
            total: Some(12),
            error: None,
            note: Some("Counts-only projection".to_string()),
            results: vec![json!({"symbol":"BRAF"})],
            links: vec![SearchAllLink {
                rel: "get.top".to_string(),
                title: "Inspect BRAF".to_string(),
                command: "biomcp get gene BRAF".to_string(),
            }],
        }],
        searches_dispatched: 1,
        searches_with_results: 1,
        wall_time_ms: 42,
        debug_plan: None,
    };

    let value = serde_json::to_value(counts_only_json(&results)).expect("counts-only json");
    let section = &value["sections"][0];

    assert_eq!(section["entity"], "gene");
    assert_eq!(section["label"], "Genes");
    assert_eq!(section["count"], 12);
    assert_eq!(section["note"], "Counts-only projection");
    assert!(section.get("results").is_none());
    assert!(section.get("links").is_none());
    assert!(section.get("total").is_none());
}

#[test]
fn counts_only_json_projection_preserves_debug_plan() {
    let results = SearchAllResults {
        query: "gene=BRAF".to_string(),
        sections: vec![SearchAllSection {
            entity: "article".to_string(),
            label: "Articles".to_string(),
            count: 1,
            total: Some(5),
            error: None,
            note: None,
            results: vec![json!({"pmid":"22663011"})],
            links: Vec::new(),
        }],
        searches_dispatched: 1,
        searches_with_results: 1,
        wall_time_ms: 42,
        debug_plan: Some(DebugPlan {
            surface: "search_all",
            query: "gene=BRAF".to_string(),
            anchor: Some("gene"),
            legs: vec![DebugPlanLeg {
                leg: "article".to_string(),
                entity: "article".to_string(),
                filters: vec!["gene=BRAF".to_string()],
                routing: vec!["anchor=gene".to_string()],
                sources: vec!["PubMed".to_string()],
                matched_sources: vec!["PubMed".to_string()],
                count: 1,
                total: Some(5),
                note: None,
                error: None,
            }],
        }),
    };

    let value = serde_json::to_value(counts_only_json(&results)).expect("counts-only json");
    assert!(value.get("debug_plan").is_some());
    assert_eq!(value["debug_plan"]["anchor"], "gene");
    assert_eq!(value["debug_plan"]["legs"][0]["leg"], "article");
}

fn gwas_row(rsid: &str, trait_name: Option<&str>, p_value: Option<f64>) -> VariantGwasAssociation {
    VariantGwasAssociation {
        rsid: rsid.to_string(),
        trait_name: trait_name.map(str::to_string),
        p_value,
        effect_size: None,
        effect_type: None,
        confidence_interval: None,
        risk_allele_frequency: None,
        risk_allele: None,
        mapped_genes: Vec::new(),
        study_accession: None,
        pmid: None,
        author: None,
        sample_description: None,
    }
}

fn drug_row(name: &str) -> DrugSearchResult {
    DrugSearchResult {
        name: name.to_string(),
        drugbank_id: None,
        drug_type: None,
        mechanism: None,
        target: None,
    }
}

#[test]
fn refine_drug_results_filters_metabolites_when_parent_like_match_exists() {
    let rows = vec![
        drug_row("desmethyl dabrafenib"),
        drug_row("dabrafenib mesylate"),
        drug_row("hydroxy dabrafenib"),
    ];
    let refined = refine_drug_results(rows, Some("dabrafenib"), 3);
    let names = refined.into_iter().map(|row| row.name).collect::<Vec<_>>();
    assert_eq!(names, vec!["dabrafenib mesylate"]);
}

#[test]
fn refine_drug_results_keeps_metabolites_when_no_parent_like_match() {
    let rows = vec![
        drug_row("desmethyl dabrafenib"),
        drug_row("hydroxy dabrafenib"),
    ];
    let refined = refine_drug_results(rows, Some("dabrafenib"), 3);
    assert_eq!(refined.len(), 2);
}

#[test]
fn variant_significance_rank_matches_clinical_priority() {
    assert!(
        variant_significance_rank(Some("Pathogenic"))
            < variant_significance_rank(Some("Likely pathogenic"))
    );
    assert!(
        variant_significance_rank(Some("Likely pathogenic"))
            < variant_significance_rank(Some("VUS"))
    );
    assert!(
        variant_significance_rank(Some("VUS")) < variant_significance_rank(Some("Likely benign"))
    );
    assert!(
        variant_significance_rank(Some("Likely benign"))
            < variant_significance_rank(Some("Benign"))
    );
}

#[test]
fn dedupe_gwas_rows_keeps_lowest_p_value() {
    let rows = vec![
        gwas_row("rs1", Some("melanoma"), Some(1e-5)),
        gwas_row("rs1", Some("melanoma"), Some(1e-7)),
        gwas_row("rs2", Some("melanoma"), Some(2e-6)),
    ];
    let deduped = dedupe_gwas_rows(rows);
    assert_eq!(deduped.len(), 2);
    let rs1 = deduped
        .iter()
        .find(|row| row.rsid == "rs1")
        .expect("rs1 row should remain");
    assert_eq!(rs1.p_value, Some(1e-7));
}

#[test]
fn format_search_all_p_value_removes_float_artifacts() {
    assert_eq!(
        format_search_all_p_value(&json!(6.000000000000001e-22)),
        "6e-22"
    );
    assert_eq!(format_search_all_p_value(&json!(0.005)), "0.005");
}
