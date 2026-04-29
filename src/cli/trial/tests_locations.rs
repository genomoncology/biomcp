//! Trial location pagination and query-summary tests.

use super::dispatch::{
    LocationPaginationMeta, paginate_trial_locations, parse_trial_location_paging,
    should_show_trial_zero_result_nickname_hint, trial_locations_json, trial_search_query_summary,
};

#[test]
fn parse_trial_location_paging_extracts_offset_limit_flags() {
    let sections = vec![
        "locations".to_string(),
        "--offset".to_string(),
        "20".to_string(),
        "--limit=10".to_string(),
    ];
    let (cleaned, offset, limit) =
        parse_trial_location_paging(&sections).expect("valid pagination flags");
    assert_eq!(cleaned, vec!["locations".to_string()]);
    assert_eq!(offset, Some(20));
    assert_eq!(limit, Some(10));
}

#[test]
fn trial_locations_json_preserves_location_pagination_and_section_sources() {
    let trial = crate::entities::trial::Trial {
        nct_id: "NCT00000001".to_string(),
        source: Some("ctgov".to_string()),
        title: "Example trial".to_string(),
        status: "Recruiting".to_string(),
        phase: Some("Phase 2".to_string()),
        study_type: Some("Interventional".to_string()),
        age_range: Some("18 Years and older".to_string()),
        conditions: vec!["melanoma".to_string()],
        interventions: vec!["osimertinib".to_string()],
        intervention_details: Vec::new(),
        sponsor: Some("Example Sponsor".to_string()),
        enrollment: Some(100),
        summary: Some("Example summary".to_string()),
        start_date: Some("2024-01-01".to_string()),
        completion_date: None,
        eligibility_text: None,
        locations: Some(vec![crate::entities::trial::TrialLocation {
            facility: "Example Hospital".to_string(),
            city: "Boston".to_string(),
            state: Some("MA".to_string()),
            country: "United States".to_string(),
            status: Some("Recruiting".to_string()),
            contact_name: None,
            contact_phone: None,
        }]),
        outcomes: None,
        arms: None,
        references: None,
    };

    let json = trial_locations_json(
        &trial,
        LocationPaginationMeta {
            total: 42,
            offset: 20,
            limit: 10,
            has_more: true,
        },
    )
    .expect("trial locations json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["nct_id"], "NCT00000001");
    assert_eq!(value["location_pagination"]["total"], 42);
    assert_eq!(value["location_pagination"]["offset"], 20);
    assert_eq!(value["location_pagination"]["limit"], 10);
    assert_eq!(value["location_pagination"]["has_more"], true);
    assert!(value.get("_meta").is_some());
    assert_eq!(value["_meta"]["section_sources"][0]["key"], "overview");
    assert_eq!(
        value["_meta"]["section_sources"][0]["sources"][0],
        "ClinicalTrials.gov"
    );
    assert!(
        value["_meta"]["section_sources"]
            .as_array()
            .expect("section sources array")
            .iter()
            .any(|entry| entry["key"] == "locations")
    );
}
#[test]
fn paginate_trial_locations_handles_missing_locations() {
    let mut trial = crate::entities::trial::Trial {
        nct_id: "NCT00000001".to_string(),
        source: Some("ctgov".to_string()),
        title: "Example trial".to_string(),
        status: "Recruiting".to_string(),
        phase: Some("Phase 2".to_string()),
        study_type: Some("Interventional".to_string()),
        age_range: Some("18 Years and older".to_string()),
        conditions: vec!["melanoma".to_string()],
        interventions: vec!["osimertinib".to_string()],
        intervention_details: Vec::new(),
        sponsor: Some("Example Sponsor".to_string()),
        enrollment: Some(100),
        summary: Some("Example summary".to_string()),
        start_date: Some("2024-01-01".to_string()),
        completion_date: None,
        eligibility_text: None,
        locations: None,
        outcomes: None,
        arms: None,
        references: None,
    };

    let meta = paginate_trial_locations(&mut trial, 20, 10);
    assert_eq!(meta.total, 0);
    assert_eq!(meta.offset, 20);
    assert_eq!(meta.limit, 10);
    assert!(!meta.has_more);
    assert!(trial.locations.is_some());
    assert_eq!(trial.locations.as_ref().map_or(usize::MAX, Vec::len), 0);
}
#[test]
fn trial_search_query_summary_includes_geo_filters() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            condition: Some("melanoma".into()),
            facility: Some("MD Anderson".into()),
            age: Some(67.0),
            sex: Some("female".into()),
            criteria: Some("mismatch repair deficient".into()),
            sponsor_type: Some("nih".into()),
            lat: Some(40.7128),
            lon: Some(-74.006),
            distance: Some(50),
            ..Default::default()
        },
        None,
        0,
        None,
    );
    assert!(summary.contains("condition=melanoma"));
    assert!(summary.contains("facility=MD Anderson"));
    assert!(summary.contains("age=67"));
    assert!(summary.contains("sex=female"));
    assert!(summary.contains("criteria=mismatch repair deficient"));
    assert!(summary.contains("sponsor_type=nih"));
    assert!(summary.contains("lat=40.7128"));
    assert!(summary.contains("lon=-74.006"));
    assert!(summary.contains("distance=50"));
}

