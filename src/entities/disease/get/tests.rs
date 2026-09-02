use super::*;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct DiseaseCardFixtureEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl DiseaseCardFixtureEnv {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn set(&mut self, key: &'static str, value: &str) {
        self.0.push((key, std::env::var_os(key)));
        // SAFETY: this test holds the serial-test process-wide environment lock.
        unsafe { std::env::set_var(key, value) };
    }
}

impl Drop for DiseaseCardFixtureEnv {
    fn drop(&mut self) {
        for (key, previous) in self.0.drain(..).rev() {
            // SAFETY: this test holds the serial-test process-wide environment lock.
            unsafe {
                if let Some(value) = previous {
                    std::env::set_var(key, value);
                } else {
                    std::env::remove_var(key);
                }
            }
        }
    }
}

fn disease_card_fixture_response(body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

async fn disease_card_fixture_server()
-> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind disease card fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = captured.clone();
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let len = stream
                    .read(&mut request)
                    .await
                    .expect("read fixture request");
                let request = String::from_utf8_lossy(&request[..len]).into_owned();
                captured
                    .lock()
                    .expect("lock fixture requests")
                    .push(request.clone());
                let body = if request.starts_with("GET /query?") {
                    r#"{"total":1,"hits":[{"_id":"MONDO:0007959","mondo":{"name":"medulloblastoma"}}]}"#
                } else if request.starts_with("GET /disease/MONDO:0007959") {
                    r#"{"_id":"MONDO:0007959","mondo":{"synonym":["cerebellum embryonal neoplasm"]}}"#
                } else if request.contains("query.cond=Medulloblastoma") {
                    r#"{"studies":[],"totalCount":36}"#
                } else {
                    r#"{"studies":[],"totalCount":0}"#
                };
                stream
                    .write_all(&disease_card_fixture_response(body))
                    .await
                    .expect("write fixture response");
            });
        }
    });
    (base, requests, task)
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn disease_card_keeps_the_resolving_term_when_detail_label_is_missing() {
    let (base, requests, server) = disease_card_fixture_server().await;
    let mut env = DiseaseCardFixtureEnv::new();
    env.set("BIOMCP_MYDISEASE_BASE", &base);
    env.set("BIOMCP_CTGOV_BASE", &base);
    env.set("BIOMCP_OLS4_BASE", "://unavailable-ols-fixture");
    env.set("BIOMCP_MYCHEM_BASE", "://unavailable-mychem-fixture");
    env.set(
        "BIOMCP_OPENTARGETS_BASE",
        "://unavailable-opentargets-fixture",
    );

    let card = crate::cli::execute(vec![
        "biomcp".to_string(),
        "get".to_string(),
        "disease".to_string(),
        "Medulloblastoma".to_string(),
    ])
    .await
    .expect("resolved disease card");
    server.abort();

    // The resolving hit's label is lowercase. These expectations require the
    // caller's differently cased term to survive the detail fetch.
    assert!(card.starts_with("# Medulloblastoma\n"));
    assert!(card.contains("Recruiting Trials (ClinicalTrials.gov): 36"));
    assert!(card.contains("Disease label unavailable; using the requested term."));
    for command in [
        "biomcp search trial -c \"Medulloblastoma\"",
        "biomcp search article -d \"Medulloblastoma\"",
        "biomcp search diagnostic --disease \"Medulloblastoma\"",
        "biomcp search drug --indication \"Medulloblastoma\"",
    ] {
        assert!(card.contains(command));
    }
    let requests = requests.lock().expect("lock fixture requests").join("\n");
    assert!(requests.contains("query.cond=Medulloblastoma"));
}

#[test]
fn parse_sections_supports_new_disease_sections() {
    let flags = parse_sections(&[
        "phenotypes".to_string(),
        "diagnostics".to_string(),
        "variants".to_string(),
        "models".to_string(),
        "prevalence".to_string(),
        "survival".to_string(),
        "funding".to_string(),
        "disgenet".to_string(),
        "all".to_string(),
    ])
    .expect("sections should parse");
    assert!(flags.include_genes);
    assert!(flags.include_pathways);
    assert!(flags.include_phenotypes);
    assert!(flags.include_diagnostics);
    assert!(flags.include_variants);
    assert!(flags.include_models);
    assert!(flags.include_prevalence);
    assert!(flags.include_survival);
    assert!(flags.include_funding);
    assert!(flags.include_civic);
    assert!(flags.include_disgenet);
    assert!(!flags.include_clinical_features);
}

