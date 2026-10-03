//! Independent accepted 0663 literals, transcribed before consumer changes.
use super::point_alias::{Compatibility, PointRoute, Preparation, point_assertion};
use super::*;
use crate::sources::myvariant::FloatOrVec;
use crate::sources::myvariant::{MyVariantHit, MyVariantSearchResponse};
use crate::utils::serde::StringOrVec;
use biodata::{
    HgvsProteinPointDisposition, HgvsProteinPointEdit, HgvsProteinResidue,
    parse_hgvs_protein_point_21_1_4,
};
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
        let assertion = point_assertion(source, None, false);
        assert_eq!(assertion.source, source);
        assert_eq!(assertion.gene, None);
        assert_eq!(
            assertion.alias().as_deref(),
            row["alias"].as_str(),
            "{}",
            row["id"]
        );
        match row["id"].as_str().unwrap() {
            "P13" => assert!(matches!(
                assertion.route,
                PointRoute::Compatibility(Compatibility::EqualSubstitution, _)
            )),
            "P14" => assert!(matches!(
                assertion.route,
                PointRoute::Compatibility(Compatibility::ZeroPosition, _)
            )),
            "P15" => assert!(matches!(
                assertion.route,
                PointRoute::Compatibility(Compatibility::InitiationChange, _)
            )),
            "P16" => assert!(matches!(
                assertion.route,
                PointRoute::Compatibility(Compatibility::TerminationReference, _)
            )),
            "P17" | "P18" | "P19" | "P20" | "P21" | "P22" | "P23" => {
                assert!(matches!(assertion.route, PointRoute::Refused))
            }
            id => {
                let PointRoute::Checked(envelope) = &assertion.route else {
                    panic!("{id} lacks the checked route");
                };
                assert!(matches!(
                    envelope.disposition(),
                    HgvsProteinPointDisposition::Parsed(_)
                ));
                let point = envelope.disposition().parsed().unwrap();
                let (candidate, reference, position, from, to, preparation) = match id {
                    "P01" => (
                        "p.V600E",
                        None,
                        "600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        Some(Preparation::BarePrefix),
                    ),
                    "P02" => (
                        "p.Val600Glu",
                        None,
                        "600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        None,
                    ),
                    "P03" => (
                        "NP_004324.2:p.Val600Glu",
                        Some("NP_004324.2"),
                        "600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        None,
                    ),
                    "P04" => (
                        "p.Val600Glu",
                        None,
                        "600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        Some(Preparation::LegacyResidueSpelling),
                    ),
                    "P05" => (
                        "p.V600E",
                        None,
                        "600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        Some(Preparation::UppercasePrefix),
                    ),
                    "P06" => (
                        "p.V600E",
                        None,
                        "600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        Some(Preparation::OuterWhitespace),
                    ),
                    "P07" => (
                        "p.V00600E",
                        None,
                        "00600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        Some(Preparation::BarePrefix),
                    ),
                    "P08" => (
                        "p.V4294967296E",
                        None,
                        "4294967296",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        Some(Preparation::BarePrefix),
                    ),
                    "P09" => (
                        "p.L39*",
                        None,
                        "39",
                        HgvsProteinResidue::Leu,
                        HgvsProteinResidue::Ter,
                        Some(Preparation::BarePrefix),
                    ),
                    "P10" => (
                        "p.Leu39Ter",
                        None,
                        "39",
                        HgvsProteinResidue::Leu,
                        HgvsProteinResidue::Ter,
                        None,
                    ),
                    "P11" => (
                        "NP_000001.1:p.Leu39Ter",
                        Some("NP_000001.1"),
                        "39",
                        HgvsProteinResidue::Leu,
                        HgvsProteinResidue::Ter,
                        Some(Preparation::LegacyResidueSpelling),
                    ),
                    "P12" => (
                        "p.V600Ter",
                        None,
                        "600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Ter,
                        Some(Preparation::LegacyResidueSpelling),
                    ),
                    "P25" | "P26" => (
                        "p.V600E",
                        None,
                        "600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        Some(Preparation::LegacyResidueTokenTrim),
                    ),
                    "P27" => (
                        "p.V600E",
                        None,
                        "600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        Some(Preparation::LegacyEmptyReferencePrefix),
                    ),
                    "P28" => (
                        "p.Val600Glu",
                        None,
                        "600",
                        HgvsProteinResidue::Val,
                        HgvsProteinResidue::Glu,
                        Some(Preparation::LegacyUncheckedReferencePrefix),
                    ),
                    _ => panic!("unregistered accepted row"),
                };
                assert_eq!(envelope.source(), candidate, "{id}");
                assert_eq!(point.reference(), reference, "{id}");
                assert_eq!(point.position(), position, "{id}");
                assert_eq!(point.reference_residue(), from, "{id}");
                assert_eq!(
                    point.edit(),
                    &HgvsProteinPointEdit::Substitution { alternate: to },
                    "{id}"
                );
                assert!(!point.is_predicted());
                if let Some(preparation) = preparation {
                    assert!(assertion.preparations.contains(&preparation), "{id}");
                }
                if id == "P27" {
                    assert_eq!(assertion.reference_prefix, Some(""));
                }
                if id == "P28" {
                    assert_eq!(assertion.reference_prefix, Some("bad ref"));
                }
            }
        }
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
        let left_assertion = point_assertion(left, None, false);
        let right_assertion = point_assertion(right, None, false);
        assert_eq!(left_assertion.source, left);
        assert_eq!(right_assertion.source, right);
        if left == "NP_004324.2:p.Val600Glu" {
            let PointRoute::Checked(left_envelope) = &left_assertion.route else {
                panic!("left reference unavailable");
            };
            let PointRoute::Checked(right_envelope) = &right_assertion.route else {
                panic!("right reference unavailable");
            };
            assert_eq!(
                left_envelope.disposition().parsed().unwrap().reference(),
                Some("NP_004324.2")
            );
            assert_eq!(
                right_envelope.disposition().parsed().unwrap().reference(),
                Some("NP_004324.3")
            );
        }
        if left == "p.Glu746_Ala750del" {
            assert!(matches!(left_assertion.route, PointRoute::Refused));
            assert!(matches!(right_assertion.route, PointRoute::Refused));
        }
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
    let assertion = point_assertion(&source, None, false);
    assert_eq!(assertion.source, source);
    assert!(matches!(
        assertion.route,
        PointRoute::Compatibility(Compatibility::OverLimit, _)
    ));
    assert_eq!(assertion.alias().as_deref(), Some(source.as_str()));
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
        let checked = identity.protein_changes.iter().map(|source| {
            let assertion = point_assertion(source, None, false);
            let PointRoute::Checked(envelope) = &assertion.route else { panic!("missing source point"); };
            let point = envelope.disposition().parsed().unwrap();
            assert_eq!(point.reference_residue(), HgvsProteinResidue::Val);
            assert_eq!(point.edit(), &HgvsProteinPointEdit::Substitution { alternate: HgvsProteinResidue::Glu });
            json!({"source":assertion.source,"reference":point.reference(),"prediction":point.is_predicted(),
                "reference_residue":"Val","position":point.position(),"edit":{"Substitution":"Glu"},"alias":assertion.alias()})
        }).collect::<Vec<_>>();
        assert_eq!(json!(checked), oracle("POINT_ASSERTIONS.json"));
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

