//! Disease search, phenotype search, and search-only match helpers.

use super::*;

use super::associations::normalize_hpo_id;
use super::resolution::{rerank_disease_search_hits, resolver_queries};

pub(super) const MAX_DISEASE_SEARCH_LIMIT: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiseaseSearchRequest {
    pub(crate) query: String,
    pub(crate) source: Option<String>,
    pub(crate) inheritance: Option<String>,
    pub(crate) phenotype: Option<String>,
    pub(crate) onset: Option<String>,
    pub(crate) limit: usize,
    pub(crate) offset: usize,
    pub(crate) fetch_size: usize,
    pub(crate) resolver_queries: Vec<String>,
    pub(crate) prefer_doid: bool,
}

impl DiseaseSearchRequest {
    fn new(
        filters: &DiseaseSearchFilters,
        limit: usize,
        offset: usize,
    ) -> Result<Self, BioMcpError> {
        if limit == 0 || limit > MAX_DISEASE_SEARCH_LIMIT {
            return Err(BioMcpError::InvalidArgument(format!(
                "--limit must be between 1 and {MAX_DISEASE_SEARCH_LIMIT}"
            )));
        }

        let query = filters
            .query
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                BioMcpError::InvalidArgument(
                    "Query is required. Example: biomcp search disease -q melanoma".into(),
                )
            })?
            .to_string();
        let source = filters
            .source
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let inheritance = filters
            .inheritance
            .as_deref()
            .map(normalize_inheritance)
            .transpose()?;
        let phenotype = filters
            .phenotype
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let onset = filters.onset.as_deref().map(normalize_onset).transpose()?;
        let needed = limit.saturating_add(offset).max(limit);
        let fetch_size = if needed >= 50 {
            needed
        } else {
            (needed.saturating_mul(5)).clamp(needed, 50)
        };
        let resolver_queries = resolver_queries(&query);
        let prefer_doid = source
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("doid"));

        Ok(Self {
            query,
            source,
            inheritance,
            phenotype,
            onset,
            limit,
            offset,
            fetch_size,
            resolver_queries,
            prefer_doid,
        })
    }
}

fn normalize_inheritance(value: &str) -> Result<String, BioMcpError> {
    const NAMES: &[&str] = &[
        "autosomal dominant",
        "autosomal recessive",
        "x-linked",
        "x-linked dominant",
        "x-linked recessive",
        "y-linked",
        "mitochondrial",
        "multifactorial",
        "oligogenic",
        "polygenic",
        "sporadic",
        "somatic mosaicism",
        "dominant",
        "recessive",
    ];
    const HPO_IDS: &[&str] = &[
        "HP:0000006",
        "HP:0000007",
        "HP:0001417",
        "HP:0001423",
        "HP:0001419",
        "HP:0001450",
        "HP:0001427",
        "HP:0001426",
        "HP:0010983",
        "HP:0010982",
        "HP:0003745",
        "HP:0001442",
    ];

    let value = value.trim();
    if let Some(name) = NAMES.iter().find(|name| value.eq_ignore_ascii_case(name)) {
        return Ok((*name).to_string());
    }
    if let Some(hpo_id) = HPO_IDS
        .iter()
        .find(|hpo_id| value.eq_ignore_ascii_case(hpo_id))
    {
        return Ok((*hpo_id).to_string());
    }
    Err(BioMcpError::InvalidArgument(format!(
        "--inheritance must be one of: {}; or HPO inheritance ID: {}",
        NAMES.join(", "),
        HPO_IDS.join(", ")
    )))
}

fn normalize_onset(value: &str) -> Result<String, BioMcpError> {
    const VALUES: &[&str] = &[
        "antenatal",
        "embryonal",
        "fetal",
        "congenital",
        "neonatal",
        "infantile",
        "childhood",
        "juvenile",
        "adolescent",
        "young adult",
        "adult",
        "middle age",
        "late onset",
    ];

    let value = value.trim();
    if value.eq_ignore_ascii_case("infancy") {
        return Ok("infantile".to_string());
    }
    if let Some(onset) = VALUES
        .iter()
        .find(|onset| value.eq_ignore_ascii_case(onset))
    {
        return Ok((*onset).to_string());
    }
    Err(BioMcpError::InvalidArgument(format!(
        "--onset must be one of: {}, infancy",
        VALUES.join(", ")
    )))
}

fn inheritance_matches(hit: &crate::sources::mydisease::MyDiseaseHit, expected: &str) -> bool {
    let needle = expected.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return true;
    }
    hit.hpo
        .as_ref()
        .map(|hpo| {
            hpo.inheritance.iter().any(|row| {
                row.hpo_name
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|v| v.to_ascii_lowercase().contains(&needle))
                    || row
                        .hpo_id
                        .as_deref()
                        .map(str::trim)
                        .is_some_and(|v| v.to_ascii_lowercase().contains(&needle))
            })
        })
        .unwrap_or(false)
}

