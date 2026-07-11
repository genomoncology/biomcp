//! Tier 3 - local-data parsing. Pure: parses committed CSV bytes and checks the
//! in-memory lookup behavior. No network.

use std::sync::Arc;

use super::super::*;

const INTERACTIONS_CSV: &[u8] = b"DDInterID_A,Drug_A,DDInterID_B,Drug_B,Level\nDDInter1,Abacavir,DDInter2,Warfarin,Moderate\nDDInter2,Warfarin,DDInter3,Aspirin,Major\n";

#[test]
fn normalize_name_key_collapses_spacing_and_case() {
    assert_eq!(
        normalize_name_key("Asparaginase Escherichia coli"),
        Some("asparaginase escherichia coli".to_string())
    );
    assert_eq!(
        normalize_name_key("  Warfarin Sodium "),
        Some("warfarin sodium".to_string())
    );
}

#[test]
fn parse_csv_rows_reads_expected_shape() {
    let rows = parse_csv_rows("fixture.csv", INTERACTIONS_CSV).expect("rows");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].drug_a, "Abacavir");
    assert_eq!(rows[0].drug_b, "Warfarin");
    assert_eq!(rows[0].level.as_deref(), Some("Moderate"));
    assert_eq!(rows[1].drug_a, "Warfarin");
    assert_eq!(rows[1].drug_b, "Aspirin");
    assert_eq!(rows[1].level.as_deref(), Some("Major"));
}

#[test]
fn parse_csv_rows_rejects_incomplete_rows() {
    let err = parse_csv_rows(
        "fixture.csv",
        b"DDInterID_A,Drug_A,DDInterID_B,Drug_B,Level\nDDInter1,,DDInter2,Warfarin,Moderate\n",
    )
    .unwrap_err();

    assert!(err.to_string().contains("incomplete interaction row"));
}

#[test]
fn client_lookup_matches_both_sides_without_duplicates() {
    let rows = parse_csv_rows("fixture.csv", INTERACTIONS_CSV).expect("rows");
    let mut index = DdinterIndex::default();
    for row in rows {
        let idx = index.rows.len();
        if let Some(key) = normalize_name_key(&row.drug_a) {
            index.by_name.entry(key).or_default().push(idx);
        }
        if let Some(key) = normalize_name_key(&row.drug_b) {
            index.by_name.entry(key).or_default().push(idx);
        }
        index.rows.push(row);
    }

    let client = DdinterClient {
        index: Arc::new(index),
        freshness: DdinterBundleFreshness::Fresh,
    };
    let identity = DdinterIdentity::with_aliases("Warfarin", None, &["warfarin".to_string()]);
    let matches = client.interactions(&identity);

    assert!(client.contains_identity(&identity));
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].drug_a, "Abacavir");
    assert_eq!(matches[1].drug_b, "Aspirin");
}

#[test]
fn client_coverage_status_distinguishes_absent_drug_from_empty_matches() {
    let rows = parse_csv_rows("fixture.csv", INTERACTIONS_CSV).expect("rows");
    let mut index = DdinterIndex::default();
    for row in rows {
        let idx = index.rows.len();
        if let Some(key) = normalize_name_key(&row.drug_a) {
            index.by_name.entry(key).or_default().push(idx);
        }
        if let Some(key) = normalize_name_key(&row.drug_b) {
            index.by_name.entry(key).or_default().push(idx);
        }
        index.rows.push(row);
    }

    let client = DdinterClient {
        index: Arc::new(index),
        freshness: DdinterBundleFreshness::Fresh,
    };
    let uncovered = DdinterIdentity::with_aliases("dabigatran", None, &[]);

    assert!(!client.contains_identity(&uncovered));
    assert!(client.interactions(&uncovered).is_empty());
}
