//! Disease ID normalization, candidate scoring, and direct name resolution.

use super::fallback::resolve_disease_hit_via_discover_fallback;
use super::*;

const EXACT_RESOLUTION_QUERY_SIZE: usize = 50;
const MAX_EXACT_SYNONYMS: usize = 20;
const MAX_PROVIDER_TERM_BYTES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExactDiseaseTerms {
    pub requested: String,
    pub canonical_id: Option<String>,
    pub canonical_name: Option<String>,
    pub synonyms: Vec<String>,
}

fn source_resolution_error() -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: "MyDisease.info".to_string(),
        reason: "exact disease identity resolution failed".to_string(),
        suggestion: "Retry the diagnostic search.".to_string(),
    }
}

fn valid_provider_term(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > MAX_PROVIDER_TERM_BYTES
        || value.chars().any(char::is_control)
    {
        return None;
    }
    Some(value.to_string())
}

fn normalized_term(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().any(char::is_control) {
        return None;
    }
    let mut normalized = String::new();
    let mut pending_space = false;
    for ch in value.chars() {
        if ch.is_whitespace() {
            pending_space = !normalized.is_empty();
        } else {
            if pending_space {
                normalized.push(' ');
            }
            normalized.push(ch.to_ascii_lowercase());
            pending_space = false;
        }
    }
    (!normalized.is_empty()).then_some(normalized)
}

fn normalized_provider_term(value: &str) -> Option<String> {
    normalized_term(&valid_provider_term(value)?)
}

fn synonym_values(value: &serde_json::Value) -> Option<Vec<String>> {
    match value {
        serde_json::Value::String(value) => valid_provider_term(value).map(|value| vec![value]),
        serde_json::Value::Array(values) => values
            .iter()
            .map(|value| value.as_str().and_then(valid_provider_term))
            .collect(),
        _ => None,
    }
}

fn provider_terms(hit: &MyDiseaseHit) -> (Vec<String>, Vec<String>, bool) {
    let mut names = Vec::new();
    let mut synonyms = Vec::new();
    let mut malformed_synonyms = false;
    for object in [hit.disease_ontology.as_ref(), hit.mondo.as_ref()]
        .into_iter()
        .flatten()
    {
        if let Some(value) = object.get("name")
            && let Some(value) = value.as_str().and_then(valid_provider_term)
        {
            names.push(value);
        }
    }
    for object in [hit.mondo.as_ref(), hit.disease_ontology.as_ref()]
        .into_iter()
        .flatten()
    {
        if let Some(value) = object.get("synonym").or_else(|| object.get("synonyms")) {
            if let Some(values) =
                synonym_values(value).or_else(|| value.get("exact").and_then(synonym_values))
            {
                synonyms.extend(values);
            } else if !value.is_null() {
                malformed_synonyms = true;
            }
        }
    }
    (names, synonyms, malformed_synonyms)
}

fn canonical_name(hit: &MyDiseaseHit) -> Option<String> {
    let (names, _, _) = provider_terms(hit);
    names
        .into_iter()
        .find_map(|value| valid_provider_term(&value))
}

fn valid_canonical_id(value: &str) -> Option<String> {
    let id = normalize_disease_id(value)?;
    let (_, rest) = id.split_once(':')?;
    rest.chars().all(|ch| ch.is_ascii_digit()).then_some(id)
}

fn exact_hit_matches(query: &str, hit: &MyDiseaseHit) -> bool {
    let query = match normalized_term(query) {
        Some(query) => query,
        None => return false,
    };
    let (names, synonyms, _) = provider_terms(hit);
    names
        .into_iter()
        .chain(synonyms)
        .any(|term| normalized_provider_term(&term).is_some_and(|term| term == query))
}

fn detail_terms(
    requested: &str,
    expected_id: &str,
    hit: MyDiseaseHit,
) -> Result<ExactDiseaseTerms, BioMcpError> {
    let id = valid_canonical_id(&hit.id).ok_or_else(source_resolution_error)?;
    let expected_id = valid_canonical_id(expected_id).ok_or_else(source_resolution_error)?;
    if id != expected_id {
        return Err(source_resolution_error());
    }
    let canonical_name = canonical_name(&hit).ok_or_else(source_resolution_error)?;
    let (_, synonyms, malformed_synonyms) = provider_terms(&hit);
    if malformed_synonyms {
        return Err(source_resolution_error());
    }

    let mut seen = std::collections::HashSet::new();
    let requested_key = normalized_term(requested).ok_or_else(source_resolution_error)?;
    let canonical_key =
        normalized_provider_term(&canonical_name).ok_or_else(source_resolution_error)?;
    let mut valid_synonyms = Vec::new();
    for synonym in synonyms {
        let key = normalized_provider_term(&synonym).ok_or_else(source_resolution_error)?;
        if key == requested_key || key == canonical_key || !seen.insert(key) {
            continue;
        }
        valid_synonyms.push(synonym);
        if valid_synonyms.len() == MAX_EXACT_SYNONYMS {
            break;
        }
    }
    Ok(ExactDiseaseTerms {
        requested: requested.trim().to_string(),
        canonical_id: Some(id),
        canonical_name: Some(canonical_name),
        synonyms: valid_synonyms,
    })
}

