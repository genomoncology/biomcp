//! Variant-article CLI input and rendering.

use tokio::io::AsyncReadExt;

use crate::cli::CommandOutcome;
use crate::entities::article::{VariantArticleStrategy, VariantArticleStrategy::Union};
use crate::error::BioMcpError;

async fn read_input(path: &str) -> Result<Vec<u8>, BioMcpError> {
    const READ_LIMIT: u64 = 64 * 1024 + 1;
    let mut bytes = Vec::new();
    if path == "-" {
        tokio::io::stdin()
            .take(READ_LIMIT)
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| {
                BioMcpError::InvalidArgument(
                    "unable to read variant article input from stdin".into(),
                )
            })?;
    } else {
        tokio::fs::File::open(path)
            .await
            .map_err(|_| {
                BioMcpError::InvalidArgument("unable to read variant article input file".into())
            })?
            .take(READ_LIMIT)
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| {
                BioMcpError::InvalidArgument("unable to read variant article input file".into())
            })?;
    }
    Ok(bytes)
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle(
    id: Option<String>,
    input: Option<String>,
    debug_plan: bool,
    verify_identity: bool,
    confirmed_only: bool,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    if confirmed_only && !verify_identity {
        return Err(BioMcpError::InvalidArgument(
            "variant articles --confirmed-only requires --verify-identity".into(),
        )
        .into());
    }
    let verification = crate::entities::article::VariantArticleVerificationOptions {
        verify_identity,
        confirmed_only,
    };
    if id.is_some() && input.is_some() {
        return Err(BioMcpError::InvalidArgument(
            "variant articles positional ID cannot be combined with --input".into(),
        )
        .into());
    }
    if (input.is_some() || debug_plan) && !json {
        return Err(BioMcpError::InvalidArgument(
            "variant articles --input and --debug-plan require --json".into(),
        )
        .into());
    }
    if let Some(path) = input {
        let requests =
            crate::entities::article::parse_variant_article_batch(&read_input(&path).await?)?;
        let outcome = crate::entities::article::search_variant_article_batch_with_options(
            requests,
            strategy,
            limit,
            offset,
            debug_plan,
            verification,
        )
        .await?;
        let text = crate::render::json::to_pretty(&outcome.response)?;
        return Ok(if outcome.hard_error {
            CommandOutcome::stdout_with_exit(text, 1)
        } else {
            CommandOutcome::stdout(text)
        });
    }

    let id = id.ok_or_else(|| {
        BioMcpError::InvalidArgument("variant articles requires an ID or --input".into())
    })?;
    let outcome = if debug_plan {
        crate::entities::article::search_variant_articles_with_options(
            &id,
            strategy,
            limit,
            offset,
            true,
            verification,
        )
        .await?
    } else {
        crate::entities::article::search_variant_articles_with_options(
            &id,
            strategy,
            limit,
            offset,
            false,
            verification,
        )
        .await?
    };
    let text = if json {
        crate::render::json::to_pretty(&outcome.response)?
    } else {
        let filters = super::super::related_article_filters();
        let results = outcome
            .response
            .results
            .iter()
            .map(|row| row.article.clone())
            .collect::<Vec<_>>();
        let query = [
            Some(format!("variant={id}")),
            (strategy != Union).then(|| format!("strategy={strategy:?}").to_ascii_lowercase()),
            (offset > 0).then(|| format!("offset={offset}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ");
        crate::render::markdown::article_search_markdown_with_footer_and_context(
            &query,
            &results,
            "",
            &filters,
            crate::render::markdown::ArticleSearchRenderContext {
                source_filter: crate::entities::article::ArticleSourceFilter::All,
                semantic_scholar_enabled: false,
                warning: (!outcome.response.complete)
                    .then_some("One or more variant article routes were incomplete."),
                note: Some(outcome.response.retrieval_path),
                debug_plan: None,
                exact_entity_commands: &[],
                source_status: &[],
            },
        )?
    };
    if !outcome.hard_error {
        return Ok(CommandOutcome::stdout(text));
    }
    if json {
        return Ok(CommandOutcome::stdout_with_exit(text, 1));
    }
    let sources = outcome
        .response
        .source_status
        .iter()
        .filter(|status| {
            status.status
                == crate::entities::article::variant_search::VariantArticleSourceStatusKind::Unavailable
        })
        .map(|status| match status.source.as_str() {
            "pubtator" => "PubTator 3",
            "myvariant" => "MyVariant.info",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(", ");
    Ok(CommandOutcome::stderr_with_exit(
        format!("{sources} variant article route unavailable; retry the request."),
        1,
    ))
}
