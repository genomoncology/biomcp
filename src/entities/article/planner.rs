//! Article backend planning, source enablement, and debug summary helpers.

use crate::error::BioMcpError;

use super::filters::{has_article_type_filter, has_author_filter, has_keyword_query};
use super::{ArticleSearchFilters, ArticleSearchResult, ArticleSource, ArticleSourceFilter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackendPlan {
    EuropeOnly,
    PubTatorOnly,
    PubMedOnly,
    SemanticScholarOnly,
    LitSense2Only,
    TypeCapable,
    Both,
}

pub(super) fn pubmed_filter_compatible(filters: &ArticleSearchFilters) -> bool {
    !filters.open_access && !filters.no_preprints
}

fn has_strict_europepmc_filters(filters: &ArticleSearchFilters) -> bool {
    filters.open_access || has_article_type_filter(filters)
}

fn pubtator_strict_filter_error(filters: &ArticleSearchFilters) -> BioMcpError {
    if has_article_type_filter(filters) && pubmed_filter_compatible(filters) {
        return BioMcpError::InvalidArgument(
            "--source pubtator does not support --type. Use --source europepmc, --source pubmed, or remove --type.".into(),
        );
    }
    if filters.open_access {
        return BioMcpError::InvalidArgument(
            "--source pubtator does not support --open-access. Use --source europepmc or relax --open-access.".into(),
        );
    }
    BioMcpError::InvalidArgument(
        "--source pubtator does not support --type with --no-preprints. Use --source europepmc or relax --no-preprints.".into(),
    )
}

fn pubmed_source_filter_error(filters: &ArticleSearchFilters) -> BioMcpError {
    match (filters.open_access, filters.no_preprints) {
        (true, true) => BioMcpError::InvalidArgument(
            "--source pubmed does not support --open-access or --no-preprints. Use --source europepmc or relax the selected filters.".into(),
        ),
        (true, false) => BioMcpError::InvalidArgument(
            "--source pubmed does not support --open-access. Use --source europepmc or remove --open-access.".into(),
        ),
        (false, true) => BioMcpError::InvalidArgument(
            "--source pubmed does not support --no-preprints. Use --source europepmc or remove --no-preprints.".into(),
        ),
        (false, false) => unreachable!("pubmed_source_filter_error called with compatible filters"),
    }
}

fn semantic_scholar_source_filter_error(filters: &ArticleSearchFilters) -> BioMcpError {
    if has_article_type_filter(filters) {
        return BioMcpError::InvalidArgument(
            "--source semanticscholar does not support --type. Use --source europepmc, --source pubmed, or remove --type.".into(),
        );
    }
    if filters.open_access {
        return BioMcpError::InvalidArgument(
            "--source semanticscholar does not support --open-access. Use --source europepmc or remove --open-access.".into(),
        );
    }
    unreachable!("semantic_scholar_source_filter_error called with compatible filters");
}

fn author_source_filter_error(source: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "--source {source} does not support --author. Use --source europepmc, --source pubmed, or --source all."
    ))
}

fn litsense2_source_filter_error(filters: &ArticleSearchFilters) -> BioMcpError {
    if !has_keyword_query(filters) {
        return BioMcpError::InvalidArgument(
            "--source litsense2 requires a keyword query. Add -k/--keyword (or a positional query) or use --source all.".into(),
        );
    }
    if has_article_type_filter(filters) {
        return BioMcpError::InvalidArgument(
            "--source litsense2 does not support --type. Use --source europepmc, --source pubmed, or remove --type.".into(),
        );
    }
    if filters.open_access {
        return BioMcpError::InvalidArgument(
            "--source litsense2 does not support --open-access. Use --source europepmc or remove --open-access.".into(),
        );
    }
    unreachable!("litsense2_source_filter_error called with compatible filters");
}

pub(crate) fn plan_backends(
    filters: &ArticleSearchFilters,
    source: ArticleSourceFilter,
) -> Result<BackendPlan, BioMcpError> {
    match source {
        ArticleSourceFilter::EuropePmc => Ok(BackendPlan::EuropeOnly),
        ArticleSourceFilter::PubTator => {
            if has_author_filter(filters) {
                return Err(author_source_filter_error("pubtator"));
            }
            if has_strict_europepmc_filters(filters) {
                return Err(pubtator_strict_filter_error(filters));
            }
            Ok(BackendPlan::PubTatorOnly)
        }
        ArticleSourceFilter::PubMed => {
            if !pubmed_filter_compatible(filters) {
                return Err(pubmed_source_filter_error(filters));
            }
            Ok(BackendPlan::PubMedOnly)
        }
        ArticleSourceFilter::SemanticScholar => {
            if has_author_filter(filters) {
                return Err(author_source_filter_error("semanticscholar"));
            }
            if has_strict_europepmc_filters(filters) {
                return Err(semantic_scholar_source_filter_error(filters));
            }
            Ok(BackendPlan::SemanticScholarOnly)
        }
        ArticleSourceFilter::LitSense2 => {
            if has_author_filter(filters) {
                return Err(author_source_filter_error("litsense2"));
            }
            if !has_keyword_query(filters)
                || has_article_type_filter(filters)
                || filters.open_access
            {
                return Err(litsense2_source_filter_error(filters));
            }
            Ok(BackendPlan::LitSense2Only)
        }
        ArticleSourceFilter::All => {
            if filters.open_access {
                Ok(BackendPlan::EuropeOnly)
            } else if has_author_filter(filters) || has_article_type_filter(filters) {
                if pubmed_filter_compatible(filters) {
                    Ok(BackendPlan::TypeCapable)
                } else {
                    Ok(BackendPlan::EuropeOnly)
                }
            } else {
                Ok(BackendPlan::Both)
            }
        }
    }
}

