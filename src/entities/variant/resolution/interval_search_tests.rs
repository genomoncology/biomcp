//! Accepted 0675 synthetic gold, MIT, BioData contributors; no runtime-derived expectations.
use super::super::interval_search::*;
use biodata::{
    HgvsProteinCoordinate, HgvsProteinIntervalEdit, HgvsProteinIntervalEnvelope,
    HgvsProteinIntervalLocation, HgvsSpan, ParsedHgvsProteinInterval,
};
use serde_json::{Value, json};

fn gold() -> Value {
    serde_json::from_str(include_str!("interval_search_oracles.json")).unwrap()
}
fn expand(value: &Value) -> String {
    if let Some(s) = value.as_str() {
        return s.to_owned();
    }
    if let Some(parts) = value["concat"].as_array() {
        return parts.iter().map(expand).collect();
    }
    value["repeat"]
        .as_str()
        .unwrap()
        .repeat(value["count"].as_u64().unwrap() as usize)
}
fn expanded(value: &Value) -> Value {
    if value.get("concat").is_some() || value.get("repeat").is_some() {
        return json!(expand(value));
    }
    match value {
        Value::Array(a) => Value::Array(a.iter().map(expanded).collect()),
        Value::Object(o) => {
            Value::Object(o.iter().map(|(k, v)| (k.clone(), expanded(v))).collect())
        }
        _ => value.clone(),
    }
}
fn span(s: HgvsSpan) -> Value {
    json!([s.start(), s.end()])
}
// Pinned producer declaration order; Debug intentionally hides residue values.
fn residue(r: biodata::HgvsProteinResidue) -> &'static str {
    const NAMES: [&str; 25] = [
        "Ala", "Arg", "Asn", "Asp", "Cys", "Gln", "Glu", "Gly", "His", "Ile", "Leu", "Lys", "Met",
        "Phe", "Pro", "Ser", "Thr", "Trp", "Tyr", "Val", "Asx", "Glx", "Sec", "Xaa", "Ter",
    ];
    NAMES[r as usize]
}
fn coordinate(c: &HgvsProteinCoordinate) -> Value {
    let narrowed = match c.position_u32() {
        Ok(value) => json!({"value":value,"error":null}),
        Err(_) => json!({"value":null,"error":"out_of_range"}),
    };
    json!({"residue":residue(c.residue()),"position":c.position(),
        "position_u32":narrowed,"residue_span":span(c.residue_span()),
        "position_span":span(c.position_span())})
}
fn parsed(p: &ParsedHgvsProteinInterval) -> Value {
    let (kind, start, end) = match p.location() {
        HgvsProteinIntervalLocation::Point(c) => ("Point", c, None),
        HgvsProteinIntervalLocation::Range { start, end } => ("Range", start, Some(end)),
        HgvsProteinIntervalLocation::InsertionFlanks { left, right } => {
            ("InsertionFlanks", left, Some(right))
        }
    };
    let (edit, inserted) = match p.edit() {
        HgvsProteinIntervalEdit::Deletion => ("deletion", None),
        HgvsProteinIntervalEdit::Duplication => ("duplication", None),
        HgvsProteinIntervalEdit::Insertion { inserted } => ("insertion", Some(inserted.residues())),
        HgvsProteinIntervalEdit::Delins { inserted } => ("delins", Some(inserted.residues())),
    };
    json!({"reference":p.reference(),"reference_unavailable":p.reference_unavailable(),
        "predicted":p.is_predicted(),"location":{"kind":kind,"start":coordinate(start),"end":end.map(coordinate)},
        "edit":{"kind":edit,"inserted":inserted.map(|a|a.iter().map(|r|residue(*r)).collect::<Vec<_>>())},
        "spans":{"whole":span(p.span()),"reference":p.reference_span().map(span),
            "location":span(p.location_span()),"marker":span(p.marker_span()),
            "sequence":p.sequence_span().map(span),"inserted_residues":p.residue_spans().iter().copied().map(span).collect::<Vec<_>>()}})
}
fn envelope(e: &HgvsProteinIntervalEnvelope) -> Value {
    assert_eq!(e.syntax_version(), "21.1.4");
    assert_eq!(e.render_source(), e.source());
    let d = match e.disposition() {
        biodata::HgvsProteinIntervalDisposition::Parsed(_) => "Parsed",
        biodata::HgvsProteinIntervalDisposition::Invalid(_) => "Invalid",
        biodata::HgvsProteinIntervalDisposition::Unsupported(_) => "Unsupported",
        biodata::HgvsProteinIntervalDisposition::ResourceLimitExceeded(_) => {
            "ResourceLimitExceeded"
        }
    };
    json!({"source":e.source(),"disposition":d,"code":e.disposition().code(),
        "input_bytes":e.input_bytes(),"reference_bytes":e.reference_bytes(),
        "diagnostic_span":e.disposition().diagnostic().and_then(|d|d.span()).map(span),
        "parsed":e.disposition().parsed().map(parsed)})
}
fn check(row: &Value, objects: &Value) {
    let source = expand(&row["input"]);
    let assertion = protein_interval_search(&source);
    assert_eq!(assertion.source(), source);
    assert_eq!(assertion.trimmed(), source.trim());
    assert_eq!(
        assertion.trim_offset(),
        source.trim().as_ptr() as usize - source.as_ptr() as usize
    );
    assert_eq!(format!("{:?}", assertion.preparation()), row["preparation"]);
    assert_eq!(
        format!("{:?}", assertion.disposition()),
        row["adapter_disposition"],
        "{}",
        row["id"]
    );
    assert_eq!(
        assertion.query_spelling().map(str::to_owned),
        (!row["query_spelling"].is_null()).then(|| expand(&row["query_spelling"]))
    );
    if let Some(expected) = row["code"].as_str() {
        assert_eq!(assertion.code(), Some(expected));
    }
    if let Some(expected) = row["candidate"].as_str() {
        assert_eq!(assertion.candidate(), Some(expected));
    }
    if let Some(n) = row["prepared_bytes"].as_u64() {
        let expected = (row["preparation"] != "None").then_some(n as usize);
        assert_eq!(assertion.candidate_bytes(), expected);
    }
    if row.get("diagnostic_span").is_some() {
        assert_eq!(
            json!(assertion.diagnostic_span().map(|(a, b)| [a, b])),
            row["diagnostic_span"]
        );
    }
    let calls = row["producer_calls"].as_u64().unwrap();
    // Envelope presence establishes admission; no production parser counter/hook is added.
    assert_eq!(assertion.envelope().is_some(), calls == 1);
    assert_eq!(assertion.candidate().is_some(), calls == 1);
    if calls == 1 {
        let candidate = assertion.candidate().unwrap();
        assert_eq!(assertion.candidate_bytes(), Some(candidate.len()));
        let actual = envelope(assertion.envelope().unwrap());
        let mut expected = if let Some(pointer) = row["producer_object"]["json_pointer"].as_str() {
            expanded(&objects[pointer])
        } else {
            row["envelope"].clone()
        };
        expected.as_object_mut().unwrap().remove("render");
        expected.as_object_mut().unwrap().remove("syntax_version");
        assert_eq!(actual, expected, "complete envelope {}", row["id"]);
        if let Some(p) = assertion.parsed() {
            for (s, key) in [
                (p.location_span(), "source_location_span"),
                (p.marker_span(), "source_marker_span"),
                (p.span(), "source_whole_candidate_span"),
            ] {
                if row.get(key).is_some() {
                    assert_eq!(
                        json!(assertion.source_span(s).map(|(a, b)| [a, b])),
                        row[key]
                    );
                }
            }
            if row.get("source_body_span").is_some() {
                let s = HgvsSpan::new(candidate, 2, candidate.len()).unwrap();
                assert_eq!(
                    json!(assertion.source_span(s).map(|(a, b)| [a, b])),
                    row["source_body_span"]
                );
            }
        }
    }
    let debug = format!("{assertion:?}");
    for private in [
        source.trim(),
        assertion.candidate().unwrap_or(""),
        "NP_000001.1",
        "A11del",
        "Ala",
        "Gly",
    ] {
        if !private.is_empty() {
            assert!(!debug.contains(private), "sanitized Debug {}", row["id"]);
        }
    }
}
#[test]
fn interval_search_adapter_table() {
    let gold = gold();
    let rows = gold["adapter_cases"].as_array().unwrap();
    assert_eq!(rows.len(), 24);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row["id"], format!("IS{:02}", index + 1));
        check(row, &gold["producer_objects"]);
        if let Some(witnesses) = row["nearest_cap_witnesses"].as_array() {
            for witness in witnesses {
                let mut row = witness.clone();
                row["id"] = json!("IS20 cap witness");
                check(&row, &gold["producer_objects"]);
            }
        }
    }
    for source in [
        "p.V600E", "V600", "p.R97fs", "p.?", "p.=", "p.A11DEL", "A11DEL",
    ] {
        let a = protein_interval_search(source);
        assert_eq!(a.disposition(), IntervalSearchDisposition::NotSelected);
        assert!(a.envelope().is_none());
        assert!(a.candidate().is_none());
    }
    // Original cap wins over reference refusal, with no megabyte candidate allocation.
    let source = format!("{}NP_1:p.A11del", " ".repeat(1_048_576));
    let a = protein_interval_search(&source);
    assert_eq!(
        a.disposition(),
        IntervalSearchDisposition::ResourceLimitExceeded
    );
    assert!(a.envelope().is_none());
    assert!(a.candidate().is_none());
}
