//! CLI outcome execution seam and MCP chart argument rewriting.

use super::response_contract::{
    JsonResponseContract, command_requests_json, finalize_structured_error,
};
use super::skill::SkillCommand;
use super::{Cli, CliOutput, CommandOutcome, Commands, GetEntity, SearchEntity, StudyCommand};
use std::io::IsTerminal;
fn bio_mcp_error_exit_code(error: &crate::error::BioMcpError) -> u8 {
    error.exit_code()
}
fn outcome_to_string(outcome: CommandOutcome) -> anyhow::Result<String> {
    if outcome.exit_code == 0 {
        if outcome.bytes.is_some() {
            anyhow::bail!("binary output cannot be represented as text");
        }
        Ok(outcome.text)
    } else {
        anyhow::bail!("{}", outcome.text)
    }
}

fn outcome_to_mcp_output(outcome: CommandOutcome) -> anyhow::Result<CliOutput> {
    if outcome.bytes.is_some() {
        anyhow::bail!("binary downloads are CLI-only and cannot be returned as MCP text");
    }
    Ok(CliOutput {
        text: outcome.text,
        metadata_json: outcome.metadata_json,
        svg: outcome.svg,
    })
}
fn mcp_output_flag_error() -> crate::error::BioMcpError {
    crate::error::BioMcpError::InvalidArgument(
        "MCP chart responses do not support --output/-o. Omit file output and consume the inline SVG image content instead.".into(),
    )
}

fn require_json_document(mut outcome: CommandOutcome) -> CommandOutcome {
    if outcome.exit_code == 0
        && outcome.stream == super::OutputStream::Stdout
        && outcome.bytes.is_none()
        && serde_json::from_str::<serde_json::Value>(&outcome.text).is_err()
    {
        let error = crate::error::BioMcpError::InternalProcessing;
        outcome.text = crate::render::json::to_error_json(&error)
            .expect("static JSON contract error must serialize");
        outcome.exit_code = error.exit_code();
    }
    outcome
}

pub fn server_json_rejection() -> CommandOutcome {
    let error = crate::error::BioMcpError::InvalidArgument(
        "--json cannot be used with a long-running MCP server command".into(),
    );
    CommandOutcome::stdout_with_exit(
        crate::render::json::to_error_json(&error)
            .expect("static server JSON rejection must serialize"),
        error.exit_code(),
    )
}

fn is_charted_mcp_study_command(cli: &Cli) -> Result<bool, crate::error::BioMcpError> {
    let chart = match &cli.command {
        Commands::Study {
            cmd:
                StudyCommand::Query { chart, .. }
                | StudyCommand::Survival { chart, .. }
                | StudyCommand::Compare { chart, .. }
                | StudyCommand::CoOccurrence { chart, .. },
        } => chart,
        _ => return Ok(false),
    };

    if chart.chart.is_none() || cli.json {
        return Ok(false);
    }
    if chart.output.is_some() {
        return Err(mcp_output_flag_error());
    }
    Ok(true)
}

fn prepare_mcp_chart(cli: &mut Cli) -> Result<(), crate::error::BioMcpError> {
    let chart = match &mut cli.command {
        Commands::Study {
            cmd:
                StudyCommand::Query { chart, .. }
                | StudyCommand::Survival { chart, .. }
                | StudyCommand::Compare { chart, .. }
                | StudyCommand::CoOccurrence { chart, .. },
        } => chart,
        _ => return Ok(()),
    };
    if chart.chart.is_none() || cli.json {
        return Ok(());
    }
    if chart.output.is_some() {
        return Err(mcp_output_flag_error());
    }
    if chart.cols.is_some() || chart.rows.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            crate::render::chart::TERMINAL_SIZE_FLAGS_ERROR.into(),
        ));
    }
    if chart.scale.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            crate::render::chart::PNG_SCALE_FLAGS_ERROR.into(),
        ));
    }
    chart.mcp_inline = true;
    Ok(())
}

