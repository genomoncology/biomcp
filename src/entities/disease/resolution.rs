//! Disease ID normalization, candidate scoring, and direct name resolution.

use super::fallback::resolve_disease_hit_via_discover_fallback;
use super::*;

const EXACT_RESOLUTION_QUERY_SIZE: usize = 50;
const MAX_EXACT_SYNONYMS: usize = 20;
const MAX_PROVIDER_TERM_BYTES: usize = 256;
/// Two-letter abbreviations never name one disease through `get disease`:
/// `MM` names Miyoshi muscular dystrophy in the source while clinicians
/// mean multiple myeloma, and `HD` names Huntington's disease while
/// oncology means Hodgkin disease. A single source holder is not evidence
/// for a token this short, so resolution refuses (ticket 2017).
const SHORT_ABBREVIATION_MAX_LEN: usize = 2;

/// A clinical reading an abbreviation names that no indexed source
/// record holds. `relation` states how strongly oncology holds the
/// reading (`usually means` for the default reading, `also names` when
/// the holders already offer one clinical reading and the pointer adds
/// another).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClinicalAbbreviationReading {
    pub(crate) token: &'static str,
    pub(crate) relation: &'static str,
    pub(crate) label: &'static str,
    pub(crate) ontology_id: &'static str,
}

/// Abbreviations whose clinical meaning the source cannot see, with the
/// sources checked and measured for each entry (ticket 2040). The
/// refusal stands on the source holders alone; this table only adds a
/// pointer line naming a reading no holder carries.
///
/// - `MM` → multiple myeloma (MONDO:0009693): MyDisease holds `MM` on
///   Miyoshi muscular dystrophy (MONDO:0009685) alone. Checked live
///   2026-10-08 and again 2026-10-11: `mondo.synonym.exact:"MM"` and
///   `disease_ontology.synonyms.exact:"MM"` return no myeloma record,
///   the MONDO:0009693 synonym list carries no `MM`, and the NCI
///   Thesaurus synonym list for Multiple Myeloma (NCIT:C3242, via EBI
///   OLS4) carries no `MM`. Measured on the ClinicalTrials.gov registry
///   2026-10-11: of the first 50 trials matching `query.cond=MM`, 30
///   list a myeloma condition, so myeloma is the reading the registry
///   corpus itself carries.
/// - `MF` → myelofibrosis (MONDO:0009692, the record `get disease
///   "myelofibrosis"` resolves to through its Disease Ontology name):
///   MyDisease holds `MF` on mycosis fungoides (MONDO:0009691) and
///   myotonia fluctuans (MONDO:0020481) alone. Checked live
///   2026-10-11: no myelofibrosis record carries `MF` in
///   `mondo.synonym.exact` or `disease_ontology.synonyms.exact`, the
///   Disease Ontology entry (DOID:4971) lists no `MF`, and the NCI
///   Thesaurus synonym lists for Myelofibrosis (NCIT:C3248) and Primary
///   Myelofibrosis (NCIT:C2862, via EBI OLS4) list no `MF`. Measured on
///   the ClinicalTrials.gov registry 2026-10-11: of the first 50 trials
///   matching `query.cond=MF`, 28 list a myelofibrosis condition
///   against 6 for mycosis fungoides, so myelofibrosis is the reading
///   the trial corpus carries most. Curated abbreviation preferences
///   stay deferred (ticket 2032).
const CLINICAL_ABBREVIATION_READINGS: &[ClinicalAbbreviationReading] = &[
    ClinicalAbbreviationReading {
        token: "MM",
        relation: "usually means",
        label: "multiple myeloma",
        ontology_id: "MONDO:0009693",
    },
    ClinicalAbbreviationReading {
        token: "MF",
        relation: "also names",
        label: "myelofibrosis",
        ontology_id: "MONDO:0009692",
    },
];

fn clinical_reading(requested: &str) -> Option<ClinicalAbbreviationReading> {
    let requested = requested.trim();
    CLINICAL_ABBREVIATION_READINGS
        .iter()
        .copied()
        .find(|reading| reading.token.eq_ignore_ascii_case(requested))
}

