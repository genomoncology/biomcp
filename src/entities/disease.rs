use std::collections::{HashMap, HashSet};
use std::time::Duration;

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::entities::SearchPage;
use crate::entities::drug::{self, DrugSearchFilters};
use crate::entities::trial::{self, TrialSearchFilters, TrialSource};
use crate::error::BioMcpError;
use crate::sources::civic::{CivicClient, CivicContext};
use crate::sources::hpo::HpoClient;
use crate::sources::monarch::{
    MonarchClient, MonarchGeneAssociation, MonarchModelAssociation, MonarchPhenotypeMatch,
};
use crate::sources::mydisease::{MyDiseaseClient, MyDiseaseHit};
use crate::sources::opentargets::OpenTargetsClient;
use crate::sources::reactome::ReactomeClient;
use crate::transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disease {
    pub id: String, // e.g., MONDO:0005105
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub associated_genes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gene_associations: Vec<DiseaseGeneAssociation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_genes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub treatment_landscape: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recruiting_trial_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pathways: Vec<DiseasePathway>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phenotypes: Vec<DiseasePhenotype>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<DiseaseVariantAssociation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<DiseaseModelAssociation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prevalence: Vec<DiseasePrevalenceEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prevalence_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civic: Option<CivicContext>,
    #[serde(default)]
    pub xrefs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseasePathway {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseasePhenotype {
    pub hpo_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_qualifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onset_qualifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex_qualifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_qualifier: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub qualifiers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseGeneAssociation {
    pub gene: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseVariantAssociation {
    pub variant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseModelAssociation {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organism: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseasePrevalenceEvidence {
    pub estimate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseSearchResult {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synonyms_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhenotypeSearchResult {
    pub disease_id: String,
    pub disease_name: String,
    pub score: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DiseaseSearchFilters {
    pub query: Option<String>,
    pub source: Option<String>,
    pub inheritance: Option<String>,
    pub phenotype: Option<String>,
    pub onset: Option<String>,
}

const DISEASE_SECTION_GENES: &str = "genes";
const DISEASE_SECTION_PATHWAYS: &str = "pathways";
const DISEASE_SECTION_PHENOTYPES: &str = "phenotypes";
const DISEASE_SECTION_VARIANTS: &str = "variants";
const DISEASE_SECTION_MODELS: &str = "models";
const DISEASE_SECTION_PREVALENCE: &str = "prevalence";
const DISEASE_SECTION_CIVIC: &str = "civic";
const DISEASE_SECTION_ALL: &str = "all";

pub const DISEASE_SECTION_NAMES: &[&str] = &[
    DISEASE_SECTION_GENES,
    DISEASE_SECTION_PATHWAYS,
    DISEASE_SECTION_PHENOTYPES,
    DISEASE_SECTION_VARIANTS,
    DISEASE_SECTION_MODELS,
    DISEASE_SECTION_PREVALENCE,
    DISEASE_SECTION_CIVIC,
    DISEASE_SECTION_ALL,
];

const OPTIONAL_ENRICHMENT_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Clone, Copy, Default)]
struct DiseaseSections {
    include_genes: bool,
    include_pathways: bool,
    include_phenotypes: bool,
    include_variants: bool,
    include_models: bool,
    include_prevalence: bool,
    include_civic: bool,
}

fn parse_sections(sections: &[String]) -> Result<DiseaseSections, BioMcpError> {
    let mut out = DiseaseSections::default();
    let mut include_all = false;

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }

        match section.as_str() {
            DISEASE_SECTION_GENES => out.include_genes = true,
            DISEASE_SECTION_PATHWAYS => out.include_pathways = true,
            DISEASE_SECTION_PHENOTYPES => out.include_phenotypes = true,
            DISEASE_SECTION_VARIANTS => out.include_variants = true,
            DISEASE_SECTION_MODELS => out.include_models = true,
            DISEASE_SECTION_PREVALENCE => out.include_prevalence = true,
            DISEASE_SECTION_CIVIC => out.include_civic = true,
            DISEASE_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for disease. Available: {}",
                    DISEASE_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.include_genes = true;
        out.include_pathways = true;
        out.include_phenotypes = true;
        out.include_variants = true;
        out.include_models = true;
        out.include_prevalence = true;
        out.include_civic = true;
    }

    Ok(out)
}

fn normalize_disease_id(value: &str) -> Option<String> {
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

fn normalize_disease_text(value: &str) -> String {
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

fn disease_candidate_score(query: &str, candidate_label: &str) -> i32 {
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

fn scored_best_candidate(query: &str, hits: Vec<MyDiseaseHit>) -> Option<MyDiseaseHit> {
    let mut ranked: Vec<(i32, usize, String, MyDiseaseHit)> = hits
        .into_iter()
        .map(|hit| {
            let primary_name = transform::disease::name_from_mydisease_hit(&hit);
            let best_score = disease_candidate_labels(&hit)
                .into_iter()
                .map(|label| disease_candidate_score(query, &label))
                .max()
                .unwrap_or(i32::MIN / 2);
            let normalized_len = normalize_disease_text(&primary_name).len();
            (best_score, normalized_len, hit.id.clone(), hit)
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    ranked.into_iter().next().map(|(_, _, _, hit)| hit)
}

fn resolver_queries(name_or_id: &str) -> Vec<String> {
    let query = name_or_id.trim();
    if query.is_empty() {
        return Vec::new();
    }

    let mut queries = vec![query.to_string()];
    if query.to_ascii_lowercase().contains("cancer") {
        let fallback = query.to_ascii_lowercase().replace("cancer", "carcinoma");
        if !fallback.eq_ignore_ascii_case(query) {
            queries.push(fallback);
        }
    }
    queries
}

async fn resolve_disease_hit_by_name(
    client: &MyDiseaseClient,
    name_or_id: &str,
) -> Result<MyDiseaseHit, BioMcpError> {
    let mut candidates: HashMap<String, MyDiseaseHit> = HashMap::new();
    for query in resolver_queries(name_or_id) {
        let resp = client.query(&query, 15, 0, None, None, None, None).await?;
        for hit in resp.hits {
            candidates.entry(hit.id.clone()).or_insert(hit);
        }
    }

    let best =
        scored_best_candidate(name_or_id, candidates.into_values().collect()).ok_or_else(|| {
            BioMcpError::NotFound {
                entity: "disease".into(),
                id: name_or_id.into(),
                suggestion: format!("Try searching: biomcp search disease -q \"{name_or_id}\""),
            }
        })?;
    Ok(best)
}

async fn add_genes_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    let mut queries: Vec<String> = Vec::new();
    for candidate in [disease.name.trim(), disease.id.trim()] {
        if candidate.is_empty() {
            continue;
        }
        if queries.iter().any(|q| q.eq_ignore_ascii_case(candidate)) {
            continue;
        }
        queries.push(candidate.to_string());
    }
    if queries.is_empty() {
        return Ok(());
    }

    let client = OpenTargetsClient::new()?;
    for query in queries {
        let rows = client.disease_associated_targets(&query, 10).await?;

        for row in rows {
            let symbol = row.symbol.trim();
            if symbol.is_empty() {
                continue;
            }
            if disease
                .associated_genes
                .iter()
                .any(|v| v.eq_ignore_ascii_case(symbol))
            {
                continue;
            }
            disease.associated_genes.push(symbol.to_string());
        }

        if !disease.associated_genes.is_empty() {
            break;
        }
    }

    disease.associated_genes.truncate(10);
    Ok(())
}

async fn add_pathways_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    if disease.associated_genes.is_empty() {
        add_genes_section(disease).await?;
    }
    if disease.associated_genes.is_empty() {
        return Ok(());
    }

    let reactome = ReactomeClient::new()?;
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out: Vec<DiseasePathway> = Vec::new();

    for gene in disease.associated_genes.iter().take(6) {
        let (rows, _) = reactome.search_pathways(gene, 6).await?;
        for row in rows {
            let id = row.id.trim().to_string();
            let name = row.name.trim().to_string();
            if id.is_empty() || name.is_empty() {
                continue;
            }
            if !seen.insert(id.to_ascii_uppercase()) {
                continue;
            }
            out.push(DiseasePathway { id, name });
            if out.len() >= 10 {
                disease.pathways = out;
                return Ok(());
            }
        }
    }

    disease.pathways = out;
    Ok(())
}

fn normalize_hpo_id(value: &str) -> Option<String> {
    let mut id = value.trim().to_ascii_uppercase();
    if id.is_empty() {
        return None;
    }
    id = id.replace('_', ":");
    if !id.starts_with("HP:") {
        return None;
    }
    let suffix = id.trim_start_matches("HP:");
    if suffix.is_empty() || !suffix.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("HP:{suffix}"))
}

async fn add_phenotypes_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    if disease.phenotypes.is_empty() {
        return Ok(());
    }

    let mut ids: Vec<String> = Vec::new();
    for row in &disease.phenotypes {
        if let Some(id) = normalize_hpo_id(&row.hpo_id) {
            ids.push(id);
        }
        if let Some(freq) = row.frequency.as_deref().and_then(normalize_hpo_id) {
            ids.push(freq);
        }
    }
    if ids.is_empty() {
        return Ok(());
    }

    let client = HpoClient::new()?;
    let names = client.resolve_terms(&ids, 20).await?;
    for row in &mut disease.phenotypes {
        if row.name.is_none()
            && let Some(id) = normalize_hpo_id(&row.hpo_id)
        {
            row.name = names.get(&id).cloned();
        }
        if let Some(freq_id) = row.frequency.as_deref().and_then(normalize_hpo_id)
            && let Some(label) = names.get(&freq_id)
        {
            row.frequency = Some(label.clone());
        }
    }
    disease.phenotypes.truncate(20);
    Ok(())
}

fn push_associated_gene(disease: &mut Disease, symbol: &str) {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return;
    }
    if disease
        .associated_genes
        .iter()
        .any(|v| v.eq_ignore_ascii_case(symbol))
    {
        return;
    }
    disease.associated_genes.push(symbol.to_string());
}

fn map_monarch_gene_association(row: MonarchGeneAssociation) -> Option<DiseaseGeneAssociation> {
    let gene = row.gene.trim();
    if gene.is_empty() {
        return None;
    }
    Some(DiseaseGeneAssociation {
        gene: gene.to_string(),
        relationship: row
            .relationship
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        source: row
            .source
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
    })
}

async fn add_monarch_gene_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    let disease_id = match normalize_disease_id(&disease.id) {
        Some(v) => v,
        None => return Ok(()),
    };

    let client = MonarchClient::new()?;
    let rows = client.disease_gene_associations(&disease_id, 50).await?;

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let Some(mapped) = map_monarch_gene_association(row) else {
            continue;
        };

        let key = mapped.gene.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        push_associated_gene(disease, &mapped.gene);
        out.push(mapped);
        if out.len() >= 20 {
            break;
        }
    }

    disease.gene_associations = out;
    disease.associated_genes.truncate(20);
    Ok(())
}

async fn add_monarch_phenotypes(disease: &mut Disease) -> Result<(), BioMcpError> {
    let disease_id = match normalize_disease_id(&disease.id) {
        Some(v) => v,
        None => return Ok(()),
    };

    let client = MonarchClient::new()?;
    let rows = client.disease_phenotypes(&disease_id, 80).await?;
    if rows.is_empty() {
        return Ok(());
    }

    for row in rows {
        let normalized = normalize_hpo_id(&row.hpo_id);
        let Some(hpo_id) = normalized else { continue };

        if let Some(existing) = disease
            .phenotypes
            .iter_mut()
            .find(|p| normalize_hpo_id(&p.hpo_id).is_some_and(|id| id == hpo_id))
        {
            if existing.name.is_none() {
                existing.name = row
                    .label
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string);
            }
            if existing.frequency_qualifier.is_none() {
                existing.frequency_qualifier = row.frequency_qualifier;
            }
            if existing.onset_qualifier.is_none() {
                existing.onset_qualifier = row.onset_qualifier;
            }
            if existing.sex_qualifier.is_none() {
                existing.sex_qualifier = row.sex_qualifier;
            }
            if existing.stage_qualifier.is_none() {
                existing.stage_qualifier = row.stage_qualifier;
            }
            if existing.source.is_none() {
                existing.source = row.source;
            }
            for qualifier in row.qualifiers {
                if qualifier.trim().is_empty() {
                    continue;
                }
                if existing
                    .qualifiers
                    .iter()
                    .any(|v| v.eq_ignore_ascii_case(&qualifier))
                {
                    continue;
                }
                existing.qualifiers.push(qualifier);
            }
            continue;
        }

        disease.phenotypes.push(DiseasePhenotype {
            hpo_id,
            name: row
                .label
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
            evidence: row.relationship,
            frequency: None,
            frequency_qualifier: row.frequency_qualifier,
            onset_qualifier: row.onset_qualifier,
            sex_qualifier: row.sex_qualifier,
            stage_qualifier: row.stage_qualifier,
            qualifiers: row.qualifiers,
            source: row.source,
        });
    }

    disease.phenotypes.truncate(40);
    Ok(())
}

fn looks_like_protein_change(token: &str) -> bool {
    let chars = token.chars().collect::<Vec<_>>();
    if chars.len() < 3 {
        return false;
    }
    chars.first().is_some_and(char::is_ascii_alphabetic)
        && chars.last().is_some_and(char::is_ascii_alphabetic)
        && chars[1..chars.len() - 1].iter().all(char::is_ascii_digit)
}

fn is_hgnc_symbol_candidate(token: &str) -> bool {
    let token = token.trim();
    if token.len() < 2 || token.len() > 15 {
        return false;
    }
    if !token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return false;
    }
    if !token
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
    {
        return false;
    }
    if looks_like_protein_change(token) {
        return false;
    }

    let upper = token.to_ascii_uppercase();
    let excluded = [
        "MUTATION",
        "MUTATIONS",
        "AMPLIFICATION",
        "DELETION",
        "FUSION",
        "WILD",
        "TYPE",
        "LOSS",
        "GAIN",
    ];
    !excluded.contains(&upper.as_str())
}

fn civic_gene_symbol_from_profile(profile: &str) -> Option<String> {
    for token in profile.split(|c: char| !c.is_ascii_alphanumeric() && c != '-') {
        let token = token.trim();
        if token.is_empty() || !is_hgnc_symbol_candidate(token) {
            continue;
        }
        return Some(token.to_ascii_uppercase());
    }
    None
}

async fn augment_genes_with_civic(disease: &mut Disease) -> Result<(), BioMcpError> {
    let Some(query) = disease_query_value(disease) else {
        return Ok(());
    };

    let client = CivicClient::new()?;
    let context = client.by_disease(&query, 25).await?;
    let mut seen = disease
        .gene_associations
        .iter()
        .map(|row| row.gene.to_ascii_lowercase())
        .collect::<HashSet<_>>();

    for symbol in context
        .evidence_items
        .iter()
        .filter_map(|row| civic_gene_symbol_from_profile(&row.molecular_profile))
        .chain(
            context
                .assertions
                .iter()
                .filter_map(|row| civic_gene_symbol_from_profile(&row.molecular_profile)),
        )
    {
        let key = symbol.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        push_associated_gene(disease, &symbol);
        disease.gene_associations.push(DiseaseGeneAssociation {
            gene: symbol,
            relationship: Some("associated with disease".into()),
            source: Some("CIViC".into()),
        });
        if disease.gene_associations.len() >= 20 {
            break;
        }
    }

    disease.gene_associations.truncate(20);
    disease.associated_genes.truncate(20);
    Ok(())
}

async fn add_civic_variants(disease: &mut Disease) -> Result<(), BioMcpError> {
    let Some(query) = disease_query_value(disease) else {
        return Ok(());
    };

    let client = CivicClient::new()?;
    let context = client.by_disease(&query, 25).await?;

    let mut counts: HashMap<String, (String, u32)> = HashMap::new();
    for profile in context
        .evidence_items
        .iter()
        .map(|row| row.molecular_profile.as_str())
        .chain(
            context
                .assertions
                .iter()
                .map(|row| row.molecular_profile.as_str()),
        )
    {
        let profile = profile.trim();
        if profile.is_empty() {
            continue;
        }
        let key = profile.to_ascii_lowercase();
        let entry = counts
            .entry(key)
            .or_insert_with(|| (profile.to_string(), 0));
        entry.1 += 1;
    }

    let mut rows = counts
        .into_values()
        .map(|(variant, evidence_count)| DiseaseVariantAssociation {
            variant,
            relationship: Some("associated with disease".into()),
            source: Some("CIViC".into()),
            evidence_count: Some(evidence_count),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.evidence_count
            .unwrap_or_default()
            .cmp(&a.evidence_count.unwrap_or_default())
            .then_with(|| a.variant.cmp(&b.variant))
    });
    rows.truncate(20);
    disease.variants = rows;
    Ok(())
}

fn map_monarch_model(row: MonarchModelAssociation) -> Option<DiseaseModelAssociation> {
    let model = row.model.trim();
    if model.is_empty() {
        return None;
    }
    Some(DiseaseModelAssociation {
        model: model.to_string(),
        organism: row
            .organism
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        relationship: row.relationship,
        source: row.source,
        evidence_count: row.evidence_count,
    })
}

async fn add_monarch_models(disease: &mut Disease) -> Result<(), BioMcpError> {
    let disease_id = match normalize_disease_id(&disease.id) {
        Some(v) => v,
        None => return Ok(()),
    };

    let client = MonarchClient::new()?;
    let rows = client.disease_models(&disease_id, 50).await?;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let Some(mapped) = map_monarch_model(row) else {
            continue;
        };
        let key = mapped.model.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(mapped);
        if out.len() >= 20 {
            break;
        }
    }
    disease.models = out;
    Ok(())
}

fn disease_query_value(disease: &Disease) -> Option<String> {
    if !disease.name.trim().is_empty() {
        Some(disease.name.trim().to_string())
    } else if !disease.id.trim().is_empty() {
        Some(disease.id.trim().to_string())
    } else {
        None
    }
}

async fn add_treatment_landscape(disease: &mut Disease) -> Result<(), BioMcpError> {
    let Some(query) = disease_query_value(disease) else {
        return Ok(());
    };

    let filters = DrugSearchFilters {
        indication: Some(query),
        ..Default::default()
    };
    let rows = drug::search(&filters, 5).await?;

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for row in rows {
        let name = row.name.trim();
        if name.is_empty() {
            continue;
        }
        let key = name.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(name.to_string());
        if out.len() >= 5 {
            break;
        }
    }

    disease.treatment_landscape = out;
    Ok(())
}

async fn add_recruiting_trial_count(disease: &mut Disease) -> Result<(), BioMcpError> {
    let Some(query) = disease_query_value(disease) else {
        return Ok(());
    };

    let filters = TrialSearchFilters {
        condition: Some(query),
        status: Some("recruiting".to_string()),
        source: TrialSource::ClinicalTrialsGov,
        ..Default::default()
    };

    let (rows, total) = trial::search(&filters, 5, 0).await?;
    disease.recruiting_trial_count = total.or(Some(rows.len() as u32));
    Ok(())
}

async fn add_prevalence_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    let mut queries: Vec<String> = Vec::new();
    for query in [disease.id.trim(), disease.name.trim()] {
        if query.is_empty() {
            continue;
        }
        if queries.iter().any(|q| q.eq_ignore_ascii_case(query)) {
            continue;
        }
        queries.push(query.to_string());
    }
    if queries.is_empty() {
        disease.prevalence.clear();
        disease.prevalence_note = Some("No prevalence data available from OpenTargets.".into());
        return Ok(());
    }

    let client = OpenTargetsClient::new()?;
    for query in queries {
        let rows = client.disease_prevalence(&query, 8).await?;
        if rows.is_empty() {
            continue;
        }
        disease.prevalence = rows
            .into_iter()
            .map(|row| DiseasePrevalenceEvidence {
                estimate: row.estimate,
                context: row.context,
                source: row.source,
            })
            .collect();
        disease.prevalence_note = None;
        return Ok(());
    }

    disease.prevalence.clear();
    disease.prevalence_note = Some("No prevalence data available from OpenTargets.".into());
    Ok(())
}

