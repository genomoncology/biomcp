//! Original authored synthetic bytes. No captures, private inputs or provider data.
//! Licensed as repository MIT source code. No external attribution or transformation.
pub struct Case {
    pub label: &'static str,
    pub get: Vec<u8>,
    pub search: Vec<u8>,
    pub error: bool,
    pub name: &'static str,
}
pub fn cases() -> Vec<Case> {
    let mut cases: Vec<Case> = [
        ("DO priority", r#"{"_id":"MONDO:1","mondo":{"name":" Other "},"disease_ontology":{"name":" Synthetic tumor "}}"#, false, "Synthetic tumor"),
        ("MONDO", r#"{"_id":"MONDO:1","mondo":{"name":" Synthetic tumor "}}"#, false, "Synthetic tumor"),
        ("missing", r#"{"_id":"MONDO:1"}"#, false, "MONDO:1"),
        ("null", r#"{"_id":"MONDO:1","mondo":{"name":null}}"#, false, "MONDO:1"),
        ("blank", r#"{"_id":"MONDO:1","mondo":{"name":" "}}"#, false, "MONDO:1"),
        ("equal", r#"{"_id":"MONDO:1","mondo":{"name":"Synthetic tumor"},"disease_ontology":{"name":"Synthetic tumor"}}"#, false, "Synthetic tumor"),
        ("blank ID", r#"{"_id":" ","canary":"SOURCE-ONLY-CANARY"}"#, true, ""),
        ("missing ID", r#"{"mondo":{"name":"SOURCE-ONLY-CANARY"}}"#, true, ""),
        ("wrong ID", r#"{"_id":17,"canary":"SOURCE-ONLY-CANARY"}"#, true, ""),
        ("malformed name", r#"{"_id":"MONDO:1","mondo":{"name":17},"canary":"SOURCE-ONLY-CANARY"}"#, true, ""),
        ("unsupported section", r#"{"_id":"MONDO:1","mondo":[],"canary":"SOURCE-ONLY-CANARY"}"#, true, ""),
        ("duplicate", r#"{"_id":"MONDO:1","_id":"MONDO:2","canary":"SOURCE-ONLY-CANARY"}"#, true, ""),
    ].into_iter().map(|(label, raw, error, name)| Case {
        label, get: raw.as_bytes().to_vec(), search: format!(r#"{{"total":1,"hits":[{raw}]}}"#).into_bytes(), error, name,
    }).collect();
    for (label, get, search) in [
        ("invalid JSON", "{SOURCE-ONLY-CANARY", "{SOURCE-ONLY-CANARY"),
        (
            "missing total",
            r#"{"_id":false}"#,
            r#"{"hits":[],"canary":"SOURCE-ONLY-CANARY"}"#,
        ),
        (
            "overflow total",
            r#"{"_id":false}"#,
            r#"{"total":18446744073709551616,"hits":[]}"#,
        ),
    ] {
        cases.push(Case {
            label,
            get: get.as_bytes().to_vec(),
            search: search.as_bytes().to_vec(),
            error: true,
            name: "",
        });
    }
    let mut too_large = br#"{"_id":"MONDO:1"}"#.to_vec();
    too_large.resize(1_048_577, b' ');
    cases.push(Case {
        label: "byte limit",
        get: too_large.clone(),
        search: too_large,
        error: true,
        name: "",
    });
    cases
}
