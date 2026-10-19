//! Enforce disease identity on providers whose query only ranks free text.

use crate::entities::disease::{ExactDiseaseTerms, resolve_exact_disease_terms};
use crate::error::BioMcpError;
use crate::sources::mydisease::MyDiseaseClient;

use super::super::{
    ArticleSearchFilters, ArticleSearchResult, ArticleSource, ArticleSourceAvailability,
    ArticleSourceStatus,
};

pub(in crate::entities::article) fn semantic_disease_unavailable_status(
    source: ArticleSource,
    error: BioMcpError,
) -> ArticleSourceStatus {
    let message = match error {
        BioMcpError::SourceUnavailable { reason, .. } => reason,
        error => error.to_string(),
    };
    ArticleSourceStatus {
        source,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Unavailable),
        message: Some(message),
    }
}

pub(in crate::entities::article) async fn resolve_semantic_disease_terms(
    filters: &ArticleSearchFilters,
    source: ArticleSource,
) -> Result<Option<ExactDiseaseTerms>, BioMcpError> {
    let Some(disease) = filters
        .disease
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return Ok(None);
    };
    let result = match MyDiseaseClient::new() {
        Ok(client) => resolve_exact_disease_terms(&client, disease).await,
        Err(error) => Err(error),
    };
    result
        .map(Some)
        .map_err(|_| BioMcpError::SourceUnavailable {
            source_name: source.display_name().into(),
            reason: "disease synonym lookup unavailable; disease filtering cannot be completed"
                .into(),
            suggestion: "Retry the article search.".into(),
        })
}

fn phrase_matches(text: &str, term: &str) -> bool {
    let term = crate::transform::article::normalize_article_search_text(term);
    !term.is_empty()
        && text.match_indices(&term).any(|(start, matched)| {
            text[..start]
                .chars()
                .next_back()
                .is_none_or(|ch| !ch.is_alphanumeric())
                && text[start + matched.len()..]
                    .chars()
                    .next()
                    .is_none_or(|ch| !ch.is_alphanumeric())
        })
}

pub(in crate::entities::article) fn filter_semantic_disease_rows(
    rows: &mut Vec<ArticleSearchResult>,
    terms: Option<&ExactDiseaseTerms>,
) {
    let Some(terms) = terms else { return };
    rows.retain(|row| {
        std::iter::once(terms.requested.as_str())
            .chain(terms.canonical_name.as_deref())
            .chain(terms.synonyms.iter().map(String::as_str))
            .any(|term| {
                phrase_matches(&row.normalized_title, term)
                    || phrase_matches(&row.normalized_abstract, term)
            })
    });
}
