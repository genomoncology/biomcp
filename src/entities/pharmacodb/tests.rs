//! Ticket 1205: the shared PharmacoDB shaping rules, replayed from recorded
//! fixture bytes and pure inputs. No network, no server.

use super::*;
use crate::sources::pharmacodb::{PharmacoDbClient, PharmacoExperiment};
use reqwest::StatusCode;

const URL: &str = "https://pharmacodb.ca/graphql";
/// The injected clock every payload test pins.
const FIXED_CLOCK: &str = "2026-09-19T12:00:00Z";

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/pharmacodb/",
            $name
        ))
    };
}

fn molm13_experiments() -> Vec<PharmacoExperiment> {
    PharmacoDbClient::decode_experiments(
        URL,
        StatusCode::OK,
        fixture!("experiments_cell_line_1248_20260918.json"),
    )
    .expect("the recorded MOLM-13 rows")
}

fn venetoclax_experiments() -> Vec<PharmacoExperiment> {
    PharmacoDbClient::decode_experiments(
        URL,
        StatusCode::OK,
        fixture!("experiments_compound_53572_20260918.json"),
    )
    .expect("the recorded venetoclax rows")
}

fn pair_experiments() -> Vec<PharmacoExperiment> {
    PharmacoDbClient::decode_experiments(
        URL,
        StatusCode::OK,
        fixture!("experiments_pair_53572_1248_20260918.json"),
    )
    .expect("the recorded pair rows")
}

fn molm13() -> PharmacoCellLine {
    PharmacoDbClient::decode_cell_line(
        URL,
        StatusCode::OK,
        fixture!("cell_line_uid_molm13_20260918.json"),
    )
    .expect("a decoded body")
    .expect("the MOLM-13 record")
}

fn hl60() -> PharmacoCellLine {
    PharmacoDbClient::decode_cell_line(
        URL,
        StatusCode::OK,
        fixture!("cell_line_name_hl60_20260918.json"),
    )
    .expect("a decoded body")
    .expect("the HL-60 record")
}

// Acceptance 4: the accession check is what makes a name fallback safe.

#[test]
fn a_record_that_claims_the_requested_accession_is_accepted() {
    assert_eq!(
        accept_cell_line("CVCL_2119", Some(molm13())),
        PharmacoCellLineJoin::Found(molm13())
    );
    assert_eq!(
        accept_cell_line("cvcl_2119", Some(molm13())),
        PharmacoCellLineJoin::Found(molm13())
    );
}

#[test]
fn a_record_that_claims_another_accession_is_a_mismatch() {
    // HL-60 (CVCL_0002) is a real PharmacoDB record, and it is not HL-60(TB).
    assert_eq!(
        accept_cell_line("CVCL_A794", Some(hl60())),
        PharmacoCellLineJoin::Mismatch
    );
}

#[test]
fn no_record_at_all_is_missing_and_not_a_mismatch() {
    assert_eq!(
        accept_cell_line("CVCL_2119", None),
        PharmacoCellLineJoin::Missing
    );
}

// Acceptance 5: a compound is accepted only when the name comes back the same.

#[test]
fn a_compound_name_matches_ignoring_ascii_case_only() {
    assert!(compound_name_matches("venetoclax", "Venetoclax"));
    assert!(compound_name_matches(" venetoclax ", "Venetoclax"));
    assert!(!compound_name_matches("venetoclax", "Venetoclax phosphate"));
}

// Acceptance 8: an unknown dataset names the ten PharmacoDB datasets.

#[test]
fn an_unknown_dataset_lists_the_ten_dataset_names() {
    let error = normalize_dataset("GDSC3").expect_err("GDSC3 is not a PharmacoDB dataset");
    let message = error.to_string();
    for name in PHARMACODB_DATASET_NAMES {
        assert!(message.contains(name), "{message}");
    }
    assert!(matches!(error, BioMcpError::InvalidArgument(_)));
}

#[test]
fn a_known_dataset_normalizes_to_the_name_pharmacodb_publishes() {
    assert_eq!(normalize_dataset("gdsc1").unwrap(), "GDSC1");
    assert_eq!(normalize_dataset(" GCSI ").unwrap(), "gCSI");
}

// Acceptance 2 and 10: the counts payload, with the time an injected clock gave.

#[test]
fn the_counts_payload_sums_the_recorded_molm13_datasets() {
    let counts = build_counts(
        1248,
        PharmacoDbClient::decode_experiment_counts(
            URL,
            StatusCode::OK,
            fixture!("experiment_counts_cell_line_1248_20260918.json"),
        )
        .expect("the recorded counts"),
        FIXED_CLOCK.to_string(),
    );

    assert_eq!(counts.pharmacodb_id, 1248);
    assert_eq!(counts.total, 1117);
    assert_eq!(
        counts
            .datasets
            .iter()
            .map(|row| (row.name.as_str(), row.count))
            .collect::<Vec<_>>(),
        vec![
            ("CTRPv2", 416),
            ("gCSI", 35),
            ("GDSC1", 426),
            ("GDSC2", 240)
        ]
    );
    assert_eq!(counts.data_as_of, FIXED_CLOCK);
    assert_eq!(counts.data_as_of_kind, "retrieved");
}

