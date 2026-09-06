//! Disease search, phenotype search, and search-only match helpers.

use super::*;

use super::associations::normalize_hpo_id;
use super::resolution::{rerank_disease_search_hits, resolver_queries};
use futures::stream::{self, StreamExt};
use tokio::time::{Duration, Instant, timeout_at};

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

const PHENOTYPE_RESOLUTION_TIMEOUT: Duration = Duration::from_secs(8);
const PHENOTYPE_SUPPORT_TIMEOUT: Duration = Duration::from_secs(8);
const PHENOTYPE_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_PHENOTYPE_IN_FLIGHT: usize = 4;
#[cfg(test)]
const MAX_PHENOTYPE_LOGICAL_OPERATIONS: usize = 12;
#[cfg(test)]
const MAX_PHENOTYPE_PHYSICAL_ATTEMPTS: usize = 48;

trait HpoResolutionSource: Sync {
    fn term<'a>(
        &'a self,
        id: &'a str,
    ) -> impl std::future::Future<Output = Result<HpoTerm, BioMcpError>> + Send + 'a;
    fn search_terms<'a>(
        &'a self,
        query: &'a str,
    ) -> impl std::future::Future<Output = Result<Vec<HpoResolvedTerm>, BioMcpError>> + Send + 'a;
}

impl HpoResolutionSource for HpoClient {
    fn term<'a>(
        &'a self,
        id: &'a str,
    ) -> impl std::future::Future<Output = Result<HpoTerm, BioMcpError>> + Send + 'a {
        self.phenotype_term(id)
    }

    fn search_terms<'a>(
        &'a self,
        query: &'a str,
    ) -> impl std::future::Future<Output = Result<Vec<HpoResolvedTerm>, BioMcpError>> + Send + 'a
    {
        self.search_terms(query)
    }
}

trait PhenotypeMonarchSource: Sync {
    fn similarity<'a>(
        &'a self,
        terms: &'a [String],
    ) -> impl std::future::Future<
        Output = Result<crate::sources::monarch::MonarchPhenotypeSearchResponse, BioMcpError>,
    > + Send
    + 'a;
    fn direct_support<'a>(
        &'a self,
        disease_ids: &'a [String],
        terms: &'a [String],
    ) -> impl std::future::Future<Output = Result<MonarchDirectSupportLookup, BioMcpError>> + Send + 'a;
}

impl PhenotypeMonarchSource for MonarchClient {
    fn similarity<'a>(
        &'a self,
        terms: &'a [String],
    ) -> impl std::future::Future<
        Output = Result<crate::sources::monarch::MonarchPhenotypeSearchResponse, BioMcpError>,
    > + Send
    + 'a {
        self.phenotype_similarity_search(terms)
    }

    fn direct_support<'a>(
        &'a self,
        disease_ids: &'a [String],
        terms: &'a [String],
    ) -> impl std::future::Future<Output = Result<MonarchDirectSupportLookup, BioMcpError>> + Send + 'a
    {
        self.phenotype_direct_support(disease_ids, terms)
    }
}

fn hpo_deadline_error() -> BioMcpError {
    BioMcpError::Api {
        api: "hpo".into(),
        message: "HPO phenotype resolution exceeded its 8-second deadline".into(),
    }
}

fn monarch_deadline_error() -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: "Monarch Initiative".into(),
        reason: "Phenotype similarity retrieval exceeded the 30-second provider deadline".into(),
        suggestion: "Retry later when Monarch is healthy.".into(),
    }
}

fn validate_hpo_label(
    raw: &str,
    requested_id: &str,
    term: HpoTerm,
) -> Result<ResolvedPhenotypeQuery, BioMcpError> {
    let returned_id = normalize_hpo_id(&term.id).ok_or_else(|| BioMcpError::Api {
        api: "hpo".into(),
        message: format!("HPO returned an invalid term identifier for {requested_id}"),
    })?;
    if returned_id != requested_id {
        return Err(BioMcpError::Api {
            api: "hpo".into(),
            message: format!("HPO returned {returned_id} while resolving {requested_id}"),
        });
    }
    let label = term.name.trim();
    if label.is_empty() {
        return Err(BioMcpError::Api {
            api: "hpo".into(),
            message: format!("HPO returned a blank label for {requested_id}"),
        });
    }
    Ok(ResolvedPhenotypeQuery {
        raw: raw.into(),
        id: requested_id.into(),
        label: label.into(),
    })
}

