use super::retained::{article_from_europepmc_fallback, resolve_variant_article_from_pmid};
use super::*;
use crate::entities::article::{ArticleAuthorCompleteness, ArticleSource};
use crate::error::BioMcpError;

#[serial_test::serial(source_env)]
#[tokio::test]
async fn completed_pubtator_enrichment_survives_later_europepmc_failure() {
    use crate::entities::article::test_support::{
        TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
    };

    const PUBTATOR: &[u8] =
        include_bytes!("../../../../testdata/sources/pubtator/export_22663011.json");
    let fixture = TestHttpFixture::spawn(|request| {
        let first_line = request.lines().next().unwrap_or_default();
        if first_line.starts_with("GET /publications/export/biocjson?") {
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", PUBTATOR))
        } else {
            TestHttpReply::Bytes(test_http_response(
                "503 Service Unavailable",
                "application/json",
                br#"{\"error\":\"fixture Europe PMC failure\"}"#,
            ))
        }
    })
    .await;
    let cache = crate::test_support::TempDirGuard::new("variant-enrichment-partial-commit");
    let mut env = TestEnv::new();
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_PUBTATOR_BASE", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    env.set("NCBI_API_KEY", "");
    let execution = super::super::variant_search::VariantArticleExecutionContext::single();

    let article =
        resolve_variant_article_from_pmid(22_663_011, "22663011", "22663011", None, &execution)
            .await
            .expect("completed PubTator leg remains usable");

    assert_eq!(article.pmid.as_deref(), Some("22663011"));
    assert!(!article.title.trim().is_empty());
    let events = execution.terminal_event_states();
    assert_eq!(events.len(), 2);
    assert_eq!(
        events[0],
        ("enrichment".into(), "pubtator".into(), "ok".into(), None)
    );
    assert_eq!(
        events[1],
        (
            "enrichment".into(),
            "europepmc".into(),
            "unavailable".into(),
            Some("provider_error")
        )
    );
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn rejected_admitted_detail_is_terminal_after_one_pubtator_request() {
    use crate::entities::article::test_support::{
        TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    let pubtator_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let europe_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let pubtator_seen = pubtator_calls.clone();
    let europe_seen = europe_calls.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        if request.starts_with("GET /publications/export/biocjson?") {
            pubtator_seen.fetch_add(1, Ordering::SeqCst);
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                br#"{"PubTator3":[{"id":"8","passages":[]}]}"#,
            ))
        } else {
            europe_seen.fetch_add(1, Ordering::SeqCst);
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", br#"{}"#))
        }
    })
    .await;
    let cache = crate::test_support::TempDirGuard::new("pubtator-admitted-refusal");
    let mut env = TestEnv::new();
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_PUBTATOR_BASE", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    env.set("NCBI_API_KEY", "");
    let pubtator = PubTatorClient::new().unwrap();
    let europe = EuropePmcClient::new().unwrap();

    let error = get_article_base_with_clients("7", &pubtator, &europe)
        .await
        .expect_err("mismatched admitted record must fail");

    assert!(format!("{error:?}").contains("identity_mismatch"));
    assert_eq!(pubtator_calls.load(Ordering::SeqCst), 1);
    assert_eq!(europe_calls.load(Ordering::SeqCst), 0);
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn empty_admitted_response_is_original_not_found_without_europepmc() {
    use crate::entities::article::test_support::{
        TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    let europe_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let europe_seen = europe_calls.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        let body = if request.starts_with("GET /publications/export/biocjson?") {
            br#"{"PubTator3":[]}"#.as_slice()
        } else {
            europe_seen.fetch_add(1, Ordering::SeqCst);
            br#"{}"#.as_slice()
        };
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", body))
    })
    .await;
    let cache = crate::test_support::TempDirGuard::new("pubtator-empty-admitted");
    let mut env = TestEnv::new();
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_PUBTATOR_BASE", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    env.set("NCBI_API_KEY", "");
    let pubtator = PubTatorClient::new().unwrap();
    let europe = EuropePmcClient::new().unwrap();

    let error = get_article_base_with_clients("7", &pubtator, &europe)
        .await
        .expect_err("empty admitted response");

    assert!(matches!(
        error,
        BioMcpError::NotFound { entity, id, .. } if entity == "article" && id == "7"
    ));
    assert_eq!(europe_calls.load(Ordering::SeqCst), 0);
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn actual_pubtator_http_404_retains_europepmc_fallback() {
    use crate::entities::article::test_support::{
        TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
    };

    let fixture = TestHttpFixture::spawn(|request| {
        if request.starts_with("GET /publications/export/biocjson?") {
            TestHttpReply::Bytes(test_http_response(
                "404 Not Found",
                "application/json",
                br#"{"message":"not ready"}"#,
            ))
        } else {
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                br#"{"hitCount":1,"resultList":{"result":[{"id":"7","pmid":"7","title":"Europe fallback"}]}}"#,
            ))
        }
    })
    .await;
    let cache = crate::test_support::TempDirGuard::new("pubtator-http-404-fallback");
    let mut env = TestEnv::new();
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_PUBTATOR_BASE", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    env.set("NCBI_API_KEY", "");
    let pubtator = PubTatorClient::new().unwrap();
    let europe = EuropePmcClient::new().unwrap();

    let article = get_article_base_with_clients("7", &pubtator, &europe)
        .await
        .expect("HTTP 404 fallback");

    assert_eq!(article.title, "Europe fallback");
    assert!(article.pubtator_fallback);
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn admitted_detail_preserves_shared_text_then_assembles_product_fields() {
    use crate::entities::article::test_support::{
        TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
    };

    let long_abstract = format!("  {}  ", "A".repeat(1700));
    let payload = serde_json::to_vec(&serde_json::json!({
        "PubTator3": [{
            "id": "7",
            "pmcid": null,
            "date": "2026-09-16T12:00:00Z",
            "journal": "  Document Journal  ",
            "authors": ["First Author", "Second Author"],
            "passages": [
                {
                    "infons": {"type": "title", "journal": "Passage Journal", "article-id_pmc": "PMC999"},
                    "offset": 0,
                    "text": "  Admitted title  ",
                    "sentences": [],
                    "annotations": [{
                        "id": "a1",
                        "infons": {"type": "Gene"},
                        "text": "BRAF",
                        "locations": [{"offset": 2, "length": 4}]
                    }],
                    "relations": []
                },
                {
                    "infons": {"type": "abstract"},
                    "offset": 20,
                    "text": long_abstract,
                    "sentences": [],
                    "annotations": [],
                    "relations": []
                }
            ]
        }]
    }))
    .unwrap();
    let fixture = TestHttpFixture::spawn(move |request| {
        if request.starts_with("GET /publications/export/biocjson?") {
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &payload))
        } else {
            TestHttpReply::Bytes(test_http_response(
                "503 Service Unavailable",
                "application/json",
                br#"{}"#,
            ))
        }
    })
    .await;
    let cache = crate::test_support::TempDirGuard::new("pubtator-admitted-projection");
    let mut env = TestEnv::new();
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_PUBTATOR_BASE", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    env.set("NCBI_API_KEY", "");
    let pubtator = PubTatorClient::new().unwrap();
    let europe = EuropePmcClient::new().unwrap();

    let article = get_article_base_with_clients("7", &pubtator, &europe)
        .await
        .expect("admitted detail");

    assert_eq!(article.pmid.as_deref(), Some("7"));
    assert_eq!(article.pmcid, None);
    assert_eq!(article.title, "Admitted title");
    assert_eq!(article.journal.as_deref(), Some("Document Journal"));
    assert_eq!(article.date.as_deref(), Some("2026-09-16"));
    assert_eq!(article.authors, ["First Author", "Second Author"]);
    assert_eq!(
        article.author_completeness,
        ArticleAuthorCompleteness::SourceLimited
    );
    assert!(
        article
            .abstract_text
            .as_deref()
            .unwrap()
            .contains("(truncated, 1700 chars total)")
    );
    assert_eq!(article.annotations.unwrap().genes[0].text, "BRAF");
}

