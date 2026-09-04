//! Tests for trial search normalization helpers.

use super::super::validate_trial_search;
use super::*;
use crate::entities::trial::{TrialSearchFilters, TrialSource};

#[test]
fn status_priority_prefers_recruiting_over_completed() {
    assert!(status_priority("RECRUITING") < status_priority("COMPLETED"));
    assert!(status_priority("ACTIVE_NOT_RECRUITING") < status_priority("UNKNOWN"));
}

#[test]
fn normalize_phase_accepts_aliases() {
    let cases = [
        ("NA", &["NA"][..]),
        ("N/A", &["NA"][..]),
        ("n/a", &["NA"][..]),
        ("EARLY_PHASE1", &["EARLY_PHASE1"][..]),
        ("early_phase1", &["EARLY_PHASE1"][..]),
        ("early1", &["EARLY_PHASE1"][..]),
        ("PHASE1", &["PHASE1"][..]),
        ("1", &["PHASE1"][..]),
        ("I", &["PHASE1"][..]),
        ("PHASE2", &["PHASE2"][..]),
        ("2", &["PHASE2"][..]),
        ("II", &["PHASE2"][..]),
        ("PHASE3", &["PHASE3"][..]),
        ("3", &["PHASE3"][..]),
        ("III", &["PHASE3"][..]),
        ("PHASE4", &["PHASE4"][..]),
        ("4", &["PHASE4"][..]),
        ("IV", &["PHASE4"][..]),
        ("PHASE1/PHASE2", &["PHASE1", "PHASE2"][..]),
        ("1 / 2", &["PHASE1", "PHASE2"][..]),
        ("I_II", &["PHASE1", "PHASE2"][..]),
        ("PHASE2/PHASE3", &["PHASE2", "PHASE3"][..]),
        ("2/3", &["PHASE2", "PHASE3"][..]),
        ("II_III", &["PHASE2", "PHASE3"][..]),
        (" phase2 / phase3 ", &["PHASE2", "PHASE3"][..]),
        ("iii", &["PHASE3"][..]),
    ];

    for (input, expected) in cases {
        assert_eq!(normalize_phase(input).unwrap(), expected, "input {input}");
    }
}

#[test]
fn normalize_phase_rejects_invalid_value() {
    for input in [
        "5",
        "PHASE1/",
        "III/IV",
        "PHASE2/PHASE1",
        "I__II",
        "PHASE1/BOGUS",
        "PHASE1/PHASE1",
    ] {
        let err = normalize_phase(input).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Unrecognized --phase value"), "input {input}");
        for supported in [
            "EARLY_PHASE1",
            "PHASE1/PHASE2",
            "PHASE2/PHASE3",
            "I_II",
            "II_III",
        ] {
            assert!(msg.contains(supported), "{supported} absent for {input}");
        }

        for source in [TrialSource::ClinicalTrialsGov, TrialSource::NciCts] {
            let error = validate_trial_search(&TrialSearchFilters {
                source,
                phase: Some(input.into()),
                ..Default::default()
            })
            .err()
            .expect("malformed phase must fail public validation");
            assert!(matches!(error, BioMcpError::InvalidArgument(_)));
        }
    }
}

#[test]
fn normalized_phase_filter_preserves_blank_as_absent() {
    let filters = TrialSearchFilters {
        phase: Some(" \t\n ".into()),
        ..Default::default()
    };
    assert_eq!(normalized_phase_filter(&filters).unwrap(), None);
}

#[test]
fn normalize_intervention_query_canonicalizes_confirmed_drug_code_pattern() {
    assert_eq!(normalize_intervention_query("HRS 4642"), "HRS-4642");
}

#[test]
fn normalize_intervention_query_preserves_generic_multiword_names() {
    assert_eq!(
        normalize_intervention_query("pembrolizumab"),
        "pembrolizumab"
    );
    assert_eq!(
        normalize_intervention_query("immune checkpoint inhibitor"),
        "immune checkpoint inhibitor"
    );
}

