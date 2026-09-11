use clap::Parser;
use futures::future::BoxFuture;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

use crate::cli::{Cli, Commands};

#[tokio::test]
async fn ten_batch_items_start_together_and_each_owns_its_deadline() {
    let inputs = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    let started = Arc::new(AtomicUsize::new(0));
    let futures: Vec<BoxFuture<'_, Result<usize, crate::error::BioMcpError>>> = (0usize..10)
        .map(|index| {
            let started = Arc::clone(&started);
            Box::pin(async move {
                started.fetch_add(1, Ordering::SeqCst);
                while started.load(Ordering::SeqCst) != 10 {
                    tokio::task::yield_now().await;
                }
                tokio::time::timeout(Duration::from_millis(30), async {
                    tokio::time::sleep(Duration::from_millis(if index == 0 { 80 } else { 2 }))
                        .await;
                    index
                })
                .await
                .map_err(|_| crate::error::BioMcpError::SourceUnavailable {
                    source_name: "GenCC".into(),
                    reason: "per-gene deadline".into(),
                    suggestion: "retry".into(),
                })
            }) as BoxFuture<'_, _>
        })
        .collect();
    let began = Instant::now();
    let outcome = super::super::settle_batch(
        "gene",
        &inputs,
        futures,
        true,
        |value| Ok((*value).into()),
        |value| Ok(value.to_string()),
    )
    .await
    .unwrap();
    assert!(began.elapsed() < Duration::from_millis(100));
    let value: serde_json::Value = serde_json::from_str(&outcome.text).unwrap();
    assert_eq!(
        value["summary"],
        serde_json::json!({"total":10,"succeeded":9,"failed":1})
    );
    assert_eq!(value["items"][9]["result"], 9);
}

#[test]
fn batch_source_is_optional_and_trial_only() {
    let cli = Cli::try_parse_from(["biomcp", "batch", "trial", "NCT02576665"])
        .expect("trial batch should parse without a source");
    let Commands::Batch(args) = cli.command else {
        panic!("expected batch command");
    };
    assert!(args.source.is_none());

    for source in ["ctgov", "nci"] {
        let cli = Cli::try_parse_from([
            "biomcp",
            "batch",
            "trial",
            "NCT02576665",
            "--source",
            source,
        ])
        .expect("known trial source should parse");
        let Commands::Batch(args) = cli.command else {
            panic!("expected batch command");
        };
        let prepared = super::super::preflight_batch(&args)
            .expect("known trial source remains valid through preflight");
        assert_eq!(prepared.entity, "trial");
        assert!(prepared.trial_source.is_some());
    }

    let cli = Cli::try_parse_from([
        "biomcp",
        "batch",
        "trial",
        "NCT02576665",
        "--source",
        "unknown",
    ])
    .expect("global grammar parses source values before preflight");
    let Commands::Batch(args) = cli.command else {
        panic!("expected batch command");
    };
    assert!(super::super::preflight_batch(&args).is_err());

    let cli = Cli::try_parse_from(["biomcp", "batch", "gene", "BRAF", "--source", "ctgov"])
        .expect("global batch grammar still parses source");
    let Commands::Batch(args) = cli.command else {
        panic!("expected batch command");
    };
    let err = super::super::validate_batch_args(&args)
        .expect_err("non-trial source should fail before provider work");
    assert!(
        err.to_string()
            .contains("--source is only supported for trial batches")
    );

    let cli = Cli::try_parse_from(["biomcp", "batch", "article", "22663011", "--source", "nci"])
        .expect("article source reaches article-only preflight rejection");
    let Commands::Batch(args) = cli.command else {
        panic!("expected batch command");
    };
    let err = super::super::validate_batch_args(&args)
        .expect_err("article source should fail before provider work");
    assert!(
        err.to_string()
            .contains("--source is only supported for trial batches")
    );
}

#[test]
fn serve_http_rejects_port_zero() {
    let error = crate::cli::try_parse_cli(["biomcp", "serve-http", "--port", "0"])
        .expect_err("port zero must not create an undisclosed listener");
    assert!(
        error
            .to_string()
            .contains("--port must be between 1 and 65535")
    );
}