#[tokio::test]
async fn get_rejects_pdf_without_fulltext_section() {
    let err = get(
        "22663013",
        &[],
        ArticleGetOptions {
            allow_pdf: true,
            ..ArticleGetOptions::default()
        },
    )
    .await
    .expect_err("pdf without fulltext should fail");

    assert!(matches!(
        err,
        BioMcpError::InvalidArgument(message)
            if message.contains("--pdf requires the fulltext section")
    ));
}

#[test]
fn parse_sections_supports_tldr_indexing_and_all() {
    let tldr_only = parse_sections(&["tldr".to_string()]).expect("tldr should parse");
    assert!(tldr_only.include_tldr);
    assert!(!tldr_only.include_annotations);
    assert!(!tldr_only.include_fulltext);
    assert!(!tldr_only.include_indexing);

    let indexing = parse_sections(&["indexing".to_string()]).expect("indexing should parse");
    assert!(indexing.include_indexing);
    assert!(!indexing.include_annotations);
    assert!(!indexing.include_fulltext);

    let all = parse_sections(&["all".to_string()]).expect("all should parse");
    assert!(all.include_tldr);
    assert!(all.include_annotations);
    assert!(all.include_fulltext);
    assert!(all.include_indexing);
}

#[test]
fn maps_pubmed_citation_to_available_indexing_without_flattening() {
    use crate::sources::pubmed::{
        PubMedAffiliation, PubMedAffiliationIdentifier, PubMedCitation, PubMedCitationAuthor,
        PubMedMeshHeading, PubMedMeshTerm,
    };

    let indexing = article_indexing_from_citation(PubMedCitation {
        authors: vec![PubMedCitationAuthor {
            name: "Ada First".into(),
            orcid: Some("0000-0002-1825-0097".into()),
            affiliations: vec![PubMedAffiliation {
                text: "Fixture University".into(),
                identifiers: vec![PubMedAffiliationIdentifier {
                    source: "ROR".into(),
                    value: "shared".into(),
                }],
            }],
        }],
        mesh_headings: vec![PubMedMeshHeading {
            descriptor: PubMedMeshTerm {
                text: "Melanoma".into(),
                ui: Some("D008545".into()),
                major_topic: true,
            },
            qualifiers: vec![PubMedMeshTerm {
                text: "genetics".into(),
                ui: Some("Q000235".into()),
                major_topic: false,
            }],
        }],
    });

    assert_eq!(indexing.status, ArticleIndexingStatus::Available);
    assert_eq!(indexing.source, ArticleSource::PubMed);
    assert!(indexing.failure.is_none());
    assert_eq!(
        indexing.authors[0].affiliations[0].identifiers[0].source,
        "ROR"
    );
    assert!(indexing.mesh_headings[0].descriptor.major_topic);
    assert!(!indexing.mesh_headings[0].qualifiers[0].major_topic);
}

