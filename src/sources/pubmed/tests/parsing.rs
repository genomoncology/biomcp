//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to decoders.
//! No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/pubmed/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

fn decode_esummary(
    ids: &[&str],
    body: serde_json::Value,
) -> Result<Vec<ESummaryEntry>, BioMcpError> {
    let ids = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>();
    PubMedClient::decode_esummary_response(
        &ids,
        StatusCode::OK,
        Some(&json_ct()),
        serde_json::to_vec(&body).unwrap().as_slice(),
    )
}

#[test]
fn parses_esearch_fixture() {
    let response = PubMedClient::decode_esearch_response(
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("esearch_braf.json"),
    )
    .unwrap();

    assert_eq!(response.count, 2);
    assert_eq!(response.idlist, vec!["123".to_string(), "456".to_string()]);
}

#[test]
fn esearch_handles_empty_idlist_and_rejects_bad_count() {
    let empty = PubMedClient::decode_esearch_response(
        StatusCode::OK,
        Some(&json_ct()),
        br#"{"esearchresult":{"count":"0","idlist":[]}}"#,
    )
    .unwrap();
    assert_eq!(empty.count, 0);
    assert!(empty.idlist.is_empty());

    let err = PubMedClient::decode_esearch_response(
        StatusCode::OK,
        Some(&json_ct()),
        br#"{"esearchresult":{"count":"not-a-number","idlist":["123"]}}"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("count"));
}

#[test]
fn parses_esummary_fixture_in_requested_order() {
    let ids = vec!["2".to_string(), "1".to_string()];
    let response = PubMedClient::decode_esummary_response(
        &ids,
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("esummary_two_ids.json"),
    )
    .unwrap();

    assert_eq!(response.len(), 2);
    assert_eq!(response[0].uid, "2");
    assert_eq!(response[0].title, "Second title");
    assert_eq!(response[0].fulljournalname.as_deref(), Some("Journal Two"));
    assert_eq!(response[1].uid, "1");
    assert_eq!(response[1].title, "First title");
    assert_eq!(response[1].edat.as_deref(), Some("2024/01/16 00:00"));
    assert_eq!(response[1].lr.as_deref(), Some("2024/01/17 00:00"));
    assert_eq!(response[1].source.as_deref(), Some("J1"));
}

#[test]
fn esummary_strictly_validates_uids_and_entries() {
    let missing_uids = decode_esummary(
        &["1"],
        serde_json::json!({
            "result": {
                "1": {"uid": "1", "title": "Only title"}
            }
        }),
    )
    .unwrap_err();
    assert!(missing_uids.to_string().contains("uids"));

    let duplicate = decode_esummary(
        &["1"],
        serde_json::json!({
            "result": {
                "uids": ["1", "1"],
                "1": {"uid": "1", "title": "Only title"}
            }
        }),
    )
    .unwrap_err();
    assert!(duplicate.to_string().contains("duplicate"));

    let missing_requested = decode_esummary(
        &["1", "2"],
        serde_json::json!({
            "result": {
                "uids": ["1"],
                "1": {"uid": "1", "title": "Only title"}
            }
        }),
    )
    .unwrap_err();
    assert!(missing_requested.to_string().contains("2"));

    let unexpected = decode_esummary(
        &["1"],
        serde_json::json!({
            "result": {
                "uids": ["1", "9"],
                "1": {"uid": "1", "title": "Only title"},
                "9": {"uid": "9", "title": "Unexpected title"}
            }
        }),
    )
    .unwrap_err();
    assert!(unexpected.to_string().contains("unexpected"));

    let missing_entry = decode_esummary(
        &["1"],
        serde_json::json!({
            "result": {"uids": ["1"]}
        }),
    )
    .unwrap_err();
    assert!(missing_entry.to_string().contains("entry"));

    let malformed = decode_esummary(
        &["1"],
        serde_json::json!({
            "result": {
                "uids": ["1"],
                "1": []
            }
        }),
    )
    .unwrap_err();
    assert!(malformed.to_string().contains("parse"));

    let conflicting = decode_esummary(
        &["1"],
        serde_json::json!({
            "result": {
                "uids": ["1"],
                "1": {"uid": "2", "title": "Conflicting title"}
            }
        }),
    )
    .unwrap_err();
    assert!(conflicting.to_string().contains("uid"));
    assert!(conflicting.to_string().contains("2"));
}

#[test]
fn parses_citation_authors_affiliations_orcid_and_mesh() {
    let xml = r#"<PubmedArticleSet><PubmedArticle><MedlineCitation>
      <PMID>22663011</PMID><Article><AuthorList>
        <Author><LastName>First</LastName><ForeName>Ada</ForeName>
          <Identifier Source="ORCID">HTTPS://ORCID.ORG/0000-0002-1825-0097</Identifier>
          <AffiliationInfo><Affiliation>First Institution</Affiliation><Identifier Source="ROR">shared</Identifier></AffiliationInfo>
          <AffiliationInfo><Affiliation>Second Institution</Affiliation><Identifier Source="GRID">grid.2</Identifier></AffiliationInfo>
        </Author>
        <Author><CollectiveName>Fixture Consortium</CollectiveName>
          <AffiliationInfo><Affiliation>First Institution</Affiliation><Identifier Source="ROR">shared</Identifier></AffiliationInfo>
        </Author>
      </AuthorList></Article><MeshHeadingList><MeshHeading>
        <DescriptorName UI="D008545" MajorTopicYN="Y">Melanoma</DescriptorName>
        <QualifierName UI="Q000235" MajorTopicYN="N">genetics</QualifierName>
      </MeshHeading></MeshHeadingList>
    </MedlineCitation></PubmedArticle></PubmedArticleSet>"#;

    let citation = parse_citation_xml("22663011", xml).unwrap();
    assert_eq!(citation.authors[0].name, "Ada First");
    assert_eq!(
        citation.authors[0].orcid.as_deref(),
        Some("0000-0002-1825-0097")
    );
    assert_eq!(citation.authors[0].affiliations.len(), 2);
    assert_eq!(citation.authors[1].name, "Fixture Consortium");
    assert_eq!(
        citation.authors[0].affiliations[0].identifiers,
        citation.authors[1].affiliations[0].identifiers
    );
    assert_eq!(
        citation.mesh_headings[0].descriptor.ui.as_deref(),
        Some("D008545")
    );
    assert!(citation.mesh_headings[0].descriptor.major_topic);
    assert!(!citation.mesh_headings[0].qualifiers[0].major_topic);
}

#[test]
fn citation_without_mesh_is_available_empty() {
    let xml = r#"<PubmedArticleSet><PubmedArticle><MedlineCitation><PMID>1</PMID><Article /></MedlineCitation></PubmedArticle></PubmedArticleSet>"#;
    let citation = parse_citation_xml("1", xml).unwrap();
    assert!(citation.authors.is_empty());
    assert!(citation.mesh_headings.is_empty());
}

#[test]
fn citation_parser_rejects_misses_errors_and_invalid_required_shape() {
    let empty = parse_citation_xml("1", "<PubmedArticleSet />").unwrap_err();
    assert!(matches!(empty, BioMcpError::NotFound { .. }));

    let provider_error =
        parse_citation_xml("1", "<eFetchResult><ERROR>bad id</ERROR></eFetchResult>").unwrap_err();
    assert!(matches!(provider_error, BioMcpError::NotFound { .. }));

    let bad_author = r#"<PubmedArticleSet><PubmedArticle><MedlineCitation><PMID>1</PMID><Article><AuthorList><Author><ForeName>Nameless</ForeName></Author></AuthorList></Article></MedlineCitation></PubmedArticle></PubmedArticleSet>"#;
    assert!(
        parse_citation_xml("1", bad_author)
            .unwrap_err()
            .to_string()
            .contains("name")
    );

    let bad_major = r#"<PubmedArticleSet><PubmedArticle><MedlineCitation><PMID>1</PMID><MeshHeadingList><MeshHeading><DescriptorName MajorTopicYN="Maybe">Term</DescriptorName></MeshHeading></MeshHeadingList></MedlineCitation></PubmedArticle></PubmedArticleSet>"#;
    assert!(
        parse_citation_xml("1", bad_major)
            .unwrap_err()
            .to_string()
            .contains("MajorTopicYN")
    );

    let bad_identifier = r#"<PubmedArticleSet><PubmedArticle><MedlineCitation><PMID>1</PMID><Article><AuthorList><Author><LastName>Author</LastName><AffiliationInfo><Affiliation>Institution</Affiliation><Identifier Source="ROR"> </Identifier></AffiliationInfo></Author></AuthorList></Article></MedlineCitation></PubmedArticle></PubmedArticleSet>"#;
    assert!(
        parse_citation_xml("1", bad_identifier)
            .unwrap_err()
            .to_string()
            .contains("identifier")
    );
    assert!(
        parse_citation_xml("1", "<PubmedArticleSet>")
            .unwrap_err()
            .to_string()
            .contains("failed to parse")
    );
}

#[test]
fn citation_decoder_accepts_xml_media_types_and_hides_bodies() {
    let body = b"<PubmedArticleSet />".to_vec();
    assert_eq!(
        PubMedClient::decode_citation_response(
            StatusCode::OK,
            Some(&HeaderValue::from_static("application/xml; charset=utf-8")),
            body.clone(),
        )
        .unwrap(),
        "<PubmedArticleSet />"
    );
    assert!(
        PubMedClient::decode_citation_response(
            StatusCode::OK,
            Some(&HeaderValue::from_static("text/xml")),
            body,
        )
        .is_ok()
    );

    assert!(
        PubMedClient::decode_citation_response(
            StatusCode::OK,
            Some(&HeaderValue::from_static("application/xml")),
            vec![0xff],
        )
        .unwrap_err()
        .to_string()
        .contains("UTF-8")
    );

    for error in [
        PubMedClient::decode_citation_response(
            StatusCode::BAD_GATEWAY,
            Some(&HeaderValue::from_static("text/plain")),
            b"raw-body-sentinel".to_vec(),
        )
        .unwrap_err(),
        PubMedClient::decode_citation_response(
            StatusCode::OK,
            Some(&HeaderValue::from_static("text/html")),
            b"raw-body-sentinel".to_vec(),
        )
        .unwrap_err(),
    ] {
        assert!(!error.to_string().contains("raw-body-sentinel"));
    }
}

#[test]
fn decode_json_maps_http_and_content_type_errors() {
    let http = PubMedClient::decode_esearch_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
    )
    .unwrap_err();
    let msg = http.to_string();
    assert!(matches!(http, BioMcpError::Api { .. }));
    assert!(msg.contains("pubmed-eutils"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");

    let html = HeaderValue::from_static("text/html");
    let content_type = PubMedClient::decode_esearch_response(
        StatusCode::OK,
        Some(&html),
        b"<html><body>error</body></html>",
    )
    .unwrap_err();
    assert!(content_type.to_string().contains("HTML"));
}
