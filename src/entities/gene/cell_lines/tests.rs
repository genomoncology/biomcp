//! Ticket 1213: fixture-backed tests for the HPA cell line helper. No network.

use reqwest::StatusCode;

use super::*;
use crate::sources::cellosaurus::{
    CellosaurusRecord, IDENTIFIER_FIELDS, decode_body, identifier_batch_query,
};
use crate::sources::hpa::HPA_CELL_LINE_GROUPS;

macro_rules! hpa_fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/hpa/",
            $name
        ))
    };
}

macro_rules! cellosaurus_fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/cellosaurus/",
            $name
        ))
    };
}

const FLT3_ENSEMBL: &str = "ENSG00000122025";
const LEUKEMIA_BATCHES: [&[u8]; 5] = [
    cellosaurus_fixture!("search_id_leukemia_batch1_20260918.json"),
    cellosaurus_fixture!("search_id_leukemia_batch2_20260918.json"),
    cellosaurus_fixture!("search_id_leukemia_batch3_20260918.json"),
    cellosaurus_fixture!("search_id_leukemia_batch4_20260918.json"),
    cellosaurus_fixture!("search_id_leukemia_batch5_20260918.json"),
];

fn leukemia_expressions() -> Vec<HpaCellLineExpression> {
    HpaClient::decode_cell_line_rna(
        "https://www.proteinatlas.org/api/search_download.php",
        StatusCode::OK,
        hpa_fixture!("cell_rna_leukemia_flt3_20260918.json"),
        FLT3_ENSEMBL,
    )
    .expect("the recorded FLT3 leukemia response")
}

fn batch_records(index: usize) -> Vec<CellosaurusRecord> {
    decode_body(
        "https://api.cellosaurus.org/search/cell-line",
        StatusCode::OK,
        LEUKEMIA_BATCHES[index],
    )
    .expect("a recorded batch window")
    .expect("a body")
    .cell_line_list
}

/// The whole group, resolved through the five recorded batch windows.
fn resolved_leukemia_accessions() -> (Vec<String>, Vec<Option<String>>) {
    let names: Vec<String> = leukemia_expressions()
        .iter()
        .map(|row| row.name.clone())
        .collect();
    let mut accessions = Vec::new();
    for (index, batch) in identifier_batches(&names).into_iter().enumerate() {
        let records = batch_records(index);
        for name in &batch {
            accessions.push(unique_identifier_accession(&records, name));
        }
    }
    (names, accessions)
}

// Acceptance 2: the parser keeps the requested gene, holds the group in key
// order, and reads the recorded MOLM-13 value.

#[test]
fn parser_keeps_the_requested_gene_and_the_recorded_group_order() {
    let rows = leukemia_expressions();

    assert_eq!(rows.len(), 93);
    assert_eq!(rows[0].name, "697");
    assert_eq!(rows[rows.len() - 1].name, "UKE-1");
    let molm13 = rows
        .iter()
        .find(|row| row.name == "MOLM-13")
        .expect("MOLM-13 is in the leukemia group");
    assert_eq!(molm13.ntpm, Some(166.1));
}

#[test]
fn parser_ignores_an_object_for_another_gene_and_answers_empty_when_none_matches() {
    let body =
        br#"[{"Gene":"BRAF","Ensembl":"ENSG00000157764","Cell line RNA - HL-60 [nTPM]":"1.0"}]"#;

    let rows =
        HpaClient::decode_cell_line_rna("https://hpa.test", StatusCode::OK, body, FLT3_ENSEMBL)
            .expect("a decoded body");

    assert!(rows.is_empty());
}

#[test]
fn parser_keeps_a_non_numeric_value_as_null() {
    let body = br#"[{"Gene":"FLT3","Ensembl":"ENSG00000122025","Cell line RNA - A [nTPM]":"n/a","Cell line RNA - B [nTPM]":"2.5"}]"#;

    let rows =
        HpaClient::decode_cell_line_rna("https://hpa.test", StatusCode::OK, body, FLT3_ENSEMBL)
            .expect("a decoded body");

    assert_eq!(rows[0].ntpm, None);
    assert_eq!(rows[1].ntpm, Some(2.5));
}

