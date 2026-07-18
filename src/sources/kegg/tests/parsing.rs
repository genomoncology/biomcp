//! Tier 3 - text response parsing and local result shaping. Pure: feeds plain
//! text into parsers and validates output. No network.

use reqwest::StatusCode;

use super::super::*;

#[test]
fn parse_search_response_keeps_human_rows_only() {
    let rows = parse_search_response(
        "path:hsa04010\tMAPK signaling pathway - Homo sapiens (human)\n\
         path:map04010\tMAPK signaling pathway - Reference pathway\n\
         path:mmu04010\tMAPK signaling pathway - Mus musculus (mouse)\n\
         bad line\n",
        10,
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "hsa04010");
    assert_eq!(
        rows[0].name,
        "MAPK signaling pathway - Homo sapiens (human)"
    );
}

#[test]
fn parse_search_response_normalizes_bare_reference_map_to_human() {
    let rows = parse_search_response("path:map04010\tMAPK signaling pathway\n", 10);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "hsa04010");
    assert_eq!(
        rows[0].name,
        "MAPK signaling pathway - Homo sapiens (human)"
    );
}

#[test]
fn parse_search_response_dedupes_normalized_and_explicit_human_id() {
    let rows = parse_search_response(
        "path:map04010\tMAPK signaling pathway\n\
         path:hsa04010\tMAPK signaling pathway - Homo sapiens (human)\n",
        10,
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "hsa04010");
}

#[test]
fn parse_pathway_record_extracts_summary_and_genes() {
    let record = parse_pathway_record(
        "ENTRY       hsa05200           Pathway\n\
         NAME        Pathways in cancer\n\
         DESCRIPTION Cancer overview pathway.\n\
         GENE        673    BRAF; B-Raf proto-oncogene\n\
                     1956   EGFR; epidermal growth factor receptor\n\
         ///\n",
    )
    .expect("record");

    assert_eq!(record.id, "hsa05200");
    assert_eq!(record.name, "Pathways in cancer");
    assert_eq!(record.summary.as_deref(), Some("Cancer overview pathway."));
    assert_eq!(record.genes, vec!["BRAF".to_string(), "EGFR".to_string()]);
}

#[test]
fn decode_text_response_maps_status_and_utf8_errors() {
    let err = KeggClient::decode_text_response(StatusCode::BAD_GATEWAY, b"upstream".to_vec())
        .unwrap_err();
    assert!(format!("{err:?}").contains("HTTP 502 Bad Gateway"));

    let err = KeggClient::decode_text_response(StatusCode::OK, vec![0xff]).unwrap_err();
    assert!(format!("{err:?}").contains("valid UTF-8"));
}
