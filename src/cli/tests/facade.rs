use clap::{CommandFactory, FromArgMatches, Parser};

use super::super::{ChartArgs, Cli, Commands, execute};

mod cache;
mod chart;
mod help;

#[tokio::test]
async fn search_all_requires_at_least_one_typed_slot() {
    let err = execute(vec![
        "biomcp".to_string(),
        "search".to_string(),
        "all".to_string(),
    ])
    .await
    .expect_err("search all should require typed slots");
    assert!(err.to_string().contains("at least one typed slot"));
    assert!(err.to_string().contains("--gene"));
}

fn render_cache_path_long_help() -> String {
    let mut command = super::super::build_cli();
    let cache = command
        .find_subcommand_mut("cache")
        .expect("cache subcommand should exist");
    let path = cache
        .find_subcommand_mut("path")
        .expect("cache path subcommand should exist");
    let mut help = Vec::new();
    path.write_long_help(&mut help)
        .expect("cache path help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

fn render_cache_long_help() -> String {
    let mut command = super::super::build_cli();
    let cache = command
        .find_subcommand_mut("cache")
        .expect("cache subcommand should exist");
    let mut help = Vec::new();
    cache
        .write_long_help(&mut help)
        .expect("cache help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

fn render_cache_stats_long_help() -> String {
    let mut command = super::super::build_cli();
    let cache = command
        .find_subcommand_mut("cache")
        .expect("cache subcommand should exist");
    let stats = cache
        .find_subcommand_mut("stats")
        .expect("cache stats subcommand should exist");
    let mut help = Vec::new();
    stats
        .write_long_help(&mut help)
        .expect("cache stats help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

fn render_cache_clean_long_help() -> String {
    let mut command = super::super::build_cli();
    let cache = command
        .find_subcommand_mut("cache")
        .expect("cache subcommand should exist");
    let clean = cache
        .find_subcommand_mut("clean")
        .expect("cache clean subcommand should exist");
    let mut help = Vec::new();
    clean
        .write_long_help(&mut help)
        .expect("cache clean help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

fn render_cache_clear_long_help() -> String {
    let mut command = super::super::build_cli();
    let cache = command
        .find_subcommand_mut("cache")
        .expect("cache subcommand should exist");
    let clear = cache
        .find_subcommand_mut("clear")
        .expect("cache clear subcommand should exist");
    let mut help = Vec::new();
    clear
        .write_long_help(&mut help)
        .expect("cache clear help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

fn parse_built_cli<I, T>(args: I) -> Cli
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = super::super::build_cli()
        .try_get_matches_from(args)
        .expect("args should parse with canonical CLI");
    Cli::from_arg_matches(&matches).expect("matches should decode into Cli")
}