#[cfg(test)]
async fn resolve_phenotype_query_terms(
    raw: &str,
    command_deadline: Instant,
) -> Result<Vec<ResolvedPhenotypeQuery>, BioMcpError> {
    let hpo = HpoClient::new()?;
    resolve_phenotype_query_terms_with_source(raw, command_deadline, &hpo).await
}

async fn resolve_phenotype_query_terms_with_source<S: HpoResolutionSource>(
    raw: &str,
    command_deadline: Instant,
    hpo: &S,
) -> Result<Vec<ResolvedPhenotypeQuery>, BioMcpError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(phenotype_query_required_error());
    }

    match parse_hpo_query_terms(raw) {
        Ok(terms) => {
            let first_raw = raw
                .split(|character: char| character.is_whitespace() || character == ',')
                .map(str::trim)
                .filter(|token| !token.is_empty())
                .fold(HashMap::<String, String>::new(), |mut map, token| {
                    if let Some(id) = normalize_hpo_id(token) {
                        map.entry(id).or_insert_with(|| token.to_string());
                    }
                    map
                });
            let deadline = (Instant::now() + PHENOTYPE_RESOLUTION_TIMEOUT).min(command_deadline);
            let mut outcomes = stream::iter(terms.into_iter().enumerate().map(|(index, id)| {
                let hpo = &hpo;
                let raw = first_raw.get(&id).cloned().unwrap_or_else(|| id.clone());
                async move {
                    let result = timeout_at(deadline, hpo.term(&id))
                        .await
                        .map_err(|_| hpo_deadline_error())
                        .and_then(|term| term)
                        .and_then(|term| validate_hpo_label(&raw, &id, term));
                    (index, result)
                }
            }))
            .buffer_unordered(MAX_PHENOTYPE_IN_FLIGHT)
            .collect::<Vec<_>>()
            .await;
            outcomes.sort_by_key(|(index, _)| *index);
            return outcomes.into_iter().map(|(_, result)| result).collect();
        }
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

    if queries.len() > MAX_PHENOTYPE_TERMS {
        return Err(BioMcpError::InvalidArgument(format!(
            "Phenotype search accepts at most {MAX_PHENOTYPE_TERMS} comma-delimited symptom phrases"
        )));
    }
    let deadline = (Instant::now() + PHENOTYPE_RESOLUTION_TIMEOUT).min(command_deadline);
    let mut outcomes = stream::iter(queries.iter().cloned().enumerate().map(|(index, query)| {
        let hpo = &hpo;
        async move {
            let result = timeout_at(deadline, hpo.search_terms(&query))
                .await
                .map_err(|_| hpo_deadline_error())
                .and_then(|rows| rows);
            (index, query, result)
        }
    }))
    .buffer_unordered(MAX_PHENOTYPE_IN_FLIGHT)
    .collect::<Vec<_>>()
    .await;
    outcomes.sort_by_key(|(index, _, _)| *index);

    if let Some(error) = outcomes
        .iter_mut()
        .find_map(|(_, _, result)| result.as_mut().err())
    {
        return Err(std::mem::replace(error, hpo_deadline_error()));
    }
    let unresolved = outcomes
        .iter()
        .filter(|(_, _, result)| result.as_ref().is_ok_and(Vec::is_empty))
        .map(|(_, query, _)| query.as_str())
        .collect::<Vec<_>>();
    if !unresolved.is_empty() {
        return Err(phenotype_query_no_match_error(&unresolved.join(", ")));
    }
    let phrase_rows = outcomes
        .into_iter()
        .map(|(_, query, rows)| (query, rows.expect("provider errors handled above")))
        .collect();
    flatten_resolved_phrase_rows(phrase_rows)
}