/// The pointer phrase both the `get disease` refusal and the default
/// trial source's keyword-search note use (ticket 2040), so one reading
/// keeps one wording across surfaces.
pub(crate) fn clinical_reading_phrase(requested: &str) -> Option<String> {
    clinical_reading(requested).map(|reading| {
        format!(
            "Clinical reading: '{}' {} {} ({})",
            requested.trim(), reading.relation, reading.label, reading.ontology_id
        )
    })
}

fn clinical_reading_line(requested: &str) -> Option<String> {
    clinical_reading(requested).map(|reading| {
        let requested = requested.trim();
        format!(
            "Clinical reading: '{requested}' {} {} ({}); \
try `biomcp get disease "{}"`.",
            reading.relation, reading.label, reading.ontology_id, reading.label
        )
    })
}

fn is_short_abbreviation_token(token: &str) -> bool {
    let token = token.trim();
    !token.is_empty()
        && token.len() <= SHORT_ABBREVIATION_MAX_LEN
        && token.chars().all(|ch| ch.is_ascii_alphabetic())
}

/// A real label from the provider, as opposed to the ID fallback that
/// `name_from_mydisease_hit` returns for records without a name.
fn hit_label(hit: &MyDiseaseHit) -> Option<String> {
    let name = transform::disease::name_from_mydisease_hit(hit);
    (!name.eq_ignore_ascii_case(hit.id.trim())).then_some(name)
}

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
    exact_term_hold(query, hit).is_some()
}

/// How a record holds the requested term exactly: as its ontology name or
/// as an exact synonym. Ticket 2017 counts the two differently for full
/// words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExactTermHold {
    Name,
    Synonym,
}

fn exact_term_hold(query: &str, hit: &MyDiseaseHit) -> Option<ExactTermHold> {
    let query = normalized_term(query)?;
    let (names, synonyms, _) = provider_terms(hit);
    if names
        .into_iter()
        .any(|term| normalized_provider_term(&term).is_some_and(|term| term == query))
    {
        return Some(ExactTermHold::Name);
    }
    synonyms
        .into_iter()
        .any(|term| normalized_provider_term(&term).is_some_and(|term| term == query))
        .then_some(ExactTermHold::Synonym)
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

pub(super) fn collect_json_strings(value: &serde_json::Value, out: &mut Vec<String>) {
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

    let mut ranked: Vec<(i32, u8, u8, usize, String, MyDiseaseHit)> = hits
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
            // Unlabelled records answer with their ID as the name, which is
            // shorter than any real label and used to win the length
            // tie-break (ticket 2017); a real label outranks the fallback.
            let unlabelled = hit_label(&hit).is_none() as u8;
            (
                best_score,
                best_exact_rank,
                unlabelled,
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
            .then_with(|| a.4.cmp(&b.4))
    });
    ranked.into_iter().next().map(|(_, _, _, _, _, hit)| hit)
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
                // Ties among equal holders resolve to the labelled record
                // first, then the shorter canonical name: `MDS` ranks
                // myelodysplastic syndrome above Miller-Dieker lissencephaly,
                // and unlabelled abbreviation holders must not surface above
                // labelled ones (ticket 2017).
                hit_label(&candidate.hit).is_none() as u8,
                normalize_disease_text(&display_name).len(),
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
            .then_with(|| a.5.cmp(&b.5))
            .then_with(|| a.6.cmp(&b.6))
    });
    ranked
        .into_iter()
        .map(|(_, _, _, _, _, _, _, hit)| hit)
        .collect()
}

/// MONDO:0005583 is `non-human animal disease`; every veterinary MONDO
/// record descends from it, including the venom-database `myeloma` that
/// once made `get disease myeloma` refuse. BioMCP answers clinicians, so a
/// non-human record never counts toward disease-name ambiguity (ticket
/// 2017).
const NON_HUMAN_ANIMAL_DISEASE_ID: &str = "MONDO:0005583";

fn is_non_human_record(hit: &MyDiseaseHit) -> bool {
    hit.mondo
        .as_ref()
        .and_then(|value| value.get("ancestors"))
        .and_then(|value| value.as_array())
        .is_some_and(|values| {
            values
                .iter()
                .any(|value| value.as_str() == Some(NON_HUMAN_ANIMAL_DISEASE_ID))
        })
}

