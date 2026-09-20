//! Tier 3 — response parsing. Pure: feeds committed PharmacoDB fixture bytes to
//! the decoders. No network, no server.

use super::super::*;
use reqwest::StatusCode;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/pharmacodb/",
            $name
        ))
    };
}

const URL: &str = "https://pharmacodb.ca/graphql";

// Ticket 1205 acceptance 1: the client parses each recorded fixture and maps the
// two upstream miss messages to `None`.

#[test]
fn the_cell_line_uid_fixture_carries_the_molm13_record() {
    let record = PharmacoDbClient::decode_cell_line(
        URL,
        StatusCode::OK,
        fixture!("cell_line_uid_molm13_20260918.json"),
    )
    .unwrap()
    .expect("MOLM-13 is a PharmacoDB cell line");

    assert_eq!(record.id, 1248);
    assert_eq!(record.uid, "MOLM13_950_2019");
    assert_eq!(record.name, "MOLM-13");
    assert_eq!(record.accession_id.as_deref(), Some("CVCL_2119"));
}

#[test]
fn the_cell_line_name_fixture_reaches_hl60tb() {
    let record = PharmacoDbClient::decode_cell_line(
        URL,
        StatusCode::OK,
        fixture!("cell_line_name_hl60tb_20260918.json"),
    )
    .unwrap()
    .expect("HL-60(TB) is a PharmacoDB cell line");

    assert_eq!(record.id, 1228);
    assert_eq!(record.name, "HL-60(TB)");
    assert_eq!(record.accession_id.as_deref(), Some("CVCL_A794"));
}

#[test]
fn a_cell_line_pharmacodb_does_not_hold_is_a_miss_and_not_a_failure() {
    let record = PharmacoDbClient::decode_cell_line(
        URL,
        StatusCode::OK,
        fixture!("cell_line_name_missing_20260918.json"),
    )
    .unwrap();

    assert!(record.is_none());
}

#[test]
fn the_compound_fixture_carries_venetoclax() {
    let record = PharmacoDbClient::decode_compound(
        URL,
        StatusCode::OK,
        fixture!("compound_venetoclax_20260918.json"),
    )
    .unwrap()
    .expect("venetoclax is a PharmacoDB compound");

    assert_eq!(record.id, 53572);
    assert_eq!(record.uid, "PDBC02030");
    assert_eq!(record.name, "Venetoclax");
}

#[test]
fn a_compound_pharmacodb_does_not_hold_is_a_miss_and_not_a_failure() {
    let record = PharmacoDbClient::decode_compound(
        URL,
        StatusCode::OK,
        fixture!("compound_missing_20260918.json"),
    )
    .unwrap();

    assert!(record.is_none());
}

#[test]
fn another_graphql_error_is_a_source_error_naming_the_url_and_the_operation() {
    let body = br#"{"errors":[{"message":"Unknown column 'cell.name'"}],"data":null}"#;
    let error = PharmacoDbClient::decode_experiments(URL, StatusCode::OK, body)
        .expect_err("an upstream SQL error is a failure");
    let message = format!("{error:?}");

    assert_eq!(error.code(), "api");
    assert!(message.contains(URL), "got: {message}");
    assert!(message.contains("Unknown column"), "got: {message}");
    assert!(message.contains("experiments"), "got: {message}");
}

// Ticket 1205 acceptance 11: an HTML body served with HTTP 200 is a provider
// error that names the URL and the operation, and no row survives it.

