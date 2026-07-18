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
    assert!(format!("{err:?}").contains("count"));
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
    assert!(format!("{missing_uids:?}").contains("uids"));

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
    assert!(format!("{duplicate:?}").contains("duplicate"));

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
    assert!(format!("{missing_requested:?}").contains("2"));

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
    assert!(format!("{unexpected:?}").contains("unexpected"));

    let missing_entry = decode_esummary(
        &["1"],
        serde_json::json!({
            "result": {"uids": ["1"]}
        }),
    )
    .unwrap_err();
    assert!(format!("{missing_entry:?}").contains("entry"));

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
    assert!(format!("{malformed:?}").contains("parse"));

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
    assert!(format!("{conflicting:?}").contains("uid"));
    assert!(format!("{conflicting:?}").contains("2"));
}

#[test]
fn parses_citation_authors_affiliations_orcid_and_mesh() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE PubmedArticleSet PUBLIC "-//NLM//DTD PubMedArticle//EN" "https://example.invalid/pubmed.dtd">
<PubmedArticleSet><PubmedArticle><MedlineCitation>
      <PMID>22663011</PMID><Article><AuthorList>
        <Author><LastName>First</LastName><ForeName>Ada</ForeName>
          <Identifier Source="ORCID">HTTPS://ORCID.ORG/0000-0002-1825-0097</Identifier>
          <AffiliationInfo><Affiliation>First Institution</Affiliation><Identifier Source="ROR">shared</Identifier></AffiliationInfo>
          <AffiliationInfo><Affiliation>Second Institution</Affiliation><Identifier Source="GRID">grid.2</Identifier></AffiliationInfo>
        </Author>
        <Author><CollectiveName>Fixture Consortium</CollectiveName>
          <AffiliationInfo><Affiliation>First Institution</Affiliation><Identifier Source="ROR">shared</Identifier></AffiliationInfo>
        </Author>
        <Author><LastName>Becker</LastName><ForeName>J&#252;rgen</ForeName></Author>
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
    assert_eq!(citation.authors[2].name, "Jürgen Becker");
    assert!(citation.authors[2].affiliations.is_empty());
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
    for xml in [
        "<PubmedArticleSet />",
        "<eFetchResult><ERROR>bad id</ERROR></eFetchResult>",
    ] {
        assert_eq!(
            parse_citation_xml("1", xml),
            Err(PubMedCitationErrorKind::NotFound)
        );
    }

    for xml in [
        r#"<PubmedArticleSet><PubmedArticle><MedlineCitation><PMID>1</PMID><Article><AuthorList><Author><ForeName>Nameless</ForeName></Author></AuthorList></Article></MedlineCitation></PubmedArticle></PubmedArticleSet>"#,
        r#"<PubmedArticleSet><PubmedArticle><MedlineCitation><PMID>1</PMID><MeshHeadingList><MeshHeading><DescriptorName MajorTopicYN="Maybe">Term</DescriptorName></MeshHeading></MeshHeadingList></MedlineCitation></PubmedArticle></PubmedArticleSet>"#,
        r#"<PubmedArticleSet><PubmedArticle><MedlineCitation><PMID>1</PMID><Article><AuthorList><Author><LastName>Author</LastName><AffiliationInfo><Affiliation>Institution</Affiliation><Identifier Source="ROR"> </Identifier></AffiliationInfo></Author></AuthorList></Article></MedlineCitation></PubmedArticle></PubmedArticleSet>"#,
        "<PubmedArticleSet>",
    ] {
        assert_eq!(
            parse_citation_xml("1", xml),
            Err(PubMedCitationErrorKind::Parse)
        );
    }
}

