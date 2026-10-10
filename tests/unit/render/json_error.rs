//! Error-shape contract tests for the JSON render surface, living in the
//! tests tree instead of the production render module (ticket 2044's open
//! Changes item, done in ticket 2047's lane).

use super::to_error_json;
use crate::error::BioMcpError;

#[test]
fn who_pq_sync_failure_json_recovery_carries_no_local_data_path() {
    // The WHO local data path is terminal-only (ticket 2038 finding 7,
    // 2035 #6): `--json who sync` with WHO broken must not put the
    // resolved directory in `error.recovery`.
    let error = BioMcpError::SourceUnavailable {
        source_name: "WHO Prequalification".to_string(),
        reason: format!(
            "{} Refresh failed for who_pq.csv. Missing required WHO Prequalification file(s) after the run: who_pq.csv.",
            crate::sources::who_pq::WHO_PQ_SYNC_FAILURE_REASON_PREFIX
        ),
        suggestion: crate::sources::who_pq::who_pq_sync_failure_recovery(std::path::Path::new(
            "/home/who/fixtures/data-dir",
        )),
    };
    let json = to_error_json(&error).expect("WHO sync error JSON");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(
        value["error"]["message"]
            .as_str()
            .expect("message")
            .starts_with(crate::sources::who_pq::WHO_PQ_SYNC_FAILURE_REASON_PREFIX)
    );
    let recovery = value["error"]["recovery"].as_str().expect("recovery");
    assert_eq!(
        recovery,
        crate::sources::who_pq::WHO_PQ_SYNC_FAILURE_PROJECTED_RECOVERY
    );
    assert!(!recovery.contains("/home/who"));
    assert!(!json.contains("/home/who"));
}
