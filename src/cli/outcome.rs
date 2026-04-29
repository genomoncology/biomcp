//! CLI outcome execution seam and MCP chart argument rewriting.

use std::io::IsTerminal;

use super::{Cli, CliOutput, CommandOutcome, Commands, GetEntity, SearchEntity, StudyCommand};

fn outcome_to_string(outcome: CommandOutcome) -> anyhow::Result<String> {
    if outcome.exit_code == 0 {
        Ok(outcome.text)
    } else {
        anyhow::bail!("{}", outcome.text)
    }
}

fn mcp_output_flag_error() -> crate::error::BioMcpError {
    crate::error::BioMcpError::InvalidArgument(
        "MCP chart responses do not support --output/-o. Omit file output and consume the inline SVG image content instead.".into(),
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::cli) enum McpChartPass {
    Text,
    Svg,
}

fn require_flag_value(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<String, crate::error::BioMcpError> {
    args.get(index + 1).cloned().ok_or_else(|| {
        crate::error::BioMcpError::InvalidArgument(format!("{flag} requires a value"))
    })
}

pub(in crate::cli) fn rewrite_mcp_chart_args(
    args: &[String],
    pass: McpChartPass,
) -> Result<Vec<String>, crate::error::BioMcpError> {
    let mut rewritten = Vec::with_capacity(args.len() + 1);
    rewritten.push(
        args.first()
            .cloned()
            .unwrap_or_else(|| "biomcp".to_string()),
    );

    let mut i = 1usize;
    let mut saw_inline_flag = false;
    while i < args.len() {
        let token = &args[i];
        match token.as_str() {
            "--chart" => {
                let value = require_flag_value(args, i, "--chart")?;
                if pass == McpChartPass::Svg {
                    rewritten.push(token.clone());
                    rewritten.push(value);
                }
                i += 2;
            }
            "--terminal" => {
                i += 1;
            }
            "--output" => {
                if pass == McpChartPass::Svg {
                    return Err(mcp_output_flag_error());
                }
                let _ = require_flag_value(args, i, "--output")?;
                i += 2;
            }
            "-o" => {
                if pass == McpChartPass::Svg {
                    return Err(mcp_output_flag_error());
                }
                let _ = require_flag_value(args, i, "-o")?;
                i += 2;
            }
            "--title" | "--theme" | "--palette" => {
                let value = require_flag_value(args, i, token)?;
                if pass == McpChartPass::Svg {
                    rewritten.push(token.clone());
                    rewritten.push(value);
                }
                i += 2;
            }
            "--width" | "--height" => {
                let value = require_flag_value(args, i, token)?;
                if pass == McpChartPass::Svg {
                    rewritten.push(token.clone());
                    rewritten.push(value);
                }
                i += 2;
            }
            "--cols" | "--rows" => {
                let _ = require_flag_value(args, i, token)?;
                if pass == McpChartPass::Svg {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        crate::render::chart::TERMINAL_SIZE_FLAGS_ERROR.into(),
                    ));
                }
                i += 2;
            }
            "--scale" => {
                let _ = require_flag_value(args, i, token)?;
                if pass == McpChartPass::Svg {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        crate::render::chart::PNG_SCALE_FLAGS_ERROR.into(),
                    ));
                }
                i += 2;
            }
            "--mcp-inline" => {
                if pass == McpChartPass::Svg {
                    rewritten.push(token.clone());
                }
                saw_inline_flag = true;
                i += 1;
            }
            _ => {
                if token.starts_with("--chart=") {
                    if pass == McpChartPass::Svg {
                        rewritten.push(token.clone());
                    }
                    i += 1;
                    continue;
                }
                if token.starts_with("--output=") || token.starts_with("-o=") {
                    if pass == McpChartPass::Svg {
                        return Err(mcp_output_flag_error());
                    }
                    i += 1;
                    continue;
                }
                if token.starts_with("-o") && token.len() > 2 {
                    if pass == McpChartPass::Svg {
                        return Err(mcp_output_flag_error());
                    }
                    i += 1;
                    continue;
                }
                if token.starts_with("--title=")
                    || token.starts_with("--theme=")
                    || token.starts_with("--palette=")
                {
                    if pass == McpChartPass::Svg {
                        rewritten.push(token.clone());
                    }
                    i += 1;
                    continue;
                }
                if token.starts_with("--width=") || token.starts_with("--height=") {
                    if pass == McpChartPass::Svg {
                        rewritten.push(token.clone());
                    }
                    i += 1;
                    continue;
                }
                if token.starts_with("--cols=") || token.starts_with("--rows=") {
                    if pass == McpChartPass::Svg {
                        return Err(crate::error::BioMcpError::InvalidArgument(
                            crate::render::chart::TERMINAL_SIZE_FLAGS_ERROR.into(),
                        ));
                    }
                    i += 1;
                    continue;
                }
                if token.starts_with("--scale=") {
                    if pass == McpChartPass::Svg {
                        return Err(crate::error::BioMcpError::InvalidArgument(
                            crate::render::chart::PNG_SCALE_FLAGS_ERROR.into(),
                        ));
                    }
                    i += 1;
                    continue;
                }
                rewritten.push(token.clone());
                i += 1;
            }
        }
    }

    if pass == McpChartPass::Svg && !saw_inline_flag {
        rewritten.push("--mcp-inline".to_string());
    }
    Ok(rewritten)
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
                entity: GetEntity::Gene(args),
            } => outcome_to_string(super::gene::handle_get(args, json, false).await?),
            Commands::Get {
                entity: GetEntity::Article(args),
            } => outcome_to_string(super::article::handle_get(args, json).await?),
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
                    outcome_to_string(super::gene::handle_search(args, json).await?)
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
                super::cache::CacheCommand::Path => Ok(crate::cli::cache::render_path()?),
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
            Commands::Ema { cmd } => outcome_to_string(super::system::handle_ema(cmd).await?),
            Commands::Who { cmd } => outcome_to_string(super::system::handle_who(cmd).await?),
            Commands::Cvx { cmd } => outcome_to_string(super::system::handle_cvx(cmd).await?),
            Commands::Ddinter { cmd } => {
                outcome_to_string(super::system::handle_ddinter(cmd).await?)
            }
            Commands::Gtr { cmd } => outcome_to_string(super::system::handle_gtr(cmd).await?),
            Commands::WhoIvd { cmd } => {
                outcome_to_string(super::system::handle_who_ivd(cmd).await?)
            }
            Commands::Skill { command } => match command {
                None => Ok(crate::cli::skill::show_overview()?),
                Some(crate::cli::skill::SkillCommand::List) => {
                    Ok(crate::cli::skill::list_use_cases()?)
                }
                Some(crate::cli::skill::SkillCommand::Render) => {
                    Ok(crate::cli::skill::render_system_prompt()?)
                }
                Some(crate::cli::skill::SkillCommand::Install { dir, force }) => {
                    Ok(crate::cli::skill::install_skills(dir.as_deref(), force)?)
                }
                Some(crate::cli::skill::SkillCommand::Show(args)) => {
                    let key = if args.is_empty() {
                        String::new()
                    } else if args.len() == 1 {
                        args[0].clone()
                    } else {
                        args.join("-")
                    };
                    Ok(crate::cli::skill::show_use_case(&key)?)
                }
            },
            Commands::Chart { command } => Ok(crate::cli::chart::show(command.as_ref())?),
            Commands::Update(super::system::UpdateArgs {
                check,
                allow_missing_checksum,
            }) => Ok(crate::cli::update::run(check, allow_missing_checksum).await?),
            Commands::Uninstall => outcome_to_string(super::system::handle_uninstall().await?),
            Commands::Enrich(args) => {
                outcome_to_string(super::system::handle_enrich(args, json).await?)
            }
            Commands::Suggest(super::system::SuggestArgs { question }) => {
                crate::cli::suggest::run(crate::cli::suggest::SuggestArgs { question }, json)
            }
            Commands::Discover(super::system::DiscoverArgs { query }) => {
                crate::cli::discover::run(crate::cli::discover::DiscoverArgs { query }, json).await
            }
            Commands::List(super::system::ListArgs { entity }) => {
                crate::cli::list::render(entity.as_deref()).map_err(Into::into)
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
            entity: SearchEntity::Variant(args),
        } => {
            crate::sources::with_no_cache(no_cache, async move {
                super::variant::handle_search(args, json, alias_suggestions_as_json).await
            })
            .await
        }
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

