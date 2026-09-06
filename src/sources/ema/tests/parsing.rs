//! Tier 3 - feed parsing and local result shaping. Pure: reads committed EMA
//! fixtures from disk and validates output structs. No network.

use super::super::*;

fn fixture_client() -> EmaClient {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("spec")
        .join("fixtures")
        .join("ema-human");
    EmaClient::from_root(root)
}

#[test]
fn validate_feed_payload_rejects_bad_payloads_before_write() {
    let err = validate_feed_payload(EMA_FEEDS[0], b"<html>error</html>")
        .expect_err("html should fail JSON validation");
    assert!(format!("{err:?}").contains("ApiJson"));

    let err = validate_feed_payload(EMA_FEEDS[0], br#"{"data":"oops"}"#)
        .expect_err("missing array should fail");
    assert!(format!("{err:?}").contains("top-level `data` array"));
}

#[test]
fn resolve_anchor_matches_brand_and_filters_non_human_rows() {
    let client = fixture_client();
    let anchor = client
        .resolve_anchor(&EmaDrugIdentity::new("Keytruda"))
        .expect("anchor");

    assert_eq!(anchor.medicines.len(), 1);
    assert_eq!(anchor.medicines[0].medicine_name, "Keytruda");
    assert_eq!(anchor.medicines[0].ema_product_number, "EMEA/H/C/003820");
}

#[test]
fn regulatory_reads_live_schema_holder_key_and_cleaned_indication() {
    let client = fixture_client();
    let anchor = client
        .resolve_anchor(&EmaDrugIdentity::new("Dupixent"))
        .expect("anchor");
    let regulatory = client.regulatory(&anchor).expect("regulatory");
    let row = regulatory.first().expect("dupixent row");

    assert_eq!(row.holder.as_deref(), Some("Sanofi Winthrop Industrie"));
    assert_eq!(
        row.marketing_authorisation_date.as_deref(),
        Some("26/09/2017")
    );
    let indication = row
        .therapeutic_indication
        .as_deref()
        .expect("therapeutic indication");
    assert!(indication.contains("atopic dermatitis"));
    assert!(!indication.contains("&nbsp;"));
    assert!(!indication.contains('<'));
}

#[test]
fn search_medicines_matches_therapeutic_indication_queries() {
    let client = fixture_client();
    let page = client
        .search_medicines(&EmaDrugIdentity::new("influenza"), 10, 0)
        .expect("search page");
    let names = page
        .results
        .iter()
        .map(|row| row.name.as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"Flucelvax Tetra"));
    assert!(names.contains(&"Fluad Tetra"));
    assert!(
        page.match_kinds
            .iter()
            .all(|kind| *kind == DrugSearchMatchKind::BroadText)
    );
}

#[test]
fn search_classifies_deduplicates_and_paginates_after_stable_tiering() {
    let root = crate::test_support::TempDirGuard::new("ema-search-ranking");
    let rows = serde_json::json!({"data": [
        {
            "name_of_medicine": ["First broad row"],
            "active_substance": ["other"],
            "ema_product_number": ["P1"],
            "medicine_status": ["Earlier broad"],
            "category": ["Human"],
            "therapeutic_indication": ["Treatment for target phrase disease"]
        },
        {
            "name_of_medicine": ["Target phrase"],
            "active_substance": ["other"],
            "ema_product_number": ["P2"],
            "medicine_status": ["First exact"],
            "category": ["Human"]
        },
        {
            "name_of_medicine": ["Alias product"],
            "active_substance": ["Generic X"],
            "ema_product_number": ["P3"],
            "medicine_status": ["Alias"],
            "category": ["Human"]
        },
        {
            "name_of_medicine": ["Target phrase"],
            "active_substance": ["replacement substance"],
            "ema_product_number": ["p1"],
            "medicine_status": ["Later exact replacement"],
            "category": ["Human"]
        },
        {
            "name_of_medicine": ["Unrelated"],
            "active_substance": ["acid"],
            "ema_product_number": ["P4"],
            "medicine_status": ["Must disappear"],
            "category": ["Human"]
        }
    ]});
    std::fs::write(
        root.path().join(MEDICINES_FILE),
        serde_json::to_vec(&rows).expect("fixture json"),
    )
    .expect("fixture write");
    let client = EmaClient::from_root(root.path().to_path_buf());
    let identity = EmaDrugIdentity::from_typed_terms(
        "target phrase",
        vec![("Generic X".into(), EmaIdentitySource::OpenFdaGenericName)],
    );

    let first = client
        .search_medicines(&identity, 1, 0)
        .expect("first page");
    assert_eq!(first.total, Some(3));
    assert_eq!(first.results[0].ema_product_number, "p1");
    assert_eq!(first.results[0].status, "Later exact replacement");
    assert_eq!(first.results[0].match_kind, "product_name");
    assert_eq!(first.results[0].matched_term, "target phrase");
    assert_eq!(first.results[0].source, "query");

    let middle = client
        .search_medicines(&identity, 1, 1)
        .expect("middle page");
    assert_eq!(middle.results[0].ema_product_number, "P2");
    let final_page = client
        .search_medicines(&identity, 1, 2)
        .expect("final page");
    assert_eq!(final_page.results[0].ema_product_number, "P3");
    assert_eq!(final_page.results[0].source, "openfda.generic_name");
    assert!(
        client
            .search_medicines(&identity, 1, 3)
            .expect("empty")
            .results
            .is_empty()
    );
    assert!(
        client
            .search_medicines(&identity, 1, 30)
            .expect("out of range")
            .results
            .is_empty()
    );
}