async fn add_civic_section(disease: &mut Disease) {
    let Some(query) = disease_query_value(disease) else {
        disease.civic = Some(CivicContext::default());
        return;
    };

    let civic_fut = async {
        let client = CivicClient::new()?;
        client.by_disease(&query, 10).await
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, civic_fut).await {
        Ok(Ok(context)) => disease.civic = Some(context),
        Ok(Err(err)) => {
            warn!(query = %query, "CIViC unavailable for disease section: {err}");
            disease.civic = Some(CivicContext::default());
        }
        Err(_) => {
            warn!(
                query = %query,
                timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
                "CIViC disease section timed out"
            );
            disease.civic = Some(CivicContext::default());
        }
    }
}

async fn enrich_base_context(disease: &mut Disease) {
    if let Err(err) = add_genes_section(disease).await {
        warn!("OpenTargets unavailable for disease genes context: {err}");
    }

    disease.top_genes = disease.associated_genes.iter().take(5).cloned().collect();

    if let Err(err) = add_treatment_landscape(disease).await {
        warn!("Drug lookup unavailable for disease treatment landscape: {err}");
    }

    if let Err(err) = add_recruiting_trial_count(disease).await {
        warn!("Trial lookup unavailable for disease recruiting count: {err}");
    }
}

async fn apply_requested_sections(disease: &mut Disease, sections: DiseaseSections) {
    if sections.include_genes {
        if let Err(err) = add_monarch_gene_section(disease).await {
            warn!("Monarch unavailable for disease genes section: {err}");
        }
        if let Err(err) = augment_genes_with_civic(disease).await {
            warn!("CIViC unavailable for disease gene augmentation: {err}");
        }
    }
    if sections.include_pathways
        && let Err(err) = add_pathways_section(disease).await
    {
        warn!("Reactome unavailable for disease pathways section: {err}");
    }
    if sections.include_phenotypes {
        if let Err(err) = add_monarch_phenotypes(disease).await {
            warn!("Monarch unavailable for disease phenotypes section: {err}");
        }
        if let Err(err) = add_phenotypes_section(disease).await {
            warn!("HPO unavailable for disease phenotypes section: {err}");
        }
    }
    if sections.include_variants
        && let Err(err) = add_civic_variants(disease).await
    {
        warn!("CIViC unavailable for disease variants section: {err}");
    }
    if sections.include_models
        && let Err(err) = add_monarch_models(disease).await
    {
        warn!("Monarch unavailable for disease models section: {err}");
    }
    if sections.include_prevalence
        && let Err(err) = add_prevalence_section(disease).await
    {
        warn!("OpenTargets unavailable for disease prevalence section: {err}");
        disease.prevalence.clear();
        disease.prevalence_note = Some("No prevalence data available from OpenTargets.".into());
    }
    if sections.include_civic {
        add_civic_section(disease).await;
    }

    if !sections.include_genes && !sections.include_pathways {
        disease.associated_genes.clear();
        disease.gene_associations.clear();
    }
    if !sections.include_phenotypes {
        disease.phenotypes.clear();
    }
    if !sections.include_variants {
        disease.variants.clear();
    }
    if !sections.include_models {
        disease.models.clear();
    }
    if !sections.include_prevalence {
        disease.prevalence.clear();
        disease.prevalence_note = None;
    }
    if !sections.include_civic {
        disease.civic = None;
    }
}

