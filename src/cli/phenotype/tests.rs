use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Commands, SearchEntity};

#[test]
fn search_phenotype_help_mentions_hpo_ids_and_symptom_phrases() {
    let mut command = Cli::command();
    let search = command
        .find_subcommand_mut("search")
        .expect("search subcommand should exist");
    let phenotype = search
        .find_subcommand_mut("phenotype")
        .expect("phenotype subcommand should exist");
    let mut help = Vec::new();
    phenotype
        .write_long_help(&mut help)
        .expect("phenotype help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("HPO IDs"));
    assert!(help.contains("space- or comma-separated"));
    assert!(help.contains("one symptom phrase"));
    assert!(help.contains("comma-separated symptom phrases"));
    assert!(help.contains("seizure, developmental delay"));
    assert!(help.contains("biomcp list phenotype"));
}

#[test]
fn search_args_reject_zero_limit_before_backend_lookup() {
    let cli = Cli::try_parse_from(["biomcp", "search", "phenotype", "seizure", "--limit", "0"])
        .expect("search phenotype should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Phenotype(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected search phenotype command");
    };

    assert!(!json);
    let err = super::dispatch::validate_search_args(&args)
        .expect_err("zero phenotype limit should fail fast");
    assert!(err.to_string().contains("--limit must be between 1 and 50"));
}

#[test]
fn search_args_reject_a_window_beyond_the_provider_budget() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "phenotype",
        "HP:0001250",
        "--limit",
        "11",
        "--offset",
        "40",
    ])
    .expect("search phenotype should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Phenotype(args),
        },
        ..
    } = cli
    else {
        panic!("expected search phenotype command");
    };

    let err = super::dispatch::validate_search_args(&args)
        .expect_err("unsupported phenotype window should fail fast");
    assert!(err.to_string().contains("--offset + --limit must be <= 50"));
}