#[tokio::test(start_paused = true)]
#[serial_test::serial(article_output_fixture)]
async fn dropping_canonical_compact_command_cancels_provider_retry() {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

    struct EnvRestore(Vec<(&'static str, Option<std::ffi::OsString>)>);
    impl EnvRestore {
        fn set(&mut self, name: &'static str, value: impl AsRef<std::ffi::OsStr>) {
            self.0.push((name, std::env::var_os(name)));
            // SAFETY: this test owns the shared article fixture serial-test key.
            unsafe { std::env::set_var(name, value) };
        }

        fn remove(&mut self, name: &'static str) {
            self.0.push((name, std::env::var_os(name)));
            // SAFETY: this test owns the shared article fixture serial-test key.
            unsafe { std::env::remove_var(name) };
        }
    }
    impl Drop for EnvRestore {
        fn drop(&mut self) {
            for (name, value) in self.0.drain(..).rev() {
                // SAFETY: this test owns the shared article fixture serial-test key.
                unsafe {
                    match value {
                        Some(value) => std::env::set_var(name, value),
                        None => std::env::remove_var(name),
                    }
                }
            }
        }
    }

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind command retry fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let s2_requests = Arc::new(AtomicUsize::new(0));
    let active_connections = Arc::new(AtomicUsize::new(0));
    let first_s2_response = Arc::new(tokio::sync::Notify::new());
    let server = tokio::spawn({
        let s2_requests = Arc::clone(&s2_requests);
        let active_connections = Arc::clone(&active_connections);
        let first_s2_response = Arc::clone(&first_s2_response);
        async move {
            loop {
                let (mut stream, _) = listener.accept().await.expect("accept fixture request");
                active_connections.fetch_add(1, Ordering::SeqCst);
                let mut request = [0_u8; 4096];
                let read = stream
                    .read(&mut request)
                    .await
                    .expect("read fixture request");
                let request = String::from_utf8_lossy(&request[..read]);
                let (status, body) = if request.starts_with("POST /graph/v1/paper/batch?") {
                    s2_requests.fetch_add(1, Ordering::SeqCst);
                    ("503 Service Unavailable", "")
                } else if request.starts_with("GET /publications/export/biocjson?") {
                    (
                        "200 OK",
                        r#"{"PubTator3":[{"pmid":22663011,"authors":["Ada First"],"passages":[{"infons":{"type":"title"},"text":"Fixture article"}]}]}"#,
                    )
                } else if request.starts_with("GET /search?") {
                    (
                        "200 OK",
                        r#"{"hitCount":1,"resultList":{"result":[{"id":"22663011","pmid":"22663011","title":"Fixture article","journalTitle":"Fixture Journal","firstPublicationDate":"2025-01-01"}]}}"#,
                    )
                } else {
                    ("404 Not Found", r#"{"error":"unexpected fixture route"}"#)
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write fixture response");
                active_connections.fetch_sub(1, Ordering::SeqCst);
                if status.starts_with("503") {
                    first_s2_response.notify_one();
                }
            }
        }
    });

    let cache = crate::test_support::TempDirGuard::new("article-command-cancel-cache");
    let mut env = EnvRestore(Vec::new());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
    env.set("BIOMCP_PUBTATOR_BASE", &base);
    env.set("BIOMCP_EUROPEPMC_BASE", &base);
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.remove("S2_API_KEY");

    let cache_gets = Arc::new(AtomicUsize::new(0));
    let cache_puts = Arc::new(AtomicUsize::new(0));
    let middleware_owner = Arc::new(());
    let middleware_dropped = Arc::downgrade(&middleware_owner);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &base,
        {
            let cache_gets = Arc::clone(&cache_gets);
            let middleware_owner = Arc::clone(&middleware_owner);
            move |_, _| {
                let _owner = &middleware_owner;
                cache_gets.fetch_add(1, Ordering::SeqCst);
            }
        },
        {
            let cache_puts = Arc::clone(&cache_puts);
            let middleware_owner = Arc::clone(&middleware_owner);
            move |_, _| {
                let _owner = &middleware_owner;
                cache_puts.fetch_add(1, Ordering::SeqCst);
            }
        },
    )
    .expect("observed Semantic Scholar client");
    drop(middleware_owner);

    let cli = crate::cli::try_parse_cli([
        "biomcp", "--json", "batch", "article", "22663011", "--mode", "compact",
    ])
    .expect("canonical compact command parses");
    let Cli {
        command: Commands::Batch(args),
        json,
        no_cache,
    } = cli
    else {
        panic!("expected canonical batch command")
    };
    let mut command = Box::pin(crate::sources::semantic_scholar::with_test_client(
        client,
        crate::sources::with_no_cache(no_cache, super::super::handle_batch(args, json)),
    ));
    tokio::select! {
        () = first_s2_response.notified() => {}
        result = &mut command => panic!("command settled before provider retry: {result:?}"),
    }
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    assert_eq!(s2_requests.load(Ordering::SeqCst), 1);
    // The canonical compact enrichment is a POST, so the real cache
    // middleware correctly performs no lookup or publication.
    assert_eq!(cache_gets.load(Ordering::SeqCst), 0);
    assert_eq!(cache_puts.load(Ordering::SeqCst), 0);
    assert!(middleware_dropped.upgrade().is_some());

    drop(command);
    assert!(middleware_dropped.upgrade().is_none());
    assert_eq!(active_connections.load(Ordering::SeqCst), 0);
    tokio::time::advance(Duration::from_secs(30)).await;
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    assert_eq!(s2_requests.load(Ordering::SeqCst), 1);
    assert_eq!(cache_gets.load(Ordering::SeqCst), 0);
    assert_eq!(cache_puts.load(Ordering::SeqCst), 0);
    assert_eq!(active_connections.load(Ordering::SeqCst), 0);
    server.abort();
    assert!(
        server
            .await
            .expect_err("fixture server should be cancelled")
            .is_cancelled()
    );
}
