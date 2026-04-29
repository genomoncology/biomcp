use clap::{CommandFactory, Parser};

use super::{CvxCommand, DdinterCommand, EmaCommand, GtrCommand, WhoCommand, WhoIvdCommand};
use crate::cli::{Cli, Commands, execute};

fn parse_built_cli<I, T>(args: I) -> Cli
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    crate::cli::try_parse_cli(args).expect("args should parse with canonical CLI")
}

#[test]
fn ddinter_sync_parses_subcommand() {
    let cli = parse_built_cli(["biomcp", "ddinter", "sync"]);
    assert!(matches!(
        cli.command,
        Commands::Ddinter {
            cmd: DdinterCommand::Sync
        }
    ));
}

#[test]
fn ddinter_help_mentions_sync_example() {
    let mut command = Cli::command();
    let ddinter = command
        .find_subcommand_mut("ddinter")
        .expect("ddinter subcommand should exist");
    let mut help = Vec::new();
    ddinter
        .write_long_help(&mut help)
        .expect("ddinter help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("biomcp ddinter sync"));
}

#[test]
fn ddinter_sync_help_describes_eight_csv_refresh() {
    let mut command = Cli::command();
    let ddinter = command
        .find_subcommand_mut("ddinter")
        .expect("ddinter subcommand should exist");
    let sync = ddinter
        .find_subcommand_mut("sync")
        .expect("ddinter sync subcommand should exist");
    let mut help = Vec::new();
    sync.write_long_help(&mut help)
        .expect("ddinter sync help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("eight DDInter CSV files"));
}

#[test]
fn ema_sync_parses_subcommand() {
    let cli = parse_built_cli(["biomcp", "ema", "sync"]);
    assert!(matches!(
        cli.command,
        Commands::Ema {
            cmd: EmaCommand::Sync
        }
    ));
}

#[test]
fn ema_help_mentions_sync_example() {
    let mut command = Cli::command();
    let ema = command
        .find_subcommand_mut("ema")
        .expect("ema subcommand should exist");
    let mut help = Vec::new();
    ema.write_long_help(&mut help)
        .expect("ema help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("biomcp ema sync"));
}

#[test]
fn who_sync_parses_subcommand() {
    let cli = parse_built_cli(["biomcp", "who", "sync"]);
    assert!(matches!(
        cli.command,
        Commands::Who {
            cmd: WhoCommand::Sync
        }
    ));
}

#[test]
fn who_help_mentions_sync_example() {
    let mut command = Cli::command();
    let who = command
        .find_subcommand_mut("who")
        .expect("who subcommand should exist");
    let mut help = Vec::new();
    who.write_long_help(&mut help)
        .expect("who help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("biomcp who sync"));
}

#[test]
fn who_sync_help_describes_dual_export_refresh() {
    let mut command = Cli::command();
    let who = command
        .find_subcommand_mut("who")
        .expect("who subcommand should exist");
    let sync = who
        .find_subcommand_mut("sync")
        .expect("who sync subcommand should exist");
    let mut help = Vec::new();
    sync.write_long_help(&mut help)
        .expect("who sync help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("WHO Prequalification local exports"));
}

#[test]
fn cvx_sync_parses_subcommand() {
    let cli = parse_built_cli(["biomcp", "cvx", "sync"]);
    assert!(matches!(
        cli.command,
        Commands::Cvx {
            cmd: CvxCommand::Sync
        }
    ));
}

#[test]
fn cvx_help_mentions_sync_example() {
    let mut command = Cli::command();
    let cvx = command
        .find_subcommand_mut("cvx")
        .expect("cvx subcommand should exist");
    let mut help = Vec::new();
    cvx.write_long_help(&mut help)
        .expect("cvx help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("biomcp cvx sync"));
}

#[test]
fn cvx_sync_help_describes_bundle_refresh() {
    let mut command = Cli::command();
    let cvx = command
        .find_subcommand_mut("cvx")
        .expect("cvx subcommand should exist");
    let sync = cvx
        .find_subcommand_mut("sync")
        .expect("cvx sync subcommand should exist");
    let mut help = Vec::new();
    sync.write_long_help(&mut help)
        .expect("cvx sync help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("CDC CVX/MVX vaccine identity bundle"));
}

#[test]
fn gtr_sync_parses_subcommand() {
    let cli = parse_built_cli(["biomcp", "gtr", "sync"]);
    assert!(matches!(
        cli.command,
        Commands::Gtr {
            cmd: GtrCommand::Sync
        }
    ));
}

#[test]
fn gtr_help_mentions_sync_example() {
    let mut command = Cli::command();
    let gtr = command
        .find_subcommand_mut("gtr")
        .expect("gtr subcommand should exist");
    let mut help = Vec::new();
    gtr.write_long_help(&mut help)
        .expect("gtr help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("biomcp gtr sync"));
}

#[test]
fn gtr_sync_help_describes_diagnostic_bundle_refresh() {
    let mut command = Cli::command();
    let gtr = command
        .find_subcommand_mut("gtr")
        .expect("gtr subcommand should exist");
    let sync = gtr
        .find_subcommand_mut("sync")
        .expect("gtr sync subcommand should exist");
    let mut help = Vec::new();
    sync.write_long_help(&mut help)
        .expect("gtr sync help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("local NCBI GTR diagnostic bundle"));
}

#[test]
fn who_ivd_sync_parses_subcommand() {
    let cli = parse_built_cli(["biomcp", "who-ivd", "sync"]);
    assert!(matches!(
        cli.command,
        Commands::WhoIvd {
            cmd: WhoIvdCommand::Sync
        }
    ));
}

#[test]
fn who_ivd_help_mentions_sync_example() {
    let mut command = Cli::command();
    let who_ivd = command
        .find_subcommand_mut("who-ivd")
        .expect("who-ivd subcommand should exist");
    let mut help = Vec::new();
    who_ivd
        .write_long_help(&mut help)
        .expect("who-ivd help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("biomcp who-ivd sync"));
}

#[test]
fn who_ivd_sync_help_describes_diagnostic_csv_refresh() {
    let mut command = Cli::command();
    let who_ivd = command
        .find_subcommand_mut("who-ivd")
        .expect("who-ivd subcommand should exist");
    let sync = who_ivd
        .find_subcommand_mut("sync")
        .expect("who-ivd sync subcommand should exist");
    let mut help = Vec::new();
    sync.write_long_help(&mut help)
        .expect("who-ivd sync help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("WHO Prequalified IVD diagnostic CSV export"));
}

#[test]
fn discover_help_includes_when_to_use_guidance() {
    let mut command = Cli::command();
    let discover = command
        .find_subcommand_mut("discover")
        .expect("discover subcommand should exist");
    let mut help = Vec::new();
    discover
        .write_long_help(&mut help)
        .expect("discover help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("When to use:"));
    assert!(help.contains("free-text biomedical phrase"));
    assert!(help.contains("single-entity resolver"));
    assert!(help.contains("search all --keyword \"<query>\""));
}

#[test]
fn discover_top_level_command_parses_query() {
    let cli = Cli::try_parse_from(["biomcp", "discover", "ERBB1"]).expect("parse");

    let Cli {
        command: Commands::Discover(crate::cli::system::DiscoverArgs { query }),
        ..
    } = cli
    else {
        panic!("expected discover command");
    };

    assert_eq!(query, "ERBB1");
}

#[test]
fn health_command_parses_apis_only() {
    let cli =
        Cli::try_parse_from(["biomcp", "health", "--apis-only"]).expect("health should parse");

    assert!(matches!(
        cli.command,
        Commands::Health(crate::cli::system::HealthArgs { apis_only: true })
    ));
}

#[test]
fn list_command_parses_entity_name() {
    let cli = Cli::try_parse_from(["biomcp", "list", "drug"]).expect("list should parse");

    let Cli {
        command: Commands::List(crate::cli::system::ListArgs { entity }),
        ..
    } = cli
    else {
        panic!("expected list command");
    };

    assert_eq!(entity.as_deref(), Some("drug"));
}

#[test]
fn batch_command_parses_sections_and_source() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "batch",
        "trial",
        "NCT02576665,NCT02693535",
        "--sections",
        "eligibility,locations",
        "--source",
        "nci",
    ])
    .expect("batch should parse");

    let Cli {
        command:
            Commands::Batch(crate::cli::system::BatchArgs {
                entity,
                ids,
                sections,
                source,
            }),
        ..
    } = cli
    else {
        panic!("expected batch command");
    };

    assert_eq!(entity, "trial");
    assert_eq!(ids, "NCT02576665,NCT02693535");
    assert_eq!(sections.as_deref(), Some("eligibility,locations"));
    assert_eq!(source, "nci");
}

#[test]
fn enrich_command_parses_limit() {
    let cli = Cli::try_parse_from(["biomcp", "enrich", "BRAF,KRAS", "--limit", "5"])
        .expect("enrich should parse");

    let Cli {
        command: Commands::Enrich(crate::cli::system::EnrichArgs { genes, limit }),
        ..
    } = cli
    else {
        panic!("expected enrich command");
    };

    assert_eq!(genes, "BRAF,KRAS");
    assert_eq!(limit, 5);
}

#[test]
fn version_command_parses_verbose_flag() {
    let cli =
        Cli::try_parse_from(["biomcp", "version", "--verbose"]).expect("version should parse");

    assert!(matches!(
        cli.command,
        Commands::Version(crate::cli::system::VersionArgs { verbose: true })
    ));
}

#[tokio::test]
async fn version_json_contract_has_identity_fields() {
    let output = execute(vec![
        "biomcp".to_string(),
        "--json".to_string(),
        "version".to_string(),
    ])
    .await
    .expect("version json should execute");
    let value: serde_json::Value = serde_json::from_str(&output).expect("valid version json");
    let object = value.as_object().expect("version json should be an object");

    assert_eq!(object.len(), 3);
    for field in ["version", "git_revision", "build_timestamp"] {
        assert!(
            object
                .get(field)
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.is_empty()),
            "{field} should be a non-empty string"
        );
    }
}

#[test]
fn serve_http_help_describes_streamable_http() {
    let mut command = crate::cli::build_cli();
    let serve_http = command
        .find_subcommand_mut("serve-http")
        .expect("serve-http subcommand should exist");
    let mut help = Vec::new();
    serve_http
        .write_long_help(&mut help)
        .expect("serve-http help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("Streamable HTTP"));
    assert!(help.contains("/mcp"));
    assert!(help.contains("--host <HOST>"));
    assert!(help.contains("--port <PORT>"));
    assert!(!help.contains("SSE transport"));
    assert!(!help.contains("--json"));
    assert!(!help.contains("--no-cache"));
}

#[test]
fn batch_help_includes_examples_and_limits() {
    let mut command = crate::cli::build_cli();
    let batch = command
        .find_subcommand_mut("batch")
        .expect("batch subcommand should exist");
    let mut help = Vec::new();
    batch
        .write_long_help(&mut help)
        .expect("batch help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("EXAMPLES"));
    assert!(help.contains("biomcp batch article 22663011,24200969"));
    assert!(help.contains("biomcp batch gene BRAF,TP53 --sections pathways,interactions"));
    assert!(help.contains("biomcp batch trial NCT02576665,NCT03715933 --source nci"));
    assert!(help.contains("biomcp batch variant \"BRAF V600E\",\"KRAS G12D\" --json"));
    assert!(help.contains("Batch accepts up to 10 IDs per call."));
    assert!(help.contains("Each call must use a single entity type."));
    assert!(help.contains("See also: biomcp list batch"));
}

#[test]
fn skill_uninstall_is_rejected_before_skill_lookup() {
    let err =
        crate::cli::try_parse_cli(["biomcp", "skill", "uninstall"]).expect_err("should reject");

    assert_eq!(err.kind(), clap::error::ErrorKind::InvalidSubcommand);
    let rendered = err.to_string();
    assert!(rendered.contains("unrecognized subcommand 'uninstall'"));
    assert!(rendered.contains("biomcp uninstall"));
}

#[tokio::test]
async fn handle_enrich_rejects_zero_limit_before_api_call() {
    let cli = Cli::try_parse_from(["biomcp", "enrich", "BRAF,KRAS", "--limit", "0"])
        .expect("enrich should parse");

    let Cli {
        command: Commands::Enrich(args),
        ..
    } = cli
    else {
        panic!("expected enrich command");
    };

    let err = super::handle_enrich(args, false)
        .await
        .expect_err("zero enrich limit should fail fast");
    assert!(err.to_string().contains("--limit must be between 1 and 50"));
}

#[tokio::test]
async fn enrich_rejects_zero_limit_before_api_call() {
    let err = execute(vec![
        "biomcp".to_string(),
        "enrich".to_string(),
        "BRCA1,TP53".to_string(),
        "--limit".to_string(),
        "0".to_string(),
    ])
    .await
    .expect_err("enrich should reject --limit 0");
    assert!(err.to_string().contains("--limit must be between 1 and 50"));
}

#[tokio::test]
async fn enrich_rejects_limit_above_max_before_api_call() {
    let err = execute(vec![
        "biomcp".to_string(),
        "enrich".to_string(),
        "BRCA1,TP53".to_string(),
        "--limit".to_string(),
        "51".to_string(),
    ])
    .await
    .expect_err("enrich should reject --limit > 50");
    assert!(err.to_string().contains("--limit must be between 1 and 50"));
}
