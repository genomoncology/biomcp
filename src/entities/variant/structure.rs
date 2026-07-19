use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::entities::section_outcome::{SectionOutcome, SectionOutcomes};
use crate::entities::source_state_registry::outcome_keys;
use crate::error::BioMcpError;
use crate::sources::cancerhotspots::{CancerHotspotRecurrence, CancerHotspotsClient};
use crate::sources::interpro::InterProClient;
use crate::sources::myvariant::MyVariantHit;
use crate::sources::uniprot::{UniProtClient, UniProtRecord};

use super::{Variant, VariantIdFormat};

fn default_variant_structure_lookup_outcomes() -> SectionOutcomes {
    SectionOutcomes::with_keys(&outcome_keys("variant_structure"))
}

fn deserialize_variant_structure_lookup_outcomes<'de, D>(
    deserializer: D,
) -> Result<SectionOutcomes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let outcomes = SectionOutcomes::deserialize(deserializer)?;
    outcomes
        .validate_keys(&outcome_keys("variant_structure"))
        .map_err(serde::de::Error::custom)?;
    Ok(outcomes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantStructureResult {
    pub variant: String,
    pub gene: String,
    pub input_kind: String,
    pub residue: VariantStructureResidue,
    pub protein: VariantStructureProtein,
    #[serde(default)]
    pub domains: Vec<VariantStructureDomain>,
    pub structures: VariantStructureReferences,
    pub cancerhotspots: Option<CancerHotspotRecurrence>,
    #[serde(
        default = "default_variant_structure_lookup_outcomes",
        deserialize_with = "deserialize_variant_structure_lookup_outcomes"
    )]
    pub lookup_outcomes: SectionOutcomes,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(rename = "_meta")]
    pub meta: VariantStructureMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantStructureResidue {
    pub requested_change: Option<String>,
    pub position: Option<u32>,
    pub reference_aa: Option<String>,
    pub alternate_aa: Option<String>,
    pub source: String,
    #[serde(default)]
    pub matched_hgvsp: Vec<String>,
    #[serde(default)]
    pub other_source_positions: Vec<u32>,
    pub position_confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantStructureProtein {
    pub accession: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantStructureDomain {
    pub accession: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub domain_type: Option<String>,
    pub start: u32,
    pub end: u32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantStructureReferences {
    #[serde(default)]
    pub pdb: Vec<crate::sources::uniprot::UniProtPdbStructure>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alphafold: Option<VariantAlphaFoldStructure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantAlphaFoldStructure {
    pub id: String,
    pub url: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantStructureMeta {
    pub next_commands: Vec<String>,
}

pub async fn structure(id: &str) -> Result<VariantStructureResult, BioMcpError> {
    let (variant, id_format, hit) = super::get::resolve_base_with_hit(id).await?;
    let gene = variant.gene.trim();
    if gene.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Variant structure requires a variant that resolves to a gene symbol".into(),
        ));
    }

    let accession = crate::entities::protein::resolve_accession(gene).await?;
    let uniprot = UniProtClient::new()?;
    let record = uniprot.get_record(&accession).await?;
    let residue = residue_summary(&variant, &id_format, &hit);
    let position = residue.position;
    let hotspot_change = residue.requested_change.clone();

    let domains_fut = async {
        let Some(position) = position else {
            return Ok(None);
        };
        let domains = InterProClient::new()?.domains(&accession, 25).await?;
        Ok::<_, BioMcpError>(Some(overlapping_domains(domains, Some(position))))
    };
    let hotspots_fut = async { cancerhotspots(gene, hotspot_change.as_deref()).await };
    let (domains_result, hotspots_result) = tokio::join!(domains_fut, hotspots_fut);

    let mut warnings = Vec::new();
    if !residue.other_source_positions.is_empty() {
        warnings.push("MyVariant.info returned additional transcript/isoform protein positions; mapped domain/structure context uses the requested HGVSp position.".to_string());
    }
    if residue.position_confidence == "unresolved" {
        warnings.push("Could not select an exact protein residue from MyVariant.info/dbNSFP HGVSp aliases; domain and structure overlap flags may be absent.".to_string());
    }

    if let Err(err) = &domains_result {
        warn!(accession = %accession, "InterPro unavailable for variant structure: {err}");
    }
    if let Err(err) = &hotspots_result {
        warn!(gene = %gene, "cancerhotspots.org unavailable for variant structure: {err}");
    }
    let (domains, domains_outcome) = apply_domains_result(domains_result);
    let (cancerhotspots, hotspots_outcome) = apply_cancerhotspots_result(hotspots_result);
    let mut lookup_outcomes = default_variant_structure_lookup_outcomes();
    lookup_outcomes.complete("domains", domains_outcome);
    lookup_outcomes.complete("cancerhotspots", hotspots_outcome);

    Ok(VariantStructureResult {
        variant: display_variant(id, &variant, &id_format),
        gene: gene.to_string(),
        input_kind: input_kind_label(&id_format).to_string(),
        residue,
        protein: protein_summary(&record),
        domains,
        structures: structure_references(&record, position),
        cancerhotspots,
        lookup_outcomes,
        warnings,
        meta: VariantStructureMeta {
            next_commands: vec![
                format!("biomcp get protein {accession} structures"),
                format!("biomcp get protein {accession} domains"),
                format!(
                    "biomcp variant articles \"{}\"",
                    display_variant(id, &variant, &id_format)
                ),
            ],
        },
    })
}

fn apply_domains_result(
    result: Result<Option<Vec<VariantStructureDomain>>, BioMcpError>,
) -> (Vec<VariantStructureDomain>, SectionOutcome) {
    match result {
        Ok(None) => (
            Vec::new(),
            SectionOutcome::inapplicable(
                "A selected protein residue is required for domain lookup.",
            ),
        ),
        Ok(Some(domains)) if domains.is_empty() => (domains, SectionOutcome::empty("InterPro")),
        Ok(Some(domains)) => (domains, SectionOutcome::data("InterPro")),
        Err(_) => (
            Vec::new(),
            SectionOutcome::unavailable("InterPro domain data is temporarily unavailable."),
        ),
    }
}

fn apply_cancerhotspots_result(
    result: Result<Option<CancerHotspotRecurrence>, BioMcpError>,
) -> (Option<CancerHotspotRecurrence>, SectionOutcome) {
    match result {
        Ok(None) => (
            None,
            SectionOutcome::inapplicable(
                "A normalizable protein change is required for Cancer Hotspots.",
            ),
        ),
        Ok(Some(recurrence)) => {
            let outcome = if recurrence.position_count.is_some()
                || recurrence.same_aa_count.is_some()
                || recurrence.matched_transcript.is_some()
            {
                SectionOutcome::data("cancerhotspots.org")
            } else {
                SectionOutcome::empty("cancerhotspots.org")
            };
            (Some(recurrence), outcome)
        }
        Err(_) => (
            None,
            SectionOutcome::unavailable("Cancer Hotspots recurrence is temporarily unavailable."),
        ),
    }
}

async fn cancerhotspots(
    gene: &str,
    change: Option<&str>,
) -> Result<Option<CancerHotspotRecurrence>, BioMcpError> {
    let Some(normalized_change) = change.and_then(super::normalize_protein_change) else {
        return Ok(None);
    };
    let rows = CancerHotspotsClient::new()?.by_gene(gene).await?;
    Ok(Some(crate::sources::cancerhotspots::recurrence_for_change(
        &rows,
        &normalized_change,
    )))
}

fn display_variant(id: &str, variant: &Variant, id_format: &VariantIdFormat) -> String {
    if let VariantIdFormat::GeneProteinChange { gene, change } = id_format {
        return format!("{} {}", gene.trim(), change.trim());
    }
    if !variant.gene.trim().is_empty()
        && let Some(hgvs_p) = variant
            .hgvs_p
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
    {
        return format!(
            "{} {}",
            variant.gene.trim(),
            hgvs_p.trim_start_matches("p.")
        );
    }
    id.trim().to_string()
}

fn input_kind_label(id_format: &VariantIdFormat) -> &'static str {
    match id_format {
        VariantIdFormat::RsId(_) => "rsid",
        VariantIdFormat::HgvsGenomic(_) => "hgvs_genomic",
        VariantIdFormat::GeneProteinChange { .. } => "gene_protein_change",
    }
}

fn protein_summary(record: &UniProtRecord) -> VariantStructureProtein {
    VariantStructureProtein {
        accession: record.primary_accession.clone(),
        entry: record.uni_prot_kb_id.clone(),
        length: record.sequence.as_ref().and_then(|s| s.length),
        source: "UniProt".to_string(),
    }
}

fn structure_references(
    record: &UniProtRecord,
    residue: Option<u32>,
) -> VariantStructureReferences {
    let pdb = record.typed_pdb_structures(residue);
    let alphafold = record
        .alphafold_ids()
        .into_iter()
        .next()
        .map(|id| VariantAlphaFoldStructure {
            url: format!("https://alphafold.ebi.ac.uk/entry/{id}"),
            id,
            source: "UniProt cross-reference / AlphaFold DB".to_string(),
        });
    VariantStructureReferences { pdb, alphafold }
}

fn overlapping_domains(
    rows: Vec<crate::sources::interpro::InterProDomain>,
    residue: Option<u32>,
) -> Vec<VariantStructureDomain> {
    let Some(position) = residue else {
        return Vec::new();
    };
    rows.into_iter()
        .filter_map(|domain| {
            domain.ranges.into_iter().find_map(|range| {
                (range.start <= position && position <= range.end).then(|| VariantStructureDomain {
                    accession: domain.accession.clone(),
                    name: domain.name.clone(),
                    domain_type: domain.domain_type.clone(),
                    start: range.start,
                    end: range.end,
                    source: "InterPro".to_string(),
                })
            })
        })
        .collect()
}

fn residue_summary(
    variant: &Variant,
    id_format: &VariantIdFormat,
    hit: &MyVariantHit,
) -> VariantStructureResidue {
    let requested_change = requested_change(variant, id_format);
    let requested_normalized = requested_change
        .as_deref()
        .and_then(super::normalize_protein_change);
    let aliases = hit
        .dbnsfp
        .as_ref()
        .map(|dbnsfp| dbnsfp.hgvsp.clone().into_vec())
        .unwrap_or_default();

    let mut matched_hgvsp = Vec::new();
    let mut positions = BTreeSet::new();
    for alias in &aliases {
        if let Some(position) = hgvsp_position(alias) {
            positions.insert(position);
        }
        if let Some(requested) = requested_normalized.as_deref()
            && normalize_hgvsp_change(alias).as_deref() == Some(requested)
        {
            matched_hgvsp.push(alias.trim().to_string());
        }
    }

    let selected_position = matched_hgvsp
        .iter()
        .find_map(|alias| hgvsp_position(alias))
        .or_else(|| requested_normalized.as_deref().and_then(hgvsp_position));
    let other_source_positions = positions
        .into_iter()
        .filter(|position| Some(*position) != selected_position)
        .collect::<Vec<_>>();
    let (reference_aa, alternate_aa) = requested_normalized
        .as_deref()
        .and_then(change_aa)
        .unwrap_or((None, None));
    let position_confidence = if selected_position.is_some() && !matched_hgvsp.is_empty() {
        "requested_hgvsp_exact_match"
    } else if selected_position.is_some() {
        "requested_hgvsp_position_only"
    } else {
        "unresolved"
    };

    VariantStructureResidue {
        requested_change,
        position: selected_position,
        reference_aa,
        alternate_aa,
        source: "MyVariant.info/dbNSFP".to_string(),
        matched_hgvsp,
        other_source_positions,
        position_confidence: position_confidence.to_string(),
    }
}

fn requested_change(variant: &Variant, id_format: &VariantIdFormat) -> Option<String> {
    match id_format {
        VariantIdFormat::GeneProteinChange { change, .. } => Some(change.trim().to_string()),
        _ => variant
            .hgvs_p
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.trim_start_matches("p.").to_string()),
    }
}

