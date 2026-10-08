//! `who sync` honest-outcome rendering (tickets 1304 and 2021).

use super::super::who_sync::who_sync_outcome;
use crate::sources::who_pq::{
    WHO_PQ_API_CSV_FILE, WHO_PQ_CSV_FILE, WHO_VACCINES_CSV_FILE, WhoPqSyncFileFailure,
    WhoPqSyncReport,
};

fn failure(file: &'static str, reason: &str) -> WhoPqSyncFileFailure {
    WhoPqSyncFileFailure {
        file,
        reason: reason.to_string(),
    }
}

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
    assert_eq!(clean.exit_code, 0);

    let none_refreshed = who_sync_outcome(
        WhoPqSyncReport {
            refreshed: Vec::new(),
            failed: vec![failure(
                WHO_PQ_CSV_FILE,
                "WHO Prequalification export headers did not match: who_pq.csv: missing required column BASIS OF LISTING",
            )],
            changed: false,
        },
        false,
    )
    .expect("who sync text should render");
    assert!(none_refreshed.text.contains("sync incomplete"));
    assert!(none_refreshed.text.contains("no files refreshed"));
    assert!(none_refreshed.text.contains("who_pq.csv"));
    assert!(none_refreshed.text.contains("missing required column"));
    assert_eq!(none_refreshed.exit_code, 1);
}

#[test]
fn who_sync_partial_run_says_partial_names_each_file_and_exits_nonzero() {
    let partial = who_sync_outcome(
        WhoPqSyncReport {
            refreshed: vec![WHO_PQ_CSV_FILE],
            failed: vec![
                failure(
                    WHO_PQ_API_CSV_FILE,
                    "API request to who-prequalification failed",
                ),
                failure(
                    WHO_VACCINES_CSV_FILE,
                    "WHO Prequalification export headers did not match: who_vaccines.csv: missing required column MANUFACTURER",
                ),
            ],
            changed: false,
        },
        false,
    )
    .expect("who sync text should render");
    assert!(partial.text.contains("partially synchronized"));
    assert!(partial.text.contains("refreshed who_pq.csv"));
    assert!(
        partial
            .text
            .contains("failed for who_api.csv (API request to who-prequalification failed), who_vaccines.csv (WHO Prequalification export headers did not match: who_vaccines.csv: missing required column MANUFACTURER)")
    );
    assert!(partial.text.contains("Existing data files were kept"));
    assert!(!partial.text.contains("synchronized successfully"));
    assert_eq!(partial.exit_code, 1);
}

#[test]
fn who_sync_partial_json_says_partial_with_per_file_outcomes_and_exits_nonzero() {
    let json = who_sync_outcome(
        WhoPqSyncReport {
            refreshed: vec![WHO_PQ_CSV_FILE],
            failed: vec![failure(
                WHO_VACCINES_CSV_FILE,
                "WHO Prequalification export headers did not match: who_vaccines.csv: missing required column MANUFACTURER",
            )],
            changed: false,
        },
        true,
    )
    .expect("who sync json should render");
    let value: serde_json::Value =
        serde_json::from_str(&json.text).expect("who sync json should parse");
    assert_eq!(value["kind"], "data_sync");
    assert_eq!(value["source"], "who");
    // A run that kept older files is not a synchronized source (ticket 2021).
    assert_eq!(value["status"], "partial");
    assert_eq!(value["changed"], false);
    assert_eq!(value["refreshed"], serde_json::json!([WHO_PQ_CSV_FILE]));
    assert_eq!(
        value["failed"],
        serde_json::json!([{
            "file": WHO_VACCINES_CSV_FILE,
            "reason": "WHO Prequalification export headers did not match: who_vaccines.csv: missing required column MANUFACTURER",
        }])
    );
    assert_eq!(json.exit_code, 1);
}

#[test]
fn who_sync_clean_json_keeps_synchronized_status_and_zero_exit() {
    let json = who_sync_outcome(
        WhoPqSyncReport {
            refreshed: vec![WHO_PQ_CSV_FILE, WHO_PQ_API_CSV_FILE, WHO_VACCINES_CSV_FILE],
            failed: Vec::new(),
            changed: true,
        },
        true,
    )
    .expect("who sync json should render");
    let value: serde_json::Value =
        serde_json::from_str(&json.text).expect("who sync json should parse");
    assert_eq!(value["status"], "synchronized");
    assert_eq!(value["failed"], serde_json::json!([]));
    assert_eq!(json.exit_code, 0);
}