// Acceptance 3: the join runs five batch requests and no sixth, and the names
// resolve to the accessions Cellosaurus published.

#[test]
fn the_group_is_resolved_in_five_batches_and_no_sixth() {
    let (names, _) = resolved_leukemia_accessions();
    let batches = identifier_batches(&names);

    assert_eq!(batches.len(), 5);
    assert_eq!(
        batches.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![20, 20, 20, 20, 13]
    );
    assert_eq!(batches[0][0], "697");

    let query = identifier_batch_query(&batches[0]);
    assert!(query.starts_with("id:(\"697\" OR "));
    assert!(query.ends_with(") AND ox:9606"));
    assert_eq!(IDENTIFIER_FIELDS, ["ac", "id"]);
}

#[test]
fn every_leukemia_name_resolves_to_one_human_accession() {
    let (names, accessions) = resolved_leukemia_accessions();
    let rows = build_rows(&leukemia_expressions(), &accessions);

    let unresolved: Vec<&str> = rows
        .iter()
        .filter(|row| row.accession.is_none())
        .map(|row| row.name.as_str())
        .collect();
    assert_eq!(unresolved, Vec::<&str>::new());
    assert_eq!(names.len(), 93);

    let accession_of = |name: &str| {
        rows.iter()
            .find(|row| row.name == name)
            .and_then(|row| row.accession.clone())
    };
    assert_eq!(accession_of("MOLM-13"), Some("CVCL_2119".to_string()));
    assert_eq!(accession_of("OCI-AML-3"), Some("CVCL_1844".to_string()));
    // The 1202 traps: NB4 is not SJNB-4, KG-1 is not KG1, HL-60 is not
    // HL-60(TB). The matcher accepts an identifier and never a synonym.
    assert_eq!(accession_of("NB4"), Some("CVCL_0005".to_string()));
    assert_eq!(accession_of("KG-1"), Some("CVCL_0374".to_string()));
    assert_eq!(accession_of("HL-60"), Some("CVCL_0002".to_string()));
    assert_eq!(accession_of("K-562"), Some("CVCL_0004".to_string()));
}

#[test]
fn a_name_no_window_claims_gets_no_accession_and_the_note() {
    let records = batch_records(0);
    assert_eq!(
        unique_identifier_accession(&records, "not-a-cell-line-name"),
        None
    );

    let rows = build_rows(
        &[HpaCellLineExpression {
            name: "not-a-cell-line-name".to_string(),
            ntpm: Some(1.0),
        }],
        &[None],
    );
    assert_eq!(rows[0].accession, None);
    assert_eq!(rows[0].note.as_deref(), Some(GENE_CELL_LINES_NO_MATCH_NOTE));
}

// Acceptance 4: a Cellosaurus failure leaves the rows intact, and an empty
// window is not a failure.

#[test]
fn a_cellosaurus_failure_leaves_the_rows_with_no_accession_and_one_note() {
    let expressions = leukemia_expressions();
    let (accessions, notes) = unresolved_accessions(expressions.len());
    let rows = build_rows(&expressions, &accessions);

    assert_eq!(notes, vec![GENE_CELL_LINES_UNAVAILABLE_NOTE.to_string()]);
    assert_eq!(rows.len(), 93);
    assert!(rows.iter().all(|row| row.accession.is_none()));
    assert_eq!(rows[0].ntpm, Some(23.0));
}

#[test]
fn an_empty_window_gives_no_accession_and_no_failure_note() {
    let empty = decode_body(
        "https://api.cellosaurus.org/search/cell-line",
        StatusCode::OK,
        br#"{"Cellosaurus":{"cell-line-list":[]}}"#,
    )
    .expect("an empty window is a normal answer")
    .expect("a body")
    .cell_line_list;

    assert_eq!(unique_identifier_accession(&empty, "MOLM-13"), None);
}

// Acceptance 5: paging keeps the total.