/// A token shaped like an abbreviation — one word, all ASCII capitals
/// (`MF`, `CAD`, `NSCLC`) — is ambiguous when several diseases hold it as
/// an exact name or synonym. Any other shape is a full word, and a full
/// word is ambiguous only when several records carry it as their exact
/// NAME: a record that merely shares the word as a synonym does not stop
/// `get disease myeloma` from reaching multiple myeloma (ticket 2017).
fn is_abbreviation_shaped_token(token: &str) -> bool {
    let token = token.trim();
    !token.is_empty()
        && !token.chars().any(char::is_whitespace)
        && token.chars().all(|ch| ch.is_ascii_uppercase())
}

/// Diseases in a candidate set that hold `query` as an exact holder,
/// ordered by ID so refusal messages are stable. Non-human records never
/// count, and for a full word only exact-NAME holders count.
fn exact_token_holder_ids<'a>(
    query: &str,
    hits: impl IntoIterator<Item = &'a MyDiseaseHit>,
) -> Vec<String> {
    let abbreviation = is_abbreviation_shaped_token(query);
    let mut holder_ids: Vec<String> = hits
        .into_iter()
        .filter(|hit| !is_non_human_record(hit))
        .filter(|hit| match exact_term_hold(query, hit) {
            Some(ExactTermHold::Name) => true,
            Some(ExactTermHold::Synonym) => abbreviation,
            None => false,
        })
        .map(|hit| hit.id.clone())
        .collect();
    holder_ids.sort();
    holder_ids
}

/// A display label for a refusal candidate. MONDO records with no `name`
/// in the search response still carry the ontology `label`, so the
/// candidate list names the disease instead of printing a bare ID
/// (ticket 2017: `CAD` refused listing `MONDO:0018922` with no label).
fn holder_display_label(hit: &MyDiseaseHit) -> Option<String> {
    hit_label(hit).or_else(|| {
        hit.mondo
            .as_ref()
            .and_then(|value| value.get("label"))
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
    })
}

fn holder_display_line(hit: &MyDiseaseHit) -> String {
    match holder_display_label(hit) {
        Some(label) => format!("{label} ({})", hit.id),
        None => format!("{} (no label in the search response)", hit.id),
    }
}

fn abbreviation_candidate_lines(holders: &[MyDiseaseHit]) -> String {
    holders
        .iter()
        .map(|hit| format!("- {}\n", holder_display_line(hit)))
        .collect()
}

/// Why the requested token cannot name one disease: several holders, or
/// a token too short for even one holder to be evidence.
fn ambiguity_reason(requested: &str, holders: &[MyDiseaseHit], short_token: bool) -> String {
    if short_token {
        format!(
            "the source holds it on {} disease{}, but a token this short cannot name one disease reliably",
            holders.len(),
            if holders.len() == 1 { "" } else { "s" },
        )
    } else if is_abbreviation_shaped_token(requested) {
        format!(
            "{} diseases hold it as an exact name or synonym",
            holders.len()
        )
    } else {
        format!(
            "{} diseases carry it as their exact name",
            holders.len()
        )
    }
}

/// What the refusal knows about an ambiguous abbreviation, exposed so
/// the default ClinicalTrials.gov trial source can say plainly what its
/// keyword search ran on instead of leaving the user to guess (ticket
/// 2040).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AbbreviationAmbiguity {
    pub(crate) requested: String,
    /// Why the token cannot name one disease (the refusal's reason text).
    pub(crate) reason: String,
    /// The named holders, one `label (ontology id)` per entry.
    pub(crate) holders: Vec<String>,
    /// A clinical reading no source record holds, from the pointer table.
    pub(crate) clinical_reading: Option<ClinicalAbbreviationReading>,
}

/// The refusal the default trial source would get from `get disease`,
/// as data instead of an error: `Some` when the token is an
/// abbreviation-shaped condition the disease resolver refuses (several
/// holders, or a holder too short to count), `None` for anything else.
/// Only abbreviation-shaped tokens reach the resolver, so ordinary
/// condition words add no grounding request (ticket 2040).
pub(crate) async fn resolve_abbreviated_disease_ambiguity(
    client: &MyDiseaseClient,
    requested: &str,
) -> Result<Option<AbbreviationAmbiguity>, BioMcpError> {
    let requested = requested.trim();
    if !is_abbreviation_shaped_token(requested) {
        return Ok(None);
    }
    match resolve_disease_hit_by_name_direct(client, requested).await? {
        DirectNameResolution::AmbiguousAbbreviation {
            requested,
            holders,
        } => {
            // Several holders explain the refusal on their own; the
            // short-token reason is for the single-holder case like `MM`.
            let short_token = holders.len() == 1 && is_short_abbreviation_token(&requested);
            Ok(Some(AbbreviationAmbiguity {
                reason: ambiguity_reason(&requested, &holders, short_token),
                holders: holders.iter().map(holder_display_line).collect(),
                clinical_reading: clinical_reading(&requested),
                requested,
            }))
        }
        DirectNameResolution::Resolved(_) | DirectNameResolution::Unresolved => Ok(None),
    }
}

