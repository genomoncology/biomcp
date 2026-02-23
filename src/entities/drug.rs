use std::collections::{HashMap, HashSet};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::chembl::ChemblClient;
use crate::sources::civic::{CivicClient, CivicContext};
use crate::sources::mychem::{
    MYCHEM_FIELDS_GET, MYCHEM_FIELDS_SEARCH, MyChemClient, MyChemHit, MyChemNdcField,
};
use crate::sources::openfda::{DrugsFdaResult, OpenFdaClient, OpenFdaResponse};
use crate::sources::opentargets::OpenTargetsClient;
use crate::transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drug {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drugbank_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chembl_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unii: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drug_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mechanism: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mechanisms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_date: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub brand_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub indications: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interactions: Vec<DrugInteraction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pharm_classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_adverse_events: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<DrugLabel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortage: Option<Vec<DrugShortageEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approvals: Option<Vec<DrugApproval>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civic: Option<CivicContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugInteraction {
    pub drug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugLabel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indications: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugShortageEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_posting_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugApproval {
    pub application_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub openfda_brand_names: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub openfda_generic_names: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub products: Vec<DrugApprovalProduct>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub submissions: Vec<DrugApprovalSubmission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugApprovalProduct {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosage_form: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketing_status: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_ingredients: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugApprovalSubmission {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugSearchResult {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drugbank_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drug_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mechanism: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DrugSearchFilters {
    pub query: Option<String>,
    pub target: Option<String>,
    pub indication: Option<String>,
    pub mechanism: Option<String>,
    pub drug_type: Option<String>,
    pub atc: Option<String>,
    pub pharm_class: Option<String>,
    pub interactions: Option<String>,
}

const DRUG_SECTION_LABEL: &str = "label";
const DRUG_SECTION_SHORTAGE: &str = "shortage";
const DRUG_SECTION_TARGETS: &str = "targets";
const DRUG_SECTION_INDICATIONS: &str = "indications";
const DRUG_SECTION_INTERACTIONS: &str = "interactions";
const DRUG_SECTION_CIVIC: &str = "civic";
const DRUG_SECTION_APPROVALS: &str = "approvals";
const DRUG_SECTION_ALL: &str = "all";

pub const DRUG_SECTION_NAMES: &[&str] = &[
    DRUG_SECTION_LABEL,
    DRUG_SECTION_SHORTAGE,
    DRUG_SECTION_TARGETS,
    DRUG_SECTION_INDICATIONS,
    DRUG_SECTION_INTERACTIONS,
    DRUG_SECTION_CIVIC,
    DRUG_SECTION_APPROVALS,
    DRUG_SECTION_ALL,
];

const OPTIONAL_SAFETY_TIMEOUT: Duration = Duration::from_secs(8);

fn normalize_query_summary(filters: &DrugSearchFilters) -> String {
    if filters.target.is_none()
        && filters.indication.is_none()
        && filters.mechanism.is_none()
        && filters.drug_type.is_none()
        && filters.atc.is_none()
        && filters.pharm_class.is_none()
        && filters.interactions.is_none()
        && let Some(q) = filters
            .query
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
    {
        return q.to_string();
    }

    let mut parts: Vec<String> = Vec::new();
    if let Some(q) = filters
        .query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(q.to_string());
    }
    if let Some(v) = filters
        .target
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("target={v}"));
    }
    if let Some(v) = filters
        .indication
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("indication={v}"));
    }
    if let Some(v) = filters
        .mechanism
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("mechanism={v}"));
    }
    if let Some(v) = filters
        .drug_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("type={v}"));
    }
    if let Some(v) = filters
        .atc
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("atc={v}"));
    }
    if let Some(v) = filters
        .pharm_class
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("pharm_class={v}"));
    }
    if let Some(v) = filters
        .interactions
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("interactions={v}"));
    }

    parts.join(", ")
}