pub async fn run(cli: Cli) -> anyhow::Result<String> {
    let Cli {
        command,
        json,
        no_cache,
    } = cli;

    crate::sources::with_no_cache(no_cache, async move {
        match command {
            Commands::Get {
                entity: GetEntity::Author(args),
            } => outcome_to_string(super::author::handle_get(args, json).await?),
            Commands::Get {
                entity: GetEntity::Gene(args),
            } => outcome_to_string(super::gene::handle_get(args, json, false).await?),
            Commands::Get {
                entity: GetEntity::Article(args),
            } => outcome_to_string(super::article::handle_get(args, json, false).await?),
            Commands::Get {
                entity: GetEntity::Disease(args),
            } => outcome_to_string(super::disease::handle_get(args, json).await?),
            Commands::Get {
                entity: GetEntity::Diagnostic(args),
            } => outcome_to_string(super::diagnostic::handle_get(args, json).await?),
            Commands::Get {
                entity: GetEntity::Pgx(args),
            } => outcome_to_string(super::pgx::handle_get(args, json).await?),
            Commands::Get {
                entity: GetEntity::Trial(args),
            } => outcome_to_string(super::trial::handle_get(args, json).await?),
            Commands::Get {
                entity: GetEntity::Variant(args),
            } => outcome_to_string(super::variant::handle_get(args, json, false).await?),
            Commands::Get {
                entity: GetEntity::Drug(args),
            } => outcome_to_string(super::drug::handle_get(args, json, false).await?),
            Commands::Get {
                entity: GetEntity::Pathway(args),
            } => outcome_to_string(super::pathway::handle_get(args, json).await?),
            Commands::Get {
                entity: GetEntity::Protein(args),
            } => outcome_to_string(super::protein::handle_get(args, json).await?),
            Commands::Get {
                entity: GetEntity::AdverseEvent(args),
            } => outcome_to_string(super::adverse_event::handle_get(args, json).await?),
            Commands::Variant { cmd } => {
                outcome_to_string(super::variant::handle_command(cmd, json).await?)
            }
            Commands::Drug { cmd } => {
                outcome_to_string(super::drug::handle_command(cmd, json, false).await?)
            }
            Commands::Disease { cmd } => {
                outcome_to_string(super::disease::handle_command(cmd, json).await?)
            }
            Commands::Article { cmd } => {
                outcome_to_string(super::article::handle_command(cmd, json).await?)
            }
            Commands::Gene { cmd } => {
                outcome_to_string(super::gene::handle_command(cmd, json, false).await?)
            }
            Commands::Pathway { cmd } => {
                outcome_to_string(super::pathway::handle_command(cmd, json).await?)
            }
            Commands::Protein { cmd } => {
                outcome_to_string(super::protein::handle_command(cmd, json).await?)
            }
            Commands::Study { cmd } => {
                outcome_to_string(super::study::handle_command(cmd, json).await?)
            }
            Commands::Batch(args) => {
                outcome_to_string(super::system::handle_batch(args, json).await?)
            }
            Commands::Search { entity } => match entity {
                SearchEntity::Author(args) => {
                    outcome_to_string(super::author::handle_search(args, json).await?)
                }
                SearchEntity::All(args) => {
                    let keyword = super::resolve_query_input(
                        args.keyword,
                        args.positional_query,
                        "--keyword",
                    )?;
                    let input = crate::cli::search_all::SearchAllInput {
                        gene: args.gene,
                        variant: args.variant,
                        disease: args.disease,
                        drug: args.drug,
                        keyword,
                        since: args.since,
                        limit: args.limit,
                        counts_only: args.counts_only,
                        debug_plan: args.debug_plan,
                    };
                    let results = crate::cli::search_all::dispatch(&input).await?;
                    if json {
                        if input.counts_only {
                            Ok(crate::render::json::to_pretty(
                                &crate::cli::search_all::counts_only_json(&results),
                            )?)
                        } else {
                            Ok(crate::render::json::to_pretty(&results)?)
                        }
                    } else {
                        Ok(crate::render::markdown::search_all_markdown(
                            &results,
                            input.counts_only,
                        )?)
                    }
                }
                SearchEntity::Gene(args) => {
                    outcome_to_string(super::gene::handle_search(args, json, false).await?)
                }
                SearchEntity::Disease(args) => {
                    outcome_to_string(super::disease::handle_search(args, json).await?)
                }
                SearchEntity::Diagnostic(args) => {
                    outcome_to_string(super::diagnostic::handle_search(args, json).await?)
                }
                SearchEntity::Pgx(args) => {
                    outcome_to_string(super::pgx::handle_search(args, json).await?)
                }
                SearchEntity::Phenotype(args) => {
                    outcome_to_string(super::phenotype::handle_search(args, json).await?)
                }
                SearchEntity::Gwas(args) => {
                    outcome_to_string(super::gwas::handle_search(args, json).await?)
                }
                SearchEntity::Article(args) => {
                    outcome_to_string(super::article::handle_search(args, json).await?)
                }
                SearchEntity::Trial(args) => {
                    outcome_to_string(super::trial::handle_search(args, json).await?)
                }
                SearchEntity::Variant(args) => {
                    outcome_to_string(super::variant::handle_search(args, json, false).await?)
                }
                SearchEntity::Drug(args) => {
                    outcome_to_string(super::drug::handle_search(args, json).await?)
                }
                SearchEntity::Pathway(args) => {
                    outcome_to_string(super::pathway::handle_search(args, json).await?)
                }
                SearchEntity::Protein(args) => {
                    outcome_to_string(super::protein::handle_search(args, json).await?)
                }
                SearchEntity::AdverseEvent(args) => {
                    outcome_to_string(super::adverse_event::handle_search(args, json).await?)
                }
            },
            Commands::Health(super::system::HealthArgs { apis_only }) => {
                let report = crate::cli::health::check(apis_only).await?;
                if json {
                    Ok(crate::render::json::to_pretty(&report)?)
                } else {
                    Ok(report.to_markdown())
                }
            }
            Commands::Cache { cmd } => match cmd {
                super::cache::CacheCommand::Path => {
                    let path = crate::cli::cache::render_path()?.trim().to_string();
                    if json {
                        Ok(crate::render::json::to_pretty(&serde_json::json!({
                            "kind": "cache_path",
                            "path": path,
                        }))?)
                    } else {
                        Ok(path)
                    }
                }
                super::cache::CacheCommand::Stats => {
                    let report = crate::cli::cache::collect_cache_stats_report()?;
                    if json {
                        Ok(crate::render::json::to_pretty(&report)?)
                    } else {
                        Ok(report.to_markdown())
                    }
                }
                super::cache::CacheCommand::Clean {
                    max_age,
                    max_size,
                    dry_run,
                } => {
                    let report = crate::cli::cache::execute_clean(max_age, max_size, dry_run)?;
                    if json {
                        Ok(crate::render::json::to_pretty(&report)?)
                    } else {
                        Ok(crate::cli::cache::render_clean_text(&report))
                    }
                }
                super::cache::CacheCommand::Clear { .. } => {
                    Err(crate::error::BioMcpError::InvalidArgument(
                        "cache clear must be executed through run_outcome()".into(),
                    )
                    .into())
                }
            },
            Commands::Ema { cmd } => outcome_to_string(super::system::handle_ema(cmd, json).await?),
            Commands::Who { cmd } => outcome_to_string(super::system::handle_who(cmd, json).await?),
            Commands::Cvx { cmd } => outcome_to_string(super::system::handle_cvx(cmd, json).await?),
            Commands::Ddinter { cmd } => {
                outcome_to_string(super::system::handle_ddinter(cmd, json).await?)
            }
            Commands::Gtr { cmd } => outcome_to_string(super::system::handle_gtr(cmd, json).await?),
            Commands::WhoIvd { cmd } => {
                outcome_to_string(super::system::handle_who_ivd(cmd, json).await?)
            }
            Commands::Skill { command } => match command {
                None => {
                    let content = crate::cli::skill::show_overview()?;
                    if json { Ok(crate::render::json::to_pretty(&serde_json::json!({"kind":"skill","action":"overview","content":content}))?) } else { Ok(content) }
                }
                Some(SkillCommand::List) => {
                    let content = crate::cli::skill::list_use_cases()?;
                    if json { Ok(crate::render::json::to_pretty(&serde_json::json!({"kind":"skill","action":"list","content":content}))?) } else { Ok(content) }
                }
                Some(SkillCommand::Render) => {
                    let content = crate::cli::skill::render_system_prompt()?;
                    if json { Ok(crate::render::json::to_pretty(&serde_json::json!({"kind":"skill","action":"render","content":content}))?) } else { Ok(content) }
                }
                Some(SkillCommand::Status { dir }) => {
                    Ok(crate::cli::skill::skill_status(dir.as_deref(), json)?)
                }
                Some(SkillCommand::Install { dir, force }) => {
                    let content = crate::cli::skill::install_skills(dir.as_deref(), force)?;
                    if json { Ok(crate::render::json::to_pretty(&serde_json::json!({"kind":"skill","action":"install","status":"installed","changed":true,"content":content}))?) } else { Ok(content) }
                }
                Some(SkillCommand::Show(args)) => {
                    let key = if args.is_empty() {
                        String::new()
                    } else if args.len() == 1 {
                        args[0].clone()
                    } else {
                        args.join("-")
                    };
                    let content = crate::cli::skill::show_use_case(&key)?;
                    if json { Ok(crate::render::json::to_pretty(&serde_json::json!({"kind":"skill","action":"show","skill":key,"content":content}))?) } else { Ok(content) }
                }
            },
            Commands::Chart { command } => {
                let content = crate::cli::chart::show(command.as_ref())?;
                if json {
                    Ok(crate::render::json::to_pretty(&serde_json::json!({
                        "kind":"chart",
                        "chart": command.map(|value| format!("{value:?}").to_ascii_lowercase()),
                        "content": content,
                    }))?)
                } else { Ok(content) }
            }
            Commands::Update(super::system::UpdateArgs { check }) => {
                let content = crate::cli::update::run(check).await?;
                if json {
                    Ok(crate::render::json::to_pretty(&serde_json::json!({
                        "kind":"update",
                        "status": if check { "checked" } else { "completed" },
                        "changed": !check && content.starts_with("Updated "),
                        "content": content,
                    }))?)
                } else { Ok(content) }
            }
            Commands::Uninstall => outcome_to_string(super::system::handle_uninstall(json).await?),
            Commands::Enrich(args) => {
                outcome_to_string(super::system::handle_enrich(args, json).await?)
            }
            Commands::Discover(super::system::DiscoverArgs { query }) => {
                crate::cli::discover::run(crate::cli::discover::DiscoverArgs { query }, json).await
            }
            Commands::List(super::system::ListArgs { entity }) => {
                if json {
                    crate::cli::list::render_json(entity.as_deref()).map_err(Into::into)
                } else {
                    crate::cli::list::render(entity.as_deref()).map_err(Into::into)
                }
            }
            Commands::McpConfig(args) => {
                let content = crate::cli::mcp_config::run(args)?;
                if json {
                    let config = serde_json::from_str::<serde_json::Value>(&content)
                        .unwrap_or(serde_json::Value::String(content));
                    crate::render::json::to_pretty(&serde_json::json!({"kind":"mcp_config","config":config})).map_err(Into::into)
                } else { Ok(content) }
            }
            Commands::Mcp | Commands::Serve | Commands::ServeHttp(_) | Commands::ServeSse => {
                anyhow::bail!("MCP/serve commands should not go through CLI run()")
            }
            Commands::Version(args) => {
                outcome_to_string(super::system::handle_version(args, json).await?)
            }
        }
    })
    .await
}

