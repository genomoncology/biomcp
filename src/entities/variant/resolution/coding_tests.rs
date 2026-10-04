//! Accepted 0668 MIT synthetic literals; captured rows retain original receipt limits.
use super::*;
use crate::sources::myvariant::{MyVariantHit, MyVariantSearchResponse};
use crate::utils::serde::StringOrVec;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[path = "coding_transport_tests.rs"]
mod transport;

fn oracle(name: &str) -> Value {
    read_json(&format!(
        "src/entities/variant/resolution/coding_oracles/{name}.json"
    ))
}

fn read_json(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root().join(name)).unwrap()).unwrap()
}

fn root() -> std::path::PathBuf {
    env!("CARGO_MANIFEST_DIR").into()
}

fn projection(source: &str) -> Value {
    // Red scaffold observes the unchanged consumer's unqualified lexical key.
    let segment = coding_change_segment(source);
    let trimmed = source.trim();
    let reference = trimmed
        .rsplit_once(':')
        .filter(|(_, c)| c.eq_ignore_ascii_case(segment))
        .map(|(p, _)| p);
    json!({"source":source,"segment":segment,"reference_prefix":reference,
        "preparations":if trimmed==source {vec![]} else {vec!["outer_whitespace"]},
        "route":"compatibility_body","diagnostic":null,
        "comparison_key":segment.to_ascii_uppercase(),"checked":null})
}

fn resource(row: &Value) -> (String, Value) {
    let (source, digits, reference) = match row["id"].as_str().unwrap() {
        "R01" => {
            let d = "1".repeat(1_048_571);
            (format!("c.{d}C>T"), d, None)
        }
        "R02" => {
            let d = "1".repeat(1_048_572);
            (format!("c.{d}C>T"), d, None)
        }
        "R03" => {
            let r = "N".repeat(4096);
            (format!("{r}:c.(19C>T)"), "19".into(), Some(r))
        }
        "R04" => {
            let r = "N".repeat(4097);
            (format!("{r}:c.(19C>T)"), "19".into(), Some(r))
        }
        _ => panic!("unregistered resource recipe"),
    };
    assert_eq!(source.len(), row["bytes"].as_u64().unwrap() as usize);
    let segment = if reference.is_some() {
        "c.(19C>T)"
    } else {
        &source
    };
    let mut expected = row["expected_projection_recipe"].clone();
    substitute(
        &mut expected,
        &json!({"$input":source,"$segment":segment,"$digits":digits,
        "$reference":reference,"$uppercase_segment":segment.to_ascii_uppercase()}),
    );
    (source, expected)
}

fn substitute(value: &mut Value, replacements: &Value) {
    match value {
        Value::String(s) if s.starts_with('$') => *value = replacements[s.as_str()].clone(),
        Value::Array(items) => items.iter_mut().for_each(|v| substitute(v, replacements)),
        Value::Object(items) => items.values_mut().for_each(|v| substitute(v, replacements)),
        _ => {}
    }
}

#[test]
fn coding_assertion_resources_and_privacy_table() {
    let rows = oracle("aliases");
    assert_eq!(rows["aliases"].as_array().unwrap().len(), 25);
    assert_eq!(rows["resources"].as_array().unwrap().len(), 4);
    let mut failures = Vec::new();
    for row in rows["aliases"].as_array().unwrap() {
        let source = row["input"]["source"].as_str().unwrap();
        if projection(source) != row["expected"] {
            failures.push(row["id"].as_str().unwrap());
        }
    }
    for row in rows["resources"].as_array().unwrap() {
        let (source, expected) = resource(row);
        if projection(&source) != expected {
            failures.push(row["id"].as_str().unwrap());
        }
    }
    assert!(
        failures.is_empty(),
        "complete alias/resource projection failures: {failures:?}"
    );
}

fn comparison(value: VariantIdentityComparison) -> Value {
    match value {
        VariantIdentityComparison::Compatible { matched_alias } => {
            json!({"Compatible":{"matched_alias":matched_alias}})
        }
        VariantIdentityComparison::Contradictory { field } => {
            json!({"Contradictory":{"field":field}})
        }
        VariantIdentityComparison::Indeterminate { field } => {
            json!({"Indeterminate":{"field":field}})
        }
    }
}

#[test]
fn coding_identity_comparison_table() {
    let rows = oracle("comparisons");
    assert_eq!(rows["cases"].as_array().unwrap().len(), 17);
    for row in rows["cases"].as_array().unwrap() {
        let request = serde_json::from_value(row["request"].clone()).unwrap();
        let source = serde_json::from_value(row["source"].clone()).unwrap();
        assert_eq!(
            comparison(compare_variant_identity(&request, &source)),
            row["expected"],
            "{}",
            row["id"]
        );
    }
}

