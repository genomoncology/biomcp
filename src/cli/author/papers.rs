use crate::cli::CommandOutcome;

pub(in crate::cli) async fn handle_papers(
    id: String,
    limit: usize,
    offset: usize,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let limit = super::super::paged_fetch_limit(limit, 0, 100)?;
    let response = crate::entities::author::papers(&id, offset, limit).await?;
    let text = if json {
        crate::render::json::to_pretty(&response)?
    } else {
        crate::render::markdown::author_papers_markdown(&response)
    };
    Ok(CommandOutcome::stdout(text))
}