#[test]
fn normalize_status_accepts_ctgov_wording_and_aliases() {
    assert_eq!(
        normalize_status("active, not recruiting").unwrap(),
        "ACTIVE_NOT_RECRUITING"
    );
    assert_eq!(normalize_status("recruiting").unwrap(), "RECRUITING");
    assert_eq!(
        normalize_status("enrolling_by_invitation").unwrap(),
        "ENROLLING_BY_INVITATION"
    );
}

#[test]
fn normalize_status_accepts_comma_separated_values() {
    assert_eq!(
        normalize_status("RECRUITING,ACTIVE_NOT_RECRUITING").unwrap(),
        "RECRUITING,ACTIVE_NOT_RECRUITING"
    );
    assert_eq!(
        normalize_status("recruiting,completed").unwrap(),
        "RECRUITING,COMPLETED"
    );
}

#[test]
fn trial_sources_reject_ambiguous_active_status_with_replacements() {
    for input in ["active", "recruiting,active"] {
        let errors = [TrialSource::ClinicalTrialsGov, TrialSource::NciCts].map(|source| {
            validate_trial_search(&TrialSearchFilters {
                source,
                status: Some(input.into()),
                ..Default::default()
            })
            .err()
            .map(|error| error.to_string())
        });

        assert!(
            errors.iter().all(Option::is_some),
            "active must be refused before either provider is queried; got {errors:?}"
        );
        let [ctgov_error, nci_error] = errors.map(Option::unwrap);
        assert_eq!(ctgov_error, nci_error);
        assert!(ctgov_error.contains("active is ambiguous"));
        assert!(ctgov_error.contains("NCI"));
        assert!(ctgov_error.contains("ClinicalTrials.gov"));
        assert!(ctgov_error.contains("--status recruiting"));
        assert!(ctgov_error.contains("open and accruing"));
        assert!(ctgov_error.contains("--status active_not_recruiting"));
        assert!(ctgov_error.contains("enrolled and no longer accruing"));
    }
}

#[test]
fn normalize_status_rejects_invalid_value() {
    let err = normalize_status("bogus").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Unrecognized --status value"));
    assert!(msg.contains("ENROLLING_BY_INVITATION"));
}

#[test]
fn normalize_status_rejects_comma_list_with_invalid_value() {
    let err = normalize_status("bogus,recruiting").unwrap_err();
    assert!(err.to_string().contains("Unrecognized --status value"));
}

#[test]
fn normalize_sex_accepts_supported_values() {
    assert_eq!(normalize_sex("female").unwrap(), Some("f"));
    assert_eq!(normalize_sex("male").unwrap(), Some("m"));
    assert_eq!(normalize_sex("all").unwrap(), None);
    assert_eq!(normalize_sex("F").unwrap(), Some("f"));
    assert_eq!(normalize_sex("M").unwrap(), Some("m"));
}

#[test]
fn normalize_sponsor_type_accepts_supported_values() {
    assert_eq!(normalize_sponsor_type("nih").unwrap(), "nih");
    assert_eq!(normalize_sponsor_type("industry").unwrap(), "industry");
    assert_eq!(normalize_sponsor_type("fed").unwrap(), "fed");
    assert_eq!(normalize_sponsor_type("federal").unwrap(), "fed");
    assert_eq!(normalize_sponsor_type("other").unwrap(), "other");
}

#[test]
fn normalize_sex_rejects_invalid_value() {
    let err = normalize_sex("unknown").unwrap_err();
    assert!(err.to_string().contains("Unrecognized --sex value"));
}

#[test]
fn normalize_sponsor_type_rejects_invalid_value() {
    let err = normalize_sponsor_type("charity").unwrap_err();
    assert!(
        err.to_string()
            .contains("Unrecognized --sponsor-type value")
    );
}
