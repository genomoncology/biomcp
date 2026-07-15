use crate::cli::CommandOutcome;
use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AuthorSource {
    #[value(name = "semanticscholar")]
    SemanticScholar,
}

#[derive(Args, Debug)]
pub struct AuthorSearchArgs {
    /// Author name to search
    #[arg(short = 'q', long = "query", required = true)]
    pub query: String,
    /// Author data source
    #[arg(long, value_enum, default_value = "semanticscholar")]
    source: AuthorSource,
    /// Maximum results, 1-100
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}

pub(in crate::cli) async fn handle_search(
    args: AuthorSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let _ = args.source;
    let response = crate::entities::author::search(&args.query, args.offset, args.limit).await?;
    let text = if json {
        crate::render::json::to_pretty(&response)?
    } else {
        crate::render::markdown::author_search_markdown(&response)
    };
    Ok(CommandOutcome::stdout(text))
}
