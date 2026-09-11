use super::super::NciCtsClient;
use reqwest::StatusCode;

#[test]
fn search_response_uses_biodata_page_without_aliases() {
    let bytes =
        include_bytes!("../../../../testdata/sources/nci_cts/search_melanoma_20260811.json");
    let page =
        NciCtsClient::decode_search_response(StatusCode::OK, bytes).expect("receipted NCI page");
    let results = page.results().expect("present data");
    assert_eq!(results.len(), 1);
    assert_eq!(page.total_count(), Some(2112));
    assert_eq!(
        results[0].projection().value().brief_title(),
        "Shorter Chemo-Immunotherapy Without Anthracycline Drugs for Early Triple Negative Breast Cancer"
    );

    let aliases = NciCtsClient::decode_search_response(
        StatusCode::OK,
        br#"{"trials":[],"total_count":1,"totalCount":2}"#,
    )
    .expect("unproved aliases are ignored");
    assert!(aliases.results().is_none());
    assert_eq!(aliases.total_count(), None);
}

#[test]
fn malformed_nci_search_values_are_sanitized() {
    const SENTINEL: &str = "NCI-SEARCH-PRIVATE-SENTINEL-0117";
    let body = format!(
        r#"{{"data":[{{"nct_id":"NCT-private","brief_title":"{SENTINEL}","current_trial_status":"Active"}}]}}"#
    );
    let error = NciCtsClient::decode_search_response(StatusCode::OK, body.as_bytes())
        .expect_err("noncanonical NCT ID must fail");
    let rendered = format!("{error:?} {error}");
    assert!(!rendered.contains(SENTINEL));
    assert!(!rendered.contains("NCT-private"));
    assert!(rendered.contains("invalid_projection"));
}
