//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use super::super::*;
use crate::sources::HttpMethod;

#[test]
fn get_fields_contacts_preserve_site_context_and_eligibility_sex() {
    let contact_fields = build_get_fields(&["contacts".to_string()]);
    for field in [
        "CentralContactEMail",
        "LocationFacility",
        "LocationCity",
        "LocationState",
        "LocationCountry",
        "LocationContactEMail",
    ] {
        assert!(contact_fields.split(',').any(|actual| actual == field));
    }

    let eligibility_fields = build_get_fields(&["eligibility".to_string()]);
    assert!(eligibility_fields.split(',').any(|field| field == "Sex"));
    assert!(
        eligibility_fields
            .split(',')
            .any(|field| field == "LargeDocumentModule")
    );

    let all_fields = build_get_fields(&["all".to_string()]);
    assert!(
        !all_fields
            .split(',')
            .any(|field| field == "LargeDocumentModule")
    );

    let document_fields = build_get_fields(&["documents".to_string()]);
    assert!(
        document_fields
            .split(',')
            .any(|field| field == "LargeDocumentModule")
    );
}

#[test]
fn search_plan_builds_expected_params() {
    let plan = ClinicalTrialsClient::search_plan(&CtGovSearchParams {
        condition: Some(" melanoma ".into()),
        intervention: Some(" \"pembrolizumab\" ".into()),
        facility: None,
        status: Some(" RECRUITING ".into()),
        agg_filters: None,
        query_term: Some(" AREA[Phase]PHASE2 ".into()),
        fields_override: None,
        count_total: true,
        page_token: None,
        page_size: 3,
        lat: None,
        lon: None,
        distance_miles: None,
    });

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "studies");
    assert_eq!(plan.query_value("query.cond"), Some("melanoma"));
    assert_eq!(plan.query_value("query.intr"), Some("\"pembrolizumab\""));
    assert_eq!(plan.query_value("filter.overallStatus"), Some("RECRUITING"));
    assert_eq!(plan.query_value("query.term"), Some("AREA[Phase]PHASE2"));
    assert_eq!(plan.query_value("countTotal"), Some("true"));
    assert_eq!(plan.query_value("pageSize"), Some("3"));
    assert_eq!(plan.query_value("fields"), Some(CTGOV_SEARCH_FIELDS));
}

#[test]
fn search_plan_includes_geo_facility_agg_and_field_override() {
    let geo = ClinicalTrialsClient::search_plan(&CtGovSearchParams {
        condition: Some("melanoma".into()),
        intervention: None,
        facility: Some("MD Anderson".into()),
        status: None,
        agg_filters: Some("sex:f,funderType:nih".into()),
        query_term: None,
        fields_override: Some(CTGOV_ADVERSE_EVENT_SEARCH_FIELDS.into()),
        count_total: false,
        page_token: Some("token-1".into()),
        page_size: 20,
        lat: Some(41.5),
        lon: Some(-81.7),
        distance_miles: Some(50),
    });

    assert_eq!(geo.query_value("query.locn"), Some("MD Anderson"));
    assert_eq!(geo.query_value("aggFilters"), Some("sex:f,funderType:nih"));
    assert_eq!(geo.query_value("pageToken"), Some("token-1"));
    assert_eq!(
        geo.query_value("filter.geo"),
        Some("distance(41.5,-81.7,50mi)")
    );
    assert_eq!(
        geo.query_value("fields"),
        Some(CTGOV_ADVERSE_EVENT_SEARCH_FIELDS)
    );
}

#[test]
fn default_get_fields_request_visible_status_context() {
    let fields = build_get_fields(&[]);

    for field in ["InterventionType", "WhyStopped"] {
        assert!(
            fields.split(',').any(|actual| actual == field),
            "default trial details must request {field} before exposing it"
        );
    }
}

#[test]
fn location_postal_code_is_requested_only_for_location_projections() {
    for (sections, expected_count) in [
        (vec![], 0),
        (vec!["contacts".to_string()], 0),
        (vec!["locations".to_string()], 1),
        (vec!["all".to_string()], 1),
    ] {
        let fields = build_get_fields(&sections);
        assert_eq!(
            fields
                .split(',')
                .filter(|field| *field == "LocationZip")
                .count(),
            expected_count,
            "LocationZip count for sections {sections:?}"
        );
    }
}

#[test]
fn intervention_detail_fields_are_requested_once_with_or_without_arms() {
    for sections in [vec![], vec!["arms".to_string()]] {
        let plan = ClinicalTrialsClient::get_plan("NCT02576665", &sections);
        let fields = plan
            .query_value("fields")
            .expect("trial detail fields query");

        for expected in [
            "InterventionName",
            "InterventionOtherName",
            "InterventionType",
            "InterventionDescription",
        ] {
            assert_eq!(
                fields
                    .split(',')
                    .filter(|actual| *actual == expected)
                    .count(),
                1,
                "{expected} must be requested exactly once for sections {sections:?}"
            );
        }
    }
}

#[test]
fn get_plan_builds_study_path_and_section_fields() {
    let sections = vec!["contacts".to_string(), "eligibility".to_string()];
    let plan = ClinicalTrialsClient::get_plan("NCT41300001", &sections);

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "studies/NCT41300001");
    let fields = plan.query_value("fields").expect("fields query");
    assert!(
        fields
            .split(',')
            .any(|field| field == "CentralContactEMail")
    );
    assert!(fields.split(',').any(|field| field == "Sex"));
}
