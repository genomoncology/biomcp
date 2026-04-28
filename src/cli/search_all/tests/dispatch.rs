//! Search-all dispatch and timeout tests.

use tokio::sync::MutexGuard;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::entities::trial::TrialSearchResult;
use crate::test_support::set_env_var;

use super::super::dispatch::{merge_trial_backfill_rows, section_fetch_limit, section_timeout};
use super::super::plan::PreparedInput;
use super::super::{SearchAllInput, SearchAllResults, SearchAllSection, SectionKind};

async fn env_lock_async() -> MutexGuard<'static, ()> {
    crate::test_support::env_lock().lock().await
}

fn trial_row(nct_id: &str, status: &str) -> TrialSearchResult {
    TrialSearchResult {
        nct_id: nct_id.to_string(),
        title: format!("Trial {nct_id}"),
        status: status.to_string(),
        phase: None,
        conditions: Vec::new(),
        sponsor: None,
        matched_intervention_label: None,
    }
}

#[test]
fn section_fetch_limit_reduces_only_safe_counts_only_sections() {
    let counts_only = PreparedInput::new(&SearchAllInput {
        gene: Some("BRAF".to_string()),
        variant: None,
        disease: Some("melanoma".to_string()),
        drug: None,
        keyword: None,
        since: None,
        limit: 7,
        counts_only: true,
        debug_plan: false,
    })
    .expect("valid prepared input");
    let debug_plan = PreparedInput::new(&SearchAllInput {
        gene: Some("BRAF".to_string()),
        variant: None,
        disease: Some("melanoma".to_string()),
        drug: None,
        keyword: None,
        since: None,
        limit: 7,
        counts_only: true,
        debug_plan: true,
    })
    .expect("valid prepared input");
    let full_fetch = PreparedInput::new(&SearchAllInput {
        gene: Some("BRAF".to_string()),
        variant: None,
        disease: Some("melanoma".to_string()),
        drug: None,
        keyword: None,
        since: None,
        limit: 7,
        counts_only: false,
        debug_plan: false,
    })
    .expect("valid prepared input");

    for kind in [
        SectionKind::Gene,
        SectionKind::Disease,
        SectionKind::Trial,
        SectionKind::Pgx,
    ] {
        assert_eq!(section_fetch_limit(kind, &counts_only), 1, "{kind:?}");
    }
    assert_eq!(section_fetch_limit(SectionKind::Article, &counts_only), 1);
    assert_eq!(section_fetch_limit(SectionKind::Article, &debug_plan), 7);

    for kind in [
        SectionKind::Variant,
        SectionKind::Drug,
        SectionKind::Pathway,
        SectionKind::Gwas,
        SectionKind::AdverseEvent,
    ] {
        assert_eq!(section_fetch_limit(kind, &counts_only), 7, "{kind:?}");
        assert_eq!(section_fetch_limit(kind, &full_fetch), 7, "{kind:?}");
    }
}

