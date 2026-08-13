use anyhow::Context;

use crate::cli::CommandOutcome;

#[derive(Debug, Clone)]
pub struct DiscoverArgs {
    pub query: String,
    pub limit: usize,
    pub offset: usize,
    pub full: bool,
}

pub async fn run(args: DiscoverArgs, json: bool) -> anyhow::Result<String> {
    Ok(run_outcome(args, json).await?.text)
}

pub async fn run_outcome(args: DiscoverArgs, json: bool) -> anyhow::Result<CommandOutcome> {
    let mut result = crate::entities::discover::resolve_query_with_options(
        &args.query,
        crate::entities::discover::DiscoverMode::Command,
        crate::entities::discover::DiscoverOptions {
            limit: args.limit,
            offset: args.offset,
            full: args.full,
        },
    )
    .await
    .context("discover requires OLS4")?;

    let budget = if args.full { 256 * 1024 } else { 32 * 1024 };
    let structured = loop {
        let candidate = crate::render::json::to_discover_json(&result)?;
        if candidate.len() <= budget || result.concepts.len() <= 1 {
            break candidate;
        }
        result.concepts.pop();
        result.preview_meta.pop();
        result.returned = result.concepts.len();
        result.has_more = true;
        result.next_offset = Some(result.offset.saturating_add(result.returned));
        result.budget_truncated = true;
        result.continuation_command = result.next_offset.map(|next| {
            crate::entities::discover::discover_continuation_command(
                &result.query,
                result.limit,
                next,
                result.full,
            )
        });
        crate::entities::discover::refresh_selected_guidance(&mut result);
    };
    let text = if json {
        structured.clone()
    } else {
        crate::render::markdown::render_discover(&result)?
    };
    Ok(CommandOutcome::stdout(text).with_metadata_json(structured))
}
