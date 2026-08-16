use super::*;

#[test]
fn cache_path_command_parses() {
    Cli::try_parse_from(["biomcp", "cache", "path"]).expect("cache path should parse");
}

#[test]
fn cache_stats_command_parses() {
    Cli::try_parse_from(["biomcp", "cache", "stats"]).expect("cache stats should parse");
}

#[test]
fn cache_clean_command_parses_with_flags() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "cache",
        "clean",
        "--max-age",
        "30d",
        "--max-size",
        "500M",
        "--dry-run",
    ])
    .expect("cache clean should parse");

    let Cli {
        command:
            Commands::Cache {
                cmd:
                    crate::cli::cache::CacheCommand::Clean {
                        max_age,
                        max_size,
                        dry_run,
                    },
            },
        ..
    } = cli
    else {
        panic!("expected cache clean command");
    };

    assert_eq!(
        max_age,
        Some(std::time::Duration::from_secs(30 * 24 * 60 * 60))
    );
    assert_eq!(max_size, Some(500_000_000));
    assert!(dry_run);
}

#[test]
fn cache_clear_command_parses() {
    Cli::try_parse_from(["biomcp", "cache", "clear"]).expect("cache clear should parse");
}

#[test]
fn cache_clear_command_parses_with_yes_flag() {
    Cli::try_parse_from(["biomcp", "cache", "clear", "--yes"])
        .expect("cache clear --yes should parse");
}

#[test]
fn cache_path_help_describes_plain_and_typed_json_output() {
    let help = render_cache_path_long_help();

    assert!(help.contains("plain path"));
    assert!(help.contains("--json"));
    assert!(help.contains("cache_path"));
    assert!(!help.contains("ignored"));
}

#[test]
fn cache_stats_help_mentions_json_and_cli_only() {
    let help = render_cache_stats_long_help();

    assert!(help.contains("cache statistics"));
    assert!(help.contains("--json"));
    assert!(help.contains("CLI-only"));
    assert!(help.contains("local filesystem paths"));
}

#[test]
fn cache_clean_help_mentions_dry_run_json_and_limits() {
    let help = render_cache_clean_long_help();

    assert!(help.contains("--max-age"));
    assert!(help.contains("--max-size"));
    assert!(help.contains("--dry-run"));
    assert!(help.contains("--json"));
    assert!(help.contains("orphan"));
}

#[test]
fn cache_clear_help_mentions_yes_tty_and_destructive_scope() {
    let help = render_cache_clear_long_help();

    assert!(help.contains("--yes"));
    assert!(help.contains("TTY"));
    assert!(help.contains("downloads"));
    assert!(help.contains("destructive"));
}

#[test]
fn cache_help_lists_clear_subcommand() {
    let help = render_cache_long_help();

    assert!(help.contains("clear"));
}

#[test]
fn json_cache_path_parses_as_plain_path_command() {
    let cli = parse_built_cli(["biomcp", "--json", "cache", "path"]);

    assert!(cli.json);
    assert!(matches!(
        cli.command,
        Commands::Cache {
            cmd: crate::cli::cache::CacheCommand::Path
        }
    ));
}

#[test]
fn json_cache_stats_parses_as_stats_command() {
    let cli = parse_built_cli(["biomcp", "--json", "cache", "stats"]);

    assert!(cli.json);
    assert!(matches!(
        cli.command,
        Commands::Cache {
            cmd: crate::cli::cache::CacheCommand::Stats
        }
    ));
}
