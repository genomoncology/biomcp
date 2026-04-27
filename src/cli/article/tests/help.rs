//! Article CLI help text and parser tests.
use clap::{CommandFactory, Parser};

use super::super::dispatch::resolved_article_date_bounds;
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

pub(super) fn parse_article_search(
    argv: impl IntoIterator<Item = &'static str>,
) -> (super::super::ArticleSearchArgs, bool) {
    let cli = Cli::try_parse_from(argv).expect("article search should parse");
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
    (args, json)
}

#[test]
fn search_article_help_includes_when_to_use_guidance() {
    let help = render_article_search_long_help();

    assert!(help.contains("When to use:"));
    assert!(help.contains("keyword search to scan a topic"));
    assert!(help.contains("Prefer --type review"));
}

#[test]
fn search_article_help_includes_query_formulation_guidance() {
    let help = render_article_search_long_help();

    assert!(help.contains("QUERY FORMULATION:"));
    assert!(help.contains(
        "Known gene/disease/drug anchors belong in `-g/--gene`, `-d/--disease`, or `--drug`."
    ));
    assert!(help.contains(
        "Use `-k/--keyword` for mechanisms, phenotypes, datasets, outcomes, and other free-text concepts."
    ));
    assert!(help.contains(
        "PubMed ESearch cleans question-format gene/disease/drug/keyword terms provider-locally; query echoes and non-PubMed sources keep the original wording."
    ));
    assert!(
        help.contains(
            "Unknown-entity questions should stay keyword-first or start with `discover`."
        )
    );
    assert!(help.contains(
        "Keyword-only result pages can suggest typed `get gene`, `get drug`, or `get disease` follow-ups when the whole `-k/--keyword` exactly matches a vocabulary label or alias."
    ));
    assert!(help.contains(
        "Multi-concept phrases and searches that already use `-g/--gene`, `-d/--disease`, or `--drug` do not get direct entity suggestions."
    ));
    assert!(help.contains(
        "Adding `-k/--keyword` on the default route brings in LitSense2 and default `hybrid` relevance."
    ));
    assert!(help.contains(
        "`semantic` sorts by the LitSense2-derived semantic signal and falls back to lexical ties."
    ));
    assert!(help.contains(
        "Hybrid score = `0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position` by default, using the same LitSense2-derived semantic signal and `semantic=0` when LitSense2 did not match."
    ));
    assert!(
        help.contains("biomcp search article -g TP53 -k \"apoptosis gene regulation\" --limit 5")
    );
    assert!(help.contains(
        "biomcp search article -k '\"cafe-au-lait spots\" neurofibromas disease' --type review --limit 5"
    ));
}

#[test]
fn article_date_help_advertises_shared_accepted_formats() {
    let help = render_article_search_long_help();

    assert!(help.contains("Published after date (YYYY, YYYY-MM, or YYYY-MM-DD)"));
    assert!(help.contains("Published before date (YYYY, YYYY-MM, or YYYY-MM-DD)"));
    assert!(help.contains("--year-min <YYYY>"));
    assert!(help.contains("--year-max <YYYY>"));
    assert!(help.contains("Published from year (YYYY)"));
    assert!(help.contains("Published through year (YYYY)"));
    assert!(help.contains("[aliases: --since]"));
    assert!(help.contains("[aliases: --until]"));
    assert!(help.contains("--max-per-source <N>"));
    assert!(help.contains(
        "Cap each federated source's contribution after deduplication and before ranking."
    ));
    assert!(help.contains(
        "Default: 40% of `--limit` on federated pools with at least three surviving primary sources."
    ));
    assert!(help.contains("`0` uses the default cap."));
    assert!(help.contains("Setting it equal to `--limit` disables capping."));
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
fn get_article_help_includes_opt_in_pdf_guidance() {
    let help = render_article_get_long_help();

    assert!(help.contains("--pdf"));
    assert!(help.contains("Allow Semantic Scholar PDF as a final fulltext fallback"));
    assert!(help.contains("`--pdf` requires the fulltext section."));
    assert!(help.contains("biomcp get article 22663011 fulltext --pdf"));
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
