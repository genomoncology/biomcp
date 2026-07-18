//! Tier 3 - response parsing and local result shaping. Pure: feeds JSON bytes
//! into decode helpers and validates output. No network.

use reqwest::StatusCode;

use super::super::*;

#[test]
fn strip_html_removes_tags_and_extra_spaces() {
    assert_eq!(strip_html("RAF <b>MAPK</b> cascade"), "RAF MAPK cascade");
}

#[test]
fn map_search_response_extracts_entries_and_limits_results() {
    let resp: ReactomeSearchResponse = serde_json::from_value(serde_json::json!({
        "totalResults": 10,
        "results": [{
            "entries": [
                {"stId": "R-HSA-1", "name": "A <b>pathway</b>"},
                {"id": "R-HSA-2", "name": "B pathway"}
            ]
        }]
    }))
    .unwrap();

    let (rows, total) = ReactomeClient::map_search_response(resp, 2);

    assert_eq!(rows.len(), 2);
    assert_eq!(total, Some(10));
    assert_eq!(rows[0].id, "R-HSA-1");
    assert_eq!(rows[0].name, "A pathway");
}

#[test]
fn map_contained_events_maps_display_names() {
    let resp: Vec<ReactomeContainedEvent> = serde_json::from_value(serde_json::json!([
        {"displayName": "RAS activates RAF"},
        9652817,
        {"displayName": " "}
    ]))
    .unwrap();

    let rows = ReactomeClient::map_contained_events(resp, 10);

    assert_eq!(rows, vec!["RAS activates RAF".to_string()]);
}

#[test]
fn decode_json_response_maps_http_and_json_errors() {
    let err = ReactomeClient::decode_json_response::<ReactomeSearchResponse>(
        StatusCode::BAD_GATEWAY,
        br#"{"error":"upstream"}"#,
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("HTTP 502 Bad Gateway"));

    let err =
        ReactomeClient::decode_json_response::<ReactomeSearchResponse>(StatusCode::OK, b"not json")
            .unwrap_err();
    assert!(matches!(err, BioMcpError::ApiJson { .. }));
}
