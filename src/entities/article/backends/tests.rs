use std::collections::HashSet;

use super::super::test_support::*;
use super::*;
use crate::sources::litsense2::LitSense2SearchHit;
use crate::sources::pubmed::{ESummaryEntry, PubMedClient};
use crate::sources::semantic_scholar::{
    SemanticScholarAuthMode, SemanticScholarExternalIds, SemanticScholarPaper,
    SemanticScholarSearchResponse,
};

fn query_value<'a>(query: &'a [(String, String)], key: &str) -> Option<&'a str> {
    query
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.as_str())
}

fn pubmed_entry(
    uid: &str,
    title: &str,
    journal: Option<&str>,
    date: Option<&str>,
) -> ESummaryEntry {
    ESummaryEntry {
        uid: uid.to_string(),
        title: title.to_string(),
        sortpubdate: date.map(str::to_string),
        pubdate: None,
        edat: None,
        lr: None,
        fulljournalname: journal.map(str::to_string),
        source: journal.map(str::to_string),
    }
}

fn collect_pubmed_rows(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    batches: Vec<Vec<ESummaryEntry>>,
) -> Result<Vec<ArticleSearchResult>, BioMcpError> {
    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    let mut skipped = 0;
    let mut source_position = 0;
    for batch in batches {
        append_pubmed_entries(
            batch,
            filters,
            None,
            None,
            limit,
            offset,
            PubMedAppendState {
                out: &mut rows,
                seen_pmids: &mut seen,
                visible_skipped: &mut skipped,
                source_position: &mut source_position,
            },
        )?;
    }
    Ok(rows)
}

fn lit_hit(pmid: u64, text: &str, score: f64, pmcid: Option<&str>) -> LitSense2SearchHit {
    LitSense2SearchHit {
        pmid,
        pmcid: pmcid.map(str::to_string),
        text: text.to_string(),
        score,
        section: None,
        annotations: Vec::new(),
    }
}

#[test]
fn search_pubmed_page_rejects_open_access() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.open_access = true;

    let err = super::super::query::build_pubmed_esearch_params(&filters, 5, 0)
        .expect_err("open-access should be rejected for PubMed page helper");

    assert!(err.to_string().contains("--open-access"));
    assert!(err.to_string().contains("PubMed"));
}

#[test]
fn search_pubmed_page_rejects_no_preprints() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.no_preprints = true;

    let err = super::super::query::build_pubmed_esearch_params(&filters, 5, 0)
        .expect_err("no-preprints should be rejected for PubMed page helper");

    assert!(err.to_string().contains("--no-preprints"));
    assert!(err.to_string().contains("PubMed"));
}

#[test]
fn search_pubmed_page_sends_standalone_not_retraction_term() {
    let mut filters = empty_filters();
    filters.gene = Some("WDR5".into());
    filters.exclude_retracted = true;

    let params =
        super::super::query::build_pubmed_esearch_params(&filters, 100, 0).expect("pubmed params");
    let plan = PubMedClient::esearch_plan(&params, None).expect("pubmed plan");

    assert_eq!(plan.path, "esearch.fcgi");
    assert_eq!(query_value(&plan.query, "db"), Some("pubmed"));
    assert_eq!(query_value(&plan.query, "retmode"), Some("json"));
    assert_eq!(query_value(&plan.query, "retstart"), Some("0"));
    assert_eq!(query_value(&plan.query, "retmax"), Some("100"));
    assert_eq!(
        query_value(&plan.query, "term"),
        Some("WDR5 NOT retracted publication[pt]")
    );
}

#[test]
fn search_pubmed_page_cleans_question_keyword_before_esearch() {
    let mut filters = empty_filters();
    filters.keyword = Some("What drug treatment can cause a spinal epidural hematoma?".into());

    let params =
        super::super::query::build_pubmed_esearch_params(&filters, 100, 0).expect("pubmed params");
    let plan = PubMedClient::esearch_plan(&params, None).expect("pubmed plan");

    assert_eq!(
        query_value(&plan.query, "term"),
        Some("drug treatment spinal epidural hematoma")
    );
}

#[test]
fn search_pubmed_page_refills_across_batches_after_filtering() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.journal = Some("Nature".into());

    let rows = collect_pubmed_rows(
        &filters,
        2,
        0,
        vec![
            vec![
                pubmed_entry(
                    "1",
                    "Filtered title",
                    Some("Other Journal"),
                    Some("2024/01/01 00:00"),
                ),
                pubmed_entry(
                    "2",
                    "First visible title",
                    Some("Nature"),
                    Some("2024/01/02 00:00"),
                ),
            ],
            vec![
                pubmed_entry(
                    "3",
                    "Second visible title",
                    Some("Nature"),
                    Some("2024/01/03 00:00"),
                ),
                pubmed_entry(
                    "4",
                    "Third visible title",
                    Some("Nature"),
                    Some("2024/01/04 00:00"),
                ),
            ],
        ],
    )
    .expect("pubmed rows");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].pmid, "2");
    assert_eq!(rows[1].pmid, "3");
}