fn build_mychem_query(filters: &DrugSearchFilters) -> Result<String, BioMcpError> {
    let mut terms: Vec<String> = Vec::new();

    if let Some(q) = filters
        .query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(MyChemClient::escape_query_value(q));
    }

    if let Some(target) = filters
        .target
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        // Prefer GtoPdb targets for consistent gene symbols.
        terms.push(format!(
            "gtopdb.interaction_targets.symbol:{}",
            MyChemClient::escape_query_value(target)
        ));
    }

    if let Some(ind) = filters
        .indication
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if ind.chars().any(|c| c.is_whitespace()) {
            terms.push(format!(
                "drugcentral.drug_use.indication.concept_name:\"{}\"",
                MyChemClient::escape_query_value(ind)
            ));
        } else {
            terms.push(format!(
                "drugcentral.drug_use.indication.concept_name:*{}*",
                MyChemClient::escape_query_value(ind)
            ));
        }
    }

    if let Some(mechanism) = filters
        .mechanism
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let escaped = MyChemClient::escape_query_value(mechanism);
        let tokens = mechanism
            .split_whitespace()
            .map(MyChemClient::escape_query_value)
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();

        let mut clauses = vec![
            format!("chembl.drug_mechanisms.action_type:\"{escaped}\""),
            format!("ndc.pharm_classes:\"{escaped}\""),
        ];

        if !tokens.is_empty() {
            let chembl_all_tokens = tokens
                .iter()
                .map(|token| format!("chembl.drug_mechanisms.action_type:*{token}*"))
                .collect::<Vec<_>>()
                .join(" AND ");
            let ndc_all_tokens = tokens
                .iter()
                .map(|token| format!("ndc.pharm_classes:*{token}*"))
                .collect::<Vec<_>>()
                .join(" AND ");
            clauses.push(format!("({chembl_all_tokens})"));
            clauses.push(format!("({ndc_all_tokens})"));
        }

        terms.push(format!("({})", clauses.join(" OR ")));
    }

    if let Some(t) = filters
        .drug_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let t_norm = t.to_ascii_lowercase();
        let mapped = match t_norm.as_str() {
            "biologic" | "biologics" | "antibody" => Some("Antibody".to_string()),
            "small-molecule" | "small_molecule" | "small molecule" | "small" => {
                Some("Small molecule".to_string())
            }
            _ => None,
        };

        let value = mapped.unwrap_or_else(|| t.to_string());
        if value.chars().any(|c| c.is_whitespace()) {
            terms.push(format!(
                "chembl.molecule_type:\"{}\"",
                MyChemClient::escape_query_value(&value)
            ));
        } else {
            terms.push(format!(
                "chembl.molecule_type:{}",
                MyChemClient::escape_query_value(&value)
            ));
        }
    }

    if let Some(atc) = filters
        .atc
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(format!(
            "chembl.atc_classifications:{}",
            MyChemClient::escape_query_value(atc)
        ));
    }

    if let Some(pharm_class) = filters
        .pharm_class
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let escaped = MyChemClient::escape_query_value(pharm_class);
        terms.push(format!(
            "(drugcentral.pharmacology_class:\"{escaped}\" OR ndc.pharm_classes:\"{escaped}\")"
        ));
    }

    if let Some(interactions) = filters
        .interactions
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let escaped = MyChemClient::escape_query_value(interactions);
        terms.push(format!("drugbank.drug_interactions.name:\"{escaped}\""));
    }

    if terms.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter is required. Example: biomcp search drug -q pembrolizumab".into(),
        ));
    }

    Ok(terms.join(" AND "))
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

    let client = MyChemClient::new()?;
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
        .query_with_fields(&q, fetch_limit, offset, MYCHEM_FIELDS_SEARCH)
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

    Ok(SearchPage::offset(out, Some(resp.total)))
}

