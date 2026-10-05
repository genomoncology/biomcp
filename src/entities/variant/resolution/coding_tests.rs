//! Accepted 0668 MIT synthetic literals; captured rows retain original receipt limits.
use super::coding_alias::{CodingRoute, coding_assertion};
use super::*;
use crate::sources::myvariant::{MyVariantHit, MyVariantSearchResponse};
use crate::utils::serde::StringOrVec;
use biodata::{HgvsEdit, HgvsLocation, HgvsMolecule, HgvsPosition, HgvsSpan, ParsedHgvsNucleotide};
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

fn span(value: HgvsSpan) -> [usize; 2] {
    [value.start(), value.end()]
}

fn projection(source: &str) -> Value {
    let assertion = coding_assertion(source);
    let checked = match &assertion.route {
        CodingRoute::Checked(envelope) => {
            let parsed = envelope.disposition().parsed().unwrap();
            let position = |p: &HgvsPosition| {
                json!({"marker":format!("{:?}",p.marker()),
                "digits":p.digits(),"offset":p.offset().map(|o|json!({"positive":o.is_positive(),"digits":o.digits()}))})
            };
            let location = match parsed.location().unwrap() {
                HgvsLocation::Point(p) => json!({"Point":position(p)}),
                HgvsLocation::Range(a, b) => json!({"Range":[position(a),position(b)]}),
                HgvsLocation::InsertionFlanks(a, b) => {
                    json!({"InsertionFlanks":[position(a),position(b)]})
                }
                HgvsLocation::UncertainBreakpoint(a, b) => {
                    json!({"UncertainBreakpoint":[position(a),position(b)]})
                }
                HgvsLocation::UncertainRange((a, b), (c, d)) => {
                    json!({"UncertainRange":[[position(a),position(b)],[position(c),position(d)]]})
                }
            };
            let edit = match parsed.edit().unwrap() {
                HgvsEdit::Substitution {
                    reference,
                    alternate,
                } => json!({"Substitution":{"reference":reference,"alternate":alternate}}),
                HgvsEdit::Deletion { deleted } => json!({"Deletion":{"deleted":deleted}}),
                HgvsEdit::Duplication { duplicated } => {
                    json!({"Duplication":{"duplicated":duplicated}})
                }
                HgvsEdit::Insertion { inserted } => json!({"Insertion":{"inserted":inserted}}),
                HgvsEdit::Delins { inserted } => json!({"Delins":{"inserted":inserted}}),
                HgvsEdit::Inversion => json!("Inversion"),
                HgvsEdit::NoChange => json!("NoChange"),
            };
            assert_eq!(parsed.span().slice(envelope.source()), Some(source.trim()));
            let displacement = source.len() - source.trim_start().len();
            for range in [
                Some(parsed.span()),
                parsed.reference_span(),
                parsed.location_span(),
            ]
            .into_iter()
            .flatten()
            {
                assert_eq!(
                    range.slice(envelope.source()),
                    source.get(displacement + range.start()..displacement + range.end())
                );
            }
            json!({"envelope_source":envelope.source(),"molecule":format!("{:?}",parsed.molecule()),
                "reference":parsed.reference(),"prediction":parsed.is_predicted(),
                "location":location,"edit":edit,
                "rna_outcome":parsed.rna_outcome().map(|v|format!("{v:?}")),
                "rna_basis":parsed.rna_basis().map(|v|format!("{v:?}")),
                "reference_unavailable":parsed.reference_unavailable(),"span":span(parsed.span()),
                "reference_span":parsed.reference_span().map(span),"location_span":parsed.location_span().map(span),
                "source_render":envelope.render_source(),"constructed_render":parsed.render_constructed()})
        }
        _ => Value::Null,
    };
    json!({"source":assertion.source,"segment":assertion.segment,"reference_prefix":assertion.reference_prefix,
        "preparations":if assertion.outer_whitespace {vec!["outer_whitespace"]} else {vec![]},
        "route":assertion.route_name(),"diagnostic":assertion.diagnostic(),
        "comparison_key":assertion.key(),"checked":checked})
}

