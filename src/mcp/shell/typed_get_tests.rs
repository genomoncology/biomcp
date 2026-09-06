use std::collections::BTreeSet;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{Response, StatusCode, Uri, header},
    routing::get as axum_get,
};
use clap::CommandFactory;
use serde_json::json;

use super::{BioMcpServer, ShellCommand, TypedGet, TypedVariantErepo, get_args};

#[test]
fn shared_mcp_error_conversion_hides_trial_design_details() {
    let relationship = biodata::ClinicalTrialArmRelationshipError::MissingArmEndpoint {
        arm_id: biodata::ClinicalTrialArmId::new(42).unwrap(),
    };
    let error = crate::error::BioMcpError::TrialDesign(
        crate::error::TrialDesignError::InvalidRelationship(relationship),
    );
    let value = serde_json::to_value(BioMcpServer::tool_error(format!("Error: {error}")))
        .expect("serialize MCP error");

    assert_eq!(
        value["content"][0]["text"],
        "Error: Internal processing failed."
    );
    assert!(!value.to_string().contains("MissingArmEndpoint"));
    assert!(!value.to_string().contains("42"));
}

struct CtGovAgeMcpEnv(Option<std::ffi::OsString>);

impl CtGovAgeMcpEnv {
    fn set(base: &str) -> Self {
        let previous = std::env::var_os("BIOMCP_CTGOV_BASE");
        // SAFETY: callers hold the serial-test process-wide environment lock.
        unsafe { std::env::set_var("BIOMCP_CTGOV_BASE", base) };
        Self(previous)
    }
}

struct NciTrialMcpEnv {
    previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    _cache: tempfile::TempDir,
}

impl NciTrialMcpEnv {
    fn set(base: &str) -> Self {
        let cache = tempfile::tempdir().expect("NCI MCP cache directory");
        let values = [
            ("BIOMCP_NCI_CTS_BASE", base.to_string()),
            ("NCI_API_KEY", "fixture-key".to_string()),
            (
                "BIOMCP_CACHE_DIR",
                cache.path().to_string_lossy().into_owned(),
            ),
            ("BIOMCP_TEST_UNPACED_ORIGIN", base.to_string()),
        ];
        let mut previous = Vec::new();
        for (name, value) in values {
            previous.push((name, std::env::var_os(name)));
            // SAFETY: callers hold the serial-test process-wide environment lock.
            unsafe { std::env::set_var(name, value) };
        }
        Self {
            previous,
            _cache: cache,
        }
    }
}

