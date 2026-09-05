//! Variant search against MyVariant.info with quality scoring and result shaping.

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::myvariant::{MyVariantClient, MyVariantHit, VariantSearchParams};
use crate::transform;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashSet};

mod diagnostics;
pub(crate) use diagnostics::SearchDiagnostic;
use diagnostics::{classify_provider_zero, search_params};

use super::{
    RequestedVariantIdentity, SourceVariantIdentity, VariantArticleResolution,
    VariantArticleResolutionBasis, VariantArticleResolutionContext, VariantIdentityComparison,
    VariantProviderValidation, VariantProviderValidationStatus, VariantResolutionStatus,
    VariantSearchFilters, VariantSearchResolution, VariantSearchResult, compare_variant_identity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VariantFilterEvaluationStatus {
    Evaluated,
    Unavailable,
}

pub(crate) type VariantFilterEvaluation = BTreeMap<&'static str, VariantFilterEvaluationStatus>;

#[derive(Debug, Clone)]
pub(crate) struct VariantSearchPage {
    pub results: Vec<VariantSearchResult>,
    pub total: Option<usize>,
    pub requested_variant: Option<RequestedVariantIdentity>,
    pub resolution: Option<VariantSearchResolution>,
    pub filter_evaluation: VariantFilterEvaluation,
    pub has_more: Option<bool>,
    pub diagnostics: Vec<SearchDiagnostic>,
}

fn filter_evaluation(
    filters: &VariantSearchFilters,
    diagnostics: &[SearchDiagnostic],
) -> VariantFilterEvaluation {
    let mut resolution = BTreeMap::new();
    let evaluated = VariantFilterEvaluationStatus::Evaluated;
    macro_rules! string_filter {
        ($field:ident) => {
            if filters
                .$field
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
            {
                resolution.insert(stringify!($field), evaluated);
            }
        };
    }

    string_filter!(gene);
    string_filter!(hgvsp);
    string_filter!(hgvsc);
    string_filter!(rsid);
    if filters.protein_alias.is_some() {
        resolution.insert("residue_alias", evaluated);
    }
    string_filter!(significance);
    if filters.max_frequency.is_some() {
        resolution.insert("max_frequency", evaluated);
    }
    if filters.min_cadd.is_some() {
        resolution.insert("min_cadd", evaluated);
    }
    string_filter!(consequence);
    string_filter!(review_status);
    string_filter!(population);
    if filters.revel_min.is_some() {
        resolution.insert("revel_min", evaluated);
    }
    if filters.gerp_min.is_some() {
        resolution.insert("gerp_min", evaluated);
    }
    string_filter!(tumor_site);
    string_filter!(condition);
    string_filter!(impact);
    if filters.lof {
        resolution.insert("lof", evaluated);
    }
    string_filter!(has);
    string_filter!(missing);
    string_filter!(therapy);

    if diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic, SearchDiagnostic::GeneUnavailable { .. }))
    {
        resolution.insert("gene", VariantFilterEvaluationStatus::Unavailable);
    }
    resolution
}

fn search_result_quality_score(row: &VariantSearchResult) -> i32 {
    let mut score = 0;
    if row
        .significance
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
    {
        score += 4;
    }
    if row.gnomad_af.is_some() {
        score += 4;
    }
    if row.clinvar_stars.is_some() {
        score += 3;
    }
    if row.revel.is_some() {
        score += 2;
    }
    if row.gerp.is_some() {
        score += 2;
    }
    if row
        .hgvs_p
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
    {
        score += 2;
    }
    if !row.gene.trim().is_empty() {
        score += 1;
    }
    score
}

pub fn search_query_summary(filters: &VariantSearchFilters) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("gene={v}"));
    }
    if let Some(alias) = filters.protein_alias.as_ref() {
        parts.push(format!("residue_alias={}", alias.label()));
    }
    if let Some(v) = filters
        .hgvsp
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("hgvsp={v}"));
    }
    if let Some(v) = filters
        .hgvsc
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("hgvsc={v}"));
    }
    if let Some(v) = filters
        .rsid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("rsid={v}"));
    }
    if let Some(v) = filters
        .significance
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("significance={v}"));
    }
    if let Some(v) = filters.max_frequency {
        parts.push(format!("max_frequency={v}"));
    }
    if let Some(v) = filters.min_cadd {
        parts.push(format!("min_cadd={v}"));
    }
    if let Some(v) = filters
        .consequence
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("consequence={v}"));
    }
    if let Some(v) = filters
        .review_status
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("review_status={v}"));
    }
    if let Some(v) = filters
        .population
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("population={v}"));
    }
    if let Some(v) = filters.revel_min {
        parts.push(format!("revel_min={v}"));
    }
    if let Some(v) = filters.gerp_min {
        parts.push(format!("gerp_min={v}"));
    }
    if let Some(v) = filters
        .tumor_site
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("tumor_site={v}"));
    }
    if let Some(v) = filters
        .condition
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("condition={v}"));
    }
    if let Some(v) = filters
        .impact
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("impact={v}"));
    }
    if filters.lof {
        parts.push("lof=true".to_string());
    }
    if let Some(v) = filters
        .has
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("has={v}"));
    }
    if let Some(v) = filters
        .missing
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("missing={v}"));
    }
    if let Some(v) = filters
        .therapy
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("therapy={v}"));
    }

    parts.join(", ")
}

