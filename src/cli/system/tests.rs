use clap::{CommandFactory, Parser};

use super::{CvxCommand, DdinterCommand, EmaCommand, GtrCommand, WhoCommand, WhoIvdCommand};
use crate::cli::{Cli, Commands, execute};

mod dev2_contracts;

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
        command:
            Commands::Discover(crate::cli::system::DiscoverArgs {
                query,
                limit,
                offset,
                full,
            }),
        ..
    } = cli
    else {
        panic!("expected discover command");
    };

    assert_eq!(query, "ERBB1");
    assert_eq!(limit, 5);
    assert_eq!(offset, 0);
    assert!(!full);
}

#[test]
fn health_command_parses_apis_only() {
    let cli =
        Cli::try_parse_from(["biomcp", "health", "--apis-only"]).expect("health should parse");

    assert!(matches!(
        cli.command,
        Commands::Health(crate::cli::system::HealthArgs {
            apis_only: true,
            ..
        })
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
fn rendered_list_help_names_exactly_the_production_catalog_entities() {
    let mut command = Cli::command();
    let list = command.find_subcommand_mut("list").expect("list command");
    let mut help = Vec::new();
    list.write_long_help(&mut help).expect("render list help");
    let help = String::from_utf8(help).expect("UTF-8 help");
    let overwide = help
        .lines()
        .filter(|line| line.chars().count() > 160)
        .collect::<Vec<_>>();
    assert!(overwide.is_empty(), "overwide list help: {overwide:?}");
    let mut lines = help.lines();
    let entity_line = lines
        .find(|line| line.trim_start().starts_with("[ENTITY]"))
        .expect("rendered [ENTITY] argument block");
    let entity_block = std::iter::once(entity_line)
        .chain(lines.take_while(|line| !line.trim().is_empty()))
        .collect::<Vec<_>>()
        .join(" ");
    let names = entity_block
        .rsplit_once('(')
        .expect("final parenthesized canonical values")
        .1
        .rsplit_once(')')
        .expect("canonical values closing parenthesis")
        .0;
    let mut help_entities = names
        .split(',')
        .map(|name| name.trim().to_ascii_lowercase())
        .collect::<std::collections::BTreeSet<_>>();
    for page in ["search-all", "discover", "batch", "enrich", "skill"] {
        assert!(help_entities.remove(page), "missing list page {page}");
    }
    let catalog_entities = crate::cli::list::catalog::entities()
        .into_iter()
        .map(|entity| entity.name.to_string())
        .collect();
    assert_eq!(help_entities, catalog_entities);
}

#[test]
fn discover_and_batch_json_keep_executable_templates_without_human_option_prose() {
    for page in ["discover", "batch"] {
        let out = crate::cli::list::render_json(Some(page)).expect("typed list page");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let templates = value["entries"]
            .as_array()
            .expect("entries")
            .iter()
            .filter_map(|entry| entry.get("template").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>();
        assert!(!templates.is_empty(), "missing {page} template");
        for template in templates {
            let command = template
                .replace("<query>", "BRCA1")
                .replace("<entity>", "gene")
                .replace("<ids>", "BRAF,TP53");
            let args = std::iter::once("biomcp".to_string())
                .chain(shlex::split(&command).expect("valid template shell syntax"));
            crate::cli::try_parse_cli(args)
                .unwrap_or_else(|error| panic!("invalid {page} template `{command}`: {error}"));
        }
        assert!(!out.contains("structured-output budget"));
        assert!(!out.contains("adverse-event batches do not support"));
    }
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
fn enrich_json_always_serializes_unresolved_genes() {
    let response = super::dispatch::EnrichResponse {
        genes: vec!["BRAF".into()],
        unresolved_genes: Vec::new(),
        count: 0,
        results: Vec::new(),
    };

    let value = serde_json::to_value(response).expect("enrich response should serialize");
    assert_eq!(value["unresolved_genes"], serde_json::json!([]));
}

#[test]
fn enrich_markdown_reports_unresolved_genes_before_results() {
    let terms = vec![crate::sources::gprofiler::GProfilerTerm {
        native: Some("R-HSA-1".into()),
        name: Some("Example".into()),
        source: Some("REAC".into()),
        p_value: Some(0.01),
    }];
    let output = super::dispatch::enrich_markdown(
        &["BRAF".into(), "ZZQQXX1".into()],
        &terms,
        &["ZZQQXX1".into()],
    );

    let unresolved = output.find("Unresolved genes: ZZQQXX1").unwrap();
    let table = output.find("| Source | ID | Name | p-value |").unwrap();
    assert!(unresolved < table);
}

#[test]
fn enrich_markdown_reports_unresolved_genes_before_empty_result() {
    let output = super::dispatch::enrich_markdown(&["ZZQQXX1".into()], &[], &["ZZQQXX1".into()]);

    let unresolved = output.find("Unresolved genes: ZZQQXX1").unwrap();
    let empty = output.find("No enriched terms found.").unwrap();
    assert!(unresolved < empty);
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

#[test]
fn clap_version_includes_the_build_version() {
    let command = crate::cli::build_cli();

    assert_eq!(
        command.render_version().to_string(),
        format!("biomcp {}\n", crate::build_identity::current().version)
    );
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
    assert_eq!(object["version"], crate::build_identity::current().version);
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
    assert!(help.contains("--allowed-hosts <ALLOWED_HOSTS>"));
    assert!(help.contains("--unsafe-allow-any-host"));
    assert!(help.contains("does not add authentication or encryption"));
    assert!(help.contains("65,536 bytes"));
    assert!(!help.contains("SSE transport"));
    assert!(!help.contains("--json"));
    assert!(!help.contains("--no-cache"));
}

#[test]
fn serve_http_host_safety_flags_conflict() {
    let error = crate::cli::try_parse_cli([
        "biomcp",
        "serve-http",
        "--allowed-hosts",
        "example.com",
        "--unsafe-allow-any-host",
    ])
    .expect_err("host allowlist and unsafe bypass must conflict");
    assert!(error.to_string().contains("cannot be used with"));
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
    assert!(help.contains("biomcp batch article 22663011,24200969 --mode compact"));
    assert!(help.contains("--mode detail --sections tldr"));
    assert!(help.contains("biomcp batch gene BRAF,TP53 --sections pathways,interactions"));
    assert!(help.contains("biomcp batch trial NCT02576665,NCT03715933 --source nci"));
    assert!(help.contains("biomcp batch variant \"BRAF V600E\",\"KRAS G12D\" --json"));
    assert!(help.contains("Article compact mode accepts up to 20 IDs"));
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

#[cfg(unix)]
#[test]
fn uninstall_removes_exactly_the_owned_binary_and_receipt() {
    use crate::cli::install::{
        INSTALLER_IDENTITY, InstallReceipt, RECEIPT_SCHEMA_VERSION, ReceiptState, receipt_path,
        sha256_file, write_receipt_atomic,
    };
    use crate::test_support::TempDirGuard;

    let root = TempDirGuard::new("uninstall-owned");
    let executable = root.path().join("biomcp");
    std::fs::write(&executable, b"owned").unwrap();
    let executable = std::fs::canonicalize(executable).unwrap();
    let receipt_path = receipt_path(&executable).unwrap();
    write_receipt_atomic(
        &receipt_path,
        &InstallReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            installer: INSTALLER_IDENTITY.into(),
            state: ReceiptState::Installed,
            executable_path: executable.clone(),
            version: "1.0.0".into(),
            sha256: sha256_file(&executable).unwrap(),
            transaction_nonce: None,
            old_version: None,
            old_sha256: None,
            new_version: None,
            new_sha256: None,
        },
    )
    .unwrap();

    let message = super::dispatch::uninstall_owned_at(&executable).unwrap();
    assert!(message.contains(executable.to_string_lossy().as_ref()));
    assert!(!executable.exists());
    assert!(!receipt_path.exists());
    assert!(matches!(
        super::dispatch::uninstall_owned_at(&executable),
        Err(crate::error::BioMcpError::NotInstalled { .. })
    ));
}