#[test]
fn limit_and_offset_page_the_rows_and_keep_the_total() {
    let expressions = leukemia_expressions();
    let (_, accessions) = resolved_leukemia_accessions();
    let payload = GeneCellLines {
        source: GENE_CELL_LINES_SOURCE.to_string(),
        gene: "FLT3".to_string(),
        ensembl_id: FLT3_ENSEMBL.to_string(),
        group: "leukemia".to_string(),
        total: expressions.len(),
        data_as_of: HPA_CELL_LINE_FILE_DATE.to_string(),
        data_as_of_kind: "release".to_string(),
        notes: Vec::new(),
        rows: build_rows(&expressions, &accessions),
    };

    let page = payload.clone().page(2, 3);
    assert_eq!(page.total, 93);
    assert_eq!(
        page.rows
            .iter()
            .map(|row| row.name.as_str())
            .collect::<Vec<_>>(),
        vec!["AML-193", "BDCM", "BV-173"]
    );

    let last = payload.page(90, 100);
    assert_eq!(last.rows.len(), 3);
    assert_eq!(last.total, 93);
}

// Acceptance 7 and 8: the recorded file date, and one license value in the
// attribution line, the licensing page, and the source registry.

#[test]
fn the_attribution_line_names_the_recorded_file_date_and_the_confirmed_license() {
    assert_eq!(HPA_CELL_LINE_FILE_DATE, "2025-11-05");
    assert_eq!(
        gene_cell_lines_attribution(HPA_CELL_LINE_FILE_DATE),
        "Human Protein Atlas, files dated 2025-11-05, CC BY 4.0. proteinatlas.org"
    );
}

#[test]
fn the_license_agrees_across_the_attribution_line_and_both_registries() {
    let registry: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/reference/sources.json"
    )))
    .expect("the source registry");
    let hpa = registry
        .as_array()
        .expect("a list of sources")
        .iter()
        .find(|source| source["id"] == "human-protein-atlas")
        .expect("the Human Protein Atlas row");
    let summary = hpa["license_summary"].as_str().expect("a license summary");
    assert!(
        summary.starts_with(GENE_CELL_LINES_LICENSE),
        "sources.json says {summary}"
    );

    let licensing = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/reference/source-licensing.md"
    ));
    assert!(licensing.contains(&format!("- License / terms summary: {summary}")));
    assert!(
        !licensing.contains("| Human Protein Atlas | 3 | direct_api | none | CC BY-SA 4.0"),
        "the licensing table still carries the retired CC BY-SA 4.0 reading"
    );
    assert!(gene_cell_lines_attribution("2025-11-05").contains(GENE_CELL_LINES_LICENSE));
}

// Acceptance 9: an HTML body served with HTTP 200 is a provider error.

#[test]
fn an_html_body_with_http_200_is_a_provider_error_naming_the_url() {
    let error = HpaClient::decode_cell_line_rna(
        "https://www.proteinatlas.org/api/search_download.php?search=ENSG00000122025",
        StatusCode::OK,
        b"<!DOCTYPE html><html><body>verify you are human</body></html>",
        FLT3_ENSEMBL,
    )
    .expect_err("an HTML page where JSON was expected is a provider error");

    let message = format!("{error:?}");
    assert_eq!(error.code(), "api");
    assert!(
        message.contains("search_download.php?search=ENSG00000122025"),
        "got: {message}"
    );
    assert!(
        message.contains("HTML page where JSON was expected"),
        "got: {message}"
    );
}

#[test]
fn the_group_list_is_the_thirty_the_data_access_page_publishes() {
    assert_eq!(HPA_CELL_LINE_GROUPS.len(), 30);
    assert!(HPA_CELL_LINE_GROUPS.contains(&"leukemia"));
    assert!(HPA_CELL_LINE_GROUPS.contains(&"neuroblastoma"));
    assert!(HPA_CELL_LINE_GROUPS.contains(&"non-cancerous"));
    let mut sorted = HPA_CELL_LINE_GROUPS.to_vec();
    sorted.sort_unstable();
    assert_eq!(sorted, HPA_CELL_LINE_GROUPS.to_vec());
}