pub(crate) async fn resolve_exact_disease_terms(
    client: &MyDiseaseClient,
    disease: &str,
) -> Result<ExactDiseaseTerms, BioMcpError> {
    let requested = disease.trim();
    if requested.len() > 512 || normalized_term(requested).is_none() {
        return Err(source_resolution_error());
    }

    if let Some(id) = normalize_disease_id(requested) {
        let detail = client
            .get(&id)
            .await
            .map_err(|_| source_resolution_error())?;
        return detail_terms(requested, &id, detail);
    }

    let response = client
        .query(
            requested,
            EXACT_RESOLUTION_QUERY_SIZE,
            0,
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(|_| source_resolution_error())?;
    if response.total > response.hits.len() {
        return Err(source_resolution_error());
    }

    let mut exact_hits = std::collections::HashMap::new();
    for hit in response.hits {
        if !exact_hit_matches(requested, &hit) {
            continue;
        }
        let Some(id) = valid_canonical_id(&hit.id) else {
            continue;
        };
        exact_hits.entry(id).or_insert(hit);
    }
    if exact_hits.len() != 1 {
        return Ok(ExactDiseaseTerms {
            requested: requested.to_string(),
            canonical_id: None,
            canonical_name: None,
            synonyms: Vec::new(),
        });
    }

    let id = exact_hits
        .keys()
        .next()
        .cloned()
        .ok_or_else(source_resolution_error)?;
    let detail = client
        .get(&id)
        .await
        .map_err(|_| source_resolution_error())?;
    detail_terms(requested, &id, detail)
}

pub(super) fn normalize_disease_id(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    if v.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    let (prefix, rest) = v.split_once(':')?;
    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }
    if prefix.eq_ignore_ascii_case("MONDO") {
        return Some(format!("MONDO:{rest}"));
    }
    if prefix.eq_ignore_ascii_case("DOID") {
        return Some(format!("DOID:{rest}"));
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DiseaseLookupInput {
    CanonicalOntologyId(String),
    CrosswalkId(DiseaseXrefKind, String),
    FreeText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum DiseaseXrefKind {
    Mesh,
    Omim,
    Icd10Cm,
}

impl DiseaseXrefKind {
    pub(super) fn source_key(self) -> &'static str {
        match self {
            Self::Mesh => "mesh",
            Self::Omim => "omim",
            Self::Icd10Cm => "icd10cm",
        }
    }

    pub(super) fn display_name(self) -> &'static str {
        match self {
            Self::Mesh => "MESH",
            Self::Omim => "OMIM",
            Self::Icd10Cm => "ICD10CM",
        }
    }

    pub(super) fn resolved_via_label(self) -> String {
        format!("{} crosswalk", self.display_name())
    }
}

pub(super) fn parse_disease_lookup_input(value: &str) -> DiseaseLookupInput {
    if let Some(id) = normalize_disease_id(value) {
        return DiseaseLookupInput::CanonicalOntologyId(id);
    }

    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return DiseaseLookupInput::FreeText;
    }

    let Some((prefix, raw_value)) = trimmed.split_once(':') else {
        return DiseaseLookupInput::FreeText;
    };
    let raw_value = raw_value.trim();
    if raw_value.is_empty() {
        return DiseaseLookupInput::FreeText;
    }

    let kind = if prefix.eq_ignore_ascii_case("MESH") {
        Some(DiseaseXrefKind::Mesh)
    } else if prefix.eq_ignore_ascii_case("OMIM") {
        Some(DiseaseXrefKind::Omim)
    } else if prefix.eq_ignore_ascii_case("ICD10CM") {
        Some(DiseaseXrefKind::Icd10Cm)
    } else {
        None
    };

    if let Some(kind) = kind {
        DiseaseLookupInput::CrosswalkId(kind, raw_value.to_string())
    } else {
        DiseaseLookupInput::FreeText
    }
}

pub(super) fn preferred_crosswalk_hit(hits: Vec<MyDiseaseHit>) -> Option<MyDiseaseHit> {
    hits.into_iter().min_by(|left, right| {
        let rank = |id: &str| {
            if id.starts_with("MONDO:") {
                0u8
            } else if id.starts_with("DOID:") {
                1u8
            } else {
                2u8
            }
        };
        rank(&left.id)
            .cmp(&rank(&right.id))
            .then_with(|| left.id.cmp(&right.id))
    })
}

