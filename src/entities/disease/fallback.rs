//! Discover-based fallback search and fallback row resolution for diseases.

use super::*;
use std::collections::HashMap;

use super::resolution::{
    DiseaseXrefKind, disease_candidate_score, normalize_disease_id, normalize_disease_text,
    preferred_crosswalk_hit, resolver_queries,
};
use super::search::MAX_DISEASE_SEARCH_LIMIT;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RankedDiseaseFallbackCandidate {
    label: String,
    synonyms: Vec<String>,
    match_tier: crate::entities::discover::MatchTier,
    confidence: crate::entities::discover::DiscoverConfidence,
    source_ids: Vec<DiseaseFallbackId>,
    original_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum DiseaseFallbackId {
    CanonicalOntology(String),
    Crosswalk(DiseaseXrefKind, String),
}

const FALLBACK_LOOKUP_CAP: usize = 12;

fn match_tier_rank(value: crate::entities::discover::MatchTier) -> u8 {
    match value {
        crate::entities::discover::MatchTier::Exact => 0,
        crate::entities::discover::MatchTier::Prefix => 1,
        crate::entities::discover::MatchTier::Contains => 2,
        crate::entities::discover::MatchTier::Weak => 3,
    }
}

fn discover_confidence_rank(value: crate::entities::discover::DiscoverConfidence) -> u8 {
    match value {
        crate::entities::discover::DiscoverConfidence::CanonicalId => 0,
        crate::entities::discover::DiscoverConfidence::UmlsOnly => 1,
        crate::entities::discover::DiscoverConfidence::LabelOnly => 2,
    }
}

fn is_generic_disease_label(value: &str) -> bool {
    let tokens = normalize_disease_text(value)
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return false;
    }

    const GENERIC: &[&str] = &["disease", "disorder", "syndrome", "condition"];
    tokens.len() <= 2
        && tokens
            .iter()
            .all(|token| GENERIC.iter().any(|generic| token == generic))
}

fn contains_all_query_tokens(query_tokens: &[String], haystacks: &[String]) -> bool {
    const GENERIC: &[&str] = &["disease", "disorder", "syndrome", "condition"];
    let query_tokens = query_tokens
        .iter()
        .filter(|token| !GENERIC.iter().any(|generic| *token == generic))
        .collect::<Vec<_>>();
    if query_tokens.is_empty() {
        return false;
    }

    haystacks.iter().any(|haystack| {
        let normalized = normalize_disease_text(haystack);
        query_tokens
            .iter()
            .all(|token| normalized.contains(token.as_str()))
    })
}

fn normalize_supported_discover_source_id(source: &str, id: &str) -> Option<DiseaseFallbackId> {
    let source = source.trim();
    let id = id.trim();
    if source.is_empty() || id.is_empty() {
        return None;
    }

    if source.eq_ignore_ascii_case("MONDO") || source.eq_ignore_ascii_case("DOID") {
        return normalize_disease_id(&format!("{source}:{id}"))
            .map(DiseaseFallbackId::CanonicalOntology);
    }

    if source.eq_ignore_ascii_case("MESH") {
        let value = id
            .trim_start_matches("MESH:")
            .trim_start_matches("mesh:")
            .trim();
        if !value.is_empty() {
            return Some(DiseaseFallbackId::Crosswalk(
                DiseaseXrefKind::Mesh,
                value.to_string(),
            ));
        }
        return None;
    }

    if source.eq_ignore_ascii_case("OMIM") {
        let value = id
            .trim_start_matches("OMIM:")
            .trim_start_matches("omim:")
            .trim();
        if !value.is_empty() {
            return Some(DiseaseFallbackId::Crosswalk(
                DiseaseXrefKind::Omim,
                value.to_string(),
            ));
        }
        return None;
    }

    if source.eq_ignore_ascii_case("ICD10CM") || source.eq_ignore_ascii_case("ICD10") {
        let value = id
            .trim_start_matches("ICD10CM:")
            .trim_start_matches("icd10cm:")
            .trim_start_matches("ICD10:")
            .trim_start_matches("icd10:")
            .trim();
        if !value.is_empty() {
            return Some(DiseaseFallbackId::Crosswalk(
                DiseaseXrefKind::Icd10Cm,
                value.to_string(),
            ));
        }
        return None;
    }

    None
}