// dead-code reason: mod::search is exercised by native tests or binary dispatch
#[allow(dead_code)]
pub async fn search(
    filters: &VariantSearchFilters,
    limit: usize,
) -> Result<Vec<VariantSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

#[derive(Debug, Clone)]
struct ProviderCandidate {
    hit: MyVariantHit,
    identity: SourceVariantIdentity,
    comparison: VariantIdentityComparison,
}

#[derive(Debug, Clone)]
struct ProviderScan {
    candidates: Vec<ProviderCandidate>,
    exhaustive: bool,
    available: bool,
}

fn provider_candidate(
    requested: &RequestedVariantIdentity,
    hit: MyVariantHit,
) -> ProviderCandidate {
    let identity = SourceVariantIdentity::from_myvariant_hit(&hit);
    let comparison = compare_variant_identity(requested, &identity);
    ProviderCandidate {
        hit,
        identity,
        comparison,
    }
}

fn article_resolution_context(
    requested: RequestedVariantIdentity,
    scan: ProviderScan,
) -> VariantArticleResolutionContext {
    let authoritative = requested.is_authoritative_refseq();
    let aliases = requested.normalized_aliases();
    let fallback_source_identities = scan
        .candidates
        .iter()
        .filter(|candidate| {
            !matches!(
                candidate.comparison,
                VariantIdentityComparison::Contradictory { .. }
            )
        })
        .map(|candidate| candidate.identity.clone())
        .collect();
    let validation = |status, matched_alias, contradictory_field| VariantProviderValidation {
        source: "myvariant".into(),
        status,
        matched_alias,
        contradictory_field,
    };
    if !scan.available {
        return VariantArticleResolutionContext {
            requested,
            resolution: VariantArticleResolution {
                status: if authoritative {
                    VariantResolutionStatus::Resolved
                } else {
                    VariantResolutionStatus::Ambiguous
                },
                normalized_aliases: aliases,
                exhaustive: false,
                basis: authoritative.then_some(VariantArticleResolutionBasis::CallerSupplied),
                provider_validation: validation(
                    VariantProviderValidationStatus::Unavailable,
                    None,
                    None,
                ),
            },
            source_id: None,
            source_identity: None,
            source_hit: None,
            fallback_source_identities,
            available: authoritative,
        };
    }

    let mut compatible = BTreeMap::<String, (BTreeSet<String>, Vec<&ProviderCandidate>)>::new();
    let mut saw_indeterminate = false;
    for candidate in &scan.candidates {
        match &candidate.comparison {
            VariantIdentityComparison::Compatible { matched_alias } => {
                let entry = compatible
                    .entry(candidate.identity.normalized_key())
                    .or_insert_with(|| (BTreeSet::new(), Vec::new()));
                if !matched_alias.trim().is_empty() {
                    entry.0.insert(matched_alias.clone());
                }
                entry.1.push(candidate);
            }
            VariantIdentityComparison::Indeterminate { .. } => saw_indeterminate = true,
            VariantIdentityComparison::Contradictory { .. } => {}
        }
    }

    if scan.exhaustive && compatible.len() == 1 && !saw_indeterminate {
        let (_, (matched_aliases, candidates)) = compatible.into_iter().next().expect("one key");
        let selected = candidates
            .into_iter()
            .min_by_key(|candidate| serde_json::to_string(&candidate.hit).unwrap_or_default())
            .expect("compatible candidate");
        return VariantArticleResolutionContext {
            requested,
            resolution: VariantArticleResolution {
                status: VariantResolutionStatus::Resolved,
                normalized_aliases: aliases,
                exhaustive: true,
                basis: Some(VariantArticleResolutionBasis::ProviderConfirmed),
                provider_validation: validation(
                    VariantProviderValidationStatus::Confirmed,
                    matched_aliases.into_iter().next(),
                    None,
                ),
            },
            source_id: Some(selected.hit.id.clone()),
            source_identity: Some(selected.identity.clone()),
            source_hit: Some(selected.hit.clone()),
            fallback_source_identities,
            available: true,
        };
    }

    if !scan.exhaustive || compatible.len() > 1 || saw_indeterminate {
        return VariantArticleResolutionContext {
            requested,
            resolution: VariantArticleResolution {
                status: if authoritative {
                    VariantResolutionStatus::Resolved
                } else {
                    VariantResolutionStatus::Ambiguous
                },
                normalized_aliases: aliases,
                exhaustive: scan.exhaustive,
                basis: authoritative.then_some(VariantArticleResolutionBasis::CallerSupplied),
                provider_validation: validation(
                    VariantProviderValidationStatus::Indeterminate,
                    None,
                    None,
                ),
            },
            source_id: None,
            source_identity: None,
            source_hit: None,
            fallback_source_identities,
            available: true,
        };
    }

    if !scan.candidates.is_empty() {
        let mut contradictory = scan
            .candidates
            .iter()
            .filter_map(|candidate| match candidate.comparison {
                VariantIdentityComparison::Contradictory { field } => {
                    Some((candidate.identity.normalized_key(), field))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        contradictory.sort();
        return VariantArticleResolutionContext {
            requested,
            resolution: VariantArticleResolution {
                status: VariantResolutionStatus::Unresolved,
                normalized_aliases: aliases,
                exhaustive: true,
                basis: None,
                provider_validation: validation(
                    VariantProviderValidationStatus::Contradictory,
                    None,
                    contradictory.first().map(|(_, field)| (*field).to_string()),
                ),
            },
            source_id: None,
            source_identity: None,
            source_hit: None,
            fallback_source_identities: Vec::new(),
            available: true,
        };
    }

    VariantArticleResolutionContext {
        requested,
        resolution: VariantArticleResolution {
            status: if authoritative {
                VariantResolutionStatus::Resolved
            } else {
                VariantResolutionStatus::Unresolved
            },
            normalized_aliases: aliases,
            exhaustive: true,
            basis: authoritative.then_some(VariantArticleResolutionBasis::CallerSupplied),
            provider_validation: validation(VariantProviderValidationStatus::NotFound, None, None),
        },
        source_id: None,
        source_identity: None,
        source_hit: None,
        fallback_source_identities,
        available: true,
    }
}

async fn direct_provider_scan(
    requested: &RequestedVariantIdentity,
    input: &str,
    execution: &crate::entities::article::variant_search::VariantArticleExecutionContext,
) -> ProviderScan {
    let Some(started) = execution.reserve("resolution") else {
        return ProviderScan {
            candidates: Vec::new(),
            exhaustive: false,
            available: false,
        };
    };
    let result = match MyVariantClient::new() {
        Ok(client) => client.get_all(input).await,
        Err(error) => Err(error),
    };
    let conclusive = result.is_ok() || matches!(result, Err(BioMcpError::NotFound { .. }));
    execution.record(
        "resolution",
        "myvariant",
        started,
        if conclusive { "ok" } else { "unavailable" },
        usize::from(conclusive),
    );
    match result {
        Ok(hits) => ProviderScan {
            candidates: hits
                .into_iter()
                .map(|hit| provider_candidate(requested, hit))
                .collect(),
            exhaustive: true,
            available: true,
        },
        Err(BioMcpError::NotFound { .. }) => ProviderScan {
            candidates: Vec::new(),
            exhaustive: true,
            available: true,
        },
        Err(_) => ProviderScan {
            candidates: Vec::new(),
            exhaustive: false,
            available: false,
        },
    }
}

async fn searched_provider_scan(
    requested: &RequestedVariantIdentity,
    execution: &crate::entities::article::variant_search::VariantArticleExecutionContext,
) -> ProviderScan {
    const SOURCE_PAGE: usize = 50;
    const MAX_CANDIDATES: usize = 1_000;
    let client = match MyVariantClient::new() {
        Ok(client) => client,
        Err(_) => {
            return ProviderScan {
                candidates: Vec::new(),
                exhaustive: false,
                available: false,
            };
        }
    };
    let mut provider_offset = 0;
    let mut candidates = Vec::new();
    let mut exhaustive = false;
    while provider_offset < MAX_CANDIDATES {
        let Some(started) = execution.reserve("resolution") else {
            break;
        };
        let result = client
            .search(&VariantSearchParams {
                gene: requested.gene.clone(),
                hgvsp: requested.protein_change.clone(),
                hgvsc: requested.coding_change.clone(),
                rsid: requested.rsid.clone(),
                protein_alias: None,
                significance: None,
                max_frequency: None,
                min_cadd: None,
                consequence: None,
                review_status: None,
                population: None,
                revel_min: None,
                gerp_min: None,
                tumor_site: None,
                condition: None,
                impact: None,
                lof: false,
                has: None,
                missing: None,
                therapy: None,
                limit: SOURCE_PAGE,
                offset: provider_offset,
            })
            .await;
        execution.record(
            "resolution",
            "myvariant",
            started,
            if result.is_ok() { "ok" } else { "unavailable" },
            usize::from(result.is_ok()),
        );
        let response = match result {
            Ok(response) => response,
            Err(_) => {
                return ProviderScan {
                    candidates: Vec::new(),
                    exhaustive: false,
                    available: false,
                };
            }
        };
        let provider_total = response.total;
        let hit_count = response.hits.len();
        let examined_count = hit_count.min(MAX_CANDIDATES - provider_offset);
        candidates.extend(
            response
                .hits
                .into_iter()
                .take(examined_count)
                .map(|hit| provider_candidate(requested, hit)),
        );
        provider_offset += examined_count;
        if candidate_scan_exhaustive(provider_total, provider_offset, hit_count) {
            exhaustive = true;
            break;
        }
    }
    ProviderScan {
        candidates,
        exhaustive,
        available: true,
    }
}

pub(crate) async fn resolve_article_variant_identity(
    requested: RequestedVariantIdentity,
    input: &str,
    execution: &crate::entities::article::variant_search::VariantArticleExecutionContext,
) -> Result<VariantArticleResolutionContext, BioMcpError> {
    let scan = if requested.genomic_accession.is_some() {
        direct_provider_scan(&requested, input, execution).await
    } else {
        searched_provider_scan(&requested, execution).await
    };
    Ok(article_resolution_context(requested, scan))
}

pub async fn search_page(
    filters: &VariantSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<VariantSearchPage, BioMcpError> {
    search_page_with_execution(filters, limit, offset, None).await
}

async fn search_page_with_execution(
    filters: &VariantSearchFilters,
    limit: usize,
    offset: usize,
    execution: Option<&crate::entities::article::variant_search::VariantArticleExecutionContext>,
) -> Result<VariantSearchPage, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let has_precision_filter = filters
        .hgvsp
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
        || filters
            .hgvsc
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .rsid
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.protein_alias.is_some()
        || filters
            .significance
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.max_frequency.is_some()
        || filters.min_cadd.is_some()
        || filters
            .review_status
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .population
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.revel_min.is_some()
        || filters.gerp_min.is_some()
        || filters
            .tumor_site
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .condition
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .impact
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.lof
        || filters
            .has
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .missing
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .therapy
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .consequence
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty());
    let fetch_limit = if has_precision_filter {
        limit
    } else {
        (limit.saturating_mul(40)).clamp(limit, 200)
    };

    let params_at = |page_limit, page_offset| {
        search_params(filters, filters.gene.clone(), page_limit, page_offset)
    };

    let client = MyVariantClient::new()?;
    let Some(requested) = filters.requested_identity.as_ref() else {
        let initial = client.search(&params_at(fetch_limit, offset)).await?;
        let (resp, diagnostics) = if initial.total == Some(0) {
            classify_provider_zero(&client, filters, initial, fetch_limit, offset).await?
        } else {
            (initial, Vec::new())
        };
        let mut out = resp
            .hits
            .iter()
            .map(transform::variant::from_myvariant_search_hit)
            .collect::<Vec<_>>();
        sort_results(&mut out);
        out.truncate(limit);
        let page = SearchPage::offset(out, resp.total);
        return Ok(VariantSearchPage {
            results: page.results,
            total: page.total,
            requested_variant: None,
            resolution: None,
            filter_evaluation: filter_evaluation(filters, &diagnostics),
            has_more: None,
            diagnostics,
        });
    };

    const SOURCE_PAGE: usize = 50;
    const MAX_CANDIDATES: usize = 1_000;
    let mut provider_offset = 0;
    let mut retained = Vec::new();
    let mut seen = HashSet::new();
    let mut saw_indeterminate = false;
    let mut exhaustive = false;
    let mut diagnostics = Vec::new();
    while provider_offset < MAX_CANDIDATES {
        let started = execution.and_then(|execution| execution.reserve("resolution"));
        if execution.is_some() && started.is_none() {
            break;
        }
        let result = client
            .search(&params_at(SOURCE_PAGE, provider_offset))
            .await;
        if let (Some(execution), Some(started)) = (execution, started) {
            execution.record(
                "resolution",
                "myvariant",
                started,
                if result.is_ok() { "ok" } else { "unavailable" },
                usize::from(result.is_ok()),
            );
        }
        let initial = result?;
        let (resp, classified) = if provider_offset == 0 && initial.total == Some(0) {
            classify_provider_zero(&client, filters, initial, SOURCE_PAGE, provider_offset).await?
        } else {
            (initial, Vec::new())
        };
        diagnostics.extend(classified);
        let provider_total = resp.total;
        let hit_count = resp.hits.len();
        let examined_count = hit_count.min(MAX_CANDIDATES - provider_offset);
        saw_indeterminate |= retain_compatible_hits(
            requested,
            resp.hits.into_iter().take(examined_count),
            &mut seen,
            &mut retained,
        );
        provider_offset += examined_count;
        if candidate_scan_exhaustive(provider_total, provider_offset, hit_count) {
            exhaustive = true;
            break;
        }
    }
    let mut page = finalize_exact_page(
        requested,
        retained,
        offset,
        limit,
        saw_indeterminate,
        exhaustive,
    );
    page.filter_evaluation = filter_evaluation(filters, &diagnostics);
    page.diagnostics = diagnostics;
    Ok(page)
}

fn candidate_scan_exhaustive(
    provider_total: Option<usize>,
    examined_offset: usize,
    returned_count: usize,
) -> bool {
    returned_count == 0 || provider_total.is_some_and(|total| examined_offset >= total)
}

fn retain_compatible_hits(
    requested: &RequestedVariantIdentity,
    hits: impl IntoIterator<Item = MyVariantHit>,
    seen: &mut HashSet<String>,
    retained: &mut Vec<VariantSearchResult>,
) -> bool {
    let mut saw_indeterminate = false;
    for hit in hits {
        let source = SourceVariantIdentity::from_myvariant_hit(&hit);
        match compare_variant_identity(requested, &source) {
            VariantIdentityComparison::Compatible { matched_alias } => {
                if seen.insert(source.normalized_key()) {
                    let mut row = transform::variant::from_myvariant_search_hit(&hit);
                    row.source_identity = Some(source);
                    row.matched_alias = Some(matched_alias);
                    retained.push(row);
                }
            }
            VariantIdentityComparison::Indeterminate { .. } => saw_indeterminate = true,
            VariantIdentityComparison::Contradictory { .. } => {}
        }
    }
    saw_indeterminate
}

fn finalize_exact_page(
    requested: &RequestedVariantIdentity,
    mut retained: Vec<VariantSearchResult>,
    offset: usize,
    limit: usize,
    saw_indeterminate: bool,
    exhaustive: bool,
) -> VariantSearchPage {
    sort_results(&mut retained);
    let compatible_count = retained.len();
    let status = resolution_status(compatible_count, saw_indeterminate, exhaustive);
    let total = exhaustive.then_some(compatible_count);
    let has_more = offset.saturating_add(limit) < compatible_count || !exhaustive;
    let results = retained.into_iter().skip(offset).take(limit).collect();
    VariantSearchPage {
        results,
        total,
        requested_variant: Some(requested.clone()),
        resolution: Some(VariantSearchResolution {
            status,
            normalized_aliases: requested.normalized_aliases(),
            exhaustive,
        }),
        filter_evaluation: BTreeMap::new(),
        has_more: Some(has_more),
        diagnostics: Vec::new(),
    }
}

fn resolution_status(
    compatible_identity_count: usize,
    saw_indeterminate: bool,
    exhaustive: bool,
) -> VariantResolutionStatus {
    if saw_indeterminate || !exhaustive || compatible_identity_count > 1 {
        VariantResolutionStatus::Ambiguous
    } else if compatible_identity_count == 1 {
        VariantResolutionStatus::Resolved
    } else {
        VariantResolutionStatus::Unresolved
    }
}

fn sort_results(out: &mut [VariantSearchResult]) {
    out.sort_by(|a, b| {
        search_result_quality_score(b)
            .cmp(&search_result_quality_score(a))
            .then_with(|| a.id.cmp(&b.id))
    });
}

#[cfg(test)]
mod tests;
