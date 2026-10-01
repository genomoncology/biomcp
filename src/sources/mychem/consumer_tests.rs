//! Frozen independent expectations from accepted BioData ticket 0224.
use super::{projection, *};
use biodata::{Document, MyChemFieldState, MyChemProfile};
use serde_json::{Value, json};
use std::path::PathBuf;

pub(crate) fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/drug_identity_oracles")
}
pub(crate) fn asset(path: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root().join(path)).unwrap()).unwrap()
}
pub(crate) fn table(number: u8) -> Vec<Value> {
    serde_json::from_value(asset(&format!("P{number}.json"))).unwrap()
}
pub(crate) fn bytes(case: &Value) -> Vec<u8> {
    let reference = case["input"]
        .get("page")
        .unwrap_or(&case["responses"][0]["body"]);
    std::fs::read(root().join(reference["path"].as_str().unwrap())).unwrap()
}
pub(crate) fn profile(case: &Value) -> MyChemProfile {
    if case["input"]["profile"] == "Search" {
        MyChemProfile::Search
    } else {
        MyChemProfile::Get
    }
}
pub(crate) fn assert_page(response: &MyChemQueryResponse, path: &str) {
    let expected = asset(path);
    assert_eq!(response.total as u64, expected["total"].as_u64().unwrap());
    assert_eq!(
        response.hits.len(),
        expected["rows"].as_array().unwrap().len()
    );
    for (hit, wanted) in response
        .hits
        .iter()
        .zip(expected["rows"].as_array().unwrap())
    {
        assert_eq!(hit.page.digest(), expected["digest"].as_str().unwrap());
        let document: Value = serde_json::from_str(
            &Document::DrugIdentity(hit.row.identity().clone())
                .to_json()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(document, wanted["document"]);
        let fields = hit.row.source().fields().iter().map(|field| {
            let values = match field.state() { MyChemFieldState::Values(values) | MyChemFieldState::Blank(values) => values.clone(), _ => Vec::new() };
            json!({"section":field.section(),"name":field.name(),"section_index":field.section_index(),
                "was_array":field.was_array(),"raw":field.raw(),"state":field.state().as_str(),"values":values})
        }).collect::<Vec<_>>();
        let raw: Value = serde_json::from_str(hit.row.source().raw()).unwrap();
        let mut enrichment = serde_json::Map::new();
        for (section, field) in [
            ("drugbank", "drug_interactions"),
            ("chembl", "molecule_type"),
            ("chembl", "drug_mechanisms"),
            ("chembl", "atc_classifications"),
            ("gtopdb", "interaction_targets"),
            ("ndc", "pharm_classes"),
            ("drugcentral", "approval"),
            ("drugcentral", "drug_use"),
        ] {
            if let Some(value) = raw.get(section).and_then(|value| value.get(field)) {
                enrichment.insert(format!("{section}.{field}"), value.clone());
            }
        }
        assert_eq!(
            json!({"ordinal":hit.row.source().ordinal(),"raw":hit.row.source().raw(),"fields":fields,
            "source_only_enrichment":enrichment}),
            wanted["companion"]
        );
        let losses = hit.row.losses().iter().map(|loss| json!({"field":loss.field(),"disposition":loss.disposition(),"reason":loss.reason()})).collect::<Vec<_>>();
        assert_eq!(json!(losses), wanted["adapter_losses"]);
    }
}
#[test]
fn original_byte_identity_and_companion_table() {
    let mut witnessed = std::collections::HashSet::new();
    for number in 1..=6 {
        for case in table(number) {
            let Some(path) = case["expected"]["page"].as_str() else {
                continue;
            };
            if !case["input"].get("page").is_some()
                && case["responses"].as_array().unwrap().len() != 1
            {
                continue;
            }
            if !witnessed.insert((path.to_owned(), format!("{:?}", profile(&case)))) {
                continue;
            }
            let input = bytes(&case);
            let expected = asset(path);
            if expected["rows"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["disposition"] == "Rejected")
            {
                assert!(projection::decode(&input, profile(&case)).is_err());
            } else if case["id"] == "P3-04/invalid-identity-companion" {
                let failure = projection::decode(&input, profile(&case)).unwrap_err();
                assert_eq!(failure.code(), "api_json");
            } else {
                let response = projection::decode(&input, profile(&case))
                    .unwrap_or_else(|error| panic!("{}: {error}", case["id"]));
                assert_page(&response, path);
            }
        }
    }
    assert!(witnessed.len() >= 25);
}
#[test]
fn request_profile_and_validation_table() {
    for case in table(1)
        .into_iter()
        .filter(|case| case["input"]["operation"] == "MyChemClient.request_plan")
    {
        let input = &case["input"];
        let result = MyChemClient::query_with_fields_plan(
            input["query"].as_str().unwrap(),
            input["size"].as_u64().unwrap() as usize,
            input["offset"].as_u64().unwrap() as usize,
            input["fields"].as_str().unwrap(),
        );
        if case["expected"]["failure"].is_null() {
            let plan = result.unwrap();
            assert_eq!(plan.path, case["expected"]["plan"]["plan_path"]);
            assert_eq!(json!(plan.query), case["expected"]["plan"]["query"]);
        } else {
            let failure = result.unwrap_err();
            assert_eq!(failure.code(), case["expected"]["failure"]["code"]);
            assert!(
                failure.to_string().contains(
                    case["expected"]["failure"]["diagnostic_contains"]
                        .as_str()
                        .unwrap()
                )
            );
        }
    }
}
#[test]
fn display_selection_and_product_table() {
    for number in [2, 3, 6] {
        for case in table(number) {
            let operation = case["input"]["operation"].as_str().unwrap_or("");
            if !matches!(
                operation,
                "admitted_row_then_from_mychem_search_hit"
                    | "admitted_page_get_selection_then_merge"
                    | "admitted_row_search_projection_and_get_merge"
                    | "admitted_identity_source_only_decode_then_merge"
                    | "composed accepted-page conversion"
            ) {
                continue;
            }
            let response = projection::decode(&bytes(&case), profile(&case)).unwrap();
            assert_page(&response, case["expected"]["page"].as_str().unwrap());
            let requested = case["input"]["requested_name"]
                .as_str()
                .unwrap_or("RequestedName");
            let selected = crate::transform::drug::select_hits_for_name(&response.hits, requested);
            if let Some(ordinals) = case["expected"]["selected_ordinals"].as_array() {
                assert_eq!(
                    json!(
                        selected
                            .iter()
                            .map(|hit| hit.row.source().ordinal())
                            .collect::<Vec<_>>()
                    ),
                    json!(ordinals)
                );
            }
            let product = crate::transform::drug::merge_mychem_hits(&selected, requested);
            let wanted = case["expected"]
                .get("product")
                .or_else(|| case["expected"].get("get_product"));
            if let Some(wanted) = wanted {
                assert_eq!(
                    serde_json::to_value(&product).unwrap(),
                    *wanted,
                    "{}",
                    case["id"]
                );
            }
            if let Some(wanted) = case["expected"].get("search_result") {
                assert_eq!(
                    serde_json::to_value(crate::transform::drug::from_mychem_search_hit(
                        &response.hits[0]
                    ))
                    .unwrap(),
                    *wanted,
                    "{}",
                    case["id"]
                );
            }
            // Product actions preserve exact source occurrence links and never erase custody.
            for hit in &response.hits {
                for event in crate::utils::sync::recover_poison(hit.conversion.lock()).iter() {
                    assert_eq!(event.response_digest, hit.page.digest());
                    if let Some(index) = event.claim_index {
                        assert_eq!(
                            event.origin.as_ref(),
                            Some(hit.row.identity().claims()[index].origin())
                        );
                    }
                }
            }
        }
    }
}
