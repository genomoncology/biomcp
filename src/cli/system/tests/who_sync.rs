//! `who sync` honest-outcome rendering (ticket 1304).

use super::super::who_sync::who_sync_outcome;
use crate::sources::who_pq::{
    WHO_PQ_API_CSV_FILE, WHO_PQ_CSV_FILE, WHO_VACCINES_CSV_FILE, WhoPqSyncReport,
};

#[test]
fn who_sync_reports_refreshed_and_failed_files_instead_of_a_bare_success_claim() {
    let clean = who_sync_outcome(
        WhoPqSyncReport {
            refreshed: vec![WHO_PQ_CSV_FILE, WHO_PQ_API_CSV_FILE, WHO_VACCINES_CSV_FILE],
            failed: Vec::new(),
            changed: true,
        },
        false,
    )
    .expect("who sync text should render");
    assert_eq!(
        clean.text,
        "WHO Prequalification data synchronized successfully (who_pq.csv, who_api.csv, who_vaccines.csv).\n"
    );

    let partial = who_sync_outcome(
        WhoPqSyncReport {
            refreshed: vec![WHO_PQ_CSV_FILE],
            failed: vec![WHO_PQ_API_CSV_FILE, WHO_VACCINES_CSV_FILE],
            changed: false,
        },
        false,
    )
    .expect("who sync text should render");
    assert!(partial.text.contains("with failures"));
    assert!(partial.text.contains("refreshed who_pq.csv"));
    assert!(
        partial
            .text
            .contains("failed for who_api.csv, who_vaccines.csv")
    );

    let none_refreshed = who_sync_outcome(
        WhoPqSyncReport {
            refreshed: Vec::new(),
            failed: vec![WHO_PQ_CSV_FILE],
            changed: false,
        },
        false,
    )
    .expect("who sync text should render");
    assert!(none_refreshed.text.contains("no files refreshed"));

    let json = who_sync_outcome(
        WhoPqSyncReport {
            refreshed: vec![WHO_PQ_CSV_FILE],
            failed: vec![WHO_VACCINES_CSV_FILE],
            changed: false,
        },
        true,
    )
    .expect("who sync json should render");
    let value: serde_json::Value =
        serde_json::from_str(&json.text).expect("who sync json should parse");
    assert_eq!(value["kind"], "data_sync");
    assert_eq!(value["source"], "who");
    assert_eq!(value["status"], "synchronized");
    assert_eq!(value["changed"], false);
    assert_eq!(value["refreshed"], serde_json::json!([WHO_PQ_CSV_FILE]));
    assert_eq!(value["failed"], serde_json::json!([WHO_VACCINES_CSV_FILE]));
}