#[test]
fn disease_parse_sections_accepts_diagnostics() {
    let flags = parse_sections(&["diagnostics".to_string()]).expect("diagnostics should parse");
    assert!(flags.include_diagnostics);
    assert!(!flags.include_genes);
    assert!(!flags.include_funding);
    assert!(!flags.include_disgenet);
    assert!(!flags.include_clinical_features);
}

#[test]
fn parse_sections_accepts_clinical_features() {
    let flags =
        parse_sections(&["clinical_features".to_string()]).expect("clinical_features should parse");
    assert!(flags.include_clinical_features);
    assert!(!flags.include_genes);
    assert!(!flags.include_pathways);
    assert!(!flags.include_phenotypes);
    assert!(!flags.include_diagnostics);
    assert!(!flags.include_variants);
    assert!(!flags.include_models);
    assert!(!flags.include_prevalence);
    assert!(!flags.include_survival);
    assert!(!flags.include_funding);
    assert!(!flags.include_civic);
    assert!(!flags.include_disgenet);
}

#[test]
fn parse_sections_all_keeps_optional_sections_opt_in() {
    let flags = parse_sections(&["all".to_string()]).expect("sections should parse");
    assert!(flags.include_survival);
    assert!(!flags.include_diagnostics);
    assert!(!flags.include_funding);
    assert!(!flags.include_disgenet);
    assert!(!flags.include_clinical_features);
}

#[test]
fn disease_parse_sections_all_keeps_diagnostics_opt_in() {
    let flags = parse_sections(&["all".to_string()]).expect("sections should parse");
    assert!(!flags.include_diagnostics);
}

#[test]
fn parse_sections_unknown_section_lists_clinical_features() {
    let err =
        parse_sections(&["not_a_section".to_string()]).expect_err("unknown section should fail");
    assert!(err.to_string().contains("clinical_features"));
}

#[test]
fn parse_sections_unknown_value_suggests_name_flag_for_multi_word_diseases() {
    let err = parse_sections_for_name(
        "chronic",
        &[
            "myeloid".to_string(),
            "leukemia".to_string(),
            "survival".to_string(),
        ],
    )
    .expect_err("ambiguous multi-word disease should fail with guidance");
    let message = err.to_string();
    assert!(message.contains("Unknown section \"myeloid\" for disease"));
    assert!(message.contains("--name \"chronic myeloid leukemia\" survival"));
}

#[test]
fn get_disease_preserves_canonical_mondo_lookup_path() {
    let plan = crate::sources::mydisease::MyDiseaseClient::get_plan("MONDO:0005105")
        .expect("canonical get plan");

    assert_eq!(plan.method, crate::sources::HttpMethod::Get);
    assert_eq!(plan.path, "disease/MONDO:0005105");
    assert!(plan.query.contains(&(
        "fields".to_string(),
        crate::sources::mydisease::MYDISEASE_GET_FIELDS.to_string()
    )));
}

#[test]
fn get_disease_resolves_mesh_and_omim_crosswalk_ids_before_fetch() {
    let mesh = crate::sources::mydisease::MyDiseaseClient::lookup_disease_by_xref_plan(
        "mesh", "D008545", 5,
    )
    .expect("mesh xref plan");
    assert_eq!(mesh.path, "query");
    assert!(mesh.query.contains(&(
        "q".to_string(),
        "(mondo.xrefs.mesh:\"D008545\" OR disease_ontology.xrefs.mesh:\"D008545\" OR umls.mesh:\"D008545\")".to_string(),
    )));

    let omim = crate::sources::mydisease::MyDiseaseClient::lookup_disease_by_xref_plan(
        "omim", "154700", 5,
    )
    .expect("omim xref plan");
    assert!(omim.query.contains(&(
        "q".to_string(),
        "(mondo.xrefs.omim:\"154700\" OR disease_ontology.xrefs.omim:\"154700\")".to_string(),
    )));
}

#[test]
fn get_disease_returns_not_found_for_unresolved_crosswalk_without_name_fallback() {
    assert!(preferred_crosswalk_hit(Vec::new()).is_none());
}