fn hit_mentions_target(hit: &MyChemHit, target: &str) -> bool {
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

fn hit_mentions_mechanism(hit: &MyChemHit, mechanism: &str) -> bool {
    let mechanism = mechanism.trim().to_ascii_lowercase();
    if mechanism.is_empty() {
        return false;
    }
    let tokens = mechanism
        .split_whitespace()
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();

    if let Some(chembl) = hit.chembl.as_ref() {
        for row in &chembl.drug_mechanisms {
            let Some(action) = row.action_type.as_deref() else {
                continue;
            };
            if text_matches_mechanism(action, &mechanism, &tokens) {
                return true;
            }
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

#[derive(Debug, Clone, Copy, Default)]
struct DrugSections {
    include_label: bool,
    include_shortage: bool,
    include_targets: bool,
    include_indications: bool,
    include_interactions: bool,
    include_civic: bool,
    include_approvals: bool,
}

fn parse_sections(sections: &[String]) -> Result<DrugSections, BioMcpError> {
    let mut out = DrugSections::default();
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
            DRUG_SECTION_LABEL => {
                out.include_label = true;
            }
            DRUG_SECTION_SHORTAGE => out.include_shortage = true,
            DRUG_SECTION_TARGETS => out.include_targets = true,
            DRUG_SECTION_INDICATIONS => out.include_indications = true,
            DRUG_SECTION_INTERACTIONS => out.include_interactions = true,
            DRUG_SECTION_CIVIC => out.include_civic = true,
            DRUG_SECTION_APPROVALS => out.include_approvals = true,
            DRUG_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for drug. Available: {}",
                    DRUG_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.include_label = true;
        out.include_shortage = true;
        out.include_targets = true;
        out.include_indications = true;
        out.include_interactions = true;
        out.include_civic = true;
        out.include_approvals = true;
    }

    Ok(out)
}

fn normalize_date_yyyymmdd(value: Option<&str>) -> Option<String> {
    let v = value?.trim();
    if v.len() != 8 || !v.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("{}-{}-{}", &v[0..4], &v[4..6], &v[6..8]))
}

fn label_text(value: Option<&serde_json::Value>) -> Option<String> {
    let value = value?;
    let text = match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
        _ => String::new(),
    };
    let text = text.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

fn truncate_with_note(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let truncated = value.chars().take(max_chars).collect::<String>();
    let total = value.chars().count();
    format!("{truncated}\n\n(truncated, {total} chars total)")
}

fn extract_inline_label(label_response: &serde_json::Value) -> Option<DrugLabel> {
    const LABEL_MAX_CHARS: usize = 2000;

    let top = label_response
        .get("results")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())?;

    let indications = label_text(top.get("indications_and_usage"))
        .map(|v| truncate_with_note(&v, LABEL_MAX_CHARS));
    let warnings = label_text(top.get("warnings_and_cautions"))
        .map(|v| truncate_with_note(&v, LABEL_MAX_CHARS));
    let dosage = label_text(top.get("dosage_and_administration"))
        .map(|v| truncate_with_note(&v, LABEL_MAX_CHARS));

    if indications.is_none() && warnings.is_none() && dosage.is_none() {
        return None;
    }

    Some(DrugLabel {
        indications,
        warnings,
        dosage,
    })
}

fn extract_openfda_values(label_response: &serde_json::Value, key: &str) -> Vec<String> {
    let Some(results) = label_response.get("results").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for result in results {
        let Some(top) = result.get("openfda").and_then(|v| v.get(key)) else {
            continue;
        };
        let values: Vec<String> = match top {
            serde_json::Value::String(s) => {
                let s = s.trim();
                if s.is_empty() {
                    Vec::new()
                } else {
                    vec![s.to_string()]
                }
            }
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
                .collect(),
            _ => Vec::new(),
        };
        for value in values {
            let key = value.to_ascii_lowercase();
            if !seen.insert(key) {
                continue;
            }
            out.push(value);
        }
    }
    out
}

fn normalize_route(route: &str) -> String {
    let route = route.trim().to_ascii_lowercase();
    if route.is_empty() {
        return String::new();
    }
    if matches!(
        route.as_str(),
        "iv" | "intravenous" | "intravenous injection" | "intravenous infusion"
    ) {
        return "IV".to_string();
    }
    if matches!(route.as_str(), "subcutaneous" | "sub-cutaneous") {
        return "subcutaneous".to_string();
    }
    route
}

fn maybe_brand_alias(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || !trimmed.contains(' ') {
        return None;
    }
    let first = trimmed.split_whitespace().next()?;
    if first.len() < 4 {
        return None;
    }
    if first
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
    {
        return Some(first.to_string());
    }
    None
}

fn route_rank(route: &str) -> usize {
    if route == "IV" {
        0
    } else if route == "subcutaneous" {
        1
    } else if route == "oral" {
        2
    } else {
        3
    }
}

fn apply_openfda_metadata(drug: &mut Drug, label_response: &serde_json::Value) {
    let mut brand_names: Vec<String> = extract_openfda_values(label_response, "brand_name");
    brand_names.extend(
        brand_names
            .iter()
            .filter_map(|name| maybe_brand_alias(name))
            .collect::<Vec<_>>(),
    );
    brand_names.extend(drug.brand_names.clone());
    let mut seen: HashSet<String> = HashSet::new();
    let mut merged: Vec<String> = Vec::new();
    for name in brand_names {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        merged.push(trimmed.to_string());
        if merged.len() >= 5 {
            break;
        }
    }
    if !merged.is_empty() {
        drug.brand_names = merged;
    }

    let mut routes = extract_openfda_values(label_response, "route")
        .into_iter()
        .map(|v| normalize_route(&v))
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();
    if let Some(existing) = drug.route.as_deref() {
        let normalized = normalize_route(existing);
        if !normalized.is_empty() {
            routes.push(normalized);
        }
    }
    routes.sort_by(|a, b| route_rank(a).cmp(&route_rank(b)).then_with(|| a.cmp(b)));
    routes.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    if !routes.is_empty() {
        drug.route = Some(routes.join(", "));
    }
}

fn trim_nonempty(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn dedupe_trimmed_casefold(values: impl IntoIterator<Item = String>, max: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let key = value.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(value.to_string());
        if out.len() >= max {
            break;
        }
    }
    out
}

fn map_drugsfda_approvals(resp: OpenFdaResponse<DrugsFdaResult>) -> Vec<DrugApproval> {
    let mut out: Vec<DrugApproval> = Vec::new();
    let mut seen_apps: HashSet<String> = HashSet::new();

    for row in resp.results {
        let Some(application_number) = row
            .application_number
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
        else {
            continue;
        };
        if !seen_apps.insert(application_number.to_ascii_lowercase()) {
            continue;
        }

        let sponsor_name = row
            .sponsor_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);

        let (openfda_brand_names, openfda_generic_names) = row
            .openfda
            .map(|meta| {
                (
                    dedupe_trimmed_casefold(meta.brand_name, 10),
                    dedupe_trimmed_casefold(meta.generic_name, 10),
                )
            })
            .unwrap_or_default();

        let mut products: Vec<DrugApprovalProduct> = row
            .products
            .into_iter()
            .filter_map(|product| {
                let brand_name = trim_nonempty(product.brand_name);
                let dosage_form = trim_nonempty(product.dosage_form);
                let route = trim_nonempty(product.route).map(|v| normalize_route(&v));
                let marketing_status = trim_nonempty(product.marketing_status);
                let active_ingredients = dedupe_trimmed_casefold(
                    product.active_ingredients.into_iter().filter_map(|ai| {
                        let name = ai.name.as_deref().map(str::trim).filter(|v| !v.is_empty());
                        let strength = ai
                            .strength
                            .as_deref()
                            .map(str::trim)
                            .filter(|v| !v.is_empty());
                        match (name, strength) {
                            (Some(name), Some(strength)) => Some(format!("{name} ({strength})")),
                            (Some(name), None) => Some(name.to_string()),
                            _ => None,
                        }
                    }),
                    6,
                );

                if brand_name.is_none()
                    && dosage_form.is_none()
                    && route.is_none()
                    && marketing_status.is_none()
                    && active_ingredients.is_empty()
                {
                    return None;
                }

                Some(DrugApprovalProduct {
                    brand_name,
                    dosage_form,
                    route,
                    marketing_status,
                    active_ingredients,
                })
            })
            .collect();

        products.truncate(6);

        let mut submissions: Vec<DrugApprovalSubmission> = row
            .submissions
            .into_iter()
            .filter_map(|submission| {
                let submission_type = trim_nonempty(submission.submission_type);
                let submission_number = trim_nonempty(submission.submission_number);
                let status = trim_nonempty(submission.submission_status);
                let status_date =
                    normalize_date_yyyymmdd(submission.submission_status_date.as_deref());

                if submission_type.is_none()
                    && submission_number.is_none()
                    && status.is_none()
                    && status_date.is_none()
                {
                    return None;
                }

                Some(DrugApprovalSubmission {
                    submission_type,
                    submission_number,
                    status,
                    status_date,
                })
            })
            .collect();

        submissions.sort_by(|a, b| b.status_date.cmp(&a.status_date));
        submissions.truncate(8);

        out.push(DrugApproval {
            application_number,
            sponsor_name,
            openfda_brand_names,
            openfda_generic_names,
            products,
            submissions,
        });
        if out.len() >= 8 {
            break;
        }
    }

    out
}