#[test]
fn an_html_body_served_with_http_200_is_a_provider_error() {
    let body = b"<!DOCTYPE html>\n<html><body>Verify you are human</body></html>";
    for (operation, error) in [
        (
            "experiments",
            PharmacoDbClient::decode_experiments(URL, StatusCode::OK, body).unwrap_err(),
        ),
        (
            "experiment counts",
            PharmacoDbClient::decode_experiment_counts(URL, StatusCode::OK, body).unwrap_err(),
        ),
        (
            "cell_line",
            PharmacoDbClient::decode_cell_line(URL, StatusCode::OK, body).unwrap_err(),
        ),
        (
            "compound",
            PharmacoDbClient::decode_compound(URL, StatusCode::OK, body).unwrap_err(),
        ),
    ] {
        let text = format!("{error:?}");
        assert_eq!(error.code(), "api");
        assert!(text.contains(URL), "got: {text}");
        assert!(text.contains(operation), "got: {text}");
        assert!(text.contains("HTML page"), "got: {text}");
    }
}

// Ticket 1205 acceptance 1 and 2: the counts projection sums per dataset in name
// order, and the row projection keeps every published metric as it stands.

#[test]
fn the_cell_line_counts_fixture_sums_per_dataset_in_name_order() {
    let counts = PharmacoDbClient::decode_experiment_counts(
        URL,
        StatusCode::OK,
        fixture!("experiment_counts_cell_line_1248_20260918.json"),
    )
    .unwrap();

    let rows: Vec<(&str, usize)> = counts
        .iter()
        .map(|row| (row.name.as_str(), row.count))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("CTRPv2", 416),
            ("gCSI", 35),
            ("GDSC1", 426),
            ("GDSC2", 240),
        ]
    );
    assert_eq!(counts.iter().map(|row| row.count).sum::<usize>(), 1117);
}

#[test]
fn the_compound_counts_fixture_sums_the_venetoclax_datasets() {
    let counts = PharmacoDbClient::decode_experiment_counts(
        URL,
        StatusCode::OK,
        fixture!("experiment_counts_compound_53572_20260918.json"),
    )
    .unwrap();

    let rows: Vec<(&str, usize)> = counts
        .iter()
        .map(|row| (row.name.as_str(), row.count))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("CTRPv2", 536),
            ("GDSC1", 981),
            ("GDSC2", 1052),
            ("NCI60", 113),
            ("PRISM", 926),
        ]
    );
    assert_eq!(counts.iter().map(|row| row.count).sum::<usize>(), 3608);
}

#[test]
fn the_pair_fixture_carries_both_venetoclax_rows_for_molm13() {
    let rows = PharmacoDbClient::decode_experiments(
        URL,
        StatusCode::OK,
        fixture!("experiments_pair_53572_1248_20260918.json"),
    )
    .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].experiment_id, 4348032);
    assert_eq!(rows[0].dataset, "GDSC1");
    assert_eq!(rows[0].cell_line_name, "MOLM-13");
    assert_eq!(rows[0].compound_name, "Venetoclax");
    assert_eq!(rows[0].compound_uid, "PDBC02030");
    assert_eq!(rows[0].tissue.as_deref(), Some("Myeloid"));
    assert_eq!(rows[0].aac, Some(0.0));
    assert_eq!(rows[0].ic50, None);
    assert_eq!(rows[0].dss1, None);
    assert_eq!(rows[1].dataset, "GDSC2");
    assert_eq!(rows[1].aac, Some(0.805_489_38));
    assert_eq!(rows[1].ic50, Some(0.002_849_89));
}

#[test]
fn the_cell_line_row_fixture_keeps_every_row_and_every_null_metric() {
    let rows = PharmacoDbClient::decode_experiments(
        URL,
        StatusCode::OK,
        fixture!("experiments_cell_line_1248_20260918.json"),
    )
    .unwrap();

    assert_eq!(rows.len(), 1117);
    assert_eq!(rows.iter().filter(|row| row.ic50.is_none()).count(), 241);
    // The same compound and dataset repeat, and BioMCP merges nothing.
    let repeated = rows
        .iter()
        .filter(|row| row.compound_name == "Afatinib" && row.dataset == "GDSC2")
        .count();
    assert_eq!(repeated, 5, "MOLM-13 repeats compound and dataset pairs");
}
