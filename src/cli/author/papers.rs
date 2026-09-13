use crate::cli::CommandOutcome;

pub(in crate::cli) async fn handle_papers(
    id: String,
    limit: usize,
    offset: usize,
    full: bool,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let limit = super::super::paged_fetch_limit(limit, 0, 100)?;
    let text = if full {
        let response = crate::entities::author::papers_full(&id, offset, limit).await?;
        if json {
            crate::render::json::to_pretty(&response)?
        } else {
            crate::render::markdown::author_papers_full_markdown(&response)
        }
    } else {
        let response = crate::entities::author::papers(&id, offset, limit).await?;
        if json {
            crate::render::json::to_pretty(&response)?
        } else {
            crate::render::markdown::author_papers_markdown(&response)
        }
    };
    Ok(CommandOutcome::stdout(text))
}