async fn fetch_shortage_entries(drug_name: &str) -> Result<Vec<DrugShortageEntry>, BioMcpError> {
    let drug_name = drug_name.trim();
    if drug_name.is_empty() {
        return Ok(Vec::new());
    }

    let escaped = OpenFdaClient::escape_query_value(drug_name);
    let q = if drug_name.chars().any(|c| c.is_whitespace()) {
        format!(
            "generic_name:\"{escaped}\" OR openfda.generic_name:\"{escaped}\" OR openfda.brand_name:\"{escaped}\""
        )
    } else {
        format!(
            "generic_name:*{escaped}* OR openfda.generic_name:*{escaped}* OR openfda.brand_name:*{escaped}*"
        )
    };

    let client = OpenFdaClient::new()?;
    let resp = client.shortage_search(&q, 5, 0).await?;
    let Some(resp) = resp else {
        return Ok(Vec::new());
    };

    let out = resp
        .results
        .into_iter()
        .map(|r| DrugShortageEntry {
            status: r
                .status
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            availability: r
                .availability
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            company_name: r
                .company_name
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            generic_name: r
                .generic_name
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            related_info: r
                .related_info
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            update_date: normalize_date_yyyymmdd(r.update_date.as_deref()),
            initial_posting_date: normalize_date_yyyymmdd(r.initial_posting_date.as_deref()),
        })
        .collect::<Vec<_>>();

    Ok(out)
}

