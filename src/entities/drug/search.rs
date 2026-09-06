//! Drug search workflows and search-only helpers.

use std::collections::{HashMap, HashSet};
use std::future::Future;

use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::cvx::{CvxAliasSource, CvxClient, CvxSyncMode};
use crate::sources::ema::{EmaClient, EmaDrugIdentity, EmaIdentitySource, EmaSyncMode};
use crate::sources::mychem::{MyChemHit, MyChemNdcField};
use crate::sources::openfda::OpenFdaClient;
use crate::sources::who_pq::{WhoPqClient, WhoPqSyncMode, WhoProductTypeFilter};
use crate::transform;

use super::label::extract_openfda_values_from_result;
use super::query::{AtcExpansion, build_mychem_query, mechanism_atc_expansions};
use super::{
    DrugRegion, DrugSearchFilters, DrugSearchMatchKind, DrugSearchPageWithRegion, DrugSearchResult,
    RankedDrugSearchPage, WhoPrequalificationEntry, WhoPrequalificationSearchResult,
    build_who_identity, direct_drug_lookup,
};

fn normalized_name(value: &str) -> String {
    value
        .trim()
        .trim_matches('.')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn string_field_matches(value: &crate::utils::serde::StringOrVec, query: &str) -> bool {
    match value {
        crate::utils::serde::StringOrVec::None => false,
        crate::utils::serde::StringOrVec::Single(value) => normalized_name(value) == query,
        crate::utils::serde::StringOrVec::Multiple(values) => {
            values.iter().any(|value| normalized_name(value) == query)
        }
    }
}

fn string_field_values(value: &crate::utils::serde::StringOrVec) -> Vec<&str> {
    match value {
        crate::utils::serde::StringOrVec::None => Vec::new(),
        crate::utils::serde::StringOrVec::Single(value) => vec![value],
        crate::utils::serde::StringOrVec::Multiple(values) => {
            values.iter().map(String::as_str).collect()
        }
    }
}

fn ndc_rows(hit: &MyChemHit) -> Vec<&crate::sources::mychem::MyChemNdc> {
    match hit.ndc.as_ref() {
        None => Vec::new(),
        Some(MyChemNdcField::One(row)) => vec![row],
        Some(MyChemNdcField::Many(rows)) => rows.iter().collect(),
    }
}

fn hit_matches_ema_query(hit: &MyChemHit, normalized_query: &str) -> bool {
    hit.openfda.as_ref().is_some_and(|openfda| {
        string_field_matches(&openfda.generic_name, normalized_query)
            || string_field_matches(&openfda.brand_name, normalized_query)
    }) || ndc_rows(hit).iter().any(|row| {
        row.nonproprietaryname
            .as_deref()
            .is_some_and(|value| normalized_name(value) == normalized_query)
    }) || hit
        .drugbank
        .as_ref()
        .and_then(|drugbank| drugbank.name.as_deref())
        .is_some_and(|value| normalized_name(value) == normalized_query)
        || hit
            .chembl
            .as_ref()
            .and_then(|chembl| chembl.pref_name.as_deref())
            .is_some_and(|value| normalized_name(value) == normalized_query)
}

fn push_ema_terms(
    out: &mut Vec<(String, EmaIdentitySource)>,
    values: impl IntoIterator<Item = (String, EmaIdentitySource)>,
) {
    out.extend(values.into_iter().filter_map(|(value, source)| {
        let cleaned = value
            .trim()
            .trim_matches('.')
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        (!cleaned.is_empty()).then_some((cleaned, source))
    }));
}

fn ema_identity_from_mychem_hits(query: &str, hits: &[MyChemHit]) -> Option<EmaDrugIdentity> {
    let normalized_query = normalized_name(query);
    let admitted = hits
        .iter()
        .filter(|hit| hit_matches_ema_query(hit, &normalized_query))
        .collect::<Vec<_>>();
    if admitted.is_empty() {
        return None;
    }
    let mut terms = Vec::new();
    for hit in admitted {
        if let Some(openfda) = hit.openfda.as_ref() {
            push_ema_terms(
                &mut terms,
                string_field_values(&openfda.generic_name)
                    .into_iter()
                    .map(|value| (value.to_string(), EmaIdentitySource::OpenFdaGenericName)),
            );
        }
        push_ema_terms(
            &mut terms,
            ndc_rows(hit).into_iter().filter_map(|row| {
                row.nonproprietaryname
                    .clone()
                    .map(|value| (value, EmaIdentitySource::NdcNonproprietaryName))
            }),
        );
        if let Some(value) = hit
            .drugbank
            .as_ref()
            .and_then(|drugbank| drugbank.name.clone())
        {
            push_ema_terms(&mut terms, [(value, EmaIdentitySource::DrugbankName)]);
        }
        if let Some(value) = hit
            .chembl
            .as_ref()
            .and_then(|chembl| chembl.pref_name.clone())
        {
            push_ema_terms(&mut terms, [(value, EmaIdentitySource::ChemblPrefName)]);
        }
        if let Some(openfda) = hit.openfda.as_ref() {
            push_ema_terms(
                &mut terms,
                string_field_values(&openfda.brand_name)
                    .into_iter()
                    .map(|value| (value.to_string(), EmaIdentitySource::OpenFdaBrandName)),
            );
        }
    }
    Some(EmaDrugIdentity::for_search(query, terms))
}

fn mychem_match_kind(hit: &MyChemHit, query: &str) -> DrugSearchMatchKind {
    let query = normalized_name(query);
    if hit
        .openfda
        .as_ref()
        .is_some_and(|value| string_field_matches(&value.brand_name, &query))
    {
        return DrugSearchMatchKind::ProductName;
    }
    let ndc_matches = hit.ndc.as_ref().is_some_and(|value| match value {
        MyChemNdcField::One(row) => row
            .nonproprietaryname
            .as_deref()
            .is_some_and(|value| normalized_name(value) == query),
        MyChemNdcField::Many(rows) => rows.iter().any(|row| {
            row.nonproprietaryname
                .as_deref()
                .is_some_and(|value| normalized_name(value) == query)
        }),
    });
    let active_matches = ndc_matches
        || hit
            .openfda
            .as_ref()
            .is_some_and(|value| string_field_matches(&value.generic_name, &query))
        || hit
            .drugbank
            .as_ref()
            .and_then(|value| value.name.as_deref())
            .is_some_and(|value| normalized_name(value) == query)
        || hit
            .chembl
            .as_ref()
            .and_then(|value| value.pref_name.as_deref())
            .is_some_and(|value| normalized_name(value) == query);
    if active_matches {
        DrugSearchMatchKind::ActiveSubstance
    } else if hit.drugbank.as_ref().is_some_and(|value| {
        value
            .synonyms
            .iter()
            .any(|alias| normalized_name(alias) == query)
    }) {
        DrugSearchMatchKind::Alias
    } else {
        DrugSearchMatchKind::BroadText
    }
}

fn rank_drug_candidates<T>(candidates: &mut [(T, DrugSearchMatchKind)]) {
    candidates.sort_by_key(|(_, kind)| kind.rank());
}

pub async fn search(
    filters: &DrugSearchFilters,
    limit: usize,
) -> Result<Vec<DrugSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub async fn search_page(
    filters: &DrugSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<DrugSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let q = build_mychem_query(filters)?;

    let client = crate::sources::mychem::MyChemClient::new()?;
    // Fetch extra hits to account for de-duplication by normalized name.
    let fetch_limit = if filters
        .mechanism
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
    {
        MAX_SEARCH_LIMIT
    } else {
        (limit.saturating_mul(2)).min(MAX_SEARCH_LIMIT)
    };
    let resp = client
        .query_with_fields(
            &q,
            fetch_limit,
            offset,
            crate::sources::mychem::MYCHEM_FIELDS_SEARCH,
        )
        .await?;

    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<DrugSearchResult> = Vec::new();
    for hit in &resp.hits {
        let Some(mut r) = transform::drug::from_mychem_search_hit(hit) else {
            continue;
        };

        if let Some(requested_target) = filters
            .target
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            if !hit_mentions_target(hit, requested_target) {
                continue;
            }
            // Display the matched target explicitly so multi-target drugs are not misleading.
            r.target = Some(requested_target.to_ascii_uppercase());
        }

        if let Some(requested_mechanism) = filters
            .mechanism
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            && !hit_mentions_mechanism(hit, requested_mechanism)
        {
            continue;
        }

        // Normalize and de-duplicate by name.
        r.name = r.name.trim().to_ascii_lowercase();
        if r.name.is_empty() {
            continue;
        }
        if !seen.insert(r.name.clone()) {
            continue;
        }

        out.push(r);
        if out.len() >= limit {
            break;
        }
    }

    if should_attempt_openfda_fallback(&out, offset, filters)
        && let Some(query) = filters
            .query
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        && let Ok(client) = OpenFdaClient::new()
        && let Ok(Some(label_response)) = client.label_search(query).await
    {
        let rows = search_results_from_openfda_label_response(&label_response, query, limit);
        if !rows.is_empty() {
            let total = rows.len();
            return Ok(SearchPage::offset(rows, Some(total)));
        }
    }

    Ok(SearchPage::offset(out, Some(resp.total)))
}

