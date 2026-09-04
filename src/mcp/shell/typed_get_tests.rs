use std::collections::BTreeSet;

use axum::{Json, Router, routing::get as axum_get};
use clap::CommandFactory;
use serde_json::json;

use super::{BioMcpServer, ShellCommand, TypedGet, TypedVariantErepo, get_args};

struct CtGovAgeMcpEnv(Option<std::ffi::OsString>);

impl CtGovAgeMcpEnv {
    fn set(base: &str) -> Self {
        let previous = std::env::var_os("BIOMCP_CTGOV_BASE");
        // SAFETY: callers hold the serial-test process-wide environment lock.
        unsafe { std::env::set_var("BIOMCP_CTGOV_BASE", base) };
        Self(previous)
    }
}

impl Drop for CtGovAgeMcpEnv {
    fn drop(&mut self) {
        // SAFETY: callers hold the serial-test process-wide environment lock.
        unsafe {
            if let Some(previous) = self.0.take() {
                std::env::set_var("BIOMCP_CTGOV_BASE", previous);
            } else {
                std::env::remove_var("BIOMCP_CTGOV_BASE");
            }
        }
    }
}

#[test]
fn cli_catalog_gettable_inventory_matches_clap_get_subcommands() {
    let catalog = crate::cli::list::catalog::entities()
        .into_iter()
        .filter(|entity| entity.gettable)
        .map(|entity| entity.name)
        .collect::<BTreeSet<_>>();
    let command = crate::cli::Cli::command();
    let clap = command
        .find_subcommand("get")
        .expect("get command")
        .get_subcommands()
        .map(clap::Command::get_name)
        .collect::<BTreeSet<_>>();

    assert_eq!(catalog, clap);
}

#[test]
fn typed_get_schema_and_mapper_match_independent_cli_catalog_oracle() {
    let schema = serde_json::to_value(rmcp::schemars::schema_for!(TypedGet)).unwrap();
    let branches = schema["oneOf"].as_array().expect("typed get branches");
    let catalog = crate::cli::list::catalog::entities()
        .into_iter()
        .filter(|entity| entity.gettable)
        .collect::<Vec<_>>();

    let branch_entities = branches
        .iter()
        .map(|branch| {
            branch["properties"]["entity"]["const"]
                .as_str()
                .expect("branch entity")
        })
        .collect::<BTreeSet<_>>();
    let expected_entities = catalog
        .iter()
        .map(|entity| entity.name)
        .collect::<BTreeSet<_>>();
    assert_eq!(branches.len(), expected_entities.len());
    assert_eq!(branch_entities, expected_entities);

    let article_catalog = crate::cli::list::catalog::sections("article");
    assert!(article_catalog.contains(&"asset"));
    assert_eq!(
        catalog
            .iter()
            .flat_map(|entity| entity
                .sections
                .iter()
                .map(move |section| (entity.name, *section)))
            .filter(|(entity, section)| *entity == "article" && *section == "asset")
            .collect::<Vec<_>>(),
        [("article", "asset")]
    );

    for entity in catalog {
        let branch = branches
            .iter()
            .find(|branch| branch["properties"]["entity"]["const"] == entity.name)
            .unwrap_or_else(|| panic!("missing {} branch", entity.name));
        if entity.name == "author" {
            assert!(branch["properties"].get("sections").is_none());
            continue;
        }

        let expected_sections = entity
            .sections
            .iter()
            .copied()
            .filter(|section| !(entity.name == "article" && *section == "asset"))
            .collect::<BTreeSet<_>>();
        let advertised_sections = branch["properties"]["sections"]["items"]["enum"]
            .as_array()
            .expect("section enum")
            .iter()
            .map(|section| section.as_str().expect("string section"))
            .collect::<BTreeSet<_>>();
        assert_eq!(advertised_sections, expected_sections, "{}", entity.name);

        for section in expected_sections {
            get_args(TypedGet(json!({
                "entity": entity.name,
                "id": "fixture-id",
                "sections": [section]
            })))
            .unwrap_or_else(|error| panic!("{} {section} did not map: {error}", entity.name));
        }
    }

    let assets = get_args(TypedGet(json!({
        "entity": "article",
        "id": "22663011",
        "sections": ["assets"]
    })))
    .expect("article asset manifest remains typed-MCP safe");
    assert_eq!(assets, ["biomcp", "get", "article", "22663011", "assets"]);

    for (entity, section, filename) in [
        ("article", "asset", "fixture.bin"),
        ("trial", "document", "fixture.pdf"),
    ] {
        let error = get_args(TypedGet(json!({
            "entity": entity,
            "id": "fixture-id",
            "sections": [section, filename]
        })))
        .expect_err("typed binary download must be rejected");
        assert!(error.to_string().contains("CLI-only"));
    }

    let trial = branches
        .iter()
        .find(|branch| branch["properties"]["entity"]["const"] == "trial")
        .expect("trial branch");
    let trial_sections = trial["properties"]["sections"]["items"]["enum"]
        .as_array()
        .expect("trial sections");
    assert!(!trial_sections.contains(&json!("document")));
    assert!(!trial_sections.contains(&json!("documents")));
}

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

#[tokio::test]
#[serial_test::serial(source_env)]
async fn typed_and_raw_trial_get_return_exact_age_objects() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let study = json!({"protocolSection": {
        "identificationModule": {"nctId":"NCT60000001","briefTitle":"Infant trial"},
        "statusModule": {"overallStatus":"RECRUITING"},
        "eligibilityModule": {"minimumAge":"6 Months","maximumAge":"N/A"}
    }});
    let router = Router::new().route(
        "/studies/{id}",
        axum_get(move || {
            let study = study.clone();
            async move { Json(study) }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let _env = CtGovAgeMcpEnv::set(&base);

    let typed = BioMcpServer::new()
        .get(rmcp::handler::server::wrapper::Parameters(TypedGet(
            json!({
                "entity":"trial", "id":"NCT60000001", "sections":["eligibility"], "json":true
            }),
        )))
        .await
        .unwrap();
    let raw = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: "biomcp get trial NCT60000001 eligibility".into(),
            json: true,
        }))
        .await
        .unwrap();

    server.abort();
    let response_json = |result| {
        let value = serde_json::to_value(result).unwrap();
        serde_json::from_str::<serde_json::Value>(value["content"][0]["text"].as_str().unwrap())
            .unwrap()
    };
    let typed = response_json(typed);
    let raw = response_json(raw);
    let expected_minimum = json!({"number":6.0,"unit":"months","original":"6 Months"});
    let expected_maximum = json!({"number":null,"unit":null,"original":"N/A"});
    assert_eq!(typed["eligibility"]["minimum_age"], expected_minimum);
    assert_eq!(typed["eligibility"]["maximum_age"], expected_maximum);
    assert_eq!(typed["eligibility"], raw["eligibility"]);
    assert!(!typed["eligibility"]["minimum_age"].is_string());
    assert!(!raw["eligibility"]["maximum_age"].is_string());
}