pub async fn get(name_or_id: &str, sections: &[String]) -> Result<Disease, BioMcpError> {
    let parsed_sections = parse_sections(sections)?;
    let name_or_id = name_or_id.trim();
    if name_or_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Disease name or ID is required. Example: biomcp get disease melanoma".into(),
        ));
    }
    if name_or_id.len() > 512 {
        return Err(BioMcpError::InvalidArgument(
            "Disease name/ID is too long.".into(),
        ));
    }

    let client = MyDiseaseClient::new()?;

    if let Some(id) = normalize_disease_id(name_or_id) {
        let hit = client.get(&id).await?;
        let mut disease = transform::disease::from_mydisease_hit(hit);
        disease.parents = resolve_parent_names(&client, &disease.parents).await;
        enrich_base_context(&mut disease).await;
        apply_requested_sections(&mut disease, parsed_sections).await;
        return Ok(disease);
    }

    let best = resolve_disease_hit_by_name(&client, name_or_id).await?;

    let hit = client.get(&best.id).await?;
    let mut disease = transform::disease::from_mydisease_hit(hit);
    disease.parents = resolve_parent_names(&client, &disease.parents).await;
    enrich_base_context(&mut disease).await;
    apply_requested_sections(&mut disease, parsed_sections).await;
    Ok(disease)
}

