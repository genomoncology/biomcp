//! Authored consumer controls, not provider evidence or a copy of the library corpus.
use serde_json::{Value, json};

pub struct Case {
    pub label: &'static str,
    pub bytes: Vec<u8>,
    pub get_error: Option<&'static str>,
    pub search_error: bool,
    pub display: Option<&'static str>,
}

fn page(
    label: &'static str,
    ensembl: Option<Value>,
    error: bool,
    display: Option<&'static str>,
) -> Case {
    let mut row = json!({"symbol":"BRAF","name":"Source name","entrezgene":673,"_id":"provider-673","source_only":"SOURCE-ONLY-CANARY"});
    if let Some(value) = ensembl {
        row["ensembl"] = value;
    }
    Case {
        label,
        bytes: serde_json::to_vec(&json!({"total":1,"hits":[row]})).unwrap(),
        get_error: error.then_some("Identity"),
        search_error: error,
        display,
    }
}

pub fn cases() -> Vec<Case> {
    let mut cases = vec![
        page("E01 omitted", None, false, None),
        page("E01 null", Some(json!(null)), false, None),
        page("E02 missing", Some(json!({})), false, None),
        page("E02 null", Some(json!({"gene":null})), false, None),
        page("E03 blank", Some(json!({"gene":""})), false, None),
        page("E03 whitespace", Some(json!({"gene":"  "})), false, None),
        page("E04 empty", Some(json!([])), false, None),
        page(
            "E05 missing first",
            Some(json!([{}, {"gene":"G1"}])),
            true,
            None,
        ),
        page(
            "E06 null first",
            Some(json!([{"gene":null}, {"gene":"G1"}])),
            true,
            None,
        ),
        page(
            "E07 blank first",
            Some(json!([{"gene":"  "}, {"gene":"G1"}])),
            true,
            None,
        ),
        page(
            "E08 missing later",
            Some(json!([{"gene":"G1"}, {}])),
            true,
            None,
        ),
        page(
            "E08 null later",
            Some(json!([{"gene":"G1"}, {"gene":null}])),
            true,
            None,
        ),
        page(
            "E08 blank later",
            Some(json!([{"gene":"G1"}, {"gene":""}])),
            true,
            None,
        ),
        page(
            "E09 distinct",
            Some(json!([{"gene":"G1"}, {"gene":"G2"}])),
            false,
            Some("G1"),
        ),
        page(
            "E10 heterogeneous",
            Some(json!([{"gene":"G1","transcript":["T1"]}, {"gene":"G2","protein":["P2"]}])),
            false,
            Some("G1"),
        ),
        page(
            "E11 repeat",
            Some(json!([{"gene":"G1"}, {"gene":"G1"}])),
            false,
            Some("G1"),
        ),
        page(
            "E12 vector",
            Some(json!({"gene":["G1","G2"]})),
            false,
            Some("G1"),
        ),
        page(
            "E12 array vector",
            Some(json!([{"gene":["G1","G2"]}])),
            false,
            Some("G1"),
        ),
        page("E13 empty vector", Some(json!({"gene":[]})), false, None),
        page(
            "E13 first empty vector",
            Some(json!([{"gene":[]}, {"gene":"G1"}])),
            false,
            None,
        ),
        page("E14 number", Some(json!({"gene":7})), true, None),
        page("E14 null member", Some(json!([null])), true, None),
        page(
            "E14 nested array",
            Some(json!([[{"gene":"G1"}]])),
            true,
            None,
        ),
        page("E14 scalar", Some(json!(7)), true, None),
    ];
    for (label, body, get_error, search_error) in [
        (
            "empty",
            json!({"total":0,"hits":[]}),
            Some("not_found"),
            false,
        ),
        (
            "mismatch",
            json!({"total":1,"hits":[{"symbol":"OTHER","entrezgene":1}]}),
            Some("Mismatch"),
            false,
        ),
        (
            "case mismatch",
            json!({"total":1,"hits":[{"symbol":"braf","entrezgene":673}]}),
            Some("Mismatch"),
            false,
        ),
        (
            "missing symbol",
            json!({"total":1,"hits":[{"entrezgene":673}]}),
            Some("Mismatch"),
            false,
        ),
        (
            "null symbol",
            json!({"total":1,"hits":[{"symbol":null}]}),
            Some("Mismatch"),
            false,
        ),
        (
            "blank symbol",
            json!({"total":1,"hits":[{"symbol":" "}]}),
            Some("Mismatch"),
            false,
        ),
        (
            "missing name",
            json!({"total":1,"hits":[{"symbol":"BRAF","entrezgene":"673"}]}),
            None,
            false,
        ),
        (
            "null name",
            json!({"total":1,"hits":[{"symbol":"BRAF","name":null}]}),
            None,
            false,
        ),
        (
            "blank name",
            json!({"total":1,"hits":[{"symbol":"BRAF","name":" "}]}),
            None,
            false,
        ),
        (
            "null id",
            json!({"total":1,"hits":[{"symbol":"BRAF","_id":null}]}),
            None,
            false,
        ),
        (
            "blank id",
            json!({"total":1,"hits":[{"symbol":"BRAF","_id":" "}]}),
            Some("Identity"),
            true,
        ),
        (
            "repeated id",
            json!({"total":2,"hits":[{"symbol":"BRAF","_id":"1","entrezgene":673},{"symbol":"OTHER","_id":"1","entrezgene":1}]}),
            None,
            false,
        ),
        (
            "ranked rows",
            json!({"total":4,"hits":[
                {"symbol":"OTHER","name":"First","entrezgene":1},
                {"symbol":"BRAF","name":"Exact","entrezgene":"673"},
                {"symbol":"OTHER","name":"Duplicate","entrezgene":"1"},
                {"symbol":"THIRD","name":"Last","entrezgene":3}
            ]}),
            None,
            false,
        ),
        (
            "ambiguous",
            json!({"total":2,"hits":[{"symbol":"BRAF"},{"symbol":"BRAF"}]}),
            Some("Ambiguous"),
            false,
        ),
        (
            "rejected neighbor",
            json!({"total":3,"hits":[{"symbol":"BRAF"},{"symbol":"OTHER","ensembl":[{}]},{"symbol":"THIRD"}]}),
            Some("Identity"),
            true,
        ),
        (
            "ambiguous before rejection",
            json!({"total":3,"hits":[{"symbol":"BRAF"},{"symbol":"BRAF"},{"symbol":"OTHER","ensembl":[{}]}]}),
            Some("Ambiguous"),
            true,
        ),
        (
            "HGNC equivalent",
            json!({"total":1,"hits":[{"symbol":"BRAF","HGNC":["HGNC:01097","1097",1097]}]}),
            None,
            false,
        ),
        (
            "HGNC distinct",
            json!({"total":1,"hits":[{"symbol":"BRAF","HGNC":["1097","42"]}]}),
            None,
            false,
        ),
        (
            "missing total",
            json!({"hits":[{"symbol":"BRAF"}]}),
            Some("representable total"),
            true,
        ),
        (
            "source shape",
            json!({"total":1,"hits":[{"symbol":"BRAF","summary":7}]}),
            Some("enrichment shape"),
            false,
        ),
    ] {
        cases.push(Case {
            label,
            bytes: serde_json::to_vec(&body).unwrap(),
            get_error,
            search_error,
            display: None,
        });
    }
    for (label, body, error) in [
        (
            "duplicate",
            r#"{"total":1,"hits":[{"symbol":"BRAF","symbol":"SOURCE-ONLY-CANARY"}]}"#,
            "Duplicate",
        ),
        ("malformed", r#"{"total":1,"hits":{}}"#, "Structure"),
        (
            "overflow total",
            r#"{"total":18446744073709551616,"hits":[]}"#,
            "Structure",
        ),
    ] {
        cases.push(Case {
            label,
            bytes: body.as_bytes().to_vec(),
            get_error: Some(error),
            search_error: true,
            display: None,
        });
    }
    cases.push(Case {
        label: "scalar alias",
        bytes:
            br#"{ "total":1, "hits":[{"symbol":"BRAF","alias":"SOURCEALIAS","entrezgene":"673"}]}"#
                .to_vec(),
        get_error: None,
        search_error: false,
        display: None,
    });
    cases.push(Case {
        label: "limit",
        bytes: vec![b' '; 1_048_577],
        get_error: Some("Limit"),
        search_error: true,
        display: None,
    });
    cases
}
