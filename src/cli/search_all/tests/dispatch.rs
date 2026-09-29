//! Search-all dispatch and timeout tests.

use crate::entities::trial::TrialSearchHit;

use super::super::dispatch::{merge_trial_backfill_rows, section_fetch_limit, section_timeout};
use super::super::plan::PreparedInput;
use super::super::{
    SearchAllInput, SearchAllLink, SearchAllResults, SearchAllSection, SectionKind,
};

fn trial_row(nct_id: &str, status: &str) -> TrialSearchHit {
    TrialSearchHit::test(nct_id, status)
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
    assert_eq!(section_fetch_limit(SectionKind::Article, &debug_plan), 1);

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

#[test]
fn search_all_pathway_section_surfaces_sanitized_wikipathways_error() {
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

    let section = SearchAllSection {
        entity: SectionKind::Pathway.entity().to_string(),
        label: SectionKind::Pathway.label().to_string(),
        count: 0,
        total: None,
        error: Some("API error from wikipathways: HTTP 404; HTML error page".to_string()),
        note: None,
        results: Vec::new(),
        links: vec![SearchAllLink {
            rel: "search.retry".into(),
            title: "Retry pathway search".into(),
            command: "biomcp search pathway BRAF --limit 3".into(),
        }],
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
    assert!(markdown.contains("## Pathways (unknown)"));
    assert!(markdown.contains("Error: API error from wikipathways: HTTP 404"));
    assert!(markdown.contains("HTML error page"));
    assert!(markdown.contains("`biomcp search pathway BRAF --limit 3`"));
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
        .map(|row| row.nct_id().to_owned())
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
        .map(|row| row.nct_id().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["NCT00000001", "NCT00000002"]);
}

#[test]
fn trial_recruiting_filter_note_names_the_failure_and_the_widening() {
    let none = super::super::dispatch::trial_recruiting_filter_note(&None);
    assert!(none.is_none(), "no preferred failure carries no note");

    let err = crate::error::BioMcpError::Api {
        api: "ClinicalTrials.gov".to_string(),
        message: "HTTP 503 Service Unavailable: upstream outage html".to_string(),
    };
    let note = super::super::dispatch::trial_recruiting_filter_note(&Some(err)).expect("note");
    assert_eq!(
        note,
        "Recruiting-status filter unavailable for ClinicalTrials.gov. Results may include trials that are not recruiting.",
        "the note names the source in our words and states the widening"
    );
    assert!(
        !note.contains("503") && !note.contains("upstream outage html"),
        "raw upstream text must not paste into the note: {note}"
    );

    // An error without a recognized provider still names something plain
    // instead of pasting the raw error text.
    let opaque = crate::error::BioMcpError::Api {
        api: "not-a-real-source".to_string(),
        message: "secret internals must not leak".to_string(),
    };
    let note = super::super::dispatch::trial_recruiting_filter_note(&Some(opaque)).expect("note");
    assert!(
        !note.contains("secret internals"),
        "no upstream or internal text in the note: {note}"
    );
}

#[test]
fn a_degraded_trial_section_renders_the_note_and_drops_count_exact() {
    let prepared = PreparedInput::new(&SearchAllInput {
        gene: None,
        variant: None,
        disease: Some("melanoma".to_string()),
        drug: None,
        keyword: None,
        since: None,
        limit: 3,
        counts_only: true,
        debug_plan: false,
    })
    .expect("valid prepared input");

    let section = SearchAllSection {
        entity: SectionKind::Trial.entity().to_string(),
        label: SectionKind::Trial.label().to_string(),
        count: 3,
        total: Some(41),
        error: None,
        note: Some(
            "Recruiting-status filter unavailable: HTTP 503. Results may include trials that are not recruiting.".to_string(),
        ),
        results: Vec::new(),
        links: Vec::new(),
    };

    let value = serde_json::to_value(super::super::counts_only_json(&SearchAllResults {
        query: prepared.query_summary(),
        sections: vec![section.clone()],
        searches_dispatched: 1,
        searches_with_results: 1,
        wall_time_ms: 0,
        debug_plan: None,
    }))
    .expect("counts-only json");
    let json_section = &value["sections"][0];
    assert_eq!(json_section["total"], 41);
    // The note is a degradation: the surviving total is no longer exact.
    assert_eq!(json_section["count_exact"], false);
    assert!(json_section["total_lower_bound"].is_null());
    assert!(
        json_section["note"]
            .as_str()
            .is_some_and(|v| v.contains("may include trials that are not recruiting"))
    );

    let markdown = crate::render::markdown::search_all_markdown(
        &SearchAllResults {
            query: prepared.query_summary(),
            sections: vec![section],
            searches_dispatched: 1,
            searches_with_results: 1,
            wall_time_ms: 0,
            debug_plan: None,
        },
        true,
    )
    .expect("counts-only markdown should render");
    assert!(markdown.contains("Recruiting-status filter unavailable"));
    assert!(markdown.contains("may include trials that are not recruiting"));
}
