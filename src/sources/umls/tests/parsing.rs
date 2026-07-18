//! Tier 3 - response parsing and local result shaping. Pure: feeds JSON bytes
//! into decode helpers and validates output. No network.

use reqwest::StatusCode;
use reqwest::header::HeaderValue;

use super::super::*;

#[test]
fn decode_search_response_filters_none_hits_and_concepts_keep_xrefs() {
    let content_type = HeaderValue::from_static("application/json");
    let search: UmlsSearchEnvelope = UmlsClient::decode_json_response(
        StatusCode::OK,
        Some(&content_type),
        br#"{
            "result": {
                "results": [
                    {
                        "ui": "C0010674",
                        "name": "Cystic Fibrosis",
                        "uri": "https://example.org/C0010674",
                        "semanticTypes": ["Disease or Syndrome"]
                    },
                    {"ui": "NONE", "name": "skip", "uri": "", "semanticTypes": []}
                ]
            }
        }"#,
    )
    .expect("search response should decode");

    let hits = UmlsClient::search_hits(search);
    assert_eq!(hits.len(), 1);

    let concept = UmlsClient::concept_from_hit(
        hits.into_iter().next().unwrap(),
        vec![UmlsXref {
            vocab: "ICD10CM".to_string(),
            id: "E84".to_string(),
            label: "Cystic fibrosis".to_string(),
        }],
    );

    assert_eq!(concept.cui, "C0010674");
    assert_eq!(concept.xrefs[0].vocab, "ICD10CM");
    assert_eq!(concept.xrefs[0].id, "E84");
}

#[test]
fn map_atoms_keeps_english_xrefs_and_dedupes_source_id_pairs() {
    let atoms: UmlsAtomsEnvelope = serde_json::from_value(serde_json::json!({
        "result": [
            {
                "rootSource": "ICD10CM",
                "code": "https://example.org/source/ICD10CM/E84",
                "language": "ENG",
                "name": "Cystic fibrosis"
            },
            {
                "rootSource": "ICD10CM",
                "code": "https://example.org/source/ICD10CM/E84",
                "language": "ENG",
                "name": "Duplicate"
            },
            {
                "rootSource": "SNOMEDCT_US",
                "code": "https://example.org/source/SNOMED/123",
                "language": "SPA",
                "name": "Skip non-English"
            }
        ]
    }))
    .expect("atoms fixture should parse");

    let rows = UmlsClient::map_atoms(atoms);

    assert_eq!(
        rows,
        vec![UmlsXref {
            vocab: "ICD10CM".to_string(),
            id: "E84".to_string(),
            label: "Cystic fibrosis".to_string(),
        }]
    );
}

#[test]
fn decode_json_response_maps_http_and_content_type_errors() {
    let content_type = HeaderValue::from_static("application/json");
    let err = UmlsClient::decode_json_response::<UmlsSearchEnvelope>(
        StatusCode::BAD_GATEWAY,
        Some(&content_type),
        br#"{"error":"upstream"}"#,
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("HTTP 502 Bad Gateway"));

    let content_type = HeaderValue::from_static("text/html");
    let err = UmlsClient::decode_json_response::<UmlsSearchEnvelope>(
        StatusCode::OK,
        Some(&content_type),
        b"<html>not json</html>",
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("Unexpected HTML response"));
}
