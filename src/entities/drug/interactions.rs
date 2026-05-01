use std::collections::{HashMap, HashSet};

use futures::stream::{self, StreamExt};
use serde::Serialize;

use crate::entities::drug::{Drug, DrugInteraction};
use crate::error::BioMcpError;
use crate::sources::ddinter::{DdinterClient, DdinterIdentity, DdinterInteractionRow};

use super::label::extract_interaction_text_from_label;

const DDINTER_SOURCE_NOTE: &str = "Structured rows come from the current DDInter download bundle. DDInter warns that missing rows do not prove no interaction exists.";
const DDINTER_EMPTY_NOTE: &str = "The current DDInter download bundle has no matching rows for this drug. DDInter warns that missing rows do not prove no interaction exists.";
const PARTNER_ENRICH_CONCURRENCY: usize = 8;

#[derive(Debug, Clone, Serialize)]
pub struct DrugInteractionClassSummary {
    pub class_name: String,
    pub interaction_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_level: Option<String>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub class_summaries: Vec<DrugInteractionClassSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_interaction_text: Option<String>,
}

#[derive(Debug, Clone)]
struct InteractionAggregation {
    drug: String,
    level: Option<String>,
}

pub(crate) async fn interaction_report(name: String) -> Result<DrugInteractionReport, BioMcpError> {
    let resolved = super::get::resolve_drug_base(&name, true, false).await?;
    interaction_report_from_base(name, resolved.drug, resolved.label_response).await
}

pub(crate) async fn interaction_report_from_base(
    requested_name: String,
    anchor: Drug,
    label_response: Option<serde_json::Value>,
) -> Result<DrugInteractionReport, BioMcpError> {
    let legacy_descriptions = interaction_description_map(&anchor);
    let anchor_name = anchor.name.clone();
    let brand_names = anchor.brand_names.clone();
    let drugbank_id = anchor.drugbank_id.clone();
    let chembl_id = anchor.chembl_id.clone();
    let label_interaction_text = label_response
        .as_ref()
        .and_then(extract_interaction_text_from_label);
    let client = DdinterClient::ready(crate::sources::ddinter::DdinterSyncMode::Auto).await?;
    let identity = DdinterIdentity::with_aliases(&requested_name, Some(&anchor_name), &brand_names);
    let rows = client.interactions(&identity);
    let interactions = aggregate_rows(&rows, &identity)?;
    let partner_classes = enrich_partner_classes(&interactions).await;
    let mut out = Vec::new();
    for interaction in interactions {
        let classes = partner_classes
            .get(&interaction.drug.to_ascii_lowercase())
            .cloned()
            .unwrap_or_default();
        out.push(DrugInteraction {
            description: interaction_description(&legacy_descriptions, &interaction.drug),
            drug: interaction.drug,
            level: interaction.level,
            partner_classes: classes,
        });
    }
    out.sort_by(|a, b| {
        severity_rank(b.level.as_deref())
            .cmp(&severity_rank(a.level.as_deref()))
            .then_with(|| a.drug.cmp(&b.drug))
    });
    let class_summaries = build_class_summaries(&out);
    let source_note = Some(if out.is_empty() {
        DDINTER_EMPTY_NOTE.to_string()
    } else {
        DDINTER_SOURCE_NOTE.to_string()
    });
    Ok(DrugInteractionReport {
        name: anchor_name,
        drugbank_id,
        chembl_id,
        interactions: out,
        class_summaries,
        source_note,
        label_interaction_text,
    })
}

pub(crate) fn apply_interaction_report(drug: &mut Drug, report: &DrugInteractionReport) {
    drug.interactions = report.interactions.clone();
    drug.interaction_text = report.label_interaction_text.clone();
}

