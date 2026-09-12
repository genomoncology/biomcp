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

#[test]
fn reversed_search_names_report_copyable_canonical_commands() {
    for entity in [
        "all",
        "author",
        "gene",
        "disease",
        "diagnostic",
        "pgx",
        "phenotype",
        "gwas",
        "article",
        "trial",
        "variant",
        "drug",
        "pathway",
        "protein",
        "adverse-event",
    ] {
        let args = ["caller", entity, "search", "--help"].map(std::ffi::OsString::from);
        let rendered = super::super::reversed_search_correction(&args)
            .expect("allowlisted reversal has a structurally valid correction");
        let command = format!("biomcp search {entity} --help");
        assert!(
            rendered.contains(&format!("reversed search syntax; use `{command}`")),
            "entity={entity}: {rendered}"
        );
        assert_eq!(
            shlex::split(&command).unwrap(),
            ["biomcp", "search", entity, "--help"]
        );
        super::super::build_cli()
            .try_get_matches_from(["biomcp", "search", entity, "--help"])
            .expect_err("raw Clap returns display help")
            .kind()
            .eq(&clap::error::ErrorKind::DisplayHelp)
            .then_some(())
            .expect("corrected argv is structurally valid help");
    }

    for entity in ["gene", "drug", "variant"] {
        let error = super::super::try_parse_cli(["biomcp", entity, "search", "--help"])
            .expect_err("external-subcommand catchalls must not bypass correction");
        assert!(error.to_string().contains(&format!(
            "reversed search syntax; use `biomcp search {entity} --help`"
        )));
    }

    for args in [
        ["biomcp", "gene", "BRAF"],
        ["biomcp", "drug", "imatinib"],
        ["biomcp", "variant", "BRAF V600E"],
    ] {
        assert!(
            super::super::try_parse_cli(args).is_ok(),
            "genuine external-subcommand shorthand changed: {args:?}"
        );
    }
}

#[test]
fn reversed_search_preserves_globals_delimiters_and_hostile_data() {
    let args = [
        "caller",
        "--json",
        "article",
        "--no-cache",
        "search",
        "--keyword",
        " $(touch nope) `ticks` a;b x>y ",
    ];
    let error = super::super::try_parse_cli(args).expect_err("reversal is a correction");
    let text = error.to_string();
    let marker = "reversed search syntax; use ";
    let span = text.split_once(marker).unwrap().1.lines().next().unwrap();
    let fence_len = span.bytes().take_while(|byte| *byte == b'`').count();
    let command = &span[fence_len..span.len() - fence_len];
    let command = if command.starts_with(' ') && command.ends_with(' ') {
        &command[1..command.len() - 1]
    } else {
        command
    };
    assert_eq!(
        shlex::split(command).unwrap(),
        [
            "biomcp",
            "--json",
            "search",
            "--no-cache",
            "article",
            "--keyword",
            " $(touch nope) `ticks` a;b x>y ",
        ]
    );
}

#[test]
fn reversed_search_detector_preserves_baseline_boundaries() {
    for args in [
        vec!["biomcp", "Article", "search"],
        vec!["biomcp", "article", "Search"],
        vec!["biomcp", "--", "article", "search"],
    ] {
        let baseline = super::super::build_cli()
            .try_get_matches_from(args.clone())
            .expect_err("invalid baseline command")
            .to_string();
        let error = super::super::try_parse_cli(args).expect_err("invalid baseline command");
        assert!(!error.to_string().contains("reversed search syntax"));
        assert_eq!(error.to_string(), baseline);
    }
    assert!(super::super::try_parse_cli(["biomcp", "search", "article"]).is_ok());
    assert!(super::super::try_parse_cli(["biomcp", "get", "article", "search"]).is_ok());

    let at_entry_cap = std::iter::once("biomcp".to_string())
        .chain(["article".into(), "search".into(), "--keyword".into()])
        .chain(std::iter::repeat_n("x".into(), 252))
        .collect::<Vec<_>>();
    assert_eq!(at_entry_cap.len(), 256);
    assert!(
        super::super::try_parse_cli(at_entry_cap)
            .unwrap_err()
            .to_string()
            .contains("reversed search syntax")
    );
    let over_entry_cap = std::iter::once("biomcp".to_string())
        .chain(["article".into(), "search".into(), "--keyword".into()])
        .chain(std::iter::repeat_n("x".into(), 253))
        .collect::<Vec<_>>();
    assert!(
        !super::super::try_parse_cli(over_entry_cap)
            .unwrap_err()
            .to_string()
            .contains("reversed search syntax")
    );
}

