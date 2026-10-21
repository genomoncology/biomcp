use super::*;
use crate::entities::drug::Drug;
use crate::sources::mychem::MyChemHit;
use crate::sources::openfda::{DrugsFdaResult, OpenFdaResponse};

macro_rules! recorded {
    ($source:expr, $name:expr) => {
        serde_json::from_slice(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/",
            $source,
            "/",
            $name
        )))
        .expect("recorded reply parses")
    };
}

/// A merged card named for the given drug, the real construction path a
/// label answer applies to.
fn card_named(name: &str) -> Drug {
    let hit: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "TESTCARD",
        "_score": 1.0,
        "drugbank": {"id": "DBTESTCARD", "name": name},
    }))
    .expect("valid card hit");
    let selected = [&hit];
    crate::transform::drug::merge_mychem_hits(&selected, name)
}

#[test]
fn map_drugsfda_approvals_extracts_key_fields() {
    let resp: OpenFdaResponse<DrugsFdaResult> = serde_json::from_value(serde_json::json!({
        "meta": {"results": {"skip": 0, "limit": 1, "total": 1}},
        "results": [{
            "application_number": "NDA021304",
            "sponsor_name": "Example Pharma",
            "openfda": {
                "brand_name": ["DrugX"],
                "generic_name": ["drugx"]
            },
            "products": [{
                "brand_name": "DrugX",
                "dosage_form": "TABLET",
                "route": "ORAL",
                "marketing_status": "Prescription",
                "active_ingredients": [{"name": "drugx", "strength": "10 mg"}]
            }],
            "submissions": [{
                "submission_type": "ORIG",
                "submission_number": "1",
                "submission_status": "AP",
                "submission_status_date": "20120101"
            }]
        }]
    }))
    .expect("response should parse");

    let rows = map_drugsfda_approvals(resp, false);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].application_number, "NDA021304");
    assert_eq!(rows[0].openfda_brand_names, vec!["DrugX"]);
    assert_eq!(
        rows[0].products[0].active_ingredients,
        vec!["drugx (10 mg)"]
    );
    assert_eq!(
        rows[0].submissions[0].status_date.as_deref(),
        Some("2012-01-01")
    );
}

#[test]
fn extract_top_adverse_events_ranks_by_frequency() {
    let resp: crate::sources::openfda::OpenFdaCountResponse =
        serde_json::from_value(serde_json::json!({
            "meta": {},
            "results": [
                {"term": "Rash", "count": 2},
                {"term": "Nausea", "count": 1},
                {"term": "Fatigue", "count": 2}
            ]
        }))
        .expect("valid openfda response");

    let out = extract_top_adverse_events(&resp);
    assert_eq!(out, vec!["Fatigue", "Rash", "Nausea"]);
}

/// The recorded openFDA answer for the trastuzumab label search (2026-10-15):
/// the five newest records — four trastuzumab products and Enhertu, whose
/// generic line "fam-trastuzumab deruxtecan-nxki" is another drug that
/// only matched the search.
#[test]
fn apply_openfda_metadata_lists_only_the_cards_own_label_products() {
    let label_response: serde_json::Value = recorded!("openfda", "label_trastuzumab_20261015.json");
    let mut drug = card_named("Trastuzumab");
    assert_eq!(drug.name, "trastuzumab");

    apply_openfda_metadata(&mut drug, &label_response);

    assert_eq!(
        drug.brand_names,
        vec!["OGIVRI", "Kanjinti", "Herceptin", "Ontruzant"],
        "only the records whose generic line names trastuzumab list their brands"
    );
    assert!(
        !drug
            .brand_names
            .iter()
            .any(|brand| brand.eq_ignore_ascii_case("Enhertu")),
        "another drug's conjugate never lists as a trastuzumab brand"
    );
}

/// The recorded openFDA answer for the erlotinib hydrochloride label search
/// (2026-10-15): every record's generic line is the card's own, under three
/// label spellings of the brand. A typed brand the rescue resolved stays
/// listed ahead of the label's own products.
#[test]
fn apply_openfda_metadata_keeps_a_resolved_typed_brand_listed() {
    let label_response: serde_json::Value =
        recorded!("openfda", "label_erlotinib_hydrochloride_20261015.json");
    let mut drug = card_named("erlotinib hydrochloride");
    drug.brand_names = vec!["Tarceva".to_string()];

    apply_openfda_metadata(&mut drug, &label_response);

    assert_eq!(
        drug.brand_names.first().map(String::as_str),
        Some("Tarceva"),
        "the typed brand the rescue resolved stays listed"
    );
    // Generic spellings supplied as brands are excluded.
    assert_eq!(drug.brand_names, ["Tarceva", "Erlotinib"]);
}

#[test]
fn brand_lists_remove_generic_names_before_applying_the_cap() {
    let mut drug = card_named("erlotinib");
    drug.brand_names = vec!["Tarceva".into()];
    let labels = serde_json::json!({"results":[{"openfda":{
        "generic_name":["erlotinib"],
        "brand_name":["erlotinib","Brand A","Brand B","Brand C","Brand D","Brand E","Tarceva"]
    }}]});
    apply_openfda_metadata(&mut drug, &labels);
    assert_eq!(
        drug.brand_names,
        ["Tarceva", "Brand A", "Brand B", "Brand C", "Brand D"]
    );
}
