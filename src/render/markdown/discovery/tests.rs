use super::*;

#[test]
fn search_all_markdown_renders_section_note() {
    let results = crate::cli::search_all::SearchAllResults {
        query: "gene=EGFR disease=non-small cell lung cancer".to_string(),
        sections: vec![crate::cli::search_all::SearchAllSection {
            entity: "variant".to_string(),
            label: "Variants".to_string(),
            count: 1,
            total: Some(1),
            error: None,
            note: Some(
                "No disease-filtered variants found; showing top gene variants.".to_string(),
            ),
            results: vec![serde_json::json!({
                "id": "rs121434568",
                "gene": "EGFR",
                "hgvs_p": "L858R",
                "significance": "Pathogenic",
            })],
            links: Vec::new(),
        }],
        searches_dispatched: 1,
        searches_with_results: 1,
        wall_time_ms: 42,
        debug_plan: None,
    };

    let markdown = search_all_markdown(&results, false).expect("markdown should render");
    assert!(markdown.contains("> No disease-filtered variants found; showing top gene variants."));
}

#[test]
fn search_all_markdown_counts_only_keeps_links_without_row_headers() {
    let results = crate::cli::search_all::SearchAllResults {
        query: "gene=BRAF".to_string(),
        sections: vec![crate::cli::search_all::SearchAllSection {
            entity: "trial".to_string(),
            label: "Trials".to_string(),
            count: 1,
            total: Some(12),
            error: None,
            note: None,
            results: vec![serde_json::json!({
                "nct_id": "NCT00000001",
                "title": "BRAF trial",
                "status": "RECRUITING",
            })],
            links: vec![crate::cli::search_all::SearchAllLink {
                rel: "cross.trials".to_string(),
                title: "Search trials".to_string(),
                command: "biomcp search trial --biomarker BRAF --limit 3".to_string(),
            }],
        }],
        searches_dispatched: 1,
        searches_with_results: 1,
        wall_time_ms: 42,
        debug_plan: None,
    };

    let markdown = search_all_markdown(&results, true).expect("counts-only markdown should render");
    assert!(markdown.contains("## Trials (12)"));
    assert!(markdown.contains("Rows omitted"));
    assert!(markdown.contains("biomcp search trial --biomarker BRAF --limit 3"));
    assert!(!markdown.contains("| NCT | Title | Status |"));
}

#[test]
fn ticket_377_discover_renderer_envelope_contracts() {
    let result = crate::entities::discover::DiscoverResult {
        query: "BRAF".to_string(),
        normalized_query: "braf".to_string(),
        concepts: vec![crate::entities::discover::DiscoverConcept {
            label: "BRAF".to_string(),
            primary_id: Some("HGNC:1097".to_string()),
            primary_type: DiscoverType::Gene,
            synonyms: vec!["B-raf".to_string()],
            xrefs: vec![crate::entities::discover::ConceptXref {
                source: "NCBI Gene".to_string(),
                id: "673".to_string(),
            }],
            sources: vec![crate::entities::discover::ConceptSource {
                source: "OLS4".to_string(),
                id: "HGNC:1097".to_string(),
                label: "BRAF".to_string(),
                source_type: "HGNC".to_string(),
            }],
            match_tier: crate::entities::discover::MatchTier::Exact,
            confidence: crate::entities::discover::DiscoverConfidence::CanonicalId,
        }],
        plain_language: None,
        next_commands: vec!["biomcp get gene BRAF".to_string()],
        notes: vec!["UMLS enrichment unavailable (set UMLS_API_KEY)".to_string()],
        ambiguous: false,
        intent: crate::entities::discover::DiscoverIntent::GeneFunction,
        offset: 0,
        limit: 5,
        returned: 1,
        has_more: false,
        next_offset: None,
        budget_truncated: false,
        malformed_candidates: 0,
        continuation_command: None,
        preview_meta: vec![crate::entities::discover::DiscoverConceptPreviewMeta::default()],
        full: false,
    };

    let json = crate::render::json::to_discover_json(&result).expect("discover JSON");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid discover JSON");
    assert_eq!(value["_meta"]["next_commands"][0], "biomcp get gene BRAF");
    assert_eq!(value["_meta"]["discovery_sources"][0], "OLS4");
    assert_eq!(
        value["_meta"]["section_sources"][0]["key"],
        "structured_concepts"
    );

    let markdown = render_discover(&result).expect("render_discover");
    assert!(markdown.contains("## Concepts"));
    assert!(markdown.contains("Sources: OLS4 (HGNC)"));
    assert!(markdown.contains("## Suggested Commands"));
    assert!(markdown.contains("UMLS enrichment unavailable"));
}

#[test]
fn render_discover_renders_grouped_concepts_and_plain_language() {
    let result = crate::entities::discover::DiscoverResult {
        query: "BRCA1".to_string(),
        normalized_query: "brca1".to_string(),
        concepts: vec![crate::entities::discover::DiscoverConcept {
            label: "BRCA1".to_string(),
            primary_id: Some("HGNC:1100".to_string()),
            primary_type: DiscoverType::Gene,
            synonyms: vec!["RNF53".to_string()],
            xrefs: vec![crate::entities::discover::ConceptXref {
                source: "NCBI Gene".to_string(),
                id: "672".to_string(),
            }],
            sources: vec![crate::entities::discover::ConceptSource {
                source: "HGNC".to_string(),
                id: "1100".to_string(),
                label: "BRCA1".to_string(),
                source_type: "canonical".to_string(),
            }],
            match_tier: crate::entities::discover::MatchTier::Exact,
            confidence: crate::entities::discover::DiscoverConfidence::CanonicalId,
        }],
        plain_language: Some(crate::entities::discover::PlainLanguageTopic {
            title: "BRCA1 mutation".to_string(),
            url: "https://example.org/brca1".to_string(),
            summary_excerpt: "Plain-language summary.".to_string(),
        }),
        next_commands: vec!["biomcp get gene BRCA1".to_string()],
        notes: vec!["Resolved via canonical symbol.".to_string()],
        ambiguous: false,
        intent: crate::entities::discover::DiscoverIntent::GeneFunction,
        offset: 0,
        limit: 5,
        returned: 1,
        has_more: false,
        next_offset: None,
        budget_truncated: false,
        malformed_candidates: 0,
        continuation_command: None,
        preview_meta: vec![crate::entities::discover::DiscoverConceptPreviewMeta::default()],
        full: false,
    };

    let markdown = render_discover(&result).expect("discover markdown");
    assert!(markdown.contains("# Discover: BRCA1") || markdown.contains("BRCA1"));
    assert!(markdown.contains("Resolved via canonical symbol."));
    assert!(markdown.contains("BRCA1 mutation"));
    assert!(markdown.contains("biomcp get gene BRCA1"));
}