async fn search_ranked_name_us_page(
    filters: &DrugSearchFilters,
    query: &str,
    limit: usize,
    offset: usize,
) -> Result<RankedDrugSearchPage<DrugSearchResult>, BioMcpError> {
    const PAGE_SIZE: usize = 50;
    crate::sources::validate_biothings_result_window("MyChem search", limit, offset)?;
    let q = build_mychem_query(filters)?;
    let client = crate::sources::mychem::MyChemClient::new()?;
    let mut provider_offset = 0;
    let mut candidates: Vec<(DrugSearchResult, DrugSearchMatchKind)> = Vec::new();
    let mut positions: HashMap<String, usize> = HashMap::new();

    loop {
        let response = client
            .query_with_fields(
                &q,
                PAGE_SIZE,
                provider_offset,
                crate::sources::mychem::MYCHEM_FIELDS_SEARCH,
            )
            .await?;
        if response.total > crate::sources::BIOTHINGS_MAX_RESULT_WINDOW {
            return Err(BioMcpError::InvalidArgument(format!(
                "Drug search matched {} MyChem rows; narrow the query so complete ranking fits the 10,000-row provider window.",
                response.total
            )));
        }
        let returned = response.hits.len();
        for hit in &response.hits {
            let Some(mut row) = transform::drug::from_mychem_search_hit(hit) else {
                continue;
            };
            row.name = normalized_name(&row.name);
            if row.name.is_empty() {
                continue;
            }
            let kind = mychem_match_kind(hit, query);
            if let Some(index) = positions.get(&row.name).copied() {
                if kind.rank() < candidates[index].1.rank() {
                    candidates[index].1 = kind;
                }
            } else {
                positions.insert(row.name.clone(), candidates.len());
                candidates.push((row, kind));
            }
        }
        provider_offset = provider_offset.saturating_add(returned);
        if returned == 0 || provider_offset >= response.total {
            break;
        }
    }

    if candidates.is_empty() {
        let page = search_page(filters, limit, offset).await?;
        let query = normalized_name(query);
        let kinds = page
            .results
            .iter()
            .map(|row| {
                if normalized_name(&row.name) == query {
                    DrugSearchMatchKind::ProductName
                } else {
                    DrugSearchMatchKind::BroadText
                }
            })
            .collect();
        return Ok(RankedDrugSearchPage::offset(
            page.results,
            page.total,
            kinds,
        ));
    }

    rank_drug_candidates(&mut candidates);
    let total = candidates.len();
    let selected = candidates
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    let kinds = selected.iter().map(|(_, kind)| *kind).collect();
    let results = selected.into_iter().map(|(row, _)| row).collect();
    Ok(RankedDrugSearchPage::offset(results, Some(total), kinds))
}