fn flatten_resolved_phrase_rows(
    phrase_rows: Vec<(String, Vec<HpoResolvedTerm>)>,
) -> Result<Vec<ResolvedPhenotypeQuery>, BioMcpError> {
    let mut resolved = Vec::new();
    let mut seen = HashSet::new();
    for (query, rows) in phrase_rows {
        for HpoResolvedTerm { id, label } in rows {
            let label = label.trim();
            if label.is_empty() {
                return Err(BioMcpError::Api {
                    api: "hpo".into(),
                    message: format!("HPO returned a blank label for {id}"),
                });
            }
            if seen.insert(id.clone()) {
                resolved.push(ResolvedPhenotypeQuery {
                    raw: query.clone(),
                    id,
                    label: label.into(),
                });
            }
        }
    }
    if resolved.len() > MAX_PHENOTYPE_TERMS {
        return Err(BioMcpError::InvalidArgument(format!(
            "Phenotype search resolved more than {MAX_PHENOTYPE_TERMS} unique HPO terms; refine the symptom phrases"
        )));
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
    pub resolved_query: Vec<ResolvedPhenotypeQuery>,
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
                direct_support: Vec::new(),
            },
        )
        .collect::<Vec<_>>();
    let returned = results.len();

    Ok(PhenotypeSearchPage {
        resolved_query: Vec::new(),
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
    let command_deadline = Instant::now() + PHENOTYPE_COMMAND_TIMEOUT;
    let hpo = HpoClient::new()?;
    let monarch = MonarchClient::new()?;
    search_phenotype_page_with_sources(hpo_terms, limit, offset, command_deadline, &hpo, &monarch)
        .await
}

async fn search_phenotype_page_with_sources<H: HpoResolutionSource, M: PhenotypeMonarchSource>(
    hpo_terms: &str,
    limit: usize,
    offset: usize,
    command_deadline: Instant,
    hpo: &H,
    monarch: &M,
) -> Result<PhenotypeSearchPage, BioMcpError> {
    let resolved_query =
        resolve_phenotype_query_terms_with_source(hpo_terms, command_deadline, hpo).await?;
    let terms = resolved_query
        .iter()
        .map(|term| term.id.clone())
        .collect::<Vec<_>>();
    let provider = timeout_at(command_deadline, monarch.similarity(&terms))
        .await
        .map_err(|_| monarch_deadline_error())??;
    let mut page = paginate_phenotype_matches(provider, limit, offset)?;
    page.resolved_query = resolved_query;
    if page.results.is_empty() {
        return Ok(page);
    }
    let disease_ids = page
        .results
        .iter()
        .map(|row| row.disease_id.clone())
        .collect::<Vec<_>>();
    let support_deadline = (Instant::now() + PHENOTYPE_SUPPORT_TIMEOUT).min(command_deadline);
    let lookup = timeout_at(
        support_deadline,
        monarch.direct_support(&disease_ids, &terms),
    )
    .await;
    match lookup {
        Ok(Ok(lookup)) => apply_direct_support(&mut page.results, &terms, Some(&lookup)),
        Ok(Err(_)) | Err(_) => apply_direct_support(&mut page.results, &terms, None),
    }
    Ok(page)
}

fn apply_direct_support(
    results: &mut [PhenotypeSearchResult],
    terms: &[String],
    lookup: Option<&MonarchDirectSupportLookup>,
) {
    for result in results {
        result.direct_support = terms
            .iter()
            .map(|hpo_id| PhenotypeDirectSupport {
                hpo_id: hpo_id.clone(),
                status: lookup.map_or(PhenotypeDirectSupportStatus::Unavailable, |lookup| {
                    lookup.status(&result.disease_id, hpo_id)
                }),
            })
            .collect();
    }
}

#[cfg(test)]
mod tests;