#[test]
fn the_counts_line_names_every_dataset_in_name_order() {
    let counts = build_counts(
        1248,
        counts_from_rows(&molm13_experiments()),
        FIXED_CLOCK.to_string(),
    );

    assert_eq!(
        counts_line(counts.total, &counts.datasets),
        "1117 experiments: CTRPv2 416, gCSI 35, GDSC1 426, GDSC2 240"
    );
}

// Acceptance 3: rows sort by counterpart name, then dataset, then experiment id,
// and a repeated pair keeps both rows.

#[test]
fn rows_sort_by_counterpart_name_then_dataset_then_experiment_id() {
    let rows = build_rows(&molm13_experiments(), PharmacoSide::CellLine);

    let keys: Vec<(String, String, i64)> = rows
        .iter()
        .map(|row| {
            (
                row.name.to_ascii_lowercase(),
                row.dataset.clone(),
                row.experiment_id,
            )
        })
        .collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
}

#[test]
fn a_repeated_compound_and_dataset_pair_keeps_every_row_with_its_own_id() {
    let rows = rows_in_dataset(
        build_rows(&molm13_experiments(), PharmacoSide::CellLine),
        Some("GDSC2"),
    );
    let repeated: Vec<i64> = rows
        .iter()
        .filter(|row| row.name == "Afatinib")
        .map(|row| row.experiment_id)
        .collect();

    assert_eq!(repeated.len(), 5);
    let mut unique = repeated.clone();
    unique.dedup();
    assert_eq!(
        unique.len(),
        5,
        "every repeat carries its own experiment id"
    );
}

#[test]
fn a_dataset_filter_keeps_only_that_dataset_and_ignores_case() {
    let rows = build_rows(&molm13_experiments(), PharmacoSide::CellLine);
    let gdsc1 = rows_in_dataset(rows.clone(), Some("gdsc1"));

    assert_eq!(gdsc1.len(), 426);
    assert!(gdsc1.iter().all(|row| row.dataset == "GDSC1"));
    assert_eq!(rows_in_dataset(rows, None).len(), 1117);
}

#[test]
fn a_cell_line_row_names_its_compound_and_a_drug_row_names_its_cell_line() {
    let cell_line_rows = build_rows(&pair_experiments(), PharmacoSide::CellLine);
    assert_eq!(cell_line_rows[0].name, "Venetoclax");
    assert_eq!(cell_line_rows[0].uid, "PDBC02030");
    assert_eq!(cell_line_rows[0].tissue, None);

    let drug_rows = build_rows(&pair_experiments(), PharmacoSide::Drug);
    assert_eq!(drug_rows[0].name, "MOLM-13");
    assert_eq!(drug_rows[0].uid, "MOLM13_950_2019");
    assert_eq!(drug_rows[0].tissue.as_deref(), Some("Myeloid"));
}

// Acceptance 3 and 9: the page keeps the whole-side counts, the matched count,
// and one window of rows.

#[test]
fn a_row_page_pages_the_matched_rows_and_keeps_the_whole_side_total() {
    let page = build_row_page(
        &molm13_experiments(),
        PharmacoRowPageInput {
            subject: "CVCL_2119".to_string(),
            pharmacodb_id: 1248,
            side: PharmacoSide::CellLine,
            filter: PharmacoDbRowFilter {
                cell_line: None,
                dataset: Some("GDSC1".to_string()),
            },
            offset: 0,
            limit: 25,
            data_as_of: FIXED_CLOCK.to_string(),
        },
    );

    assert_eq!(page.total, 1117);
    assert_eq!(page.matched, 426);
    assert_eq!(page.rows.len(), 25);
    assert_eq!(page.filter.dataset.as_deref(), Some("GDSC1"));
    assert_eq!(page.data_as_of, FIXED_CLOCK);
    assert_eq!(page.data_as_of_kind, "retrieved");

    let second = build_row_page(
        &molm13_experiments(),
        PharmacoRowPageInput {
            subject: "CVCL_2119".to_string(),
            pharmacodb_id: 1248,
            side: PharmacoSide::CellLine,
            filter: PharmacoDbRowFilter {
                cell_line: None,
                dataset: Some("GDSC1".to_string()),
            },
            offset: 25,
            limit: 25,
            data_as_of: FIXED_CLOCK.to_string(),
        },
    );
    assert_ne!(page.rows[0].experiment_id, second.rows[0].experiment_id);
}