pub(super) fn should_attempt_openfda_fallback(
    out: &[DrugSearchResult],
    offset: usize,
    filters: &DrugSearchFilters,
) -> bool {
    out.is_empty() && offset == 0 && !filters.has_structured_filters()
}

pub(super) fn hit_mentions_target(hit: &MyChemHit, target: &str) -> bool {
    let target = target.trim();
    if target.is_empty() {
        return false;
    }
    let target_upper = target.to_ascii_uppercase();

    if let Some(gtopdb) = hit.gtopdb.as_ref() {
        for row in &gtopdb.interaction_targets {
            if row
                .symbol
                .as_deref()
                .map(str::trim)
                .is_some_and(|s| s.eq_ignore_ascii_case(&target_upper))
            {
                return true;
            }
        }
    }

    if let Some(chembl) = hit.chembl.as_ref() {
        for row in &chembl.drug_mechanisms {
            if row
                .target_name
                .as_deref()
                .map(str::trim)
                .is_some_and(|s| s.eq_ignore_ascii_case(&target_upper))
            {
                return true;
            }
        }
    }

    false
}

fn text_matches_mechanism(candidate: &str, mechanism: &str, tokens: &[&str]) -> bool {
    let candidate = candidate.trim();
    if candidate.is_empty() {
        return false;
    }
    let candidate_lower = candidate.to_ascii_lowercase();
    if candidate_lower.contains(mechanism) {
        return true;
    }
    tokens.iter().all(|token| candidate_lower.contains(token))
}