#[test]
fn checked_refusals_and_internal_debug_are_terminal_and_private() {
    for source in [
        "p.V600=",
        "p.(Val600Glu)",
        "p.V600Eextra",
        "p.Met1Val",
        "private-input:p.?",
        "bad ref:p.V600E",
    ] {
        let assertion = point_alias::PointAssertion {
            source,
            gene: Some("private-gene"),
            reference_prefix: Some("private-prefix"),
            preparations: Vec::new(),
            route: PointRoute::Checked(parse_hgvs_protein_point_21_1_4(source)),
        };
        assert_eq!(assertion.alias(), None);
        let debug = format!("{assertion:?}");
        for text in [source, "private-gene", "private-prefix"] {
            assert!(!debug.contains(text));
        }
        if let PointRoute::Checked(envelope) = &assertion.route {
            if let Some(diagnostic) = envelope.disposition().diagnostic() {
                assert!(!format!("{diagnostic:?}").contains(source));
            }
        }
    }
    // Shared reference refusal must never open unchecked-prefix preparation after selection.
    let source = format!("{}:p.V600E", "R".repeat(4097));
    let envelope = parse_hgvs_protein_point_21_1_4(&source);
    assert!(matches!(
        envelope.disposition(),
        HgvsProteinPointDisposition::ResourceLimitExceeded(_)
    ));
    let assertion = point_alias::PointAssertion {
        source: &source,
        gene: None,
        reference_prefix: None,
        preparations: Vec::new(),
        route: PointRoute::Checked(envelope),
    };
    assert_eq!(assertion.alias(), None);
    // Preselected internal over-limit compatibility remains distinct from shared refusal.
    assert!(matches!(
        point_assertion(&source, None, false).route,
        PointRoute::Compatibility(Compatibility::OverLimit, _)
    ));
    for (source, class) in [
        ("B600E", Compatibility::DirectRegex),
        ("V600X", Compatibility::DirectRegex),
        ("V600V", Compatibility::EqualSubstitution),
    ] {
        let assertion = point_assertion(source, Some("BRAF"), true);
        assert_eq!(assertion.gene, Some("BRAF"));
        assert_eq!(assertion.source, source);
        assert_eq!(assertion.alias().as_deref(), Some(source));
        assert!(matches!(assertion.route, PointRoute::Compatibility(actual, _) if actual == class));
    }
    let ordinary = point_assertion("V600E", Some("BRAF"), true);
    assert_eq!(ordinary.gene, Some("BRAF"));
    assert!(matches!(ordinary.route, PointRoute::Checked(_)));
    for source in ["V6 00E", "V600 Eextra", "bad ref:p.V600Eextra"] {
        assert_eq!(normalize_protein_change(source), None);
    }
}
