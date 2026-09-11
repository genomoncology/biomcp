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