#[test]
fn shared_pagination_and_section_provenance_regressions() {
    let total = super::super::shared::PaginationMeta::cursor(
        4_000,
        5,
        0,
        Some(3_738),
        Some("stale".into()),
    );
    assert!(!total.has_more);
    assert_eq!(total.next_page_token, None);
    let without_token = super::super::shared::PaginationMeta::cursor(0, 5, 5, Some(10), None);
    assert!(!without_token.has_more);
    assert_eq!(without_token.next_page_token, None);

    let meta = super::super::shared::search_meta_with_section_sources(
        Vec::new(),
        vec![crate::render::provenance::SectionSource {
            key: "faers".to_string(),
            label: "Adverse events (OpenFDA FAERS)".to_string(),
            outcome: crate::entities::section_outcome::SectionOutcomeState::Unavailable,
            sources: Vec::new(),
        }],
    )
    .expect("section provenance should create metadata");
    let value = serde_json::to_value(meta).expect("metadata JSON");
    assert_eq!(value["section_sources"][0]["key"], "faers");
    assert_eq!(value["section_sources"][0]["outcome"], "unavailable");
}

#[test]
fn reversed_search_global_delimiter_and_display_precedence_matrix() {
    for global in ["--json", "-j", "--no-cache"] {
        for args in [
            vec!["biomcp", global, "article", "search", "--help"],
            vec!["biomcp", "article", global, "search", "--help"],
            vec!["biomcp", "article", "search", global, "--help"],
        ] {
            let mut candidate = args.clone();
            let positions = candidate
                .iter()
                .enumerate()
                .skip(1)
                .filter(|(_, value)| !["--json", "-j", "--no-cache"].contains(value))
                .take(2)
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            candidate.swap(positions[0], positions[1]);
            let command = shlex::try_join(candidate).unwrap();
            let args = args
                .into_iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>();
            assert_eq!(
                super::super::reversed_search_correction(&args),
                Some(format!("reversed search syntax; use `{command}`"))
            );
        }
    }
    for (args, command) in [
        (
            vec!["biomcp", "--json", "article", "search", "--help"],
            "biomcp --json search article --help",
        ),
        (
            vec!["biomcp", "article", "-j", "search", "--help"],
            "biomcp search -j article --help",
        ),
        (
            vec!["biomcp", "article", "search", "--no-cache", "--help"],
            "biomcp search article --no-cache --help",
        ),
        (
            vec![
                "biomcp",
                "--json",
                "article",
                "--no-cache",
                "search",
                "--",
                "BRAF",
            ],
            "biomcp --json search --no-cache article -- BRAF",
        ),
        (
            vec!["biomcp", "article", "search", "--", "--json"],
            "biomcp search article -- --json",
        ),
    ] {
        let args = args
            .into_iter()
            .map(std::ffi::OsString::from)
            .collect::<Vec<_>>();
        assert_eq!(
            super::super::reversed_search_correction(&args),
            Some(format!("reversed search syntax; use `{command}`"))
        );
    }
    let after_delimiter = ["biomcp", "--", "article", "search"].map(std::ffi::OsString::from);
    assert!(super::super::reversed_search_correction(&after_delimiter).is_none());

    for args in [
        vec!["biomcp", "--help"],
        vec!["biomcp", "article", "--help"],
        vec!["biomcp", "--version"],
        vec!["biomcp", "-V"],
        vec!["biomcp", "gene", "--help"],
        vec!["biomcp", "drug", "--version"],
        vec!["biomcp", "variant", "-V"],
    ] {
        let expected = super::super::build_cli()
            .try_get_matches_from(args.clone())
            .expect_err("display request")
            .to_string();
        assert_eq!(
            super::super::try_parse_cli(args).unwrap_err().to_string(),
            expected
        );
    }
    for flag in ["--help", "-h"] {
        let error = super::super::try_parse_cli(["biomcp", "article", "search", flag])
            .expect_err("reversed display flag remains a correction");
        assert!(error.to_string().contains(&format!(
            "reversed search syntax; use `biomcp search article {flag}`"
        )));
    }
    for flag in ["--version", "-V"] {
        let args = ["biomcp", "article", "search", flag];
        let baseline = super::super::build_cli()
            .try_get_matches_from(args)
            .unwrap_err()
            .to_string();
        assert_eq!(
            super::super::try_parse_cli(args).unwrap_err().to_string(),
            baseline
        );
    }
}