pub(crate) fn semantic_scholar_search_enabled(
    filters: &ArticleSearchFilters,
    source: ArticleSourceFilter,
) -> bool {
    matches!(
        source,
        ArticleSourceFilter::All | ArticleSourceFilter::SemanticScholar
    ) && !has_strict_europepmc_filters(filters)
        && !has_author_filter(filters)
}

pub(crate) fn litsense2_search_enabled(
    _filters: &ArticleSearchFilters,
    source: ArticleSourceFilter,
) -> bool {
    source == ArticleSourceFilter::LitSense2
}

pub(crate) fn article_type_limitation_note(
    filters: &ArticleSearchFilters,
    source: ArticleSourceFilter,
) -> Option<String> {
    if source != ArticleSourceFilter::All || !has_article_type_filter(filters) {
        return None;
    }
    if pubmed_filter_compatible(filters) {
        Some(
            "Note: --type restricts article search to Europe PMC and PubMed. PubTator3, LitSense2, and Semantic Scholar do not support publication-type filtering."
                .into(),
        )
    } else {
        Some(
            "Note: --type restricts this article search to Europe PMC. PubTator3, LitSense2, and Semantic Scholar do not support publication-type filtering, and PubMed does not support the other selected filters."
                .into(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArticleSearchDebugSummary {
    pub routing: Vec<String>,
    pub sources: Vec<String>,
    pub matched_sources: Vec<String>,
}

pub(crate) fn summarize_debug_plan(
    filters: &ArticleSearchFilters,
    source: ArticleSourceFilter,
    results: &[ArticleSearchResult],
) -> Result<ArticleSearchDebugSummary, BioMcpError> {
    let plan = plan_backends(filters, source)?;
    let planner = match (plan, source) {
        (BackendPlan::EuropeOnly, ArticleSourceFilter::All)
            if filters.open_access
                || (has_article_type_filter(filters) && !pubmed_filter_compatible(filters)) =>
        {
            "planner=europe_only_strict_filters"
        }
        (BackendPlan::EuropeOnly, _) => "planner=europe_only",
        (BackendPlan::PubTatorOnly, _) => "planner=pubtator_only",
        (BackendPlan::PubMedOnly, _) => "planner=pubmed_only",
        (BackendPlan::SemanticScholarOnly, _) => "planner=semanticscholar_only",
        (BackendPlan::LitSense2Only, _) => "planner=litsense2_only",
        (BackendPlan::TypeCapable, _) => "planner=type_capable",
        (BackendPlan::Both, _) => "planner=federated",
    };

    let mut sources = match plan {
        BackendPlan::EuropeOnly => vec!["Europe PMC".to_string()],
        BackendPlan::PubTatorOnly => vec!["PubTator3".to_string()],
        BackendPlan::PubMedOnly => vec!["PubMed".to_string()],
        BackendPlan::SemanticScholarOnly => vec!["Semantic Scholar".to_string()],
        BackendPlan::LitSense2Only => vec!["LitSense2".to_string()],
        BackendPlan::TypeCapable => vec!["Europe PMC".to_string(), "PubMed".to_string()],
        BackendPlan::Both => {
            let mut sources = vec!["PubTator3".to_string(), "Europe PMC".to_string()];
            if pubmed_filter_compatible(filters) {
                sources.push("PubMed".to_string());
            }
            sources
        }
    };
    if semantic_scholar_search_enabled(filters, source)
        && !matches!(plan, BackendPlan::SemanticScholarOnly)
    {
        sources.push("Semantic Scholar".to_string());
    }

    let matched_sources = [
        ArticleSource::PubTator,
        ArticleSource::EuropePmc,
        ArticleSource::PubMed,
        ArticleSource::SemanticScholar,
        ArticleSource::LitSense2,
    ]
    .into_iter()
    .filter(|candidate| {
        results.iter().any(|row| {
            row.source == *candidate || row.matched_sources.iter().any(|source| source == candidate)
        })
    })
    .map(|source| source.display_name().to_string())
    .collect();

    Ok(ArticleSearchDebugSummary {
        routing: vec![planner.to_string()],
        sources,
        matched_sources,
    })
}

#[cfg(test)]
mod tests;