fn extract_top_adverse_events(
    resp: &crate::sources::openfda::OpenFdaResponse<crate::sources::openfda::FaersEventResult>,
) -> Vec<String> {
    let mut counts: HashMap<String, (String, usize)> = HashMap::new();
    for report in &resp.results {
        let Some(patient) = report.patient.as_ref() else {
            continue;
        };
        for reaction in &patient.reaction {
            let Some(term) = reaction
                .reactionmeddrapt
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            let key = term.to_ascii_lowercase();
            let entry = counts.entry(key).or_insert_with(|| (term.to_string(), 0));
            entry.1 += 1;
        }
    }

    let mut ranked: Vec<(String, usize)> = counts.into_values().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(3);
    ranked.into_iter().map(|(label, _)| label).collect()
}

async fn fetch_top_adverse_events(drug_name: &str) -> Result<Vec<String>, BioMcpError> {
    let drug_name = drug_name.trim();
    if drug_name.is_empty() {
        return Ok(Vec::new());
    }

    let escaped = OpenFdaClient::escape_query_value(drug_name);
    let q = format!(
        "(patient.drug.openfda.generic_name:\"{escaped}\" OR patient.drug.openfda.brand_name:\"{escaped}\" OR patient.drug.medicinalproduct:\"{escaped}\") AND patient.drug.drugcharacterization:1"
    );
    let client = OpenFdaClient::new()?;
    let resp = client.faers_search(&q, 50, 0).await?;
    let Some(resp) = resp else {
        return Ok(Vec::new());
    };
    Ok(extract_top_adverse_events(&resp))
}

fn merge_unique_casefold(dst: &mut Vec<String>, values: impl IntoIterator<Item = String>) {
    let mut seen: HashSet<String> = dst.iter().map(|v| v.to_ascii_lowercase()).collect();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let key = value.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        dst.push(value.to_string());
    }
}