#[test]
fn a_venetoclax_page_reports_the_recorded_drug_side_counts() {
    let page = build_row_page(
        &venetoclax_experiments(),
        PharmacoRowPageInput {
            subject: "Venetoclax".to_string(),
            pharmacodb_id: 53572,
            side: PharmacoSide::Drug,
            filter: PharmacoDbRowFilter {
                cell_line: None,
                dataset: Some("NCI60".to_string()),
            },
            offset: 0,
            limit: 25,
            data_as_of: FIXED_CLOCK.to_string(),
        },
    );

    assert_eq!(page.total, 3608);
    assert_eq!(page.matched, 113);
    assert_eq!(
        counts_line(page.total, &page.datasets),
        "3608 experiments: CTRPv2 536, GDSC1 981, GDSC2 1052, NCI60 113, PRISM 926"
    );
}

// Acceptance 3 and 9: a null metric stays null in JSON and prints as `-`.

#[test]
fn a_null_metric_stays_null_in_json_and_prints_as_a_dash() {
    let rows = build_rows(&pair_experiments(), PharmacoSide::CellLine);
    let json = serde_json::to_value(&rows[0]).expect("row JSON");

    assert_eq!(json["ic50"], serde_json::Value::Null);
    assert_eq!(json["dss1"], serde_json::Value::Null);
    assert_eq!(format_metric(rows[0].ic50), "-");
    assert_eq!(format_metric(rows[0].aac), "0");
}

#[test]
fn markdown_prints_a_metric_with_up_to_four_significant_digits() {
    assert_eq!(format_metric(Some(0.805_489_38)), "0.8055");
    assert_eq!(format_metric(Some(0.002_849_89)), "0.00285");
    assert_eq!(format_metric(Some(969_234.526_527_05)), "969235");
    assert_eq!(format_metric(Some(100.0)), "100");
    assert_eq!(format_metric(None), "-");
}

#[test]
fn json_keeps_the_published_value_without_rounding() {
    let rows = build_rows(&pair_experiments(), PharmacoSide::CellLine);
    let json = serde_json::to_value(&rows[1]).expect("row JSON");

    assert_eq!(json["aac"], serde_json::json!(0.805_489_38));
    assert_eq!(json["ic50"], serde_json::json!(0.002_849_89));
}

// Acceptance 6: a body over the cap becomes an unavailable section that says so.

#[test]
fn a_body_over_the_cap_is_an_unavailable_section_naming_the_size() {
    let outcome = failure_outcome(&BioMcpError::BodyLimit {
        source_name: "pharmacodb".to_string(),
        max_bytes: 32 * 1024 * 1024,
    });
    let json = serde_json::to_value(&outcome).expect("outcome JSON");

    assert_eq!(json["outcome"], "unavailable");
    assert_eq!(json["message"], PHARMACODB_BODY_LIMIT_MESSAGE);
}

#[test]
fn another_failure_is_unavailable_without_the_size_message() {
    let outcome = failure_outcome(&BioMcpError::Api {
        api: "pharmacodb".to_string(),
        message: "connection reset".to_string(),
    });
    let json = serde_json::to_value(&outcome).expect("outcome JSON");

    assert_eq!(json["outcome"], "unavailable");
    assert_ne!(json["message"], PHARMACODB_BODY_LIMIT_MESSAGE);
}

// Acceptance 9: the one fixed attribution line, and the licence it names.

#[test]
fn the_attribution_line_names_the_terms_the_version_the_units_and_the_limits() {
    let line = pharmacodb_attribution(FIXED_CLOCK);

    assert_eq!(line.lines().count(), 1, "{line}");
    assert!(line.contains("Values as published by PharmacoDB"), "{line}");
    assert!(line.contains(PHARMACODB_LICENSE_SUMMARY), "{line}");
    assert!(line.contains("non-commercial"), "{line}");
    assert!(
        line.contains("publishes no version; retrieved 2026-09-19T12:00:00Z"),
        "{line}"
    );
    assert!(line.contains("gives no units"), "{line}");
    assert!(
        line.contains("BioMCP does not interpret sensitivity"),
        "{line}"
    );
}

#[test]
fn the_license_agrees_across_the_attribution_line_and_both_registries() {
    let registry: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/reference/sources.json"
    )))
    .expect("the source registry");
    let pharmacodb = registry
        .as_array()
        .expect("a list of sources")
        .iter()
        .find(|source| source["id"] == "pharmacodb")
        .expect("the PharmacoDB row");
    let summary = pharmacodb["license_summary"]
        .as_str()
        .expect("a license summary");
    assert_eq!(summary, PHARMACODB_LICENSE_SUMMARY);
    assert_eq!(pharmacodb["reviewed_on"], "2026-09-18");
    assert_eq!(pharmacodb["tier"], 3);

    let licensing = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/reference/source-licensing.md"
    ));
    assert!(
        licensing.contains(&format!("- License / terms summary: {summary}")),
        "the licensing page and the registry disagree"
    );
    assert!(
        !licensing.contains("| PharmacoDB | 3 | direct_api | none | CC BY-NC 4.0 |"),
        "the licensing table claims a licence PharmacoDB never published"
    );
    assert!(pharmacodb_attribution(FIXED_CLOCK).contains(summary));
}