async fn run_outcome_inner(
    cli: Cli,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    let Cli {
        command,
        json,
        no_cache,
    } = cli;

    match command {
        Commands::Cache {
            cmd: super::cache::CacheCommand::Clear { yes },
        } => {
            if !yes && !std::io::stdin().is_terminal() {
                return Ok(CommandOutcome::stderr_with_exit(
                    "Error: biomcp cache clear requires a TTY or --yes for non-interactive use."
                        .to_string(),
                    1,
                ));
            }

            let config = crate::cache::resolve_cache_config()?;
            let cache_path = config.cache_root.join("http");

            let report = if yes || crate::cli::cache::prompt_clear_confirmation(&cache_path)? {
                crate::cache::execute_cache_clear(&cache_path)?
            } else {
                crate::cache::ClearReport {
                    bytes_freed: None,
                    entries_removed: 0,
                }
            };

            let text = if json {
                crate::render::json::to_pretty(&report)?
            } else {
                crate::cli::cache::render_clear_text(&report)
            };
            Ok(CommandOutcome::stdout(text))
        }
        Commands::Get {
            entity: GetEntity::Gene(args),
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::gene::handle_get(args, json, alias_suggestions_as_json).await
            })
            .await
        }
        Commands::Get {
            entity: GetEntity::Drug(args),
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::drug::handle_get(args, json, alias_suggestions_as_json).await
            })
            .await
        }
        Commands::Get {
            entity: GetEntity::Variant(args),
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::variant::handle_get(args, json, alias_suggestions_as_json).await
            })
            .await
        }
        Commands::Search {
            entity: SearchEntity::Gene(args),
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::gene::handle_search(args, json, alias_suggestions_as_json).await
            })
            .await
        }
        Commands::Search {
            entity: SearchEntity::Variant(args),
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::variant::handle_search(args, json, alias_suggestions_as_json).await
            })
            .await
        }
        Commands::Get {
            entity: GetEntity::Article(args),
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::article::handle_get(args, json, alias_suggestions_as_json).await
            })
            .await
        }
        Commands::Get {
            entity: GetEntity::Trial(args),
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::trial::handle_get(args, json).await
            })
            .await
        }
        Commands::Discover(super::system::DiscoverArgs { query }) => {
            crate::sources::with_no_cache(no_cache, async move {
                crate::cli::discover::run_outcome(
                    crate::cli::discover::DiscoverArgs { query },
                    json,
                )
                .await
            })
            .await
        }
        Commands::Gene {
            cmd:
                super::GeneCommand::Cspec(super::gene::CspecArgs {
                    command: Some(super::gene::CspecCommand::Document { capture_id }),
                    ..
                }),
        } => super::gene::cspec::document(capture_id, json),
        Commands::Gene {
            cmd: super::GeneCommand::Definition { symbol },
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::gene::handle_command(
                    super::GeneCommand::Definition { symbol },
                    json,
                    alias_suggestions_as_json,
                )
                .await
            })
            .await
        }
        Commands::Drug {
            cmd: super::DrugCommand::External(args),
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::drug::handle_command(
                    super::DrugCommand::External(args),
                    json,
                    alias_suggestions_as_json,
                )
                .await
            })
            .await
        }
        Commands::Gene {
            cmd: super::GeneCommand::External(args),
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::gene::handle_command(
                    super::GeneCommand::External(args),
                    json,
                    alias_suggestions_as_json,
                )
                .await
            })
            .await
        }
        Commands::Variant { cmd } => {
            crate::sources::with_no_cache(no_cache, async move {
                Box::pin(super::variant::handle_command(cmd, json)).await
            })
            .await
        }
        Commands::Study { cmd } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::study::handle_command(cmd, json).await
            })
            .await
        }
        command => Ok(CommandOutcome::stdout(
            run(Cli {
                command,
                json,
                no_cache,
            })
            .await?,
        )),
    }
}