async fn enrich_targets(drug: &mut Drug) {
    let Some(chembl_id) = drug.chembl_id.as_deref() else {
        return;
    };

    match ChemblClient::new() {
        Ok(client) => match client.drug_targets(chembl_id, 15).await {
            Ok(rows) => {
                let targets = rows
                    .iter()
                    .filter(|row| !row.target.eq_ignore_ascii_case("Unknown target"))
                    .map(|row| row.target.clone())
                    .collect::<Vec<_>>();
                merge_unique_casefold(&mut drug.targets, targets);

                let mechanisms = rows
                    .into_iter()
                    .filter(|row| !row.target.eq_ignore_ascii_case("Unknown target"))
                    .map(|row| {
                        row.mechanism
                            .unwrap_or_else(|| format!("{} of {}", row.action, row.target))
                    })
                    .collect::<Vec<_>>();
                merge_unique_casefold(&mut drug.mechanisms, mechanisms);
            }
            Err(err) => warn!("ChEMBL unavailable for drug targets section: {err}"),
        },
        Err(err) => warn!("ChEMBL client init failed: {err}"),
    }

    match OpenTargetsClient::new() {
        Ok(client) => match client.drug_sections(chembl_id, 15).await {
            Ok(sections) => {
                let targets = sections
                    .targets
                    .into_iter()
                    .map(|t| t.approved_symbol)
                    .collect::<Vec<_>>();
                merge_unique_casefold(&mut drug.targets, targets);
            }
            Err(err) => warn!("OpenTargets unavailable for drug targets section: {err}"),
        },
        Err(err) => warn!("OpenTargets client init failed: {err}"),
    }

    drug.targets.truncate(12);
    if !drug.mechanisms.is_empty() {
        drug.mechanism = drug.mechanisms.first().cloned();
    }
    drug.mechanisms.truncate(6);
}

async fn enrich_indications(drug: &mut Drug) {
    let Some(chembl_id) = drug.chembl_id.as_deref() else {
        return;
    };

    match OpenTargetsClient::new() {
        Ok(client) => match client.drug_sections(chembl_id, 15).await {
            Ok(sections) => {
                let indications = sections
                    .indications
                    .into_iter()
                    .map(|i| match i.max_phase {
                        Some(phase) => format!("{} (max phase {:.0})", i.disease_name, phase),
                        None => i.disease_name,
                    })
                    .collect::<Vec<_>>();
                merge_unique_casefold(&mut drug.indications, indications);
            }
            Err(err) => warn!("OpenTargets unavailable for drug indications section: {err}"),
        },
        Err(err) => warn!("OpenTargets client init failed: {err}"),
    }

    drug.indications.truncate(12);
}

async fn add_civic_section(drug: &mut Drug) {
    let name = drug.name.trim();
    if name.is_empty() {
        drug.civic = Some(CivicContext::default());
        return;
    }

    let civic_fut = async {
        let client = CivicClient::new()?;
        client.by_therapy(name, 10).await
    };

    match tokio::time::timeout(OPTIONAL_SAFETY_TIMEOUT, civic_fut).await {
        Ok(Ok(context)) => drug.civic = Some(context),
        Ok(Err(err)) => {
            warn!(drug = %drug.name, "CIViC unavailable for drug section: {err}");
            drug.civic = Some(CivicContext::default());
        }
        Err(_) => {
            warn!(
                drug = %drug.name,
                timeout_secs = OPTIONAL_SAFETY_TIMEOUT.as_secs(),
                "CIViC drug section timed out"
            );
            drug.civic = Some(CivicContext::default());
        }
    }
}

async fn add_approvals_section(drug: &mut Drug) {
    let name = drug.name.trim();
    if name.is_empty() {
        drug.approvals = Some(Vec::new());
        return;
    }

    let escaped = OpenFdaClient::escape_query_value(name);
    let query = if name.chars().any(|c| c.is_whitespace()) {
        format!(
            "openfda.generic_name:\"{escaped}\" OR openfda.brand_name:\"{escaped}\" OR products.brand_name:\"{escaped}\""
        )
    } else {
        format!(
            "openfda.generic_name:*{escaped}* OR openfda.brand_name:*{escaped}* OR products.brand_name:*{escaped}*"
        )
    };

    let approvals_fut = async {
        let client = OpenFdaClient::new()?;
        client.drugsfda_search(&query, 8, 0).await
    };

    match tokio::time::timeout(OPTIONAL_SAFETY_TIMEOUT, approvals_fut).await {
        Ok(Ok(resp)) => {
            let approvals = resp.map(map_drugsfda_approvals).unwrap_or_default();
            drug.approvals = Some(approvals);
        }
        Ok(Err(err)) => {
            warn!(drug = %drug.name, "OpenFDA Drugs@FDA unavailable: {err}");
            drug.approvals = Some(Vec::new());
        }
        Err(_) => {
            warn!(
                drug = %drug.name,
                timeout_secs = OPTIONAL_SAFETY_TIMEOUT.as_secs(),
                "OpenFDA Drugs@FDA section timed out"
            );
            drug.approvals = Some(Vec::new());
        }
    }
}