const MIN_DIRECT_DISEASE_MATCH_SCORE: i32 = 120;

pub(super) fn normalize_disease_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch.is_whitespace() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(' ');
        }
    }
    let out = out
        .replace("carcinoma", "cancer")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    out.trim().to_string()
}

pub(super) fn disease_exact_rank(name: &str, query: &str) -> u8 {
    let name = name.trim().to_ascii_lowercase();
    let query = query.trim().to_ascii_lowercase();
    if name == query {
        3
    } else if name.starts_with(&query) {
        2
    } else if name.contains(&query) {
        1
    } else {
        0
    }
}

fn has_subtype_marker(value: &str) -> bool {
    let normalized = normalize_disease_text(value);
    if normalized.is_empty() {
        return false;
    }

    let markers = [
        "sporadic",
        "hereditary",
        "familial",
        "metastatic",
        "recurrent",
        "adenocarcinoma",
        "squamous",
        "triple negative",
        "triple positive",
        "er positive",
        "er negative",
        "pr positive",
        "pr negative",
        "her2 positive",
        "her2 negative",
        "in situ",
    ];
    if markers.iter().any(|marker| normalized.contains(marker)) {
        return true;
    }

    let words = normalized.split_whitespace().collect::<Vec<_>>();
    for pair in words.windows(2) {
        if pair[0] == "type" && pair[1].chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }
    false
}

pub(super) fn disease_candidate_score(query: &str, candidate_label: &str) -> i32 {
    let query_trimmed = query.trim();
    let candidate_trimmed = candidate_label.trim();
    if query_trimmed.is_empty() || candidate_trimmed.is_empty() {
        return i32::MIN / 2;
    }

    let query_norm = normalize_disease_text(query_trimmed);
    let candidate_norm = normalize_disease_text(candidate_trimmed);
    let mut score = 0;

    if candidate_trimmed.eq_ignore_ascii_case(query_trimmed) {
        score += 200;
    }
    if candidate_norm == query_norm {
        score += 120;
    } else if candidate_norm.contains(&query_norm) {
        score += 40;
    } else if query_norm.contains(&candidate_norm) {
        score += 20;
    }

    let query_has_subtype = has_subtype_marker(query_trimmed);
    let candidate_has_subtype = has_subtype_marker(candidate_trimmed);
    if candidate_has_subtype && !query_has_subtype {
        score -= 60;
    }
    if !candidate_has_subtype && query_has_subtype {
        score -= 20;
    }

    score
}

fn collect_json_strings(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(v) => {
            let v = v.trim();
            if !v.is_empty() {
                out.push(v.to_string());
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_json_strings(value, out);
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values() {
                collect_json_strings(value, out);
            }
        }
        _ => {}
    }
}

fn disease_candidate_labels(hit: &MyDiseaseHit) -> Vec<String> {
    let mut labels = vec![transform::disease::name_from_mydisease_hit(hit)];
    if let Some(value) = hit.mondo.as_ref().and_then(|v| v.get("synonym")) {
        collect_json_strings(value, &mut labels);
    }
    if let Some(value) = hit
        .disease_ontology
        .as_ref()
        .and_then(|v| v.get("synonyms"))
    {
        collect_json_strings(value, &mut labels);
    }

    let mut deduped = Vec::new();
    for label in labels {
        if deduped
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&label))
        {
            continue;
        }
        deduped.push(label);
    }
    deduped
}

fn best_disease_candidate_score(query: &str, hit: &MyDiseaseHit) -> i32 {
    disease_candidate_labels(hit)
        .into_iter()
        .map(|label| disease_candidate_score(query, &label))
        .max()
        .unwrap_or(i32::MIN / 2)
}

fn best_disease_candidate_score_for_queries(queries: &[String], hit: &MyDiseaseHit) -> i32 {
    queries
        .iter()
        .map(|query| best_disease_candidate_score(query, hit))
        .max()
        .unwrap_or(i32::MIN / 2)
}

fn scored_best_candidate_for_queries(
    queries: &[String],
    hits: Vec<MyDiseaseHit>,
) -> Option<MyDiseaseHit> {
    if queries.is_empty() {
        return None;
    }

    let mut ranked: Vec<(i32, u8, usize, String, MyDiseaseHit)> = hits
        .into_iter()
        .map(|hit| {
            let primary_name = transform::disease::name_from_mydisease_hit(&hit);
            let best_score = best_disease_candidate_score_for_queries(queries, &hit);
            let best_exact_rank = queries
                .iter()
                .map(|query| disease_exact_rank(&primary_name, query))
                .max()
                .unwrap_or(0);
            let normalized_len = normalize_disease_text(&primary_name).len();
            (
                best_score,
                best_exact_rank,
                normalized_len,
                hit.id.clone(),
                hit,
            )
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| a.3.cmp(&b.3))
    });
    ranked.into_iter().next().map(|(_, _, _, _, hit)| hit)
}

