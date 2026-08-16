use clap::Parser;

use crate::cli::{Cli, Commands};

#[test]
fn batch_source_is_optional_and_trial_only() {
    let cli = Cli::try_parse_from(["biomcp", "batch", "trial", "NCT02576665"])
        .expect("trial batch should parse without a source");
    let Commands::Batch(args) = cli.command else {
        panic!("expected batch command");
    };
    assert!(args.source.is_none());

    let cli = Cli::try_parse_from(["biomcp", "batch", "gene", "BRAF", "--source", "ctgov"])
        .expect("global batch grammar still parses source");
    let Commands::Batch(args) = cli.command else {
        panic!("expected batch command");
    };
    let err = super::super::dispatch::validate_batch_args(&args)
        .expect_err("non-trial source should fail before provider work");
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