#[test]
fn search_medicines_matches_cvx_alias_tokens_on_active_substance() {
    let client = fixture_client();
    let aliases = vec![
        (
            "Pneumococcal conjugate PCV 13".to_string(),
            EmaIdentitySource::CvxShortDescription,
        ),
        (
            "pneumococcal conjugate vaccine, 13 valent".to_string(),
            EmaIdentitySource::CvxFullVaccineName,
        ),
        (
            "pneumococcal polysaccharide conjugate vaccine (13 valent, adsorbed)".to_string(),
            EmaIdentitySource::CvxFullVaccineName,
        ),
    ];
    let page = client
        .search_medicines(
            &EmaDrugIdentity::from_typed_terms("prevnar", aliases),
            10,
            0,
        )
        .expect("search page");
    let names = page
        .results
        .iter()
        .map(|row| row.name.as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"Prevenar 13"));
    assert!(page.match_kinds.contains(&DrugSearchMatchKind::Alias));
}

#[test]
fn cvx_signatures_keep_distinctive_initialisms_and_ordered_constraints() {
    assert_eq!(cvx_signature("HPV vaccine"), Some(vec!["hpv".into()]));
    assert_eq!(cvx_signature("HPV9 vaccine"), Some(vec!["hpv9".into()]));
    assert_eq!(
        cvx_signature("Pneumococcal conjugate PCV 13"),
        Some(vec!["pneumococcal".into(), "pcv".into(), "13".into()])
    );
    assert_eq!(cvx_signature("abc vaccine 13"), None);
    assert_eq!(cvx_signature("vaccine conjugate trivalent 13"), None);
}

#[test]
fn cvx_ordered_signature_compacts_but_does_not_reverse_or_cross_fields() {
    assert!(cvx_description_matches(
        "human papilloma virus vaccine, quadrivalent",
        &["Silgard", "human papillomavirus vaccine, quadrivalent", ""]
    ));
    assert!(!cvx_description_matches(
        "pneumococcal vaccine 13",
        &["13 product", "pneumococcal substance", ""]
    ));
    assert!(!cvx_description_matches(
        "pneumococcal vaccine 13",
        &["13 pneumococcal", "", ""]
    ));
}