fn phenotype_matches(hit: &crate::sources::mydisease::MyDiseaseHit, expected: &str) -> bool {
    let needle = expected.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return true;
    }
    hit.hpo
        .as_ref()
        .map(|hpo| {
            hpo.phenotype_related_to_disease.iter().any(|row| {
                row.hpo_id
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|v| v.to_ascii_lowercase().contains(&needle))
            })
        })
        .unwrap_or(false)
}

fn onset_matches(hit: &crate::sources::mydisease::MyDiseaseHit, expected: &str) -> bool {
    let needle = expected.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return true;
    }
    hit.hpo
        .as_ref()
        .map(|hpo| {
            hpo.clinical_course.iter().any(|row| {
                row.hpo_name
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|v| v.to_ascii_lowercase().contains(&needle))
            })
        })
        .unwrap_or(false)
}

pub async fn search_page(
    filters: &DiseaseSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<DiseaseSearchResult>, BioMcpError> {
    let request = DiseaseSearchRequest::new(filters, limit, offset)?;

    let client = MyDiseaseClient::new()?;
    let mut merged_total = 0usize;
    let mut query_hits = Vec::new();
    for (query_idx, resolved_query) in request.resolver_queries.iter().enumerate() {
        let resp = client
            .query(
                resolved_query,
                request.fetch_size,
                0,
                request.source.as_deref(),
                request.inheritance.as_deref(),
                request.phenotype.as_deref(),
                request.onset.as_deref(),
            )
            .await?;
        merged_total = merged_total.max(resp.total);
        let hits = resp
            .hits
            .into_iter()
            .filter(|hit| {
                request
                    .inheritance
                    .as_deref()
                    .is_none_or(|value| inheritance_matches(hit, value))
                    && request
                        .phenotype
                        .as_deref()
                        .is_none_or(|value| phenotype_matches(hit, value))
                    && request
                        .onset
                        .as_deref()
                        .is_none_or(|value| onset_matches(hit, value))
            })
            .collect::<Vec<_>>();
        query_hits.push((query_idx, hits));
    }

    let ranked_hits = rerank_disease_search_hits(&request.query, query_hits);
    let total = Some(merged_total.max(ranked_hits.len()));
    let results = ranked_hits
        .into_iter()
        .skip(request.offset)
        .take(request.limit)
        .map(|hit| {
            let mut row = transform::disease::from_mydisease_search_hit(&hit);
            if request.prefer_doid
                && let Some(doid) = transform::disease::doid_from_mydisease_hit(&hit)
            {
                row.id = doid;
            }
            row
        })
        .collect::<Vec<_>>();

    Ok(SearchPage::offset(results, total))
}

pub fn search_query_summary(filters: &DiseaseSearchFilters) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = filters
        .query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(v.to_string());
    }
    if let Some(v) = filters
        .source
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("source={v}"));
    }
    if let Some(v) = filters
        .inheritance
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("inheritance={v}"));
    }
    if let Some(v) = filters
        .phenotype
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("phenotype={v}"));
    }
    if let Some(v) = filters
        .onset
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("onset={v}"));
    }
    parts.join(", ")
}

const PHENOTYPE_QUERY_EXAMPLES: &str = "Examples: biomcp search phenotype \"HP:0001250 HP:0001263\" or biomcp search phenotype \"seizure, developmental delay\"";

fn phenotype_query_required_error() -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Phenotype terms are required. Use HPO IDs or symptom phrases. {PHENOTYPE_QUERY_EXAMPLES}"
    ))
}

fn phenotype_query_no_match_error(raw: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "No HPO terms matched query: {raw}. Try HPO IDs like HP:0001250 or refine the symptom phrases."
    ))
}

fn parse_hpo_query_terms(raw: &str) -> Result<Vec<String>, BioMcpError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(phenotype_query_required_error());
    }

    let mut terms = Vec::new();
    let mut seen = HashSet::new();
    for token in raw
        .split(|c: char| c.is_whitespace() || c == ',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let Some(id) = normalize_hpo_id(token) else {
            return Err(BioMcpError::InvalidArgument(format!(
                "Invalid HPO term: {token}. Expected format HP:0001250"
            )));
        };
        if seen.insert(id.clone()) {
            terms.push(id);
            if terms.len() > MAX_PHENOTYPE_TERMS {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Phenotype search accepts at most {MAX_PHENOTYPE_TERMS} unique HPO terms"
                )));
            }
        }
    }

    if terms.is_empty() {
        return Err(phenotype_query_required_error());
    }

    Ok(terms)
}

