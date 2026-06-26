//! Tier 3 - response parsing and local result shaping. Pure: feeds fixture bytes
//! into decode helpers and validates output. No network.

use flate2::Compression;
use flate2::write::GzEncoder;
use reqwest::StatusCode;
use reqwest::header::{HeaderMap, HeaderValue, LINK};
use std::io::Write as _;

use super::super::*;

#[test]
fn decode_search_response_reads_results_total_and_next_page_link() {
    let mut headers = HeaderMap::new();
    headers.insert("x-total-results", HeaderValue::from_static("42"));
    headers.insert(
        LINK,
        HeaderValue::from_static(
            r#"<https://rest.uniprot.org/uniprotkb/search?cursor=abc>; rel="next""#,
        ),
    );

    let page = UniProtClient::decode_search_response(
        StatusCode::OK,
        &headers,
        &super::search_response_json(),
    )
    .unwrap();

    assert_eq!(page.total, Some(42));
    assert_eq!(
        page.next_page_token.as_deref(),
        Some("https://rest.uniprot.org/uniprotkb/search?cursor=abc")
    );
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.results[0].primary_accession, "P15056");
    assert_eq!(
        page.results[0].primary_gene_symbol().as_deref(),
        Some("BRAF")
    );
}

#[test]
fn decode_json_response_maps_http_and_json_errors() {
    let err = UniProtClient::decode_json_response::<UniProtSearchResponse>(
        StatusCode::BAD_GATEWAY,
        br#"{"error":"upstream"}"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("HTTP 502 Bad Gateway"));

    let err =
        UniProtClient::decode_json_response::<UniProtSearchResponse>(StatusCode::OK, b"not json")
            .unwrap_err();
    assert!(err.to_string().contains("Invalid JSON response"));
}

#[test]
fn decode_json_response_accepts_gzip_payload() {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&super::search_response_json())
        .expect("gzip fixture should write");
    let payload = encoder.finish().expect("gzip fixture should finish");

    let response: UniProtSearchResponse =
        UniProtClient::decode_json_response(StatusCode::OK, &payload).unwrap();

    assert_eq!(response.results[0].primary_accession, "P15056");
}

#[test]
fn record_helpers_extract_display_function_and_structures() {
    let record: UniProtRecord = serde_json::from_value(serde_json::json!({
        "primaryAccession": "P15056",
        "proteinDescription": {
            "recommendedName": {"fullName": {"value": " Kinase X "}}
        },
        "comments": [
            {"commentType": "FUNCTION", "texts": [{"value": " Signal transduction. "}]}
        ],
        "uniProtKBCrossReferences": [
            {
                "database": "PDB",
                "id": "1UWH",
                "properties": [
                    {"key": "Method", "value": "X-ray"},
                    {"key": "Resolution", "value": "2.95 A"},
                    {"key": "Chains", "value": "A/B=448-723"}
                ]
            },
            {"database": "PDB", "id": "1UWH"},
            {"database": "AlphaFoldDB", "id": "AF-P15056-F1"},
            {"database": "GO", "id": "GO:0004672"}
        ]
    }))
    .unwrap();

    assert_eq!(record.display_name(), "Kinase X");
    assert_eq!(
        record.function_summary().as_deref(),
        Some("Signal transduction.")
    );
    assert_eq!(
        record.structure_ids(),
        vec!["1UWH".to_string(), "AF-P15056-F1".to_string()]
    );
    assert_eq!(record.structure_count(), 2);
    assert_eq!(record.alphafold_ids(), vec!["AF-P15056-F1".to_string()]);
    let pdb = record.typed_pdb_structures(Some(600));
    assert_eq!(pdb.len(), 2);
    assert_eq!(pdb[0].id, "1UWH");
    assert_eq!(pdb[0].chains.as_deref(), Some("A/B=448-723"));
    assert_eq!(pdb[0].residue_covered, Some(true));
    assert_eq!(
        record.structure_summaries(10),
        vec![
            "1UWH (X-ray, 2.95 A)".to_string(),
            "AF-P15056-F1 (AlphaFold model)".to_string()
        ]
    );
}