#[test]
fn unavailable_indexing_maps_every_cause_to_a_static_failure() {
    let cases = [
        (
            IndexingUnavailableCause::MissingPmid,
            ArticleIndexingFailureCode::MissingPmid,
            "This article has no PMID for PubMed indexing.",
        ),
        (
            IndexingUnavailableCause::Client,
            ArticleIndexingFailureCode::ClientError,
            "PubMed indexing could not initialize its client.",
        ),
        (
            IndexingUnavailableCause::PubMed(PubMedCitationErrorKind::Network),
            ArticleIndexingFailureCode::NetworkError,
            "PubMed indexing could not reach PubMed.",
        ),
        (
            IndexingUnavailableCause::PubMed(PubMedCitationErrorKind::Http),
            ArticleIndexingFailureCode::HttpError,
            "PubMed returned an unsuccessful response for indexing.",
        ),
        (
            IndexingUnavailableCause::PubMed(PubMedCitationErrorKind::RateLimited),
            ArticleIndexingFailureCode::RateLimited,
            "PubMed indexing was rate limited.",
        ),
        (
            IndexingUnavailableCause::PubMed(PubMedCitationErrorKind::InvalidResponse),
            ArticleIndexingFailureCode::InvalidResponse,
            "PubMed returned an invalid indexing response.",
        ),
        (
            IndexingUnavailableCause::PubMed(PubMedCitationErrorKind::ResponseTooLarge),
            ArticleIndexingFailureCode::ResponseTooLarge,
            "PubMed indexing response exceeded the size limit.",
        ),
        (
            IndexingUnavailableCause::PubMed(PubMedCitationErrorKind::Parse),
            ArticleIndexingFailureCode::ParseError,
            "PubMed indexing response could not be parsed.",
        ),
        (
            IndexingUnavailableCause::PubMed(PubMedCitationErrorKind::NotFound),
            ArticleIndexingFailureCode::NotFound,
            "PubMed indexing metadata was not found for this article.",
        ),
        (
            IndexingUnavailableCause::Timeout,
            ArticleIndexingFailureCode::Timeout,
            "PubMed indexing timed out.",
        ),
    ];

    for (cause, code, message) in cases {
        let indexing = unavailable_indexing(cause);
        assert_eq!(indexing.status, ArticleIndexingStatus::Unavailable);
        assert_eq!(indexing.source, ArticleSource::PubMed);
        assert!(indexing.authors.is_empty());
        assert!(indexing.mesh_headings.is_empty());
        assert_eq!(
            indexing.failure,
            Some(ArticleIndexingFailure {
                code,
                message: message.into(),
            })
        );
    }
    assert_eq!(ARTICLE_INDEXING_TIMEOUT, std::time::Duration::from_secs(10));
}

