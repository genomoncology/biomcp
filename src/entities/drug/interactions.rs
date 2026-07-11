use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::entities::drug::{Drug, DrugInteraction};
use crate::error::BioMcpError;
use crate::sources::ddinter::{
    DdinterBundleFreshness, DdinterClient, DdinterIdentity, DdinterInteractionRow,
};

use super::label::extract_interaction_text_from_label;

const DDINTER_SOURCE_NOTE: &str = "Structured rows come from the current DDInter download bundle. DDInter warns that missing rows do not prove no interaction exists.";
const DDINTER_EMPTY_NOTE: &str = "The current DDInter download bundle has no matching rows for this drug. DDInter warns that missing rows do not prove no interaction exists.";
const DDINTER_NOT_IN_COVERAGE_NOTE: &str = "Coverage status: not_in_ddinter_coverage. The queried drug is not present in the current DDInter download bundle; this is a source coverage miss, not evidence of no interactions.";
pub(crate) const DEFAULT_INTERACTION_LIMIT: usize = 25;
pub(crate) const MAX_INTERACTION_LIMIT: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrugInteractionCoverageStatus {
    InDdinterCoverage,
    NotInDdinterCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrugInteractionFreshnessStatus {
    Fresh,
    Stale,
}

impl DrugInteractionFreshnessStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Stale => "stale",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugInteractionBundleFreshness {
    pub status: DrugInteractionFreshnessStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugInteractionPagination {
    pub total: usize,
    pub count: usize,
    pub offset: usize,
    pub limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_command: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DrugInteractionReport {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drugbank_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chembl_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interactions: Vec<DrugInteraction>,
    pub pagination: DrugInteractionPagination,
    pub bundle_freshness: DrugInteractionBundleFreshness,
    pub coverage_status: DrugInteractionCoverageStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_interaction_text: Option<String>,
}

#[derive(Debug, Clone)]
struct InteractionAggregation {
    drug: String,
    level: Option<String>,
}

pub(crate) async fn interaction_report(
    name: String,
    limit: usize,
    offset: usize,
) -> Result<DrugInteractionReport, BioMcpError> {
    let resolved = super::get::resolve_drug_base(&name, true, false).await?;
    interaction_report_from_base(name, resolved.drug, resolved.label_response, limit, offset).await
}

pub(crate) async fn interaction_report_from_base(
    requested_name: String,
    anchor: Drug,
    label_response: Option<serde_json::Value>,
    limit: usize,
    offset: usize,
) -> Result<DrugInteractionReport, BioMcpError> {
    let legacy_descriptions = interaction_description_map(&anchor);
    let anchor_name = anchor.name.clone();
    let brand_names = anchor.brand_names.clone();
    let drugbank_id = anchor.drugbank_id.clone();
    let chembl_id = anchor.chembl_id.clone();
    let label_interaction_text = label_response
        .as_ref()
        .and_then(extract_interaction_text_from_label);
    let client = DdinterClient::ready().await?;
    let identity = DdinterIdentity::with_aliases(&requested_name, Some(&anchor_name), &brand_names);
    let rows = client.interactions(&identity);
    let in_ddinter_coverage = client.contains_identity(&identity);
    let mut interactions = aggregate_rows(&rows, &identity)?
        .into_iter()
        .map(|interaction| DrugInteraction {
            description: interaction_description(&legacy_descriptions, &interaction.drug),
            drug: interaction.drug,
            level: interaction.level,
            partner_classes: Vec::new(),
        })
        .collect::<Vec<_>>();
    interactions.sort_by(|a, b| {
        severity_rank(b.level.as_deref())
            .cmp(&severity_rank(a.level.as_deref()))
            .then_with(|| a.drug.cmp(&b.drug))
    });
    let (interactions, total) = crate::cli::paginate_results(interactions, offset, limit);
    let count = interactions.len();
    let next_offset = offset.saturating_add(count);
    let next_command = (next_offset < total).then(|| {
        format!(
            "biomcp drug interactions {} --limit {limit} --offset {next_offset}",
            anchor_name.to_ascii_lowercase()
        )
    });
    let coverage_status = if in_ddinter_coverage {
        DrugInteractionCoverageStatus::InDdinterCoverage
    } else {
        DrugInteractionCoverageStatus::NotInDdinterCoverage
    };
    let source_note = Some(if total == 0 {
        DDINTER_EMPTY_NOTE.to_string()
    } else {
        DDINTER_SOURCE_NOTE.to_string()
    });
    let coverage_note = (!in_ddinter_coverage).then(|| DDINTER_NOT_IN_COVERAGE_NOTE.to_string());
    let status = match client.freshness() {
        DdinterBundleFreshness::Fresh => DrugInteractionFreshnessStatus::Fresh,
        DdinterBundleFreshness::Stale => DrugInteractionFreshnessStatus::Stale,
    };
    Ok(DrugInteractionReport {
        name: anchor_name,
        drugbank_id,
        chembl_id,
        interactions,
        pagination: DrugInteractionPagination {
            total,
            count,
            offset,
            limit,
            next_command,
        },
        bundle_freshness: DrugInteractionBundleFreshness { status },
        coverage_status,
        source_note,
        coverage_note,
        label_interaction_text,
    })
}

pub(crate) fn apply_interaction_report(drug: &mut Drug, report: &DrugInteractionReport) {
    drug.interactions = report.interactions.clone();
    drug.interaction_text = report.label_interaction_text.clone();
    drug.interaction_pagination = Some(report.pagination.clone());
    drug.interaction_bundle_freshness = Some(report.bundle_freshness.clone());
}

fn aggregate_rows(
    rows: &[DdinterInteractionRow],
    identity: &DdinterIdentity,
) -> Result<Vec<InteractionAggregation>, BioMcpError> {
    let anchor_terms = identity.terms().iter().cloned().collect::<HashSet<_>>();
    let mut by_partner: HashMap<String, InteractionAggregation> = HashMap::new();
    for row in rows {
        let a_matches = crate::sources::ddinter::normalize_name_key(&row.drug_a)
            .is_some_and(|value| anchor_terms.contains(&value));
        let b_matches = crate::sources::ddinter::normalize_name_key(&row.drug_b)
            .is_some_and(|value| anchor_terms.contains(&value));
        let (partner_id, partner_name) = match (a_matches, b_matches) {
            (true, false) => (&row.drug_b_id, &row.drug_b),
            (false, true) => (&row.drug_a_id, &row.drug_a),
            (true, true) | (false, false) => continue,
        };
        let key = if !partner_id.trim().is_empty() {
            partner_id.to_ascii_lowercase()
        } else {
            partner_name.to_ascii_lowercase()
        };
        let entry = by_partner
            .entry(key)
            .or_insert_with(|| InteractionAggregation {
                drug: partner_name.to_string(),
                level: row.level.clone(),
            });
        if severity_rank(row.level.as_deref()) > severity_rank(entry.level.as_deref()) {
            entry.level = row.level.clone();
        }
    }
    Ok(by_partner.into_values().collect())
}

fn interaction_description_map(anchor: &Drug) -> HashMap<String, String> {
    anchor
        .interactions
        .iter()
        .filter_map(|row| {
            let description = row
                .description
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())?;
            let key = crate::sources::ddinter::normalize_name_key(&row.drug)?;
            Some((key, description.to_string()))
        })
        .collect()
}

fn interaction_description(
    descriptions: &HashMap<String, String>,
    partner_name: &str,
) -> Option<String> {
    let key = crate::sources::ddinter::normalize_name_key(partner_name)?;
    descriptions.get(&key).cloned()
}

fn severity_rank(level: Option<&str>) -> usize {
    match level
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "contraindicated" => 4,
        "major" => 3,
        "moderate" => 2,
        "minor" => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_description_uses_legacy_partner_narrative() {
        let descriptions = HashMap::from([(
            "aspirin".to_string(),
            "May increase bleeding risk.".to_string(),
        )]);

        assert_eq!(
            interaction_description(&descriptions, "Aspirin"),
            Some("May increase bleeding risk.".to_string())
        );
        assert_eq!(interaction_description(&descriptions, "clopidogrel"), None);
    }
}
