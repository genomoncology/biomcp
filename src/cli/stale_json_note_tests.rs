//! End-to-end stale-cache note tests.
//! What this file proves, exactly: the note reaches article search
//! JSON and search-all JSON through `_meta.notes` (parsed, not
//! grepped); the ClinGen prefetch inside `get gene` carries a stale
//! serve's note to the markdown card and CLI/MCP get JSON. Day-cached GWAS replies remain
//! free of stale notes inside the day window even when the provider's
//! shorter freshness window has expired.
//! Fixtures answer with a one-second freshness window, then stop.

/// Answer every request with `body` as JSON, fresh-cacheable for one
/// second: after the window, with the server gone, the cache serves
/// stale and stamps the age marker the note reads.
async fn stale_note_fixture_server(body: &'static str) -> (String, tokio::task::JoinHandle<()>) {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind stale-note fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let app = axum::Router::new().fallback(move || async move {
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/json"),
                (header::CACHE_CONTROL, "max-age=1"),
            ],
            body,
        )
            .into_response()
    });
    let task =
        tokio::spawn(async move { axum::serve(listener, app).await.expect("fixture serves") });
    (base, task)
}

/// Answer each path prefix with its body, fresh-cacheable for one
/// second (the freshness windows can differ per route via the
/// max_age map; the ClinGen test pins MyGene fresh and ClinGen
/// stale so the note can only come from the stale leg).
/// A routes fixture whose ClinGen routes live on a separate
/// listener, so a test can kill only ClinGen and leave MyGene live.
#[allow(clippy::type_complexity)]
async fn clingen_split_fixture(
    mygene_routes: Vec<(String, &'static str, u64)>,
    clingen_routes_in: Vec<(String, &'static str, u64)>,
) -> (
    (String, String),
    tokio::task::JoinHandle<()>,
    tokio::task::JoinHandle<()>,
) {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;

    async fn serve(
        routes: Vec<(String, &'static str, u64)>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind split fixture");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let routes: std::sync::Arc<Vec<(String, &'static str, u64)>> = std::sync::Arc::new(routes);
        let app = axum::Router::new().fallback(move |uri: axum::http::Uri| {
            let routes = std::sync::Arc::clone(&routes);
            async move {
                let path = uri.path().to_string();
                let matched = routes.iter().find(|(prefix, _, _)| {
                    path == prefix.as_str() || path.starts_with(&format!("{prefix}?"))
                });
                match matched {
                    Some((_, body, max_age)) => {
                        let cache = format!("max-age={max_age}");
                        (
                            StatusCode::OK,
                            [
                                (
                                    header::CONTENT_TYPE,
                                    header::HeaderValue::from_static("application/json"),
                                ),
                                (
                                    header::CACHE_CONTROL,
                                    header::HeaderValue::from_str(&cache)
                                        .expect("valid cache-control header"),
                                ),
                            ],
                            *body,
                        )
                            .into_response()
                    }
                    None => StatusCode::NOT_FOUND.into_response(),
                }
            }
        });
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("split fixture serves")
        });
        (base, task)
    }
    let (mygene_base, mygene_task) = serve(mygene_routes).await;
    let (clingen_base, clingen_task) = serve(clingen_routes_in).await;
    ((mygene_base, clingen_base), mygene_task, clingen_task)
}