#[test]
fn protein_isoforms_prefer_synonyms_and_track_displayed_status() {
    let record: UniProtRecord = serde_json::from_value(serde_json::json!({
        "primaryAccession": "P01116",
        "comments": [
            {
                "commentType": "ALTERNATIVE PRODUCTS",
                "isoforms": [
                    {
                        "name": {"value": "2A"},
                        "synonyms": [{"value": "K-Ras4A"}],
                        "isoformSequenceStatus": "Displayed"
                    },
                    {
                        "name": {"value": "Beta"},
                        "synonyms": [],
                        "isoformSequenceStatus": "described"
                    },
                    {
                        "name": {"value": "  "},
                        "synonyms": [{"value": " "}],
                        "isoformSequenceStatus": "Displayed"
                    }
                ]
            }
        ]
    }))
    .unwrap();

    assert_eq!(
        record.protein_isoforms(),
        vec![
            UniProtProteinIsoformSummary {
                name: "K-Ras4A".to_string(),
                is_displayed: true,
            },
            UniProtProteinIsoformSummary {
                name: "Beta".to_string(),
                is_displayed: false,
            },
        ]
    );
}

#[test]
fn protein_isoforms_fall_back_to_name_when_synonyms_are_missing() {
    let record: UniProtRecord = serde_json::from_value(serde_json::json!({
        "primaryAccession": "O15350",
        "comments": [
            {
                "commentType": "alternative products",
                "isoforms": [
                    {
                        "name": {"value": "Alpha"},
                        "synonyms": [],
                        "isoformSequenceStatus": "displayed"
                    }
                ]
            }
        ]
    }))
    .unwrap();

    assert_eq!(
        record.protein_isoforms(),
        vec![UniProtProteinIsoformSummary {
            name: "Alpha".to_string(),
            is_displayed: true,
        }]
    );
}

#[test]
fn protein_isoforms_return_empty_when_alternative_products_comment_is_missing() {
    let record: UniProtRecord = serde_json::from_value(serde_json::json!({
        "primaryAccession": "P15056",
        "comments": [
            {
                "commentType": "FUNCTION",
                "texts": [{"value": "Kinase."}]
            }
        ]
    }))
    .unwrap();

    assert!(record.protein_isoforms().is_empty());
}

#[test]
fn alternative_protein_names_flatten_short_and_full_names_in_source_order() {
    let record: UniProtRecord = serde_json::from_value(serde_json::json!({
        "primaryAccession": "Q99541",
        "proteinDescription": {
            "recommendedName": {
                "fullName": {"value": "Perilipin-2"}
            },
            "alternativeNames": [
                {
                    "fullName": {"value": "Adipophilin"}
                },
                {
                    "fullName": {"value": "Adipose differentiation-related protein"},
                    "shortNames": [{"value": "ADRP"}]
                }
            ]
        }
    }))
    .unwrap();

    assert_eq!(
        record.alternative_protein_names(),
        vec![
            "Adipophilin".to_string(),
            "ADRP".to_string(),
            "Adipose differentiation-related protein".to_string(),
        ]
    );
}

#[test]
fn alternative_protein_names_trim_deduplicate_and_skip_recommended_name() {
    let record: UniProtRecord = serde_json::from_value(serde_json::json!({
        "primaryAccession": "O60240",
        "proteinDescription": {
            "recommendedName": {
                "fullName": {"value": "Perilipin-1"}
            },
            "alternativeNames": [
                {
                    "fullName": {"value": "  Perilipin-1  "},
                    "shortNames": [
                        {"value": "  "},
                        {"value": "PERI"}
                    ]
                },
                {
                    "fullName": {"value": "Lipid droplet-associated protein"},
                    "shortNames": [{"value": "peri"}]
                }
            ]
        }
    }))
    .unwrap();

    assert_eq!(
        record.alternative_protein_names(),
        vec![
            "PERI".to_string(),
            "Lipid droplet-associated protein".to_string(),
        ]
    );
}

#[test]
fn alternative_protein_names_return_empty_when_alternative_names_are_missing() {
    let record: UniProtRecord = serde_json::from_value(serde_json::json!({
        "primaryAccession": "P15056",
        "proteinDescription": {
            "recommendedName": {
                "fullName": {"value": "Serine/threonine-protein kinase B-raf"}
            }
        }
    }))
    .unwrap();

    assert!(record.alternative_protein_names().is_empty());
}
