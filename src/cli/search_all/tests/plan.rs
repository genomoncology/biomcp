//! Search-all planning and debug-plan tests.

use serde_json::json;

use crate::entities::article::ArticleRankingMode;

use super::super::plan::{
    PreparedInput, article_filters, build_dispatch_plan_prepared, build_result_plan,
};
use super::super::{SearchAllInput, SearchAllSection, build_dispatch_plan};

fn input_with_gene() -> SearchAllInput {
    SearchAllInput {
        gene: Some("BRAF".to_string()),
        variant: None,
        disease: None,
        drug: None,
        keyword: None,
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    }
}

fn input_with_keyword(keyword: &str) -> SearchAllInput {
    SearchAllInput {
        gene: None,
        variant: None,
        disease: None,
        drug: None,
        keyword: Some(keyword.into()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    }
}

#[test]
fn build_dispatch_plan_gene_only_matches_contract() {
    let plan = build_dispatch_plan(&input_with_gene());
    let entities = plan.iter().map(|spec| spec.entity).collect::<Vec<_>>();
    assert_eq!(
        entities,
        vec![
            "gene", "variant", "drug", "trial", "article", "pathway", "pgx"
        ]
    );
}

#[test]
fn build_dispatch_plan_keyword_only_routes_to_article() {
    let plan = build_dispatch_plan(&SearchAllInput {
        gene: None,
        variant: None,
        disease: None,
        drug: None,
        keyword: Some("resistance".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    });
    let entities = plan.iter().map(|spec| spec.entity).collect::<Vec<_>>();
    assert_eq!(entities, vec!["article"]);
}

#[test]
fn preparation_rejects_article_owned_keyword_and_gene_shapes_before_fanout() {
    for (input, expected) in [
        (
            input_with_keyword("gene:RB1"),
            "Invalid argument: keyword is provider-neutral and does not accept gene: filter syntax. Use --gene RB1 for CLI or raw MCP, or the typed MCP field, for example \"gene\":\"RB1\".",
        ),
        (
            input_with_keyword("disease:melanoma"),
            "Invalid argument: keyword is provider-neutral and does not accept disease: filter syntax. Use --disease melanoma for CLI or raw MCP, or the typed MCP field, for example \"disease\":\"melanoma\".",
        ),
        (
            input_with_keyword("drug:vemurafenib"),
            "Invalid argument: keyword is provider-neutral and does not accept drug: filter syntax. Use --drug vemurafenib for CLI or raw MCP, or the typed MCP field, for example \"drug\":\"vemurafenib\".",
        ),
        (
            SearchAllInput {
                gene: Some("TPMT mercaptopurine".into()),
                ..input_with_gene()
            },
            "Invalid argument: gene accepts one symbol, for example TPMT. Put additional concepts in keyword: use --gene TPMT --keyword mercaptopurine for CLI or raw MCP, or typed MCP fields \"gene\":\"TPMT\" and \"keyword\":[\"mercaptopurine\"].",
        ),
        (
            SearchAllInput {
                gene: Some(" \u{a0} ".into()),
                ..input_with_gene()
            },
            "Invalid argument: gene accepts one symbol, for example TPMT. Put additional concepts in keyword: use --gene TPMT --keyword mercaptopurine for CLI or raw MCP, or typed MCP fields \"gene\":\"TPMT\" and \"keyword\":[\"mercaptopurine\"].",
        ),
    ] {
        let err =
            PreparedInput::new(&input).expect_err("invalid article input must fail preparation");
        assert_eq!(err.to_string(), expected);
    }
}

#[test]
fn preparation_keeps_literal_colons_and_valid_gene_tokens() {
    for keyword in [
        "NM_004333.6:c.1799T>A",
        "protein:protein interaction",
        "oncogene:RB1",
        "MYGENE:RB1",
        "ratio 1:2",
        "BRAF[variant]",
        "\"gene:gene interaction\"",
    ] {
        let prepared = PreparedInput::new(&input_with_keyword(keyword))
            .unwrap_or_else(|err| panic!("keyword {keyword:?} should prepare: {err}"));
        assert_eq!(build_dispatch_plan_prepared(&prepared).len(), 1);
    }

    for gene in ["BRAF", "braf", "PD-L1", "H3-3A", "  BRAF  "] {
        let prepared = PreparedInput::new(&SearchAllInput {
            gene: Some(gene.into()),
            ..input_with_gene()
        })
        .unwrap_or_else(|err| panic!("gene {gene:?} should prepare: {err}"));
        assert_eq!(prepared.gene.as_deref(), Some(gene.trim()));
        assert_eq!(build_dispatch_plan_prepared(&prepared).len(), 7);
    }
}

#[test]
fn article_filters_follow_keyword_dependent_ranking_defaults() {
    let keyword_prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: None,
        drug: None,
        keyword: Some("checkpoint inhibitor".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid prepared input");
    let keyword_filters = article_filters(&keyword_prepared);
    assert_eq!(
        crate::entities::article::article_effective_ranking_mode(&keyword_filters),
        Some(ArticleRankingMode::Hybrid)
    );

    let gene_prepared = PreparedInput::new(&input_with_gene()).expect("valid prepared input");
    let gene_filters = article_filters(&gene_prepared);
    assert_eq!(
        crate::entities::article::article_effective_ranking_mode(&gene_filters),
        Some(ArticleRankingMode::Lexical)
    );
}

#[test]
fn build_dispatch_plan_variant_with_gene_fanout() {
    let plan = build_dispatch_plan(&SearchAllInput {
        gene: None,
        variant: Some("BRAF V600E".to_string()),
        disease: None,
        drug: None,
        keyword: None,
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    });
    let entities = plan.iter().map(|spec| spec.entity).collect::<Vec<_>>();
    assert_eq!(
        entities,
        vec!["variant", "gene", "trial", "article", "drug", "pathway"]
    );
}

#[test]
fn prepared_input_rejects_empty_typed_slots() {
    let err = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: None,
        drug: None,
        keyword: None,
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: false,
    })
    .expect_err("expected validation error");
    assert!(err.to_string().contains("at least one typed slot"));
}

#[test]
fn build_result_plan_includes_fallback_and_article_matched_sources() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: Some("BRAF".to_string()),
        variant: None,
        disease: Some("melanoma".to_string()),
        drug: None,
        keyword: None,
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: true,
    })
    .expect("valid prepared input");
    let sections = vec![
        SearchAllSection {
            entity: "variant".to_string(),
            label: "Variants".to_string(),
            count: 1,
            total: Some(5),
            error: None,
            note: Some(
                "No disease-filtered variants found; showing top gene variants.".to_string(),
            ),
            results: vec![json!({"id":"rs113488022","gene":"BRAF"})],
            links: Vec::new(),
        },
        SearchAllSection {
            entity: "article".to_string(),
            label: "Articles".to_string(),
            count: 1,
            total: Some(10),
            error: None,
            note: None,
            results: vec![json!({
                "pmid": "22663011",
                "matched_sources": ["pubtator", "pubmed", "semanticscholar"]
            })],
            links: Vec::new(),
        },
    ];

    let plan = build_result_plan(&prepared, &sections);
    let article_leg = plan
        .legs
        .iter()
        .find(|leg| leg.leg == "article")
        .expect("article leg");

    assert_eq!(plan.surface, "search_all");
    assert_eq!(plan.anchor, Some("gene"));
    assert_eq!(plan.query, "gene=BRAF disease=melanoma");
    assert!(plan.legs[0].routing.contains(&"anchor=gene".to_string()));
    assert!(
        plan.legs[0]
            .routing
            .contains(&"fallback=gene_only_variant_backfill".to_string())
    );
    assert_eq!(
        article_leg.sources,
        vec![
            "PubTator3".to_string(),
            "Europe PMC".to_string(),
            "PubMed".to_string(),
            "Semantic Scholar".to_string()
        ]
    );
    assert_eq!(
        article_leg.matched_sources,
        vec![
            "PubTator3".to_string(),
            "PubMed".to_string(),
            "Semantic Scholar".to_string()
        ]
    );
}

