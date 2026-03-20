use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;

use crate::sources::medlineplus::MedlinePlusTopic;
use crate::sources::ols4::OlsDoc;
use crate::sources::umls::{UmlsConcept, UmlsXref};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DiscoverResult {
    pub query: String,
    pub normalized_query: String,
    pub concepts: Vec<DiscoverConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plain_language: Option<PlainLanguageTopic>,
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
    General,
    TrialSearch,
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

pub(crate) fn build_result(
    query: &str,
    ols_docs: &[OlsDoc],
    umls_concepts: &[UmlsConcept],
    medline_topics: &[MedlinePlusTopic],
    notes: Vec<String>,
) -> DiscoverResult {
    let normalized_query = normalize_query(query);
    let intent = detect_intent(&normalized_query);

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
    let ambiguous = is_ambiguous(&concepts);
    let plain_language = select_plain_language(&concepts, medline_topics, intent);
    let next_commands = generate_commands(query, &concepts, ambiguous, intent);

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
    let primary_type = infer_ols_type(doc, query);
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

fn normalize_primary_id(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
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

fn detect_intent(normalized_query: &str) -> DiscoverIntent {
    let has_trial_token = normalized_query.split_whitespace().any(|token| {
        matches!(
            token,
            "trial" | "trials" | "study" | "studies" | "recruiting" | "recruitment"
        )
    });
    if has_trial_token {
        DiscoverIntent::TrialSearch
    } else {
        DiscoverIntent::General
    }
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
    let Some(top) = concepts.first() else {
        return commands;
    };

    if intent == DiscoverIntent::TrialSearch {
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
        return commands;
    }

    match top.primary_type {
        DiscoverType::Gene if !ambiguous => commands.push(format!("biomcp get gene {}", top.label)),
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
                "biomcp search article -k \"{}\" --limit 5",
                query.trim()
            ));
        }
    }

    dedupe_strings(commands)
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

#[cfg(test)]
mod tests {
    use super::{DiscoverIntent, DiscoverType, build_result};
    use crate::sources::medlineplus::MedlinePlusTopic;
    use crate::sources::ols4::OlsDoc;
    use crate::sources::umls::{UmlsConcept, UmlsXref};

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

        assert_eq!(result.intent, DiscoverIntent::General);
        assert_eq!(result.concepts[0].primary_type, DiscoverType::Symptom);
        assert!(result.plain_language.is_some());
        assert!(
            result
                .next_commands
                .contains(&"biomcp search disease -q \"chest pain\" --limit 10".to_string())
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
}
