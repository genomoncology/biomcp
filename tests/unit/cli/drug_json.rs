//! Drug search JSON rendering tests kept outside production source.

use super::super::render::drug_search_json;

#[test]
fn drug_search_json_single_region_keeps_selected_bucket_and_who_fields() {
    let json = drug_search_json(
        crate::entities::drug::DrugSearchPageWithRegion::Who(
            crate::entities::SearchPage::offset(
                vec![crate::entities::drug::WhoPrequalificationSearchResult {
                    kind: crate::entities::drug::WhoPrequalificationKind::FinishedPharma,
                    inn: "Trastuzumab".to_string(),
                    product_type: "Biotherapeutic Product".to_string(),
                    therapeutic_area: "Oncology".to_string(),
                    presentation: Some(
                        "Trastuzumab Powder for concentrate for solution for infusion 150 mg"
                            .to_string(),
                    ),
                    dosage_form: Some(
                        "Powder for concentrate for solution for infusion".to_string(),
                    ),
                    applicant: "Samsung Bioepis NL B.V.".to_string(),
                    who_reference_number: Some("BT-ON001".to_string()),
                    who_product_id: None,
                    listing_basis: Some("Prequalification - Abridged".to_string()),
                    prequalification_date: Some("2019-12-18".to_string()),
                    vaccine_type: None,
                    commercial_name: None,
                    dose_count: None,
                    manufacturer: None,
                    responsible_nra: None,
                }],
                Some(1),
            )
            .into(),
        ),
        Some("trastuzumab"),
        0,
        5,
        None,
    )
    .expect("WHO search json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["region"], "who");
    assert_eq!(
        value["regions"].as_object().map(|regions| regions.len()),
        Some(1)
    );
    assert!(value.get("pagination").is_none());
    assert!(value.get("count").is_none());
    assert!(value.get("results").is_none());
    assert!(value.get("query").is_none());
    assert_eq!(value["regions"]["who"]["count"], 1);
    assert_eq!(value["regions"]["who"]["pagination"]["returned"], 1);
    assert_eq!(
        value["regions"]["who"]["results"][0]["who_reference_number"],
        "BT-ON001"
    );
    assert_eq!(
        value["regions"]["who"]["results"][0]["product_type"],
        "Biotherapeutic Product"
    );
    assert_eq!(
        value["regions"]["who"]["results"][0]["listing_basis"],
        "Prequalification - Abridged"
    );
    assert_eq!(
        value["regions"]["who"]["results"][0]["prequalification_date"],
        "2019-12-18"
    );
    assert!(
        value["regions"]["who"]["results"][0]
            .get("matched_term")
            .is_none()
    );
    assert!(
        value["regions"]["who"]["results"][0]
            .get("source")
            .is_none()
    );
    assert_eq!(
        value["_meta"]["next_commands"][0],
        serde_json::Value::String("biomcp get drug Trastuzumab".into())
    );
    assert_eq!(
        value["_meta"]["next_commands"][1],
        serde_json::Value::String("biomcp list drug".into())
    );
}

#[test]
fn drug_search_json_single_region_keeps_api_identifier_when_present() {
    let json = drug_search_json(
        crate::entities::drug::DrugSearchPageWithRegion::Who(
            crate::entities::SearchPage::offset(
                vec![crate::entities::drug::WhoPrequalificationSearchResult {
                    kind: crate::entities::drug::WhoPrequalificationKind::Api,
                    inn: "Artesunate".to_string(),
                    product_type: "Active Pharmaceutical Ingredient".to_string(),
                    therapeutic_area: "Malaria".to_string(),
                    presentation: None,
                    dosage_form: None,
                    applicant: "Ipca Laboratories Ltd".to_string(),
                    who_reference_number: None,
                    who_product_id: Some("WHOAPI-001".to_string()),
                    listing_basis: None,
                    prequalification_date: Some("2012-04-04".to_string()),
                    vaccine_type: None,
                    commercial_name: None,
                    dose_count: None,
                    manufacturer: None,
                    responsible_nra: None,
                }],
                Some(1),
            )
            .into(),
        ),
        Some("artesunate"),
        0,
        5,
        None,
    )
    .expect("WHO API search json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(
        value["regions"]["who"]["results"][0]["who_product_id"],
        "WHOAPI-001"
    );
    assert!(
        value["regions"]["who"]["results"][0]
            .get("who_reference_number")
            .is_none()
    );
    assert_eq!(
        value["regions"]["who"]["results"][0]["product_type"],
        "Active Pharmaceutical Ingredient"
    );
}

#[test]
fn drug_search_json_single_region_omits_get_follow_up_for_vaccine_results() {
    let json = drug_search_json(
        crate::entities::drug::DrugSearchPageWithRegion::Who(
            crate::entities::SearchPage::offset(
                vec![crate::entities::drug::WhoPrequalificationSearchResult {
                    kind: crate::entities::drug::WhoPrequalificationKind::Vaccine,
                    inn: "BCG".to_string(),
                    product_type: "Vaccine".to_string(),
                    therapeutic_area: "Vaccine".to_string(),
                    presentation: Some("Ampoule".to_string()),
                    dosage_form: None,
                    applicant: "Japan BCG Laboratory".to_string(),
                    who_reference_number: None,
                    who_product_id: None,
                    listing_basis: None,
                    prequalification_date: Some("1987-01-01".to_string()),
                    vaccine_type: Some("BCG".to_string()),
                    commercial_name: Some("BCG Freeze Dried Glutamate vaccine".to_string()),
                    dose_count: Some("10".to_string()),
                    manufacturer: Some("Japan BCG Laboratory".to_string()),
                    responsible_nra: Some("Pharmaceutical and Medical Devices Agency".to_string()),
                }],
                Some(1),
            )
            .into(),
        ),
        Some("BCG"),
        0,
        5,
        None,
    )
    .expect("WHO vaccine search json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["regions"]["who"]["results"][0]["vaccine_type"], "BCG");
    assert_eq!(
        value["regions"]["who"]["results"][0]["commercial_name"],
        "BCG Freeze Dried Glutamate vaccine"
    );
    assert_eq!(
        value["_meta"]["next_commands"],
        serde_json::json!(["biomcp list drug"])
    );
}

#[test]
fn drug_search_json_single_region_keeps_empty_selected_bucket_and_omits_meta() {
    let json = drug_search_json(
        crate::entities::drug::DrugSearchPageWithRegion::Eu(
            crate::entities::SearchPage::offset(
                Vec::<crate::entities::drug::EmaDrugSearchResult>::new(),
                Some(0),
            )
            .into(),
        ),
        Some("keytruda"),
        0,
        5,
        None,
    )
    .expect("empty EU search json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["region"], "eu");
    assert_eq!(
        value["regions"].as_object().map(|regions| regions.len()),
        Some(1)
    );
    assert!(value["regions"].get("us").is_none());
    assert!(value["regions"].get("who").is_none());
    assert_eq!(value["regions"]["eu"]["count"], 0);
    assert_eq!(value["regions"]["eu"]["pagination"]["returned"], 0);
    assert_eq!(value["regions"]["eu"]["results"], serde_json::json!([]));
    assert!(value.get("_meta").is_none());
    assert!(value.get("pagination").is_none());
    assert!(value.get("count").is_none());
    assert!(value.get("results").is_none());
    assert!(value.get("query").is_none());
}

#[test]
fn drug_search_json_preserves_region_envelope_with_workflow_meta() {
    let workflow =
        crate::workflow_ladders::meta_for(crate::workflow_ladders::Workflow::TreatmentLookup)
            .expect("workflow metadata");
    let expected_playbook = workflow.playbook.clone();
    let json = drug_search_json(
        crate::entities::drug::DrugSearchPageWithRegion::Us(
            crate::entities::SearchPage::offset(
                vec![crate::entities::drug::DrugSearchResult {
                    name: "pyridostigmine".to_string(),
                    drugbank_id: None,
                    drug_type: None,
                    mechanism: None,
                    target: None,
                }],
                Some(1),
            )
            .into(),
        ),
        None,
        0,
        5,
        Some(workflow),
    )
    .expect("US search json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["region"], "us");
    assert_eq!(value["regions"]["us"]["count"], 1);
    assert!(
        value["regions"]["us"]["results"][0]
            .get("matched_term")
            .is_none()
    );
    assert!(value["regions"]["us"]["results"][0].get("source").is_none());
    assert_eq!(value["_meta"]["workflow"], "treatment-lookup");
    assert_eq!(value["_meta"]["workflow_playbook"], expected_playbook);
    assert!(value["_meta"].get("ladder").is_none());
    assert_eq!(
        value["_meta"]["next_commands"][0],
        serde_json::Value::String("biomcp get drug pyridostigmine".into())
    );
}

#[test]
fn drug_search_json_all_region_uses_unified_regions_envelope() {
    let json = drug_search_json(
        crate::entities::drug::DrugSearchPageWithRegion::All {
            us: crate::entities::SearchPage::offset(
                vec![crate::entities::drug::DrugSearchResult {
                    name: "pembrolizumab".to_string(),
                    drugbank_id: None,
                    drug_type: None,
                    mechanism: None,
                    target: Some("PDCD1".to_string()),
                }],
                Some(1),
            )
            .into(),
            eu: crate::entities::SearchPage::offset(
                vec![crate::entities::drug::EmaDrugSearchResult {
                    name: "Keytruda".to_string(),
                    active_substance: "pembrolizumab".to_string(),
                    ema_product_number: "EMEA/H/C/003820".to_string(),
                    status: "Authorised".to_string(),
                    match_kind: "active_substance".to_string(),
                    matched_term: "pembrolizumab".to_string(),
                    source: "query".to_string(),
                }],
                Some(1),
            )
            .into(),
            who: crate::entities::SearchPage::offset(
                vec![crate::entities::drug::WhoPrequalificationSearchResult {
                    kind: crate::entities::drug::WhoPrequalificationKind::FinishedPharma,
                    inn: "Pembrolizumab".to_string(),
                    product_type: "Biotherapeutic Product".to_string(),
                    therapeutic_area: "Oncology".to_string(),
                    presentation: Some("Pembrolizumab Concentrate".to_string()),
                    dosage_form: Some("Concentrate".to_string()),
                    applicant: "Merck Sharp & Dohme".to_string(),
                    who_reference_number: Some("BT-ON002".to_string()),
                    who_product_id: None,
                    listing_basis: Some("Prequalification".to_string()),
                    prequalification_date: Some("2020-01-01".to_string()),
                    vaccine_type: None,
                    commercial_name: None,
                    dose_count: None,
                    manufacturer: None,
                    responsible_nra: None,
                }],
                Some(1),
            )
            .into(),
        },
        Some("keytruda"),
        0,
        5,
        None,
    )
    .expect("all-region drug search json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["region"], "all");
    assert_eq!(
        value["regions"].as_object().map(|regions| regions.len()),
        Some(3)
    );
    assert!(value.get("pagination").is_none());
    assert!(value.get("count").is_none());
    assert!(value.get("results").is_none());
    assert!(value.get("query").is_none());
    assert_eq!(value["regions"]["us"]["count"], 1);
    assert_eq!(value["regions"]["eu"]["count"], 1);
    assert_eq!(value["regions"]["who"]["count"], 1);
    assert_eq!(
        value["regions"]["us"]["results"][0],
        serde_json::json!({
            "name": "pembrolizumab",
            "target": "PDCD1",
            "match_kind": "broad_text"
        })
    );
    assert_eq!(
        value["regions"]["eu"]["results"][0],
        serde_json::json!({
            "name": "Keytruda",
            "active_substance": "pembrolizumab",
            "ema_product_number": "EMEA/H/C/003820",
            "status": "Authorised",
            "match_kind": "active_substance",
            "matched_term": "pembrolizumab",
            "source": "query"
        })
    );
    assert_eq!(
        value["regions"]["who"]["results"][0],
        serde_json::json!({
            "inn": "Pembrolizumab",
            "product_type": "Biotherapeutic Product",
            "therapeutic_area": "Oncology",
            "presentation": "Pembrolizumab Concentrate",
            "dosage_form": "Concentrate",
            "applicant": "Merck Sharp & Dohme",
            "who_reference_number": "BT-ON002",
            "listing_basis": "Prequalification",
            "prequalification_date": "2020-01-01",
            "match_kind": "broad_text"
        })
    );
    assert_eq!(
        value["_meta"]["next_commands"][0],
        serde_json::Value::String("biomcp get drug Keytruda".into())
    );
    assert_eq!(
        value["_meta"]["next_commands"][1],
        serde_json::Value::String("biomcp list drug".into())
    );
}

#[test]
fn drug_search_json_all_region_keeps_empty_buckets() {
    let json = drug_search_json(
        crate::entities::drug::DrugSearchPageWithRegion::All {
            us: crate::entities::SearchPage::offset(Vec::new(), Some(0)).into(),
            eu: crate::entities::SearchPage::offset(
                vec![crate::entities::drug::EmaDrugSearchResult {
                    name: "Keytruda".to_string(),
                    active_substance: "pembrolizumab".to_string(),
                    ema_product_number: "EMEA/H/C/003820".to_string(),
                    status: "Authorised".to_string(),
                    match_kind: "active_substance".to_string(),
                    matched_term: "pembrolizumab".to_string(),
                    source: "query".to_string(),
                }],
                Some(1),
            )
            .into(),
            who: crate::entities::SearchPage::offset(Vec::new(), Some(0)).into(),
        },
        Some("keytruda"),
        0,
        5,
        None,
    )
    .expect("all-region empty bucket json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["regions"]["us"]["count"], 0);
    assert_eq!(value["regions"]["us"]["results"], serde_json::json!([]));
    assert_eq!(value["regions"]["who"]["count"], 0);
    assert_eq!(value["regions"]["who"]["results"], serde_json::json!([]));
    assert_eq!(
        value["_meta"]["next_commands"][0],
        serde_json::Value::String("biomcp get drug Keytruda".into())
    );
}

#[test]
fn drug_search_json_orders_match_tiers_stably_within_one_region() {
    let rows = vec![
        crate::entities::drug::EmaDrugSearchResult {
            name: "Pembrolizumab".into(),
            active_substance: "different".into(),
            ema_product_number: "2".into(),
            status: "Authorised".into(),
            match_kind: "product_name".into(),
            matched_term: "pembrolizumab".into(),
            source: "query".into(),
        },
        crate::entities::drug::EmaDrugSearchResult {
            name: "Keytruda".into(),
            active_substance: "Pembrolizumab".into(),
            ema_product_number: "3".into(),
            status: "Authorised".into(),
            match_kind: "active_substance".into(),
            matched_term: "pembrolizumab".into(),
            source: "query".into(),
        },
        crate::entities::drug::EmaDrugSearchResult {
            name: "Earlier broad product".into(),
            active_substance: "different".into(),
            ema_product_number: "1".into(),
            status: "Authorised".into(),
            match_kind: "broad_text".into(),
            matched_term: "pembrolizumab".into(),
            source: "query".into(),
        },
    ];
    let json = drug_search_json(
        crate::entities::drug::DrugSearchPageWithRegion::Eu(
            crate::entities::drug::RankedDrugSearchPage::offset(
                rows,
                Some(3),
                vec![
                    crate::entities::drug::DrugSearchMatchKind::ProductName,
                    crate::entities::drug::DrugSearchMatchKind::ActiveSubstance,
                    crate::entities::drug::DrugSearchMatchKind::BroadText,
                ],
            ),
        ),
        Some("pembrolizumab"),
        0,
        3,
        None,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let results = value["regions"]["eu"]["results"].as_array().unwrap();
    assert_eq!(results[0]["match_kind"], "product_name");
    assert_eq!(results[1]["match_kind"], "active_substance");
    assert_eq!(results[2]["match_kind"], "broad_text");
    assert_eq!(results[0]["matched_term"], "pembrolizumab");
    assert_eq!(results[0]["source"], "query");
}