impl Drop for NciTrialMcpEnv {
    fn drop(&mut self) {
        // SAFETY: callers hold the serial-test process-wide environment lock.
        unsafe {
            for (name, previous) in self.previous.drain(..) {
                if let Some(previous) = previous {
                    std::env::set_var(name, previous);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
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

struct ClinvarMcpEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl ClinvarMcpEnv {
    fn set(base: &str) -> Self {
        let values = [
            ("BIOMCP_MYVARIANT_BASE", format!("{base}/v1")),
            ("BIOMCP_CLINVAR_BASE", format!("{base}/eutils")),
            ("BIOMCP_GNOMAD_BASE", format!("{base}/unavailable")),
            ("BIOMCP_DBSNP_BASE", format!("{base}/unavailable")),
            ("BIOMCP_CBIOPORTAL_BASE", format!("{base}/unavailable")),
            ("BIOMCP_CANCERHOTSPOTS_BASE", format!("{base}/unavailable")),
            ("BIOMCP_CIVIC_BASE", format!("{base}/unavailable")),
            ("BIOMCP_GWAS_BASE", format!("{base}/unavailable")),
        ];
        let mut previous = Vec::new();
        for (name, value) in values {
            previous.push((name, std::env::var_os(name)));
            // SAFETY: callers hold the serial-test process-wide environment lock.
            unsafe { std::env::set_var(name, value) };
        }
        Self(previous)
    }
}

impl Drop for ClinvarMcpEnv {
    fn drop(&mut self) {
        // SAFETY: callers hold the serial-test process-wide environment lock.
        unsafe {
            for (name, previous) in self.0.drain(..) {
                if let Some(previous) = previous {
                    std::env::set_var(name, previous);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }
}

#[derive(Clone)]
struct ClinvarMcpFixture {
    requests: Arc<AtomicUsize>,
}

fn fixture_response(
    status: StatusCode,
    content_type: &'static str,
    body: String,
) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(body))
        .expect("fixture response")
}

async fn clinvar_mcp_fixture(State(state): State<ClinvarMcpFixture>, uri: Uri) -> Response<Body> {
    let request = uri.to_string();
    if request.starts_with("/v1/query") {
        let (rsid, clinvar) = if request.contains("rs1154003") {
            ("rs1154003", serde_json::Value::Null)
        } else if request.contains("rs1154002") {
            (
                "rs1154002",
                json!({"variant_id":1154002,"rcv":{"accession":"RCV001154002","version":4,"clinical_significance":"Pathogenic","last_evaluated":"2024-02-03","number_submitters":3}}),
            )
        } else if request.contains("rs1154004") {
            (
                "rs1154004",
                json!({"variant_id":1154004,"rcv":{"accession":"RCV001154004","version":1,"clinical_significance":"Uncertain significance","last_evaluated":"2023-01-01","number_submitters":1}}),
            )
        } else {
            (
                "rs1154001",
                json!({"variant_id":1154001,"rcv":{"accession":"RCV001154001","version":2,"clinical_significance":"Likely pathogenic","last_evaluated":"2020-08-04","number_submitters":1}}),
            )
        };
        let mut hit = json!({
            "_id":"chr5:g.118860951A>G",
            "dbsnp":{"rsid":rsid}
        });
        if !clinvar.is_null() {
            hit["clinvar"] = clinvar;
        }
        return fixture_response(
            StatusCode::OK,
            "application/json",
            json!({"hits":[hit]}).to_string(),
        );
    }
    if request.starts_with("/eutils/efetch.fcgi") {
        state.requests.fetch_add(1, Ordering::SeqCst);
        if request.contains("id=1154002") {
            return fixture_response(StatusCode::OK, "text/html", "<html/>".into());
        }
        let id = if request.contains("id=1154004") {
            1_154_004
        } else {
            1_154_001
        };
        let extra = usize::from(id == 1_154_004);
        let criteria = "c".repeat(32 * 1024 + extra);
        let xml = format!(
            "<ClinVarResult-Set><VariationArchive VariationID=\"{id}\" Accession=\"VCV{id:09}\" Version=\"1\"><RecordStatus>current</RecordStatus><ClassifiedRecord><RCVList><RCVAccession Accession=\"RCV{id:09}\" Version=\"1\"><RCVClassifications><GermlineClassification><Description>Pathogenic</Description></GermlineClassification></RCVClassifications></RCVAccession></RCVList><ClinicalAssertionList><ClinicalAssertion ID=\"{id}\" ContributesToAggregateClassification=\"true\"><ClinVarAccession Accession=\"SCV{id:09}\" Version=\"1\" SubmitterName=\"Fixture Lab\"/><RecordStatus>current</RecordStatus><Classification><GermlineClassification>Pathogenic</GermlineClassification></Classification><AttributeSet><Attribute Type=\"AssertionMethod\">{criteria}</Attribute></AttributeSet></ClinicalAssertion></ClinicalAssertionList></ClassifiedRecord></VariationArchive></ClinVarResult-Set>"
        );
        return fixture_response(StatusCode::OK, "application/xml", xml);
    }
    fixture_response(StatusCode::NOT_FOUND, "application/json", "{}".into())
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
        "eligibilityModule": {"minimumAge":"6 Months","maximumAge":"N/A"},
        "armsInterventionsModule": {
            "armGroups": [{"label":"Arm one","interventionNames":["Drug: first drug"]}],
            "interventions": [{"type":"DRUG","name":"first drug","armGroupLabels":["Arm one"]}]
        }
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
                "entity":"trial", "id":"NCT60000001", "sections":["eligibility", "arms"], "json":true
            }),
        )))
        .await
        .unwrap();
    let raw = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: "biomcp get trial NCT60000001 eligibility arms".into(),
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
    assert_eq!(typed["interventions"], raw["interventions"]);
    assert_eq!(typed["arms"], raw["arms"]);
    assert_eq!(
        typed["arm_intervention_assignments"],
        raw["arm_intervention_assignments"]
    );
    assert_eq!(
        typed["arm_intervention_assignments"],
        json!([{"arm_id":1,"intervention_id":1}])
    );
    assert!(!typed["eligibility"]["minimum_age"].is_string());
    assert!(!raw["eligibility"]["maximum_age"].is_string());
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn typed_and_raw_nci_trial_get_preserve_all_recorded_assignments() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let bytes =
        include_bytes!("../../../testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json");
    let router = Router::new().route(
        "/trials",
        axum_get(move || async move {
            Response::builder()
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(bytes.as_slice()))
                .unwrap()
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let prior_env = [
        "BIOMCP_NCI_CTS_BASE",
        "NCI_API_KEY",
        "BIOMCP_CACHE_DIR",
        "BIOMCP_TEST_UNPACED_ORIGIN",
    ]
    .map(|name| (name, std::env::var_os(name)));
    let env = NciTrialMcpEnv::set(&base);

    let typed = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        BioMcpServer::new().get(rmcp::handler::server::wrapper::Parameters(TypedGet(
            json!({
                "entity":"trial", "id":"NCT05879926", "source":"nci",
                "sections":["all"], "json":true
            }),
        ))),
    )
    .await
    .expect("typed NCI get timeout")
    .unwrap();
    let raw = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        BioMcpServer::new().biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: "biomcp get trial NCT05879926 --source nci all".into(),
            json: true,
        })),
    )
    .await
    .expect("raw NCI get timeout")
    .unwrap();

    server.abort();
    let response_json = |result| {
        let value = serde_json::to_value(result).unwrap();
        serde_json::from_str::<serde_json::Value>(value["content"][0]["text"].as_str().unwrap())
            .unwrap()
    };
    let typed = response_json(typed);
    let raw = response_json(raw);
    assert_eq!(typed["interventions"], raw["interventions"]);
    assert_eq!(typed["arms"], raw["arms"]);
    assert_eq!(
        typed["arm_intervention_assignments"],
        raw["arm_intervention_assignments"]
    );
    assert_eq!(typed["interventions"].as_array().map(Vec::len), Some(53));
    assert_eq!(
        typed["arm_intervention_assignments"]
            .as_array()
            .map(Vec::len),
        Some(53)
    );
    drop(env);
    for (name, previous) in prior_env {
        assert_eq!(std::env::var_os(name), previous);
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn clinvar_override_is_consistent_across_request_modes_and_mcp_surfaces() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(AtomicUsize::new(0));
    let router = Router::new()
        .fallback(clinvar_mcp_fixture)
        .with_state(ClinvarMcpFixture {
            requests: Arc::clone(&requests),
        });
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let _env = ClinvarMcpEnv::set(&base);

    let default = crate::entities::variant::get("rs1154001", &[])
        .await
        .expect("default variant");
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    assert!(default.clinvar.is_none());

    let explicit = crate::entities::variant::get("rs1154001", &["clinvar".into()])
        .await
        .expect("explicit ClinVar");
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    let explicit_json = serde_json::to_value(&explicit).expect("variant JSON");
    assert_eq!(
        explicit_json["section_outcomes"]["clinvar"]["outcome"],
        "data"
    );
    assert_eq!(
        explicit_json["section_outcomes"]["clinvar"]["sources"],
        json!(["NCBI ClinVar"])
    );
    assert_eq!(
        explicit_json["clinvar"]["submissions"][0]["criteria"]
            .as_str()
            .expect("criteria at exact boundary")
            .len(),
        32 * 1024
    );

    crate::entities::variant::get("rs1154001", &["all".into()])
        .await
        .expect("all sections");
    assert_eq!(requests.load(Ordering::SeqCst), 2);

    let over_boundary = crate::entities::variant::get("rs1154004", &["clinvar".into()])
        .await
        .expect("over-boundary direct record degrades");
    let over_json = serde_json::to_value(over_boundary).expect("over-boundary JSON");
    assert_eq!(
        over_json["section_outcomes"]["clinvar"]["outcome"],
        "degraded"
    );
    assert_eq!(over_json["clinvar"]["source"], "MyVariant.info");

    let json_result = |result| {
        let value = serde_json::to_value(result).expect("MCP result JSON");
        serde_json::from_str::<serde_json::Value>(
            value["content"][0]["text"].as_str().expect("MCP JSON text"),
        )
        .expect("MCP response body")
    };
    let fallback_entity = crate::entities::variant::get("rs1154002", &["clinvar".into()])
        .await
        .expect("fallback JSON");
    let fallback_json = serde_json::to_value(fallback_entity).expect("fallback entity JSON");
    let fallback_raw = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: "biomcp get variant rs1154002 clinvar".into(),
            json: true,
        }))
        .await
        .map(json_result)
        .expect("raw MCP fallback");
    let fallback_typed = BioMcpServer::new()
        .get(rmcp::handler::server::wrapper::Parameters(TypedGet(
            json!({
                "entity":"variant", "id":"rs1154002", "sections":["clinvar"], "json":true
            }),
        )))
        .await
        .map(json_result)
        .expect("typed MCP fallback");
    for value in [&fallback_json, &fallback_raw, &fallback_typed] {
        assert_eq!(value["clinvar"]["source"], "MyVariant.info");
        assert_eq!(value["clinvar"]["aggregates"][0]["version"], 4);
        assert_eq!(
            value["clinvar"]["aggregates"][0]["evaluation_date"],
            "2024-02-03"
        );
        assert_eq!(value["clinvar"]["aggregates"][0]["number_submitters"], 3);
        assert_eq!(value["section_outcomes"]["clinvar"]["outcome"], "degraded");
        assert_eq!(
            value["section_outcomes"]["clinvar"]["sources"],
            json!(["MyVariant.info"])
        );
        if let Some(meta) = value.get("_meta") {
            let row = meta["section_sources"]
                .as_array()
                .and_then(|rows| rows.iter().find(|row| row["key"] == "clinvar"))
                .expect("MCP ClinVar provenance");
            assert_eq!(row["outcome"], "degraded");
            assert_eq!(row["sources"], json!(["MyVariant.info"]));
        }
    }

    let before_inapplicable = requests.load(Ordering::SeqCst);
    let inapplicable_entity = crate::entities::variant::get("rs1154003", &["clinvar".into()])
        .await
        .expect("inapplicable JSON");
    let inapplicable_json = serde_json::to_value(inapplicable_entity).expect("entity JSON");
    let inapplicable_raw = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: "biomcp get variant rs1154003 clinvar".into(),
            json: true,
        }))
        .await
        .map(json_result)
        .expect("raw MCP inapplicable");
    let inapplicable_typed = BioMcpServer::new()
        .get(rmcp::handler::server::wrapper::Parameters(TypedGet(
            json!({
                "entity":"variant", "id":"rs1154003", "sections":["clinvar"], "json":true
            }),
        )))
        .await
        .map(json_result)
        .expect("typed MCP inapplicable");
    assert_eq!(requests.load(Ordering::SeqCst), before_inapplicable);
    for value in [&inapplicable_json, &inapplicable_raw, &inapplicable_typed] {
        assert!(value.get("clinvar").is_none());
        assert_eq!(
            value["section_outcomes"]["clinvar"]["outcome"],
            "inapplicable"
        );
        assert_eq!(value["section_outcomes"]["clinvar"]["sources"], json!([]));
        if let Some(meta) = value.get("_meta") {
            let row = meta["section_sources"]
                .as_array()
                .and_then(|rows| rows.iter().find(|row| row["key"] == "clinvar"))
                .expect("MCP ClinVar provenance");
            assert_eq!(row["outcome"], "inapplicable");
            assert_eq!(row["sources"], json!([]));
        }
    }

    server.abort();
}
