use crate::cli::CommandOutcome;
use clap::Args;

#[derive(Args, Debug)]
pub struct AuthorGetArgs {
    /// Provider-qualified author ID (`semanticscholar:<id>`)
    pub id: String,
}

pub(in crate::cli) async fn handle_get(
    args: AuthorGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let response = crate::entities::author::detail(&args.id).await?;
    let text = if json {
        crate::render::json::to_pretty(&response)?
    } else {
        crate::render::markdown::author_detail_markdown(&response)
    };
    Ok(CommandOutcome::stdout(text))
}
