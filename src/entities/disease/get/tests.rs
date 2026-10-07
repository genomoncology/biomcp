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
                    // The embedded DisGeNET block seeds the card's genes
                    // (ticket 1256): with Open Targets unreachable these
                    // genes must carry DisGeNET's name, not Open Targets'.
                    r#"{"_id":"MONDO:0007959","mondo":{"synonym":["cerebellum embryonal neoplasm"]},"disgenet":{"genes_related_to_disease":[{"gene_symbol":"PIK3CA","score":0.7}]}}"#
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

/// A fixture that answers every request with a fresh-cacheable 200:
/// one-second freshness window, so a later fetch past that window with
/// the server gone is a stale serve.
async fn stale_serve_fixture_server()
-> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind stale-serve fixture");
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
                    // Same DisGeNET seed as the card fixture: the stale
                    // card must still credit DisGeNET for its genes, so
                    // the fixture has to seed them (ticket 1256).
                    r#"{"_id":"MONDO:0007959","mondo":{"synonym":["cerebellum embryonal neoplasm"]},"disgenet":{"genes_related_to_disease":[{"gene_symbol":"PIK3CA","score":0.7}]}}"#
                } else if request.contains("query.cond=Medulloblastoma") {
                    r#"{"studies":[],"totalCount":36}"#
                } else {
                    r#"{"studies":[],"totalCount":0}"#
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nCache-Control: max-age=1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write fixture response");
            });
        }
    });
    (base, requests, task)
}

