//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use super::super::*;
use crate::sources::HttpMethod;

#[test]
fn get_fields_contacts_preserve_site_context_and_eligibility_sex() {
    let contact_plan = ClinicalTrialsClient::get_plan("NCT00000001", &["contacts".to_string()]);
    let contact_fields = contact_plan.query_value("fields").unwrap();
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

    let eligibility_plan =
        ClinicalTrialsClient::get_plan("NCT00000001", &["eligibility".to_string()]);
    let eligibility_fields = eligibility_plan.query_value("fields").unwrap();
    assert!(eligibility_fields.split(',').any(|field| field == "Sex"));
    assert!(
        eligibility_fields
            .split(',')
            .any(|field| field == "LargeDocumentModule")
    );

    let all_plan = ClinicalTrialsClient::get_plan("NCT00000001", &["all".to_string()]);
    let all_fields = all_plan.query_value("fields").unwrap();
    assert!(
        !all_fields
            .split(',')
            .any(|field| field == "LargeDocumentModule")
    );

    let document_plan = ClinicalTrialsClient::get_plan("NCT00000001", &["documents".to_string()]);
    let document_fields = document_plan.query_value("fields").unwrap();
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
    let plan = ClinicalTrialsClient::get_plan("NCT00000001", &[]);
    let fields = plan.query_value("fields").unwrap();

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
        let plan = ClinicalTrialsClient::get_plan("NCT00000001", &sections);
        let fields = plan.query_value("fields").unwrap();
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

#[test]
fn biodata_detail_plan_uses_the_exact_shared_path_and_fields() {
    let plan = ClinicalTrialsClient::get_plan("NCT02576665", &["references".to_string()]);
    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "studies/NCT02576665");
    assert_eq!(
        plan.query_value("fields"),
        Some(
            "BriefSummary,BriefTitle,CompletionDate,Condition,EnrollmentCount,InterventionDescription,InterventionName,InterventionOtherName,InterventionType,LeadSponsorName,MaximumAge,MinimumAge,NCTId,OverallStatus,Phase,ReferenceCitation,ReferencePMID,ReferenceType,StartDate,StudyType,WhyStopped"
        )
    );
}

#[test]
fn every_product_detail_route_has_one_exact_composed_request() {
    let cases = [
        (
            vec![],
            "BriefSummary,BriefTitle,CompletionDate,Condition,EnrollmentCount,InterventionDescription,InterventionName,InterventionOtherName,InterventionType,LeadSponsorName,MaximumAge,MinimumAge,NCTId,OverallStatus,Phase,StartDate,StudyType,WhyStopped",
        ),
        (
            vec!["arms"],
            "ArmGroupDescription,ArmGroupInterventionName,ArmGroupLabel,ArmGroupType,BriefSummary,BriefTitle,CompletionDate,Condition,EnrollmentCount,InterventionArmGroupLabel,InterventionDescription,InterventionName,InterventionOtherName,InterventionType,LeadSponsorName,MaximumAge,MinimumAge,NCTId,OverallStatus,Phase,StartDate,StudyType,WhyStopped",
        ),
        (
            vec!["all"],
            "ArmGroupDescription,ArmGroupInterventionName,ArmGroupLabel,ArmGroupType,BriefSummary,BriefTitle,CentralContactEMail,CentralContactName,CentralContactPhone,CentralContactRole,CompletionDate,Condition,EligibilityCriteria,EnrollmentCount,InterventionArmGroupLabel,InterventionDescription,InterventionName,InterventionOtherName,InterventionType,LeadSponsorName,LocationCity,LocationContactEMail,LocationContactName,LocationContactPhone,LocationContactRole,LocationCountry,LocationFacility,LocationGeoPoint,LocationState,LocationStatus,LocationZip,MaximumAge,MinimumAge,NCTId,OverallStatus,Phase,PrimaryOutcomeDescription,PrimaryOutcomeMeasure,PrimaryOutcomeTimeFrame,ReferenceCitation,ReferencePMID,ReferenceType,SecondaryOutcomeDescription,SecondaryOutcomeMeasure,SecondaryOutcomeTimeFrame,Sex,StartDate,StudyType,WhyStopped",
        ),
        (
            vec!["eligibility"],
            "BriefSummary,BriefTitle,CompletionDate,Condition,EligibilityCriteria,EnrollmentCount,InterventionDescription,InterventionName,InterventionOtherName,InterventionType,LargeDocumentModule,LeadSponsorName,MaximumAge,MinimumAge,NCTId,OverallStatus,Phase,Sex,StartDate,StudyType,WhyStopped",
        ),
        (
            vec!["documents"],
            "BriefSummary,BriefTitle,CompletionDate,Condition,EnrollmentCount,InterventionDescription,InterventionName,InterventionOtherName,InterventionType,LargeDocumentModule,LeadSponsorName,MaximumAge,MinimumAge,NCTId,OverallStatus,Phase,StartDate,StudyType,WhyStopped",
        ),
        (
            vec!["arms", "outcomes"],
            "ArmGroupDescription,ArmGroupInterventionName,ArmGroupLabel,ArmGroupType,BriefSummary,BriefTitle,CompletionDate,Condition,EnrollmentCount,InterventionArmGroupLabel,InterventionDescription,InterventionName,InterventionOtherName,InterventionType,LeadSponsorName,MaximumAge,MinimumAge,NCTId,OverallStatus,Phase,PrimaryOutcomeDescription,PrimaryOutcomeMeasure,PrimaryOutcomeTimeFrame,SecondaryOutcomeDescription,SecondaryOutcomeMeasure,SecondaryOutcomeTimeFrame,StartDate,StudyType,WhyStopped",
        ),
    ];

    for (sections, expected_fields) in cases {
        let sections = sections.into_iter().map(str::to_owned).collect::<Vec<_>>();
        let plan = ClinicalTrialsClient::get_plan("NCT02576665", &sections);
        assert_eq!(plan.method, HttpMethod::Get);
        assert_eq!(plan.path, "studies/NCT02576665");
        assert_eq!(plan.query.len(), 1);
        assert_eq!(plan.query_value("fields"), Some(expected_fields));
    }
}