#[test]
fn reversed_search_candidate_errors_and_byte_caps_preserve_baseline() {
    for entity in [
        "all",
        "author",
        "disease",
        "diagnostic",
        "pgx",
        "phenotype",
        "gwas",
        "article",
        "trial",
        "pathway",
        "protein",
        "adverse-event",
    ] {
        let args = vec!["biomcp", entity, "search", "--not-a-search-flag"];
        let expected = super::super::build_cli()
            .try_get_matches_from(args.clone())
            .expect_err("baseline parse error")
            .to_string();
        assert_eq!(
            super::super::try_parse_cli(args).unwrap_err().to_string(),
            expected,
            "entity={entity}"
        );
    }
    for args in [
        vec!["biomcp", "article", "search", "--limit", "not-a-number"],
        vec!["biomcp", "unknown", "search"],
        vec!["biomcp", "ARTICLE", "search"],
        vec!["biomcp", "article", "SEARCH"],
        vec!["biomcp", "get", "--query", "search"],
    ] {
        let expected = super::super::build_cli()
            .try_get_matches_from(args.clone())
            .expect_err("baseline parse error")
            .to_string();
        assert_eq!(
            super::super::try_parse_cli(args).unwrap_err().to_string(),
            expected
        );
    }

    let at_payload = "x".repeat(16_356);
    let at =
        ["biomcp", "article", "search", "--keyword", &at_payload].map(std::ffi::OsString::from);
    assert_eq!(at.iter().map(|arg| arg.len()).sum::<usize>(), 16_384);
    assert!(super::super::reversed_search_correction(&at).is_some());
    let over_payload = "x".repeat(16_357);
    let over =
        ["biomcp", "article", "search", "--keyword", &over_payload].map(std::ffi::OsString::from);
    assert!(super::super::reversed_search_correction(&over).is_none());

    let control = ["biomcp", "article", "search", "--keyword", "bad\u{85}value"]
        .map(std::ffi::OsString::from);
    assert!(super::super::reversed_search_correction(&control).is_none());
}

#[test]
fn reversed_search_catchalls_fall_back_without_echo_when_correction_is_suppressed() {
    fn assert_fallback(
        args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
        entity: &str,
    ) {
        let error = super::super::try_parse_cli(args).expect_err("catchall reversal must stop");
        assert_eq!(
            error.to_string(),
            format!(
                "error: reversed search syntax; use `biomcp search {entity}`; the supplied search arguments were not accepted\n\nUsage: biomcp [OPTIONS] <COMMAND>\n\nFor more information, try '--help'.\n"
            )
        );
    }

    for entity in ["gene", "drug", "variant"] {
        assert!(
            super::super::build_cli()
                .try_get_matches_from(["biomcp", entity, "search", "--not-a-search-flag", "secret"])
                .is_ok(),
            "the original catchall parse must be the success being intercepted"
        );
        assert_fallback(
            ["biomcp", entity, "search", "--not-a-search-flag", "secret"],
            entity,
        );
    }

    let over_entries = std::iter::once("biomcp".to_string())
        .chain(["gene".into(), "search".into()])
        .chain(std::iter::repeat_n("secret".into(), 254))
        .collect::<Vec<_>>();
    assert_eq!(over_entries.len(), 257);
    assert_fallback(over_entries, "gene");

    let fixed_bytes = ["biomcp", "drug", "search", "--query"]
        .iter()
        .map(|value| value.len())
        .sum::<usize>();
    let over_bytes = "x".repeat(16_385 - fixed_bytes);
    assert_fallback(["biomcp", "drug", "search", "--query", &over_bytes], "drug");
    assert_fallback(
        ["biomcp", "variant", "search", "bad\u{85}secret"],
        "variant",
    );

    let output_overflow = "'$".repeat(5_453);
    assert!(output_overflow.len() < 16_384);
    assert_fallback(
        ["biomcp", "gene", "search", "--query", &output_overflow],
        "gene",
    );
}

#[cfg(unix)]
#[test]
fn reversed_search_catchall_non_utf8_uses_safe_fallback() {
    use std::os::unix::ffi::OsStringExt;
    let args = vec![
        std::ffi::OsString::from("biomcp"),
        std::ffi::OsString::from("variant"),
        std::ffi::OsString::from("search"),
        std::ffi::OsString::from_vec(vec![0xff]),
    ];
    let error = super::super::try_parse_cli(args).expect_err("non-UTF-8 catchall reversal");
    assert!(error.to_string().contains(
        "reversed search syntax; use `biomcp search variant`; the supplied search arguments were not accepted"
    ));
}

#[cfg(unix)]
#[test]
fn reversed_search_rejects_non_utf8_before_candidate_parsing() {
    use std::os::unix::ffi::OsStringExt;
    let args = vec![
        std::ffi::OsString::from("biomcp"),
        std::ffi::OsString::from("article"),
        std::ffi::OsString::from("search"),
        std::ffi::OsString::from_vec(vec![0xff]),
    ];
    assert!(super::super::reversed_search_correction(&args).is_none());
}

#[test]
fn reversed_search_complete_sentence_cap_is_inclusive() {
    fn sentence(payload: &str) -> String {
        let command =
            shlex::try_join(["biomcp", "search", "article", "--keyword", payload]).unwrap();
        format!(
            "reversed search syntax; use {}",
            crate::render::markdown::markdown_command_code_span(&command)
        )
    }

    let exact = "'$".repeat(5_451);
    let over = format!("{exact}x");
    assert_eq!(sentence(&exact).len(), 32_768);
    assert_eq!(sentence(&over).len(), 32_769);
    assert!(exact.len() <= 16_356 && over.len() <= 16_356);
    let args = |payload: &str| {
        ["biomcp", "article", "search", "--keyword", payload].map(std::ffi::OsString::from)
    };
    assert!(super::super::reversed_search_correction(&args(&exact)).is_some());
    assert!(super::super::reversed_search_correction(&args(&over)).is_none());
}