#[test]
fn search_pubmed_page_applies_offset_after_filtering() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.journal = Some("Nature".into());

    let rows = collect_pubmed_rows(
        &filters,
        2,
        1,
        vec![
            vec![
                pubmed_entry(
                    "1",
                    "Filtered title",
                    Some("Other Journal"),
                    Some("2024/01/01 00:00"),
                ),
                pubmed_entry(
                    "2",
                    "First visible title",
                    Some("Nature"),
                    Some("2024/01/02 00:00"),
                ),
            ],
            vec![
                pubmed_entry(
                    "3",
                    "Second visible title",
                    Some("Nature"),
                    Some("2024/01/03 00:00"),
                ),
                pubmed_entry(
                    "4",
                    "Third visible title",
                    Some("Nature"),
                    Some("2024/01/04 00:00"),
                ),
            ],
        ],
    )
    .expect("pubmed rows");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].pmid, "3");
    assert_eq!(rows[1].pmid, "4");
    assert_eq!(rows[0].source_local_position, 1);
    assert_eq!(rows[1].source_local_position, 2);
}

#[test]
fn search_pubmed_page_hard_fails_on_blank_title() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());

    let err = collect_pubmed_rows(
        &filters,
        1,
        0,
        vec![vec![pubmed_entry(
            "1",
            "   ",
            Some("Nature"),
            Some("2024/01/01 00:00"),
        )]],
    )
    .expect_err("blank title should be a contract error");

    let msg = err.to_string();
    assert!(msg.contains("pubmed-eutils"));
    assert!(msg.contains("1"));
    assert!(msg.contains("title"));
}

#[test]
fn semantic_scholar_candidates_keep_unknown_retraction_rows() {
    let rows = semantic_scholar_rows_from_response(
        &ArticleSearchFilters {
            keyword: Some("alternative microexon splicing metastasis".into()),
            exclude_retracted: true,
            ..empty_filters()
        },
        None,
        None,
        SemanticScholarSearchResponse {
            total: Some(1),
            data: vec![SemanticScholarPaper {
                paper_id: Some("paper-1".into()),
                external_ids: Some(SemanticScholarExternalIds {
                    pubmed: Some("22663011".into()),
                    doi: Some("10.1000/example".into()),
                    arxiv: Some("2401.12345".into()),
                    ..Default::default()
                }),
                title: Some("Alternative microexon splicing in metastasis".into()),
                venue: Some("Cancer Cell".into()),
                year: Some(2025),
                citation_count: Some(12),
                influential_citation_count: Some(4),
                abstract_text: Some(
                    "Microexon splicing contributes to metastatic progression.".into(),
                ),
                ..Default::default()
            }],
        },
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, ArticleSource::SemanticScholar);
    assert_eq!(rows[0].arxiv_id.as_deref(), Some("2401.12345"));
    assert_eq!(rows[0].semantic_scholar_id.as_deref(), Some("paper-1"));
    assert_eq!(rows[0].is_retracted, None);
}

#[test]
fn ticket_376_article_source_status_contracts_semantic_scholar_unavailable_status_without_key() {
    let outcome = semantic_scholar_unavailable_outcome(SemanticScholarAuthMode::SharedPool);

    assert!(outcome.rows.is_empty());
    assert_eq!(outcome.status.source, ArticleSource::SemanticScholar);
    assert_eq!(
        outcome.status.auth_mode,
        Some(SemanticScholarAuthMode::SharedPool)
    );
    assert_eq!(
        outcome.status.status,
        Some(ArticleSourceAvailability::Unavailable)
    );
    assert!(
        outcome
            .status
            .message
            .as_deref()
            .is_some_and(|message| message.contains("unavailable"))
    );
}

#[test]
fn semantic_scholar_year_filter_uses_normalized_date_bounds() {
    assert_eq!(
        semantic_scholar_year_filter(Some("2000-01-01"), Some("2013-12-31")).as_deref(),
        Some("2000-2013")
    );
    assert_eq!(
        semantic_scholar_year_filter(Some("2000-01-01"), None).as_deref(),
        Some("2000-")
    );
    assert_eq!(
        semantic_scholar_year_filter(None, Some("2013-12-31")).as_deref(),
        Some("-2013")
    );
    assert_eq!(semantic_scholar_year_filter(None, None), None);
}

