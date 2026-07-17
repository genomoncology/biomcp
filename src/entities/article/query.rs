//! Article query builders and query-side source helpers.

use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::pubmed::PubMedESearchParams;
use crate::sources::pubtator::{PubTatorAutocompleteResult, PubTatorClient};

use super::filters::{
    normalize_article_type, normalized_date_bounds, validate_required_search_filters,
    validate_search_filter_values,
};
use super::ranking::validate_article_ranking_options;
use super::{ArticleSearchFilters, ArticleSort, MAX_FEDERATED_FETCH_RESULTS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntityBiotype {
    Gene,
    Disease,
    Chemical,
    Variant,
}

pub(super) fn europepmc_escape(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }

    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(
            ch,
            '\\' | '\"'
                | '+'
                | '-'
                | '!'
                | '('
                | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '^'
                | '~'
                | '*'
                | '?'
                | ':'
                | '|'
        ) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }

    escaped
}

pub(super) fn europepmc_phrase(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    let escaped = europepmc_escape(value);
    if value.chars().any(|c| c.is_whitespace()) || value.contains('/') {
        format!("\"{escaped}\"")
    } else {
        escaped
    }
}

pub(super) fn europepmc_keyword(value: &str) -> String {
    europepmc_escape(value)
}

pub(super) fn build_search_query(filters: &ArticleSearchFilters) -> Result<String, BioMcpError> {
    validate_required_search_filters(filters)?;
    validate_search_filter_values(filters)?;
    validate_article_ranking_options(filters)?;
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;
    let mut terms: Vec<String> = Vec::new();

    if let Some(gene) = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if filters.gene_anchored {
            terms.push(format!("GENE_PROTEIN:{}", europepmc_phrase(gene)));
        } else {
            terms.push(europepmc_phrase(gene));
        }
    }
    if let Some(disease) = filters
        .disease
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(europepmc_phrase(disease));
    }
    if let Some(drug) = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(europepmc_phrase(drug));
    }
    if let Some(author) = filters
        .author
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(format!("AUTH:{}", europepmc_phrase(author)));
    }
    if let Some(keyword) = filters
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(europepmc_keyword(keyword));
    }

    if let Some(article_type) = filters
        .article_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let normalized = normalize_article_type(article_type)?;
        terms.push(format!("PUB_TYPE:\"{normalized}\""));
    }

    if let Some(journal) = filters
        .journal
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(format!("JOURNAL:{}", europepmc_phrase(journal)));
    }

    if filters.open_access {
        terms.push("OPEN_ACCESS:y".into());
    }

    if let Some(from) = normalized_date_from.as_deref() {
        let to = normalized_date_to.as_deref().unwrap_or("*");
        terms.push(format!("FIRST_PDATE:[{from} TO {to}]"));
    } else if let Some(to) = normalized_date_to.as_deref() {
        terms.push(format!("FIRST_PDATE:[* TO {to}]"));
    }

    if filters.no_preprints {
        terms.push("NOT SRC:PPR".into());
    }
    if filters.exclude_retracted {
        terms.push("NOT PUB_TYPE:\"retracted publication\"".into());
    }

    Ok(terms.join(" AND "))
}

const PUBMED_STOPWORDS: &[&str] = &[
    "what", "which", "how", "are", "is", "do", "does", "can", "could", "list", "be", "thought",
    "cause", "the", "a", "an", "in", "of", "for", "to", "with", "by", "on", "or", "and",
];

fn strip_pubmed_stopwords(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut retained = Vec::new();
    for token in trimmed.split(|ch: char| ch.is_ascii_whitespace() || ch == '/') {
        let token = token.trim().trim_end_matches(['?', '.']);
        if token.is_empty() {
            continue;
        }
        if PUBMED_STOPWORDS
            .iter()
            .any(|stopword| token.eq_ignore_ascii_case(stopword))
        {
            continue;
        }
        retained.push(token);
    }

    if retained.is_empty() {
        return trimmed.to_string();
    }

    retained.join(" ")
}