fn normalize_hgvsp_change(value: &str) -> Option<String> {
    super::normalize_protein_change(protein_change_segment(value))
}

fn protein_change_segment(value: &str) -> &str {
    let trimmed = value.trim();
    trimmed
        .rsplit_once(":p.")
        .map(|(_, change)| change)
        .unwrap_or(trimmed)
}

fn hgvsp_position(value: &str) -> Option<u32> {
    let change = protein_change_segment(value);
    let mut digits = String::new();
    for ch in change.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else if !digits.is_empty() {
            break;
        }
    }
    digits.parse::<u32>().ok()
}

fn change_aa(value: &str) -> Option<(Option<String>, Option<String>)> {
    let change = protein_change_segment(value)
        .trim()
        .trim_start_matches("p.");
    let position_start = change.find(|ch: char| ch.is_ascii_digit())?;
    let position_end =
        change[position_start..].find(|ch: char| !ch.is_ascii_digit())? + position_start;
    let reference = change[..position_start].trim();
    let alternate = change[position_end..].trim();
    Some((
        (!reference.is_empty()).then(|| reference.to_string()),
        (!alternate.is_empty()).then(|| alternate.to_string()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::section_outcome::SectionOutcomeState;

    #[test]
    fn hgvsp_position_extracts_three_letter_short_and_accession_prefixed_aliases() {
        assert_eq!(hgvsp_position("p.Val600Glu"), Some(600));
        assert_eq!(hgvsp_position("p.V600E"), Some(600));
        assert_eq!(hgvsp_position("NP_004324.2:p.Val600Glu"), Some(600));
    }

    #[test]
    fn normalize_hgvsp_change_matches_accession_prefixed_aliases() {
        assert_eq!(
            normalize_hgvsp_change("NP_004324.2:p.Val600Glu").as_deref(),
            Some("V600E")
        );
    }

    #[test]
    fn domain_result_matrix_classifies_contact_and_applicability() {
        let domain = VariantStructureDomain {
            accession: "IPR000719".into(),
            name: Some("Protein kinase domain".into()),
            domain_type: Some("domain".into()),
            start: 457,
            end: 717,
            source: "InterPro".into(),
        };
        let cases = [
            (Ok(None), SectionOutcomeState::Inapplicable, 0, false),
            (Ok(Some(Vec::new())), SectionOutcomeState::Empty, 0, true),
            (Ok(Some(vec![domain])), SectionOutcomeState::Data, 1, true),
            (
                Err(BioMcpError::Api {
                    api: "InterPro".into(),
                    message: "fixture failure".into(),
                }),
                SectionOutcomeState::Unavailable,
                0,
                false,
            ),
        ];

        for (result, expected_state, expected_rows, credited) in cases {
            let (domains, outcome) = apply_domains_result(result);
            assert_eq!(outcome.outcome(), expected_state);
            assert_eq!(domains.len(), expected_rows);
            assert_eq!(!outcome.sources().is_empty(), credited);
        }
    }

    #[test]
    fn cancerhotspots_result_matrix_classifies_contact_and_applicability() {
        let recurrence = |position_count| CancerHotspotRecurrence {
            source: "cancerhotspots.org".into(),
            position_count,
            same_aa_count: None,
            matched_transcript: None,
        };
        let cases = [
            (Ok(None), SectionOutcomeState::Inapplicable, false, false),
            (
                Ok(Some(recurrence(None))),
                SectionOutcomeState::Empty,
                true,
                true,
            ),
            (
                Ok(Some(recurrence(Some(12)))),
                SectionOutcomeState::Data,
                true,
                true,
            ),
            (
                Err(BioMcpError::Api {
                    api: "cancerhotspots.org".into(),
                    message: "fixture failure".into(),
                }),
                SectionOutcomeState::Unavailable,
                false,
                false,
            ),
        ];

        for (result, expected_state, recurrence_present, credited) in cases {
            let (result_recurrence, outcome) = apply_cancerhotspots_result(result);
            assert_eq!(outcome.outcome(), expected_state);
            assert_eq!(result_recurrence.is_some(), recurrence_present);
            assert_eq!(!outcome.sources().is_empty(), credited);
        }
    }
}
