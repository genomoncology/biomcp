use super::*;
use crate::entities::article::{ArticleAuthorCompleteness, ArticleSource};
use crate::error::BioMcpError;

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
    assert_eq!(
        indexing.authors[0].affiliations[0].identifiers[0].source,
        "ROR"
    );
    assert!(indexing.mesh_headings[0].descriptor.major_topic);
    assert!(!indexing.mesh_headings[0].qualifiers[0].major_topic);
}

#[test]
fn unavailable_indexing_is_explicit_and_empty() {
    let indexing = unavailable_indexing();
    assert_eq!(indexing.status, ArticleIndexingStatus::Unavailable);
    assert_eq!(indexing.source, ArticleSource::PubMed);
    assert!(indexing.authors.is_empty());
    assert!(indexing.mesh_headings.is_empty());
    assert_eq!(ARTICLE_INDEXING_TIMEOUT, std::time::Duration::from_secs(10));
}

#[tokio::test]
async fn indexing_timeout_and_missing_pmid_become_unavailable() {
    let timeout = citation_with_timeout(
        std::time::Duration::ZERO,
        std::future::pending::<Result<crate::sources::pubmed::PubMedCitation, BioMcpError>>(),
    )
    .await
    .expect_err("pending citation should time out");
    assert!(timeout.to_string().contains("timed out"));

    let hit = serde_json::from_value(serde_json::json!({"title": "No PMID"}))
        .expect("Europe PMC fixture");
    let mut article = article_from_europepmc_fallback(&hit);
    enrich_article_with_indexing(&mut article).await;
    assert_eq!(
        article.indexing.expect("requested indexing").status,
        ArticleIndexingStatus::Unavailable
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
        api: "pubtator3".into(),
        message: "HTTP 404 Not Found: pending".into(),
    };
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