fn stale_serve_env(env: &mut DiseaseCardFixtureEnv, cache_root: &std::path::Path, base: &str) {
    env.set(
        "BIOMCP_CACHE_DIR",
        cache_root.to_str().expect("utf-8 cache root"),
    );
    env.set("BIOMCP_MYDISEASE_BASE", base);
    env.set("BIOMCP_CTGOV_BASE", base);
    env.set("BIOMCP_OLS4_BASE", "://unavailable-ols-fixture");
    env.set("BIOMCP_MYCHEM_BASE", "://unavailable-mychem-fixture");
    env.set(
        "BIOMCP_OPENTARGETS_BASE",
        "://unavailable-opentargets-fixture",
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_stale_disease_card_tells_the_clinician_the_cache_age() {
    let (base, _requests, server) = stale_serve_fixture_server().await;
    let root = crate::test_support::TempDirGuard::new("stale-card-cache");
    let mut env = DiseaseCardFixtureEnv::new();
    stale_serve_env(&mut env, root.path(), &base);

    let first = crate::cli::execute(vec![
        "biomcp".to_string(),
        "get".to_string(),
        "disease".to_string(),
        "Medulloblastoma".to_string(),
    ])
    .await
    .expect("fresh disease card");
    assert!(
        !first.contains("Cache note:"),
        "a fresh serve carries no cache note: {first}"
    );

    tokio::time::sleep(std::time::Duration::from_millis(1600)).await; // watchdog: freshness-window wait, bounded at 1.6 s
    server.abort();

    let stale = crate::cli::execute(vec![
        "biomcp".to_string(),
        "get".to_string(),
        "disease".to_string(),
        "Medulloblastoma".to_string(),
    ])
    .await
    .expect("stale disease card");
    assert!(
        stale.contains("Cache note: MyDisease.info data served from cache,"),
        "the stale card states the provider and the cache fact: {stale}"
    );
    assert!(
        stale.contains("older than the provider's freshness window)."),
        "the note keeps the log line's honest wording: {stale}"
    );
    assert!(
        stale.contains("Genes (DisGeNET): PIK3CA"),
        "with Open Targets unreachable, the seeded genes carry DisGeNET's name, not Open Targets': {stale}"
    );
    assert!(
        !stale.contains("Genes (Open Targets)"),
        "Open Targets must not be credited for another source's genes: {stale}"
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn stale_search_json_carries_the_cache_age_in_the_meta_notes() {
    let (base, _requests, server) = stale_serve_fixture_server().await;
    let root = crate::test_support::TempDirGuard::new("stale-search-cache");
    let mut env = DiseaseCardFixtureEnv::new();
    stale_serve_env(&mut env, root.path(), &base);

    let first = crate::cli::execute(vec![
        "biomcp".to_string(),
        "search".to_string(),
        "disease".to_string(),
        "-q".to_string(),
        "Medulloblastoma".to_string(),
        "--json".to_string(),
    ])
    .await
    .expect("fresh search json");
    assert!(
        !first.contains("served from cache"),
        "a fresh serve carries no stale note: {first}"
    );

    tokio::time::sleep(std::time::Duration::from_millis(1600)).await; // watchdog: freshness-window wait, bounded at 1.6 s
    server.abort();

    let stale = crate::cli::execute(vec![
        "biomcp".to_string(),
        "search".to_string(),
        "disease".to_string(),
        "-q".to_string(),
        "Medulloblastoma".to_string(),
        "--json".to_string(),
    ])
    .await
    .expect("stale search json");
    let body: serde_json::Value = serde_json::from_str(&stale).expect("search json parses");
    let notes = body
        .pointer("/_meta/notes")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("_meta.notes must exist on a stale search: {stale}"));
    let joined = notes
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("MyDisease.info data served from cache,"),
        "the JSON consumer sees the provider and the cache fact: {joined}"
    );
    assert!(
        joined.contains("older than the provider's freshness window)."),
        "the note keeps the log line's honest wording: {joined}"
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_stale_mcp_call_tells_the_clinician_the_cache_age_on_both_channels() {
    // Ticket 1256, review P0-3: the MCP path drives the alias arm of
    // run_outcome_with_worker_stack, which once called run_outcome_inner
    // directly — outside the stale-serve scope — so neither the card note
    // nor the JSON _meta.notes sentence could ever appear over MCP. This
    // test drives execute_mcp (the MCP entry) against the stale fixture
    // and proves both channels.
    let (base, _requests, server) = stale_serve_fixture_server().await;
    let root = crate::test_support::TempDirGuard::new("stale-mcp-cache");
    let mut env = DiseaseCardFixtureEnv::new();
    stale_serve_env(&mut env, root.path(), &base);

    let fresh_card = crate::cli::execute_mcp(vec![
        "biomcp".to_string(),
        "get".to_string(),
        "disease".to_string(),
        "Medulloblastoma".to_string(),
    ])
    .await
    .expect("fresh mcp card");
    assert!(
        !fresh_card.text.contains("Cache note:"),
        "a fresh MCP serve carries no cache note: {}",
        fresh_card.text
    );
    let fresh_json = crate::cli::execute_mcp(vec![
        "biomcp".to_string(),
        "search".to_string(),
        "disease".to_string(),
        "-q".to_string(),
        "Medulloblastoma".to_string(),
        "--json".to_string(),
    ])
    .await
    .expect("fresh mcp search json");
    assert!(
        !fresh_json.text.contains("served from cache"),
        "a fresh MCP serve carries no stale note: {}",
        fresh_json.text
    );

    tokio::time::sleep(std::time::Duration::from_millis(1600)).await; // watchdog: freshness-window wait, bounded at 1.6 s
    server.abort();

    let stale_card = crate::cli::execute_mcp(vec![
        "biomcp".to_string(),
        "get".to_string(),
        "disease".to_string(),
        "Medulloblastoma".to_string(),
    ])
    .await
    .expect("stale mcp card");
    assert!(
        stale_card
            .text
            .contains("Cache note: MyDisease.info data served from cache,"),
        "the MCP card states the provider and the cache fact: {}",
        stale_card.text
    );
    assert!(
        stale_card
            .text
            .contains("older than the provider's freshness window)."),
        "the MCP note keeps the log line's honest wording: {}",
        stale_card.text
    );

    let stale_json = crate::cli::execute_mcp(vec![
        "biomcp".to_string(),
        "search".to_string(),
        "disease".to_string(),
        "-q".to_string(),
        "Medulloblastoma".to_string(),
        "--json".to_string(),
    ])
    .await
    .expect("stale mcp search json");
    let body: serde_json::Value =
        serde_json::from_str(&stale_json.text).expect("mcp search json parses");
    let notes = body
        .pointer("/_meta/notes")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("_meta.notes must exist over MCP: {}", stale_json.text));
    let joined = notes
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("MyDisease.info data served from cache,"),
        "the MCP JSON consumer sees the provider and the cache fact: {joined}"
    );
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

macro_rules! recorded_mydisease {
    ($name:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/mydisease/",
            $name
        ))
    };
}

/// Ticket 2017: the recorded MyDisease responses for the ambiguous
/// abbreviations. The fixture answers only the recorded queries, so the
/// refusals cannot pass vacuously.
async fn abbreviation_fixture_server()
-> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind abbreviation fixture");
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
                let body = if request.contains("/query?") {
                    if request.contains("MF") {
                        recorded_mydisease!("query_mf.json")
                    } else if request.contains("MM") {
                        recorded_mydisease!("query_mm.json")
                    } else if request.contains("MDS") {
                        recorded_mydisease!("query_mds.json")
                    } else if request.contains("CAD") {
                        recorded_mydisease!("query_cad.json")
                    } else if request.contains("CRC") {
                        recorded_mydisease!("query_crc.json")
                    } else {
                        r#"{"total":0,"hits":[]}"#
                    }
                } else if request.contains("/disease/MONDO:0024331") {
                    recorded_mydisease!("get_mondo_0024331.json")
                } else {
                    r#"{"total":0,"hits":[]}"#
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

async fn get_disease_card_or_error(query: &str) -> Result<String, String> {
    let (base, requests, server) = abbreviation_fixture_server().await;
    let mut env = DiseaseCardFixtureEnv::new();
    env.set("BIOMCP_MYDISEASE_BASE", &base);
    env.set("BIOMCP_CTGOV_BASE", &base);
    env.set("BIOMCP_OLS4_BASE", "://unavailable-ols-fixture");
    env.set("BIOMCP_MYCHEM_BASE", "://unavailable-mychem-fixture");
    env.set(
        "BIOMCP_OPENTARGETS_BASE",
        "://unavailable-opentargets-fixture",
    );
    let outcome = crate::cli::execute(vec![
        "biomcp".to_string(),
        "get".to_string(),
        "disease".to_string(),
        query.to_string(),
    ])
    .await;
    server.abort();
    drop(env);
    outcome.map_err(|err| err.to_string()).map(|text| {
        text + "\n<<requests>>\n" + &requests.lock().expect("lock fixture requests").join("\n")
    })
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn get_disease_mf_refuses_instead_of_mixing_two_diseases() {
    let error = get_disease_card_or_error("MF")
        .await
        .expect_err("MF holds on two diseases and must refuse");
    assert!(
        error.contains("Ambiguous disease abbreviation 'MF'"),
        "{error}"
    );
    assert!(
        error.contains("mycosis fungoides (MONDO:0009691)"),
        "{error}"
    );
    assert!(error.contains("MONDO:0020481"), "{error}");
    assert!(error.contains("search disease"), "{error}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn get_disease_cad_refuses_naming_every_exact_holder() {
    let error = get_disease_card_or_error("CAD")
        .await
        .expect_err("CAD holds on three diseases and must refuse");
    for candidate in [
        "coronary artery disease (MONDO:0005010)",
        "MONDO:0018922",
        "MONDO:0100077",
    ] {
        assert!(error.contains(candidate), "missing {candidate}: {error}");
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn get_disease_mm_refuses_even_with_one_source_holder() {
    let error = get_disease_card_or_error("MM")
        .await
        .expect_err("a two-letter abbreviation must not resolve to Miyoshi");
    assert!(
        error.contains("Miyoshi muscular dystrophy (MONDO:0009685)"),
        "{error}"
    );
    assert!(
        error.contains("a token this short cannot name one disease reliably"),
        "{error}"
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn get_disease_mds_refuses_naming_both_holders() {
    let error = get_disease_card_or_error("MDS")
        .await
        .expect_err("MDS holds on two diseases and must refuse");
    assert!(
        error.contains("myelodysplastic syndrome (MONDO:0018881)"),
        "{error}"
    );
    assert!(
        error.contains("Miller-Dieker lissencephaly syndrome (MONDO:0009532)"),
        "{error}"
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn get_disease_single_holder_abbreviation_still_resolves_one_record() {
    let card = get_disease_card_or_error("CRC")
        .await
        .expect("CRC is held by one disease and must resolve");
    assert!(card.starts_with("# colorectal carcinoma"), "{card}");
    assert!(card.contains("ID: MONDO:0024331"), "{card}");
}

/// Ticket 2017 structural guard: when a single-holder abbreviation resolves
/// to a record without a label, the requested abbreviation stays display-only.
/// The identity joins (Open Targets) must query the record's own terms, never
/// the abbreviation — that leak is what mixed mycosis fungoides genes into
/// the Myotonia fluctuans card.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn get_disease_alias_lookup_does_not_feed_the_abbreviation_to_open_targets() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind alias fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = captured.clone();
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let len = stream.read(&mut request).await.expect("read request");
                let request = String::from_utf8_lossy(&request[..len]).into_owned();
                captured
                    .lock()
                    .expect("lock alias fixture requests")
                    .push(request.clone());
                // ZQX appears only in the initial resolution query; every
                // later request must answer without it.
                let body = if request.starts_with("GET /query?") && request.contains("ZQX") {
                    r#"{"total":1,"hits":[{"_id":"MONDO:0900001","mondo":{"synonym":{"exact":["ZQX"]}}}]}"#
                } else if request.contains("/disease/MONDO:0900001") {
                    // No name: the record is the abbreviation's unlabelled
                    // single holder.
                    r#"{"_id":"MONDO:0900001","mondo":{"synonym":{"exact":["ZQX"]}},"disease_ontology":{}}"#
                } else {
                    r#"{"total":0,"hits":[]}"#
                };
                stream
                    .write_all(&disease_card_fixture_response(body))
                    .await
                    .expect("write fixture response");
            });
        }
    });
    let mut env = DiseaseCardFixtureEnv::new();
    env.set("BIOMCP_MYDISEASE_BASE", &base);
    env.set("BIOMCP_CTGOV_BASE", &base);
    env.set("BIOMCP_OLS4_BASE", &base);
    env.set("BIOMCP_MYCHEM_BASE", &base);
    env.set("BIOMCP_OPENTARGETS_BASE", &base);
    let card = crate::cli::execute(vec![
        "biomcp".to_string(),
        "get".to_string(),
        "disease".to_string(),
        "ZQX".to_string(),
    ])
    .await
    .expect("single-holder abbreviation resolves");
    task.abort();
    drop(env);

    assert!(card.contains("ID: MONDO:0900001"), "{card}");
    let requests = requests
        .lock()
        .expect("lock alias fixture requests")
        .join("\n");
    // The resolution query repeats the token once per Lucene clause; count
    // requests, not occurrences.
    let querying_requests = requests.lines().filter(|line| line.contains("ZQX")).count();
    assert!(
        querying_requests == 1,
        "the abbreviation may appear only in the resolution query: {requests}"
    );
}
