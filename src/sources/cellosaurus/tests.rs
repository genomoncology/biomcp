use super::*;

fn client() -> CellosaurusClient {
    CellosaurusClient {
        client: crate::sources::shared_client().expect("shared client"),
        base: Cow::Borrowed("https://api.cellosaurus.org"),
    }
}

#[test]
fn record_url_asks_for_the_card_projection_without_cross_references() {
    let url = client()
        .record_url("CVCL_2119", CARD_FIELDS)
        .expect("record url");
    assert_eq!(
        url,
        "https://api.cellosaurus.org/cell-line/CVCL_2119?format=json&fields=ac%2Cid%2Csy%2Cox%2Cdi%2Cca%2Csx%2Cag"
    );
    assert!(!url.contains("dr"));
}

#[test]
fn name_search_targets_the_identifier_and_synonym_field() {
    let url = client()
        .search_url("idsy:\"MOLM13\"", SEARCH_FIELDS, SEARCH_ROWS)
        .expect("search url");
    assert!(url.starts_with("https://api.cellosaurus.org/search/cell-line?q=idsy%3A%22MOLM13%22"));
    assert!(url.contains("rows=1000"));
}

#[test]
fn solr_escaping_covers_the_quote_and_the_backslash() {
    assert_eq!(escape_solr_value("MV4;11"), "MV4;11");
    assert_eq!(escape_solr_value("HL-60\"TB\""), "HL-60\\\"TB\\\"");
    assert_eq!(escape_solr_value("a\\b"), "a\\\\b");
}

#[test]
fn a_missing_record_is_a_missing_record_and_not_a_failure() {
    let decoded = decode_body(
        "https://api.cellosaurus.org/cell-line/CVCL_ZZZZ",
        StatusCode::NOT_FOUND,
        b"not found",
    )
    .expect("404 decodes");
    assert!(decoded.is_none());
}

#[test]
fn an_html_page_served_as_json_is_a_provider_error_naming_the_url() {
    let error = decode_body(
        "https://api.cellosaurus.org/cell-line/CVCL_2119",
        StatusCode::OK,
        b"<!DOCTYPE html><html><body>Verify you are human</body></html>",
    )
    .expect_err("HTML body fails");
    let BioMcpError::Api { message, .. } = error else {
        panic!("HTML body is a provider error");
    };
    assert!(
        message.contains("https://api.cellosaurus.org/cell-line/CVCL_2119"),
        "error names the URL: {message}"
    );
    assert!(
        message.contains("HTML"),
        "error names the HTML page: {message}"
    );
}

#[test]
fn a_record_splits_primary_and_secondary_accessions_and_names() {
    let body = decode_body(
        "https://api.cellosaurus.org/cell-line/CVCL_2119",
        StatusCode::OK,
        br#"{"Cellosaurus":{"cell-line-list":[{
            "accession-list":[{"type":"primary","value":"CVCL_2119"},{"type":"secondary","value":"CVCL_X999"}],
            "name-list":[{"type":"synonym","value":"MOLM13"},{"type":"identifier","value":"MOLM-13"}]
        }]}}"#,
    )
    .expect("decodes")
    .expect("body");
    let record = &body.cell_line_list[0];
    assert_eq!(record.primary_accession(), Some("CVCL_2119"));
    assert_eq!(record.secondary_accessions(), vec!["CVCL_X999".to_string()]);
    assert_eq!(record.identifier(), Some("MOLM-13"));
    assert_eq!(record.synonyms(), vec!["MOLM13".to_string()]);
}

#[test]
fn a_record_recorded_without_variants_carries_no_variation_rows() {
    let body = decode_body(
        "https://api.cellosaurus.org/cell-line/CVCL_0007",
        StatusCode::OK,
        br#"{"Cellosaurus":{"cell-line-list":[{"accession-list":[{"type":"primary","value":"CVCL_0007"}]}]}}"#,
    )
    .expect("decodes")
    .expect("body");
    assert!(body.cell_line_list[0].sequence_variation_list.is_empty());
}

#[test]
fn a_variation_source_yields_its_pubmed_id() {
    let source = CellosaurusVariationSource {
        reference: Some(CellosaurusVariationReference {
            resource_internal_ref: Some("PubMed=31739141".to_string()),
        }),
    };
    assert_eq!(source.pubmed_id(), Some("31739141".to_string()));
    let other = CellosaurusVariationSource { reference: None };
    assert_eq!(other.pubmed_id(), None);
}

#[test]
fn release_info_reads_the_version_and_the_date() {
    let body = decode_body(
        "https://api.cellosaurus.org/release-info",
        StatusCode::OK,
        br#"{"Cellosaurus":{"header":{"release":{"version":"56.0","updated":"2026-06-25","nb-cell-lines":"168970"}}}}"#,
    )
    .expect("decodes")
    .expect("body");
    let release = body.header.unwrap().release.unwrap();
    assert_eq!(release.version.as_deref(), Some("56.0"));
    assert_eq!(release.updated.as_deref(), Some("2026-06-25"));
}
