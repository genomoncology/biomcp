//! Admit complete written coding fragments without reference or integer normalization.
use super::{
    RequestedVariantIdentity, VariantIdFormat, VariantInputKind, classify_variant_input,
    coding_change_re, is_exact_gene_token, parse_variant_id, split_gene_change_tokens,
};
use crate::error::BioMcpError;
use biodata::{HgvsMolecule, parse_hgvs_nucleotide_21_1_4};
use std::fmt;

const LIMIT: &str = "Gene coding lookup exceeds its input limit.";
const INVALID: &str =
    "Gene coding lookup requires an exact gene and a complete supported coding fragment.";

pub(in crate::entities::variant) struct CodingGet<'a> {
    source_bytes: usize,
    pub(in crate::entities::variant) gene: &'a str,
    pub(in crate::entities::variant) change: &'a str,
}
impl fmt::Debug for CodingGet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodingGet")
            .field("source_bytes", &self.source_bytes)
            .field("gene_bytes", &self.gene.len())
            .field("change_bytes", &self.change.len())
            .finish()
    }
}
impl CodingGet<'_> {
    pub(in crate::entities::variant) fn format(&self) -> VariantIdFormat {
        VariantIdFormat::GeneCodingChange {
            gene: self.gene.into(),
            change: self.change.into(),
        }
    }
    pub(in crate::entities::variant) fn requested(&self) -> RequestedVariantIdentity {
        RequestedVariantIdentity {
            gene: Some(self.gene.into()),
            coding_change: Some(self.change.into()),
            ..RequestedVariantIdentity::default()
        }
    }
}

pub(in crate::entities::variant) fn selects(input: &str) -> bool {
    input.split_whitespace().nth(1).is_some_and(|change| {
        change.starts_with("c.") || change.starts_with("C.") || change.contains(":c.")
    })
}

/// Preserve the original-input boundary before typed MCP removes wrapper whitespace.
pub(crate) fn original_input_limit(input: &str) -> Option<&'static str> {
    if input.len() <= 512 {
        return None;
    }
    if selects(input) {
        return Some(LIMIT);
    }
    // Keep the existing interval preflight's exact token admission.
    split_gene_change_tokens(input)
        .filter(|(gene, change)| {
            is_exact_gene_token(gene) && super::IntervalSearchAssertion::selects(change)
        })
        .map(|_| "Protein interval lookup exceeds its input limit.")
}

pub(in crate::entities::variant) fn preflight(input: &str) -> Result<(), BioMcpError> {
    if selects(input) && input.len() > 512 {
        return Err(BioMcpError::InvalidArgument(LIMIT.into()));
    }
    Ok(())
}

pub(in crate::entities::variant) fn prepare(
    input: &str,
) -> Result<Option<CodingGet<'_>>, BioMcpError> {
    preflight(input)?;
    if !selects(input) {
        return Ok(None);
    }
    let invalid = || BioMcpError::InvalidArgument(INVALID.into());
    let (gene, change) = split_gene_change_tokens(input).ok_or_else(invalid)?;
    if !is_exact_gene_token(gene) || !change.starts_with("c.") {
        return Err(invalid());
    }
    let envelope = parse_hgvs_nucleotide_21_1_4(change);
    let parsed = envelope.disposition().parsed().ok_or_else(invalid)?;
    if parsed.molecule() != HgvsMolecule::Coding
        || parsed.reference().is_some()
        || parsed.location().is_none()
        || parsed.edit().is_none()
        || envelope.render_source().as_deref() != Some(change)
        || parsed.render_constructed() != change
    {
        return Err(invalid());
    }
    Ok(Some(CodingGet {
        source_bytes: input.len(),
        gene,
        change,
    }))
}

impl RequestedVariantIdentity {
    pub(crate) fn from_variant_input(input: &str) -> Result<Self, BioMcpError> {
        let supplied = input.trim();
        if let Some((gene, coding)) = supplied.split_once(char::is_whitespace)
            && coding_change_re().is_match(coding.trim())
        {
            return Ok(Self {
                gene: Some(gene.to_string()),
                coding_change: Some(coding.trim().to_string()),
                ..Self::default()
            });
        }
        match classify_variant_input(supplied) {
            VariantInputKind::Exact(VariantIdFormat::RsId(_)) => Ok(Self {
                rsid: Some(supplied.to_string()),
                ..Self::default()
            }),
            VariantInputKind::Exact(VariantIdFormat::HgvsGenomic(_)) => {
                let mut identity = Self::default();
                identity.populate_genomic(supplied);
                Ok(identity)
            }
            VariantInputKind::Exact(VariantIdFormat::GeneProteinChange { gene, .. }) => {
                let protein_change =
                    split_gene_change_tokens(supplied).map(|(_, change)| change.to_string());
                Ok(Self {
                    gene: Some(gene),
                    protein_change,
                    ..Self::default()
                })
            }
            VariantInputKind::Exact(VariantIdFormat::GeneCodingChange { gene, change }) => {
                Ok(Self {
                    gene: Some(gene),
                    coding_change: Some(change),
                    ..Self::default()
                })
            }
            VariantInputKind::TranscriptCodingHgvs(value) => {
                let (transcript, coding) = value.split_once(':').unwrap_or(("", value.as_str()));
                Ok(Self {
                    transcript: (!transcript.is_empty()).then(|| transcript.to_string()),
                    coding_change: Some(coding.to_string()),
                    ..Self::default()
                })
            }
            _ => Err(parse_variant_id(supplied).unwrap_err()),
        }
    }
}