fn split_phenotype_queries(raw: &str) -> Vec<String> {
    let mut queries = raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if queries.is_empty() {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            queries.push(trimmed.to_string());
        }
    }
    queries
}

async fn resolve_phenotype_query_terms(raw: &str) -> Result<Vec<String>, BioMcpError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(phenotype_query_required_error());
    }

    match parse_hpo_query_terms(raw) {
        Ok(terms) => return Ok(terms),
        Err(error)
            if raw
                .split(|character: char| character.is_whitespace() || character == ',')
                .filter(|token| !token.is_empty())
                .all(|token| {
                    let token = token.to_ascii_uppercase();
                    token.starts_with("HP:") || token.starts_with("HP_")
                }) =>
        {
            return Err(error);
        }
        Err(_) => {}
    }

    let queries = split_phenotype_queries(raw);
    if queries.is_empty() {
        return Err(phenotype_query_required_error());
    }

    let hpo = HpoClient::new()?;
    let mut resolved = Vec::new();
    let mut seen = HashSet::new();
    for query in queries {
        let ids = hpo.search_term_ids(&query, MAX_PHENOTYPE_TERMS).await?;
        for id in ids {
            if seen.insert(id.clone()) {
                resolved.push(id);
                if resolved.len() >= MAX_PHENOTYPE_TERMS {
                    return Ok(resolved);
                }
            }
        }
    }

    if resolved.is_empty() {
        return Err(phenotype_query_no_match_error(raw));
    }

    Ok(resolved)
}

const MAX_PHENOTYPE_TERMS: usize = 10;
const MAX_PHENOTYPE_WINDOW: usize = crate::sources::monarch::MONARCH_PHENOTYPE_WINDOW_LIMIT;

pub(crate) fn validate_phenotype_search_window(
    limit: usize,
    offset: usize,
) -> Result<usize, BioMcpError> {
    if limit == 0 || limit > MAX_PHENOTYPE_WINDOW {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_PHENOTYPE_WINDOW}"
        )));
    }
    let end = offset.checked_add(limit).ok_or_else(|| {
        BioMcpError::InvalidArgument("--offset + --limit must be <= 50 for phenotype search".into())
    })?;
    if end > MAX_PHENOTYPE_WINDOW {
        return Err(BioMcpError::InvalidArgument(
            "--offset + --limit must be <= 50 for phenotype search".into(),
        ));
    }
    Ok(end)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PhenotypePagination {
    pub offset: usize,
    pub limit: usize,
    pub returned: usize,
    pub total: Option<usize>,
    pub has_more: bool,
    pub next_page_token: Option<String>,
    pub provider_window_limit: usize,
    pub provider_raw_row_count: usize,
    pub provider_window_exhausted: bool,
}

impl PhenotypePagination {
    pub(crate) fn next_window(&self) -> Option<(usize, usize)> {
        if !self.has_more {
            return None;
        }
        let offset = self.offset + self.returned;
        let limit = self.limit.min(MAX_PHENOTYPE_WINDOW.saturating_sub(offset));
        (limit > 0).then_some((limit, offset))
    }
}

#[derive(Debug, Clone)]
pub struct PhenotypeSearchPage {
    pub results: Vec<PhenotypeSearchResult>,
    pub pagination: PhenotypePagination,
}

fn paginate_phenotype_matches(
    provider: crate::sources::monarch::MonarchPhenotypeSearchResponse,
    limit: usize,
    offset: usize,
) -> Result<PhenotypeSearchPage, BioMcpError> {
    let window_end = validate_phenotype_search_window(limit, offset)?;
    let has_more = provider.matches.len() > window_end;
    let results = provider
        .matches
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(
            |MonarchPhenotypeMatch {
                 disease_id,
                 disease_name,
                 score,
             }| PhenotypeSearchResult {
                disease_id,
                disease_name,
                score,
            },
        )
        .collect::<Vec<_>>();
    let returned = results.len();

    Ok(PhenotypeSearchPage {
        results,
        pagination: PhenotypePagination {
            offset,
            limit,
            returned,
            total: None,
            has_more,
            next_page_token: None,
            provider_window_limit: crate::sources::monarch::MONARCH_PHENOTYPE_WINDOW_LIMIT,
            provider_raw_row_count: provider.raw_row_count,
            provider_window_exhausted: provider.provider_window_exhausted,
        },
    })
}

pub async fn search_phenotype_page(
    hpo_terms: &str,
    limit: usize,
    offset: usize,
) -> Result<PhenotypeSearchPage, BioMcpError> {
    validate_phenotype_search_window(limit, offset)?;

    let terms = resolve_phenotype_query_terms(hpo_terms).await?;
    let client = MonarchClient::new()?;
    let provider = client.phenotype_similarity_search(&terms).await?;
    paginate_phenotype_matches(provider, limit, offset)
}

#[cfg(test)]
mod tests;