pub async fn get(name: &str, sections: &[String]) -> Result<Drug, BioMcpError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Drug name is required. Example: biomcp get drug pembrolizumab".into(),
        ));
    }
    if name.len() > 256 {
        return Err(BioMcpError::InvalidArgument(
            "Drug name is too long.".into(),
        ));
    }

    let section_flags = parse_sections(sections)?;

    let client = MyChemClient::new()?;
    let resp = client
        .query_with_fields(name, 25, 0, MYCHEM_FIELDS_GET)
        .await?;

    if resp.hits.is_empty() {
        return Err(BioMcpError::NotFound {
            entity: "drug".into(),
            id: name.to_string(),
            suggestion: format!("Try searching: biomcp search drug -q \"{name}\""),
        });
    }

    let selected = transform::drug::select_hits_for_name(&resp.hits, name);
    let mut drug = transform::drug::merge_mychem_hits(&selected, name);

    let mut label_response_opt: Option<serde_json::Value> = None;
    match OpenFdaClient::new() {
        Ok(client) => match client.label_search(&drug.name).await {
            Ok(v) => label_response_opt = v,
            Err(err) => {
                if section_flags.include_label {
                    return Err(err);
                }
            }
        },
        Err(err) => {
            if section_flags.include_label {
                return Err(err);
            }
        }
    }

    if let Some(label_response) = label_response_opt.as_ref() {
        apply_openfda_metadata(&mut drug, label_response);
        if section_flags.include_label {
            drug.label = extract_inline_label(label_response);
        }
    }

    if section_flags.include_shortage {
        drug.shortage = Some(fetch_shortage_entries(&drug.name).await?);
    }

    if section_flags.include_targets {
        enrich_targets(&mut drug).await;
    }

    if section_flags.include_indications {
        enrich_indications(&mut drug).await;
    }

    if !section_flags.include_interactions {
        drug.interactions.clear();
    }
    if section_flags.include_civic {
        add_civic_section(&mut drug).await;
    } else {
        drug.civic = None;
    }
    if section_flags.include_approvals {
        add_approvals_section(&mut drug).await;
    } else {
        drug.approvals = None;
    }

    match tokio::time::timeout(
        OPTIONAL_SAFETY_TIMEOUT,
        fetch_top_adverse_events(&drug.name),
    )
    .await
    {
        Ok(Ok(events)) => {
            drug.top_adverse_events = events;
        }
        Ok(Err(err)) => {
            warn!(
                drug = %drug.name,
                "OpenFDA adverse-event preview unavailable: {err}"
            );
        }
        Err(_) => {
            warn!(
                drug = %drug.name,
                timeout_secs = OPTIONAL_SAFETY_TIMEOUT.as_secs(),
                "OpenFDA adverse-event preview timed out"
            );
        }
    }

    Ok(drug)
}

