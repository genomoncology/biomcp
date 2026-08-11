//! Tests for NCI CTS trial search helpers.

use super::super::validate_trial_search;
use super::*;
use crate::entities::trial::TrialSource;
use crate::sources::nci_cts::NciCtsClient;

fn mydisease_hit(value: serde_json::Value) -> crate::sources::mydisease::MyDiseaseHit {
    serde_json::from_value(value).expect("valid MyDisease hit")
}

#[test]
fn nci_search_prefers_grounded_disease_concept_id() {
    let filter = nci_disease_filter_from_hit(
        "melanoma",
        mydisease_hit(serde_json::json!({
            "_id": "MONDO:0005105",
            "mondo": {
                "name": "Melanoma",
                "xrefs": {
                    "ncit": ["C3224"]
                }
            }
        })),
    );
    let plan = NciCtsClient::search_plan(
        "test-key",
        &NciSearchParams {
            disease: Some(filter),
            size: 1,
            from: 0,
            ..NciSearchParams::default()
        },
    );

    assert!(
        plan.query
            .contains(&("diseases.nci_thesaurus_concept_id".into(), "C3224".into()))
    );
    assert!(!plan.query.iter().any(|(key, _)| *key == "keyword"));
}

#[test]
fn nci_search_falls_back_to_keyword_when_grounding_is_unavailable() {
    let plan = NciCtsClient::search_plan(
        "test-key",
        &NciSearchParams {
            disease: Some(NciDiseaseFilter::Keyword("melanoma".into())),
            size: 1,
            from: 0,
            ..NciSearchParams::default()
        },
    );

    assert!(plan.query.contains(&("keyword".into(), "melanoma".into())));
    assert!(
        !plan
            .query
            .iter()
            .any(|(key, _)| *key == "diseases.nci_thesaurus_concept_id")
    );
}

#[test]
fn nci_search_falls_back_to_keyword_when_best_hit_lacks_nci_xref() {
    let filter = nci_disease_filter_from_hit(
        "melanoma",
        mydisease_hit(serde_json::json!({
            "_id": "MONDO:0005105",
            "mondo": {
                "name": "Melanoma"
            }
        })),
    );

    match filter {
        NciDiseaseFilter::Keyword(value) => assert_eq!(value, "melanoma"),
        other => panic!("expected keyword fallback, got {other:?}"),
    }
}

#[test]
fn nci_keyword_fallback_request_uses_keyword_not_concept_id() {
    let plan = NciCtsClient::search_plan(
        "test-key",
        &NciSearchParams {
            disease: Some(NciDiseaseFilter::Keyword("melanoma".into())),
            size: 1,
            from: 0,
            ..NciSearchParams::default()
        },
    );

    assert!(plan.query.contains(&("keyword".into(), "melanoma".into())));
    assert!(
        !plan
            .query
            .iter()
            .any(|(key, _)| *key == "diseases.nci_thesaurus_concept_id")
    );
}

#[test]
fn nci_status_mapping_uses_documented_single_value_filters() {
    let cases = [
        ("recruiting", "site", "ACTIVE"),
        ("not yet recruiting", "current", "Approved"),
        (
            "enrolling by invitation",
            "current",
            "Enrolling by Invitation",
        ),
        ("active, not recruiting", "site", "CLOSED_TO_ACCRUAL"),
        ("completed", "current", "Complete"),
        ("suspended", "current", "Temporarily Closed to Accrual"),
        ("terminated", "current", "Administratively Complete"),
        ("withdrawn", "current", "Withdrawn"),
    ];

    for &(input, expected_kind, expected_value) in &cases {
        let normalized = validate_trial_search(&TrialSearchFilters {
            source: TrialSource::NciCts,
            status: Some(input.into()),
            ..Default::default()
        })
        .expect("status should normalize");
        let filter = nci_status_filter(normalized.normalized_status.as_deref())
            .expect("status should map")
            .expect("status filter");
        match (expected_kind, filter) {
            ("current", NciStatusFilter::CurrentTrialStatus(value)) => {
                assert_eq!(value, expected_value);
            }
            ("site", NciStatusFilter::SiteRecruitmentStatus(value)) => {
                assert_eq!(value, expected_value);
            }
            (_, other) => panic!("unexpected status filter for {input}: {other:?}"),
        }
    }
}