pub(super) fn hit_mentions_mechanism(hit: &MyChemHit, mechanism: &str) -> bool {
    let mechanism = mechanism.trim().to_ascii_lowercase();
    if mechanism.is_empty() {
        return false;
    }
    let tokens = mechanism
        .split_whitespace()
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();
    let atc_expansions = mechanism_atc_expansions(&mechanism);

    if let Some(chembl) = hit.chembl.as_ref() {
        for row in &chembl.drug_mechanisms {
            if row
                .action_type
                .as_deref()
                .is_some_and(|action| text_matches_mechanism(action, &mechanism, &tokens))
                || row
                    .mechanism_of_action
                    .as_deref()
                    .is_some_and(|action| text_matches_mechanism(action, &mechanism, &tokens))
            {
                return true;
            }
        }

        if chembl
            .atc_classifications
            .clone()
            .into_vec()
            .iter()
            .any(|code| {
                atc_expansions.iter().any(|expansion| match expansion {
                    AtcExpansion::Prefix(prefix) => code.starts_with(prefix),
                    AtcExpansion::Exact(exact) => code == exact,
                })
            })
        {
            return true;
        }
    }

    if let Some(ndc) = hit.ndc.as_ref() {
        let matches_class = |value: &str| text_matches_mechanism(value, &mechanism, &tokens);
        match ndc {
            MyChemNdcField::One(v) => {
                if v.pharm_classes
                    .iter()
                    .filter_map(|cls| cls.as_str())
                    .any(matches_class)
                {
                    return true;
                }
            }
            MyChemNdcField::Many(rows) => {
                if rows.iter().any(|row| {
                    row.pharm_classes
                        .iter()
                        .filter_map(|cls| cls.as_str())
                        .any(matches_class)
                }) {
                    return true;
                }
            }
        }
    }

    false
}

pub(super) fn search_results_from_openfda_label_response(
    label_response: &serde_json::Value,
    query: &str,
    max_results: usize,
) -> Vec<DrugSearchResult> {
    let query = query.trim();
    if query.is_empty() || max_results == 0 {
        return Vec::new();
    }

    let Some(results) = label_response.get("results").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let mut exact_matches: Vec<DrugSearchResult> = Vec::new();
    let mut others: Vec<DrugSearchResult> = Vec::new();
    for result in results {
        let brand_names = extract_openfda_values_from_result(result, "brand_name");
        let generic_names = extract_openfda_values_from_result(result, "generic_name");
        let Some(name) = generic_names
            .first()
            .cloned()
            .or_else(|| brand_names.first().cloned())
        else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }

        let row = DrugSearchResult {
            name,
            drugbank_id: None,
            drug_type: None,
            mechanism: None,
            target: None,
        };
        let is_exact_brand_match = brand_names
            .iter()
            .map(|value| value.trim())
            .any(|value| value.eq_ignore_ascii_case(query));
        if is_exact_brand_match {
            exact_matches.push(row);
        } else {
            others.push(row);
        }
    }

    let mut out: Vec<DrugSearchResult> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for row in exact_matches.into_iter().chain(others) {
        if !seen.insert(row.name.clone()) {
            continue;
        }
        out.push(row);
        if out.len() >= max_results {
            break;
        }
    }
    out
}

async fn try_resolve_drug_identity(
    name: &str,
) -> Option<crate::sources::mychem::MyChemQueryResponse> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    match direct_drug_lookup(name).await {
        Ok(resp) => Some(resp),
        Err(err) => {
            warn!(query = %name, "Drug identity resolution unavailable for EMA alias expansion: {err}");
            None
        }
    }
}

fn should_resolve_drug_identity(region: DrugRegion, product_type: WhoProductTypeFilter) -> bool {
    !(matches!(region, DrugRegion::Who) && matches!(product_type, WhoProductTypeFilter::Vaccine))
}