async fn stale_note_routes_server(
    routes: Vec<(String, &'static str, u64)>,
) -> (String, tokio::task::JoinHandle<()>) {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind stale-note routes fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let routes: std::sync::Arc<Vec<(String, &'static str, u64)>> = std::sync::Arc::new(routes);
    let app = axum::Router::new().fallback(move |uri: axum::http::Uri| {
        let routes = std::sync::Arc::clone(&routes);
        async move {
            let path = uri.path().to_string();
            let matched = routes.iter().find(|(prefix, _, _)| {
                path == prefix.as_str() || path.starts_with(&format!("{prefix}?"))
            });
            match matched {
                Some((_, body, max_age)) => {
                    let cache = format!("max-age={max_age}");
                    (
                        StatusCode::OK,
                        [
                            (
                                header::CONTENT_TYPE,
                                header::HeaderValue::from_static("application/json"),
                            ),
                            (
                                header::CACHE_CONTROL,
                                header::HeaderValue::from_str(&cache)
                                    .expect("valid cache-control header"),
                            ),
                        ],
                        *body,
                    )
                        .into_response()
                }
                None => (StatusCode::NOT_FOUND, "no such fixture route").into_response(),
            }
        }
    });
    let task =
        tokio::spawn(async move { axum::serve(listener, app).await.expect("fixture serves") });
    (base, task)
}

/// One command's stdout, driven through the real CLI entry.
async fn run(args: &[&str]) -> String {
    let argv = std::iter::once("biomcp")
        .chain(args.iter().copied())
        .map(str::to_string)
        .collect();
    crate::cli::execute(argv)
        .await
        .expect("command succeeds against the fixture")
}

/// The disk-free floor every constructor pins beside the cache
/// directory: `1B` is the smallest legal value, so no host
/// filesystem's free space can evict the cached reply before the
/// stale serve. A test that sets its own floor after the
/// constructor still wins.
const MIN_DISK_FREE: &str = "1B";

struct StaleNoteEnv {
    /// Held (never read) so the cache directory outlives the env.
    // dead-code reason: held-for-drop guard keeps the temp cache root alive
    #[allow(dead_code)]
    root: crate::test_support::TempDirGuard,
    /// (key, previous value) pairs restored on drop, so one test's
    /// fixture bases never leak into the next.
    previous: Vec<(&'static str, Option<String>)>,
}

impl StaleNoteEnv {
    fn two_bases(mygene: &str, clingen: &str, keys: &[&'static str]) -> Self {
        let root = crate::test_support::TempDirGuard::new("stale-json-notes");
        let mut previous = Vec::new();
        for key in keys
            .iter()
            .copied()
            .chain(["BIOMCP_CACHE_DIR", "BIOMCP_CACHE_MIN_DISK_FREE"])
        {
            let value = if key == "BIOMCP_CACHE_DIR" {
                root.path().to_str().expect("utf-8 cache root").to_string()
            } else if key == "BIOMCP_CACHE_MIN_DISK_FREE" {
                MIN_DISK_FREE.to_string()
            } else if key == "BIOMCP_MYGENE_BASE" {
                mygene.to_string()
            } else {
                clingen.to_string()
            };
            // SAFETY: serialized on the source_env key; restored on drop.
            unsafe {
                previous.push((key, std::env::var(key).ok()));
                std::env::set_var(key, value);
            }
        }
        Self { root, previous }
    }

    /// Distinct bases per source key (the search-all fixture pins
    /// four federated legs to two different servers).
    fn bases(pairs: &[(&'static str, &str)]) -> Self {
        let root = crate::test_support::TempDirGuard::new("stale-json-notes");
        let mut previous = Vec::new();
        for (key, base) in pairs.iter().copied() {
            // SAFETY: serialized on the source_env key; restored on drop.
            unsafe {
                previous.push((key, std::env::var(key).ok()));
                std::env::set_var(key, base);
            }
        }
        // SAFETY: as above.
        unsafe {
            previous.push(("BIOMCP_CACHE_DIR", std::env::var("BIOMCP_CACHE_DIR").ok()));
            std::env::set_var(
                "BIOMCP_CACHE_DIR",
                root.path().to_str().expect("utf-8 cache root"),
            );
            previous.push((
                "BIOMCP_CACHE_MIN_DISK_FREE",
                std::env::var("BIOMCP_CACHE_MIN_DISK_FREE").ok(),
            ));
            std::env::set_var("BIOMCP_CACHE_MIN_DISK_FREE", MIN_DISK_FREE);
        }
        Self { root, previous }
    }

    fn new(base: &str, keys: &[&'static str]) -> Self {
        let root = crate::test_support::TempDirGuard::new("stale-json-notes");
        let mut previous = Vec::new();
        for key in keys
            .iter()
            .copied()
            .chain(["BIOMCP_CACHE_DIR", "BIOMCP_CACHE_MIN_DISK_FREE"])
        {
            // SAFETY: serialized on the source_env key; values restore on drop.
            unsafe {
                previous.push((key, std::env::var(key).ok()));
                std::env::set_var(
                    key,
                    if key == "BIOMCP_CACHE_DIR" {
                        root.path().to_str().expect("utf-8 cache root")
                    } else if key == "BIOMCP_CACHE_MIN_DISK_FREE" {
                        MIN_DISK_FREE
                    } else {
                        base
                    },
                );
            }
        }
        Self { root, previous }
    }
}

impl Drop for StaleNoteEnv {
    fn drop(&mut self) {
        for (key, value) in self.previous.drain(..) {
            // SAFETY: serialized on the source_env key.
            unsafe {
                match value {
                    Some(previous) => std::env::set_var(key, previous),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}

const EPMC_BODY: &str = r#"{"hitCount":1,"resultList":{"result":[{"id":"32794606","source":"MED","title":"Aspirin","firstPublicationDate":"1990-01-01"}]}}"#;

/// Parse a command's stdout as JSON and return the `_meta.notes`
/// strings (empty when the channel is absent).
fn meta_notes(output: &str) -> Vec<String> {
    let value: serde_json::Value = serde_json::from_str(output)
        .unwrap_or_else(|error| panic!("output is not JSON ({error}): {output}"));
    value["_meta"]["notes"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_stale_article_search_json_states_the_cache_age_in_meta_notes() {
    let (base, server) = stale_note_fixture_server(EPMC_BODY).await;
    let _env = StaleNoteEnv::new(&base, &["BIOMCP_EUROPEPMC_BASE"]);
    let args = [
        "--json",
        "search",
        "article",
        "--keyword",
        "aspirin",
        "--limit",
        "1",
        "--source",
        "europepmc",
    ];
    let fresh = run(&args).await;
    assert!(
        meta_notes(&fresh).is_empty(),
        "fresh serve carries no note: {fresh}"
    );

    tokio::time::sleep(std::time::Duration::from_millis(1600)).await; // watchdog: freshness-window wait, bounded at 1.6 s
    server.abort();

    let stale = run(&args).await;
    let notes = meta_notes(&stale);
    assert!(
        notes
            .iter()
            .any(|note| note.contains("older than the provider's freshness window")),
        "the stale article search JSON carries the note in _meta.notes: {stale}"
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_day_cached_gwas_search_inside_the_day_window_has_no_cache_note() {
    const BODY: &str = r#"{"_embedded":{"associations":[{"accession_id":"GCST90002321","p_value":5e-8,"mapped_genes":["BRAF"],"reported_trait":["melanoma"],"snp_allele":[{"rs_id":"rs121913524","effect_allele":"T"}]}]},"page":{"totalElements":1}}"#;
    let (base, server) = stale_note_fixture_server(BODY).await;
    let _env = StaleNoteEnv::new(&base, &["BIOMCP_GWAS_BASE"]);
    let args = ["--json", "search", "gwas", "-g", "BRAF"];
    let fresh = run(&args).await;
    assert!(meta_notes(&fresh).is_empty(), "{fresh}");
    tokio::time::sleep(std::time::Duration::from_millis(1600)).await; // watchdog: provider freshness wait
    server.abort();
    let cached = run(&args).await;
    assert!(
        meta_notes(&cached).is_empty(),
        "the day-cached answer is still fresh: {cached}"
    );
}

/// The search-all end-to-end stale serve (restored 2026-09-29 per
/// Ian's refusal of the deferral): Europe PMC alone rides the
/// killable fixture whose body the cache holds; the other three
/// federated legs (PubMed, PubTator, Semantic Scholar) point at a
/// live fixture that answers 404 on every path. A 404 is terminal,
/// so those legs never retry a dead port and burn the 12 s
/// per-source budget — the 2026-09-29 review measured the stale run
/// at 11.8 s when all four legs hit the killed port and ~4.6 s with
/// the 404 legs.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_stale_search_all_json_states_the_cache_age_in_meta_notes() {
    let (epmc_base, epmc_server) = stale_note_fixture_server(EPMC_BODY).await;
    let (notfound_base, _notfound_server) = stale_note_routes_server(Vec::new()).await;
    let _env = StaleNoteEnv::bases(&[
        ("BIOMCP_EUROPEPMC_BASE", epmc_base.as_str()),
        ("BIOMCP_PUBMED_BASE", notfound_base.as_str()),
        ("BIOMCP_PUBTATOR_BASE", notfound_base.as_str()),
        ("BIOMCP_S2_BASE", notfound_base.as_str()),
    ]);
    let args = [
        "--json",
        "search",
        "all",
        "--keyword",
        "aspirin",
        "--limit",
        "1",
    ];
    let fresh = run(&args).await;
    assert!(
        meta_notes(&fresh).is_empty(),
        "fresh serve carries no note: {fresh}"
    );
    assert!(
        fresh.contains("\"article\""),
        "the article section ran: {fresh}"
    );

    tokio::time::sleep(std::time::Duration::from_millis(1600)).await; // watchdog: freshness-window wait, bounded at 1.6 s
    epmc_server.abort();

    let stale = run(&args).await;
    let notes = meta_notes(&stale);
    assert!(
        notes
            .iter()
            .any(|note| note.contains("older than the provider's freshness window")),
        "the stale search-all JSON carries the parsed note: {stale}"
    );
}

const MYGENE_BODY: &str =
    r#"{"total":1,"hits":[{"symbol":"BRAF","name":"B-Raf proto-oncogene","entrezgene":"673"}]}"#;
const CLINGEN_LOOKUP_BODY: &str = r#"[{"label":"BRAF","hgnc":"HGNC:1097","curated":true}]"#;
const CLINGEN_VALIDITY_BODY: &str = "GENE SYMBOL,GENE ID (HGNC),DISEASE LABEL,CLASSIFICATION,CLASSIFICATION DATE,MOI\nBRAF,HGNC:1097,Noonan syndrome,Definitive,2024-01-01,AD\n";
const CLINGEN_DOSAGE_BODY: &str = "GENE SYMBOL,HGNC ID,HAPLOINSUFFICIENCY,TRIPLOSENSITIVITY,DATE\nBRAF,HGNC:1097,3,3,2024-01-01\n";

/// The ClinGen prefetch runs on a bare `tokio::spawn`, which drops
/// the command's task-locals; the note scope travels by handle
/// and the no-cache flag with it. A
/// stale ClinGen serve inside the prefetch must reach the gene
/// card's note. MyGene is served fresh-cacheable for an hour so the
/// only stale leg is ClinGen.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_stale_clingen_prefetch_note_reaches_the_gene_card() {
    let (base, server) = stale_note_routes_server(vec![
        ("/query".to_string(), MYGENE_BODY, 3600),
        ("/api/genes/look/BRAF".to_string(), CLINGEN_LOOKUP_BODY, 1),
        (
            "/kb/gene-validity/download".to_string(),
            CLINGEN_VALIDITY_BODY,
            1,
        ),
        (
            "/kb/gene-dosage/download".to_string(),
            CLINGEN_DOSAGE_BODY,
            1,
        ),
    ])
    .await;
    // Both bases point at one fixture server; the path router picks
    // the body. MyGene is pinned fresh, ClinGen stale.
    let _env = StaleNoteEnv::new(&base, &["BIOMCP_MYGENE_BASE", "BIOMCP_CLINGEN_BASE"]);
    let fresh = run(&["get", "gene", "BRAF", "clingen"]).await;
    assert!(
        !fresh.contains("Cache note:"),
        "fresh serve carries no card note: {fresh}"
    );
    assert!(
        fresh.contains("Noonan syndrome") || fresh.contains("ClinGen"),
        "the clingen section rendered: {fresh}"
    );

    tokio::time::sleep(std::time::Duration::from_millis(1600)).await; // watchdog: freshness-window wait, bounded at 1.6 s
    server.abort();

    let stale = run(&["get", "gene", "BRAF", "clingen"]).await;
    assert!(
        stale.contains("Cache note:")
            && stale.contains("older than the provider's freshness window"),
        "the stale ClinGen prefetch serve reaches the card note: {stale}"
    );
}

/// `--no-cache` must reach the spawned ClinGen prefetch: the first run persists the fixture answers; after the
/// server dies, a `--no-cache` run must fail rather than read the
/// cached entries — which is exactly what a leaked flag would do.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn no_cache_skips_the_cache_for_the_spawned_clingen_fetch() {
    let ((mygene_base, clingen_base), _mygene_task, clingen_routes) = clingen_split_fixture(
        vec![("/query".to_string(), MYGENE_BODY, 3600)],
        vec![
            (
                "/api/genes/look/BRAF".to_string(),
                CLINGEN_LOOKUP_BODY,
                3600,
            ),
            (
                "/kb/gene-validity/download".to_string(),
                CLINGEN_VALIDITY_BODY,
                3600,
            ),
            (
                "/kb/gene-dosage/download".to_string(),
                CLINGEN_DOSAGE_BODY,
                3600,
            ),
        ],
    )
    .await;
    let _env = StaleNoteEnv::two_bases(
        &mygene_base,
        &clingen_base,
        &["BIOMCP_MYGENE_BASE", "BIOMCP_CLINGEN_BASE"],
    );
    let warm = run(&["get", "gene", "BRAF", "clingen"]).await;
    assert!(
        warm.contains("ClinGen") || warm.contains("Noonan syndrome"),
        "the warm run populated the cache: {warm}"
    );
    // Kill ONLY the ClinGen routes; MyGene stays live so the parent's
    // own uncached fetch succeeds and the card renders. A prefetch
    // that wrongly reads the cache then visibly serves the cached
    // Noonan rows; with the carry, it hits the dead routes and its
    // absence is the observable.
    clingen_routes.abort();

    let bypassed = run(&["--no-cache", "get", "gene", "BRAF", "clingen"]).await;
    assert!(
        bypassed.contains("BRAF"),
        "the parent gene fetch rendered (MyGene live): {bypassed}"
    );
    assert!(
        !bypassed.contains("Noonan syndrome"),
        "--no-cache must not serve the cached ClinGen rows to the prefetch: {bypassed}"
    );
}

/// Search-all's note plumbing, pinned deterministically: the JSON
/// builder inserts `_meta` exactly when notes exist. The end-to-end
/// stale serve runs again above (restored 2026-09-29 with the 404
/// federated legs); this pin covers the builder shape directly.
#[test]
fn search_all_json_body_carries_notes_only_when_present() {
    let results = crate::cli::search_all::SearchAllResults {
        query: Default::default(),
        sections: Vec::new(),
        searches_dispatched: 0,
        searches_with_results: 0,
        wall_time_ms: 0,
        debug_plan: None,
    };
    let with = crate::cli::search_all::json_body(
        &results,
        false,
        vec!["Europe PMC data served from cache, 2 h old (older than the provider's freshness window).".to_string()],
    )
    .expect("json body builds");
    assert!(
        with["_meta"]["notes"][0]
            .as_str()
            .expect("note text")
            .contains("older than the provider's freshness window")
    );

    let without =
        crate::cli::search_all::json_body(&results, false, Vec::new()).expect("json body builds");
    assert!(without.get("_meta").is_none());
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn stale_get_json_reports_cache_notes_without_changing_the_fresh_card() {
    let (base, server) = stale_note_routes_server(vec![
        ("/query".to_string(), MYGENE_BODY, 3600),
        ("/api/genes/look/BRAF".to_string(), CLINGEN_LOOKUP_BODY, 1),
        (
            "/kb/gene-validity/download".to_string(),
            CLINGEN_VALIDITY_BODY,
            1,
        ),
        (
            "/kb/gene-dosage/download".to_string(),
            CLINGEN_DOSAGE_BODY,
            1,
        ),
    ])
    .await;
    let _env = StaleNoteEnv::new(&base, &["BIOMCP_MYGENE_BASE", "BIOMCP_CLINGEN_BASE"]);
    let args = ["--json", "get", "gene", "BRAF", "clingen"];
    let fresh = run(&args).await;
    let fresh_value: serde_json::Value = serde_json::from_str(&fresh).expect("fresh JSON");
    assert!(fresh_value["_meta"].get("notes").is_none(), "{fresh}");
    assert_eq!(
        run(&args).await,
        fresh,
        "fresh cache preserves the JSON bytes"
    );

    tokio::time::sleep(std::time::Duration::from_millis(1600)).await; // watchdog: bounded freshness-window wait
    server.abort();
    let stale = run(&args).await;
    assert!(
        meta_notes(&stale)
            .iter()
            .any(|note| { note.contains("older than the provider's freshness window") }),
        "stale get JSON must carry the cache note: {stale}"
    );
    let mut stale_value: serde_json::Value = serde_json::from_str(&stale).expect("stale JSON");
    stale_value["_meta"]
        .as_object_mut()
        .expect("metadata")
        .remove("notes");
    assert_eq!(stale_value, fresh_value, "only cache notes change");

    let mcp_args = std::iter::once("biomcp")
        .chain(args)
        .map(str::to_string)
        .collect();
    let mcp = crate::cli::execute_mcp(mcp_args)
        .await
        .expect("MCP JSON get");
    assert!(!meta_notes(&mcp.text).is_empty(), "{}", mcp.text);
    assert_eq!(mcp.metadata_json.as_deref(), Some(mcp.text.as_str()));
    let markdown_args = ["biomcp", "get", "gene", "BRAF", "clingen"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let markdown = crate::cli::execute_mcp(markdown_args)
        .await
        .expect("MCP Markdown get");
    assert!(markdown.text.contains("Cache note:"), "{}", markdown.text);
    assert!(
        markdown
            .text
            .contains("older than the provider's freshness window")
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn stale_author_get_json_reaches_the_shared_finalizer() {
    let (base, server) = stale_note_fixture_server(include_str!(
        "../../testdata/sources/semantic_scholar/author_detail.json"
    ))
    .await;
    let _env = StaleNoteEnv::new(&base, &["BIOMCP_S2_BASE"]);
    let args = ["--json", "get", "author", "semanticscholar:1716151"];
    let fresh = run(&args).await;
    let fresh_value: serde_json::Value = serde_json::from_str(&fresh).expect("fresh author JSON");
    assert!(fresh_value["_meta"].get("notes").is_none(), "{fresh}");

    tokio::time::sleep(std::time::Duration::from_millis(1600)).await; // watchdog: bounded freshness-window wait
    server.abort();
    let stale = run(&args).await;
    assert!(
        meta_notes(&stale)
            .iter()
            .any(|note| note.contains("older than the provider's freshness window")),
        "author get through the fallback dispatcher must carry the cache note: {stale}"
    );
    let mcp_args = std::iter::once("biomcp")
        .chain(args)
        .map(str::to_string)
        .collect();
    let mcp = crate::cli::execute_mcp(mcp_args)
        .await
        .expect("MCP author JSON get");
    assert!(!meta_notes(&mcp.text).is_empty(), "{}", mcp.text);
}