fn ambiguous_abbreviation_error(
    requested: &str,
    holders: &[MyDiseaseHit],
    short_token: bool,
) -> BioMcpError {
    let abbreviation = is_abbreviation_shaped_token(requested);
    let subject = if abbreviation { "abbreviation" } else { "name" };
    let reason = ambiguity_reason(requested, holders, short_token);
    let reading = clinical_reading_line(requested)
        .map(|line| format!("{line}\n"))
        .unwrap_or_default();
    BioMcpError::InvalidArgument(format!(
        "Ambiguous disease {subject} '{requested}': {reason}; \
BioMCP refuses rather than return one disease's definition with another's genes.\n\
Candidates:\n{}{reading}\
Retry `biomcp get disease` with one candidate's ontology ID or full name, or run `biomcp search disease -q \"{requested}\"` to see every match.",
        abbreviation_candidate_lines(holders),
    ))
}

pub(crate) async fn resolve_disease_hit_by_name(
    client: &MyDiseaseClient,
    name_or_id: &str,
) -> Result<MyDiseaseHit, BioMcpError> {
    match resolve_disease_hit_by_name_direct(client, name_or_id).await? {
        DirectNameResolution::AmbiguousAbbreviation { requested, holders } => {
            // Several holders explain the refusal on their own; the
            // short-token reason is for the single-holder case like `MM`.
            let short_token = holders.len() == 1 && is_short_abbreviation_token(&requested);
            return Err(ambiguous_abbreviation_error(
                &requested,
                &holders,
                short_token,
            ));
        }
        DirectNameResolution::Resolved(best) => return Ok(best),
        DirectNameResolution::Unresolved => {}
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

pub(super) enum DirectNameResolution {
    Resolved(MyDiseaseHit),
    Unresolved,
    /// The requested token is held by several diseases, or by one disease
    /// while being too short to name any disease; either way `get disease`
    /// must refuse instead of picking silently (ticket 2017).
    AmbiguousAbbreviation {
        requested: String,
        holders: Vec<MyDiseaseHit>,
    },
}

pub(super) async fn resolve_disease_hit_by_name_direct(
    client: &MyDiseaseClient,
    name_or_id: &str,
) -> Result<DirectNameResolution, BioMcpError> {
    let queries = resolver_queries(name_or_id);
    if queries.is_empty() {
        return Ok(DirectNameResolution::Unresolved);
    }

    let mut candidates: HashMap<String, MyDiseaseHit> = HashMap::new();
    for query in &queries {
        let resp = client.query(query, 15, 0, None, None, None, None).await?;
        for hit in resp.hits {
            candidates.entry(hit.id.clone()).or_insert(hit);
        }
    }

    let requested = name_or_id.trim();
    let holder_ids = exact_token_holder_ids(requested, candidates.values());
    if holder_ids.len() > 1 || (holder_ids.len() == 1 && is_short_abbreviation_token(requested)) {
        let holders = holder_ids
            .iter()
            .filter_map(|id| candidates.remove(id))
            .collect();
        return Ok(DirectNameResolution::AmbiguousAbbreviation {
            requested: requested.to_string(),
            holders,
        });
    }

    Ok(
        match scored_best_candidate_for_queries(&queries, candidates.into_values().collect())
            .filter(|hit| {
                best_disease_candidate_score_for_queries(&queries, hit)
                    >= MIN_DIRECT_DISEASE_MATCH_SCORE
            }) {
            Some(hit) => DirectNameResolution::Resolved(hit),
            None => DirectNameResolution::Unresolved,
        },
    )
}

#[cfg(test)]
mod tests;
