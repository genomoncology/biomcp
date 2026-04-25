use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Duration;

use regex::Regex;
use serde::Serialize;

use crate::error::BioMcpError;
use crate::sources::hpo::HpoClient;
use crate::sources::medlineplus::MedlinePlusTopic;
use crate::sources::ols4::OlsDoc;
use crate::sources::umls::{UmlsConcept, UmlsXref};

const OLS4_TIMEOUT: Duration = Duration::from_millis(4000);
const UMLS_TIMEOUT: Duration = Duration::from_millis(2500);
const MEDLINEPLUS_TIMEOUT: Duration = Duration::from_millis(800);
const GENERAL_DISCOVER_MAX_SURVIVORS: usize = 5;
const DISCOVER_GENERAL_STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "in", "into", "of", "on", "or", "the",
    "to", "what", "which", "with",
];
const DISCOVER_GENERAL_RELATIONAL_TOKENS: &[&str] = &[
    "associated",
    "classes",
    "drug",
    "drugs",
    "interact",
    "interaction",
    "located",
    "regulated",
    "genes",
];
const DISCOVER_GENERAL_RELATIONAL_PHRASES: &[&str] = &[
    "associated with",
    "interact with",
    "interacts with",
    "regulated by",
    "regulated genes",
    "located in",
    "in the",
];

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DiscoverResult {
    pub query: String,
    pub normalized_query: String,
    pub concepts: Vec<DiscoverConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plain_language: Option<PlainLanguageTopic>,
    #[serde(skip)]
    pub next_commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    pub ambiguous: bool,
    pub intent: DiscoverIntent,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DiscoverConcept {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_id: Option<String>,
    pub primary_type: DiscoverType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub xrefs: Vec<ConceptXref>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<ConceptSource>,
    pub match_tier: MatchTier,
    pub confidence: DiscoverConfidence,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConceptXref {
    pub source: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConceptSource {
    pub source: String,
    pub id: String,
    pub label: String,
    pub source_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PlainLanguageTopic {
    pub title: String,
    pub url: String,
    pub summary_excerpt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum DiscoverType {
    Gene,
    Drug,
    Disease,
    Symptom,
    Pathway,
    Variant,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum DiscoverIntent {
    TrialSearch,
    DrugSafety,
    TreatmentSearch,
    SymptomSearch,
    GeneDiseaseOrientation,
    GeneFunction,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum MatchTier {
    Exact,
    Prefix,
    Contains,
    Weak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum DiscoverConfidence {
    CanonicalId,
    UmlsOnly,
    LabelOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiscoverMode {
    Command,
    AliasFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GeneralDiscoverScore {
    overlap_count: usize,
    protected_exact: bool,
    strong_survivor: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AliasCanonicalMatch {
    pub requested_entity: DiscoverType,
    pub query: String,
    pub canonical: String,
    pub canonical_id: String,
    pub confidence: DiscoverConfidence,
    pub match_tier: MatchTier,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    pub next_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AliasCandidateSummary {
    pub label: String,
    pub primary_type: DiscoverType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_id: Option<String>,
    pub confidence: DiscoverConfidence,
    pub match_tier: MatchTier,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AliasAmbiguity {
    pub requested_entity: DiscoverType,
    pub query: String,
    pub candidates: Vec<AliasCandidateSummary>,
    pub next_commands: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum AliasFallbackDecision {
    Canonical(AliasCanonicalMatch),
    Ambiguous(AliasAmbiguity),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExactArticleKeywordEntity {
    pub entity_type: DiscoverType,
    pub label: String,
    pub primary_id: Option<String>,
    pub matched_query: String,
    pub matched_alias: bool,
}

impl DiscoverType {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Gene => "Gene",
            Self::Drug => "Drug",
            Self::Disease => "Disease",
            Self::Symptom => "Symptom",
            Self::Pathway => "Pathway",
            Self::Variant => "Variant",
            Self::Unknown => "Unknown",
        }
    }

    pub(crate) fn cli_name(self) -> &'static str {
        match self {
            Self::Gene => "gene",
            Self::Drug => "drug",
            Self::Disease => "disease",
            Self::Symptom => "symptom",
            Self::Pathway => "pathway",
            Self::Variant => "variant",
            Self::Unknown => "unknown",
        }
    }
}

impl MatchTier {
    fn rank(self) -> u8 {
        match self {
            Self::Exact => 0,
            Self::Prefix => 1,
            Self::Contains => 2,
            Self::Weak => 3,
        }
    }
}

impl DiscoverConfidence {
    fn rank(self) -> u8 {
        match self {
            Self::CanonicalId => 0,
            Self::UmlsOnly => 1,
            Self::LabelOnly => 2,
        }
    }
}

pub(crate) async fn resolve_query(
    query: &str,
    mode: DiscoverMode,
) -> Result<DiscoverResult, BioMcpError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Free-text query is required. Example: biomcp discover BRCA1".into(),
        ));
    }

    let ols_client = crate::sources::ols4::OlsClient::new()?;
    let umls_client = crate::sources::umls::UmlsClient::new()?;
    let medline_client = match mode {
        DiscoverMode::Command => Some(crate::sources::medlineplus::MedlinePlusClient::new()?),
        DiscoverMode::AliasFallback => None,
    };

    let ols_query = symptom_disease_lookup_query(query);
    let ols_future = async {
        match tokio::time::timeout(OLS4_TIMEOUT, ols_client.search(&ols_query)).await {
            Ok(result) => result,
            Err(_) => Err(BioMcpError::Api {
                api: "ols4".to_string(),
                message: format!(
                    "Timed out after {}ms while resolving discover query",
                    OLS4_TIMEOUT.as_millis()
                ),
            }),
        }
    };

    let query_owned = query.to_string();
    let umls_future = async move {
        let Some(client) = umls_client else {
            let note = match mode {
                DiscoverMode::Command => {
                    Some("UMLS enrichment unavailable (set UMLS_API_KEY)".to_string())
                }
                DiscoverMode::AliasFallback => None,
            };
            return (Vec::new(), note);
        };

        match tokio::time::timeout(UMLS_TIMEOUT, client.search(&query_owned)).await {
            Ok(Ok(rows)) => (rows, None),
            Ok(Err(err)) => (
                Vec::new(),
                match mode {
                    DiscoverMode::Command => Some(format!("UMLS enrichment unavailable ({err})")),
                    DiscoverMode::AliasFallback => None,
                },
            ),
            Err(_) => (
                Vec::new(),
                match mode {
                    DiscoverMode::Command => {
                        Some("UMLS enrichment unavailable (timed out)".to_string())
                    }
                    DiscoverMode::AliasFallback => None,
                },
            ),
        }
    };

    let query_owned = query.to_string();
    let medline_future = async move {
        let Some(client) = medline_client else {
            return Vec::new();
        };
        match tokio::time::timeout(MEDLINEPLUS_TIMEOUT, client.search(&query_owned)).await {
            Ok(Ok(rows)) => rows,
            Ok(Err(_)) | Err(_) => Vec::new(),
        }
    };

    let (ols_docs, (umls_rows, umls_note), medline_topics) = match mode {
        DiscoverMode::Command => tokio::join!(ols_future, umls_future, medline_future),
        DiscoverMode::AliasFallback => {
            crate::sources::with_no_cache(true, async {
                tokio::join!(ols_future, umls_future, medline_future)
            })
            .await
        }
    };

    let mut ols_docs = ols_docs.map_err(|err| match err {
        BioMcpError::Api { api, message } if api == "ols4" => BioMcpError::Api {
            api,
            message: format!("discover requires OLS4: {message}"),
        },
        other => other,
    })?;
    if !is_disease_symptom_request(query)
        && !ols_docs.iter().any(|doc| {
            doc.obo_id
                .as_deref()
                .or(doc.short_form.as_deref())
                .and_then(normalize_primary_id)
                .is_some_and(|id| id.starts_with("HP:"))
        })
        && let Ok(hpo_client) = HpoClient::new()
        && let Ok(Ok(ids)) =
            tokio::time::timeout(OLS4_TIMEOUT, hpo_client.search_term_ids(query, 1)).await
        && let Some(id) = ids.into_iter().next()
        && let Ok(Ok(labels)) = tokio::time::timeout(
            OLS4_TIMEOUT,
            hpo_client.resolve_terms(std::slice::from_ref(&id), 1),
        )
        .await
        && let Some(label) = labels.get(&id)
    {
        ols_docs.push(OlsDoc {
            iri: format!("https://ontology.jax.org/api/hp/terms/{id}"),
            ontology_name: "hp".to_string(),
            ontology_prefix: "hp".to_string(),
            short_form: Some(id.replace(':', "_")),
            obo_id: Some(id.clone()),
            label: label.clone(),
            description: Vec::new(),
            exact_synonyms: Vec::new(),
            is_defining_ontology: true,
            doc_type: Some("class".to_string()),
        });
    }
    let mut notes = Vec::new();
    if let Some(note) = umls_note
        && !note.trim().is_empty()
    {
        notes.push(note);
    }

    Ok(build_result(
        query,
        &ols_docs,
        &umls_rows,
        &medline_topics,
        notes,
    ))
}

pub(crate) async fn resolve_exact_article_keyword_entity(
    query: &str,
) -> Result<Option<ExactArticleKeywordEntity>, BioMcpError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(None);
    }

    let client = crate::sources::ols4::OlsClient::new()?;
    let docs = match tokio::time::timeout(OLS4_TIMEOUT, client.search(query)).await {
        Ok(result) => result?,
        Err(_) => {
            return Err(BioMcpError::Api {
                api: "ols4".to_string(),
                message: format!(
                    "Timed out after {}ms while resolving exact article keyword entity",
                    OLS4_TIMEOUT.as_millis()
                ),
            });
        }
    };

    Ok(resolve_exact_article_keyword_entity_from_ols_docs(
        query, &docs,
    ))
}

pub(crate) fn resolve_exact_article_keyword_entity_from_ols_docs(
    query: &str,
    docs: &[OlsDoc],
) -> Option<ExactArticleKeywordEntity> {
    let query = query.trim();
    if query.is_empty() {
        return None;
    }
    let normalized_query = normalize_query(query);
    let mut candidates: Vec<((DiscoverType, String), ExactArticleKeywordEntity)> = Vec::new();

    for doc in docs {
        let concept = concept_from_ols(doc, query);
        if concept.match_tier != MatchTier::Exact
            || concept.confidence != DiscoverConfidence::CanonicalId
            || !matches!(
                concept.primary_type,
                DiscoverType::Gene | DiscoverType::Drug | DiscoverType::Disease
            )
        {
            continue;
        }

        let key_value = concept
            .primary_id
            .as_deref()
            .map(|id| id.to_ascii_lowercase())
            .unwrap_or_else(|| normalize_query(&concept.label));
        if key_value.is_empty() {
            continue;
        }

        let matched_alias = normalize_query(&concept.label) != normalized_query
            && concept
                .synonyms
                .iter()
                .any(|synonym| normalize_query(synonym) == normalized_query);
        let key = (concept.primary_type, key_value);
        let entity = ExactArticleKeywordEntity {
            entity_type: concept.primary_type,
            label: concept.label,
            primary_id: concept.primary_id,
            matched_query: query.to_string(),
            matched_alias,
        };
        if entity
            .primary_id
            .as_deref()
            .is_some_and(|_| exact_article_keyword_primary_id_rank(&entity) == usize::MAX)
        {
            continue;
        }

        if let Some((_, existing)) = candidates
            .iter_mut()
            .find(|(existing_key, _)| existing_key == &key)
        {
            if !entity.matched_alias {
                existing.matched_alias = false;
                existing.label = entity.label;
                existing.primary_id = entity.primary_id;
            }
        } else {
            candidates.push((key, entity));
        }
    }

    match candidates.len() {
        0 => None,
        1 => candidates.pop().map(|(_, entity)| entity),
        _ => choose_preferred_exact_article_keyword_candidate(&candidates),
    }
}

fn choose_preferred_exact_article_keyword_candidate(
    candidates: &[((DiscoverType, String), ExactArticleKeywordEntity)],
) -> Option<ExactArticleKeywordEntity> {
    let entity_type = candidates.first()?.1.entity_type;
    if candidates
        .iter()
        .any(|(_, entity)| entity.entity_type != entity_type)
    {
        return None;
    }

    let canonical_label = normalize_query(&candidates.first()?.1.label);
    if canonical_label.is_empty()
        || candidates
            .iter()
            .any(|(_, entity)| normalize_query(&entity.label) != canonical_label)
    {
        return None;
    }

    let best_rank = candidates
        .iter()
        .map(|(_, entity)| exact_article_keyword_primary_id_rank(entity))
        .min()?;
    if best_rank == usize::MAX {
        return None;
    }

    let mut best = candidates
        .iter()
        .filter(|(_, entity)| exact_article_keyword_primary_id_rank(entity) == best_rank)
        .map(|(_, entity)| entity);
    let selected = best.next()?;
    if best.next().is_some() {
        return None;
    }
    Some(selected.clone())
}

fn exact_article_keyword_primary_id_rank(entity: &ExactArticleKeywordEntity) -> usize {
    let Some(primary_id) = entity.primary_id.as_deref() else {
        return usize::MAX;
    };
    let (source, _) = split_prefixed_id(primary_id);
    let preferred = match entity.entity_type {
        DiscoverType::Gene => &["HGNC", "NCBIGENE"][..],
        DiscoverType::Drug => &["CHEBI", "RXNORM", "DRON", "MESH", "NCIT"][..],
        DiscoverType::Disease => &["MONDO", "DOID", "ORDO", "MESH", "NCIT"][..],
        _ => &[][..],
    };
    preferred
        .iter()
        .position(|candidate| source.eq_ignore_ascii_case(candidate))
        .unwrap_or(usize::MAX)
}

pub(crate) fn classify_alias_fallback(
    result: &DiscoverResult,
    requested_entity: DiscoverType,
) -> AliasFallbackDecision {
    if !matches!(requested_entity, DiscoverType::Gene | DiscoverType::Drug) {
        return AliasFallbackDecision::None;
    }

    let Some(top) = result.concepts.first() else {
        return AliasFallbackDecision::None;
    };

    if result.ambiguous {
        return AliasFallbackDecision::Ambiguous(AliasAmbiguity {
            requested_entity,
            query: result.query.clone(),
            candidates: alias_candidates(result),
            next_commands: alias_ambiguous_next_commands(requested_entity, &result.query),
        });
    }

    let canonical = canonical_alias_label(requested_entity, &top.label);
    if top.primary_type == requested_entity
        && top.confidence == DiscoverConfidence::CanonicalId
        && top.match_tier == MatchTier::Exact
        && let (Some(canonical), Some(canonical_id)) = (canonical, top.primary_id.clone())
    {
        return AliasFallbackDecision::Canonical(AliasCanonicalMatch {
            requested_entity,
            query: result.query.clone(),
            canonical: canonical.clone(),
            canonical_id,
            confidence: top.confidence,
            match_tier: top.match_tier,
            sources: alias_sources(top),
            next_commands: alias_canonical_next_commands(requested_entity, &canonical),
        });
    }

    AliasFallbackDecision::Ambiguous(AliasAmbiguity {
        requested_entity,
        query: result.query.clone(),
        candidates: alias_candidates(result),
        next_commands: alias_ambiguous_next_commands(requested_entity, &result.query),
    })
}

pub(crate) fn build_result(
    query: &str,
    ols_docs: &[OlsDoc],
    umls_concepts: &[UmlsConcept],
    medline_topics: &[MedlinePlusTopic],
    mut notes: Vec<String>,
) -> DiscoverResult {
    let normalized_query = normalize_query(query);

    let mut concepts = Vec::new();
    for doc in ols_docs {
        merge_candidate(&mut concepts, concept_from_ols(doc, query));
    }
    for concept in umls_concepts {
        merge_candidate(&mut concepts, concept_from_umls(concept, query));
    }

    if looks_like_variant(query)
        && concepts
            .iter()
            .all(|concept| concept.primary_type == DiscoverType::Unknown)
    {
        concepts.push(DiscoverConcept {
            label: query.trim().to_string(),
            primary_id: None,
            primary_type: DiscoverType::Variant,
            synonyms: Vec::new(),
            xrefs: Vec::new(),
            sources: Vec::new(),
            match_tier: MatchTier::Exact,
            confidence: DiscoverConfidence::LabelOnly,
        });
    }

    concepts.sort_by(|left, right| compare_concepts(left, right, query));
    let intent = detect_intent(query, &normalized_query, &concepts);
    let relational_redirect = if intent == DiscoverIntent::General {
        let filtered = filter_general_discover_concepts(query, &concepts);
        let should_redirect = looks_relational_general_query(&normalized_query)
            && (filtered.is_empty()
                || !filtered.iter().any(|concept| {
                    general_discover_relevance_score(query, concept).strong_survivor
                }));
        concepts = if should_redirect {
            Vec::new()
        } else {
            filtered
        };
        should_redirect
    } else {
        false
    };
    let ambiguous = is_ambiguous(&concepts);
    let plain_language = select_plain_language(&concepts, medline_topics, intent);
    let next_commands = if relational_redirect {
        vec![general_discover_redirect_command(query)]
    } else {
        generate_commands(query, &concepts, ambiguous, intent)
    };
    let has_high_confidence = concepts
        .iter()
        .any(|concept| concept.confidence == DiscoverConfidence::CanonicalId);

    if relational_redirect {
        notes.push(general_discover_redirect_note(query));
    } else if concepts.is_empty() {
        notes.push(empty_results_article_fallback_note(query));
    } else if !has_high_confidence {
        notes.push(broad_results_article_fallback_note(query));
    }

    DiscoverResult {
        query: query.trim().to_string(),
        normalized_query,
        concepts,
        plain_language,
        next_commands,
        notes,
        ambiguous,
        intent,
    }
}

fn compare_concepts(left: &DiscoverConcept, right: &DiscoverConcept, query: &str) -> Ordering {
    let left_key = (
        type_rank(left.primary_type, query),
        primary_id_rank(left),
        left.match_tier.rank(),
        left.confidence.rank(),
        std::cmp::Reverse(left.xrefs.len()),
        std::cmp::Reverse(source_breadth(left)),
        label_tiebreak(left),
    );
    let right_key = (
        type_rank(right.primary_type, query),
        primary_id_rank(right),
        right.match_tier.rank(),
        right.confidence.rank(),
        std::cmp::Reverse(right.xrefs.len()),
        std::cmp::Reverse(source_breadth(right)),
        label_tiebreak(right),
    );
    left_key.cmp(&right_key)
}

fn primary_id_rank(concept: &DiscoverConcept) -> u8 {
    let Some(primary_id) = concept.primary_id.as_deref() else {
        return 3;
    };
    let prefix = primary_id
        .split(':')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    match concept.primary_type {
        DiscoverType::Gene if matches!(prefix.as_str(), "HGNC" | "NCBIGENE") => 0,
        DiscoverType::Drug
            if matches!(
                prefix.as_str(),
                "RXNORM" | "MESH" | "CHEBI" | "DRON" | "NCIT"
            ) =>
        {
            0
        }
        DiscoverType::Disease
            if matches!(
                prefix.as_str(),
                "MONDO" | "DOID" | "ORDO" | "ICD10CM" | "SNOMEDCT"
            ) =>
        {
            0
        }
        DiscoverType::Symptom if matches!(prefix.as_str(), "HP" | "ICD10CM" | "SNOMEDCT") => 0,
        DiscoverType::Pathway if matches!(prefix.as_str(), "GO" | "WIKIPATHWAYS") => 0,
        DiscoverType::Variant if prefix == "SO" => 0,
        _ if prefix == "UMLS" => 2,
        _ => 1,
    }
}

fn type_rank(kind: DiscoverType, query: &str) -> u8 {
    match kind {
        DiscoverType::Unknown => 4,
        DiscoverType::Gene if looks_like_gene_query(query) => 0,
        DiscoverType::Variant if looks_like_variant(query) => 0,
        DiscoverType::Symptom => 1,
        _ => 2,
    }
}

fn label_tiebreak(concept: &DiscoverConcept) -> (String, String) {
    (
        concept.label.to_ascii_lowercase(),
        concept.primary_id.clone().unwrap_or_default(),
    )
}

fn source_breadth(concept: &DiscoverConcept) -> usize {
    concept
        .sources
        .iter()
        .map(|source| source.source.to_ascii_lowercase())
        .collect::<HashSet<_>>()
        .len()
}

fn concept_from_ols(doc: &OlsDoc, query: &str) -> DiscoverConcept {
    let label = doc.label.trim().to_string();
    let primary_id = doc
        .obo_id
        .as_deref()
        .or(doc.short_form.as_deref())
        .and_then(normalize_primary_id);
    let mut primary_type = infer_ols_type(doc, query);
    if primary_id
        .as_deref()
        .is_some_and(|id| id.starts_with("HP:"))
    {
        primary_type = DiscoverType::Symptom;
    }
    let mut xrefs = Vec::new();
    if let Some(id) = primary_id.clone() {
        let (source, bare_id) = split_prefixed_id(&id);
        xrefs.push(ConceptXref {
            source,
            id: bare_id,
        });
    }

    DiscoverConcept {
        label: label.clone(),
        primary_id,
        primary_type,
        synonyms: dedupe_strings(doc.exact_synonyms.clone()),
        xrefs,
        sources: vec![ConceptSource {
            source: "OLS4".to_string(),
            id: doc.obo_id.clone().unwrap_or_else(|| doc.iri.clone()),
            label,
            source_type: doc.ontology_prefix.to_ascii_uppercase(),
        }],
        match_tier: classify_match(query, &doc.label, &doc.exact_synonyms),
        confidence: if doc.obo_id.as_deref().is_some() || doc.short_form.as_deref().is_some() {
            DiscoverConfidence::CanonicalId
        } else {
            DiscoverConfidence::LabelOnly
        },
    }
}

fn concept_from_umls(concept: &UmlsConcept, query: &str) -> DiscoverConcept {
    let primary_type = infer_umls_type(concept, query);
    let mut xrefs = concept
        .xrefs
        .iter()
        .filter_map(normalize_umls_xref)
        .collect::<Vec<_>>();
    xrefs.push(ConceptXref {
        source: "UMLS".to_string(),
        id: concept.cui.clone(),
    });
    let primary_id =
        choose_primary_xref(primary_type, &xrefs).or_else(|| Some(format!("UMLS:{}", concept.cui)));

    DiscoverConcept {
        label: concept.name.trim().to_string(),
        primary_id,
        primary_type,
        synonyms: dedupe_strings(
            concept
                .xrefs
                .iter()
                .map(|xref| xref.label.clone())
                .filter(|label| !label.eq_ignore_ascii_case(&concept.name))
                .collect(),
        ),
        xrefs,
        sources: vec![ConceptSource {
            source: "UMLS".to_string(),
            id: concept.cui.clone(),
            label: concept.name.clone(),
            source_type: concept.semantic_types.join(", "),
        }],
        match_tier: classify_match(query, &concept.name, &[]),
        confidence: DiscoverConfidence::UmlsOnly,
    }
}

fn merge_candidate(concepts: &mut Vec<DiscoverConcept>, candidate: DiscoverConcept) {
    if let Some(existing) = concepts
        .iter_mut()
        .find(|existing| concepts_match(existing, &candidate))
    {
        merge_concept(existing, candidate);
    } else {
        concepts.push(candidate);
    }
}

fn concepts_match(left: &DiscoverConcept, right: &DiscoverConcept) -> bool {
    if left.xrefs.iter().any(|xref| {
        right.xrefs.iter().any(|other| {
            xref_key(xref) == xref_key(other)
                && can_merge_on_xref(left.primary_type, right.primary_type, &xref.source)
        })
    }) {
        return true;
    }

    left.primary_type == right.primary_type
        && normalize_query(&left.label) == normalize_query(&right.label)
}

fn can_merge_on_xref(left: DiscoverType, right: DiscoverType, source: &str) -> bool {
    let source = source.to_ascii_uppercase();
    match (left, right) {
        (DiscoverType::Gene, DiscoverType::Gene) => {
            matches!(source.as_str(), "HGNC" | "NCBIGENE" | "OMIM" | "UMLS")
        }
        (DiscoverType::Drug, DiscoverType::Drug) => {
            matches!(
                source.as_str(),
                "RXNORM" | "MESH" | "CHEBI" | "DRON" | "NCIT" | "NCI" | "UMLS"
            )
        }
        (DiscoverType::Disease, DiscoverType::Disease) => {
            matches!(source.as_str(), "MONDO" | "DOID" | "ORDO" | "HP" | "UMLS")
        }
        (DiscoverType::Symptom, DiscoverType::Symptom) => {
            matches!(source.as_str(), "HP" | "UMLS")
        }
        (DiscoverType::Pathway, DiscoverType::Pathway) => {
            matches!(source.as_str(), "GO" | "WIKIPATHWAYS" | "UMLS")
        }
        (DiscoverType::Variant, DiscoverType::Variant) => matches!(source.as_str(), "SO" | "UMLS"),
        _ => false,
    }
}

fn merge_concept(existing: &mut DiscoverConcept, candidate: DiscoverConcept) {
    if candidate.match_tier.rank() < existing.match_tier.rank() {
        existing.match_tier = candidate.match_tier;
    }
    if existing.primary_id.is_none() {
        existing.primary_id = candidate.primary_id.clone();
    }
    existing.primary_type = better_type(existing.primary_type, candidate.primary_type);
    if candidate.confidence.rank() < existing.confidence.rank() {
        existing.confidence = candidate.confidence;
    }
    existing.synonyms.extend(candidate.synonyms);
    existing.synonyms = dedupe_strings(existing.synonyms.clone());
    existing.xrefs.extend(candidate.xrefs);
    existing.xrefs = dedupe_xrefs(existing.xrefs.clone());
    existing.sources.extend(candidate.sources);
    existing.sources = dedupe_sources(existing.sources.clone());
}

fn better_type(existing: DiscoverType, candidate: DiscoverType) -> DiscoverType {
    if existing == DiscoverType::Unknown {
        candidate
    } else if candidate == DiscoverType::Unknown {
        existing
    } else if matches!(existing, DiscoverType::Disease)
        && matches!(candidate, DiscoverType::Symptom)
    {
        candidate
    } else {
        existing
    }
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_ascii_lowercase();
        if seen.insert(key) {
            out.push(trimmed.to_string());
        }
    }
    out
}

fn dedupe_xrefs(values: Vec<ConceptXref>) -> Vec<ConceptXref> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for value in values {
        let key = xref_key(&value);
        if seen.insert(key) {
            out.push(value);
        }
    }
    out
}

fn dedupe_sources(values: Vec<ConceptSource>) -> Vec<ConceptSource> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for value in values {
        let key = format!(
            "{}:{}:{}",
            value.source.to_ascii_lowercase(),
            value.id.to_ascii_lowercase(),
            value.source_type.to_ascii_lowercase()
        );
        if seen.insert(key) {
            out.push(value);
        }
    }
    out
}

fn normalize_query(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn discover_relevance_tokens(value: &str) -> Vec<String> {
    normalize_query(value)
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn discover_retained_query_tokens(query: &str) -> Vec<String> {
    discover_relevance_tokens(query)
        .into_iter()
        .filter(|token| {
            let token = token.as_str();
            !DISCOVER_GENERAL_STOPWORDS.contains(&token)
                && !DISCOVER_GENERAL_RELATIONAL_TOKENS.contains(&token)
        })
        .collect()
}

fn general_discover_relevance_score(
    query: &str,
    concept: &DiscoverConcept,
) -> GeneralDiscoverScore {
    general_discover_relevance_score_with_query_parts(
        &discover_retained_query_tokens(query),
        &normalize_query(query),
        concept,
    )
}

fn general_discover_relevance_score_with_query_parts(
    retained_query_tokens: &[String],
    normalized_query: &str,
    concept: &DiscoverConcept,
) -> GeneralDiscoverScore {
    let query_token_set = retained_query_tokens
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let overlap_count = std::iter::once(&concept.label)
        .chain(concept.synonyms.iter())
        .map(|surface| {
            discover_relevance_tokens(surface)
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .filter(|token| query_token_set.contains(token))
                .count()
        })
        .max()
        .unwrap_or(0);
    let protected_exact = normalize_query(&concept.label) == normalized_query
        || concept
            .synonyms
            .iter()
            .any(|synonym| normalize_query(synonym) == normalized_query);

    GeneralDiscoverScore {
        overlap_count,
        protected_exact,
        strong_survivor: protected_exact || overlap_count >= 2,
    }
}

fn filter_general_discover_concepts(
    query: &str,
    concepts: &[DiscoverConcept],
) -> Vec<DiscoverConcept> {
    let retained_query_tokens = discover_retained_query_tokens(query);
    if retained_query_tokens.len() <= 1 {
        return concepts
            .iter()
            .take(GENERAL_DISCOVER_MAX_SURVIVORS)
            .cloned()
            .collect();
    }

    let normalized_query = normalize_query(query);
    let mut scored = concepts
        .iter()
        .filter_map(|concept| {
            let score = general_discover_relevance_score_with_query_parts(
                &retained_query_tokens,
                &normalized_query,
                concept,
            );
            (score.protected_exact || score.overlap_count > 0).then_some((concept, score))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|(left_concept, left_score), (right_concept, right_score)| {
        right_score
            .strong_survivor
            .cmp(&left_score.strong_survivor)
            .then_with(|| right_score.overlap_count.cmp(&left_score.overlap_count))
            .then_with(|| compare_concepts(left_concept, right_concept, query))
    });

    scored
        .into_iter()
        .take(GENERAL_DISCOVER_MAX_SURVIVORS)
        .map(|(concept, _)| concept.clone())
        .collect()
}

fn looks_relational_general_query(normalized_query: &str) -> bool {
    contains_any_phrase(normalized_query, DISCOVER_GENERAL_RELATIONAL_PHRASES)
}

fn normalize_primary_id(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let upper = value.to_ascii_uppercase();
    if let Some(id) = upper.strip_prefix("HP_") {
        return Some(format!("HP:{id}"));
    }
    Some(
        value
            .replace("hgnc:", "HGNC:")
            .replace("mesh:", "MESH:")
            .replace("go:", "GO:")
            .replace("doid:", "DOID:")
            .replace("mondo:", "MONDO:")
            .replace("hp:", "HP:")
            .replace("ordo:", "ORDO:")
            .replace("chebi:", "CHEBI:")
            .replace("dron:", "DRON:")
            .replace("ncit:", "NCIT:")
            .replace("so:", "SO:")
            .replace("wikipathways:", "WIKIPATHWAYS:"),
    )
}

fn split_prefixed_id(value: &str) -> (String, String) {
    let mut parts = value.splitn(2, ':');
    let source = parts.next().unwrap_or("OLS4").to_ascii_uppercase();
    let id = parts.next().unwrap_or(value).to_string();
    (source, id)
}

fn normalize_umls_xref(xref: &UmlsXref) -> Option<ConceptXref> {
    let source = match xref.vocab.trim() {
        "SNOMEDCT_US" => "SNOMEDCT".to_string(),
        "MSH" => "MESH".to_string(),
        "" => return None,
        other => other.to_string(),
    };
    let id = xref.id.trim();
    if id.is_empty() {
        return None;
    }
    let (_, bare_id) = split_prefixed_id(id);
    Some(ConceptXref {
        source,
        id: bare_id,
    })
}

fn choose_primary_xref(kind: DiscoverType, xrefs: &[ConceptXref]) -> Option<String> {
    let preferred = match kind {
        DiscoverType::Gene => &["HGNC", "NCBIGENE"][..],
        DiscoverType::Drug => &["RXNORM", "MESH", "CHEBI", "DRON", "NCIT"][..],
        DiscoverType::Disease => &[
            "MONDO", "DOID", "ORDO", "ICD10CM", "SNOMEDCT", "OMIM", "MESH",
        ][..],
        DiscoverType::Symptom => &["HP", "ICD10CM", "SNOMEDCT", "MESH"][..],
        DiscoverType::Pathway => &["GO", "WIKIPATHWAYS"][..],
        DiscoverType::Variant => &["SO"][..],
        DiscoverType::Unknown => &[][..],
    };

    for source in preferred {
        if let Some(xref) = xrefs
            .iter()
            .find(|xref| xref.source.eq_ignore_ascii_case(source))
        {
            return Some(format!("{}:{}", xref.source, xref.id));
        }
    }
    None
}

fn classify_match(query: &str, label: &str, synonyms: &[String]) -> MatchTier {
    let query = normalize_query(query);
    let label = normalize_query(label);
    let synonyms = synonyms
        .iter()
        .map(|value| normalize_query(value))
        .collect::<Vec<_>>();

    if label == query || synonyms.iter().any(|syn| syn == &query) {
        MatchTier::Exact
    } else if label.starts_with(&query)
        || synonyms.iter().any(|syn| syn.starts_with(&query))
        || query.starts_with(&label)
    {
        MatchTier::Prefix
    } else if label.contains(&query) || synonyms.iter().any(|syn| syn.contains(&query)) {
        MatchTier::Contains
    } else {
        MatchTier::Weak
    }
}

fn infer_ols_type(doc: &OlsDoc, query: &str) -> DiscoverType {
    let prefix = doc.ontology_prefix.to_ascii_uppercase();
    match prefix.as_str() {
        "HGNC" => DiscoverType::Gene,
        "CHEBI" | "DRON" => DiscoverType::Drug,
        "MONDO" | "DOID" | "ORDO" => DiscoverType::Disease,
        "HP" => DiscoverType::Symptom,
        "GO" | "WIKIPATHWAYS" => DiscoverType::Pathway,
        "SO" => DiscoverType::Variant,
        "MESH" | "NCIT" => heuristic_type(&doc.label, query),
        _ => DiscoverType::Unknown,
    }
}

fn infer_umls_type(concept: &UmlsConcept, query: &str) -> DiscoverType {
    for semantic_type in &concept.semantic_types {
        match semantic_type.trim() {
            "Gene or Genome" => return DiscoverType::Gene,
            "Pharmacologic Substance" | "Clinical Drug" | "Organic Chemical" => {
                return DiscoverType::Drug;
            }
            "Disease or Syndrome" | "Neoplastic Process" => return DiscoverType::Disease,
            "Sign or Symptom" => return DiscoverType::Symptom,
            "Cell Function" | "Molecular Function" | "Biologic Function" | "Biological Process" => {
                return DiscoverType::Pathway;
            }
            "Mutation" => return DiscoverType::Variant,
            _ => {}
        }
    }

    for xref in &concept.xrefs {
        match xref.vocab.as_str() {
            "HGNC" | "NCBIGENE" => return DiscoverType::Gene,
            "RXNORM" | "CHEBI" | "DRON" => return DiscoverType::Drug,
            "MONDO" | "DOID" | "ORDO" | "ICD10CM" | "OMIM" => return DiscoverType::Disease,
            "HP" => return DiscoverType::Symptom,
            "GO" | "WIKIPATHWAYS" => return DiscoverType::Pathway,
            "SO" => return DiscoverType::Variant,
            _ => {}
        }
    }

    heuristic_type(&concept.name, query)
}

fn heuristic_type(label: &str, query: &str) -> DiscoverType {
    let text = format!("{label} {query}").to_ascii_lowercase();
    if looks_like_variant(label)
        || text.contains("mutation")
        || text.contains("variant")
        || text.contains("allele")
    {
        DiscoverType::Variant
    } else if text.contains("pain")
        || text.contains("symptom")
        || text.contains("shortness of breath")
    {
        DiscoverType::Symptom
    } else if text.contains("pathway") || text.contains("signaling") || text.contains("process") {
        DiscoverType::Pathway
    } else if text.contains("mab")
        || text.contains("nib")
        || text.contains("therapy")
        || text.contains("drug")
        || text.contains("injection")
    {
        DiscoverType::Drug
    } else if looks_like_gene_query(query) && crate::sources::is_valid_gene_symbol(label) {
        DiscoverType::Gene
    } else if text.contains("syndrome")
        || text.contains("disease")
        || text.contains("disorder")
        || text.contains("cancer")
        || text.contains("diabetes")
        || text.contains("fibrosis")
    {
        DiscoverType::Disease
    } else {
        DiscoverType::Unknown
    }
}

fn detect_intent(
    query: &str,
    normalized_query: &str,
    concepts: &[DiscoverConcept],
) -> DiscoverIntent {
    if contains_any_phrase(
        normalized_query,
        &[
            "trial",
            "trials",
            "study",
            "studies",
            "recruiting",
            "recruitment",
        ],
    ) {
        return DiscoverIntent::TrialSearch;
    }
    if contains_any_phrase(
        normalized_query,
        &[
            "side effect",
            "side effects",
            "adverse event",
            "adverse events",
            "adverse effect",
            "adverse effects",
            "toxicity",
            "toxicities",
            "safety",
            "warning",
            "warnings",
        ],
    ) && top_concept_of_type(concepts, DiscoverType::Drug).is_some()
    {
        return DiscoverIntent::DrugSafety;
    }
    if contains_any_phrase(
        normalized_query,
        &[
            "treat",
            "treats",
            "treated",
            "treatment",
            "therapy",
            "therapies",
            "drug for",
            "drugs for",
            "medication",
            "medications",
        ],
    ) && treatment_focus(query, concepts).is_some()
    {
        return DiscoverIntent::TreatmentSearch;
    }
    if contains_any_phrase(
        normalized_query,
        &[
            "symptom",
            "symptoms",
            "sign",
            "signs",
            "phenotype",
            "phenotypes",
            "clinical feature",
            "clinical features",
        ],
    ) || matches!(
        concepts.first(),
        Some(concept)
            if concept.primary_type == DiscoverType::Symptom && concept.match_tier != MatchTier::Weak
    ) {
        return DiscoverIntent::SymptomSearch;
    }
    if gene_disease_focus(query, concepts).is_some() {
        return DiscoverIntent::GeneDiseaseOrientation;
    }
    if contains_any_phrase(
        normalized_query,
        &["what does", "function", "role", "activity", "what is"],
    ) && top_concept_of_type(concepts, DiscoverType::Gene).is_some()
    {
        return DiscoverIntent::GeneFunction;
    }
    DiscoverIntent::General
}

fn select_plain_language(
    concepts: &[DiscoverConcept],
    medline_topics: &[MedlinePlusTopic],
    intent: DiscoverIntent,
) -> Option<PlainLanguageTopic> {
    if intent == DiscoverIntent::TrialSearch {
        return None;
    }
    let top = concepts.first()?;
    if !matches!(
        top.primary_type,
        DiscoverType::Disease | DiscoverType::Symptom
    ) {
        return None;
    }

    medline_topics
        .iter()
        .find(|topic| related_medline_topic(&top.label, topic))
        .cloned()
        .map(|topic| PlainLanguageTopic {
            title: topic.title,
            url: topic.url,
            summary_excerpt: topic.summary_excerpt,
        })
}

fn related_medline_topic(label: &str, topic: &MedlinePlusTopic) -> bool {
    let label = normalize_query(label);
    let title = normalize_query(&topic.title);
    title.contains(&label) || label.contains(&title)
}

fn is_ambiguous(concepts: &[DiscoverConcept]) -> bool {
    if concepts.len() < 2 {
        return false;
    }
    let top = &concepts[0];
    let same_type_match_limit = match top.primary_type {
        DiscoverType::Disease => MatchTier::Contains.rank(),
        _ => MatchTier::Prefix.rank(),
    };
    let same_type_competitors = concepts
        .iter()
        .take_while(|concept| concept.primary_type == top.primary_type)
        .filter(|concept| {
            concept.match_tier.rank() <= same_type_match_limit
                && concept.confidence.rank() <= DiscoverConfidence::UmlsOnly.rank()
        })
        .count();
    if same_type_competitors > 1 {
        return true;
    }

    concepts.iter().skip(1).take(3).any(|concept| {
        concept.primary_type != top.primary_type
            && concept.match_tier.rank() <= MatchTier::Prefix.rank()
            && concept.confidence.rank() <= DiscoverConfidence::UmlsOnly.rank()
    })
}

fn generate_commands(
    query: &str,
    concepts: &[DiscoverConcept],
    ambiguous: bool,
    intent: DiscoverIntent,
) -> Vec<String> {
    let mut commands = Vec::new();
    let has_high_confidence = concepts
        .iter()
        .any(|concept| concept.confidence == DiscoverConfidence::CanonicalId);
    let Some(top) = concepts.first() else {
        commands.push(review_article_fallback_command(query));
        return commands;
    };

    match intent {
        DiscoverIntent::TrialSearch => {
            if matches!(
                top.primary_type,
                DiscoverType::Disease | DiscoverType::Symptom
            ) {
                commands.push(format!(
                    "biomcp search trial -c \"{}\" --limit 5",
                    top.label
                ));
                commands.push(format!(
                    "biomcp search article -k \"{}\" --limit 5",
                    top.label
                ));
            }
            return dedupe_strings(commands);
        }
        DiscoverIntent::DrugSafety => {
            if let Some(drug) = top_concept_of_type(concepts, DiscoverType::Drug) {
                let drug_name = drug.label.to_ascii_lowercase();
                commands.push(format!("biomcp drug adverse-events {drug_name}"));
                commands.push(format!("biomcp get drug {drug_name} safety"));
                commands.push(format!(
                    "biomcp search article --drug {} --type review --limit 5",
                    quote_query_term(&drug.label)
                ));
            }
        }
        DiscoverIntent::TreatmentSearch => {
            if let Some(disease) = treatment_focus(query, concepts) {
                commands.push(format!(
                    "biomcp search drug --indication {} --limit 5",
                    quote_query_term(&disease)
                ));
                commands.push(format!(
                    "biomcp search article -d {} --type review --limit 5",
                    quote_query_term(&disease)
                ));
            }
        }
        DiscoverIntent::SymptomSearch => {
            if is_disease_symptom_request(query)
                && let Some(disease) = top_concept_of_type(concepts, DiscoverType::Disease)
            {
                if let Some(disease_ref) = disease_command_ref(disease) {
                    commands.push(format!("biomcp get disease {disease_ref} phenotypes"));
                }
                commands.push(format!(
                    "biomcp search article -d {} --type review --limit 5",
                    quote_query_term(&disease.label)
                ));
            } else {
                let hpo_ids = collect_hpo_ids(concepts);
                if !hpo_ids.is_empty() {
                    commands.push(format!("biomcp search phenotype \"{}\"", hpo_ids.join(" ")));
                }
                commands.push(format!(
                    "biomcp search disease -q {} --limit 10",
                    quote_query_term(query.trim())
                ));
                commands.push(format!(
                    "biomcp search trial -c {} --limit 5",
                    quote_query_term(query.trim())
                ));
                commands.push(format!(
                    "biomcp search article -k {} --limit 5",
                    quote_query_term(query.trim())
                ));
            }
            return dedupe_strings(commands);
        }
        DiscoverIntent::GeneDiseaseOrientation => {
            if let Some((gene, disease)) = gene_disease_focus(query, concepts) {
                commands.push(format!(
                    "biomcp search all --gene {} --disease \"{}\"",
                    gene, disease
                ));
                commands.push(format!("biomcp get gene {}", gene));
                if let Some(disease_ref) =
                    disease_concept_for_label(concepts, &disease).and_then(disease_command_ref)
                {
                    commands.push(format!("biomcp get disease {disease_ref}"));
                }
                return dedupe_strings(commands);
            }
        }
        DiscoverIntent::GeneFunction => {
            if let Some(gene) = top_concept_of_type(concepts, DiscoverType::Gene) {
                commands.push(format!("biomcp get gene {}", gene.label));
                if let Some(topic) = gene_article_topic(query, &gene.label, intent) {
                    commands.push(format!(
                        "biomcp search article -g {} -k {} --limit 5",
                        gene.label,
                        crate::render::markdown::shell_quote_arg(&topic)
                    ));
                } else {
                    commands.push(format!("biomcp search article -g {} --limit 5", gene.label));
                }
                return dedupe_strings(commands);
            }
        }
        DiscoverIntent::General => {}
    }

    match top.primary_type {
        DiscoverType::Gene if !ambiguous => {
            commands.push(format!("biomcp get gene {}", top.label));
            if let Some(topic) = gene_article_topic(query, &top.label, intent) {
                commands.push(format!(
                    "biomcp search article -g {} -k {} --limit 5",
                    top.label,
                    crate::render::markdown::shell_quote_arg(&topic)
                ));
            }
        }
        DiscoverType::Gene => commands.push(format!(
            "biomcp search gene -q \"{}\" --limit 10",
            query.trim()
        )),
        DiscoverType::Drug => commands.push(format!(
            "biomcp get drug \"{}\"",
            top.label.to_ascii_lowercase()
        )),
        DiscoverType::Disease if ambiguous => commands.push(format!(
            "biomcp search disease -q \"{}\" --limit 10",
            query.trim()
        )),
        DiscoverType::Disease => {
            commands.push(format!("biomcp get disease \"{}\"", top.label));
            commands.push(format!("biomcp disease trials \"{}\"", top.label));
            commands.push(format!(
                "biomcp search article -k \"{}\" --limit 5",
                top.label
            ));
        }
        DiscoverType::Symptom => {
            commands.push(format!(
                "biomcp search disease -q \"{}\" --limit 10",
                query.trim()
            ));
            commands.push(format!(
                "biomcp search trial -c \"{}\" --limit 5",
                query.trim()
            ));
            commands.push(format!(
                "biomcp search article -k \"{}\" --limit 5",
                query.trim()
            ));
        }
        DiscoverType::Pathway => commands.push(format!(
            "biomcp search pathway -q \"{}\" --limit 5",
            top.label
        )),
        DiscoverType::Variant => {
            if let Ok(crate::entities::variant::VariantIdFormat::GeneProteinChange {
                gene,
                change,
            }) = crate::entities::variant::parse_variant_id(query.trim())
            {
                commands.push(format!("biomcp get variant \"{gene} {change}\""));
            }
            commands.push(format!(
                "biomcp search article -k \"{}\" --limit 5",
                query.trim()
            ));
        }
        DiscoverType::Unknown => {
            commands.push(format!(
                "biomcp search article -k {} --limit 5",
                quote_query_term(query.trim())
            ));
        }
    }

    if !has_high_confidence
        && !commands
            .iter()
            .any(|command| command.starts_with("biomcp search article"))
    {
        commands.push(broad_article_fallback_command(query));
    }

    dedupe_strings(commands)
}

fn contains_any_phrase(normalized_query: &str, phrases: &[&str]) -> bool {
    let query_tokens = normalized_query.split_whitespace().collect::<Vec<_>>();
    phrases.iter().any(|phrase| {
        let phrase = normalize_query(phrase);
        let phrase_tokens = phrase.split_whitespace().collect::<Vec<_>>();
        !phrase_tokens.is_empty()
            && query_tokens
                .windows(phrase_tokens.len())
                .any(|window| window == phrase_tokens.as_slice())
    })
}

fn top_concept_of_type(
    concepts: &[DiscoverConcept],
    kind: DiscoverType,
) -> Option<&DiscoverConcept> {
    concepts.iter().find(|concept| concept.primary_type == kind)
}

fn collect_hpo_ids(concepts: &[DiscoverConcept]) -> Vec<String> {
    let mut ids = Vec::new();
    let mut seen = HashSet::new();

    for concept in concepts {
        let Some(id) = concept.primary_id.as_deref() else {
            continue;
        };
        if concept.primary_type == DiscoverType::Symptom
            && id.starts_with("HP:")
            && seen.insert(id.to_string())
        {
            ids.push(id.to_string());
            if ids.len() == 5 {
                break;
            }
        }
    }

    ids
}

fn disease_concept_for_label<'a>(
    concepts: &'a [DiscoverConcept],
    label: &str,
) -> Option<&'a DiscoverConcept> {
    let normalized = normalize_query(label);
    concepts.iter().find(|concept| {
        concept.primary_type == DiscoverType::Disease
            && normalize_query(&concept.label) == normalized
    })
}

fn treatment_focus(query: &str, concepts: &[DiscoverConcept]) -> Option<String> {
    if let Some(disease) = top_concept_of_type(concepts, DiscoverType::Disease) {
        return Some(disease.label.clone());
    }

    strip_ascii_prefixes(
        query,
        &[
            "what drugs treat ",
            "what drug treats ",
            "what treats ",
            "drugs for ",
            "drug for ",
            "treatment for ",
            "treatments for ",
            "therapy for ",
            "therapies for ",
            "medication for ",
            "medications for ",
            "treat ",
        ],
    )
    .map(ToString::to_string)
}

fn is_disease_symptom_request(query: &str) -> bool {
    contains_any_phrase(
        &normalize_query(query),
        &[
            "symptoms of",
            "signs of",
            "phenotypes of",
            "clinical features of",
        ],
    )
}

fn symptom_disease_lookup_query(query: &str) -> String {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    static SYMPTOM_DISEASE_LOOKUP_PREFIX_RE: OnceLock<Regex> = OnceLock::new();
    let stripped = SYMPTOM_DISEASE_LOOKUP_PREFIX_RE
        .get_or_init(|| {
            Regex::new(r"(?i)^\s*(symptoms of|signs of|phenotypes of)\s+")
                .expect("valid symptom disease lookup regex")
        })
        .replace(trimmed, "");
    let stripped = stripped.trim();

    if stripped.is_empty() {
        trimmed.to_string()
    } else {
        stripped.to_string()
    }
}

fn gene_disease_focus(query: &str, concepts: &[DiscoverConcept]) -> Option<(String, String)> {
    if matches!(
        concepts.first().map(|concept| concept.primary_type),
        Some(DiscoverType::Pathway)
    ) || contains_any_phrase(
        &normalize_query(query),
        &["pathway", "signaling", "cascade", "process"],
    ) {
        return None;
    }

    if let (Some(gene), Some(disease)) = (
        top_concept_of_type(concepts, DiscoverType::Gene),
        top_concept_of_type(concepts, DiscoverType::Disease),
    ) {
        return Some((gene.label.clone(), disease.label.clone()));
    }

    None
}

fn gene_article_topic(query: &str, gene_label: &str, intent: DiscoverIntent) -> Option<String> {
    let mut tokens = query
        .split_whitespace()
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    let remove_index = tokens.iter().position(|token| {
        token
            .trim_matches(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
            .eq_ignore_ascii_case(gene_label)
    })?;
    tokens.remove(remove_index);

    let mut topic = tokens.join(" ");
    if topic.is_empty() {
        return None;
    }

    if intent == DiscoverIntent::GeneFunction {
        topic = strip_gene_function_scaffolding(&topic);
    }

    (!topic.is_empty()).then_some(topic)
}

fn strip_gene_function_scaffolding(topic: &str) -> String {
    const PHRASES: &[&[&str]] = &[&["what", "is"], &["what", "does"]];
    const FILLER: &[&str] = &[
        "function", "role", "activity", "do", "does", "is", "gene", "protein", "of", "in",
    ];

    let mut tokens = topic
        .split_whitespace()
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    loop {
        let mut removed_phrase = false;
        for phrase in PHRASES {
            if tokens.len() >= phrase.len()
                && tokens
                    .iter()
                    .take(phrase.len())
                    .map(|token| token.to_ascii_lowercase())
                    .eq(phrase.iter().map(|token| (*token).to_string()))
            {
                tokens.drain(..phrase.len());
                removed_phrase = true;
                break;
            }
        }
        if !removed_phrase {
            break;
        }
    }

    while let Some(first) = tokens.first() {
        let lowered = first.to_ascii_lowercase();
        if FILLER.contains(&lowered.as_str()) {
            tokens.remove(0);
        } else {
            break;
        }
    }

    tokens.join(" ")
}

pub(crate) fn looks_like_gene_symbol_token(token: &str) -> bool {
    let token = token.trim();
    !token.is_empty()
        && token.len() <= 10
        && token == token.to_ascii_uppercase()
        && crate::sources::is_valid_gene_symbol(token)
}

fn strip_ascii_prefixes<'a>(query: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    let trimmed = query.trim();
    let lowered = trimmed.to_ascii_lowercase();
    prefixes.iter().find_map(|prefix| {
        if lowered.starts_with(prefix) {
            Some(trimmed[prefix.len()..].trim()).filter(|rest| !rest.is_empty())
        } else {
            None
        }
    })
}

fn disease_command_ref(concept: &DiscoverConcept) -> Option<String> {
    concept
        .primary_id
        .as_deref()
        .filter(|value| {
            value.starts_with("MONDO:")
                || value.starts_with("DOID:")
                || value.starts_with("ORDO:")
                || value.starts_with("ICD10CM:")
        })
        .map(ToString::to_string)
        .or_else(|| {
            let label = quote_query_term(&concept.label);
            (!label.is_empty()).then_some(label)
        })
}

fn quote_query_term(value: &str) -> String {
    quote_arg(value).if_empty_then(format!("\"{}\"", value.trim()))
}

fn review_article_fallback_command(query: &str) -> String {
    format!(
        "biomcp search article -k {} --type review --limit 5",
        quote_query_term(query.trim())
    )
}

fn broad_article_fallback_command(query: &str) -> String {
    format!(
        "biomcp search article -k {}",
        quote_query_term(query.trim())
    )
}

fn empty_results_article_fallback_note(query: &str) -> String {
    format!(
        "No biomedical entities resolved. Try: {}",
        review_article_fallback_command(query)
    )
}

fn broad_results_article_fallback_note(query: &str) -> String {
    format!(
        "For broader results: {}",
        broad_article_fallback_command(query)
    )
}

fn general_discover_redirect_command(query: &str) -> String {
    format!(
        "biomcp search all --keyword {}",
        quote_query_term(query.trim())
    )
}

fn general_discover_redirect_note(query: &str) -> String {
    format!(
        "`discover` resolves single entities. For relational questions, try: {}",
        general_discover_redirect_command(query)
    )
}

trait StringExt {
    fn if_empty_then(self, fallback: String) -> String;
}

impl StringExt for String {
    fn if_empty_then(self, fallback: String) -> String {
        if self.trim().is_empty() {
            fallback
        } else {
            self
        }
    }
}

fn quote_arg(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        String::new()
    } else if trimmed.contains(char::is_whitespace) || trimmed.contains('"') {
        format!("\"{}\"", trimmed.replace('"', "\\\""))
    } else {
        trimmed.to_string()
    }
}

fn looks_like_gene_query(query: &str) -> bool {
    let query = query.trim();
    !query.is_empty()
        && query.len() <= 10
        && query
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        && query.chars().any(|ch| ch.is_ascii_digit())
}

fn looks_like_variant(query: &str) -> bool {
    static VARIANT_RE: OnceLock<Regex> = OnceLock::new();
    VARIANT_RE
        .get_or_init(|| Regex::new(r"^[A-Z]\d+[A-Z*]$").expect("valid regex"))
        .is_match(query.trim())
}

fn xref_key(xref: &ConceptXref) -> String {
    format!(
        "{}:{}",
        xref.source.to_ascii_lowercase(),
        xref.id.to_ascii_lowercase()
    )
}

fn canonical_alias_label(requested_entity: DiscoverType, label: &str) -> Option<String> {
    let label = label.trim();
    match requested_entity {
        DiscoverType::Gene if crate::sources::is_valid_gene_symbol(label) => {
            Some(label.to_string())
        }
        DiscoverType::Drug if !label.is_empty() => Some(label.to_ascii_lowercase()),
        _ => None,
    }
}

fn alias_sources(concept: &DiscoverConcept) -> Vec<String> {
    dedupe_strings(
        concept
            .sources
            .iter()
            .map(|source| {
                let source_type = source.source_type.trim();
                if source_type.is_empty() {
                    source.source.clone()
                } else {
                    format!("{}/{}", source.source, source_type)
                }
            })
            .collect(),
    )
}

fn alias_command(entity: DiscoverType, value: &str) -> Option<String> {
    let quoted = crate::render::markdown::quote_arg(value);
    if quoted.is_empty() {
        return None;
    }
    Some(format!("biomcp get {} {quoted}", entity.cli_name()))
}

fn alias_canonical_next_commands(entity: DiscoverType, canonical: &str) -> Vec<String> {
    alias_command(entity, canonical).into_iter().collect()
}

fn alias_ambiguous_next_commands(entity: DiscoverType, query: &str) -> Vec<String> {
    let query = crate::render::markdown::quote_arg(query);
    if query.is_empty() {
        return Vec::new();
    }
    vec![
        format!("biomcp discover {query}"),
        format!("biomcp search {} -q {query}", entity.cli_name()),
    ]
}

fn alias_candidates(result: &DiscoverResult) -> Vec<AliasCandidateSummary> {
    result
        .concepts
        .iter()
        .take(3)
        .map(|concept| AliasCandidateSummary {
            label: concept.label.clone(),
            primary_type: concept.primary_type,
            primary_id: concept.primary_id.clone(),
            confidence: concept.confidence,
            match_tier: concept.match_tier,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        AliasFallbackDecision, ConceptSource, ConceptXref, DiscoverConcept, DiscoverConfidence,
        DiscoverIntent, DiscoverResult, DiscoverType, MatchTier, build_result,
        classify_alias_fallback, concept_from_ols, generate_commands,
        resolve_exact_article_keyword_entity, resolve_exact_article_keyword_entity_from_ols_docs,
        symptom_disease_lookup_query,
    };
    use crate::sources::medlineplus::MedlinePlusTopic;
    use crate::sources::ols4::OlsDoc;
    use crate::sources::umls::{UmlsConcept, UmlsXref};
    use crate::test_support::{env_lock, set_env_var};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn hgnc_doc(label: &str, obo_id: &str, exact_synonyms: &[&str]) -> OlsDoc {
        OlsDoc {
            iri: format!(
                "http://example.org/{}",
                obo_id.replace(':', "/").to_ascii_lowercase()
            ),
            ontology_name: "hgnc".to_string(),
            ontology_prefix: "hgnc".to_string(),
            short_form: Some(obo_id.to_ascii_lowercase()),
            obo_id: Some(obo_id.to_string()),
            label: label.to_string(),
            description: Vec::new(),
            exact_synonyms: exact_synonyms
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            is_defining_ontology: true,
            doc_type: Some("class".to_string()),
        }
    }

    fn ols_doc(prefix: &str, label: &str, obo_id: &str, exact_synonyms: &[&str]) -> OlsDoc {
        OlsDoc {
            iri: format!(
                "http://example.org/{}",
                obo_id.replace(':', "/").to_ascii_lowercase()
            ),
            ontology_name: prefix.to_ascii_lowercase(),
            ontology_prefix: prefix.to_ascii_lowercase(),
            short_form: Some(obo_id.to_ascii_lowercase()),
            obo_id: Some(obo_id.to_string()),
            label: label.to_string(),
            description: Vec::new(),
            exact_synonyms: exact_synonyms
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            is_defining_ontology: true,
            doc_type: Some("class".to_string()),
        }
    }

    fn disease_umls(cui: &str, name: &str, xrefs: &[(&str, &str, &str)]) -> UmlsConcept {
        UmlsConcept {
            cui: cui.to_string(),
            name: name.to_string(),
            semantic_types: vec!["Disease or Syndrome".to_string()],
            xrefs: xrefs
                .iter()
                .map(|(vocab, id, label)| UmlsXref {
                    vocab: (*vocab).to_string(),
                    id: (*id).to_string(),
                    label: (*label).to_string(),
                })
                .collect(),
            uri: format!("https://example.org/{cui}"),
        }
    }

    fn drug_umls(cui: &str, name: &str) -> UmlsConcept {
        UmlsConcept {
            cui: cui.to_string(),
            name: name.to_string(),
            semantic_types: vec!["Clinical Drug".to_string()],
            xrefs: vec![UmlsXref {
                vocab: "RXNORM".to_string(),
                id: "1547545".to_string(),
                label: name.to_string(),
            }],
            uri: format!("https://example.org/{cui}"),
        }
    }

    fn symptom_concept(label: &str, primary_id: &str) -> DiscoverConcept {
        let bare_id = primary_id.trim().trim_start_matches("HP:").to_string();
        DiscoverConcept {
            label: label.to_string(),
            primary_id: Some(primary_id.to_string()),
            primary_type: DiscoverType::Symptom,
            synonyms: Vec::new(),
            xrefs: vec![ConceptXref {
                source: "HP".to_string(),
                id: bare_id.clone(),
            }],
            sources: vec![ConceptSource {
                source: "OLS4".to_string(),
                id: primary_id.to_string(),
                label: label.to_string(),
                source_type: "HP".to_string(),
            }],
            match_tier: MatchTier::Exact,
            confidence: DiscoverConfidence::CanonicalId,
        }
    }

    fn test_concept(
        label: &str,
        primary_type: DiscoverType,
        confidence: DiscoverConfidence,
    ) -> DiscoverConcept {
        DiscoverConcept {
            label: label.to_string(),
            primary_id: None,
            primary_type,
            synonyms: Vec::new(),
            xrefs: Vec::new(),
            sources: Vec::new(),
            match_tier: MatchTier::Exact,
            confidence,
        }
    }

    #[test]
    fn ols_hpo_identifier_overrides_non_hp_prefix_to_symptom() {
        let concept = concept_from_ols(
            &OlsDoc {
                iri: "http://example.org/doid/9256".to_string(),
                ontology_name: "hp".to_string(),
                ontology_prefix: "doid".to_string(),
                short_form: Some("HP_0001263".to_string()),
                obo_id: None,
                label: "Global developmental delay".to_string(),
                description: Vec::new(),
                exact_synonyms: Vec::new(),
                is_defining_ontology: true,
                doc_type: Some("class".to_string()),
            },
            "developmental delay",
        );

        assert_eq!(concept.primary_id.as_deref(), Some("HP:0001263"));
        assert_eq!(concept.primary_type, DiscoverType::Symptom);
    }

    #[test]
    fn symptom_search_with_hpo_ids_suggests_capped_phenotype_bridge_first() {
        let commands = generate_commands(
            "developmental delay",
            &[
                symptom_concept("Global developmental delay", "HP:0001263"),
                symptom_concept("Seizure", "HP:0001250"),
                symptom_concept("Microcephaly", "HP:0000252"),
                symptom_concept("Hypotonia", "HP:0001252"),
                symptom_concept("Ataxia", "HP:0001251"),
                symptom_concept("Developmental regression", "HP:0002376"),
                symptom_concept("Duplicate developmental delay", "HP:0001263"),
            ],
            false,
            DiscoverIntent::SymptomSearch,
        );

        assert_eq!(
            commands[0],
            "biomcp search phenotype \"HP:0001263 HP:0001250 HP:0000252 HP:0001252 HP:0001251\""
        );
        assert_eq!(
            commands[1],
            "biomcp search disease -q \"developmental delay\" --limit 10"
        );
        assert_eq!(commands.len(), 4);
        assert!(!commands[0].contains("HP:0002376"));
    }

    #[test]
    fn hpo_backed_symptom_concepts_route_to_phenotype_search_first() {
        let result = build_result(
            "developmental delay",
            &[
                ols_doc(
                    "mondo",
                    "developmental delay with sleep apnea",
                    "MONDO:0980728",
                    &[],
                ),
                ols_doc("ncit", "Developmental Delay", "NCIT:C116942", &[]),
                ols_doc("hp", "Global developmental delay", "HP:0001263", &[]),
            ],
            &[],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::SymptomSearch);
        assert_eq!(result.concepts[0].primary_id.as_deref(), Some("HP:0001263"));
        assert_eq!(
            result.next_commands[0],
            "biomcp search phenotype \"HP:0001263\""
        );
    }

    #[test]
    fn symptom_queries_keep_search_suggestions_and_plain_language() {
        let result = build_result(
            "chest pain",
            &[],
            &[UmlsConcept {
                cui: "C0008031".to_string(),
                name: "Chest Pain".to_string(),
                semantic_types: vec!["Sign or Symptom".to_string()],
                xrefs: vec![UmlsXref {
                    vocab: "ICD10CM".to_string(),
                    id: "R07.9".to_string(),
                    label: "Chest pain".to_string(),
                }],
                uri: "https://example.org".to_string(),
            }],
            &[MedlinePlusTopic {
                title: "Chest Pain".to_string(),
                url: "https://medlineplus.gov/chestpain.html".to_string(),
                summary_excerpt: "Summary".to_string(),
            }],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::SymptomSearch);
        assert_eq!(result.concepts[0].primary_type, DiscoverType::Symptom);
        assert!(result.plain_language.is_some());
        assert_eq!(
            result.next_commands,
            vec![
                "biomcp search disease -q \"chest pain\" --limit 10".to_string(),
                "biomcp search trial -c \"chest pain\" --limit 5".to_string(),
                "biomcp search article -k \"chest pain\" --limit 5".to_string(),
            ]
        );
        assert!(
            !result
                .next_commands
                .iter()
                .any(|command| command.starts_with("biomcp search phenotype "))
        );
    }

    #[test]
    fn exact_gene_query_promotes_hgnc_result() {
        let result = build_result(
            "ERBB1",
            &[OlsDoc {
                iri: "http://example.org/hgnc/3236".to_string(),
                ontology_name: "hgnc".to_string(),
                ontology_prefix: "hgnc".to_string(),
                short_form: Some("hgnc:3236".to_string()),
                obo_id: Some("HGNC:3236".to_string()),
                label: "EGFR".to_string(),
                description: Vec::new(),
                exact_synonyms: Vec::new(),
                is_defining_ontology: true,
                doc_type: Some("class".to_string()),
            }],
            &[],
            &[],
            Vec::new(),
        );

        assert_eq!(result.concepts[0].label, "EGFR");
        assert_eq!(result.concepts[0].primary_type, DiscoverType::Gene);
        assert_eq!(result.next_commands[0], "biomcp get gene EGFR");
    }

    #[test]
    fn exact_article_keyword_resolver_accepts_gene_drug_and_disease_docs() {
        let gene = resolve_exact_article_keyword_entity_from_ols_docs(
            "BRAF",
            &[hgnc_doc("BRAF", "HGNC:1097", &[])],
        )
        .expect("BRAF should resolve as an exact gene");
        assert_eq!(gene.entity_type, DiscoverType::Gene);
        assert_eq!(gene.label, "BRAF");
        assert_eq!(gene.primary_id.as_deref(), Some("HGNC:1097"));
        assert_eq!(gene.matched_query, "BRAF");
        assert!(!gene.matched_alias);

        let drug = resolve_exact_article_keyword_entity_from_ols_docs(
            "imatinib",
            &[ols_doc("chebi", "imatinib", "CHEBI:45783", &[])],
        )
        .expect("imatinib should resolve as an exact drug");
        assert_eq!(drug.entity_type, DiscoverType::Drug);
        assert_eq!(drug.label, "imatinib");
        assert_eq!(drug.primary_id.as_deref(), Some("CHEBI:45783"));

        let disease = resolve_exact_article_keyword_entity_from_ols_docs(
            "melanoma",
            &[ols_doc("mondo", "melanoma", "MONDO:0005105", &[])],
        )
        .expect("melanoma should resolve as an exact disease");
        assert_eq!(disease.entity_type, DiscoverType::Disease);
        assert_eq!(disease.label, "melanoma");
        assert_eq!(disease.primary_id.as_deref(), Some("MONDO:0005105"));
    }

    #[test]
    fn exact_article_keyword_resolver_reports_alias_to_canonical_match() {
        let resolved = resolve_exact_article_keyword_entity_from_ols_docs(
            "Gleevec",
            &[ols_doc("chebi", "imatinib", "CHEBI:45783", &["Gleevec"])],
        )
        .expect("drug alias should resolve to canonical label");

        assert_eq!(resolved.entity_type, DiscoverType::Drug);
        assert_eq!(resolved.label, "imatinib");
        assert_eq!(resolved.matched_query, "Gleevec");
        assert!(resolved.matched_alias);
    }

    #[test]
    fn exact_article_keyword_resolver_rejects_prefix_unsupported_and_ambiguous_matches() {
        assert!(
            resolve_exact_article_keyword_entity_from_ols_docs(
                "BRAF V600E",
                &[hgnc_doc("BRAF", "HGNC:1097", &[])],
            )
            .is_none()
        );
        assert!(
            resolve_exact_article_keyword_entity_from_ols_docs(
                "seizure",
                &[ols_doc("hp", "Seizure", "HP:0001250", &[])],
            )
            .is_none()
        );
        let preferred_disease = resolve_exact_article_keyword_entity_from_ols_docs(
            "melanoma",
            &[
                ols_doc("doid", "melanoma", "DOID:1909", &[]),
                ols_doc("mondo", "melanoma", "MONDO:0005105", &[]),
            ],
        )
        .expect("live duplicate disease ontology hits should prefer MONDO");
        assert_eq!(
            preferred_disease.primary_id.as_deref(),
            Some("MONDO:0005105")
        );
        assert!(
            resolve_exact_article_keyword_entity_from_ols_docs(
                "skin cancer",
                &[
                    ols_doc(
                        "mondo",
                        "cutaneous melanoma",
                        "MONDO:0005105",
                        &["skin cancer"]
                    ),
                    ols_doc("doid", "skin carcinoma", "DOID:9999", &["skin cancer"]),
                ],
            )
            .is_none()
        );
        assert!(
            resolve_exact_article_keyword_entity_from_ols_docs(
                "BRAF",
                &[
                    hgnc_doc("BRAF", "HGNC:1097", &[]),
                    ols_doc("mondo", "BRAF", "MONDO:9999999", &[]),
                ],
            )
            .is_none()
        );
        let gene = resolve_exact_article_keyword_entity_from_ols_docs(
            "BRAF",
            &[
                hgnc_doc("BRAF", "HGNC:1097", &[]),
                ols_doc("mondo", "BRAF", "1097", &[]),
            ],
        )
        .expect("unsupported duplicate OLS identifiers should not hide HGNC exact matches");
        assert_eq!(gene.entity_type, DiscoverType::Gene);
        assert_eq!(gene.primary_id.as_deref(), Some("HGNC:1097"));
    }

    #[tokio::test]
    async fn exact_article_keyword_resolver_uses_ols4_and_canonicalizes_aliases() {
        let _guard = env_lock().lock().await;
        let server = MockServer::start().await;
        let _ols_base = set_env_var("BIOMCP_OLS4_BASE", Some(&server.uri()));
        Mock::given(method("GET"))
            .and(path("/api/search"))
            .and(query_param("q", "Gleevec"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "response": {
                    "docs": [
                        {
                            "iri": "http://example.org/chebi/45783",
                            "ontology_name": "chebi",
                            "ontology_prefix": "chebi",
                            "short_form": "chebi:45783",
                            "obo_id": "CHEBI:45783",
                            "label": "imatinib",
                            "description": [],
                            "exact_synonyms": ["Gleevec"],
                            "type": "class"
                        }
                    ]
                }
            })))
            .mount(&server)
            .await;

        let resolved = resolve_exact_article_keyword_entity("Gleevec")
            .await
            .expect("resolver should query OLS4")
            .expect("alias should resolve");

        assert_eq!(resolved.entity_type, DiscoverType::Drug);
        assert_eq!(resolved.label, "imatinib");
        assert!(resolved.matched_alias);
    }

    #[test]
    fn trial_intent_suppresses_plain_language() {
        let result = build_result(
            "breast cancer trial",
            &[],
            &[UmlsConcept {
                cui: "C0006142".to_string(),
                name: "Breast Cancer".to_string(),
                semantic_types: vec!["Disease or Syndrome".to_string()],
                xrefs: Vec::new(),
                uri: "https://example.org".to_string(),
            }],
            &[MedlinePlusTopic {
                title: "Breast Cancer".to_string(),
                url: "https://medlineplus.gov/breastcancer.html".to_string(),
                summary_excerpt: "Summary".to_string(),
            }],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::TrialSearch);
        assert!(result.plain_language.is_none());
        assert_eq!(
            result.next_commands[0],
            "biomcp search trial -c \"Breast Cancer\" --limit 5"
        );
    }

    #[test]
    fn treatment_queries_prefer_structured_indication_search() {
        let result = build_result(
            "what drugs treat myasthenia gravis",
            &[],
            &[disease_umls(
                "C0026896",
                "Myasthenia gravis",
                &[("MONDO", "0007192", "Myasthenia gravis")],
            )],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::TreatmentSearch);
        assert_eq!(
            result.next_commands[0],
            "biomcp search drug --indication \"Myasthenia gravis\" --limit 5"
        );
    }

    #[test]
    fn symptom_disease_lookup_query_strips_intent_prefixes() {
        assert_eq!(
            symptom_disease_lookup_query("symptoms of Marfan syndrome"),
            "Marfan syndrome"
        );
        assert_eq!(
            symptom_disease_lookup_query(" Signs of Brugada syndrome "),
            "Brugada syndrome"
        );
        assert_eq!(
            symptom_disease_lookup_query("PHENOTYPES OF Loeys-Dietz syndrome"),
            "Loeys-Dietz syndrome"
        );
    }

    #[test]
    fn symptom_queries_about_disease_prefer_phenotype_section() {
        let result = build_result(
            "symptoms of Marfan syndrome",
            &[],
            &[disease_umls(
                "C0024796",
                "Marfan syndrome",
                &[("MONDO", "0007947", "Marfan syndrome")],
            )],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::SymptomSearch);
        assert_eq!(
            result.next_commands[0],
            "biomcp get disease MONDO:0007947 phenotypes"
        );
    }

    #[test]
    fn drug_safety_queries_beat_treatment_language() {
        let result = build_result(
            "pembrolizumab safety in melanoma",
            &[],
            &[
                drug_umls("C3277863", "pembrolizumab"),
                disease_umls("C0025202", "melanoma", &[("MONDO", "0005105", "melanoma")]),
            ],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::DrugSafety);
        assert_eq!(
            result.next_commands[0],
            "biomcp drug adverse-events pembrolizumab"
        );
        assert_eq!(
            result.next_commands[1],
            "biomcp get drug pembrolizumab safety"
        );
    }

    #[test]
    fn gene_disease_queries_prefer_search_all_orientation() {
        let result = build_result(
            "BRAF melanoma",
            &[hgnc_doc("BRAF", "HGNC:1097", &[])],
            &[disease_umls(
                "C0025202",
                "melanoma",
                &[("MONDO", "0005105", "melanoma")],
            )],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::GeneDiseaseOrientation);
        assert_eq!(
            result.next_commands[0],
            "biomcp search all --gene BRAF --disease \"melanoma\""
        );
    }

    #[test]
    fn gene_function_queries_prefer_get_gene() {
        let result = build_result(
            "what does OPA1 do",
            &[hgnc_doc("OPA1", "HGNC:8141", &[])],
            &[],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::GeneFunction);
        assert_eq!(result.next_commands[0], "biomcp get gene OPA1");
    }

    #[test]
    fn discover_general_gene_topic_adds_filtered_article_search() {
        let result = build_result(
            "CTCF cohesin",
            &[hgnc_doc("CTCF", "HGNC:13723", &[])],
            &[],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::General);
        assert_eq!(result.next_commands[0], "biomcp get gene CTCF");
        assert_eq!(
            result.next_commands[1],
            "biomcp search article -g CTCF -k cohesin --limit 5"
        );
    }

    #[test]
    fn relational_warfarin_query_redirects_instead_of_returning_collocation_noise() {
        let query = "drug classes that interact with warfarin";
        let result = build_result(
            query,
            &[
                ols_doc("chebi", "warfarin", "CHEBI:10033", &[]),
                ols_doc(
                    "ncit",
                    "Interact with Friends Less than I Would Because of Hearing Question",
                    "NCIT:C183192",
                    &["I interact with friends less than I would because of my hearing"],
                ),
                ols_doc(
                    "mesh",
                    "pindone, warfarin drug combination",
                    "MESH:C013793",
                    &[],
                ),
            ],
            &[],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::General);
        assert!(result.concepts.is_empty());
        assert_eq!(
            result.notes,
            vec![format!(
                "`discover` resolves single entities. For relational questions, try: biomcp search all --keyword \"{query}\""
            )]
        );
        assert_eq!(
            result.next_commands,
            vec![format!("biomcp search all --keyword \"{query}\"")]
        );
    }

    #[test]
    fn relational_mef2_query_redirects_when_only_weak_general_hits_remain() {
        let query = "genes regulated by MEF2 in the heart";
        let result = build_result(
            query,
            &[
                ols_doc(
                    "wikipathways",
                    "RalA downstream regulated genes",
                    "WIKIPATHWAYS:WP2290",
                    &[],
                ),
                ols_doc("mesh", "MEF2 Transcription Factors", "MESH:D064326", &[]),
                ols_doc(
                    "ncit",
                    "Metastatic Carcinoma in the Heart",
                    "NCIT:C214847",
                    &[],
                ),
            ],
            &[],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::General);
        assert!(result.concepts.is_empty());
        assert_eq!(
            result.notes,
            vec![format!(
                "`discover` resolves single entities. For relational questions, try: biomcp search all --keyword \"{query}\""
            )]
        );
        assert_eq!(
            result.next_commands,
            vec![format!("biomcp search all --keyword \"{query}\"")]
        );
    }

    #[test]
    fn single_entity_gene_alias_queries_stay_stable_after_general_filtering() {
        let result = build_result(
            "STING",
            &[hgnc_doc("TMEM173", "HGNC:27962", &["STING"])],
            &[],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::General);
        assert_eq!(result.concepts[0].label, "TMEM173");
        assert_eq!(result.next_commands[0], "biomcp get gene TMEM173");
    }

    #[test]
    fn single_entity_disease_queries_stay_stable_after_general_filtering() {
        let result = build_result(
            "scarlet fever",
            &[],
            &[disease_umls(
                "C0037267",
                "Scarlet fever",
                &[("MONDO", "0001234", "Scarlet fever")],
            )],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::General);
        assert_eq!(result.concepts[0].label, "Scarlet fever");
        assert_eq!(
            result.next_commands[0],
            "biomcp get disease \"Scarlet fever\""
        );
    }

    #[test]
    fn empty_results_add_review_article_fallback_note_and_command() {
        let result = build_result("SCENAR therapy", &[], &[], &[], Vec::new());

        assert!(result.concepts.is_empty());
        assert_eq!(
            result.notes,
            vec![
                "No biomedical entities resolved. Try: biomcp search article -k \"SCENAR therapy\" --type review --limit 5"
                    .to_string()
            ]
        );
        assert_eq!(
            result.next_commands,
            vec!["biomcp search article -k \"SCENAR therapy\" --type review --limit 5".to_string()]
        );
    }

    #[test]
    fn low_confidence_variant_adds_note_without_duplicate_article_fallback() {
        let result = build_result("V600E", &[], &[], &[], Vec::new());

        assert_eq!(result.concepts[0].primary_type, DiscoverType::Variant);
        assert_eq!(result.concepts[0].confidence, DiscoverConfidence::LabelOnly);
        assert_eq!(
            result.notes,
            vec!["For broader results: biomcp search article -k V600E".to_string()]
        );
        assert_eq!(
            result.next_commands,
            vec!["biomcp search article -k \"V600E\" --limit 5".to_string()]
        );
    }

    #[test]
    fn low_confidence_noncanonical_paths_append_article_search_once_when_missing() {
        let commands = generate_commands(
            "FAKE1",
            &[test_concept(
                "FAKE1",
                DiscoverType::Gene,
                DiscoverConfidence::LabelOnly,
            )],
            true,
            DiscoverIntent::General,
        );

        assert_eq!(
            commands,
            vec![
                "biomcp search gene -q \"FAKE1\" --limit 10".to_string(),
                "biomcp search article -k FAKE1".to_string(),
            ]
        );
    }

    #[test]
    fn discover_gene_function_with_topic_preserves_meaningful_keyword() {
        let result = build_result(
            "CTCF function cohesin",
            &[hgnc_doc("CTCF", "HGNC:13723", &[])],
            &[],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::GeneFunction);
        assert_eq!(result.next_commands[0], "biomcp get gene CTCF");
        assert_eq!(
            result.next_commands[1],
            "biomcp search article -g CTCF -k cohesin --limit 5"
        );
    }

    #[test]
    fn discover_gene_function_without_topic_keeps_gene_only_article_search() {
        let result = build_result(
            "what does CTCF do",
            &[hgnc_doc("CTCF", "HGNC:13723", &[])],
            &[],
            &[],
            Vec::new(),
        );

        assert_eq!(result.intent, DiscoverIntent::GeneFunction);
        assert_eq!(result.next_commands[0], "biomcp get gene CTCF");
        assert_eq!(
            result.next_commands[1],
            "biomcp search article -g CTCF --limit 5"
        );
        assert!(
            !result
                .next_commands
                .iter()
                .any(|command| command.contains(" -k "))
        );
    }

    #[test]
    fn merge_prefers_shared_xrefs() {
        let result = build_result(
            "Keytruda",
            &[OlsDoc {
                iri: "http://example.org/mesh/C582435".to_string(),
                ontology_name: "mesh".to_string(),
                ontology_prefix: "mesh".to_string(),
                short_form: Some("mesh:C582435".to_string()),
                obo_id: Some("MESH:C582435".to_string()),
                label: "pembrolizumab".to_string(),
                description: Vec::new(),
                exact_synonyms: Vec::new(),
                is_defining_ontology: true,
                doc_type: Some("class".to_string()),
            }],
            &[UmlsConcept {
                cui: "C3277863".to_string(),
                name: "Pembrolizumab".to_string(),
                semantic_types: vec!["Clinical Drug".to_string()],
                xrefs: vec![UmlsXref {
                    vocab: "RXNORM".to_string(),
                    id: "1547545".to_string(),
                    label: "pembrolizumab".to_string(),
                }],
                uri: "https://example.org".to_string(),
            }],
            &[],
            Vec::new(),
        );

        assert_eq!(result.concepts.len(), 1);
        assert_eq!(result.concepts[0].primary_type, DiscoverType::Drug);
    }

    #[test]
    fn umbrella_disease_queries_stay_ambiguous_and_search_oriented() {
        let result = build_result(
            "diabetes",
            &[OlsDoc {
                iri: "http://example.org/doid/9351".to_string(),
                ontology_name: "doid".to_string(),
                ontology_prefix: "doid".to_string(),
                short_form: Some("doid:9351".to_string()),
                obo_id: Some("DOID:9351".to_string()),
                label: "diabetes mellitus".to_string(),
                description: Vec::new(),
                exact_synonyms: vec!["diabetes".to_string()],
                is_defining_ontology: true,
                doc_type: Some("class".to_string()),
            }],
            &[
                UmlsConcept {
                    cui: "C0011854".to_string(),
                    name: "Type 1 diabetes mellitus".to_string(),
                    semantic_types: vec!["Disease or Syndrome".to_string()],
                    xrefs: vec![UmlsXref {
                        vocab: "ICD10CM".to_string(),
                        id: "E10".to_string(),
                        label: "Type 1 diabetes mellitus".to_string(),
                    }],
                    uri: "https://example.org/type1".to_string(),
                },
                UmlsConcept {
                    cui: "C0011860".to_string(),
                    name: "Type 2 diabetes mellitus".to_string(),
                    semantic_types: vec!["Disease or Syndrome".to_string()],
                    xrefs: vec![UmlsXref {
                        vocab: "ICD10CM".to_string(),
                        id: "E11".to_string(),
                        label: "Type 2 diabetes mellitus".to_string(),
                    }],
                    uri: "https://example.org/type2".to_string(),
                },
            ],
            &[],
            Vec::new(),
        );

        assert!(result.ambiguous);
        assert_eq!(
            result.next_commands[0],
            "biomcp search disease -q \"diabetes\" --limit 10"
        );
    }

    #[test]
    fn alias_fallback_classifier_returns_canonical_for_exact_gene_alias() {
        let result = build_result(
            "ERBB1",
            &[OlsDoc {
                iri: "http://example.org/hgnc/3236".to_string(),
                ontology_name: "hgnc".to_string(),
                ontology_prefix: "hgnc".to_string(),
                short_form: Some("hgnc:3236".to_string()),
                obo_id: Some("HGNC:3236".to_string()),
                label: "EGFR".to_string(),
                description: Vec::new(),
                exact_synonyms: vec!["ERBB1".to_string()],
                is_defining_ontology: true,
                doc_type: Some("class".to_string()),
            }],
            &[],
            &[],
            Vec::new(),
        );

        let decision = classify_alias_fallback(&result, DiscoverType::Gene);
        match decision {
            AliasFallbackDecision::Canonical(alias) => {
                assert_eq!(alias.canonical, "EGFR");
                assert_eq!(alias.canonical_id, "HGNC:3236");
                assert_eq!(alias.confidence, DiscoverConfidence::CanonicalId);
                assert_eq!(alias.match_tier, MatchTier::Exact);
                assert_eq!(
                    alias.next_commands,
                    vec!["biomcp get gene EGFR".to_string()]
                );
            }
            other => panic!("expected canonical alias decision, got {other:?}"),
        }
    }

    #[test]
    fn alias_fallback_classifier_returns_ambiguous_for_type_mismatch() {
        let result = build_result(
            "V600E",
            &[OlsDoc {
                iri: "http://example.org/so/0001583".to_string(),
                ontology_name: "so".to_string(),
                ontology_prefix: "so".to_string(),
                short_form: Some("so:0001583".to_string()),
                obo_id: Some("SO:0001583".to_string()),
                label: "V600E".to_string(),
                description: Vec::new(),
                exact_synonyms: vec!["V600E".to_string()],
                is_defining_ontology: true,
                doc_type: Some("class".to_string()),
            }],
            &[],
            &[],
            Vec::new(),
        );

        let decision = classify_alias_fallback(&result, DiscoverType::Gene);
        match decision {
            AliasFallbackDecision::Ambiguous(alias) => {
                assert_eq!(alias.requested_entity, DiscoverType::Gene);
                assert_eq!(alias.query, "V600E");
                assert_eq!(alias.next_commands[0], "biomcp discover V600E");
                assert_eq!(alias.next_commands[1], "biomcp search gene -q V600E");
                assert_eq!(alias.candidates[0].primary_type, DiscoverType::Variant);
            }
            other => panic!("expected ambiguous alias decision, got {other:?}"),
        }
    }

    #[test]
    fn alias_fallback_classifier_returns_none_without_discovery_signal() {
        let result = DiscoverResult {
            query: "notarealalias".to_string(),
            normalized_query: "notarealalias".to_string(),
            concepts: Vec::new(),
            plain_language: None,
            next_commands: Vec::new(),
            notes: Vec::new(),
            ambiguous: false,
            intent: DiscoverIntent::General,
        };

        assert!(matches!(
            classify_alias_fallback(&result, DiscoverType::Gene),
            AliasFallbackDecision::None
        ));
    }
}