async fn resolve_parent_label(client: &MyDiseaseClient, parent_id: &str) -> String {
    let parent_id = parent_id.trim();
    if parent_id.is_empty() {
        return String::new();
    }

    if let Ok(hit) = client.get(parent_id).await {
        let parent_name = transform::disease::name_from_mydisease_hit(&hit);
        if !parent_name.eq_ignore_ascii_case(parent_id) {
            return format!("{parent_name} ({parent_id})");
        }
    }

    if let Ok(resp) = client.query(parent_id, 1, 0, None, None, None, None).await
        && let Some(hit) = resp.hits.first()
    {
        let parent_name = transform::disease::name_from_mydisease_hit(hit);
        if !parent_name.eq_ignore_ascii_case(parent_id) {
            return format!("{parent_name} ({parent_id})");
        }
    }

    parent_id.to_string()
}

async fn resolve_parent_names(client: &MyDiseaseClient, parents: &[String]) -> Vec<String> {
    let mut lookups = Vec::new();
    for parent in parents {
        let parent_id = parent.trim();
        if parent_id.is_empty() {
            continue;
        }
        lookups.push(async move { resolve_parent_label(client, parent_id).await });
    }
    join_all(lookups)
        .await
        .into_iter()
        .filter(|v| !v.is_empty())
        .collect()
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

#[allow(dead_code)]
pub async fn search(
    filters: &DiseaseSearchFilters,
    limit: usize,
) -> Result<Vec<DiseaseSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub async fn search_page(
    filters: &DiseaseSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<DiseaseSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
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
        })?;

    let inheritance = filters
        .inheritance
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let phenotype = filters
        .phenotype
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let onset = filters
        .onset
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());

    let client = MyDiseaseClient::new()?;
    let resp = client
        .query(
            query,
            limit,
            offset,
            filters.source.as_deref(),
            inheritance,
            phenotype,
            onset,
        )
        .await?;
    let prefer_doid = filters
        .source
        .as_deref()
        .map(str::trim)
        .is_some_and(|s| s.eq_ignore_ascii_case("doid"));
    Ok(SearchPage::offset(
        resp.hits
            .iter()
            .filter(|hit| {
                inheritance.is_none_or(|value| inheritance_matches(hit, value))
                    && phenotype.is_none_or(|value| phenotype_matches(hit, value))
                    && onset.is_none_or(|value| onset_matches(hit, value))
            })
            .map(|hit| {
                let mut row = transform::disease::from_mydisease_search_hit(hit);
                if prefer_doid && let Some(doid) = transform::disease::doid_from_mydisease_hit(hit)
                {
                    row.id = doid;
                }
                row
            })
            .collect(),
        Some(resp.total),
    ))
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