#[tokio::test]
async fn indexing_timeout_and_missing_pmid_become_unavailable() {
    let timeout = citation_with_timeout(
        std::time::Duration::ZERO,
        std::future::pending::<
            Result<crate::sources::pubmed::PubMedCitation, PubMedCitationErrorKind>,
        >(),
    )
    .await
    .expect_err("pending citation should time out");
    assert_eq!(timeout, IndexingUnavailableCause::Timeout);

    let hit = serde_json::from_value(serde_json::json!({"title": "No PMID"}))
        .expect("Europe PMC fixture");
    let mut article = article_from_europepmc_fallback(&hit);
    enrich_article_with_indexing(&mut article).await;
    let outcome = article
        .section_outcomes
        .get("indexing")
        .expect("indexing outcome");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Unavailable
    );
    assert!(outcome.sources().is_empty());
    let indexing = article.indexing.expect("requested indexing");
    assert_eq!(indexing.status, ArticleIndexingStatus::Unavailable);
    assert_eq!(
        indexing.failure.expect("unavailable failure").code,
        ArticleIndexingFailureCode::MissingPmid
    );
}

#[test]
fn is_doi_basic() {
    assert!(is_doi("10.1056/NEJMoa1203421"));
    assert!(is_doi("10.1056/nejmoa1203421"));
    assert!(!is_doi("22663011"));
    assert!(!is_doi("doi:10.1056/NEJMoa1203421"));
}

#[test]
fn parse_pmid_basic() {
    assert_eq!(parse_pmid("22663011"), Some(22663011));
    assert_eq!(parse_pmid(" 22663011 "), Some(22663011));
    assert_eq!(parse_pmid(""), None);
    assert_eq!(parse_pmid("10.1056/NEJMoa1203421"), None);
    assert_eq!(parse_pmid("abc"), None);
}

#[test]
fn parse_pmcid_basic() {
    assert_eq!(parse_pmcid("PMC9984800"), Some("PMC9984800".into()));
    assert_eq!(parse_pmcid("pmc9984800"), Some("PMC9984800".into()));
    assert_eq!(parse_pmcid("PMCID:PMC9984800"), Some("PMC9984800".into()));
    assert_eq!(parse_pmcid(" PMC9984800 "), Some("PMC9984800".into()));
    assert_eq!(parse_pmcid("PMC"), None);
    assert_eq!(parse_pmcid("PMCX"), None);
    assert_eq!(parse_pmcid("PMC-123"), None);
    assert_eq!(parse_pmcid("22663011"), None);
}

#[test]
fn parse_article_id_basic() {
    match parse_article_id("PMC9984800") {
        ArticleIdType::Pmc(v) => assert_eq!(v, "PMC9984800"),
        _ => panic!("expected PMCID"),
    }
    match parse_article_id("10.1056/NEJMoa1203421") {
        ArticleIdType::Doi(v) => assert_eq!(v, "10.1056/NEJMoa1203421"),
        _ => panic!("expected DOI"),
    }
    match parse_article_id("22663011") {
        ArticleIdType::Pmid(v) => assert_eq!(v, 22663011),
        _ => panic!("expected PMID"),
    }
    assert!(matches!(
        parse_article_id("doi:10.1056/NEJMoa1203421"),
        ArticleIdType::Invalid
    ));
}

#[test]
fn parse_article_id_publisher_pii_is_invalid() {
    assert!(matches!(
        parse_article_id("S1535610826000103"),
        ArticleIdType::Invalid
    ));
}

#[test]
fn europepmc_fallback_keeps_authorship_provenance_and_flag() {
    let hit: crate::sources::europepmc::EuropePmcResult =
        serde_json::from_value(serde_json::json!({
            "id": "22663011",
            "authorString": "First Author, Middle Author, Last Author"
        }))
        .expect("valid Europe PMC hit");

    let article = article_from_europepmc_fallback(&hit);

    assert_eq!(
        article.authors,
        vec!["First Author", "Middle Author", "Last Author"]
    );
    assert_eq!(article.author_count, article.authors.len());
    assert_eq!(
        article.author_completeness,
        ArticleAuthorCompleteness::SourceLimited
    );
    assert_eq!(article.author_source, ArticleSource::EuropePmc);
    assert!(article.pubtator_fallback);
}

#[test]
fn pubtator_lag_error_is_400_or_404_only() {
    let err_400 = BioMcpError::Api {
        api: "pubtator3".into(),
        message: "HTTP 400 Bad Request: pending".into(),
    };
    let err_404 = BioMcpError::Api {
        api: "PubTator 3".into(),
        message: "HTTP 404 Not Found: pending".into(),
    }
    .with_source_context(crate::error::SourceContext::retry(
        crate::error::SourceProvider::PUBTATOR3,
    ));
    let err_500 = BioMcpError::Api {
        api: "pubtator3".into(),
        message: "HTTP 500 Internal Server Error".into(),
    };
    let other_api_400 = BioMcpError::Api {
        api: "europepmc".into(),
        message: "HTTP 400 Bad Request".into(),
    };

    assert!(is_pubtator_lag_error(&err_400));
    assert!(is_pubtator_lag_error(&err_404));
    assert!(!is_pubtator_lag_error(&err_500));
    assert!(!is_pubtator_lag_error(&other_api_400));
}