pub(super) fn build_pubmed_search_term(
    filters: &ArticleSearchFilters,
) -> Result<String, BioMcpError> {
    validate_required_search_filters(filters)?;
    validate_search_filter_values(filters)?;

    let mut clauses: Vec<String> = Vec::new();
    for value in [
        filters.gene.as_deref(),
        filters.disease.as_deref(),
        filters.drug.as_deref(),
    ] {
        if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
            clauses.push(strip_pubmed_stopwords(value));
        }
    }
    if let Some(value) = filters
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let clause = strip_pubmed_stopwords(value);
        if clause.starts_with("p.") && !clause.contains(char::is_whitespace) {
            clauses.push(format!("\"{clause}\""));
        } else {
            clauses.push(clause);
        }
    }

    if let Some(author) = filters
        .author
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let author = author.replace('"', " ");
        clauses.push(format!("\"{author}\"[author]"));
    }

    if let Some(journal) = filters
        .journal
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        clauses.push(format!("\"{journal}\"[journal]"));
    }

    if let Some(article_type) = filters
        .article_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let normalized = normalize_article_type(article_type)?;
        let clause = match normalized {
            "review" => "review[pt]",
            "research-article" => "journal article[pt]",
            "case-reports" => "case reports[pt]",
            "meta-analysis" => "meta-analysis[pt]",
            _ => {
                return Err(BioMcpError::InvalidArgument(
                    "--type must be one of: review, research, research-article, case-reports, meta-analysis".into(),
                ))
            }
        };
        clauses.push(clause.to_string());
    }

    let base = clauses.join(" AND ");
    if filters.exclude_retracted {
        if base.is_empty() {
            return Ok("NOT retracted publication[pt]".to_string());
        }
        return Ok(format!("{base} NOT retracted publication[pt]"));
    }

    Ok(base)
}

// dead-code reason: query::build_pubmed_esearch_params is exercised by native tests or binary dispatch
#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn build_pubmed_esearch_params(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<PubMedESearchParams, BioMcpError> {
    let term = build_pubmed_search_term(filters)?;
    let (date_from, date_to) = normalized_date_bounds(filters)?;

    if limit == 0 || limit > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_FEDERATED_FETCH_RESULTS}"
        )));
    }

    let fetch_count = limit.saturating_add(offset);
    if fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for federated article search"
        )));
    }

    if filters.open_access {
        return Err(BioMcpError::InvalidArgument(
            "PubMed ESearch does not support --open-access filtering".into(),
        ));
    }
    if filters.no_preprints {
        return Err(BioMcpError::InvalidArgument(
            "PubMed ESearch does not support --no-preprints filtering".into(),
        ));
    }

    Ok(PubMedESearchParams {
        term,
        retstart: offset,
        retmax: limit,
        date_from,
        date_to,
    })
}

fn matches_entity_biotype(value: Option<&str>, expected: EntityBiotype) -> bool {
    let Some(value) = value else {
        return false;
    };
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }
    match expected {
        EntityBiotype::Gene => normalized.contains("gene"),
        EntityBiotype::Disease => normalized.contains("disease"),
        EntityBiotype::Chemical => normalized.contains("chemical") || normalized.contains("drug"),
        EntityBiotype::Variant => normalized.contains("variant") || normalized.contains("mutation"),
    }
}

fn normalize_variant_match_token(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("p.")
        .trim_start_matches("P.")
        .chars()
        .filter_map(|ch| match ch {
            '*' => Some('X'),
            ch if ch.is_ascii_alphanumeric() => Some(ch.to_ascii_uppercase()),
            _ => None,
        })
        .collect()
}

