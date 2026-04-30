#[allow(unused_imports)]
use super::super::test_support::*;
use super::*;
#[allow(unused_imports)]
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
#[allow(unused_imports)]
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn search_pubmed_page_rejects_open_access() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.open_access = true;

    let err = search_pubmed_page(&filters, 5, 0)
        .await
        .expect_err("open-access should be rejected for PubMed page helper");

    assert!(err.to_string().contains("--open-access"));
    assert!(err.to_string().contains("PubMed"));
}

#[tokio::test]
async fn search_pubmed_page_rejects_no_preprints() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.no_preprints = true;

    let err = search_pubmed_page(&filters, 5, 0)
        .await
        .expect_err("no-preprints should be rejected for PubMed page helper");

    assert!(err.to_string().contains("--no-preprints"));
    assert!(err.to_string().contains("PubMed"));
}

#[tokio::test]
async fn search_pubmed_page_sends_standalone_not_retraction_term() {
    let _guard = lock_env().await;
    let server = MockServer::start().await;
    let _pubmed_base = set_env_var("BIOMCP_PUBMED_BASE", Some(&server.uri()));
    let mut filters = empty_filters();
    filters.gene = Some("WDR5".into());
    filters.exclude_retracted = true;

    Mock::given(method("GET"))
        .and(path("/esearch.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("retstart", "0"))
        .and(query_param("retmax", "100"))
        .and(query_param("term", "WDR5 NOT retracted publication[pt]"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "esearchresult": {
                "count": "0",
                "idlist": []
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let page = search_pubmed_page(&filters, 1, 0)
        .await
        .expect("pubmed page should accept standalone NOT query");

    assert!(page.results.is_empty());
    assert_eq!(page.total, Some(0));
}

#[tokio::test]
async fn search_pubmed_page_cleans_question_keyword_before_esearch() {
    let _guard = lock_env().await;
    let server = MockServer::start().await;
    let _pubmed_base = set_env_var("BIOMCP_PUBMED_BASE", Some(&server.uri()));
    let mut filters = empty_filters();
    filters.keyword = Some("What drug treatment can cause a spinal epidural hematoma?".into());

    Mock::given(method("GET"))
        .and(path("/esearch.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("retstart", "0"))
        .and(query_param("retmax", "100"))
        .and(query_param(
            "term",
            "drug treatment spinal epidural hematoma",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "esearchresult": {
                "count": "0",
                "idlist": []
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let page = search_pubmed_page(&filters, 1, 0)
        .await
        .expect("pubmed page should use cleaned question keyword");

    assert!(page.results.is_empty());
    assert_eq!(page.total, Some(0));
}

#[tokio::test]
async fn search_pubmed_page_refills_across_batches_after_filtering() {
    let _guard = lock_env().await;
    let server = MockServer::start().await;
    let _pubmed_base = set_env_var("BIOMCP_PUBMED_BASE", Some(&server.uri()));
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.journal = Some("Nature".into());

    Mock::given(method("GET"))
        .and(path("/esearch.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("retstart", "0"))
        .and(query_param("retmax", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "esearchresult": {
                "count": "4",
                "idlist": ["1", "2"]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/esummary.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("id", "1,2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "uids": ["1", "2"],
                "1": {
                    "uid": "1",
                    "title": "Filtered title",
                    "sortpubdate": "2024/01/01 00:00",
                    "fulljournalname": "Other Journal",
                    "source": "Other J"
                },
                "2": {
                    "uid": "2",
                    "title": "First visible title",
                    "sortpubdate": "2024/01/02 00:00",
                    "fulljournalname": "Nature",
                    "source": "Nature"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/esearch.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("retstart", "2"))
        .and(query_param("retmax", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "esearchresult": {
                "count": "4",
                "idlist": ["3", "4"]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/esummary.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("id", "3,4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "uids": ["3", "4"],
                "3": {
                    "uid": "3",
                    "title": "Second visible title",
                    "sortpubdate": "2024/01/03 00:00",
                    "fulljournalname": "Nature",
                    "source": "Nature"
                },
                "4": {
                    "uid": "4",
                    "title": "Third visible title",
                    "sortpubdate": "2024/01/04 00:00",
                    "fulljournalname": "Nature",
                    "source": "Nature"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let page = search_pubmed_page(&filters, 2, 0)
        .await
        .expect("pubmed page should fill visible results");

    assert_eq!(page.total, Some(4));
    assert_eq!(page.results.len(), 2);
    assert_eq!(page.results[0].pmid, "2");
    assert_eq!(page.results[1].pmid, "3");
}

#[tokio::test]
async fn search_pubmed_page_applies_offset_after_filtering() {
    let _guard = lock_env().await;
    let server = MockServer::start().await;
    let _pubmed_base = set_env_var("BIOMCP_PUBMED_BASE", Some(&server.uri()));
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.journal = Some("Nature".into());

    Mock::given(method("GET"))
        .and(path("/esearch.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("retstart", "0"))
        .and(query_param("retmax", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "esearchresult": {
                "count": "4",
                "idlist": ["1", "2"]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/esummary.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("id", "1,2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "uids": ["1", "2"],
                "1": {
                    "uid": "1",
                    "title": "Filtered title",
                    "sortpubdate": "2024/01/01 00:00",
                    "fulljournalname": "Other Journal",
                    "source": "Other J"
                },
                "2": {
                    "uid": "2",
                    "title": "First visible title",
                    "sortpubdate": "2024/01/02 00:00",
                    "fulljournalname": "Nature",
                    "source": "Nature"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/esearch.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("retstart", "2"))
        .and(query_param("retmax", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "esearchresult": {
                "count": "4",
                "idlist": ["3", "4"]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/esummary.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("id", "3,4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "uids": ["3", "4"],
                "3": {
                    "uid": "3",
                    "title": "Second visible title",
                    "sortpubdate": "2024/01/03 00:00",
                    "fulljournalname": "Nature",
                    "source": "Nature"
                },
                "4": {
                    "uid": "4",
                    "title": "Third visible title",
                    "sortpubdate": "2024/01/04 00:00",
                    "fulljournalname": "Nature",
                    "source": "Nature"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let page = search_pubmed_page(&filters, 2, 1)
        .await
        .expect("offset should apply after filtering");

    assert_eq!(page.total, Some(4));
    assert_eq!(page.results.len(), 2);
    assert_eq!(page.results[0].pmid, "3");
    assert_eq!(page.results[1].pmid, "4");
    assert_eq!(page.results[0].source_local_position, 1);
    assert_eq!(page.results[1].source_local_position, 2);
}

#[tokio::test]
async fn search_pubmed_page_hard_fails_on_blank_title() {
    let _guard = lock_env().await;
    let server = MockServer::start().await;
    let _pubmed_base = set_env_var("BIOMCP_PUBMED_BASE", Some(&server.uri()));
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());

    Mock::given(method("GET"))
        .and(path("/esearch.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("retstart", "0"))
        .and(query_param("retmax", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "esearchresult": {
                "count": "1",
                "idlist": ["1"]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/esummary.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("id", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "uids": ["1"],
                "1": {
                    "uid": "1",
                    "title": "   ",
                    "sortpubdate": "2024/01/01 00:00",
                    "fulljournalname": "Nature",
                    "source": "Nature"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let err = search_pubmed_page(&filters, 1, 0)
        .await
        .expect_err("blank title should be a contract error");

    let msg = err.to_string();
    assert!(msg.contains("pubmed-eutils"));
    assert!(msg.contains("1"));
    assert!(msg.contains("title"));
}

#[tokio::test]
async fn semantic_scholar_candidates_keep_unknown_retraction_rows() {
    let _guard = lock_env().await;
    let server = MockServer::start().await;
    let _s2_base = set_env_var("BIOMCP_S2_BASE", Some(&server.uri()));
    let _s2_key = set_env_var("S2_API_KEY", Some("dummy-key"));

    Mock::given(method("GET"))
        .and(path("/graph/v1/paper/search"))
        .and(query_param(
            "query",
            "alternative microexon splicing metastasis",
        ))
        .and(query_param("limit", "3"))
        .and(header("x-api-key", "dummy-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total": 1,
            "data": [{
                "paperId": "paper-1",
                "externalIds": {
                    "PubMed": "22663011",
                    "DOI": "10.1000/example"
                },
                "title": "Alternative microexon splicing in metastasis",
                "venue": "Cancer Cell",
                "year": 2025,
                "citationCount": 12,
                "influentialCitationCount": 4,
                "abstract": "Microexon splicing contributes to metastatic progression."
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let rows = search_semantic_scholar_candidates(
        &ArticleSearchFilters {
            keyword: Some("alternative microexon splicing metastasis".into()),
            exclude_retracted: true,
            ..empty_filters()
        },
        3,
    )
    .await
    .expect("semantic scholar search should succeed");

    assert_eq!(rows.rows.len(), 1);
    assert_eq!(rows.rows[0].source, ArticleSource::SemanticScholar);
    assert_eq!(rows.rows[0].is_retracted, None);
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

#[tokio::test]
async fn semantic_scholar_candidates_send_effective_year_filter() {
    let _guard = lock_env().await;
    let server = MockServer::start().await;
    let _s2_base = set_env_var("BIOMCP_S2_BASE", Some(&server.uri()));
    let _s2_key = set_env_var("S2_API_KEY", Some("dummy-key"));

    Mock::given(method("GET"))
        .and(path("/graph/v1/paper/search"))
        .and(query_param("query", "braf melanoma"))
        .and(query_param("limit", "3"))
        .and(query_param("year", "2000-2013"))
        .and(header("x-api-key", "dummy-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total": 1,
            "data": [{
                "paperId": "paper-1",
                "externalIds": {
                    "PubMed": "22663011",
                    "DOI": "10.1000/example"
                },
                "title": "BRAF melanoma historical cohort",
                "venue": "Cancer Cell",
                "year": 2005,
                "citationCount": 12,
                "influentialCitationCount": 4,
                "abstract": "Historical cohort."
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let rows = search_semantic_scholar_candidates(
        &ArticleSearchFilters {
            keyword: Some("braf melanoma".into()),
            date_from: Some("2000-01-01".into()),
            date_to: Some("2013-12-31".into()),
            exclude_retracted: true,
            ..empty_filters()
        },
        3,
    )
    .await
    .expect("semantic scholar search should include the effective year filter");

    assert_eq!(rows.rows.len(), 1);
    assert_eq!(rows.rows[0].date.as_deref(), Some("2005"));
}
#[tokio::test]
async fn litsense2_candidates_deduplicate_and_hydrate_pubmed_metadata() {
    let _guard = lock_env().await;
    let litsense2 = MockServer::start().await;
    let pubmed = MockServer::start().await;
    let _litsense2_base = set_env_var("BIOMCP_LITSENSE2_BASE", Some(&litsense2.uri()));
    let _pubmed_base = set_env_var("BIOMCP_PUBMED_BASE", Some(&pubmed.uri()));

    Mock::given(method("GET"))
        .and(path("/sentences/"))
        .and(query_param("query", "Hirschsprung disease ganglion cells"))
        .and(query_param("rerank", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "pmid": 22663011,
                "pmcid": "PMC9984800",
                "text": "First weaker sentence",
                "score": 0.5,
                "section": "INTRO",
                "annotations": ["0|12|disease|MESH:D006627"]
            },
            {
                "pmid": 22663011,
                "pmcid": "PMC9984800",
                "text": "Stronger sentence for the same PMID",
                "score": 0.9,
                "section": "RESULTS",
                "annotations": []
            },
            {
                "pmid": 24200969,
                "pmcid": null,
                "text": "Fallback title text that should be truncated when PubMed has no title",
                "score": 0.7,
                "section": null,
                "annotations": null
            }
        ])))
        .expect(1)
        .mount(&litsense2)
        .await;

    Mock::given(method("GET"))
        .and(path("/esummary.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("id", "22663011,24200969"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "uids": ["22663011", "24200969"],
                "22663011": {
                    "uid": "22663011",
                    "title": "Hydrated LitSense2 title",
                    "sortpubdate": "2024/01/15 00:00",
                    "pubdate": "2024 Jan 15",
                    "fulljournalname": "Journal One",
                    "source": "J1"
                },
                "24200969": {
                    "uid": "24200969",
                    "title": " ",
                    "sortpubdate": null,
                    "pubdate": null,
                    "fulljournalname": null,
                    "source": null
                }
            }
        })))
        .expect(1)
        .mount(&pubmed)
        .await;

    let rows = search_litsense2_candidates(
        &ArticleSearchFilters {
            keyword: Some("Hirschsprung disease ganglion cells".into()),
            exclude_retracted: true,
            ..empty_filters()
        },
        10,
    )
    .await
    .expect("litsense2 search should succeed");

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

#[tokio::test]
async fn litsense2_candidates_apply_hydrated_journal_and_date_filters() {
    let _guard = lock_env().await;
    let litsense2 = MockServer::start().await;
    let pubmed = MockServer::start().await;
    let _litsense2_base = set_env_var("BIOMCP_LITSENSE2_BASE", Some(&litsense2.uri()));
    let _pubmed_base = set_env_var("BIOMCP_PUBMED_BASE", Some(&pubmed.uri()));

    Mock::given(method("GET"))
        .and(path("/sentences/"))
        .and(query_param("query", "Hirschsprung disease"))
        .and(query_param("rerank", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "pmid": 22663011,
                "pmcid": "PMC9984800",
                "text": "Hydrated row",
                "score": 0.9,
                "section": "INTRO",
                "annotations": []
            },
            {
                "pmid": 24200969,
                "pmcid": null,
                "text": "Fallback row",
                "score": 0.7,
                "section": null,
                "annotations": null
            }
        ])))
        .expect(1)
        .mount(&litsense2)
        .await;

    Mock::given(method("GET"))
        .and(path("/esummary.fcgi"))
        .and(query_param("db", "pubmed"))
        .and(query_param("retmode", "json"))
        .and(query_param("id", "22663011,24200969"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "uids": ["22663011", "24200969"],
                "22663011": {
                    "uid": "22663011",
                    "title": "Hydrated LitSense2 title",
                    "sortpubdate": "2024/01/15 00:00",
                    "pubdate": "2024 Jan 15",
                    "fulljournalname": "Journal One",
                    "source": "J1"
                },
                "24200969": {
                    "uid": "24200969",
                    "title": "Fallback title",
                    "sortpubdate": "2023/01/15 00:00",
                    "pubdate": "2023 Jan 15",
                    "fulljournalname": "Journal Two",
                    "source": "J2"
                }
            }
        })))
        .expect(1)
        .mount(&pubmed)
        .await;

    let rows = search_litsense2_candidates(
        &ArticleSearchFilters {
            keyword: Some("Hirschsprung disease".into()),
            journal: Some("Journal One".into()),
            date_from: Some("2024".into()),
            date_to: Some("2024-12".into()),
            exclude_retracted: true,
            ..empty_filters()
        },
        10,
    )
    .await
    .expect("litsense2 search should respect hydrated filters");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].pmid, "22663011");
    assert_eq!(rows[0].journal.as_deref(), Some("Journal One"));
    assert_eq!(rows[0].date.as_deref(), Some("2024-01-15"));
}