#[test]
fn nci_source_rejects_status_lists() {
    let err = nci_status_filter(Some("RECRUITING,COMPLETED"))
        .expect_err("NCI should reject comma-separated status lists");
    assert!(err.to_string().contains("one mapped status at a time"));
    assert!(err.to_string().contains("--source nci"));
}

#[test]
fn nci_phase_mapping_uses_i_ii_for_combined_phase() {
    let cases = [
        ("1", vec!["I"]),
        ("2", vec!["II"]),
        ("3", vec!["III"]),
        ("4", vec!["IV"]),
        ("na", vec!["NA"]),
        ("1/2", vec!["I_II"]),
    ];

    for (input_phase, expected) in cases {
        let normalized = validate_trial_search(&TrialSearchFilters {
            source: TrialSource::NciCts,
            phase: Some(input_phase.into()),
            ..Default::default()
        })
        .expect("phase should normalize");
        assert_eq!(
            nci_phase_filters(normalized.normalized_phase.as_deref()).expect("phase should map"),
            expected
        );
    }
}

#[test]
fn nci_source_rejects_early_phase1() {
    let err = nci_phase_filters(Some(&["EARLY_PHASE1".to_string()]))
        .expect_err("NCI should reject early_phase1");
    assert!(err.to_string().contains("early_phase1"));
    assert!(err.to_string().contains("--source nci"));
}

#[test]
fn nci_public_filter_table_is_explicit() {
    let mut cases = Vec::new();
    macro_rules! case {
        ($name:literal, $field:ident, $value:expr, $mapped:literal) => {{
            let mut filters = TrialSearchFilters {
                source: TrialSource::NciCts,
                ..Default::default()
            };
            filters.$field = $value;
            cases.push(($name, filters, $mapped));
        }};
    }
    case!("condition", condition, Some("melanoma".into()), true);
    case!("intervention", intervention, Some("drug".into()), true);
    case!("facility", facility, Some("clinic".into()), true);
    case!("status", status, Some("recruiting".into()), true);
    case!("phase", phase, Some("2".into()), true);
    case!("biomarker", biomarker, Some("BRAF".into()), true);
    case!("mutation", mutation, Some("V600E".into()), true);
    case!("criteria", criteria, Some("ECOG 0".into()), true);
    case!(
        "study type",
        study_type,
        Some("interventional".into()),
        false
    );
    case!("age", age, Some(67.0), false);
    case!("sex", sex, Some("female".into()), false);
    case!("sponsor", sponsor, Some("NCI".into()), false);
    case!("sponsor type", sponsor_type, Some("nih".into()), false);
    case!("date from", date_from, Some("2026-01-01".into()), false);
    case!("date to", date_to, Some("2026-01-01".into()), false);
    case!(
        "prior therapies",
        prior_therapies,
        Some("platinum".into()),
        false
    );
    case!("progression on", progression_on, Some("drug".into()), false);
    case!("line of therapy", line_of_therapy, Some("2L".into()), false);
    case!("results", results_available, true, false);
    let mut geo = TrialSearchFilters {
        source: TrialSource::NciCts,
        lat: Some(42.0),
        lon: Some(-71.0),
        distance: Some(50),
        ..Default::default()
    };
    cases.push(("complete geo", geo.clone(), true));
    geo.no_alias_expand = true;
    cases.push(("no alias expansion", geo, false));

    for (name, filters, mapped) in cases {
        assert_eq!(validate_trial_search(&filters).is_ok(), mapped, "{name}");
    }
}

#[test]
fn nci_biomarker_like_fields_never_choose_or_duplicate_a_value() {
    for field in ["biomarker", "mutation", "criteria"] {
        let mut filters = TrialSearchFilters::default();
        match field {
            "biomarker" => filters.biomarker = Some("BRAF".into()),
            "mutation" => filters.mutation = Some("BRAF".into()),
            _ => filters.criteria = Some("BRAF".into()),
        }
        let value = nci_biomarker_value(&filters).unwrap();
        let plan = NciCtsClient::search_plan(
            "key",
            &NciSearchParams {
                biomarkers: value,
                ..Default::default()
            },
        );
        assert_eq!(
            plan.query
                .iter()
                .filter(|(key, _)| key == "biomarkers")
                .count(),
            1
        );
    }
    let filters = TrialSearchFilters {
        biomarker: Some("BRAF".into()),
        mutation: Some("V600E".into()),
        ..Default::default()
    };
    assert!(nci_biomarker_value(&filters).is_err());
}