async fn run_outcome_on_current_stack(cli: Cli) -> anyhow::Result<CommandOutcome> {
    let json = cli.json || command_requests_json(&cli.command);
    let trusted_terminal_chart = is_charted_mcp_study_command(&cli).unwrap_or(false);
    let contract = JsonResponseContract::for_command(&cli.command);
    match Box::pin(run_outcome_inner(cli, false)).await {
        Ok(mut outcome) => Ok(if json {
            outcome = finalize_structured_error(outcome, contract);
            require_json_document(outcome)
        } else {
            if outcome.bytes.is_none() && !trusted_terminal_chart {
                outcome.text = crate::render::human::sanitize_document(&outcome.text);
            }
            outcome
        }),
        Err(err) => {
            if json && let Some(bio_err) = err.downcast_ref::<crate::error::BioMcpError>() {
                return Ok(finalize_structured_error(
                    CommandOutcome::stdout_with_exit(
                        crate::render::json::to_error_json(bio_err)?,
                        bio_mcp_error_exit_code(bio_err),
                    ),
                    contract,
                ));
            }
            Err(err)
        }
    }
}

async fn run_outcome_with_worker_stack(
    cli: Cli,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    const EXECUTE_STACK_BYTES: usize = 8 * 1024 * 1024;
    tokio::task::spawn_blocking(move || {
        let handle = std::thread::Builder::new()
            .name("biomcp-cli-execute".into())
            .stack_size(EXECUTE_STACK_BYTES)
            .spawn(move || -> anyhow::Result<CommandOutcome> {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                if alias_suggestions_as_json {
                    runtime.block_on(run_outcome_inner(cli, true))
                } else {
                    runtime.block_on(run_outcome_on_current_stack(cli))
                }
            })?;

        handle
            .join()
            .map_err(|_| anyhow::anyhow!("in-process CLI worker panicked"))?
    })
    .await
    .map_err(|err| anyhow::anyhow!("failed to join in-process CLI worker: {err}"))?
}