pub async fn search_name_query_with_region(
    query: &str,
    limit: usize,
    offset: usize,
    region: DrugRegion,
    product_type: WhoProductTypeFilter,
) -> Result<DrugSearchPageWithRegion, BioMcpError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter is required. Example: biomcp search drug -q pembrolizumab".into(),
        ));
    }

    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let filters = DrugSearchFilters {
        query: Some(query.to_string()),
        ..Default::default()
    };

    let who_vaccine_search = region.includes_who()
        && matches!(
            product_type,
            crate::sources::who_pq::WhoProductTypeFilter::Vaccine
        );
    let resolved_response = if should_resolve_drug_identity(region, product_type) {
        try_resolve_drug_identity(query).await
    } else {
        None
    };
    let resolved_ema_identity = resolved_response
        .as_ref()
        .and_then(|response| ema_identity_from_mychem_hits(query, &response.hits));
    let resolved_drug = resolved_response.as_ref().and_then(|response| {
        (!response.hits.is_empty()).then(|| {
            let selected = transform::drug::select_hits_for_name(&response.hits, query);
            transform::drug::merge_mychem_hits(&selected, query)
        })
    });
    let needs_cvx_aliases =
        (region.includes_eu() && resolved_ema_identity.is_none()) || who_vaccine_search;
    let (cvx_search_aliases, cvx_aliases) = if needs_cvx_aliases {
        match CvxClient::ready(CvxSyncMode::Auto).await {
            Ok(client) => match client.lookup_search_alias_sets(query) {
                Ok(aliases) => aliases,
                Err(err) => {
                    warn!(
                        query = %query,
                        "CDC CVX/MVX alias lookup unavailable for vaccine bridge: {err}"
                    );
                    (Vec::new(), Vec::new())
                }
            },
            Err(err) => {
                warn!(
                    query = %query,
                    "CDC CVX/MVX auto-sync unavailable for vaccine bridge: {err}"
                );
                (Vec::new(), Vec::new())
            }
        }
    } else {
        (Vec::new(), Vec::new())
    };
    let eu_identity = if region.includes_eu() {
        match resolved_ema_identity {
            Some(identity) => identity,
            None => {
                if cvx_search_aliases.is_empty() {
                    EmaDrugIdentity::for_search(query, Vec::new())
                } else {
                    EmaDrugIdentity::for_search(
                        query,
                        cvx_search_aliases
                            .into_iter()
                            .map(|alias| {
                                let source = match alias.source {
                                    CvxAliasSource::ShortDescription => {
                                        EmaIdentitySource::CvxShortDescription
                                    }
                                    CvxAliasSource::FullVaccineName => {
                                        EmaIdentitySource::CvxFullVaccineName
                                    }
                                };
                                (alias.value, source)
                            })
                            .collect(),
                    )
                }
            }
        }
    } else {
        EmaDrugIdentity::for_search(query, Vec::new())
    };
    let who_identity = if who_vaccine_search {
        if cvx_aliases.is_empty() {
            crate::sources::who_pq::WhoPqIdentity::new(query)
        } else {
            crate::sources::who_pq::WhoPqIdentity::with_aliases(query, None, &cvx_aliases)
        }
    } else {
        match resolved_drug.as_ref() {
            Some(drug) => build_who_identity(query, drug),
            None => crate::sources::who_pq::WhoPqIdentity::new(query),
        }
    };

    let eu_client = if region.includes_eu() {
        Some(EmaClient::ready(EmaSyncMode::Auto).await?)
    } else {
        None
    };
    let who_client = if region.includes_who() {
        Some(WhoPqClient::ready(WhoPqSyncMode::Auto).await?)
    } else {
        None
    };

    match region {
        DrugRegion::Us => Ok(DrugSearchPageWithRegion::Us(
            search_ranked_name_us_page(&filters, query, limit, offset).await?,
        )),
        DrugRegion::Eu => Ok(DrugSearchPageWithRegion::Eu(
            eu_client
                .as_ref()
                .expect("EU client should exist for EU region")
                .search_medicines(&eu_identity, limit, offset)?,
        )),
        DrugRegion::Who => Ok(DrugSearchPageWithRegion::Who(
            who_client
                .as_ref()
                .expect("WHO client should exist for WHO region")
                .search(&who_identity, limit, offset, product_type)?,
        )),
        DrugRegion::All => Ok(DrugSearchPageWithRegion::All {
            us: search_ranked_name_us_page(&filters, query, limit, offset).await?,
            eu: eu_client
                .as_ref()
                .expect("EU client should exist for all region")
                .search_medicines(&eu_identity, limit, offset)?,
            who: who_client
                .as_ref()
                .expect("WHO client should exist for all region")
                .search(&who_identity, limit, offset, product_type)?,
        }),
    }
}

