use clap::Parser;

use crate::cli::{Cli, Commands, VariantCommand};
use crate::entities::variant::GenomeBuild;

#[tokio::test]
async fn rejects_assembly_for_transcript_hgvs() {
    let err = crate::cli::variant::handle_get(
        crate::cli::variant::VariantGetArgs {
            assembly: Some(GenomeBuild::Grch38),
            id: "NM_004333.6:c.1799T>A".into(),
            sections: Vec::new(),
        },
        false,
        false,
    )
    .await
    .expect_err("assembly should reject transcript HGVS before lookup");

    assert_eq!(
        err.to_string(),
        "Invalid argument: --assembly only applies to chromosome-prefixed genomic coordinates"
    );
}

#[test]
fn preserves_positional_syntax_and_accepts_structured_input() {
    let positional = Cli::try_parse_from([
        "biomcp",
        "--json",
        "variant",
        "articles",
        "BRAF V600E",
        "--debug-plan",
    ])
    .expect("positional variant articles");
    let input = Cli::try_parse_from(["biomcp", "--json", "variant", "articles", "--input", "-"])
        .expect("structured variant articles");

    assert!(matches!(
        positional.command,
        Commands::Variant {
            cmd: VariantCommand::Articles {
                id: Some(_),
                input: None,
                debug_plan: true,
                ..
            }
        }
    ));
    assert!(matches!(
        input.command,
        Commands::Variant {
            cmd: VariantCommand::Articles {
                id: None,
                input: Some(_),
                ..
            }
        }
    ));
}

#[tokio::test]
async fn rejects_positional_input_combination_with_the_batch_error_envelope() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "--json",
        "variant",
        "articles",
        "BRAF V600E",
        "--input",
        "variants.json",
    ])
    .expect("the handler owns the structured conflict");
    let outcome = crate::cli::run_outcome(cli)
        .await
        .expect("typed invalid argument outcome");
    let value: serde_json::Value =
        serde_json::from_str(&outcome.text).expect("structured error JSON");

    assert_eq!(outcome.exit_code, 2);
    assert_eq!(value["error"]["code"], "invalid_argument");
    assert_eq!(value["items"], serde_json::json!([]));
    assert_eq!(value["complete"], false);
    assert_eq!(value["truncated"], false);
    assert_eq!(value["_meta"]["next_commands"], serde_json::json!([]));
}
