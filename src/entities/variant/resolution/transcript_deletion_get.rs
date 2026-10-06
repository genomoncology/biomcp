//! Strict decorated transcript admission around the locked shared coding parser.
use super::{VariantIdFormat, is_exact_gene_token};
use crate::error::BioMcpError;
use biodata::{HgvsEdit, HgvsMolecule, parse_hgvs_nucleotide_21_1_4};
use std::fmt;

pub(in crate::entities::variant) const LIMIT: &str =
    "Transcript gene coding lookup exceeds its input limit.";
const INVALID: &str = "Transcript gene coding lookup requires an exact versioned transcript, gene and complete supported coding fragment.";
pub(super) const ARTICLES: &str = "Transcript gene coding lookup is not supported for variant articles. Use get variant for coding detail.";

pub(in crate::entities::variant) struct TranscriptDeletion<'a> {
    pub transcript: &'a str,
    pub gene: &'a str,
    pub change: &'a str,
    deletion: bool,
}
impl fmt::Debug for TranscriptDeletion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranscriptDeletion")
            .field("transcript_bytes", &self.transcript.len())
            .field("gene_bytes", &self.gene.len())
            .field("change_bytes", &self.change.len())
            .finish()
    }
}
impl TranscriptDeletion<'_> {
    pub(in crate::entities::variant) fn format(&self) -> VariantIdFormat {
        let transcript = self.transcript.into();
        let gene = self.gene.into();
        let change = self.change.into();
        if self.deletion {
            VariantIdFormat::TranscriptGeneDeletion {
                transcript,
                gene,
                change,
            }
        } else {
            VariantIdFormat::TranscriptGeneCodingChange {
                transcript,
                gene,
                change,
            }
        }
    }
}
pub(in crate::entities::variant) fn selects(input: &str) -> bool {
    let input = input.split_whitespace().next().unwrap_or_default();
    input.starts_with("NM_")
        && input.split_once('(').is_some_and(|(prefix, _)| {
            !prefix.contains(':') && !prefix.contains(char::is_whitespace)
        })
}
pub(in crate::entities::variant) fn versioned_transcript(input: &str) -> bool {
    input
        .strip_prefix("NM_")
        .and_then(|tail| tail.split_once('.'))
        .is_some_and(|(id, version)| {
            !id.is_empty()
                && !version.is_empty()
                && id.bytes().all(|b| b.is_ascii_digit())
                && version.bytes().all(|b| b.is_ascii_digit())
        })
}
fn coding_kind(change: &str) -> Option<bool> {
    let envelope = parse_hgvs_nucleotide_21_1_4(change);
    let parsed = envelope.disposition().parsed()?;
    (parsed.molecule() == HgvsMolecule::Coding
        && parsed.reference().is_none()
        && parsed.location().is_some()
        && parsed.edit().is_some()
        && envelope.render_source() == Some(change)
        && parsed.render_constructed() == change)
        .then(|| !parsed.is_predicted() && matches!(parsed.edit(), Some(HgvsEdit::Deletion { .. })))
}
pub(in crate::entities::variant) fn coding_valid(change: &str) -> bool {
    coding_kind(change).is_some()
}
pub(in crate::entities::variant) fn preflight(input: &str) -> Result<(), BioMcpError> {
    if input.len() > 512 && selects(input) {
        return Err(BioMcpError::InvalidArgument(LIMIT.into()));
    }
    Ok(())
}
pub(in crate::entities::variant) fn prepare(
    input: &str,
) -> Result<Option<TranscriptDeletion<'_>>, BioMcpError> {
    if !selects(input) {
        return Ok(None);
    }
    preflight(input)?;
    tuple(input).map(Some)
}
pub(in crate::entities::variant) fn tuple(
    input: &str,
) -> Result<TranscriptDeletion<'_>, BioMcpError> {
    let invalid = || BioMcpError::InvalidArgument(INVALID.into());
    let input = input.trim();
    let (transcript, tail) = input.split_once('(').ok_or_else(invalid)?;
    let (gene, tail) = tail.split_once(')').ok_or_else(invalid)?;
    let change = tail
        .strip_prefix(':')
        .or_else(|| {
            tail.starts_with(char::is_whitespace)
                .then(|| tail.trim_start())
        })
        .ok_or_else(invalid)?;
    let deletion = coding_kind(change).ok_or_else(invalid)?;
    if !versioned_transcript(transcript) || !is_exact_gene_token(gene) {
        return Err(invalid());
    }
    Ok(TranscriptDeletion {
        transcript,
        gene,
        change,
        deletion,
    })
}