pub(super) fn resolver_queries(name_or_id: &str) -> Vec<String> {
    let query = name_or_id.trim();
    if query.is_empty() {
        return Vec::new();
    }

    let mut queries = vec![query.to_string()];
    let mut push_query = |candidate: String| {
        if queries
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&candidate))
        {
            return;
        }
        queries.push(candidate);
    };

    let lower = query.to_ascii_lowercase();
    if lower.contains("cancer") {
        push_query(lower.replace("cancer", "carcinoma"));
    }
    for (from, to) in [
        ("myeloid leukemia", "myelogenous leukemia"),
        ("myeloid leukaemia", "myelogenous leukaemia"),
        ("myelogenous leukemia", "myeloid leukemia"),
        ("myelogenous leukaemia", "myeloid leukaemia"),
        ("hodgkin lymphoma", "hodgkins lymphoma"),
        ("hodgkins lymphoma", "hodgkin lymphoma"),
    ] {
        if lower.contains(from) {
            push_query(lower.replace(from, to));
        }
    }
    if lower == "chronic myeloid leukemia" {
        push_query("chronic myelogenous leukemia, bcr-abl1 positive".to_string());
    }
    if matches!(lower.as_str(), "hodgkin lymphoma" | "hodgkins lymphoma") {
        push_query("hodgkin disease".to_string());
    }
    if matches!(
        lower.as_str(),
        "arnold chiari syndrome" | "arnold-chiari syndrome"
    ) {
        push_query("arnold-chiari malformation".to_string());
    }
    queries
}

struct DiseaseSearchCandidate {
    hit: MyDiseaseHit,
    first_seen_query_idx: usize,
    first_seen_upstream_idx: usize,
}

pub(super) fn rerank_disease_search_hits(
    query: &str,
    query_hits: Vec<(usize, Vec<MyDiseaseHit>)>,
) -> Vec<MyDiseaseHit> {
    let mut deduped: HashMap<String, DiseaseSearchCandidate> = HashMap::new();
    for (query_idx, hits) in query_hits {
        for (upstream_idx, hit) in hits.into_iter().enumerate() {
            deduped
                .entry(hit.id.clone())
                .or_insert(DiseaseSearchCandidate {
                    hit,
                    first_seen_query_idx: query_idx,
                    first_seen_upstream_idx: upstream_idx,
                });
        }
    }

    let mut ranked = deduped
        .into_values()
        .map(|candidate| {
            let display_name = transform::disease::name_from_mydisease_hit(&candidate.hit);
            (
                best_disease_candidate_score(query, &candidate.hit),
                disease_exact_rank(&display_name, query),
                candidate.first_seen_query_idx,
                candidate.first_seen_upstream_idx,
                candidate.hit.id.clone(),
                candidate.hit,
            )
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| a.3.cmp(&b.3))
            .then_with(|| a.4.cmp(&b.4))
    });
    ranked.into_iter().map(|(_, _, _, _, _, hit)| hit).collect()
}

pub(crate) async fn resolve_disease_hit_by_name(
    client: &MyDiseaseClient,
    name_or_id: &str,
) -> Result<MyDiseaseHit, BioMcpError> {
    if let Some(best) = resolve_disease_hit_by_name_direct(client, name_or_id).await? {
        return Ok(best);
    }
    if let Some(best) = resolve_disease_hit_via_discover_fallback(client, name_or_id).await? {
        return Ok(best);
    }

    Err(BioMcpError::NotFound {
        entity: "disease".into(),
        id: name_or_id.into(),
        suggestion: format!("Try searching: biomcp search disease -q \"{name_or_id}\""),
    })
}

pub(super) async fn resolve_disease_hit_by_name_direct(
    client: &MyDiseaseClient,
    name_or_id: &str,
) -> Result<Option<MyDiseaseHit>, BioMcpError> {
    let queries = resolver_queries(name_or_id);
    if queries.is_empty() {
        return Ok(None);
    }

    let mut candidates: HashMap<String, MyDiseaseHit> = HashMap::new();
    for query in &queries {
        let resp = client.query(query, 15, 0, None, None, None, None).await?;
        for hit in resp.hits {
            candidates.entry(hit.id.clone()).or_insert(hit);
        }
    }

    Ok(
        scored_best_candidate_for_queries(&queries, candidates.into_values().collect()).filter(
            |hit| {
                best_disease_candidate_score_for_queries(&queries, hit)
                    >= MIN_DIRECT_DISEASE_MATCH_SCORE
            },
        ),
    )
}

#[cfg(test)]
mod tests;