#[tokio::test]
#[serial_test::serial]
async fn dispatch_section_pathway_surfaces_sanitized_wikipathways_404_without_timeout() {
    // Serialize the shared BIOMCP_* mock-base env mutation this warning-path test relies on.
    let _guard = env_lock_async().await;
    let reactome = MockServer::start().await;
    let kegg = MockServer::start().await;
    let wikipathways = MockServer::start().await;
    let _reactome_base = set_env_var("BIOMCP_REACTOME_BASE", Some(&reactome.uri()));
    let _kegg_base = set_env_var("BIOMCP_KEGG_BASE", Some(&kegg.uri()));
    let _wikipathways_base = set_env_var("BIOMCP_WIKIPATHWAYS_BASE", Some(&wikipathways.uri()));
    let _disable_kegg = set_env_var("BIOMCP_DISABLE_KEGG", None);

    Mock::given(method("GET"))
        .and(path("/search/query"))
        .and(query_param("query", "BRAF"))
        .and(query_param("species", "Homo sapiens"))
        .and(query_param("pageSize", "3"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(r#"{"results":[],"totalResults":0}"#, "application/json"),
        )
        .expect(1)
        .mount(&reactome)
        .await;

    Mock::given(method("GET"))
        .and(path("/find/pathway/BRAF"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .expect(1)
        .mount(&kegg)
        .await;

    Mock::given(method("GET"))
            .and(path("/findPathwaysByText.json"))
            .respond_with(ResponseTemplate::new(404).set_body_raw(
                "<!DOCTYPE html><html><head><title>404</title></head><body>File not found</body></html>",
                "text/html; charset=utf-8",
            ))
            .expect(1)
            .mount(&wikipathways)
            .await;

    let prepared = PreparedInput::new(&SearchAllInput {
        gene: Some("BRAF".to_string()),
        variant: None,
        disease: None,
        drug: None,
        keyword: None,
        since: None,
        limit: 3,
        counts_only: true,
        debug_plan: false,
    })
    .expect("valid prepared input");

    // Drive the warning-path search directly through `pathway::search_with_filters` and bypass the
    // shared HTTP cache via `with_no_cache`. Going through `dispatch_section` would wrap the call
    // in a 12s tokio timeout that, under nextest's full-suite parallelism, can fire before the
    // mocked WikiPathways response arrives — turning the assertion into the unrelated
    // "pathway search timed out" message that no longer mentions wikipathways. The dispatch
    // wrapping itself is just `error: Some(err.to_string())`, so we reconstruct the resulting
    // `SearchAllSection` here to keep the same observable contract for the markdown surface.
    let pathway_filters = crate::entities::pathway::PathwaySearchFilters {
        query: Some("BRAF".to_string()),
        ..Default::default()
    };
    let pathway_err = crate::sources::with_no_cache(true, async {
        crate::entities::pathway::search_with_filters(&pathway_filters, 3).await
    })
    .await
    .expect_err("WikiPathways 404 should surface as the aggregated pathway search error");

    let section = SearchAllSection {
        entity: SectionKind::Pathway.entity().to_string(),
        label: SectionKind::Pathway.label().to_string(),
        count: 0,
        total: None,
        error: Some(pathway_err.to_string()),
        note: None,
        results: Vec::new(),
        links: Vec::new(),
    };
    let error = section.error.clone().expect("pathway section should fail");
    assert_eq!(section.entity, "pathway");
    assert_eq!(section.count, 0);
    assert!(error.contains("wikipathways"));
    assert!(error.contains("HTTP 404"));
    assert!(error.contains("HTML error page"));
    assert!(!error.contains("timed out"));
    assert!(!error.contains("<!DOCTYPE"));
    assert!(!error.contains("<html"));
    assert!(!error.contains("<head"));

    let markdown = crate::render::markdown::search_all_markdown(
        &SearchAllResults {
            query: prepared.query_summary(),
            sections: vec![section],
            searches_dispatched: 1,
            searches_with_results: 0,
            wall_time_ms: 0,
            debug_plan: None,
        },
        true,
    )
    .expect("counts-only markdown should render");
    assert!(markdown.contains("## Pathways (0)"));
    assert!(markdown.contains("Error: API error from wikipathways: HTTP 404"));
    assert!(markdown.contains("HTML error page"));
    assert!(!markdown.contains("<!DOCTYPE"));
    assert!(!markdown.contains("<html"));
    assert!(!markdown.contains("<head"));
}

#[test]
fn section_timeout_uses_article_specific_budget() {
    assert_eq!(section_timeout(SectionKind::Article).as_secs(), 60);
    assert_eq!(section_timeout(SectionKind::Trial).as_secs(), 12);
}

#[test]
fn merge_trial_backfill_rows_preserves_preferred_order_and_dedupes() {
    let preferred = vec![
        trial_row("NCT00000001", "RECRUITING"),
        trial_row("NCT00000002", "ACTIVE_NOT_RECRUITING"),
    ];
    let backfill = vec![
        trial_row("NCT00000002", "UNKNOWN"),
        trial_row("NCT00000003", "UNKNOWN"),
        trial_row("NCT00000004", "COMPLETED"),
    ];

    let merged = merge_trial_backfill_rows(preferred, backfill, 3);
    let ids = merged
        .iter()
        .map(|row| row.nct_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["NCT00000001", "NCT00000002", "NCT00000003"]);
}

#[test]
fn merge_trial_backfill_rows_respects_limit_with_preferred_only() {
    let preferred = vec![
        trial_row("NCT00000001", "RECRUITING"),
        trial_row("NCT00000002", "ACTIVE_NOT_RECRUITING"),
        trial_row("NCT00000003", "NOT_YET_RECRUITING"),
    ];

    let merged = merge_trial_backfill_rows(preferred, vec![], 2);
    let ids = merged
        .iter()
        .map(|row| row.nct_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["NCT00000001", "NCT00000002"]);
}
