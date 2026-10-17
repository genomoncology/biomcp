//! Article CLI help text and parser tests.
use clap::{CommandFactory, Parser};

use super::super::workflow::resolved_article_date_bounds;
use crate::cli::{Cli, Commands, GetEntity, SearchEntity};

fn render_article_search_long_help() -> String {
    let mut command = Cli::command();
    let search = command
        .find_subcommand_mut("search")
        .expect("search subcommand should exist");
    let article = search
        .find_subcommand_mut("article")
        .expect("article subcommand should exist");
    let mut help = Vec::new();
    article
        .write_long_help(&mut help)
        .expect("article help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

fn render_article_get_long_help() -> String {
    let mut command = Cli::command();
    let get = command
        .find_subcommand_mut("get")
        .expect("get subcommand should exist");
    let article = get
        .find_subcommand_mut("article")
        .expect("article subcommand should exist");
    let mut help = Vec::new();
    article
        .write_long_help(&mut help)
        .expect("help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

#[test]
fn article_full_flag_parses_and_help_explains_compact_default() {
    let cli = Cli::try_parse_from([
        "biomcp", "--json", "search", "article", "-g", "BRAF", "--full",
    ])
    .expect("article full flag should parse");
    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Article(args),
        },
        ..
    } = cli
    else {
        panic!("expected article search command");
    };

    assert!(args.full);
    let help = render_article_search_long_help();
    assert!(help.contains("--full"));
    assert!(help.contains("JSON search rows are compact by default"));
    assert!(help.contains("`--sort date` replaces relevance ranking"));
}

#[test]
fn article_session_flag_parses_and_help_documents_json_loop_breaker() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "--json",
        "search",
        "article",
        "-k",
        "Oncotype DX review",
        "--session",
        "lit-review-1",
        "--limit",
        "5",
    ])
    .expect("article session flag should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Article(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected article search command");
    };

    assert!(json);
    assert_eq!(args.session.as_deref(), Some("lit-review-1"));

    let help = render_article_search_long_help();
    assert!(help.contains("--session <TOKEN>"));
    assert!(help.contains("Local caller label for JSON loop-breaker suggestions"));
    assert!(help.contains("SESSION LOOP BREAKER:"));
    assert!(help.contains("Tokens are not secrets"));
    assert!(help.contains(
        "biomcp --json search article -k \"Oncotype DX review\" --session lit-review-1 --limit 5"
    ));
}

#[test]
fn article_get_pdf_modifier_parses_before_fulltext() {
    let cli = Cli::try_parse_from(["biomcp", "get", "article", "22663011", "--pdf", "fulltext"])
        .expect("article get should accept --pdf before fulltext");

    let Cli {
        command: Commands::Get {
            entity: GetEntity::Article(args),
        },
        ..
    } = cli
    else {
        panic!("expected article get command");
    };

    assert_eq!(args.id, "22663011");
    assert_eq!(args.sections, vec!["fulltext"]);
}

#[test]
fn article_get_pdf_modifier_parses_after_fulltext() {
    let cli = Cli::try_parse_from(["biomcp", "get", "article", "22663011", "fulltext", "--pdf"])
        .expect("article get should accept --pdf after fulltext");

    let Cli {
        command: Commands::Get {
            entity: GetEntity::Article(args),
        },
        ..
    } = cli
    else {
        panic!("expected article get command");
    };

    assert_eq!(args.id, "22663011");
    assert_eq!(args.sections.first().map(String::as_str), Some("fulltext"));
}

#[test]
fn article_asset_paging_modifiers_parse_after_assets() {
    let commands = [
        [
            "biomcp",
            "get",
            "article",
            "42125600",
            "--asset-view",
            "coverage",
            "--asset-limit",
            "25",
            "--asset-offset",
            "3",
            "assets",
        ],
        [
            "biomcp",
            "get",
            "article",
            "42125600",
            "assets",
            "--asset-view",
            "coverage",
            "--asset-limit",
            "25",
            "--asset-offset",
            "3",
        ],
    ];

    for command in commands {
        let cli = Cli::try_parse_from(command)
            .expect("asset paging modifiers should parse after the assets section");
        let Cli {
            command: Commands::Get {
                entity: GetEntity::Article(args),
            },
            ..
        } = cli
        else {
            panic!("expected article get command");
        };

        assert_eq!(args.sections, vec!["assets".to_string()]);
        assert_eq!(args.asset_view, "coverage");
        assert_eq!(args.asset_limit, Some(25));
        assert_eq!(args.asset_offset, 3);
    }
}

#[test]
fn article_year_flags_parse_and_expand_to_date_bounds() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "article",
        "-g",
        "BRAF",
        "--year-min",
        "2000",
        "--year-max",
        "2013",
        "--limit",
        "1",
    ])
    .expect("article year flags should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Article(args),
        },
        ..
    } = cli
    else {
        panic!("expected article search command");
    };

    assert_eq!(args.year_min, Some(2000));
    assert_eq!(args.year_max, Some(2013));
    let (date_from, date_to) = resolved_article_date_bounds(&args);
    assert_eq!(date_from.as_deref(), Some("2000-01-01"));
    assert_eq!(date_to.as_deref(), Some("2013-12-31"));
}

#[test]
fn article_year_flags_reject_non_yyyy_values() {
    let err = Cli::try_parse_from([
        "biomcp",
        "search",
        "article",
        "-g",
        "BRAF",
        "--year-min",
        "200",
    ])
    .expect_err("non-YYYY year should fail to parse");

    let message = err.to_string();
    assert!(message.contains("invalid value '200' for '--year-min <YYYY>'"));
    assert!(message.contains("expected YYYY"));
}

#[test]
fn article_year_flags_conflict_with_explicit_dates() {
    let err = Cli::try_parse_from([
        "biomcp",
        "search",
        "article",
        "-g",
        "BRAF",
        "--year-min",
        "2000",
        "--date-from",
        "2000-01-01",
    ])
    .expect_err("year-min and date-from should conflict");

    assert!(err.to_string().contains(
        "the argument '--year-min <YYYY>' cannot be used with '--date-from <DATE_FROM>'"
    ));
}

#[test]
fn article_year_max_conflicts_with_date_to() {
    let err = Cli::try_parse_from([
        "biomcp",
        "search",
        "article",
        "-g",
        "BRAF",
        "--year-max",
        "2013",
        "--date-to",
        "2013-12-31",
    ])
    .expect_err("year-max and date-to should conflict");

    assert!(
        err.to_string()
            .contains("the argument '--year-max <YYYY>' cannot be used with '--date-to <DATE_TO>'")
    );
}
