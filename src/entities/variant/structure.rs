use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::cancerhotspots::{CancerHotspotRecurrence, CancerHotspotsClient};
use crate::sources::interpro::InterProClient;
use crate::sources::myvariant::MyVariantHit;
use crate::sources::uniprot::{UniProtClient, UniProtRecord};

use super::{Variant, VariantIdFormat};

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
    pub cancerhotspots: CancerHotspotRecurrence,
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
        let domains = InterProClient::new()?.domains(&accession, 25).await?;
        Ok::<_, BioMcpError>(domains)
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

    let domains = match domains_result {
        Ok(rows) => overlapping_domains(rows, position),
        Err(err) => {
            warn!(accession = %accession, "InterPro unavailable for variant structure: {err}");
            warnings.push(format!("InterPro unavailable: {err}"));
            Vec::new()
        }
    };
    let cancerhotspots = match hotspots_result {
        Ok(recurrence) => recurrence,
        Err(err) => {
            warn!(gene = %gene, "cancerhotspots.org unavailable for variant structure: {err}");
            warnings.push(format!("cancerhotspots.org unavailable: {err}"));
            CancerHotspotRecurrence {
                source: "cancerhotspots.org".to_string(),
                position_count: None,
                same_aa_count: None,
                matched_transcript: None,
            }
        }
    };

    Ok(VariantStructureResult {
        variant: display_variant(id, &variant, &id_format),
        gene: gene.to_string(),
        input_kind: input_kind_label(&id_format).to_string(),
        residue,
        protein: protein_summary(&record),
        domains,
        structures: structure_references(&record, position),
        cancerhotspots,
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

async fn cancerhotspots(
    gene: &str,
    change: Option<&str>,
) -> Result<CancerHotspotRecurrence, BioMcpError> {
    let Some(normalized_change) = change.and_then(super::normalize_protein_change) else {
        return Ok(CancerHotspotRecurrence {
            source: "cancerhotspots.org".to_string(),
            position_count: None,
            same_aa_count: None,
            matched_transcript: None,
        });
    };
    let rows = CancerHotspotsClient::new()?.by_gene(gene).await?;
    Ok(crate::sources::cancerhotspots::recurrence_for_change(
        &rows,
        &normalized_change,
    ))
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
}