pub async fn run_outcome(cli: Cli) -> anyhow::Result<CommandOutcome> {
    run_outcome_inner(cli, false).await
}

async fn run_outcome_with_worker_stack(cli: Cli) -> anyhow::Result<CommandOutcome> {
    const EXECUTE_STACK_BYTES: usize = 8 * 1024 * 1024;

    tokio::task::spawn_blocking(move || {
        let handle = std::thread::Builder::new()
            .name("biomcp-cli-execute".into())
            .stack_size(EXECUTE_STACK_BYTES)
            .spawn(move || -> anyhow::Result<CommandOutcome> {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                runtime.block_on(run_outcome(cli))
            })?;

        handle
            .join()
            .map_err(|_| anyhow::anyhow!("in-process CLI worker panicked"))?
    })
    .await
    .map_err(|err| anyhow::anyhow!("failed to join in-process CLI worker: {err}"))?
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
    let outcome = run_outcome_with_worker_stack(cli).await?;
    outcome_to_string(outcome)
}

pub async fn execute_mcp(mut args: Vec<String>) -> anyhow::Result<CliOutput> {
    if args.is_empty() {
        args.push("biomcp".to_string());
    }

    let cli = crate::cli::try_parse_cli(args.clone())?;
    if !is_charted_mcp_study_command(&cli)? {
        let outcome = Box::pin(run_outcome_inner(cli, true)).await?;
        return Ok(CliOutput {
            text: outcome.text,
            svg: None,
        });
    }

    let text = Box::pin(execute(rewrite_mcp_chart_args(&args, McpChartPass::Text)?)).await?;
    let svg = Box::pin(execute(rewrite_mcp_chart_args(&args, McpChartPass::Svg)?)).await?;
    Ok(CliOutput {
        text,
        svg: Some(svg),
    })
}