fn parse_hpo_query_terms(raw: &str) -> Result<Vec<String>, BioMcpError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "HPO terms are required. Example: biomcp search phenotype \"HP:0001250 HP:0001263\""
                .into(),
        ));
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
        }
    }

    if terms.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "HPO terms are required. Example: biomcp search phenotype \"HP:0001250 HP:0001263\""
                .into(),
        ));
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
    const MAX_RESOLVED_TERMS: usize = 10;

    let raw = raw.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "HPO terms are required. Example: biomcp search phenotype \"HP:0001250 HP:0001263\""
                .into(),
        ));
    }

    if let Ok(terms) = parse_hpo_query_terms(raw) {
        return Ok(terms);
    }

    let queries = split_phenotype_queries(raw);
    if queries.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "HPO terms are required. Example: biomcp search phenotype \"HP:0001250 HP:0001263\""
                .into(),
        ));
    }

    let hpo = HpoClient::new()?;
    let mut resolved = Vec::new();
    let mut seen = HashSet::new();
    for query in queries {
        let ids = hpo.search_term_ids(&query, MAX_RESOLVED_TERMS).await?;
        for id in ids {
            if seen.insert(id.clone()) {
                resolved.push(id);
                if resolved.len() >= MAX_RESOLVED_TERMS {
                    return Ok(resolved);
                }
            }
        }
    }

    if resolved.is_empty() {
        return Err(BioMcpError::InvalidArgument(format!(
            "No HPO terms matched query: {raw}. Try HPO IDs like HP:0001250"
        )));
    }

    Ok(resolved)
}