#[test]
fn build_result_plan_keyword_article_leg_excludes_litsense2_default_source() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: None,
        drug: None,
        keyword: Some("Hirschsprung disease".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: true,
    })
    .expect("valid prepared input");
    let sections = vec![SearchAllSection {
        entity: "article".to_string(),
        label: "Articles".to_string(),
        count: 1,
        total: Some(10),
        error: None,
        note: None,
        results: vec![json!({
            "pmid": "36741595",
            "matched_sources": ["pubmed", "semanticscholar"]
        })],
        links: Vec::new(),
    }];

    let plan = build_result_plan(&prepared, &sections);
    let article_leg = plan
        .legs
        .iter()
        .find(|leg| leg.leg == "article")
        .expect("article leg");

    assert_eq!(plan.surface, "search_all");
    assert_eq!(plan.anchor, Some("keyword"));
    assert_eq!(plan.query, "keyword=Hirschsprung disease");
    assert!(!article_leg.sources.contains(&"LitSense2".to_string()));
    assert!(article_leg.sources.contains(&"PubMed".to_string()));
    assert_eq!(
        article_leg.matched_sources,
        vec!["PubMed".to_string(), "Semantic Scholar".to_string()]
    );
}

#[test]
fn build_result_plan_marks_shared_disease_keyword_orientation_fallback() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: Some("cancer".to_string()),
        drug: None,
        keyword: Some("Cancer".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: true,
    })
    .expect("valid prepared input");
    let sections = vec![SearchAllSection {
        entity: "article".to_string(),
        label: "Articles".to_string(),
        count: 1,
        total: None,
        error: None,
        note: None,
        results: vec![json!({"pmid":"1"})],
        links: Vec::new(),
    }];

    let plan = build_result_plan(&prepared, &sections);

    let article_leg = plan
        .legs
        .iter()
        .find(|l| l.leg == "article")
        .expect("article leg");
    assert!(
        article_leg
            .routing
            .contains(&"fallback=shared_disease_keyword_orientation".to_string()),
        "article leg routing should include shared-token fallback marker: {:?}",
        article_leg.routing
    );
    assert!(
        !article_leg.filters.contains(&"disease=cancer".to_string()),
        "article leg filters should drop the duplicate disease token: {:?}",
        article_leg.filters
    );
}