#[test]
fn citation_parser_enforces_node_and_entity_limits() {
    let mut over_limit = String::from("<PubmedArticleSet>");
    for _ in 0..PUBMED_CITATION_NODE_LIMIT {
        over_limit.push_str("<Node />");
    }
    over_limit.push_str("</PubmedArticleSet>");
    assert_eq!(
        parse_citation_xml("1", &over_limit),
        Err(PubMedCitationErrorKind::Parse)
    );

    let entity_loop = r#"<!DOCTYPE root [<!ENTITY a "&b;"><!ENTITY b "&a;">]><root>&a;</root>"#;
    let entity_amplification = r#"<!DOCTYPE root [
      <!ENTITY a "lol">
      <!ENTITY b "&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;">
      <!ENTITY c "&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;">
      <!ENTITY d "&c;&c;&c;&c;&c;&c;&c;&c;&c;&c;">
    ]><root>&d;</root>"#;
    for xml in [entity_loop, entity_amplification] {
        assert_eq!(
            parse_citation_xml("1", xml),
            Err(PubMedCitationErrorKind::Parse)
        );
    }

    let entity_value = "x".repeat(1_024);
    let entity_references = "&flat;".repeat(1_000);
    let flat_entity_expansion = format!(
        "<!DOCTYPE PubmedArticleSet [<!ENTITY flat \"{entity_value}\">]>\
         <PubmedArticleSet><PubmedArticle><MedlineCitation><PMID>1</PMID>\
         <Article><AuthorList><Author><LastName>{entity_references}</LastName></Author>\
         </AuthorList></Article></MedlineCitation></PubmedArticle></PubmedArticleSet>"
    );
    assert_eq!(
        parse_citation_xml("1", &flat_entity_expansion),
        Err(PubMedCitationErrorKind::Parse)
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

    assert_eq!(
        PubMedClient::decode_citation_response(
            StatusCode::OK,
            Some(&HeaderValue::from_static("application/xml")),
            vec![0xff],
        ),
        Err(PubMedCitationErrorKind::InvalidResponse)
    );

    for (status, expected) in [
        (
            StatusCode::TOO_MANY_REQUESTS,
            PubMedCitationErrorKind::RateLimited,
        ),
        (StatusCode::NOT_FOUND, PubMedCitationErrorKind::NotFound),
        (StatusCode::GONE, PubMedCitationErrorKind::NotFound),
        (StatusCode::BAD_GATEWAY, PubMedCitationErrorKind::Http),
    ] {
        assert_eq!(
            PubMedClient::decode_citation_response(
                status,
                Some(&HeaderValue::from_static("text/plain")),
                b"raw-body-sentinel".to_vec(),
            ),
            Err(expected)
        );
    }
    assert_eq!(
        PubMedClient::decode_citation_response(
            StatusCode::OK,
            Some(&HeaderValue::from_static("text/html")),
            b"raw-body-sentinel".to_vec(),
        ),
        Err(PubMedCitationErrorKind::InvalidResponse)
    );
}

#[test]
fn citation_request_errors_are_payload_free_and_total() {
    let http = reqwest::Client::new().get("://").build().unwrap_err();
    assert_eq!(
        PubMedClient::citation_request_error(BioMcpError::Http(http)),
        PubMedCitationErrorKind::Network
    );
    let middleware = reqwest_middleware::Error::middleware(std::io::Error::other(
        "https://example.test/?api_key=secret-sentinel",
    ));
    assert_eq!(
        PubMedClient::citation_request_error(BioMcpError::HttpMiddleware(middleware)),
        PubMedCitationErrorKind::Network
    );
    assert_eq!(
        PubMedClient::citation_request_error(BioMcpError::BodyLimit {
            source_name: "raw-body-sentinel".into(),
            max_bytes: 1,
        }),
        PubMedCitationErrorKind::ResponseTooLarge
    );
    assert_eq!(
        PubMedClient::citation_request_error(BioMcpError::InvalidArgument(
            "parser-internal-sentinel".into(),
        )),
        PubMedCitationErrorKind::InvalidResponse
    );
}

#[test]
fn decode_json_maps_http_and_content_type_errors() {
    let http = PubMedClient::decode_esearch_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
    )
    .unwrap_err();
    let msg = format!("{http:?}");
    assert_eq!(http.code(), "api");
    assert!(msg.contains("PubMed"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");

    let html = HeaderValue::from_static("text/html");
    let content_type = PubMedClient::decode_esearch_response(
        StatusCode::OK,
        Some(&html),
        b"<html><body>error</body></html>",
    )
    .unwrap_err();
    assert!(format!("{content_type:?}").contains("HTML"));
}