pub(crate) fn interaction_class_summaries(
    interactions: &[DrugInteraction],
) -> Vec<DrugInteractionClassSummary> {
    build_class_summaries(interactions)
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
            (true, true) => continue,
            (false, false) => continue,
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

async fn enrich_partner_classes(
    interactions: &[InteractionAggregation],
) -> HashMap<String, Vec<String>> {
    let names = interactions
        .iter()
        .map(|row| row.drug.clone())
        .collect::<Vec<_>>();
    stream::iter(names)
        .map(|drug_name| async move {
            let classes = match crate::sources::mychem::MyChemClient::new() {
                Ok(client) => match client
                    .query_with_fields(&drug_name, 25, 0, crate::sources::mychem::MYCHEM_FIELDS_GET)
                    .await
                {
                    Ok(response) => {
                        let hits = response.hits.iter().collect::<Vec<_>>();
                        let merged = crate::transform::drug::merge_mychem_hits(&hits, &drug_name);
                        merged.pharm_classes
                    }
                    Err(_) => Vec::new(),
                },
                Err(_) => Vec::new(),
            };
            (drug_name.to_ascii_lowercase(), classes)
        })
        .buffer_unordered(PARTNER_ENRICH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter(|(_, classes)| !classes.is_empty())
        .collect()
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

fn build_class_summaries(interactions: &[DrugInteraction]) -> Vec<DrugInteractionClassSummary> {
    let normalized = build_normalized_class_summaries(interactions);
    if !normalized.is_empty() {
        return normalized;
    }

    let mut by_class: HashMap<String, DrugInteractionClassSummary> = HashMap::new();
    for row in interactions {
        for class_name in &row.partner_classes {
            let key = class_name.to_ascii_lowercase();
            let entry = by_class
                .entry(key)
                .or_insert_with(|| DrugInteractionClassSummary {
                    class_name: class_name.clone(),
                    interaction_count: 0,
                    highest_level: row.level.clone(),
                });
            entry.interaction_count += 1;
            if severity_rank(row.level.as_deref()) > severity_rank(entry.highest_level.as_deref()) {
                entry.highest_level = row.level.clone();
            }
        }
    }
    let mut out = by_class.into_values().collect::<Vec<_>>();
    out.sort_by(|a, b| {
        b.interaction_count
            .cmp(&a.interaction_count)
            .then_with(|| a.class_name.cmp(&b.class_name))
    });
    out
}

fn build_normalized_class_summaries(
    interactions: &[DrugInteraction],
) -> Vec<DrugInteractionClassSummary> {
    let mut by_class: HashMap<String, DrugInteractionClassSummary> = HashMap::new();
    for row in interactions {
        for bucket in normalized_interaction_buckets(row) {
            let key = bucket.to_ascii_lowercase();
            let entry = by_class
                .entry(key)
                .or_insert_with(|| DrugInteractionClassSummary {
                    class_name: bucket.clone(),
                    interaction_count: 0,
                    highest_level: row.level.clone(),
                });
            entry.interaction_count += 1;
            if severity_rank(row.level.as_deref()) > severity_rank(entry.highest_level.as_deref()) {
                entry.highest_level = row.level.clone();
            }
        }
    }
    let mut out = by_class.into_values().collect::<Vec<_>>();
    out.sort_by(|a, b| {
        b.interaction_count
            .cmp(&a.interaction_count)
            .then_with(|| a.class_name.cmp(&b.class_name))
    });
    out
}

fn normalized_interaction_buckets(row: &DrugInteraction) -> Vec<String> {
    let mut buckets = Vec::new();
    for class_name in &row.partner_classes {
        if let Some(bucket) = normalized_bucket_from_class(class_name) {
            buckets.push(bucket.to_string());
        }
    }
    if let Some(bucket) = normalized_bucket_from_drug_name(&row.drug) {
        buckets.push(bucket.to_string());
    }
    buckets.sort();
    buckets.dedup();
    buckets
}

fn normalized_bucket_from_class(class_name: &str) -> Option<&'static str> {
    let lowered = class_name.to_ascii_lowercase();
    if lowered.contains("cytochrome p450 3a4") {
        Some("CYP3A4")
    } else if lowered.contains("cytochrome p450 3a") {
        Some("CYP3A")
    } else if lowered.contains("cytochrome p450 2c9") {
        Some("CYP2C9")
    } else if lowered.contains("cytochrome p450 2c19") {
        Some("CYP2C19")
    } else if lowered.contains("cytochrome p450 2d6") {
        Some("CYP2D6")
    } else if lowered.contains("cyclooxygenase")
        || lowered.contains("p2y12")
        || lowered.contains("platelet")
    {
        Some("antiplatelets")
    } else if lowered.contains("hydroxymethylglutaryl")
        || lowered.contains("hmg-coa")
        || lowered.contains("statin")
    {
        Some("statins")
    } else if lowered.contains("beta lactamase")
        || lowered.contains("protease inhibitor")
        || lowered.contains("reverse transcriptase")
        || lowered.contains("neuraminidase")
        || lowered.contains("rna replicase")
        || lowered.contains("rna synthetase")
        || lowered.contains("dna polymerase")
        || lowered.contains("protein synthesis")
        || lowered.contains("dihydrofolate")
        || lowered.contains("hiv ")
        || lowered.contains("hcv ")
    {
        Some("anti-infectives")
    } else {
        None
    }
}

fn normalized_bucket_from_drug_name(drug_name: &str) -> Option<&'static str> {
    let lowered = drug_name.to_ascii_lowercase();
    if lowered.ends_with("floxacin")
        || lowered.ends_with("cillin")
        || lowered.ends_with("cycline")
        || lowered.ends_with("penem")
        || lowered.ends_with("cef")
        || lowered.ends_with("mycin")
        || lowered.ends_with("vir")
        || lowered == "metronidazole"
        || lowered == "fluconazole"
        || lowered == "voriconazole"
    {
        Some("anti-infectives")
    } else if lowered.contains("platelet")
        || matches!(
            lowered.as_str(),
            "aspirin" | "clopidogrel" | "prasugrel" | "ticagrelor"
        )
    {
        Some("antiplatelets")
    } else if lowered.contains("statin") {
        Some("statins")
    } else if lowered.contains("cyp3a4") {
        Some("CYP3A4")
    } else {
        None
    }
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
    fn normalized_class_summaries_surface_design_buckets() {
        let interactions = vec![
            DrugInteraction {
                drug: "clopidogrel".to_string(),
                level: Some("Major".to_string()),
                description: None,
                partner_classes: vec!["P2Y12 Receptor Antagonists".to_string()],
            },
            DrugInteraction {
                drug: "ciprofloxacin".to_string(),
                level: Some("Major".to_string()),
                description: None,
                partner_classes: vec!["Cytochrome P450 1A2 Inhibitors".to_string()],
            },
            DrugInteraction {
                drug: "atorvastatin".to_string(),
                level: Some("Moderate".to_string()),
                description: None,
                partner_classes: vec!["Hydroxymethylglutaryl-CoA Reductase Inhibitors".to_string()],
            },
            DrugInteraction {
                drug: "imatinib".to_string(),
                level: Some("Major".to_string()),
                description: None,
                partner_classes: vec!["Cytochrome P450 3A4 Inhibitors".to_string()],
            },
        ];

        let summaries = build_class_summaries(&interactions)
            .into_iter()
            .map(|summary| summary.class_name)
            .collect::<Vec<_>>();

        assert!(summaries.contains(&"anti-infectives".to_string()));
        assert!(summaries.contains(&"antiplatelets".to_string()));
        assert!(summaries.contains(&"statins".to_string()));
        assert!(summaries.contains(&"CYP3A4".to_string()));
    }

    #[test]
    fn apply_interaction_report_preserves_anchor_pharm_classes() {
        let mut drug = Drug {
            name: "warfarin".to_string(),
            drugbank_id: None,
            chembl_id: None,
            unii: None,
            drug_type: None,
            mechanism: None,
            mechanisms: Vec::new(),
            approval_date: None,
            approval_date_raw: None,
            approval_date_display: None,
            approval_summary: None,
            brand_names: Vec::new(),
            route: None,
            targets: Vec::new(),
            variant_targets: Vec::new(),
            target_family: None,
            target_family_name: None,
            indications: Vec::new(),
            interactions: Vec::new(),
            interaction_text: None,
            pharm_classes: vec!["Vitamin K antagonists".to_string()],
            top_adverse_events: Vec::new(),
            faers_query: None,
            label: None,
            label_set_id: None,
            shortage: None,
            approvals: None,
            us_safety_warnings: None,
            ema_regulatory: None,
            ema_safety: None,
            ema_shortage: None,
            who_prequalification: None,
            civic: None,
        };
        let report = DrugInteractionReport {
            name: "warfarin".to_string(),
            drugbank_id: None,
            chembl_id: None,
            interactions: vec![DrugInteraction {
                drug: "aspirin".to_string(),
                level: Some("Major".to_string()),
                description: None,
                partner_classes: vec!["antiplatelets".to_string()],
            }],
            class_summaries: vec![DrugInteractionClassSummary {
                class_name: "antiplatelets".to_string(),
                interaction_count: 1,
                highest_level: Some("Major".to_string()),
            }],
            source_note: None,
            label_interaction_text: Some("Additive label text".to_string()),
        };

        apply_interaction_report(&mut drug, &report);

        assert_eq!(
            drug.pharm_classes,
            vec!["Vitamin K antagonists".to_string()]
        );
        assert_eq!(drug.interactions.len(), 1);
        assert_eq!(
            drug.interaction_text.as_deref(),
            Some("Additive label text")
        );
    }

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