pub(super) async fn search_structured_who_page(
    filters: &DrugSearchFilters,
    limit: usize,
    offset: usize,
    product_type: WhoProductTypeFilter,
) -> Result<SearchPage<WhoPrequalificationSearchResult>, BioMcpError> {
    let who_rows = crate::sources::who_pq::filter_rows_by_product_type(
        &WhoPqClient::ready(WhoPqSyncMode::Auto).await?.read_rows()?,
        product_type,
    );
    search_structured_who_page_with(
        filters,
        limit,
        offset,
        product_type,
        |filters, page_limit, page_offset| {
            let filters = filters.clone();
            async move { search_page(&filters, page_limit, page_offset).await }
        },
        |name| {
            crate::sources::who_pq::filter_regulatory_rows(
                &who_rows,
                &crate::sources::who_pq::WhoPqIdentity::new(name),
            )
        },
    )
    .await
}

pub(super) async fn search_structured_who_page_with<F, Fut, M>(
    filters: &DrugSearchFilters,
    limit: usize,
    offset: usize,
    product_type: WhoProductTypeFilter,
    mut fetch_page: F,
    mut regulatory_rows: M,
) -> Result<SearchPage<WhoPrequalificationSearchResult>, BioMcpError>
where
    F: FnMut(&DrugSearchFilters, usize, usize) -> Fut,
    Fut: Future<Output = Result<SearchPage<DrugSearchResult>, BioMcpError>>,
    M: FnMut(&str) -> Vec<WhoPrequalificationEntry>,
{
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let mut expanded = Vec::new();
    let mut seen_ids = HashSet::new();
    let mut mychem_offset = 0usize;
    let page_size = MAX_SEARCH_LIMIT;

    loop {
        let page = fetch_page(filters, page_size, mychem_offset).await?;
        for candidate in &page.results {
            for row in crate::sources::who_pq::filter_rows_by_product_type(
                &regulatory_rows(&candidate.name),
                product_type,
            ) {
                if seen_ids.insert(row.stable_identifier_key()) {
                    expanded.push(WhoPrequalificationSearchResult {
                        kind: row.kind,
                        inn: row.inn,
                        product_type: row.product_type,
                        therapeutic_area: row.therapeutic_area,
                        presentation: row.presentation,
                        dosage_form: row.dosage_form,
                        applicant: row.applicant,
                        who_reference_number: row.who_reference_number,
                        who_product_id: row.who_product_id,
                        listing_basis: row.listing_basis,
                        prequalification_date: row.prequalification_date,
                        vaccine_type: row.vaccine_type,
                        commercial_name: row.commercial_name,
                        dose_count: row.dose_count,
                        manufacturer: row.manufacturer,
                        responsible_nra: row.responsible_nra,
                    });
                }
            }
        }

        let exhausted = page.results.is_empty()
            || page
                .total
                .is_some_and(|total| mychem_offset + page_size >= total);
        if exhausted {
            let total = expanded.len();
            let results = expanded.into_iter().skip(offset).take(limit).collect();
            return Ok(SearchPage::offset(results, Some(total)));
        }

        if expanded.len() > offset + limit {
            let results = expanded.into_iter().skip(offset).take(limit).collect();
            return Ok(SearchPage::offset(results, None));
        }

        mychem_offset += page_size;
    }
}

pub async fn search_page_with_region(
    filters: &DrugSearchFilters,
    limit: usize,
    offset: usize,
    region: DrugRegion,
    product_type: WhoProductTypeFilter,
) -> Result<DrugSearchPageWithRegion, BioMcpError> {
    if filters.has_structured_filters() {
        if matches!(region, DrugRegion::Who)
            && matches!(
                product_type,
                crate::sources::who_pq::WhoProductTypeFilter::Vaccine
            )
        {
            return Err(BioMcpError::InvalidArgument(
                "WHO vaccine search is plain name/brand only. Use a vaccine name or brand with --region who --product-type vaccine.".into(),
            ));
        }
        return match region {
            DrugRegion::Us => Ok(DrugSearchPageWithRegion::Us(
                search_page(filters, limit, offset).await?.into(),
            )),
            DrugRegion::Who => Ok(DrugSearchPageWithRegion::Who(
                search_structured_who_page(filters, limit, offset, product_type)
                    .await?
                    .into(),
            )),
            DrugRegion::Eu | DrugRegion::All => Err(BioMcpError::InvalidArgument(
                "EMA and all-region search currently support name/alias lookups only; use --region us for structured MyChem filters or --region who to filter structured U.S. hits through WHO prequalification.".into(),
            )),
        };
    }

    search_name_query_with_region(
        filters.query.as_deref().unwrap_or_default(),
        limit,
        offset,
        region,
        product_type,
    )
    .await
}

#[cfg(test)]
mod tests;