/// Execute a parsed CLI command on the bounded worker stack used by every
/// in-process caller, including the native CLI and MCP transports.
pub async fn run_outcome(cli: Cli) -> anyhow::Result<CommandOutcome> {
    run_outcome_with_worker_stack(cli, false).await
}
/// Main CLI execution - called by the MCP `biomcp` tool.
///
/// # Errors
///
/// Returns an error when CLI args cannot be parsed or when command execution fails.
pub async fn execute(mut args: Vec<String>) -> anyhow::Result<String> {
    if args.is_empty() {
        args.push("biomcp".to_string());
    }
    let cli = crate::cli::try_parse_cli(args)?;
    let outcome = run_outcome_with_worker_stack(cli, false).await?;
    outcome_to_string(outcome)
}

pub async fn execute_mcp(mut args: Vec<String>) -> anyhow::Result<CliOutput> {
    if args.is_empty() {
        args.push("biomcp".to_string());
    }

    let mut cli = crate::cli::try_parse_cli(args)?;
    prepare_mcp_chart(&mut cli)?;
    let outcome = run_outcome_with_worker_stack(cli, true).await?;
    outcome_to_mcp_output(outcome)
}

#[cfg(test)]
mod mcp_binary_tests {
    use super::outcome_to_mcp_output;
    use crate::cli::CommandOutcome;

    #[test]
    fn non_utf8_binary_outcome_is_never_converted_to_mcp_text() {
        let error = outcome_to_mcp_output(CommandOutcome::stdout_bytes(vec![0xff, 0xfe]))
            .expect_err("MCP must reject binary output");
        assert!(error.to_string().contains("binary downloads are CLI-only"));
        assert!(!error.to_string().contains('\u{fffd}'));
    }
}
