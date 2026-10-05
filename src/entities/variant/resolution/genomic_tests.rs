//! Accepted 0670 MIT synthetic literals. They establish engineering compatibility only.
use super::genomic_assertion::{Admission, Components, GenomicAssertion, genomic_assertion};
use super::*;
use biodata::{
    HgvsEdit, HgvsLocation, HgvsMarker, HgvsMolecule, HgvsSpan, parse_hgvs_nucleotide_21_1_4,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn corpus() -> Value {
    serde_json::from_str(include_str!("genomic_oracles/cases.json")).unwrap()
}
fn component_view(c: Option<Components<'_>>) -> Value {
    c.map(|c| {
        json!({"accession":c.accession,"position_lexeme":c.position_lexeme,
        "reference":c.reference,"alternate":c.alternate})
    })
    .unwrap_or(Value::Null)
}
fn source_view(a: &GenomicAssertion<'_>) -> Value {
    let c = a
        .source_assertion()
        .map(|s| s.comparison_fields())
        .unwrap_or(super::genomic_assertion::LegacyComparisonFields {
            build: a.build,
            ..Default::default()
        });
    json!({"build":c.build,"accession":c.accession,"position":c.position,
        "reference":c.reference,"alternate":c.alternate})
}
fn span(s: HgvsSpan) -> [usize; 2] {
    [s.start(), s.end()]
}
fn complete_facts(p: &biodata::ParsedHgvsNucleotide) -> Value {
    let location = p.location().unwrap();
    let kind = match location {
        HgvsLocation::Point(_) => "Point",
        HgvsLocation::Range(..) => "Range",
        HgvsLocation::InsertionFlanks(..) => "InsertionFlanks",
        HgvsLocation::UncertainBreakpoint(..) => "UncertainBreakpoint",
        HgvsLocation::UncertainRange(..) => "UncertainRange",
    };
    let positions: Vec<_> = location
        .positions()
        .iter()
        .map(|p| {
            json!({"marker": match p.marker() { HgvsMarker::Unknown => "Unknown",
            HgvsMarker::Ordinary => "Ordinary", _ => panic!("unexpected marker") },
            "digits":p.digits(),"offset":p.offset().map(|o| json!({
                "positive":o.is_positive(),"digits":o.digits()}))})
        })
        .collect();
    let edit = match p.edit().unwrap() {
        HgvsEdit::Substitution {
            reference,
            alternate,
        } => {
            json!({"kind":"Substitution","reference":reference.to_string(),"alternate":alternate.to_string()})
        }
        HgvsEdit::Deletion { deleted } => json!({"kind":"Deletion","deleted":deleted}),
        HgvsEdit::Duplication { duplicated } => {
            json!({"kind":"Duplication","duplicated":duplicated})
        }
        HgvsEdit::Insertion { inserted } => json!({"kind":"Insertion","inserted":inserted}),
        HgvsEdit::Delins { inserted } => json!({"kind":"Delins","inserted":inserted}),
        HgvsEdit::Inversion => json!({"kind":"Inversion"}),
        HgvsEdit::NoChange => json!({"kind":"NoChange"}),
    };
    assert_eq!(p.molecule(), HgvsMolecule::Genomic);
    json!({"molecule":"Genomic","reference":p.reference(),"prediction":p.is_predicted(),
        "location":{"kind":kind,"positions":positions},"edit":edit,
        "spans":{"whole":span(p.span()),"reference":span(p.reference_span().unwrap()),
            "location":span(p.location_span().unwrap())},"rendered":p.render_constructed()})
}
fn checked(a: &GenomicAssertion<'_>, row: &Value) {
    let e = a.envelope.as_ref().unwrap();
    let p = e.disposition().parsed().unwrap();
    assert_eq!(e.source(), a.candidate);
    assert_eq!(p.molecule(), HgvsMolecule::Genomic);
    assert_eq!(p.is_predicted(), row["checked_prediction"]);
    let Some(HgvsLocation::Point(position)) = p.location() else {
        panic!("point missing")
    };
    assert_eq!(position.marker(), HgvsMarker::Ordinary);
    assert!(position.offset().is_none());
    assert_eq!(position.digits(), row["checked_digits"].as_str());
    let c = a.components().unwrap();
    assert_eq!(p.reference(), Some(c.accession));
    let Some(HgvsEdit::Substitution {
        reference,
        alternate,
    }) = p.edit()
    else {
        panic!("edit missing")
    };
    assert_eq!(reference.to_string(), c.reference);
    assert_eq!(alternate.to_string(), c.alternate);
    assert_eq!(p.render_constructed(), a.candidate);
    assert_eq!(&a.candidate[p.span().start()..p.span().end()], a.candidate);
    let r = p.reference_span().unwrap();
    let l = p.location_span().unwrap();
    assert_eq!(&a.candidate[r.start()..r.end()], c.accession);
    assert_eq!(&a.candidate[l.start()..l.end()], c.position_lexeme);
    assert_eq!(
        &a.source[a.candidate_offset_bytes..a.candidate_offset_bytes + a.candidate.len()],
        a.candidate
    );
    if !row["checked_spans"].is_null() {
        assert_eq!(
            json!({"whole":span(p.span()),"reference":span(r),"location":span(l)}),
            row["checked_spans"]
        );
    }
}
fn parser_calls(a: &GenomicAssertion<'_>) -> usize {
    usize::from(a.envelope.is_some())
}
fn recipe(prefix: &str, bytes: usize) -> String {
    format!("{prefix}{}1A>T", "0".repeat(bytes - prefix.len() - 4))
}
fn retained_transport_custody() {
    // The inherited coding table is the sole execution owner. Reference its entire
    // sealed oracle rather than derive or execute a second transport expectation.
    let raw = include_bytes!("coding_oracles/transports.json");
    assert_eq!(
        format!("{:x}", Sha256::digest(raw)),
        "f8bde31e7d94a4ea34b9636d235e8adcbc2339ba0145933d79d7a7ff35944412"
    );
    assert_eq!(corpus()["retained_transport_cases"], 6);
    assert_eq!(corpus()["complete_source_transport_cases"], 4);
    assert_eq!(
        corpus()["retained_transport_execution_owner"],
        "entities::variant::resolution::tests::coding::transport::coding_cli_and_mcp_table"
    );
}
#[test]
fn genomic_assertion_and_resource_table() {
    let all = corpus();
    for row in all["extractions"].as_array().unwrap() {
        let input = row["input"].as_str().unwrap();
        let source = genomic_assertion(input, Admission::Source, true);
        assert_eq!(source.route_name(), row["route"], "{}", row["id"]);
        assert_eq!(
            source_view(&source),
            row["source_components"],
            "{}",
            row["id"]
        );
        assert_eq!(
            source_view(&genomic_assertion(input, Admission::Source, true)),
            row["source_components"]
        );
        if !row["checked_facts"].is_null() {
            assert_eq!(
                complete_facts(
                    source
                        .envelope
                        .as_ref()
                        .unwrap()
                        .disposition()
                        .parsed()
                        .unwrap()
                ),
                row["checked_facts"]
            );
            assert!(source.components().is_none());
        } else if source.route_name() == "checked" {
            checked(&source, row);
        } else if let Some(code) = row["classification_code"].as_str() {
            assert_eq!(source.envelope.as_ref().unwrap().disposition().code(), code);
        } else {
            assert!(source.envelope.is_none());
        }
        for (admission, field) in [
            (Admission::Chromosome, "chromosome_components"),
            (Admission::Structured, "structured_components"),
        ] {
            let assertion = genomic_assertion(input, admission, true);
            assert_eq!(
                component_view(assertion.components()),
                row[field],
                "{} {field}",
                row["id"]
            );
        }
        let prediction = genomic_assertion(input, Admission::Chromosome, false);
        assert_eq!(
            component_view(prediction.components()),
            row["prediction_components"],
            "{}",
            row["id"]
        );
        assert_eq!(
            parser_calls(&prediction),
            row["prediction_parser_calls"].as_u64().unwrap() as usize
        );
        if let Some(calls) = row["source_parser_calls"].as_u64() {
            assert_eq!(parser_calls(&source), calls as usize);
            let e = parse_hgvs_nucleotide_21_1_4(
                row["syntax_only_observation"]["candidate"]
                    .as_str()
                    .unwrap(),
            );
            assert_eq!(
                e.disposition().code(),
                row["syntax_only_observation"]["code"]
            );
        }
    }
    for row in all["complete_source_cases"].as_array().unwrap() {
        let input = row["input"].as_str().unwrap();
        let a = genomic_assertion(input, Admission::Source, true);
        assert_eq!(a.route_name(), row["route"], "{}", row["id"]);
        assert_eq!(a.source, input);
        assert_eq!(a.candidate, row["candidate"]);
        assert_eq!(json!(a.build), row["build"]);
        assert_eq!(
            a.candidate_offset_bytes,
            row["offset"].as_u64().unwrap() as usize
        );
        assert_eq!(
            a.envelope.as_ref().unwrap().disposition().code(),
            row["code"]
        );
        let source = a.source_assertion().unwrap();
        let observed = match &source {
            super::genomic_assertion::SourceGenomicAssertion::Checked { parsed, .. } => {
                Some(complete_facts(parsed))
            }
            super::genomic_assertion::SourceGenomicAssertion::Compatibility {
                diagnostic, ..
            } => {
                assert_eq!(json!(diagnostic), row["code"]);
                None
            }
            _ => panic!("source unexpectedly absent"),
        };
        assert_eq!(json!(observed), row["facts"], "{}", row["id"]);
        assert_eq!(component_view(a.components()), row["point"]);
        assert_eq!(source_view(&a), row["legacy_fields"]);
        // The same complete borrowed facts survive consumer point refusal.
        if let super::genomic_assertion::SourceGenomicAssertion::Checked { parsed, .. } = source {
            assert_eq!(complete_facts(parsed), row["facts"]);
        }
        for admission in [Admission::Chromosome, Admission::Structured] {
            assert!(
                genomic_assertion(input, admission, true)
                    .components()
                    .is_none()
            );
        }
        let debug = format!("{a:?}");
        assert!(!debug.contains(input));
        assert!(!debug.contains(a.candidate));
    }
    for row in all["resource_recipes"].as_array().unwrap() {
        let input = if let Some(bytes) = row["candidate_bytes"].as_u64() {
            recipe("chr7:g.", bytes as usize)
        } else {
            format!(
                "{}:g.1A>T",
                "x".repeat(row["reference_bytes"].as_u64().unwrap() as usize)
            )
        };
        let a = genomic_assertion(&input, Admission::Source, true);
        assert_eq!(a.route_name(), row["expected_route"]);
        if row["expected_route"] == "checked" {
            assert_eq!(parser_calls(&a), 1);
            assert!(a.diagnostic().is_none());
        } else {
            assert_eq!(
                parser_calls(&a),
                row["parser_calls"].as_u64().unwrap() as usize
            );
        }
        if let Some(pos) = row["expected_position_u64"].as_u64() {
            assert_eq!(
                a.source_assertion().unwrap().comparison_fields().position,
                Some(pos)
            );
        }
    }
    for row in all["assertion_cases"].as_array().unwrap() {
        let input = row["input"].as_str().map(str::to_owned).unwrap_or_else(|| {
            recipe(
                "opaque:g.",
                row["input_recipe"]["candidate_bytes"].as_u64().unwrap() as usize,
            )
        });
        let envelope =
            parse_hgvs_nucleotide_21_1_4(row["envelope_input"].as_str().unwrap_or(&input));
        let disposition = match envelope.disposition() {
            biodata::HgvsDisposition::Parsed(_) => "Parsed",
            biodata::HgvsDisposition::Invalid(_) => "Invalid",
            biodata::HgvsDisposition::Unsupported(_) => "Unsupported",
            biodata::HgvsDisposition::ResourceLimitExceeded(_) => "ResourceLimitExceeded",
        };
        assert_eq!(disposition, row["producer_disposition"]);
        let a = GenomicAssertion::from_checked_envelope(&input, envelope);
        if !row["complete_facts"].is_null() {
            let super::genomic_assertion::SourceGenomicAssertion::Checked { parsed, .. } =
                a.source_assertion().unwrap()
            else {
                panic!("checked facts lost")
            };
            assert_eq!(complete_facts(parsed), row["complete_facts"]);
        }
        assert_eq!(a.route_name(), row["expected"]["route"]);
        assert_eq!(json!(a.diagnostic()), row["expected"]["diagnostic"]);
        if row["complete_facts"].is_null() && !row["expected"]["diagnostic"].is_null() {
            assert_eq!(a.source_assertion().err(), row["expected"]["diagnostic"].as_str());
            assert_eq!(source_view(&a), json!({"build":null,"accession":null,"position":null,"reference":null,"alternate":null}));
        }
        // No compatibility is called at this seam; refusal leaves the accumulator unset.
        let components = component_view(a.components());
        assert_eq!(components, row["expected"]["components"]);
        assert_eq!(row["expected"]["compatibility_retry_calls"], 0);
        assert_eq!(row["expected"]["post_refusal_parser_calls"], 0);
        assert_eq!(parser_calls(&a), 1);
        let m = &row["debug"]["members"];
        let reference_bytes = m["reference_bytes"].as_u64().map(|v| v as usize);
        let prediction = m["prediction"].as_bool();
        let diagnostic = m["diagnostic"].as_str();
        let expected_debug = format!(
            "GenomicAssertion {{ source_bytes: {}, reference_bytes: {:?}, outer_whitespace: {}, route: {:?}, prediction: {:?}, diagnostic: {:?} }}",
            m["source_bytes"],
            reference_bytes,
            m["outer_whitespace"],
            m["route"].as_str().unwrap(),
            prediction,
            diagnostic
        );
        let debug = format!("{a:?}");
        assert_eq!(debug, expected_debug);
        for text in row["debug"]["forbidden_text"].as_array().unwrap() {
            assert!(!debug.contains(text.as_str().unwrap()));
        }
    }
    retained_transport_custody();
}
fn comparison(c: VariantIdentityComparison) -> Value {
    match c {
        VariantIdentityComparison::Compatible { matched_alias } => {
            json!({"status":"Compatible","matched_alias":matched_alias})
        }
        VariantIdentityComparison::Contradictory { field } => {
            json!({"status":"Contradictory","field":field})
        }
        VariantIdentityComparison::Indeterminate { field } => {
            json!({"status":"Indeterminate","field":field})
        }
    }
}
#[test]
fn genomic_identity_comparison_table() {
    for row in corpus()["comparisons"].as_array().unwrap() {
        let requested = serde_json::from_value(row["requested"].clone()).unwrap();
        let source = serde_json::from_value(row["source"].clone()).unwrap();
        assert_eq!(
            comparison(compare_variant_identity(&requested, &source)),
            row["expected"],
            "{}",
            row["id"]
        );
    }
}
fn identity_view(i: &RequestedVariantIdentity) -> Value {
    json!({"gene":i.gene,"protein_change":i.protein_change,"coding_change":i.coding_change,
        "transcript":i.transcript,"genomic_accession":i.genomic_accession,"genome_build":i.genome_build,
        "position":i.position,"reference":i.reference,"alternate":i.alternate,"rsid":i.rsid})
}
fn identity_observations(i: &RequestedVariantIdentity, expected: &Value) {
    assert_eq!(identity_view(i), expected["identity"]);
    let a = i.normalized_aliases();
    assert_eq!(
        json!({"protein_changes":a.protein_changes,"coding_changes":a.coding_changes,
        "genomic_ids":a.genomic_ids,"rsids":a.rsids}),
        expected["normalized_aliases"]
    );
    assert_eq!(i.human_label(), expected["human_label"]);
}
fn custody(input: &str, row: &Value, admission: Admission) {
    let a = genomic_assertion(input, admission, true);
    let expected = &row["helper_observation"];
    assert_eq!(a.source, expected["source"]);
    assert_eq!(a.candidate, expected["candidate"]);
    assert_eq!(
        a.candidate_offset_bytes,
        expected["candidate_offset_bytes"].as_u64().unwrap() as usize
    );
    if !expected["position_lexeme"].is_null() {
        assert_eq!(
            a.components().unwrap().position_lexeme,
            expected["position_lexeme"]
        );
    }
}
#[test]
fn genomic_consumer_boundary_table() {
    let all = corpus();
    let rows = all["consumer_boundaries"].as_array().unwrap();
    let b = &rows[0];
    let input = b["input"].as_str().unwrap();
    let identity = RequestedVariantIdentity::from_variant_input(input).unwrap();
    identity_observations(&identity, &b["expected"]);
    custody(input, b, Admission::Chromosome);
    let b = &rows[1];
    let request: VariantArticleRequest = serde_json::from_value(b["input"].clone()).unwrap();
    let identity = request.validate_identity().unwrap();
    identity_observations(&identity, &b["expected"]);
    assert_eq!(
        request.display_input(&identity),
        b["expected"]["display_input"]
    );
    assert_eq!(
        identity.is_authoritative_refseq(),
        b["expected"]["is_authoritative_refseq"]
    );
    custody(
        request.genomic.as_deref().unwrap(),
        b,
        Admission::Structured,
    );
    for row in b["validation_cases"].as_array().unwrap() {
        let request: VariantArticleRequest = serde_json::from_value(row["input"].clone()).unwrap();
        let error = request.validate_identity().unwrap_err();
        let BioMcpError::InvalidArgument(message) = &error else {
            panic!("wrong error variant")
        };
        assert_eq!(
            json!({"result":"Err","variant":"InvalidArgument","message":message,
            "display":error.to_string(),"code":error.code(),"exit_code":error.exit_code(),
            "identity":null,"normalized_aliases":null}),
            row["expected"],
            "{}",
            row["id"]
        );
    }
    let comparison_ids: Vec<_> = all["comparisons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].clone())
        .collect();
    assert_eq!(json!(comparison_ids), rows[2]["expected_case_ids"]);
    for row in rows[3]["cases"].as_array().unwrap() {
        assert_eq!(
            json!(gnomad_variant_slug(row["input"].as_str().unwrap())),
            row["expected"]
        );
    }
    super::super::super::get::tests::genomic_prediction_preparation_cases();
    retained_transport_custody();
}