fn variant_candidate_matches(
    row: &PubTatorAutocompleteResult,
    filters: &super::ArticleVariantIntent,
) -> bool {
    if !matches_entity_biotype(row.biotype.as_deref(), EntityBiotype::Variant) {
        return false;
    }

    let haystack = [row.id.as_deref(), row.name.as_deref()]
        .into_iter()
        .flatten()
        .map(normalize_variant_match_token)
        .collect::<Vec<_>>()
        .join(" ");

    if let Some(gene) = filters.gene.as_deref() {
        let gene = normalize_variant_match_token(gene);
        if !gene.is_empty() && !haystack.contains(&gene) {
            return false;
        }
    }
    if let Some(change) = filters.change.as_deref() {
        let change = crate::entities::variant::normalize_protein_change(change)
            .unwrap_or_else(|| change.to_string());
        let change = normalize_variant_match_token(&change);
        if !change.is_empty() && !haystack.contains(&change) {
            return false;
        }
    }

    true
}

pub(crate) async fn resolve_variant_entity_token(
    pubtator: &PubTatorClient,
    intent: &super::ArticleVariantIntent,
) -> Option<String> {
    if let Some(entity_id) = intent
        .entity_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(entity_id.to_string());
    }

    let query = intent.original.trim();
    if query.is_empty() {
        return None;
    }

    match pubtator.entity_autocomplete(query).await {
        Ok(rows) => rows
            .iter()
            .find(|row| variant_candidate_matches(row, intent))
            .and_then(|row| row.id.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string()),
        Err(err) => {
            warn!(?err, token = query, "pubtator variant autocomplete failed");
            None
        }
    }
}

async fn normalize_entity_token(
    pubtator: &PubTatorClient,
    token: Option<&str>,
    expected: EntityBiotype,
) -> Option<String> {
    let token = token.map(str::trim).filter(|value| !value.is_empty())?;
    match pubtator.entity_autocomplete(token).await {
        Ok(rows) => rows
            .iter()
            .find(|row| matches_entity_biotype(row.biotype.as_deref(), expected))
            .and_then(|row| row.id.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .or_else(|| Some(token.to_string())),
        Err(err) => {
            warn!(
                ?err,
                token, "pubtator autocomplete failed; falling back to raw token"
            );
            Some(token.to_string())
        }
    }
}

pub(super) fn pubtator_sort(sort: ArticleSort) -> Option<&'static str> {
    match sort {
        ArticleSort::Date => Some("date desc"),
        ArticleSort::Citations | ArticleSort::Relevance => None,
    }
}

pub(super) async fn build_pubtator_query(
    filters: &ArticleSearchFilters,
    pubtator: &PubTatorClient,
) -> Result<String, BioMcpError> {
    validate_required_search_filters(filters)?;
    let gene = normalize_entity_token(pubtator, filters.gene.as_deref(), EntityBiotype::Gene).await;
    let disease =
        normalize_entity_token(pubtator, filters.disease.as_deref(), EntityBiotype::Disease).await;
    let drug =
        normalize_entity_token(pubtator, filters.drug.as_deref(), EntityBiotype::Chemical).await;
    let variant = match filters.variant.as_ref() {
        Some(intent) => resolve_variant_entity_token(pubtator, intent).await,
        None => None,
    };

    let mut terms: Vec<String> = Vec::new();
    if let Some(gene) = gene {
        terms.push(gene);
    }
    if let Some(disease) = disease {
        terms.push(disease);
    }
    if let Some(drug) = drug {
        terms.push(drug);
    }
    if let Some(variant) = variant {
        terms.push(variant);
    }
    if let Some(author) = filters
        .author
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(author.to_string());
    }
    if let Some(keyword) = filters
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(keyword.to_string());
    }

    if terms.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "At least one queryable token is required for --source pubtator.".into(),
        ));
    }

    Ok(terms.join(" "))
}

pub(super) fn build_free_text_article_query(filters: &ArticleSearchFilters) -> String {
    [
        filters.gene.as_deref(),
        filters.disease.as_deref(),
        filters.drug.as_deref(),
        filters.keyword.as_deref(),
        filters.author.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

#[cfg(test)]
mod tests;
