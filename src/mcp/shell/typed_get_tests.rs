use std::collections::BTreeSet;

use serde_json::json;

use super::{TypedGet, TypedVariantErepo, get_args};

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

#[test]
fn typed_variant_erepo_schema_prevents_selector_mixing_before_calls() {
    let schema =
        serde_json::to_value(rmcp::schemars::schema_for!(TypedVariantErepo)).expect("ERepo schema");
    let branches = schema["oneOf"].as_array().expect("selector branches");
    assert_eq!(branches.len(), 3);

    let branch = |selector: &str| {
        branches
            .iter()
            .find(|branch| {
                branch["required"]
                    .as_array()
                    .is_some_and(|required| required.contains(&json!(selector)))
            })
            .unwrap_or_else(|| panic!("missing {selector} selector branch"))
    };
    let caid = branch("caid");
    let caids = branch("caids");
    let gene = branch("gene");

    for selector_branch in [caid, caids, gene] {
        assert_eq!(selector_branch["additionalProperties"], false);
    }
    let property_names = |branch: &serde_json::Value| {
        branch["properties"]
            .as_object()
            .expect("branch properties")
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(
        property_names(caid),
        BTreeSet::from_iter(["caid", "detail", "assertion_id", "version"].map(str::to_owned))
    );
    assert_eq!(
        property_names(caids),
        BTreeSet::from_iter(["caids"].map(str::to_owned))
    );
    assert_eq!(
        property_names(gene),
        BTreeSet::from_iter(["gene", "limit", "offset"].map(str::to_owned))
    );
    assert_eq!(caid["properties"]["caid"]["minLength"], 1);
    assert_eq!(caids["properties"]["caids"]["minItems"], 1);
    assert_eq!(caids["properties"]["caids"]["maxItems"], 50);
    assert_eq!(caids["properties"]["caids"]["items"]["minLength"], 1);
    assert_eq!(gene["properties"]["gene"]["minLength"], 1);
    assert_eq!(gene["properties"]["limit"]["minimum"], 1);
    assert_eq!(gene["properties"]["limit"]["maximum"], 100);
    assert_eq!(gene["properties"]["limit"]["default"], 25);
    assert_eq!(gene["properties"]["offset"]["minimum"], 0);
    assert_eq!(gene["properties"]["offset"]["default"], 0);
}
