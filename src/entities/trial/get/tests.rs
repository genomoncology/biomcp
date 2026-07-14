//! Tests for trial detail helpers.

use super::*;
use crate::error::BioMcpError;

#[test]
fn normalize_nct_id_uppercases_prefix() {
    assert_eq!(normalize_nct_id("nct06162221"), "NCT06162221");
    assert_eq!(normalize_nct_id("NCT06162221"), "NCT06162221");
}

#[test]
fn parse_sections_accepts_contacts_and_all_includes_contacts() {
    let contacts = parse_sections(&["contacts".to_string()]).unwrap();
    assert!(contacts.include_contacts);
    assert!(!contacts.include_eligibility);

    let all = parse_sections(&["all".to_string()]).unwrap();
    assert!(all.include_contacts);
    assert!(all.include_eligibility);
    assert!(!all.include_eligibility_provenance);
    assert!(all.include_locations);
}

#[tokio::test]
async fn get_rejects_non_nct_id_with_format_hint() {
    let err = get("WRONG", &[], TrialSource::ClinicalTrialsGov)
        .await
        .expect_err("invalid trial id should fail before API call");

    match err {
        BioMcpError::InvalidArgument(message) => {
            assert!(message.contains("Expected an NCT ID like NCT02576665"));
            assert!(message.contains("got 'WRONG'"));
        }
        other => panic!("expected InvalidArgument, got: {other}"),
    }
}