#[test]
fn ema_truth_table_reports_every_typed_source_and_stable_ties() {
    let product = EmaDrugIdentity::new("Product");
    let matched = classify_ema_match(
        &product,
        Some("product"),
        Some("other"),
        &["Product", "other", ""],
    )
    .expect("product exact");
    assert_eq!(matched.0, DrugSearchMatchKind::ProductName);
    assert_eq!(
        (matched.1.as_str(), matched.2.as_str()),
        ("Product", "query")
    );

    let active = EmaDrugIdentity::new("Substance");
    assert_eq!(
        classify_ema_match(
            &active,
            Some("other"),
            Some("substance"),
            &["other", "Substance", ""]
        )
        .expect("active exact")
        .0,
        DrugSearchMatchKind::ActiveSubstance
    );

    for source in [
        EmaIdentitySource::OpenFdaGenericName,
        EmaIdentitySource::NdcNonproprietaryName,
        EmaIdentitySource::DrugbankName,
        EmaIdentitySource::ChemblPrefName,
        EmaIdentitySource::OpenFdaBrandName,
    ] {
        let identity =
            EmaDrugIdentity::from_typed_terms("request", vec![("Typed alias".into(), source)]);
        let matched = classify_ema_match(
            &identity,
            Some("other"),
            Some("typed alias"),
            &["other", "Typed alias", ""],
        )
        .expect("typed alias exact");
        assert_eq!(matched.0, DrugSearchMatchKind::Alias);
        assert_eq!(matched.2, source);
    }

    let stable_tie = EmaDrugIdentity::from_typed_terms(
        "request",
        vec![
            ("Same alias".into(), EmaIdentitySource::DrugbankName),
            ("same alias".into(), EmaIdentitySource::OpenFdaBrandName),
        ],
    );
    assert_eq!(
        classify_ema_match(
            &stable_tie,
            Some("same alias"),
            Some("other"),
            &["same alias", "other", ""],
        )
        .expect("stable alias tie")
        .2,
        EmaIdentitySource::DrugbankName
    );

    let cvx = EmaDrugIdentity::from_typed_terms(
        "request",
        vec![("HPV".into(), EmaIdentitySource::CvxShortDescription)],
    );
    assert_eq!(
        classify_ema_match(&cvx, Some("hpv"), Some("other"), &["HPV", "other", ""])
            .expect("CVX exact")
            .2,
        EmaIdentitySource::CvxShortDescription
    );

    let broad = EmaDrugIdentity::new("target phrase");
    let matched = classify_ema_match(
        &broad,
        Some("other"),
        Some("other"),
        &["other", "other", "for target phrase disease"],
    )
    .expect("boundary phrase");
    assert_eq!(matched.0, DrugSearchMatchKind::BroadText);
    assert_eq!(matched.2, EmaIdentitySource::Query);
}

#[test]
fn recorded_cvx_descriptions_bridge_gardasil_prevnar_and_fluzone() {
    let client = fixture_client();
    let identity = EmaDrugIdentity::from_typed_terms(
        "brand",
        vec![
            (
                "human papilloma virus vaccine, quadrivalent".into(),
                EmaIdentitySource::CvxFullVaccineName,
            ),
            (
                "pneumococcal conjugate vaccine, 13 valent".into(),
                EmaIdentitySource::CvxFullVaccineName,
            ),
            (
                "Influenza, split virus, trivalent, injectable, preservative free".into(),
                EmaIdentitySource::CvxFullVaccineName,
            ),
        ],
    );
    let page = client.search_medicines(&identity, 10, 0).expect("search");
    let names = page
        .results
        .iter()
        .map(|row| row.name.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"Silgard"));
    assert!(names.contains(&"Prevenar 13"));
    assert!(names.contains(&"Flucelvax Tetra"));
    assert!(names.contains(&"Fluad Tetra"));
    assert!(page.results.iter().all(|row| row.match_kind == "alias"));
}

#[test]
fn safety_ozempic_has_dhpcs_but_empty_referrals_and_psusas() {
    let client = fixture_client();
    let anchor = client
        .resolve_anchor(&EmaDrugIdentity::new("Ozempic"))
        .expect("anchor");
    let safety = client.safety(&anchor).expect("safety");

    assert_eq!(safety.dhpcs.len(), 4);
    assert!(safety.referrals.is_empty());
    assert!(safety.psusas.is_empty());
}

#[test]
fn shortage_matches_resolved_human_medicine_anchor() {
    let client = fixture_client();
    let anchor = client
        .resolve_anchor(&EmaDrugIdentity::new("Ozempic"))
        .expect("anchor");
    let shortages = client.shortages(&anchor).expect("shortages");

    assert_eq!(shortages.len(), 1);
    assert_eq!(shortages[0].status.as_deref(), Some("Resolved"));
    assert_eq!(
        shortages[0].availability_of_alternatives.as_deref(),
        Some("Yes")
    );
}
