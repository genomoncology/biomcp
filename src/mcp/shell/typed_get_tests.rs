use serde_json::json;

use super::{TypedGet, get_args};

#[test]
fn adverse_event_schema_and_mapper_deduplicate_sections_only_for_that_entity() {
    let schema = serde_json::to_value(rmcp::schemars::schema_for!(TypedGet)).unwrap();
    let branches = schema["oneOf"].as_array().unwrap();
    let branch = |entity| {
        branches
            .iter()
            .find(|branch| branch["properties"]["entity"]["const"] == entity)
            .unwrap()
    };
    assert_eq!(
        branch("adverse-event")["properties"]["sections"].get("uniqueItems"),
        None
    );
    assert_eq!(
        branch("gene")["properties"]["sections"]["uniqueItems"],
        true
    );

    let args = get_args(TypedGet(json!({
        "entity": "adverse-event",
        "id": "1001",
        "sections": ["guidance", "reactions", "guidance", "reactions"],
        "json": true
    })))
    .expect("adverse-event duplicate sections are idempotent");
    assert_eq!(
        args,
        [
            "biomcp",
            "get",
            "adverse-event",
            "1001",
            "guidance",
            "reactions",
            "--json"
        ]
    );

    let error = get_args(TypedGet(json!({
        "entity": "gene",
        "id": "BRAF",
        "sections": ["pathways", "pathways"]
    })))
    .expect_err("other entities retain duplicate rejection");
    assert!(
        error
            .to_string()
            .contains("duplicate gene section: pathways")
    );
}
