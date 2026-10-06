//! Literal transcription of accepted 0673 cases, MIT, BioData contributors.
//! Independently authored synthetic specifications; no provider payload or execution-derived gold.
use super::*;
use super::interval_comparison::{self, IntervalAssertion, INPUT_LIMIT};
use biodata::{HgvsProteinCoordinate, HgvsProteinIntervalEdit, HgvsProteinIntervalLocation};
use serde_json::{Value, json};

// Pinned producer declaration ordinals expose typed getter values without point-only markers.
const RESIDUES: [&str; 25] = [
    "Ala", "Arg", "Asn", "Asp", "Cys", "Gln", "Glu", "Gly", "His", "Ile",
    "Leu", "Lys", "Met", "Phe", "Pro", "Ser", "Thr", "Trp", "Tyr", "Val",
    "Asx", "Glx", "Sec", "Xaa", "Ter",
];
// Complete semantic gold: prediction, location, ordered residue/decimal endpoints, edit, sequence.
#[rustfmt::skip]
const PAIRS: [(&str, &str, bool, &str, &str, &str); 48] = [
    ("p.A11del", "p.Ala11del", true, "I", "0|Point|Ala:11|Deletion|", "0|Point|Ala:11|Deletion|"),
    ("p.A11_G12del", "p.Ala11_Gly12del", true, "I", "0|Range|Ala:11,Gly:12|Deletion|", "0|Range|Ala:11,Gly:12|Deletion|"),
    ("p.A11dup", "p.Ala11dup", true, "I", "0|Point|Ala:11|Duplication|", "0|Point|Ala:11|Duplication|"),
    ("p.A11_G12dup", "p.Ala11_Gly12dup", true, "I", "0|Range|Ala:11,Gly:12|Duplication|", "0|Range|Ala:11,Gly:12|Duplication|"),
    ("p.A11_G12insS", "p.Ala11_Gly12insSer", true, "I", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Ser", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Ser"),
    ("p.A11delinsGS", "p.Ala11delinsGlySer", true, "I", "0|Point|Ala:11|Delins|Gly,Ser", "0|Point|Ala:11|Delins|Gly,Ser"),
    ("p.A11_G12delinsS", "p.Ala11_Gly12delinsSer", true, "I", "0|Range|Ala:11,Gly:12|Delins|Ser", "0|Range|Ala:11,Gly:12|Delins|Ser"),
    ("p.A11_G12insS*", "p.Ala11_Gly12insSerTer", true, "I", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Ser,Ter", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Ser,Ter"),
    ("p.A11_G12insXaa", "p.Ala11_Gly12insXaa", true, "I", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Xaa", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Xaa"),
    ("p.A11_G12insU", "p.Ala11_Gly12insSec", true, "I", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Sec", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Sec"),
    ("p.(A11del)", "p.(Ala11del)", true, "I", "1|Point|Ala:11|Deletion|", "1|Point|Ala:11|Deletion|"),
    ("NP_000001.1:p.A11del", "NP_000002.2:p.Ala11del", true, "I", "0|Point|Ala:11|Deletion|", "0|Point|Ala:11|Deletion|"),
    ("  A11del  ", "p.Ala11del", true, "I", "0|Point|Ala:11|Deletion|", "0|Point|Ala:11|Deletion|"),
    ("P.A11del", "p.Ala11del", true, "I", "0|Point|Ala:11|Deletion|", "0|Point|Ala:11|Deletion|"),
    ("p.A011del", "p.Ala11del", false, "I", "0|Point|Ala:011|Deletion|", "0|Point|Ala:11|Deletion|"),
    ("p.(A11del)", "p.Ala11del", false, "I", "1|Point|Ala:11|Deletion|", "0|Point|Ala:11|Deletion|"),
    ("p.A11del", "p.Gly11del", false, "I", "0|Point|Ala:11|Deletion|", "0|Point|Gly:11|Deletion|"),
    ("p.A11del", "p.Ala12del", false, "I", "0|Point|Ala:11|Deletion|", "0|Point|Ala:12|Deletion|"),
    ("p.A11del", "p.Ala11dup", false, "I", "0|Point|Ala:11|Deletion|", "0|Point|Ala:11|Duplication|"),
    ("p.A11del", "p.Ala11_Gly12del", false, "I", "0|Point|Ala:11|Deletion|", "0|Range|Ala:11,Gly:12|Deletion|"),
    ("p.A11_G12insGS", "p.Ala11_Gly12insSerGly", false, "I", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Gly,Ser", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Ser,Gly"),
    ("p.A11_G12insS", "p.Ala11_Gly12insSerTer", false, "I", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Ser", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Ser,Ter"),
    ("p.A11_G13insS", "p.Ala11_Gly12insSer", false, "I", "invalid", "unparsed"),
    ("p.A11del", "p.Ala0del", false, "I", "0|Point|Ala:11|Deletion|", "invalid"),
    ("p.X11del", "p.Ala11del", false, "I", "legacy_x", "unparsed"),
    ("p.A11del", "p.Ala11delinsXG", false, "I", "0|Point|Ala:11|Deletion|", "legacy_x"),
    ("p.A11delinsG", "p.Ala11delinsGly", false, "I", "invalid", "unparsed"),
    ("p.A11_G12ins*S", "p.Ala11_Gly12insSer", false, "I", "invalid", "unparsed"),
    ("p.A11del!", "p.Ala11del", false, "I", "invalid", "unparsed"),
    ("p.(A11del", "p.Ala11del", false, "I", "invalid", "unparsed"),
    ("p.A11del", "p.Ala11delA", false, "I", "0|Point|Ala:11|Deletion|", "invalid"),
    ("p.A11del", "p.Ala11dupA", false, "I", "0|Point|Ala:11|Deletion|", "invalid"),
    ("p.A11del", "p.ALA11DEL", false, "N", "", ""),
    ("p.X11del", "p.x11DEL", true, "L", "", ""),
    ("p.A11_G13insS", "p.A11_G13insS", true, "L", "", ""),
    ("NP_1:p.Glu746_Ala750del", "NP_2:p.Glu746_Ala750del", true, "L", "", ""),
    ("p.V600E", "p.Val600Glu", true, "P", "", ""),
    ("p.R97fs", "p.Arg97fs", false, "N", "", ""),
    ("p.?", "p.=", false, "N", "", ""),
    ("", "p.", true, "L", "", ""),
    ("p.A11_G12del", "p.Ala11_Ser12del", false, "I", "0|Range|Ala:11,Gly:12|Deletion|", "0|Range|Ala:11,Ser:12|Deletion|"),
    ("p.A11_G012del", "p.Ala11_Gly12del", false, "I", "0|Range|Ala:11,Gly:012|Deletion|", "0|Range|Ala:11,Gly:12|Deletion|"),
    ("p.A11_G12dup", "p.Ala11_Ser12dup", false, "I", "0|Range|Ala:11,Gly:12|Duplication|", "0|Range|Ala:11,Ser:12|Duplication|"),
    ("p.A11_G012dup", "p.Ala11_Gly12dup", false, "I", "0|Range|Ala:11,Gly:012|Duplication|", "0|Range|Ala:11,Gly:12|Duplication|"),
    ("p.A11_G12delinsS", "p.Ala11_Ser12delinsSer", false, "I", "0|Range|Ala:11,Gly:12|Delins|Ser", "0|Range|Ala:11,Ser:12|Delins|Ser"),
    ("p.A11_G012delinsS", "p.Ala11_Gly12delinsSer", false, "I", "0|Range|Ala:11,Gly:012|Delins|Ser", "0|Range|Ala:11,Gly:12|Delins|Ser"),
    ("p.A11_G12insS", "p.Ala11_Ser12insSer", false, "I", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Ser", "0|InsertionFlanks|Ala:11,Ser:12|Insertion|Ser"),
    ("p.A11_G012insS", "p.Ala11_Gly12insSer", false, "I", "0|InsertionFlanks|Ala:11,Gly:012|Insertion|Ser", "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Ser"),
];

fn coordinate(c: &HgvsProteinCoordinate) -> String {
    format!("{}:{}", RESIDUES[c.residue() as usize], c.position())
}

fn observe(assertion: &IntervalAssertion<'_>, expected: &str) {
    if expected == "unparsed" {
        assert!(assertion.envelope.is_none());
        return;
    }
    let e = assertion.envelope.as_ref().expect("complete envelope");
    assert_eq!(e.syntax_version(), "21.1.4");
    assert_eq!(e.source(), format!("p.{}", assertion.body));
    assert_eq!(e.render_source(), e.source());
    assert_eq!(e.input_bytes(), assertion.body.len() + 2);
    assert_eq!(e.reference_bytes(), None);
    if matches!(expected, "invalid" | "legacy_x") {
        let code = if expected == "invalid" {
            "hgvs_protein_interval_invalid"
        } else {
            "hgvs_protein_interval_unsupported_legacy_x"
        };
        assert!(e.disposition().parsed().is_none());
        assert_eq!(e.disposition().code(), code);
        let diagnostic = e.disposition().diagnostic().unwrap();
        assert_eq!(diagnostic.code(), code);
        assert_eq!(diagnostic.span().map(|s| (s.start(), s.end())), Some((0, e.input_bytes())));
        return;
    }
    assert_eq!(e.disposition().code(), "hgvs_protein_interval_parsed");
    assert!(e.disposition().diagnostic().is_none());
    let p = e.disposition().parsed().expect("Parsed, not refusal");
    assert_eq!(p.reference(), None);
    assert!(p.reference_unavailable());
    assert_eq!(p.reference_span(), None);
    let (shape, endpoints) = match p.location() {
        HgvsProteinIntervalLocation::Point(c) => ("Point", coordinate(c)),
        HgvsProteinIntervalLocation::Range { start, end } => ("Range", format!("{},{}", coordinate(start), coordinate(end))),
        HgvsProteinIntervalLocation::InsertionFlanks { left, right } => ("InsertionFlanks", format!("{},{}", coordinate(left), coordinate(right))),
    };
    let (edit, sequence) = match p.edit() {
        HgvsProteinIntervalEdit::Deletion => ("Deletion", String::new()),
        HgvsProteinIntervalEdit::Duplication => ("Duplication", String::new()),
        HgvsProteinIntervalEdit::Insertion { inserted } => ("Insertion", inserted.residues().iter().map(|r| RESIDUES[*r as usize]).collect::<Vec<_>>().join(",")),
        HgvsProteinIntervalEdit::Delins { inserted } => ("Delins", inserted.residues().iter().map(|r| RESIDUES[*r as usize]).collect::<Vec<_>>().join(",")),
    };
    assert_eq!(format!("{}|{shape}|{endpoints}|{edit}|{sequence}", u8::from(p.is_predicted())), expected);
}

// Whole/location/marker/second-residue/second-decimal/sequence/inserted spans, Q41–Q48.
#[rustfmt::skip]
const ENDPOINT_SPANS: [[usize; 11]; 16] = [
    [12,9,9,12,6,7,7,9,0,0,0], [16,13,13,16,8,11,11,13,0,0,0],
    [13,10,10,13,6,7,7,10,0,0,0], [16,13,13,16,8,11,11,13,0,0,0],
    [12,9,9,12,6,7,7,9,0,0,0], [16,13,13,16,8,11,11,13,0,0,0],
    [13,10,10,13,6,7,7,10,0,0,0], [16,13,13,16,8,11,11,13,0,0,0],
    [16,9,9,15,6,7,7,9,15,16,1], [22,13,13,19,8,11,11,13,19,22,1],
    [17,10,10,16,6,7,7,10,16,17,1], [22,13,13,19,8,11,11,13,19,22,1],
    [13,9,9,12,6,7,7,9,12,13,1], [19,13,13,16,8,11,11,13,16,19,1],
    [14,10,10,13,6,7,7,10,13,14,1], [19,13,13,16,8,11,11,13,16,19,1],
];

fn spans(a: &IntervalAssertion<'_>, gold: [usize; 11], long: bool) {
    let p = a.envelope.as_ref().unwrap().disposition().parsed().unwrap();
    let span = |s: biodata::HgvsSpan| (s.start(), s.end());
    assert_eq!(span(p.span()), (0, gold[0]));
    assert_eq!(span(p.location_span()), (2, gold[1]));
    assert_eq!(span(p.marker_span()), (gold[2], gold[3]));
    let (first, second) = match p.location() {
        HgvsProteinIntervalLocation::Range { start, end } => (start, end),
        HgvsProteinIntervalLocation::InsertionFlanks { left, right } => (left, right),
        _ => panic!("two complete endpoints required"),
    };
    assert_eq!(span(first.residue_span()), (2, if long { 5 } else { 3 }));
    assert_eq!(span(first.position_span()), (if long { 5 } else { 3 }, if long { 7 } else { 5 }));
    assert_eq!(span(second.residue_span()), (gold[4], gold[5]));
    assert_eq!(span(second.position_span()), (gold[6], gold[7]));
    let sequence = (gold[10] == 1).then_some((gold[8], gold[9]));
    assert_eq!(p.sequence_span().map(span), sequence);
    assert_eq!(p.residue_spans().iter().copied().map(span).collect::<Vec<_>>(), sequence.into_iter().collect::<Vec<_>>());
    assert_eq!((a.body_offset, a.body.len()), (2, gold[0] - 2));
    assert_eq!(a.source_span(p.location_span()), Some((2, gold[1])));
    assert_eq!(a.source, a.envelope.as_ref().unwrap().source());
    assert_eq!(a.reference, None);
}

#[test]
fn interval_alias_pair_resource_privacy_table() {
    for (i, &(left, right, equal, route, l, r)) in PAIRS.iter().enumerate() {
        assert_eq!(protein_changes_equivalent(left, right), equal, "Q{:02}", i + 1);
        if route == "I" {
            let result = interval_comparison::compare(left, right);
            assert_eq!(result.equivalent, equal);
            assert_eq!(result.left.source, left);
            assert_eq!(result.right.source, right);
            observe(&result.left, l);
            observe(&result.right, r);
            if i >= 40 {
                spans(&result.left, ENDPOINT_SPANS[(i - 40) * 2], false);
                spans(&result.right, ENDPOINT_SPANS[(i - 40) * 2 + 1], true);
            }
        } else if route == "N" {
            let result = interval_comparison::compare(left, right);
            assert!(result.left.envelope.is_none() && result.right.envelope.is_none());
        }
    }
    let reversed = interval_comparison::compare(PAIRS[22].1, PAIRS[22].0);
    assert!(!reversed.equivalent);
    observe(&reversed.left, "0|InsertionFlanks|Ala:11,Gly:12|Insertion|Ser");
    observe(&reversed.right, "invalid");
    assert!(!protein_changes_equivalent(PAIRS[22].1, PAIRS[22].0));
    for (index, expected) in [(0, None), (22, Some("hgvs_protein_interval_invalid")), (24, Some("hgvs_protein_interval_unsupported_legacy_x"))] {
        let result = interval_comparison::compare(PAIRS[index].0, PAIRS[index].1);
        let (bytes, prediction) = if index == 22 { (13, "None") } else if index == 24 { (8, "None") } else { (8, "Some(false)") };
        debug(&result.left, "interval", bytes, bytes, 2, bytes - 2, prediction, expected);
    }
    let reference = interval_comparison::compare(PAIRS[11].0, PAIRS[11].1);
    debug(&reference.left, "interval", 20, 8, 14, 6, "Some(false)", None);
    assert_eq!(reference.left.reference, Some("NP_000001.1"));
    assert!(!format!("{:?}", reference.left).contains("NP_000001.1"));
    let d = "1".repeat(INPUT_LIMIT - 8);
    let (left, right) = (format!("p.A{d}del"), format!("p.Ala{d}del"));
    assert!(protein_changes_equivalent(&left, &right));
    let result = interval_comparison::compare(&left, &right);
    assert!(result.equivalent);
    assert_eq!(result.left.candidate_bytes, Some(INPUT_LIMIT - 2));
    assert_eq!(result.right.candidate_bytes, Some(INPUT_LIMIT));
    for a in [&result.left, &result.right] {
        let p = a.envelope.as_ref().unwrap().disposition().parsed().unwrap();
        let HgvsProteinIntervalLocation::Point(c) = p.location() else { panic!("Point") };
        assert_eq!(c.position(), d);
    }
    let d = "1".repeat(INPUT_LIMIT - 7);
    let (left, right) = (format!("p.A{d}del"), format!("p.Ala{d}del"));
    assert!(!protein_changes_equivalent(&left, &right));
    let result = interval_comparison::compare(&left, &right);
    assert!(result.left.envelope.is_none() && result.right.envelope.is_none());
    debug(&result.right, "resource", INPUT_LIMIT + 1, INPUT_LIMIT + 1, 2, INPUT_LIMIT - 1, "None", Some("hgvs_input_limit"));
    let oversized = format!("p.A{}del", "1".repeat(INPUT_LIMIT));
    assert!(protein_changes_equivalent(&oversized, &oversized));
    let source = format!("{}:p.A11del", "R".repeat(4097));
    assert!(protein_changes_equivalent(&source, "p.Ala11del"));
    let result = interval_comparison::compare(&source, "p.Ala11del");
    assert!(result.equivalent);
    assert_eq!(result.left.reference, Some(&source[..4097]));
    observe(&result.left, "0|Point|Ala:11|Deletion|");
    observe(&result.right, "0|Point|Ala:11|Deletion|");
    // Literal Q01/Q05 span anchors and original body translation.
    let q1 = interval_comparison::compare(PAIRS[0].0, PAIRS[0].1);
    for (a, end, residue_end) in [(&q1.left, 5, 3), (&q1.right, 7, 5)] {
        let p = a.envelope.as_ref().unwrap().disposition().parsed().unwrap();
        assert_eq!((p.span().start(), p.span().end()), (0, end + 3));
        assert_eq!((p.location_span().start(), p.location_span().end()), (2, end));
        assert_eq!((p.marker_span().start(), p.marker_span().end()), (end, end + 3));
        let HgvsProteinIntervalLocation::Point(c) = p.location() else { panic!("Point") };
        assert_eq!((c.residue_span().start(), c.residue_span().end()), (2, residue_end));
        assert_eq!((c.position_span().start(), c.position_span().end()), (residue_end, end));
        assert_eq!(p.sequence_span(), None);
        assert!(p.residue_spans().is_empty());
    }
    let q5 = interval_comparison::compare(PAIRS[4].0, PAIRS[4].1);
    spans(&q5.left, [13,9,9,12,6,7,7,9,12,13,1], false);
    spans(&q5.right, [19,13,13,16,8,11,11,13,16,19,1], true);
    let q13 = interval_comparison::compare(PAIRS[12].0, PAIRS[12].1);
    assert_eq!((q13.left.body_offset, q13.left.body.len()), (2, 6));
    let p = reference.left.envelope.as_ref().unwrap().disposition().parsed().unwrap();
    assert_eq!(reference.left.source_span(p.location_span()), Some((14, 17)));
}

fn debug(a: &IntervalAssertion<'_>, route: &str, source: usize, candidate: usize, offset: usize, body: usize, prediction: &str, diagnostic: Option<&str>) {
    assert_eq!(format!("{a:?}"), format!("IntervalAssertion {{ route: {route:?}, source_bytes: {source}, candidate_bytes: Some({candidate}), body_offset: {offset}, body_bytes: {body}, prediction: {prediction}, diagnostic: {diagnostic:?} }}"));
}

fn request(protein: &str) -> RequestedVariantIdentity {
    RequestedVariantIdentity {
        clinvar_variation_id: None,
        gene: Some("EGFR".into()), protein_change: Some(protein.into()),
        coding_change: None, transcript: None, genomic_accession: None, genome_build: None,
        position: None, reference: None, alternate: None, rsid: None,
    }
}

#[test]
fn interval_identity_comparison_table() {
    for i in 0..18 {
        let pair = match i { 0..=3 => [0,2,4,5][i], 10..=17 => i + 30, _ => 0 };
        let mut requested = request(PAIRS[pair].0);
        let mut source = SourceVariantIdentity {
            genomic_id: "chr7:g.55242465_55242479del".into(), genome_build: "GRCh37".into(),
            genes: vec!["EGFR".into()], protein_changes: vec![format!("NP_1:{}", PAIRS[pair].1)],
            coding_changes: vec![], rsids: vec![],
        };
        let mut expected = VariantIdentityComparison::Compatible { matched_alias: source.protein_changes[0].clone() };
        match i {
            4 => { source.protein_changes.clear(); expected = VariantIdentityComparison::Indeterminate { field: "protein_change" }; }
            5 => { source.protein_changes = vec!["NP_1:p.Ala12del".into()]; expected = VariantIdentityComparison::Contradictory { field: "protein_change" }; }
            6 => { source.genes = vec!["OTHER".into()]; expected = VariantIdentityComparison::Contradictory { field: "gene" }; }
            7 => { source.genes.push("OTHER".into()); expected = VariantIdentityComparison::Indeterminate { field: "gene_annotation_tuple" }; }
            8 => { source.protein_changes = vec!["NP_bad:p.X11del".into(), "NP_1:p.Ala11del".into(), "NP_2:p.A11del".into()]; }
            9 => { requested.transcript = Some("NM_1".into()); source.coding_changes = vec!["NM_2:c.1A>T".into()]; expected = VariantIdentityComparison::Contradictory { field: "transcript" }; }
            10..=17 => expected = VariantIdentityComparison::Contradictory { field: "protein_change" },
            _ => {}
        }
        let before = (requested.clone(), source.clone());
        assert_eq!(compare_variant_identity(&requested, &source), expected, "C{:02}", i + 1);
        assert_eq!((requested, source), before);
    }
}

pub(crate) fn annotation_table<T>(
    matches: impl Fn(&crate::sources::myvariant::MyVariantSnpeffAnnotation, &RequestedVariantIdentity) -> bool,
    retain: impl Fn(&RequestedVariantIdentity, crate::sources::myvariant::MyVariantHit, &mut std::collections::HashSet<String>, &mut Vec<T>) -> bool,
    finalize: impl Fn(&RequestedVariantIdentity, Vec<T>, usize, usize, bool, bool) -> crate::entities::variant::search::VariantSearchPage,
) {
    use crate::sources::myvariant::MyVariantSnpeffAnnotation;
    for i in 0..16 {
        let pair = match i { 0..=3 => [0,2,4,5][i], 8..=15 => i + 32, _ => 0 };
        let mut requested = request(PAIRS[pair].0);
        let mut annotation = MyVariantSnpeffAnnotation {
            genename: Some("EGFR".into()), feature_id: Some("NM_1".into()),
            hgvs_c: Some("c.1A>T".into()), hgvs_p: Some(format!("NP_1:{}", PAIRS[pair].1)),
        };
        match i {
            4 => annotation.genename = Some("OTHER".into()),
            5 => requested.transcript = Some("NM_2".into()),
            6 => requested.coding_change = Some("c.2A>T".into()),
            7 => annotation.hgvs_p = None,
            _ => {}
        }
        let before = (requested.clone(), serde_json::to_value(&annotation).unwrap());
        assert_eq!(matches(&annotation, &requested), i < 4, "A{}", if i < 8 { i + 1 } else { i + 2 });
        assert_eq!((requested, serde_json::to_value(&annotation).unwrap()), before);
    }
    for (protein, count) in [("NP_005219.2:p.E746_A750del", 1), ("NP_005219.2:p.E746_S750del", 0), ("NP_005219.2:p.E746_A0750del", 0)] {
        let requested = request("p.Glu746_Ala750del");
        let before = requested.clone();
        let input = json!({
            "_id":"chr7:g.55242465_55242479del",
            "dbnsfp":{"genename":"EGFR", "hgvsp":protein},
            "snpeff":{"ann":[{"feature_id":"NM_005228.5", "genename":"EGFR", "hgvs_c":"c.2235_2249del", "hgvs_p":protein}]}
        });
        let (mut seen, mut retained) = (std::collections::HashSet::new(), Vec::new());
        let uncertain = retain(&requested, serde_json::from_value(input).unwrap(), &mut seen, &mut retained);
        assert!(!uncertain);
        assert_eq!((seen.len(), retained.len()), (count, count));
        let actual = finalize(&requested, retained, 0, 10, false, true);
        assert_eq!(requested, before);
        assert_eq!(page_projection(actual), page_gold(count));
    }
}

fn request_projection(r: RequestedVariantIdentity) -> Value {
    json!({"gene":r.gene, "protein_change":r.protein_change, "coding_change":r.coding_change,
        "transcript":r.transcript, "genomic_accession":r.genomic_accession, "genome_build":r.genome_build,
        "position":r.position, "reference":r.reference, "alternate":r.alternate, "rsid":r.rsid})
}

fn page_projection(p: crate::entities::variant::search::VariantSearchPage) -> Value {
    let rows = p.results.into_iter().map(|r| json!({
        "id":r.id, "genome_build":r.genome_build, "genome_build_provenance":r.genome_build_provenance,
        "gene":r.gene, "hgvs_p":r.hgvs_p, "hgvs_c":r.hgvs_c, "transcript":r.transcript,
        "legacy_name":r.legacy_name, "significance":r.significance, "clinvar_stars":r.clinvar_stars,
        "gnomad_af":r.gnomad_af, "revel":r.revel, "gerp":r.gerp, "source_identity":r.source_identity,
        "matched_alias":r.matched_alias, "transcript_annotations_complete":r.transcript_annotations_complete,
        "transcript_annotations":r.transcript_annotations
    })).collect::<Vec<_>>();
    json!({"results":rows, "total":p.total, "requested_variant":p.requested_variant.map(request_projection),
        "resolution":p.resolution, "filter_evaluation":p.filter_evaluation, "has_more":p.has_more,
        "diagnostics":p.diagnostics})
}

fn page_gold(count: usize) -> Value {
    let rows = if count == 0 { json!([]) } else { json!([{
        "id":"chr7:g.55242465_55242479del", "genome_build":"GRCh37",
        "genome_build_provenance":"MyVariant.info provider default", "gene":"EGFR",
        "hgvs_p":"NP_005219.2:p.E746_A750del", "hgvs_c":"c.2235_2249del", "transcript":"NM_005228.5",
        "legacy_name":null, "significance":null, "clinvar_stars":null, "gnomad_af":null, "revel":null, "gerp":null,
        "source_identity":{"genomic_id":"chr7:g.55242465_55242479del", "genome_build":"GRCh37", "genes":["EGFR"],
            "protein_changes":["NP_005219.2:p.E746_A750del"], "coding_changes":[], "rsids":[]},
        "matched_alias":"NP_005219.2:p.E746_A750del", "transcript_annotations_complete":true,
        "transcript_annotations":[{"source":"myvariant.info/snpeff.ann", "gene":"EGFR", "transcript":"NM_005228.5",
            "hgvs_c":"c.2235_2249del", "hgvs_p":"NP_005219.2:p.E746_A750del", "roles":["displayed","matched"]}]
    }]) };
    json!({"results":rows, "total":count,
        "requested_variant":{"gene":"EGFR", "protein_change":"p.Glu746_Ala750del", "coding_change":null,
            "transcript":null, "genomic_accession":null, "genome_build":null, "position":null,
            "reference":null, "alternate":null, "rsid":null},
        "resolution":{"status":if count == 0 { "unresolved" } else { "resolved" },
            "normalized_aliases":{"protein_changes":[], "coding_changes":[], "genomic_ids":[], "rsids":[]}, "exhaustive":true},
        "filter_evaluation":{}, "has_more":false, "diagnostics":[]})
}