fn privacy(source: &str) {
    let assertion = coding_assertion(source);
    let debug = format!("{assertion:?}");
    for value in [
        Some(source),
        assertion.reference_prefix,
        assertion.key().as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if !value.trim().is_empty() {
            assert!(!debug.contains(value), "input-derived Debug field");
        }
    }
}

fn constructor_controls(row: &Value) {
    let expected = &row["expected"]["checked"];
    let reference = expected["reference"].as_str();
    let predicted = expected["prediction"].as_bool().unwrap();
    let position = |p: &Value| {
        let marker = match p["marker"].as_str().unwrap() {
            "Ordinary" => "",
            "Upstream" => "-",
            "CodingEnd" => "*",
            "Unknown" => "?",
            _ => panic!("unknown gold position marker"),
        };
        let offset = &p["offset"];
        let written = format!(
            "{marker}{}{}{}",
            p["digits"].as_str().unwrap_or(""),
            if offset.is_null() {
                ""
            } else if offset["positive"] == true {
                "+"
            } else {
                "-"
            },
            offset["digits"].as_str().unwrap_or("")
        );
        HgvsPosition::new(&written, HgvsMolecule::Coding).unwrap()
    };
    let (kind, value) = expected["location"]
        .as_object()
        .unwrap()
        .iter()
        .next()
        .unwrap();
    let location = match kind.as_str() {
        "Point" => HgvsLocation::Point(position(value)),
        "Range" => HgvsLocation::Range(position(&value[0]), position(&value[1])),
        "InsertionFlanks" => {
            HgvsLocation::InsertionFlanks(position(&value[0]), position(&value[1]))
        }
        "UncertainBreakpoint" => {
            HgvsLocation::UncertainBreakpoint(position(&value[0]), position(&value[1]))
        }
        "UncertainRange" => HgvsLocation::UncertainRange(
            (position(&value[0][0]), position(&value[0][1])),
            (position(&value[1][0]), position(&value[1][1])),
        ),
        _ => panic!("unknown gold location"),
    };
    let value = &expected["edit"];
    let edit = if let Some(kind) = value.as_str() {
        match kind {
            "Inversion" => HgvsEdit::Inversion,
            "NoChange" => HgvsEdit::NoChange,
            _ => panic!("unknown gold edit"),
        }
    } else {
        let (kind, value) = value.as_object().unwrap().iter().next().unwrap();
        match kind.as_str() {
            "Substitution" => HgvsEdit::Substitution {
                reference: value["reference"].as_str().unwrap().chars().next().unwrap(),
                alternate: value["alternate"].as_str().unwrap().chars().next().unwrap(),
            },
            "Deletion" => HgvsEdit::Deletion {
                deleted: value["deleted"].as_str().map(str::to_owned),
            },
            "Duplication" => HgvsEdit::Duplication {
                duplicated: value["duplicated"].as_str().map(str::to_owned),
            },
            "Insertion" => HgvsEdit::Insertion {
                inserted: value["inserted"].as_str().unwrap().into(),
            },
            "Delins" => HgvsEdit::Delins {
                inserted: value["inserted"].as_str().unwrap().into(),
            },
            _ => panic!("unknown gold edit"),
        }
    };
    let independent = ParsedHgvsNucleotide::construct(
        reference,
        HgvsMolecule::Coding,
        location.clone(),
        edit.clone(),
        predicted,
    )
    .unwrap();
    let assertion = coding_assertion(row["input"]["source"].as_str().unwrap());
    let CodingRoute::Checked(envelope) = &assertion.route else {
        panic!("constructor control lacks checked assertion");
    };
    assert_eq!(
        independent.render_constructed(),
        expected["constructed_render"]
    );
    assert_eq!(
        envelope.render_source_for(&independent),
        expected["source_render"].as_str()
    );
    let (reference, location, predicted) = match row["id"].as_str().unwrap() {
        "A03" => (reference, location, !predicted),
        "A05" => (
            reference,
            HgvsLocation::Point(HgvsPosition::new("19", HgvsMolecule::Coding).unwrap()),
            predicted,
        ),
        _ => (Some("OTHER"), location, predicted),
    };
    let changed =
        ParsedHgvsNucleotide::construct(reference, HgvsMolecule::Coding, location, edit, predicted)
            .unwrap();
    assert_eq!(envelope.render_source_for(&changed), None);
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
    assert_eq!(rows["aliases"].as_array().unwrap().len(), 205);
    assert_eq!(rows["resources"].as_array().unwrap().len(), 4);
    let mut failures = Vec::new();
    for row in rows["aliases"].as_array().unwrap() {
        let source = row["input"]["source"].as_str().unwrap();
        privacy(source);
        if !row["expected"]["checked"].is_null() {
            constructor_controls(row);
        }
        if row["expected_parser_calls"] == 0 {
            assert!(!matches!(
                coding_assertion(source).route,
                CodingRoute::Checked(_)
            ));
        }
        if projection(source) != row["expected"] {
            failures.push(row["id"].as_str().unwrap());
        }
    }
    for row in rows["resources"].as_array().unwrap() {
        let (source, expected) = resource(row);
        privacy(&source);
        if row["expected_route"] != "checked" {
            assert!(!matches!(
                coding_assertion(&source).route,
                CodingRoute::Checked(_)
            ));
        }
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
    assert_eq!(rows["cases"].as_array().unwrap().len(), 173);
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
    let aliases = oracle("aliases");
    assert_eq!(rows.as_object().unwrap().len(), 56);
    for (id, row) in rows
        .as_object()
        .unwrap()
        .iter()
        .filter(|(id, _)| !matches!(id.as_str(), "S03" | "S04"))
    {
        let raw = serde_json::to_vec(&row["input"]).unwrap();
        let hit: MyVariantHit = if id != "S02" {
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
            hit
        };
        assert_eq!(source_internal(&hit), row["expected_internal"]);
        let identity = SourceVariantIdentity::from_myvariant_hit(&hit);
        assert_eq!(
            serde_json::to_value(&identity).unwrap(),
            row["expected_identity"]
        );
        if id != "S02" {
            let observed = transport::search_row(
                row["input"].clone(),
                row.get("request")
                    .cloned()
                    .unwrap_or_else(|| json!({"gene":"GENE","coding_change":"c.19C>T"})),
            )
            .await;
            assert_eq!(observed, row["expected_exact_search_row"]);
            assert_eq!(
                identity.coding_changes.len(),
                row["expected_aliases"].as_array().unwrap().len()
            );
            for (source, alias_id) in identity
                .coding_changes
                .iter()
                .zip(row["expected_aliases"].as_array().unwrap())
            {
                let alias = aliases["aliases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|alias| alias["id"] == *alias_id)
                    .unwrap();
                assert_eq!(projection(source), alias["expected"], "{id}");
            }
        } else {
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
    let control = &aliases["aliases"][24]["retained_consumers"];
    assert_eq!(aliases["source_keys"].as_array().unwrap().len(), 56);
    for row in aliases["source_keys"].as_array().unwrap() {
        let key: SourceVariantIdentity = serde_json::from_value(row["input"].clone()).unwrap();
        assert_eq!(key.normalized_key(), row["expected"], "{}", row["id"]);
        assert_eq!(
            json!(key.coding_changes),
            row["expected_source_coding_changes_unchanged"]
        );
    }
    let match_case = &control["annotation_match"];
    let annotation = &match_case["annotation"];
    // Reuse accepted S02 source aliases to satisfy the separate source-transcript filter.
    // The annotation control retains its complete original request and DEL-bearing annotation.
    let mut hit = rows["S02"]["input"].clone();
    hit["snpeff"]["ann"][0] = annotation.clone();
    let page = json!({"total":1,"hits":[hit]});
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
