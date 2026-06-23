use clap::{CommandFactory, Parser};

use super::ProteinCommand;
use crate::cli::{Cli, Commands, SearchEntity};

fn render_search_protein_help() -> String {
    let mut command = Cli::command();
    let search = command
        .find_subcommand_mut("search")
        .expect("search subcommand should exist");
    let protein = search
        .find_subcommand_mut("protein")
        .expect("search protein subcommand should exist");
    let mut help = Vec::new();
    protein
        .write_long_help(&mut help)
        .expect("help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

#[test]
fn protein_structures_parses_offset_flag() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "protein",
        "structures",
        "P15056",
        "--limit",
        "5",
        "--offset",
        "5",
    ])
    .expect("protein structures pagination flags should parse");

    match cli.command {
        Commands::Protein {
            cmd:
                ProteinCommand::Structures {
                    accession,
                    limit,
                    offset,
                },
        } => {
            assert_eq!(accession, "P15056");
            assert_eq!(limit, 5);
            assert_eq!(offset, 5);
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn search_protein_help_shows_limit_range() {
    let help = render_search_protein_help();
    assert!(help.contains("Maximum results, 1-100"));
}

#[test]
fn search_args_reject_too_large_limit() {
    let cli = Cli::try_parse_from(["biomcp", "search", "protein", "kinase", "--limit", "101"])
        .expect("protein search should parse before validation");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Protein(args),
        },
        ..
    } = cli
    else {
        panic!("expected protein search command");
    };

    let err = super::dispatch::validate_search_args(&args)
        .expect_err("too-large protein search limit should fail fast");
    assert!(
        err.to_string()
            .contains("--limit for search protein must be 1-100")
    );
}

#[test]
fn search_args_reject_next_page_with_offset() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "protein",
        "BRAF",
        "--next-page",
        "cursor-1",
        "--offset",
        "1",
    ])
    .expect("protein search should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Protein(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected protein search command");
    };

    assert!(!json);
    let err = super::dispatch::validate_search_args(&args)
        .expect_err("next-page plus offset should fail fast");
    assert!(
        err.to_string()
            .contains("--next-page cannot be used together with --offset")
    );
}