fn ranked_fallback_source_ids(
    concept: &crate::entities::discover::DiscoverConcept,
) -> Vec<DiseaseFallbackId> {
    let mut canonical_ids = Vec::new();
    let mut per_kind: HashMap<DiseaseXrefKind, Vec<String>> = HashMap::new();
    let mut seen = HashSet::new();
    let mut push = |value: DiseaseFallbackId| {
        let key = match &value {
            DiseaseFallbackId::CanonicalOntology(id) => {
                format!("canonical:{}", id.to_ascii_uppercase())
            }
            DiseaseFallbackId::Crosswalk(kind, id) => {
                format!("{}:{}", kind.display_name(), id.to_ascii_uppercase())
            }
        };
        if seen.insert(key) {
            match value {
                DiseaseFallbackId::CanonicalOntology(id) => canonical_ids.push(id),
                DiseaseFallbackId::Crosswalk(kind, id) => {
                    per_kind.entry(kind).or_default().push(id);
                }
            }
        }
    };

    if let Some(primary_id) = concept.primary_id.as_deref()
        && let Some((source, value)) = primary_id.split_once(':')
        && let Some(normalized) = normalize_supported_discover_source_id(source, value)
    {
        push(normalized);
    }

    for xref in &concept.xrefs {
        if let Some(normalized) = normalize_supported_discover_source_id(&xref.source, &xref.id) {
            push(normalized);
        }
    }

    let mut ranked = Vec::new();
    for id in canonical_ids {
        ranked.push(DiseaseFallbackId::CanonicalOntology(id));
    }
    for kind in [
        DiseaseXrefKind::Mesh,
        DiseaseXrefKind::Omim,
        DiseaseXrefKind::Icd10Cm,
    ] {
        if let Some(values) = per_kind.remove(&kind) {
            for value in values {
                ranked.push(DiseaseFallbackId::Crosswalk(kind, value));
            }
        }
    }
    ranked
}

fn rank_disease_fallback_candidates(
    query: &str,
    concepts: &[crate::entities::discover::DiscoverConcept],
) -> Vec<RankedDiseaseFallbackCandidate> {
    let query_tokens = normalize_disease_text(query)
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();

    let mut candidates = concepts
        .iter()
        .enumerate()
        .filter(|(_, concept)| {
            concept.primary_type == crate::entities::discover::DiscoverType::Disease
        })
        .filter_map(|(index, concept)| {
            let source_ids = ranked_fallback_source_ids(concept);
            if source_ids.is_empty() {
                return None;
            }

            Some(RankedDiseaseFallbackCandidate {
                label: concept.label.clone(),
                synonyms: concept.synonyms.clone(),
                match_tier: concept.match_tier,
                confidence: concept.confidence,
                source_ids,
                original_index: index,
            })
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        let left_haystacks = std::iter::once(left.label.clone())
            .chain(left.synonyms.iter().cloned())
            .collect::<Vec<_>>();
        let right_haystacks = std::iter::once(right.label.clone())
            .chain(right.synonyms.iter().cloned())
            .collect::<Vec<_>>();
        let left_token_match = contains_all_query_tokens(&query_tokens, &left_haystacks);
        let right_token_match = contains_all_query_tokens(&query_tokens, &right_haystacks);
        let left_generic = is_generic_disease_label(&left.label);
        let right_generic = is_generic_disease_label(&right.label);

        right_token_match
            .cmp(&left_token_match)
            .then_with(|| left_generic.cmp(&right_generic))
            .then_with(|| match_tier_rank(left.match_tier).cmp(&match_tier_rank(right.match_tier)))
            .then_with(|| {
                discover_confidence_rank(left.confidence)
                    .cmp(&discover_confidence_rank(right.confidence))
            })
            .then_with(|| left.original_index.cmp(&right.original_index))
    });

    candidates
}

async fn resolve_fallback_row(
    client: &MyDiseaseClient,
    prefer_doid: bool,
    source_id: &DiseaseFallbackId,
) -> Result<Option<DiseaseSearchResult>, BioMcpError> {
    let row = match source_id {
        DiseaseFallbackId::CanonicalOntology(id) => {
            let hit = match client.get(id).await {
                Ok(hit) => hit,
                Err(BioMcpError::NotFound { .. }) => return Ok(None),
                Err(err) => return Err(err),
            };
            let mut row = transform::disease::from_mydisease_search_hit(&hit);
            if prefer_doid && let Some(doid) = transform::disease::doid_from_mydisease_hit(&hit) {
                row.id = doid;
            }
            row.resolved_via = Some(if id.starts_with("DOID:") {
                "DOID canonical".to_string()
            } else {
                "MONDO canonical".to_string()
            });
            row.source_id = Some(id.clone());
            row
        }
        DiseaseFallbackId::Crosswalk(kind, source_value) => {
            let response = client
                .lookup_disease_by_xref(kind.source_key(), source_value, 5)
                .await?;
            let Some(best) = preferred_crosswalk_hit(response.hits) else {
                return Ok(None);
            };
            let mut row = transform::disease::from_mydisease_search_hit(&best);
            if prefer_doid && let Some(doid) = transform::disease::doid_from_mydisease_hit(&best) {
                row.id = doid;
            }
            row.resolved_via = Some(kind.resolved_via_label());
            row.source_id = Some(format!("{}:{source_value}", kind.display_name()));
            row
        }
    };

    Ok(Some(row))
}

