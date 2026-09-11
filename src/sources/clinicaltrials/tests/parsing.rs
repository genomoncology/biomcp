use super::super::ClinicalTrialsClient;
use reqwest::StatusCode;

#[test]
fn search_response_uses_biodata_page_without_rewriting_fixture_bytes() {
    let bytes =
        include_bytes!("../../../../testdata/sources/ctgov/search_keytruda_limit3_20260811.json");
    let page = ClinicalTrialsClient::decode_search_response(StatusCode::OK, bytes)
        .expect("receipted CTGov page");
    let results = page.results().expect("present studies");
    assert_eq!(results.len(), 3);
    assert_eq!(page.total_count(), Some(2924));
    assert_eq!(
        page.next_page_token(),
        Some("ZVt07cGHkvI2wRk2CJf6_LLq14bEL8swd7KrgP4YnzmVtA")
    );
    assert_eq!(
        results[0].projection().value().brief_title(),
        "Dose Escalation/Expansion Study of Mavrostobart (PT199), an Anti-CD73 mAb, Administered Alone and in Combination With a PD-1 Inhibitor or Chemotherapy (the MORNINGSTAR Study)"
    );
}

#[test]
fn malformed_search_values_are_sanitized() {
    const SENTINEL: &str = "CTGOV-SEARCH-PRIVATE-SENTINEL-0117";
    let body = format!(
        r#"{{"studies":[{{"protocolSection":{{"identificationModule":{{"nctId":"bad","briefTitle":"{SENTINEL}"}},"statusModule":{{"overallStatus":"RECRUITING"}}}}}}]}}"#
    );
    let error = ClinicalTrialsClient::decode_search_response(StatusCode::OK, body.as_bytes())
        .expect_err("noncanonical NCT ID must fail");
    let rendered = format!("{error:?} {error}");
    assert!(!rendered.contains(SENTINEL));
    assert!(!rendered.contains("bad"));
    assert!(rendered.contains("invalid_projection"));
}

#[test]
fn intervention_rejection_remains_classified() {
    const SENTINEL: &str = "CTGOV-PROVIDER-BODY-SENTINEL-0117";
    let body = format!("Error parsing query in Intervention / treatment: {SENTINEL}");
    let error =
        ClinicalTrialsClient::decode_search_response(StatusCode::BAD_REQUEST, body.as_bytes())
            .expect_err("provider rejection must fail");
    assert!(matches!(
        error,
        crate::error::BioMcpError::CtGovInterventionQueryRejected
    ));
    assert!(!format!("{error:?} {error}").contains(SENTINEL));
}

#[test]
fn search_params_debug_redacts_queries_and_cursor() {
    const SENTINEL: &str = "CTGOV-PARAM-PRIVATE-SENTINEL-0117";
    let params = super::super::CtGovSearchParams {
        condition: Some(SENTINEL.into()),
        intervention: Some(SENTINEL.into()),
        query_term: Some(SENTINEL.into()),
        page_token: Some(SENTINEL.into()),
        page_size: 1,
        ..Default::default()
    };
    assert!(!format!("{params:?}").contains(SENTINEL));
}

#[test]
fn adverse_event_parser_debug_redacts_every_nested_value() {
    const SENTINEL: &str = "ADVERSE-EVENT-PRIVATE-SENTINEL-0117";
    let page: super::super::CtGovAdverseEventSearchPage = serde_json::from_value(
        serde_json::json!({
            "studies": [{
                "protocolSection": {"identificationModule": {"nctId": "NCT00000001"}},
                "resultsSection": {"adverseEventsModule": {
                    "seriousEvents": [{"term": SENTINEL, "stats": [{"groupId": SENTINEL, "numAffected": 1, "numAtRisk": 2}]}]
                }}
            }],
            "nextPageToken": SENTINEL
        }),
    )
    .unwrap();
    let rendered = format!("{page:?} {:?}", page.studies[0]);
    assert!(!rendered.contains(SENTINEL));
}