#[test]
fn trial_search_query_summary_includes_nci_source_marker() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            condition: Some("melanoma".into()),
            source: crate::entities::trial::TrialSource::NciCts,
            ..Default::default()
        },
        None,
        0,
        None,
    );

    assert!(summary.contains("condition=melanoma"));
    assert!(summary.contains("source=nci"));
}

#[test]
fn trial_search_query_summary_includes_alias_opt_out_marker() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            intervention: Some("daraxonrasib".into()),
            no_alias_expand: true,
            ..Default::default()
        },
        Some("daraxonrasib"),
        0,
        None,
    );

    assert!(summary.contains("intervention=daraxonrasib"));
    assert!(summary.contains("alias_expand=off"));
}

#[test]
fn trial_search_query_summary_omits_alias_opt_out_marker_when_not_applicable() {
    let no_intervention = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            condition: Some("melanoma".into()),
            no_alias_expand: true,
            ..Default::default()
        },
        None,
        0,
        None,
    );
    let nci = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            intervention: Some("daraxonrasib".into()),
            no_alias_expand: true,
            source: crate::entities::trial::TrialSource::NciCts,
            ..Default::default()
        },
        Some("daraxonrasib"),
        0,
        None,
    );

    assert!(!no_intervention.contains("alias_expand=off"));
    assert!(!nci.contains("alias_expand=off"));
}

#[test]
fn trial_search_query_summary_can_show_canonical_intervention() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            intervention: Some("Keytruda".into()),
            ..Default::default()
        },
        Some("pembrolizumab"),
        0,
        None,
    );

    assert!(summary.contains("intervention=pembrolizumab"));
    assert!(!summary.contains("intervention=Keytruda"));
}

#[test]
fn trial_zero_result_nickname_hint_requires_positional_ctgov_query_with_zero_results() {
    use crate::entities::trial::TrialSource;

    assert!(should_show_trial_zero_result_nickname_hint(
        Some("CodeBreaK 300"),
        TrialSource::ClinicalTrialsGov,
        0
    ));
    assert!(!should_show_trial_zero_result_nickname_hint(
        None,
        TrialSource::ClinicalTrialsGov,
        0
    ));
    assert!(!should_show_trial_zero_result_nickname_hint(
        Some("CodeBreaK 300"),
        TrialSource::NciCts,
        0
    ));
    assert!(!should_show_trial_zero_result_nickname_hint(
        Some("CodeBreaK 300"),
        TrialSource::ClinicalTrialsGov,
        1
    ));
}
