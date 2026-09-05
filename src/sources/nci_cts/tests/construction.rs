//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query / header that would be sent. Nothing is sent.

use crate::sources::HttpMethod;
use crate::sources::nci_cts::{
    NciCtsClient, NciDiseaseFilter, NciGeoFilter, NciSearchParams, NciStatusFilter,
};

fn params() -> NciSearchParams {
    NciSearchParams {
        size: 2,
        from: 0,
        ..Default::default()
    }
}

#[test]
fn search_plan_sets_method_path_and_api_key_header() {
    let plan = NciCtsClient::search_plan("test-key", &params());
    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "trials");
    assert_eq!(plan.header_value("X-API-KEY"), Some("test-key"));
    assert_eq!(plan.query_value("size"), Some("2"));
    assert_eq!(plan.query_value("from"), Some("0"));
}

#[test]
fn search_plan_keyword_disease_maps_to_keyword_param() {
    let p = NciSearchParams {
        disease: Some(NciDiseaseFilter::Keyword("melanoma".into())),
        ..params()
    };
    let plan = NciCtsClient::search_plan("test-key", &p);
    assert_eq!(plan.query_value("keyword"), Some("melanoma"));
    assert!(!plan.has_query("diseases.nci_thesaurus_concept_id"));
}

#[test]
fn search_plan_concept_id_disease_maps_to_concept_param() {
    let p = NciSearchParams {
        disease: Some(NciDiseaseFilter::ConceptId("C3224".into())),
        ..params()
    };
    let plan = NciCtsClient::search_plan("test-key", &p);
    assert_eq!(
        plan.query_value("diseases.nci_thesaurus_concept_id"),
        Some("C3224")
    );
    assert!(!plan.has_query("keyword"));
}

#[test]
fn search_plan_includes_sites_org_name() {
    let p = NciSearchParams {
        sites_org_name: Some("MD Anderson".into()),
        ..params()
    };
    let plan = NciCtsClient::search_plan("test-key", &p);
    assert_eq!(plan.query_value("sites.org_name"), Some("MD Anderson"));
}

#[test]
fn search_plan_serializes_status_phase_and_geo_contract_params() {
    let p = NciSearchParams {
        disease: Some(NciDiseaseFilter::Keyword("melanoma".into())),
        status: Some(NciStatusFilter::SiteRecruitmentStatus("ACTIVE".into())),
        phases: vec!["I_II".into()],
        geo: Some(NciGeoFilter {
            lat: 41.9742,
            lon: -87.8073,
            distance_miles: 100,
        }),
        ..params()
    };
    let plan = NciCtsClient::search_plan("test-key", &p);
    assert_eq!(plan.query_value("keyword"), Some("melanoma"));
    assert!(!plan.has_query("diseases.nci_thesaurus_concept_id"));
    assert_eq!(plan.query_value("sites.recruitment_status"), Some("ACTIVE"));
    assert!(!plan.has_query("current_trial_status"));
    assert_eq!(plan.query_value("phase"), Some("I_II"));
    assert_eq!(
        plan.query_value("sites.org_coordinates_lat"),
        Some("41.9742")
    );
    assert_eq!(
        plan.query_value("sites.org_coordinates_lon"),
        Some("-87.8073")
    );
    assert_eq!(
        plan.query_value("sites.org_coordinates_dist"),
        Some("100mi")
    );
}

#[test]
fn search_plan_current_trial_status_variant() {
    let p = NciSearchParams {
        status: Some(NciStatusFilter::CurrentTrialStatus("Complete".into())),
        ..params()
    };
    let plan = NciCtsClient::search_plan("test-key", &p);
    assert_eq!(plan.query_value("current_trial_status"), Some("Complete"));
    assert!(!plan.has_query("sites.recruitment_status"));
}

#[test]
fn search_plan_skips_blank_phases() {
    let p = NciSearchParams {
        phases: vec!["   ".into(), "II".into()],
        ..params()
    };
    let plan = NciCtsClient::search_plan("test-key", &p);
    assert_eq!(plan.query_value("phase"), Some("II"));
}

#[test]
fn search_plan_includes_interventions_and_biomarkers_when_present() {
    let p = NciSearchParams {
        interventions: Some("vemurafenib".into()),
        biomarkers: Some("BRAF".into()),
        ..params()
    };
    let plan = NciCtsClient::search_plan("test-key", &p);
    assert_eq!(plan.query_value("interventions"), Some("vemurafenib"));
    assert_eq!(plan.query_value("biomarkers"), Some("BRAF"));
}

#[test]
fn get_plan_executes_the_exact_biodata_detail_plan_with_one_credential_header() {
    let biodata_plan = biodata::NciCtsV2DetailPlan::new("nct01234567", true).unwrap();
    let plan = NciCtsClient::get_plan("test-key", &biodata_plan);
    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "trials");
    assert_eq!(plan.header_value("X-API-KEY"), Some("test-key"));
    assert_eq!(
        plan.headers
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("X-API-KEY"))
            .count(),
        1
    );
    let expected = biodata_plan
        .query_pairs()
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(plan.query, expected);
    assert_eq!(
        plan.query
            .iter()
            .filter(|(name, _)| name == "include")
            .count(),
        16
    );
    let request_text = format!("{:?}{:?}", plan.path, plan.query);
    for forbidden in [
        "test-key",
        "sites",
        "contact",
        "principal_investigator",
        "email",
        "phone",
        "primary_purpose",
    ] {
        assert!(!request_text.to_ascii_lowercase().contains(forbidden));
    }
}
