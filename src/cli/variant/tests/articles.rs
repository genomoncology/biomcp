use clap::Parser;

use crate::cli::{Cli, Commands, VariantCommand};

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

#[test]
fn rejects_positional_input_combination() {
    let error = Cli::try_parse_from([
        "biomcp",
        "--json",
        "variant",
        "articles",
        "BRAF V600E",
        "--input",
        "variants.json",
    ])
    .expect_err("positional ID and --input must conflict");

    assert!(error.to_string().contains("cannot be used with"));
}