fn source_internal(hit: &MyVariantHit) -> Value {
    let db = hit.dbnsfp.as_ref().unwrap();
    let shape = |v: &StringOrVec| match v {
        StringOrVec::Single(_) => "StringOrVec::Single",
        StringOrVec::Multiple(_) => "StringOrVec::Multiple",
        StringOrVec::None => "StringOrVec::None",
    };
    json!({"genename_variant":shape(&db.genename),"hgvsc_variant":shape(&db.hgvsc),
        "hgvsp_variant":shape(&db.hgvsp),"snpeff_complete":hit.snpeff.as_ref().unwrap().complete})
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn coding_source_projection_table() {
    let rows = oracle("source");
    assert_eq!(rows.as_object().unwrap().len(), 4);
    for id in ["S01", "S02"] {
        let row = &rows[id];
        let raw = serde_json::to_vec(&row["input"]).unwrap();
        let hit: MyVariantHit = if id == "S01" {
            let page: MyVariantSearchResponse = serde_json::from_slice(&raw).unwrap();
            assert_eq!(
                serde_json::to_value(&page).unwrap(),
                row["expected_decoded"]
            );
            assert_eq!(page.total, Some(1));
            assert_eq!(page.hits.len(), 1);
            page.hits.into_iter().next().unwrap()
        } else {
            let hit: MyVariantHit = serde_json::from_slice(&raw).unwrap();
            assert_eq!(serde_json::to_value(&hit).unwrap(), row["expected_decoded"]);
            assert_eq!(source_internal(&hit), row["expected_internal"]);
            hit
        };
        let identity = SourceVariantIdentity::from_myvariant_hit(&hit);
        assert_eq!(
            serde_json::to_value(&identity).unwrap(),
            row["expected_identity"]
        );
        if id == "S01" {
            let observed = transport::search_row(
                row["input"].clone(),
                json!({"gene":"GENE","coding_change":"c.19C>T"}),
            )
            .await;
            assert_eq!(observed, row["expected_exact_search_row"]);
            let aliases = oracle("aliases");
            assert_eq!(
                projection(&identity.coding_changes[0]),
                aliases["aliases"][0]["expected"]
            );
        } else {
            assert_eq!(identity.normalized_key(), row["expected_normalized_key"]);
            let request =
                serde_json::from_value(json!({"gene":"GENE","coding_change":"c.19C>T"})).unwrap();
            assert_eq!(
                comparison(compare_variant_identity(&request, &identity)),
                row["expected_comparison"]
            );
            let routes: Vec<_> = identity
                .coding_changes
                .iter()
                .map(|s| projection(s)["route"].clone())
                .collect();
            assert_eq!(json!(routes), row["expected_alias_routes"]);
        }
    }
    for (id, digest) in [
        (
            "S03",
            "9627c5551d7b9e4435e3a8c3db32509a4f64fa860451407d4cf37c71e3caa469",
        ),
        (
            "S04",
            "bba12795c6fbb52f9ddd04540de1bb1971532a9fa59d06be34b983ec18ddf6ca",
        ),
    ] {
        let row = &rows[id];
        let raw = std::fs::read(root().join(row["input_file"].as_str().unwrap())).unwrap();
        assert_eq!(raw.len(), if id == "S03" { 416 } else { 334 });
        assert_eq!(format!("{:x}", Sha256::digest(&raw)), digest);
        let hit: MyVariantHit = if id == "S03" {
            let page: MyVariantSearchResponse = serde_json::from_slice(&raw).unwrap();
            assert_eq!(page.total, Some(1));
            assert_eq!(page.hits.len(), 1);
            page.hits.into_iter().next().unwrap()
        } else {
            serde_json::from_slice(&raw).unwrap()
        };
        let point = row["expected_oracle_root"].as_str().unwrap();
        assert_eq!(
            serde_json::to_value(&hit).unwrap(),
            read_json(&format!(
                "{point}/{}",
                row["expected_hit_file"].as_str().unwrap()
            ))
        );
        let identity = SourceVariantIdentity::from_myvariant_hit(&hit);
        assert_eq!(
            serde_json::to_value(&identity).unwrap(),
            read_json(&format!(
                "{point}/{}",
                row["expected_identity_file"].as_str().unwrap()
            ))
        );
        assert_eq!(
            json!(identity.coding_changes),
            row["expected_coding_aliases"]
        );
        assert!(matches!(
            hit.dbnsfp.as_ref().unwrap().hgvsc,
            StringOrVec::None
        ));
        assert_eq!(
            serde_json::to_value(crate::transform::variant::from_myvariant_search_hit(&hit))
                .unwrap(),
            read_json(&format!(
                "{point}/{}",
                row["expected_projection_file"].as_str().unwrap()
            ))
        );
    }
    let aliases = oracle("aliases");
    let control = &aliases["aliases"][24]["retained_consumers"];
    let key: SourceVariantIdentity =
        serde_json::from_value(control["normalized_key"]["input"].clone()).unwrap();
    assert_eq!(key.normalized_key(), control["normalized_key"]["expected"]);
    let match_case = &control["annotation_match"];
    let annotation = &match_case["annotation"];
    let page = json!({"total":1,"hits":[{"_id":"chr1:g.19C>T","dbnsfp":{"genename":"GENE","hgvsc":annotation["hgvs_c"]},"snpeff":{"ann":annotation}}]});
    let observed = transport::search_row(page, match_case["request"].clone()).await;
    let actual = &observed["transcript_annotations"][0];
    assert_eq!(
        json!({"feature_id":actual["transcript"],"genename":actual["gene"],"hgvs_c":actual["hgvs_c"],"hgvs_p":actual["hgvs_p"]}),
        *annotation
    );
    assert_eq!(
        json!(
            actual["roles"]
                .as_array()
                .unwrap()
                .contains(&json!("matched"))
        ),
        match_case["expected"]
    );
}