#[test]
fn semantic_scholar_candidates_send_effective_year_filter() {
    let mut filters = empty_filters();
    filters.keyword = Some("braf melanoma".into());
    filters.date_from = Some("2000-01-01".into());
    filters.date_to = Some("2013-12-31".into());
    let (date_from, date_to) =
        super::super::filters::normalized_date_bounds(&filters).expect("date bounds");
    let year_filter = semantic_scholar_year_filter(date_from.as_deref(), date_to.as_deref());

    let plan = crate::sources::semantic_scholar::SemanticScholarClient::paper_search_plan(
        "braf melanoma",
        3,
        year_filter.as_deref(),
        Some("dummy-key"),
    )
    .expect("semantic scholar plan");

    assert_eq!(plan.path, "graph/v1/paper/search");
    assert_eq!(query_value(&plan.query, "query"), Some("braf melanoma"));
    assert_eq!(query_value(&plan.query, "limit"), Some("3"));
    assert_eq!(query_value(&plan.query, "year"), Some("2000-2013"));
    assert_eq!(query_value(&plan.headers, "x-api-key"), Some("dummy-key"));
}

#[test]
fn litsense2_candidates_deduplicate_and_hydrate_pubmed_metadata() {
    let hits = dedupe_litsense2_hits(vec![
        lit_hit(22663011, "First weaker sentence", 0.5, Some("PMC9984800")),
        lit_hit(
            22663011,
            "Stronger sentence for the same PMID",
            0.9,
            Some("PMC9984800"),
        ),
        lit_hit(
            24200969,
            "Fallback title text that should be truncated when PubMed has no title",
            0.7,
            None,
        ),
    ]);
    let rows = litsense2_rows_from_hits(
        &ArticleSearchFilters {
            keyword: Some("Hirschsprung disease ganglion cells".into()),
            exclude_retracted: true,
            ..empty_filters()
        },
        10,
        None,
        None,
        hits,
        hydrate_pubmed_entries(vec![
            pubmed_entry(
                "22663011",
                "Hydrated LitSense2 title",
                Some("Journal One"),
                Some("2024/01/15 00:00"),
            ),
            pubmed_entry("24200969", " ", None, None),
        ]),
    );

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].pmid, "22663011");
    assert_eq!(rows[0].title, "Hydrated LitSense2 title");
    assert_eq!(rows[0].pmcid.as_deref(), Some("PMC9984800"));
    assert_eq!(rows[0].journal.as_deref(), Some("Journal One"));
    assert_eq!(rows[0].date.as_deref(), Some("2024-01-15"));
    assert_eq!(rows[0].score, Some(0.9));
    assert_eq!(rows[0].source, ArticleSource::LitSense2);
    assert_eq!(rows[0].matched_sources, vec![ArticleSource::LitSense2]);
    assert_eq!(rows[0].source_local_position, 0);
    assert!(
        rows[0]
            .abstract_snippet
            .as_deref()
            .is_some_and(|snippet| snippet.contains("Stronger sentence for the same PMID"))
    );
    assert_eq!(rows[1].pmid, "24200969");
    assert!(!rows[1].title.trim().is_empty());
    assert_eq!(rows[1].score, Some(0.7));
    assert_eq!(rows[1].source_local_position, 1);
    assert!(
        rows[1]
            .abstract_snippet
            .as_deref()
            .is_some_and(|snippet| snippet.contains("Fallback title text"))
    );
}

#[test]
fn litsense2_candidates_apply_hydrated_journal_and_date_filters() {
    let filters = ArticleSearchFilters {
        keyword: Some("Hirschsprung disease".into()),
        journal: Some("Journal One".into()),
        date_from: Some("2024".into()),
        date_to: Some("2024-12".into()),
        exclude_retracted: true,
        ..empty_filters()
    };
    let (date_from, date_to) =
        super::super::filters::normalized_date_bounds(&filters).expect("date bounds");
    let rows = litsense2_rows_from_hits(
        &filters,
        10,
        date_from.as_deref(),
        date_to.as_deref(),
        dedupe_litsense2_hits(vec![
            lit_hit(22663011, "Hydrated row", 0.9, Some("PMC9984800")),
            lit_hit(24200969, "Fallback row", 0.7, None),
        ]),
        hydrate_pubmed_entries(vec![
            pubmed_entry(
                "22663011",
                "Hydrated LitSense2 title",
                Some("Journal One"),
                Some("2024/01/15 00:00"),
            ),
            pubmed_entry(
                "24200969",
                "Fallback title",
                Some("Journal Two"),
                Some("2023/01/15 00:00"),
            ),
        ]),
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].pmid, "22663011");
    assert_eq!(rows[0].journal.as_deref(), Some("Journal One"));
    assert_eq!(rows[0].date.as_deref(), Some("2024-01-15"));
}
