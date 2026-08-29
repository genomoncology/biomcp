use crate::cli::CommandOutcome;
use clap::Args;

#[derive(Args, Debug)]
pub struct AuthorGetArgs {
    /// Provider-qualified author ID (`semanticscholar:<id>`)
    pub id: String,
}

pub(crate) fn render_loaded_card(
    author: &crate::entities::author::AuthorDetail,
    json_output: bool,
) -> anyhow::Result<String> {
    if json_output {
        Ok(crate::render::json::to_pretty(author)?)
    } else {
        Ok(crate::render::markdown::author_detail_markdown(author))
    }
}

pub(in crate::cli) async fn handle_get(
    args: AuthorGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let response = crate::entities::author::detail(&args.id).await?;
    let text = render_loaded_card(&response, json)?;
    Ok(CommandOutcome::stdout(text))
}