async fn collect_fallback_search_page<F, Fut>(
    query: &str,
    limit: usize,
    offset: usize,
    candidates: Vec<RankedDiseaseFallbackCandidate>,
    mut resolve_source_id: F,
) -> Result<Option<SearchPage<DiseaseSearchResult>>, BioMcpError>
where
    F: FnMut(DiseaseFallbackId) -> Fut,
    Fut: std::future::Future<Output = Result<Option<DiseaseSearchResult>, BioMcpError>>,
{
    let needed = offset.saturating_add(limit);
    let query_tokens = normalize_disease_text(query)
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut lookups = 0usize;
    let mut deduped = Vec::new();
    let mut seen_ids = HashSet::new();

    for candidate in candidates {
        if deduped.len() >= needed || lookups >= FALLBACK_LOOKUP_CAP {
            break;
        }

        for source_id in candidate.source_ids {
            if deduped.len() >= needed || lookups >= FALLBACK_LOOKUP_CAP {
                break;
            }

            lookups += 1;
            let Some(mut row) = (match resolve_source_id(source_id.clone()).await {
                Ok(row) => row,
                Err(err) => {
                    warn!("Disease search fallback row resolution failed: {err}");
                    continue;
                }
            }) else {
                continue;
            };

            let candidate_score = disease_candidate_score(query, &candidate.label);
            let row_score = disease_candidate_score(query, &row.name);
            if row.name == row.id || candidate_score > row_score {
                row.name = candidate.label.clone();
            }
            let mut haystacks = vec![row.name.clone()];
            haystacks.extend(candidate.synonyms.iter().cloned());
            if !contains_all_query_tokens(&query_tokens, &haystacks) {
                continue;
            }
            if !seen_ids.insert(row.id.clone()) {
                break;
            }

            deduped.push(row);
            break;
        }
    }

    if deduped.len() <= offset {
        return Ok(None);
    }

    let total = deduped.len();
    let results = deduped
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    if results.is_empty() {
        return Ok(None);
    }

    Ok(Some(SearchPage::offset(results, Some(total))))
}

pub(crate) async fn fallback_search_page(
    filters: &DiseaseSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<Option<SearchPage<DiseaseSearchResult>>, BioMcpError> {
    if limit == 0 || limit > MAX_DISEASE_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_DISEASE_SEARCH_LIMIT}"
        )));
    }

    let query = filters
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search disease -q melanoma".into(),
            )
        })?;

    if filters
        .source
        .as_deref()
        .map(str::trim)
        .is_some_and(|source| source.eq_ignore_ascii_case("mesh"))
    {
        return Ok(None);
    }

    let discover = match crate::entities::discover::resolve_query(
        query,
        crate::entities::discover::DiscoverMode::AliasFallback,
    )
    .await
    {
        Ok(result) => result,
        Err(err) => {
            warn!("Disease search discover fallback unavailable: {err}");
            return Ok(None);
        }
    };

    let candidates = rank_disease_fallback_candidates(query, &discover.concepts);
    if candidates.is_empty() {
        return Ok(None);
    }

    let prefer_doid = filters
        .source
        .as_deref()
        .map(str::trim)
        .is_some_and(|source| source.eq_ignore_ascii_case("doid"));
    let client = MyDiseaseClient::new()?;
    collect_fallback_search_page(query, limit, offset, candidates, |source_id| {
        let client = client.clone();
        async move { resolve_fallback_row(&client, prefer_doid, &source_id).await }
    })
    .await
}

pub(super) async fn resolve_disease_hit_via_discover_fallback(
    client: &MyDiseaseClient,
    name_or_id: &str,
) -> Result<Option<MyDiseaseHit>, BioMcpError> {
    for query in resolver_queries(name_or_id) {
        let filters = DiseaseSearchFilters {
            query: Some(query),
            ..Default::default()
        };
        let Some(page) = fallback_search_page(&filters, 1, 0).await? else {
            continue;
        };
        let Some(row) = page.results.into_iter().next() else {
            continue;
        };

        match client.get(&row.id).await {
            Ok(hit) => return Ok(Some(hit)),
            Err(err) => {
                warn!("Disease get fallback canonical fetch failed: {err}");
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests;