pub fn search_query_summary(filters: &DrugSearchFilters) -> String {
    normalize_query_summary(filters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_mychem_query_requires_at_least_one_filter() {
        let filters = DrugSearchFilters::default();
        let err = build_mychem_query(&filters).unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[test]
    fn build_mychem_query_includes_target_and_mechanism_filters() {
        let filters = DrugSearchFilters {
            query: Some("pembrolizumab".into()),
            target: Some("BRAF".into()),
            indication: None,
            mechanism: Some("inhibitor".into()),
            drug_type: Some("small molecule".into()),
            atc: None,
            pharm_class: None,
            interactions: None,
        };
        let q = build_mychem_query(&filters).unwrap();
        assert!(q.contains("pembrolizumab"));
        assert!(q.contains("gtopdb.interaction_targets.symbol:BRAF"));
        assert!(q.contains("chembl.drug_mechanisms.action_type:*inhibitor*"));
        assert!(q.contains("ndc.pharm_classes"));
        assert!(q.contains("chembl.molecule_type:\"Small molecule\""));
    }

    #[test]
    fn build_mychem_query_escapes_free_text_query() {
        let filters = DrugSearchFilters {
            query: Some("EGFR:inhibitor (3rd-gen)".into()),
            target: None,
            indication: None,
            mechanism: None,
            drug_type: None,
            atc: None,
            pharm_class: None,
            interactions: None,
        };

        let q = build_mychem_query(&filters).unwrap();
        assert!(q.contains(r"EGFR\:inhibitor"));
        assert!(q.contains(r"\(3rd\-gen\)"));
    }

    #[test]
    fn mechanism_match_uses_mechanism_fields_not_drug_name() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "x",
            "_score": 1.0,
            "drugbank": {"name": "alpha.1-proteinase inhibitor human"},
            "chembl": {
                "drug_mechanisms": [{"action_type": "protease inhibitor", "target_name": "ELANE"}]
            }
        }))
        .expect("valid hit");

        assert!(!hit_mentions_mechanism(&hit, "kinase inhibitor"));
        assert!(hit_mentions_mechanism(&hit, "protease inhibitor"));
    }

    #[test]
    fn parse_sections_supports_all_and_rejects_unknown() {
        let flags = parse_sections(&["all".to_string()]).unwrap();
        assert!(flags.include_label);
        assert!(flags.include_shortage);
        assert!(flags.include_targets);
        assert!(flags.include_indications);
        assert!(flags.include_interactions);
        assert!(flags.include_civic);
        assert!(flags.include_approvals);

        let err = parse_sections(&["bad".to_string()]).unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[test]
    fn parse_sections_all_with_explicit_label_keeps_label() {
        let flags = parse_sections(&["all".to_string(), "label".to_string()]).unwrap();
        assert!(flags.include_label);
    }

    #[test]
    fn map_drugsfda_approvals_extracts_key_fields() {
        let resp: OpenFdaResponse<DrugsFdaResult> = serde_json::from_value(serde_json::json!({
            "meta": {"results": {"skip": 0, "limit": 1, "total": 1}},
            "results": [{
                "application_number": "NDA021304",
                "sponsor_name": "Example Pharma",
                "openfda": {
                    "brand_name": ["DrugX"],
                    "generic_name": ["drugx"]
                },
                "products": [{
                    "brand_name": "DrugX",
                    "dosage_form": "TABLET",
                    "route": "ORAL",
                    "marketing_status": "Prescription",
                    "active_ingredients": [{"name": "drugx", "strength": "10 mg"}]
                }],
                "submissions": [{
                    "submission_type": "ORIG",
                    "submission_number": "1",
                    "submission_status": "AP",
                    "submission_status_date": "20120101"
                }]
            }]
        }))
        .expect("response should parse");

        let rows = map_drugsfda_approvals(resp);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].application_number, "NDA021304");
        assert_eq!(rows[0].openfda_brand_names, vec!["DrugX"]);
        assert_eq!(
            rows[0].products[0].active_ingredients,
            vec!["drugx (10 mg)"]
        );
        assert_eq!(
            rows[0].submissions[0].status_date.as_deref(),
            Some("2012-01-01")
        );
    }

    #[test]
    fn extract_top_adverse_events_ranks_by_frequency() {
        let resp: crate::sources::openfda::OpenFdaResponse<
            crate::sources::openfda::FaersEventResult,
        > = serde_json::from_value(serde_json::json!({
            "meta": {"results": {"skip": 0, "limit": 3, "total": 3}},
            "results": [
                {
                    "safetyreportid": "1",
                    "patient": {
                        "reaction": [
                            {"reactionmeddrapt": "Rash"},
                            {"reactionmeddrapt": "Nausea"}
                        ]
                    }
                },
                {
                    "safetyreportid": "2",
                    "patient": {
                        "reaction": [
                            {"reactionmeddrapt": "Rash"},
                            {"reactionmeddrapt": "Fatigue"}
                        ]
                    }
                },
                {
                    "safetyreportid": "3",
                    "patient": {
                        "reaction": [
                            {"reactionmeddrapt": "Fatigue"}
                        ]
                    }
                }
            ]
        }))
        .expect("valid openfda response");

        let out = extract_top_adverse_events(&resp);
        assert_eq!(out, vec!["Fatigue", "Rash", "Nausea"]);
    }
}
