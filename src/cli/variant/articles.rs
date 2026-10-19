//! Variant-article CLI input and rendering.

use tokio::io::AsyncReadExt;

use crate::cli::{CommandOutcome, VariantArticlesMcpDisposition};
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
            CommandOutcome::stdout_with_exit(text, 1).with_variant_articles_mcp_disposition(
                VariantArticlesMcpDisposition::StructuredError,
            )
        } else {
            CommandOutcome::stdout(text)
                .with_variant_articles_mcp_disposition(VariantArticlesMcpDisposition::Success)
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
    let settled_empty_page = outcome.response.complete
        && outcome.response.results.is_empty()
        && outcome
            .response
            .pagination
            .total
            .is_some_and(|total| total > 0 && offset >= total);
    let mut text = if json {
        crate::render::json::to_pretty(&outcome.response)?
    } else if settled_empty_page {
        format!(
            "# Variant articles\n\nNo articles on this page. Known total: {}. Requested offset: {}.",
            outcome.response.pagination.total.unwrap_or_default(),
            offset,
        )
    } else {
        let filters = super::super::related_article_filters();
        let results = outcome
            .response
            .results
            .iter()
            .filter(|row| row.match_reason != "unverified_alias_match")
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
        let render = |results: &[crate::entities::article::ArticleSearchResult],
                      header: Option<&str>| {
            crate::render::markdown::variant_article_search_markdown(
                &query,
                results,
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
                    retry_page: None,

                    header,
                },
                &outcome.response.source_status,
            )
        };
        let mut markdown = if results.is_empty() && !outcome.response.results.is_empty() {
            format!("# Variant articles: {id}")
        } else {
            render(&results, None)?
        };
        let aliases = outcome
            .response
            .results
            .iter()
            .filter(|row| row.match_reason == "unverified_alias_match")
            .collect::<Vec<_>>();
        if !aliases.is_empty() {
            markdown.push_str("\n\n## Unverified alias match\n\nThese matches may describe a different variant and are not verified evidence.\n\n");
            for row in &aliases {
                for alias in &row.alias_matches {
                    let transcript = alias.source_transcript.as_deref().unwrap_or(alias.reason);
                    markdown.push_str(&format!(
                        "- {}: {} ({transcript})\n",
                        row.article.pmid, alias.alias
                    ));
                }
            }
            let rows = aliases
                .iter()
                .map(|row| row.article.clone())
                .collect::<Vec<_>>();
            markdown.push_str(&render(&rows, Some("Alias-only articles"))?);
        }
        markdown
    };
    let mut alias_coverage_text = String::new();
    if !json && !outcome.response.alias_coverage.is_empty() {
        alias_coverage_text.push_str(
            "\n\n## Alias query coverage\n\nAn attempted query may fail or return no articles.\n\n",
        );
        for coverage in &outcome.response.alias_coverage {
            alias_coverage_text.push_str(&format!(
                "- {}: {} ({}; planned: {})\n",
                coverage.alias, coverage.disposition, coverage.reason, coverage.planned
            ));
        }
    }
    text.push_str(&alias_coverage_text);
    if !json && settled_empty_page {
        text.push_str("\n\n");
        text.push_str(&crate::render::markdown::variant_article_sources_markdown(
            &outcome.response.source_status,
        )?);
    }
    if !outcome.hard_error {
        return Ok(CommandOutcome::stdout(text)
            .with_variant_articles_mcp_disposition(VariantArticlesMcpDisposition::Success));
    }
    if json {
        return Ok(CommandOutcome::stdout_with_exit(text, 1)
            .with_variant_articles_mcp_disposition(
                VariantArticlesMcpDisposition::StructuredError,
            ));
    }
    let sources = outcome
        .response
        .source_status
        .iter()
        .filter(|status| {
            status.status
                != crate::entities::article::variant_search::VariantArticleSourceStatusKind::Ok
                && status.status
                    != crate::entities::article::variant_search::VariantArticleSourceStatusKind::Skipped
        })
        .map(|status| {
            let source = match status.source.as_str() {
                "pubtator" => "PubTator 3",
                "myvariant" => "MyVariant.info",
                other => other,
            };
            format!("{}/{}", status.route, source)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let cause = if outcome
        .response
        .source_status
        .iter()
        .any(|status| status.reason_codes.contains(&"invocation_deadline"))
    {
        "The invocation deadline expired"
    } else {
        "Provider work failed"
    };
    Ok(CommandOutcome::stderr_with_exit(
        format!(
            "{cause}; unfinished variant article routes: {sources}. Retry the request.{alias_coverage_text}\n\n{}",
            crate::render::markdown::variant_article_sources_markdown(
                &outcome.response.source_status,
            )?
        ),
        1,
    )
    .with_variant_articles_mcp_disposition(VariantArticlesMcpDisposition::StructuredError))
}
