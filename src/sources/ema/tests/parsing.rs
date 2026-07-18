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
        .search_medicines(&EmaDrugIdentity::new("influenza vaccine"), 10, 0)
        .expect("search page");
    let names = page
        .results
        .iter()
        .map(|row| row.name.as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"Flucelvax Tetra"));
    assert!(names.contains(&"Fluad Tetra"));
}

#[test]
fn search_medicines_matches_cvx_alias_tokens_on_active_substance() {
    let client = fixture_client();
    let aliases = vec![
        "Pneumococcal conjugate PCV 13".to_string(),
        "pneumococcal conjugate vaccine, 13 valent".to_string(),
    ];
    let page = client
        .search_medicines(
            &EmaDrugIdentity::with_aliases("prevnar", None, &aliases),
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
