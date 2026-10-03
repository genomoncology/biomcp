//! Independent accepted 0663 literals, transcribed before consumer changes.
use super::*;
use crate::sources::myvariant::FloatOrVec;
use crate::sources::myvariant::{MyVariantHit, MyVariantSearchResponse};
use crate::utils::serde::StringOrVec;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[path = "point_transport_tests.rs"]
mod transport;

fn oracle(name: &str) -> Value {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    serde_json::from_slice(
        &std::fs::read(
            root.join("src/entities/variant/resolution/point_oracles")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn accepted_point_alias_and_comparison_table() {
    let rows = oracle("engineering.json");
    assert_eq!(rows.as_array().unwrap().len(), 27);
    for row in rows.as_array().unwrap() {
        let source = row["source"].as_str().unwrap();
        assert_eq!(
            normalize_protein_change(source).as_deref(),
            row["alias"].as_str(),
            "{}",
            row["id"]
        );
    }
    for (left, right, matches) in [
        ("p.Val600Glu", "p.V600E", true),
        ("V600E", "V601E", false),
        ("V600E", "V600K", false),
        ("V00600E", "V600E", false),
        ("NP_004324.2:p.Val600Glu", "NP_004324.3:p.Val600Glu", true),
        ("p.Glu746_Ala750del", "NP_005219.2:p.Glu746_Ala750del", true),
    ] {
        assert_eq!(protein_changes_equivalent(left, right), matches);
    }
    for change in ["B600E", "V600V"] {
        assert_eq!(
            classify_variant_input(&format!("BRAF {change}")),
            VariantInputKind::Exact(VariantIdFormat::GeneProteinChange {
                gene: "BRAF".into(),
                change: change.into()
            })
        );
    }
}

#[test]
fn accepted_point_prefix_limit_keeps_internal_compatibility() {
    // P24 is materialized only here. It supplies no public admission claim.
    let source = format!("V{}E", "1".repeat(1_048_574));
    assert_eq!(
        normalize_protein_change(&source).as_deref(),
        Some(source.as_str())
    );
}

#[test]
fn accepted_original_byte_source_and_product_table() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for (case, file, bytes, digest) in [
        (
            "A01",
            "search_braf_v600e_20260806.json",
            416,
            "9627c5551d7b9e4435e3a8c3db32509a4f64fa860451407d4cf37c71e3caa469",
        ),
        (
            "A02",
            "get_braf_v600e_20260805.json",
            334,
            "bba12795c6fbb52f9ddd04540de1bb1971532a9fa59d06be34b983ec18ddf6ca",
        ),
    ] {
        let raw = std::fs::read(root.join("testdata/sources/myvariant").join(file)).unwrap();
        assert_eq!(raw.len(), bytes);
        assert_eq!(format!("{:x}", Sha256::digest(&raw)), digest);
        let hit: MyVariantHit = if case == "A01" {
            let page: MyVariantSearchResponse = serde_json::from_slice(&raw).unwrap();
            assert_eq!(page.total, Some(1));
            assert_eq!(page.hits.len(), 1);
            page.hits.into_iter().next().unwrap()
        } else {
            serde_json::from_slice(&raw).unwrap()
        };
        assert_eq!(
            serde_json::to_value(&hit).unwrap(),
            oracle(&format!("{case}_HIT.json"))
        );
        let db = hit.dbnsfp.as_ref().unwrap();
        assert!(
            matches!(&db.genename, StringOrVec::Multiple(v) if v == &["BRAF", "BRAF", "BRAF", "BRAF"])
        );
        assert!(
            matches!(&db.hgvsp, StringOrVec::Multiple(v) if v == &["p.Val640Glu", "p.Val600Glu", "p.Val207Glu", "p.V600E"])
        );
        assert!(matches!(db.hgvsc, StringOrVec::None));
        assert!(
            matches!(&hit.cadd.as_ref().unwrap().consequence, Some(StringOrVec::Single(v)) if v == "NON_SYNONYMOUS")
        );
        if let Some(score) = &db.revel {
            assert!(matches!(score.score, Some(FloatOrVec::Single(v)) if v == 0.931));
        }
        let scores = db.bayesdel.as_ref().unwrap();
        assert!(
            matches!(scores.add_af.as_ref().unwrap().score, Some(FloatOrVec::Single(v)) if v == 0.399079)
        );
        assert!(
            matches!(scores.no_af.as_ref().unwrap().score, Some(FloatOrVec::Single(v)) if v == 0.335473)
        );
        let identity = SourceVariantIdentity::from_myvariant_hit(&hit);
        assert_eq!(
            serde_json::to_value(&identity).unwrap(),
            oracle("SOURCE_IDENTITY.json")
        );
        let requested: RequestedVariantIdentity =
            serde_json::from_value(json!({"gene":"BRAF","protein_change":"V600E"})).unwrap();
        assert_eq!(
            compare_variant_identity(&requested, &identity),
            VariantIdentityComparison::Compatible {
                matched_alias: "p.Val600Glu".into()
            }
        );
        assert_eq!(
            identity
                .protein_changes
                .iter()
                .map(|v| protein_changes_equivalent(v, "V600E"))
                .collect::<Vec<_>>(),
            [false, true, false, true]
        );
        let variant = crate::transform::variant::from_myvariant_hit(&hit);
        assert_eq!(
            serde_json::to_value(&variant).unwrap(),
            oracle(&format!("{case}_VARIANT.json"))
        );
        let search = crate::transform::variant::from_myvariant_search_hit(&hit);
        assert_eq!(
            serde_json::to_value(&search).unwrap(),
            oracle(&format!("{case}_SEARCH_ROW.json"))
        );
    }
}