#[test]
fn build_result_plan_marks_ungrounded_disease_fallback_on_article_leg() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: Some("cancer".to_string()),
        drug: None,
        keyword: Some("cancer".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: true,
    })
    .expect("valid prepared input");
    let sections = vec![
        SearchAllSection {
            entity: "disease".to_string(),
            label: "Diseases".to_string(),
            count: 0,
            total: Some(0),
            error: None,
            note: None,
            results: vec![],
            links: Vec::new(),
        },
        SearchAllSection {
            entity: "article".to_string(),
            label: "Articles".to_string(),
            count: 1,
            total: Some(1),
            error: None,
            note: None,
            results: vec![json!({"pmid":"1"})],
            links: Vec::new(),
        },
    ];

    let plan = build_result_plan(&prepared, &sections);
    let article_leg = plan
        .legs
        .iter()
        .find(|l| l.leg == "article")
        .expect("article leg");
    assert!(
        article_leg
            .routing
            .contains(&"fallback=disease_leg_ungrounded_keyword_survived".to_string()),
        "article leg routing should note the ungrounded disease fallback: {:?}",
        article_leg.routing
    );
}

#[test]
fn build_result_plan_skips_ungrounded_marker_when_disease_leg_errors() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: Some("cancer".to_string()),
        drug: None,
        keyword: Some("cancer".to_string()),
        since: None,
        limit: 3,
        counts_only: false,
        debug_plan: true,
    })
    .expect("valid prepared input");
    let sections = vec![
        SearchAllSection {
            entity: "disease".to_string(),
            label: "Diseases".to_string(),
            count: 0,
            total: None,
            error: Some("upstream timeout".to_string()),
            note: None,
            results: vec![],
            links: Vec::new(),
        },
        SearchAllSection {
            entity: "article".to_string(),
            label: "Articles".to_string(),
            count: 1,
            total: Some(1),
            error: None,
            note: None,
            results: vec![json!({"pmid":"1"})],
            links: Vec::new(),
        },
    ];

    let plan = build_result_plan(&prepared, &sections);
    let article_leg = plan
        .legs
        .iter()
        .find(|l| l.leg == "article")
        .expect("article leg");
    assert!(
        !article_leg
            .routing
            .contains(&"fallback=disease_leg_ungrounded_keyword_survived".to_string()),
        "transport errors must not masquerade as ungrounded disease fallback: {:?}",
        article_leg.routing
    );
}
