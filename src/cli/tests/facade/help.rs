use super::*;

#[test]
fn skill_help_examples_match_installed_surface() {
    let mut command = Cli::command();
    let skill = command
        .find_subcommand_mut("skill")
        .expect("skill subcommand should exist");
    let mut help = Vec::new();
    skill
        .write_long_help(&mut help)
        .expect("skill help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("biomcp skill            # show skill overview"));
    assert!(help.contains("biomcp skill render     # print canonical agent prompt"));
    assert!(help.contains("biomcp skill status     # compare installed guidance with this binary"));
    assert!(help.contains("biomcp skill install    # install skill to your agent config"));
    assert!(help.contains("Commands:\n  list"));
    assert!(help.contains("render"));
    assert!(!help.contains("biomcp skill 03"));
    assert!(!help.contains("variant-to-treatment"));
    assert!(!help.contains("drug-investigation"));
    assert!(!help.contains("gene-function-lookup"));
    assert!(!help.contains("trial-searching"));
    assert!(!help.contains("literature-synthesis"));
}

#[test]
fn runtime_help_hides_query_only_global_flags() {
    for subcommand_name in crate::cli::RUNTIME_HELP_SUBCOMMANDS {
        let mut command = crate::cli::build_cli();
        let runtime = command
            .find_subcommand_mut(subcommand_name)
            .expect("runtime subcommand should exist");
        let mut help = Vec::new();
        runtime
            .write_long_help(&mut help)
            .expect("runtime help should render");
        let help = String::from_utf8(help).expect("help should be utf-8");

        assert!(
            !help.contains("--json"),
            "{subcommand_name} help should not advertise --json"
        );
        assert!(
            !help.contains("--no-cache"),
            "{subcommand_name} help should not advertise --no-cache"
        );
    }
}

#[test]
fn runtime_commands_still_parse_hidden_global_flags() {
    let cli = parse_built_cli([
        "biomcp",
        "serve-http",
        "--json",
        "--no-cache",
        "--host",
        "127.0.0.1",
        "--port",
        "8080",
    ]);
    assert!(cli.json);
    assert!(cli.no_cache);
    assert!(matches!(
        cli.command,
        Commands::ServeHttp(crate::cli::system::ServeHttpArgs { host, port, .. })
            if host == "127.0.0.1" && port == 8080
    ));

    for args in [
        ["biomcp", "mcp", "--json", "--no-cache"].as_slice(),
        ["biomcp", "serve", "--json", "--no-cache"].as_slice(),
        ["biomcp", "serve-sse", "--json", "--no-cache"].as_slice(),
    ] {
        let cli = parse_built_cli(args);
        assert!(cli.json);
        assert!(cli.no_cache);
    }
}

#[test]
fn serve_sse_help_stays_callable_and_deprecated() {
    let mut command = crate::cli::build_cli();
    let serve_sse = command
        .find_subcommand_mut("serve-sse")
        .expect("serve-sse subcommand should exist");
    let mut help = Vec::new();
    serve_sse
        .write_long_help(&mut help)
        .expect("serve-sse help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("serve-sse"));
    assert!(help.contains("removed"));
    assert!(help.contains("serve-http"));
    assert!(help.contains("/mcp"));
    assert!(!help.contains("--json"));
    assert!(!help.contains("--no-cache"));
}

#[test]
fn top_level_help_hides_serve_sse_but_keeps_serve_http() {
    let mut command = crate::cli::build_cli();
    let mut help = Vec::new();
    command
        .write_long_help(&mut help)
        .expect("top-level help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("serve-http"));
    assert!(!help.contains("serve-sse"));
}

#[test]
fn top_level_help_lists_cache_command() {
    let mut command = crate::cli::build_cli();
    let mut help = Vec::new();
    command
        .write_long_help(&mut help)
        .expect("top-level help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(
        help.lines()
            .any(|line| line.trim_start().starts_with("cache")),
        "top-level help should list the cache family: {help}"
    );
}

#[test]
fn top_level_help_omits_suggest_command() {
    let mut command = crate::cli::build_cli();
    let mut help = Vec::new();
    command
        .write_long_help(&mut help)
        .expect("top-level help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(
        !help
            .lines()
            .any(|line| line.trim_start().starts_with("suggest")),
        "top-level help should omit suggest: {help}"
    );
}

#[test]
fn top_level_help_has_no_cache_path_json_exception() {
    let mut command = crate::cli::build_cli();
    let mut help = Vec::new();
    command
        .write_long_help(&mut help)
        .expect("top-level help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("Output as JSON instead of Markdown"));
    assert!(!help.contains("except biomcp cache path"));
    assert!(!help.contains("stays plain text"));
}

#[test]
fn top_level_help_describes_cache_family_not_path_only() {
    let mut command = crate::cli::build_cli();
    let mut help = Vec::new();
    command
        .write_long_help(&mut help)
        .expect("top-level help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains(
        "Inspect the managed HTTP cache (CLI-only; cache commands reveal workstation-local filesystem paths)"
    ));
    assert!(
        !help
            .contains("Print the managed HTTP cache path (CLI-only; plain text; ignores `--json`)")
    );
}

#[test]
fn top_level_help_uses_count_free_source_phrase() {
    let mut command = crate::cli::build_cli();
    let mut help = Vec::new();
    command
        .write_long_help(&mut help)
        .expect("top-level help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("leading public biomedical data sources"));
    assert!(!help.contains("15 biomedical sources"));
}

#[test]
fn search_all_help_mentions_counts_only_json_contract() {
    let mut command = crate::cli::build_cli();
    let search = command
        .find_subcommand_mut("search")
        .expect("search subcommand should exist");
    let search_all = search
        .find_subcommand_mut("all")
        .expect("search all subcommand should exist");
    let mut help = Vec::new();
    search_all
        .write_long_help(&mut help)
        .expect("search all help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("markdown keeps follow-up links"));
    assert!(help.contains("JSON omits per-section results and links"));
}

#[test]
fn discover_help_mentions_article_search_fallback_for_non_canonical_queries() {
    let mut command = Cli::command();
    let discover = command
        .find_subcommand_mut("discover")
        .expect("discover subcommand should exist");
    let mut help = Vec::new();
    discover
        .write_long_help(&mut help)
        .expect("discover help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains(
        "When discover cannot resolve a canonical biomedical concept, it suggests article search instead of leaving an empty dead end."
    ));
}
