use anyhow::Context;

use crate::cli::CommandOutcome;

#[derive(Debug, Clone)]
pub struct DiscoverArgs {
    pub query: String,
}

pub async fn run(args: DiscoverArgs, json: bool) -> anyhow::Result<String> {
    Ok(run_outcome(args, json).await?.text)
}

pub async fn run_outcome(args: DiscoverArgs, json: bool) -> anyhow::Result<CommandOutcome> {
    let result = crate::entities::discover::resolve_query(
        &args.query,
        crate::entities::discover::DiscoverMode::Command,
    )
    .await
    .context("discover requires OLS4")?;

    let structured = crate::render::json::to_discover_json(&result)?;
    let text = if json {
        structured.clone()
    } else {
        crate::render::markdown::render_discover(&result)?
    };
    Ok(CommandOutcome::stdout(text).with_metadata_json(structured))
}
