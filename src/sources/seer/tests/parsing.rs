//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to the SEER
//! decoders and local resolver logic. No network, no server.

use super::super::*;
use crate::entities::disease::Disease;
use crate::error::BioMcpError;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde_json::Value;
use std::collections::HashMap;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/seer/",
            $name
        ))
    };
}

fn test_catalog() -> SeerSiteCatalog {
    SeerClient::decode_site_catalog_response(
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/json")),
        fixture!("site_catalog.json"),
    )
    .expect("site catalog")
}

fn survival_inner_json(site_code: u16) -> String {
    let mut value: Value = serde_json::from_slice(fixture!("survival_payload_97.json")).unwrap();
    if site_code != 97 {
        let old_key = "5_1_1_1_97";
        let old_male_key = "5_2_1_1_97";
        let old_age_key = "5_1_1_157_97";
        let old_race_key = "5_1_2_1_97";
        let data = value["data"].as_object_mut().unwrap();
        let both = data.remove(old_key).unwrap();
        let male = data.remove(old_male_key).unwrap();
        let age = data.remove(old_age_key).unwrap();
        let race = data.remove(old_race_key).unwrap();
        data.insert(format!("5_1_1_1_{site_code}"), both);
        data.insert(format!("5_2_1_1_{site_code}"), male);
        data.insert(format!("5_1_1_157_{site_code}"), age);
        data.insert(format!("5_1_2_1_{site_code}"), race);
    }
    serde_json::to_string(&value).unwrap()
}

fn disease(name: &str, synonyms: Vec<String>) -> Disease {
    Disease {
        id: "MONDO:123".to_string(),
        name: name.to_string(),
        definition: None,
        synonyms,
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: Vec::new(),
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        survival: None,
        survival_note: None,
        civic: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: HashMap::new(),
    }
}

#[test]
fn site_catalog_decodes_live_variable_formats() {
    let catalog = test_catalog();

    assert_eq!(
        catalog.site_label(97),
        Some("Chronic Myeloid Leukemia (CML)")
    );
    assert_eq!(catalog.sex_label(2), Some("Male"));
    assert_eq!(catalog.race_label(1), Some("All Races / Ethnicities"));
    assert_eq!(catalog.age_range_label(157), Some("Ages 65+"));
    assert!(catalog.is_active_site(97));
    assert!(!catalog.is_active_site(56));
}

#[test]
fn decode_json_response_maps_bad_status_and_content_type_to_source_unavailable() {
    let err = SeerClient::decode_json_response::<Value>(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(&HeaderValue::from_static("application/json")),
        b"upstream failed",
    )
    .unwrap_err();
    assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
    assert!(format!("{err:?}").contains("HTTP 500"));

    let err = SeerClient::decode_json_response::<Value>(
        StatusCode::OK,
        Some(&HeaderValue::from_static("text/html")),
        b"<html></html>",
    )
    .unwrap_err();
    assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
}

#[test]
fn body_limit_error_remains_source_unavailable() {
    let err = remap_seer_error(BioMcpError::BodyLimit {
        source_name: SEER_API.to_string(),
        max_bytes: crate::sources::DEFAULT_MAX_BODY_BYTES,
    });

    assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
}

#[test]
fn decode_double_encoded_survival_payload_and_filter_all_ages() {
    let catalog = test_catalog();
    let payload =
        SeerClient::decode_survival_payload(97, &catalog, &survival_inner_json(97)).unwrap();

    assert_eq!(payload.site_code, 97);
    assert_eq!(payload.site_label, "Chronic Myeloid Leukemia (CML)");
    assert_eq!(payload.series.len(), 2);
    assert_eq!(payload.series[0].sex_label, "Both Sexes");
    assert_eq!(payload.series[0].points.len(), 3);
    assert_eq!(payload.series[0].points[1].year, 2017);
    assert_eq!(
        payload.series[0].points[1].relative_survival_rate,
        Some(69.4)
    );
    assert_eq!(
        payload.series[0].points[2].modeled_relative_survival_rate,
        Some(70.0)
    );
    assert_eq!(payload.series[1].sex_label, "Male");
}

#[test]
fn resolve_site_prefers_exact_alias_and_rejects_ambiguous_matches() {
    let catalog = test_catalog();

    let cml = disease("CML", vec!["Chronic myelogenous leukemia".to_string()]);
    assert_eq!(
        resolve_site(&cml, &catalog),
        Some(ResolvedSeerSite {
            site_code: 97,
            site_label: "Chronic Myeloid Leukemia (CML)".to_string(),
        })
    );

    let ambiguous = disease("Leukemia", vec!["CML".to_string()]);
    assert_eq!(resolve_site(&ambiguous, &catalog), None);

    let breast = disease("breast cancer", Vec::new());
    assert_eq!(
        resolve_site(&breast, &catalog),
        Some(ResolvedSeerSite {
            site_code: 55,
            site_label: "Breast".to_string(),
        })
    );

    let breast_carcinoma = disease("breast carcinoma", vec!["carcinoma of breast".to_string()]);
    assert_eq!(
        resolve_site(&breast_carcinoma, &catalog),
        Some(ResolvedSeerSite {
            site_code: 55,
            site_label: "Breast".to_string(),
        })
    );
}

#[test]
fn rejects_response_when_requested_site_code_is_not_returned() {
    let catalog = test_catalog();
    let err = SeerClient::decode_survival_payload(97, &catalog, &survival_inner_json(1))
        .expect_err("site mismatch should fail");

    assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
    assert!(format!("{err:?}").contains("different site"));
}