#[allow(dead_code)]
pub async fn search_phenotype(
    hpo_terms: &str,
    limit: usize,
) -> Result<Vec<PhenotypeSearchResult>, BioMcpError> {
    Ok(search_phenotype_page(hpo_terms, limit, 0).await?.results)
}

pub async fn search_phenotype_page(
    hpo_terms: &str,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<PhenotypeSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let terms = resolve_phenotype_query_terms(hpo_terms).await?;
    let client = MonarchClient::new()?;
    let fetch_limit = limit.saturating_add(offset).max(limit);
    let mut rows = client
        .phenotype_similarity_search(&terms, fetch_limit)
        .await?;
    rows.sort_by(|a, b| b.score.total_cmp(&a.score));
    let total = rows.len();
    rows.truncate(fetch_limit);

    Ok(SearchPage::offset(
        rows.into_iter()
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
            .collect(),
        Some(total),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_disease_id_basic() {
        assert_eq!(
            normalize_disease_id("MONDO:0005105"),
            Some("MONDO:0005105".into())
        );
        assert_eq!(
            normalize_disease_id("mondo:0005105"),
            Some("MONDO:0005105".into())
        );
        assert_eq!(
            normalize_disease_id(" DOID:1909 "),
            Some("DOID:1909".into())
        );
        assert_eq!(normalize_disease_id("lung cancer"), None);
        assert_eq!(normalize_disease_id("MONDO:"), None);
        assert_eq!(normalize_disease_id("HP:0002861"), None);
    }

    #[test]
    fn parse_sections_supports_new_disease_sections() {
        let flags = parse_sections(&[
            "phenotypes".to_string(),
            "variants".to_string(),
            "models".to_string(),
            "prevalence".to_string(),
            "all".to_string(),
        ])
        .expect("sections should parse");
        assert!(flags.include_genes);
        assert!(flags.include_pathways);
        assert!(flags.include_phenotypes);
        assert!(flags.include_variants);
        assert!(flags.include_models);
        assert!(flags.include_prevalence);
        assert!(flags.include_civic);
    }

    #[test]
    fn parse_hpo_query_terms_requires_valid_ids() {
        let parsed = parse_hpo_query_terms("HP:0001250 HP:0001263").expect("valid terms");
        assert_eq!(parsed.len(), 2);
        assert!(parse_hpo_query_terms("NOT_AN_HPO").is_err());
    }

    #[test]
    fn disease_candidate_score_prefers_broad_match_over_subtype() {
        let broad = disease_candidate_score("breast cancer", "breast carcinoma");
        let subtype = disease_candidate_score("breast cancer", "sporadic breast carcinoma");
        assert!(broad > subtype);
    }

    #[test]
    fn resolver_queries_adds_carcinoma_fallback_for_cancer_terms() {
        let queries = resolver_queries("breast cancer");
        assert!(queries.iter().any(|q| q == "breast cancer"));
        assert!(queries.iter().any(|q| q == "breast carcinoma"));
    }

    #[test]
    fn civic_gene_symbol_extraction_ignores_protein_change_tokens() {
        assert_eq!(
            civic_gene_symbol_from_profile("BRAF V600E").as_deref(),
            Some("BRAF")
        );
        assert_eq!(
            civic_gene_symbol_from_profile("V600E BRAF").as_deref(),
            Some("BRAF")
        );
        assert_eq!(civic_gene_symbol_from_profile("V600E"), None);
    }
}
