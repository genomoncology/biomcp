//! rsID, HGVS, and protein change parsing and classification.

use regex::Regex;
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::OnceLock;

use super::{
    VariantGuidance, VariantGuidanceKind, VariantInputKind, VariantProteinAlias, VariantShorthand,
    transcript_coding_hgvs_re,
};
use crate::error::BioMcpError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantIdFormat {
    TranscriptGeneDeletion {
        transcript: String,
        gene: String,
        change: String,
    },
    TranscriptGeneCodingChange {
        transcript: String,
        gene: String,
        change: String,
    },
    RsId(String),
    ClinvarVariationId(u64),
    HgvsGenomic(String),
    GeneProteinChange {
        gene: String,
        change: String,
    },
    GeneCodingChange {
        gene: String,
        change: String,
    },
}
mod coding_alias;
pub(super) mod coding_get;
pub(super) mod transcript_deletion_get;
pub(crate) use coding_get::original_input_limit as coding_get_input_limit;
pub(super) mod genomic_assertion;
mod genomic_lookup;
mod interval_comparison;
mod interval_search;
pub(crate) use interval_search::{
    IntervalSearchAssertion, IntervalSearchDisposition, protein_interval_search,
};
mod point_alias;
pub(super) use point_alias::complete_source_protein_point;
pub(super) mod protein_get;
pub(super) use coding_alias::coding_changes_equivalent;
use coding_alias::coding_key;
use protein_get::parse_exact_gene_protein_change;

fn rsid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^(rs\d+)$").expect("valid regex"))
}

fn clinvar_variation_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^([1-9][0-9]{0,9})$").expect("valid regex"))
}

/// A bare positive integer is a ClinVar VariationID (ticket 1292).
pub(crate) fn parse_clinvar_variation_id(value: &str) -> Option<u64> {
    clinvar_variation_id_re()
        .captures(value.trim())
        .and_then(|caps| caps[1].parse().ok())
}

pub(crate) fn is_rsid(value: &str) -> bool {
    rsid_re().is_match(value.trim())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NormalizedGenomicCoordinate {
    pub id: String,
    pub genome_build: Option<super::GenomeBuild>,
    pub requires_comparison: bool,
}

const REFSEQ_GENOMIC_BUILDS: &[(&str, &str, super::GenomeBuild)] = &[
    ("NC_000001.10", "chr1", super::GenomeBuild::Grch37),
    ("NC_000001.11", "chr1", super::GenomeBuild::Grch38),
    ("NC_000002.11", "chr2", super::GenomeBuild::Grch37),
    ("NC_000002.12", "chr2", super::GenomeBuild::Grch38),
    ("NC_000003.11", "chr3", super::GenomeBuild::Grch37),
    ("NC_000003.12", "chr3", super::GenomeBuild::Grch38),
    ("NC_000004.11", "chr4", super::GenomeBuild::Grch37),
    ("NC_000004.12", "chr4", super::GenomeBuild::Grch38),
    ("NC_000005.9", "chr5", super::GenomeBuild::Grch37),
    ("NC_000005.10", "chr5", super::GenomeBuild::Grch38),
    ("NC_000006.11", "chr6", super::GenomeBuild::Grch37),
    ("NC_000006.12", "chr6", super::GenomeBuild::Grch38),
    ("NC_000007.13", "chr7", super::GenomeBuild::Grch37),
    ("NC_000007.14", "chr7", super::GenomeBuild::Grch38),
    ("NC_000008.10", "chr8", super::GenomeBuild::Grch37),
    ("NC_000008.11", "chr8", super::GenomeBuild::Grch38),
    ("NC_000009.11", "chr9", super::GenomeBuild::Grch37),
    ("NC_000009.12", "chr9", super::GenomeBuild::Grch38),
    ("NC_000010.10", "chr10", super::GenomeBuild::Grch37),
    ("NC_000010.11", "chr10", super::GenomeBuild::Grch38),
    ("NC_000011.9", "chr11", super::GenomeBuild::Grch37),
    ("NC_000011.10", "chr11", super::GenomeBuild::Grch38),
    ("NC_000012.11", "chr12", super::GenomeBuild::Grch37),
    ("NC_000012.12", "chr12", super::GenomeBuild::Grch38),
    ("NC_000013.10", "chr13", super::GenomeBuild::Grch37),
    ("NC_000013.11", "chr13", super::GenomeBuild::Grch38),
    ("NC_000014.8", "chr14", super::GenomeBuild::Grch37),
    ("NC_000014.9", "chr14", super::GenomeBuild::Grch38),
    ("NC_000015.9", "chr15", super::GenomeBuild::Grch37),
    ("NC_000015.10", "chr15", super::GenomeBuild::Grch38),
    ("NC_000016.9", "chr16", super::GenomeBuild::Grch37),
    ("NC_000016.10", "chr16", super::GenomeBuild::Grch38),
    ("NC_000017.10", "chr17", super::GenomeBuild::Grch37),
    ("NC_000017.11", "chr17", super::GenomeBuild::Grch38),
    ("NC_000018.9", "chr18", super::GenomeBuild::Grch37),
    ("NC_000018.10", "chr18", super::GenomeBuild::Grch38),
    ("NC_000019.9", "chr19", super::GenomeBuild::Grch37),
    ("NC_000019.10", "chr19", super::GenomeBuild::Grch38),
    ("NC_000020.10", "chr20", super::GenomeBuild::Grch37),
    ("NC_000020.11", "chr20", super::GenomeBuild::Grch38),
    ("NC_000021.8", "chr21", super::GenomeBuild::Grch37),
    ("NC_000021.9", "chr21", super::GenomeBuild::Grch38),
    ("NC_000022.10", "chr22", super::GenomeBuild::Grch37),
    ("NC_000022.11", "chr22", super::GenomeBuild::Grch38),
    ("NC_000023.10", "chrX", super::GenomeBuild::Grch37),
    ("NC_000023.11", "chrX", super::GenomeBuild::Grch38),
    ("NC_000024.9", "chrY", super::GenomeBuild::Grch37),
    ("NC_000024.10", "chrY", super::GenomeBuild::Grch38),
];

pub(crate) fn normalize_genomic_coordinate(
    input: &str,
) -> Result<Option<NormalizedGenomicCoordinate>, BioMcpError> {
    genomic_lookup::genomic_lookup(input, genomic_lookup::LookupRoute::Coordinate)?
        .map(|lookup| lookup.coordinate())
        .transpose()
        .map(Option::flatten)
}

fn refseq_accession_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^NC_[0-9]+\.[0-9]+$").expect("valid regex"))
}

fn gene_protein_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^([A-Z][A-Z0-9]+)\s+([A-Z]\d+[A-Z*])$").expect("valid regex"))
}

fn gene_residue_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^([A-Z][A-Z0-9]+)\s+(\d+)([A-Z*])$").expect("valid regex"))
}

fn residue_alias_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(\d+)([A-Z*])$").expect("valid regex"))
}

fn clinvar_name_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^([A-Z]{2}_[0-9]+\.[0-9]+)\([A-Za-z0-9]+\)(?::|\s+)(c\.[^\s]+)$")
            .expect("valid regex")
    })
}

/// ClinVar-style names keep the gene in parentheses (`NM_177438.3(DICER1)
/// c.4449G>A`). BioMCP refuses them instead of half-parsing; when the rest of
/// the name is a complete coding change, the colon form is a working input.
fn clinvar_style_name_working_form(id: &str) -> Option<String> {
    let caps = clinvar_name_re().captures(id.trim())?;
    let working_form = format!("{}:{}", &caps[1], &caps[2]);
    (transcript_coding_hgvs_re().is_match(&working_form) && coding_change_re().is_match(&caps[2]))
        .then_some(working_form)
}

fn quote_command_arg(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.chars().any(|c| c.is_whitespace()) {
        return format!("\"{}\"", trimmed.replace('\"', "\\\""));
    }
    trimmed.to_string()
}

pub fn parse_variant_protein_alias(alias: &str) -> Option<VariantProteinAlias> {
    let trimmed = alias.trim();
    let caps = residue_alias_re().captures(trimmed)?;
    Some(VariantProteinAlias {
        position: caps[1].parse().ok()?,
        residue: caps[2].chars().next()?,
    })
}

fn parse_gene_residue_alias(query: &str) -> Option<(String, VariantProteinAlias)> {
    let trimmed = query.trim();
    let caps = gene_residue_re().captures(trimmed)?;
    Some((
        caps[1].to_string(),
        VariantProteinAlias {
            position: caps[2].parse().ok()?,
            residue: caps[3].chars().next()?,
        },
    ))
}

pub(crate) fn is_exact_gene_token(token: &str) -> bool {
    let mut chars = token.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_uppercase())
        && chars.clone().next().is_some()
        && chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

/// Transcript accessions are not gene symbols. A first token shaped like
/// `NM_000249.4` or `NM_177438.3(DICER1)` is a transcript anchor, so a
/// two-token gene+coding reading would half-parse a ClinVar-style name.
fn is_transcript_qualified_anchor(token: &str) -> bool {
    token.contains('(') || transcript_anchor_re().is_match(token)
}

fn transcript_anchor_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Z]{2}_[0-9]+").expect("valid regex"))
}

fn split_gene_change_tokens(input: &str) -> Option<(&str, &str)> {
    let mut parts = input.split_whitespace();
    let gene = parts.next()?;
    let change = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    Some((gene, change))
}

pub fn classify_variant_input(input: &str) -> VariantInputKind {
    if transcript_deletion_get::selects(input) {
        return transcript_deletion_get::prepare(input)
            .ok()
            .flatten()
            .map_or(VariantInputKind::Unsupported, |value| {
                VariantInputKind::Exact(value.format())
            });
    }
    if coding_get::selects(input) {
        return coding_get::prepare(input)
            .ok()
            .flatten()
            .map_or(VariantInputKind::Unsupported, |value| {
                VariantInputKind::Exact(value.format())
            });
    }
    if protein_get::selects(input) {
        return protein_get::prepare(input)
            .ok()
            .flatten()
            .map_or(VariantInputKind::Unsupported, |prepared| {
                VariantInputKind::Exact(prepared.format())
            });
    }
    let input = input.trim();
    if input.is_empty() {
        return VariantInputKind::Unsupported;
    }

    if let Some(caps) = rsid_re().captures(input) {
        return VariantInputKind::Exact(VariantIdFormat::RsId(caps[1].to_ascii_lowercase()));
    }
    if let Some(variation_id) = parse_clinvar_variation_id(input) {
        return VariantInputKind::Exact(VariantIdFormat::ClinvarVariationId(variation_id));
    }
    if let Ok(Some(lookup)) =
        genomic_lookup::genomic_lookup(input, genomic_lookup::LookupRoute::Direct)
        && let Ok(Some(id)) = lookup.exact_id()
    {
        return VariantInputKind::Exact(VariantIdFormat::HgvsGenomic(id.to_string()));
    }
    if transcript_coding_hgvs_re().is_match(input) {
        return VariantInputKind::TranscriptCodingHgvs(input.to_string());
    }
    if let Some(caps) = gene_protein_re().captures(input) {
        let Some(change) = point_alias::point_assertion(&caps[2], Some(&caps[1]), true).alias()
        else {
            return VariantInputKind::Unsupported;
        };
        return VariantInputKind::Exact(VariantIdFormat::GeneProteinChange {
            gene: caps[1].to_string(),
            change,
        });
    }
    if let Some(exact) = parse_exact_gene_protein_change(input) {
        return VariantInputKind::Exact(exact);
    }
    if let Some((gene, alias)) = parse_gene_residue_alias(input) {
        let alias_label = alias.label();
        return VariantInputKind::Shorthand(VariantShorthand::GeneResidueAlias {
            gene,
            alias: alias_label,
            position: alias.position,
            residue: alias.residue,
        });
    }
    if let Some(change) = normalize_protein_change(input) {
        return VariantInputKind::Shorthand(VariantShorthand::ProteinChangeOnly { change });
    }

    VariantInputKind::Unsupported
}

pub fn variant_guidance(input: &str) -> Option<VariantGuidance> {
    let query = input.trim();
    let shorthand = match classify_variant_input(query) {
        VariantInputKind::Shorthand(shorthand) => shorthand,
        _ => return None,
    };

    Some(match shorthand {
        VariantShorthand::GeneResidueAlias { gene, alias, .. } => VariantGuidance {
            query: query.to_string(),
            kind: VariantGuidanceKind::GeneResidueAlias {
                gene: gene.clone(),
                alias: alias.clone(),
            },
            next_commands: vec![
                format!(
                    "biomcp search variant {} --limit 10",
                    quote_command_arg(query)
                ),
                format!("biomcp search variant -g {gene} --limit 10"),
            ],
        },
        VariantShorthand::ProteinChangeOnly { change } => VariantGuidance {
            query: query.to_string(),
            kind: VariantGuidanceKind::ProteinChangeOnly {
                change: change.clone(),
            },
            next_commands: vec![
                format!("biomcp search variant --hgvsp {change} --limit 10"),
                format!("biomcp discover {}", quote_command_arg(query)),
            ],
        },
    })
}

pub fn parse_variant_id(id: &str) -> Result<VariantIdFormat, BioMcpError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Variant ID is required. Example: biomcp get variant rs113488022".into(),
        ));
    }

    if let VariantInputKind::Exact(exact) = classify_variant_input(id) {
        return Ok(exact);
    }

    let looks_like_search_phrase = {
        let lower = id.to_ascii_lowercase();
        [
            "exon",
            "deletion",
            "insertion",
            "duplication",
            "fusion",
            "rearrangement",
            "amplification",
            "splice",
            "promoter",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
    };

    let clinvar_name_working_form = clinvar_style_name_working_form(id);
    let search_hint = match classify_variant_input(id) {
        VariantInputKind::Shorthand(VariantShorthand::GeneResidueAlias { .. }) => format!(
            "\n\nThis looks like search-only shorthand, not an exact variant ID.\n\
Use `biomcp search variant \"{id}\"` to resolve it, or pass an exact rsID/HGVS/gene+protein change to `get variant`."
        ),
        VariantInputKind::Shorthand(VariantShorthand::ProteinChangeOnly { change }) => format!(
            "\n\nThis looks like search-only shorthand, not an exact variant ID.\n\
Try:\n\
1. biomcp search variant --hgvsp {change} --limit 10\n\
2. biomcp discover {change}"
        ),
        _ if clinvar_name_working_form.is_some() => format!(
            "\n\nThis looks like a ClinVar-style name with the gene in parentheses.\n\
BioMCP parses the colon form instead.\n\
Working form: biomcp get variant {}\n\
A ClinVar VariationID (e.g. biomcp get variant 577152) or rsID (e.g. biomcp get variant rs113488022) also works.",
            clinvar_name_working_form.expect("checked above")
        ),
        _ if looks_like_search_phrase => format!(
            "\n\nThis looks like a search phrase or alteration description, not an exact variant ID.\n\
Use `biomcp search variant \"{id}\"` to search, or pass an exact rsID/HGVS/gene+protein change to `get variant`."
        ),
        VariantInputKind::TranscriptCodingHgvs(_) => format!(
            "\n\nThis looks like transcript HGVS. `biomcp get variant` normalizes transcript HGVS before lookup; if normalization fails, try `biomcp variant normalize all {id}` first."
        ),
        _ => String::new(),
    };

    Err(BioMcpError::InvalidArgument(format!(
        "Unrecognized variant format: '{id}'{search_hint}\n\n\
Supported formats:\n\
- rsID: rs113488022\n\
- ClinVar VariationID: 577152\n\
- HGVS genomic: chr7:g.140453136A>T\n\
- Genomic indels: exact-copy repeat, range deletion, sequence-qualified deletion, duplication, insertion, inversion, and delins\n\
- Transcript HGVS: NM_004333.6:c.1799T>A\n\
- Gene + protein: BRAF V600E, BRAF p.Val600Glu, EGFR E746_A750del"
    )))
}

pub(crate) fn gnomad_variant_slug(id: &str) -> Option<String> {
    let VariantIdFormat::HgvsGenomic(hgvs) = parse_variant_id(id).ok()? else {
        return None;
    };
    let assertion =
        genomic_assertion::genomic_assertion(&hgvs, genomic_assertion::Admission::Chromosome, true);
    let components = assertion.components()?;
    Some(format!(
        "{}-{}-{}-{}",
        components.accession.strip_prefix("chr")?,
        components.position_lexeme,
        components.reference,
        components.alternate
    ))
}

pub(crate) fn protein_change_segment(value: &str) -> &str {
    let trimmed = value.trim();
    trimmed
        .rsplit_once(":p.")
        .map(|(_, change)| change)
        .unwrap_or(trimmed)
}

fn protein_alias_body(value: &str) -> &str {
    protein_change_segment(value)
        .trim()
        .strip_prefix("p.")
        .or_else(|| protein_change_segment(value).trim().strip_prefix("P."))
        .unwrap_or_else(|| protein_change_segment(value).trim())
}

pub(crate) fn protein_changes_equivalent(left: &str, right: &str) -> bool {
    // Retained identical complex-body comparison claims no shared point payload.
    if protein_alias_body(left).eq_ignore_ascii_case(protein_alias_body(right)) {
        return true;
    }
    let point_equal = normalize_protein_change(left).zip(normalize_protein_change(right));
    let point_equal = point_equal.is_some_and(|(left, right)| left == right);
    point_equal || interval_comparison::compare(left, right).equivalent
}

pub(crate) fn normalize_protein_change(value: &str) -> Option<String> {
    point_alias::point_assertion(value, None, false).alias()
}

/// Search filters normalize only complete supported point substitutions.
pub(crate) fn normalize_protein_substitution(value: &str) -> Option<String> {
    point_alias::point_assertion(value, None, false).alias()
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct VariantArticleRequest {
    pub request_id: Option<String>,
    pub gene: Option<String>,
    pub protein: Option<String>,
    pub coding: Option<String>,
    pub transcript: Option<String>,
    pub genomic: Option<String>,
    pub accession: Option<String>,
    pub build: Option<String>,
    pub position: Option<u64>,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    pub alt: Option<String>,
    pub rsid: Option<String>,
}

impl VariantArticleRequest {
    fn clean(value: &Option<String>) -> Option<String> {
        value
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    pub(crate) fn requested_identity(&self) -> RequestedVariantIdentity {
        let mut identity = RequestedVariantIdentity {
            gene: Self::clean(&self.gene),
            protein_change: Self::clean(&self.protein),
            coding_change: Self::clean(&self.coding),
            transcript: Self::clean(&self.transcript),
            clinvar_variation_id: None,
            genomic_accession: Self::clean(&self.accession),
            genome_build: Self::clean(&self.build),
            position: self.position,
            reference: Self::clean(&self.reference),
            alternate: Self::clean(&self.alt),
            rsid: Self::clean(&self.rsid),
        };
        if let Some(genomic) = Self::clean(&self.genomic) {
            identity.populate_structured_genomic(&genomic);
        }
        identity
    }

    pub(crate) fn validate_identity(&self) -> Result<RequestedVariantIdentity, BioMcpError> {
        let component_genomic = self.accession.is_some()
            || self.position.is_some()
            || self.reference.is_some()
            || self.alt.is_some();
        let structured_genomic = component_genomic || self.build.is_some();
        if self.genomic.is_some() && component_genomic {
            return Err(BioMcpError::InvalidArgument(
                "variant article item cannot combine genomic with accession, position, ref, or alt"
                    .into(),
            ));
        }
        let identity = self.requested_identity();
        let complete_genomic = identity.genomic_accession.is_some()
            && identity.position.is_some()
            && identity.reference.is_some()
            && identity.alternate.is_some();
        if (structured_genomic || self.genomic.is_some()) && !complete_genomic {
            return Err(BioMcpError::InvalidArgument(
                "genomic identity requires a complete genomic HGVS or accession, position, ref, and alt"
                    .into(),
            ));
        }
        if complete_genomic
            && (identity.position == Some(0)
                || !genomic_assertion::structured_identity_admitted(&identity))
        {
            return Err(BioMcpError::InvalidArgument(
                "variant article item genomic identity must use chr or versioned RefSeq coordinates, a positive position, and A/C/G/T ref and alt bases"
                    .into(),
            ));
        }
        if identity.genome_build.as_deref().is_some_and(|build| {
            !build.eq_ignore_ascii_case("GRCh37") && !build.eq_ignore_ascii_case("GRCh38")
        }) {
            return Err(BioMcpError::InvalidArgument(
                "variant article item build must be GRCh37 or GRCh38".into(),
            ));
        }
        if identity
            .genomic_accession
            .as_deref()
            .is_some_and(|accession| refseq_accession_re().is_match(accession))
            && identity.genome_build.is_none()
        {
            return Err(BioMcpError::InvalidArgument(
                "variant article item versioned RefSeq identity requires an explicit GRCh37 or GRCh38 build"
                    .into(),
            ));
        }
        for (name, supplied, cleaned) in [
            ("gene", self.gene.is_some(), identity.gene.as_ref()),
            (
                "coding",
                self.coding.is_some(),
                identity.coding_change.as_ref(),
            ),
            (
                "transcript",
                self.transcript.is_some(),
                identity.transcript.as_ref(),
            ),
            (
                "accession",
                self.accession.is_some(),
                identity.genomic_accession.as_ref(),
            ),
            (
                "build",
                self.build.is_some(),
                identity.genome_build.as_ref(),
            ),
            ("ref", self.reference.is_some(), identity.reference.as_ref()),
            ("alt", self.alt.is_some(), identity.alternate.as_ref()),
        ] {
            if supplied && cleaned.is_none() {
                return Err(BioMcpError::InvalidArgument(format!(
                    "variant article item field {name} must not be empty"
                )));
            }
        }
        if self.protein.is_some()
            && identity
                .protein_change
                .as_deref()
                .and_then(normalize_protein_change)
                .is_none()
        {
            return Err(BioMcpError::InvalidArgument(
                "variant article item protein must be a complete protein change".into(),
            ));
        }
        if identity.transcript.as_deref().is_some_and(|transcript| {
            !transcript_coding_hgvs_re().is_match(&format!("{transcript}:c.1A>T"))
        }) {
            return Err(BioMcpError::InvalidArgument(
                "variant article item transcript must be a versioned transcript accession".into(),
            ));
        }
        if let Some(coding) = identity.coding_change.as_deref() {
            let segment = coding
                .rsplit_once(':')
                .map(|(_, value)| value)
                .unwrap_or(coding);
            let complete = coding_change_re().is_match(segment);
            let transcript_compatible = identity.transcript.as_deref().is_none_or(|transcript| {
                transcript_coding_hgvs_re().is_match(&format!("{transcript}:{segment}"))
            });
            if !complete || !transcript_compatible {
                return Err(BioMcpError::InvalidArgument(
                    "variant article item coding must be a complete coding change".into(),
                ));
            }
        }
        if self.rsid.is_some() && !identity.rsid.as_deref().is_some_and(is_rsid) {
            return Err(BioMcpError::InvalidArgument(
                "variant article item rsid must be a valid rsID".into(),
            ));
        }
        let usable = identity.rsid.as_deref().is_some_and(is_rsid)
            || complete_genomic
            || (identity.gene.is_some()
                && identity
                    .protein_change
                    .as_deref()
                    .and_then(normalize_protein_change)
                    .is_some())
            || (identity.coding_change.is_some()
                && (identity.gene.is_some() || identity.transcript.is_some()));
        if !usable {
            return Err(BioMcpError::InvalidArgument(
                "variant article item needs an rsID, complete genomic identity, gene plus protein, or coding plus gene/transcript"
                    .into(),
            ));
        }
        Ok(identity)
    }

    pub(crate) fn display_input(&self, identity: &RequestedVariantIdentity) -> String {
        Self::clean(&self.genomic)
            .or_else(|| genomic_alias(identity))
            .or_else(|| identity.rsid.clone())
            .or_else(|| {
                identity.protein_change.as_ref().map(|change| {
                    identity
                        .gene
                        .as_ref()
                        .map(|gene| format!("{gene} {change}"))
                        .unwrap_or_else(|| change.clone())
                })
            })
            .or_else(|| {
                identity.coding_change.as_ref().map(|change| {
                    identity
                        .gene
                        .as_ref()
                        .or(identity.transcript.as_ref())
                        .map(|anchor| format!("{anchor} {change}"))
                        .unwrap_or_else(|| change.clone())
                })
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RequestedVariantIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protein_change: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coding_change: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clinvar_variation_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genomic_accession: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genome_build: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rsid: Option<String>,
}

impl RequestedVariantIdentity {
    pub(crate) fn human_label(&self) -> String {
        if let (Some(gene), Some(change)) = (&self.gene, &self.protein_change) {
            return format!("{gene} {change}");
        }
        if let (Some(gene), Some(change)) = (&self.gene, &self.coding_change) {
            return format!("{gene} {change}");
        }
        if let (Some(transcript), Some(change)) = (&self.transcript, &self.coding_change) {
            return format!("{transcript}:{change}");
        }
        if let Some(variation_id) = &self.clinvar_variation_id {
            return format!("ClinVar VariationID {variation_id}");
        }
        if let Some(rsid) = &self.rsid {
            return rsid.clone();
        }
        genomic_alias(self).unwrap_or_else(|| "unknown variant".into())
    }

    pub(crate) fn for_search(
        gene: Option<String>,
        protein_change: Option<String>,
        coding_change: Option<String>,
        rsid: Option<String>,
    ) -> Self {
        Self {
            gene,
            protein_change,
            coding_change,
            rsid,
            ..Self::default()
        }
    }

    pub(crate) fn populate_genomic(&mut self, value: &str) {
        self.populate_genomic_assertion(value, genomic_assertion::Admission::Chromosome);
    }

    fn populate_structured_genomic(&mut self, value: &str) {
        self.populate_genomic_assertion(value, genomic_assertion::Admission::Structured);
    }

    fn populate_genomic_assertion(&mut self, value: &str, admission: genomic_assertion::Admission) {
        let assertion = genomic_assertion::genomic_assertion(value, admission, true);
        let Some(components) = assertion.components() else {
            return;
        };
        self.genomic_accession = Some(components.accession.to_string());
        self.position = components.position_lexeme.parse().ok();
        self.reference = Some(components.reference.to_string());
        self.alternate = Some(components.alternate.to_string());
    }

    pub(crate) fn is_authoritative_refseq(&self) -> bool {
        self.genomic_accession
            .as_deref()
            .is_some_and(|accession| refseq_accession_re().is_match(accession))
            && self.genome_build.is_some()
            && self.position.is_some()
            && self.reference.is_some()
            && self.alternate.is_some()
    }

    pub(crate) fn normalized_aliases(&self) -> NormalizedVariantAliases {
        NormalizedVariantAliases {
            protein_changes: self
                .protein_change
                .as_deref()
                .and_then(normalize_protein_change)
                .into_iter()
                .collect(),
            coding_changes: self.coding_change.clone().into_iter().collect(),
            genomic_ids: genomic_alias(self).into_iter().collect(),
            rsids: self
                .rsid
                .as_deref()
                .map(|v| v.to_ascii_lowercase())
                .into_iter()
                .collect(),
        }
    }
}

fn genomic_alias(identity: &RequestedVariantIdentity) -> Option<String> {
    Some(format!(
        "{}:g.{}{}>{}",
        identity.genomic_accession.as_deref()?,
        identity.position?,
        identity.reference.as_deref()?,
        identity.alternate.as_deref()?
    ))
}

fn coding_change_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^c\.[*+-]?\d+(?:[+-]\d+)?(?:_[*+-]?\d+(?:[+-]\d+)?)?(?:[ACGT]+>[ACGT]+|del(?:[ACGT]+)?|dup(?:[ACGT]+)?|ins[ACGT]+|delins[ACGT]+)$",
        )
        .expect("valid regex")
    })
}

pub(crate) fn coding_change_segment(value: &str) -> &str {
    let trimmed = value.trim();
    trimmed
        .rsplit_once(':')
        .filter(|(_, change)| change.to_ascii_lowercase().starts_with("c."))
        .map(|(_, change)| change)
        .unwrap_or(trimmed)
}

fn transcript_prefix(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    trimmed
        .split_once(":c.")
        .or_else(|| trimmed.split_once(":p."))
        .map(|(prefix, _)| prefix)
        .filter(|prefix| !prefix.is_empty())
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NormalizedVariantAliases {
    pub protein_changes: Vec<String>,
    pub coding_changes: Vec<String>,
    pub genomic_ids: Vec<String>,
    pub rsids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SourceVariantIdentity {
    pub genomic_id: String,
    pub genome_build: String,
    pub genes: Vec<String>,
    pub protein_changes: Vec<String>,
    pub coding_changes: Vec<String>,
    pub rsids: Vec<String>,
}

impl SourceVariantIdentity {
    pub(crate) fn from_myvariant_hit(hit: &crate::sources::myvariant::MyVariantHit) -> Self {
        let (genes, protein_changes, coding_changes) = hit
            .dbnsfp
            .as_ref()
            .map(|db| {
                (
                    db.genename().values().to_vec(),
                    db.hgvsp().values().to_vec(),
                    db.hgvsc().values().to_vec(),
                )
            })
            .unwrap_or_default();
        let rsids = hit
            .dbsnp
            .as_ref()
            .and_then(|db| db.rsid().map(str::to_owned))
            .into_iter()
            .collect();
        Self {
            genomic_id: hit.id.clone(),
            genome_build: "GRCh37".into(),
            genes,
            protein_changes,
            coding_changes,
            rsids,
        }
    }

    pub(crate) fn normalized_key(&self) -> String {
        let mut genes = normalized_set(&self.genes, |v| Some(v.trim().to_ascii_uppercase()));
        let mut proteins = normalized_set(&self.protein_changes, normalize_protein_change);
        let mut coding = normalized_set(&self.coding_changes, coding_key);
        let mut rsids = normalized_set(&self.rsids, |v| Some(v.trim().to_ascii_lowercase()));
        genes.sort();
        proteins.sort();
        coding.sort();
        rsids.sort();
        format!(
            "{}|{}|{}|{}|{}|{}",
            self.genomic_id.to_ascii_uppercase(),
            self.genome_build.to_ascii_uppercase(),
            genes.join(","),
            proteins.join(","),
            coding.join(","),
            rsids.join(",")
        )
    }
}

fn normalized_set<F>(values: &[String], normalize: F) -> Vec<String>
where
    F: Fn(&str) -> Option<String>,
{
    values
        .iter()
        .filter_map(|v| normalize(v))
        .filter(|v| !v.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VariantIdentityComparison {
    Compatible { matched_alias: String },
    Contradictory { field: &'static str },
    Indeterminate { field: &'static str },
}

pub(crate) fn compare_variant_identity(
    requested: &RequestedVariantIdentity,
    source: &SourceVariantIdentity,
) -> VariantIdentityComparison {
    let mut indeterminate = None;
    let mut matched_alias = None;
    let requested_gene = requested
        .gene
        .as_deref()
        .map(|v| v.trim().to_ascii_uppercase());
    let genes = normalized_set(&source.genes, |v| Some(v.trim().to_ascii_uppercase()));
    if let Some(gene) = requested_gene.as_deref() {
        if genes.is_empty() {
            indeterminate = Some("gene");
        } else if !genes.iter().any(|v| v == gene) {
            return VariantIdentityComparison::Contradictory { field: "gene" };
        }
    }
    if requested_gene.is_some()
        && (requested.protein_change.is_some() || requested.coding_change.is_some())
        && genes.len() > 1
    {
        indeterminate = Some("gene_annotation_tuple");
    }
    if let Some(value) = requested.protein_change.as_deref() {
        if let Some(alias) = source
            .protein_changes
            .iter()
            .find(|alias| protein_changes_equivalent(alias, value))
        {
            matched_alias = Some(alias.clone());
        } else if let Some(want) = normalize_protein_change(value) {
            let usable = source
                .protein_changes
                .iter()
                .filter_map(|alias| normalize_protein_change(alias).map(|v| (alias, v)))
                .collect::<Vec<_>>();
            if usable.is_empty() {
                indeterminate = Some("protein_change");
            } else if let Some((alias, _)) = usable.iter().find(|(_, value)| value == &want) {
                matched_alias = Some((*alias).clone());
            } else {
                return VariantIdentityComparison::Contradictory {
                    field: "protein_change",
                };
            }
        } else if source.protein_changes.is_empty() {
            indeterminate = Some("protein_change");
        } else {
            return VariantIdentityComparison::Contradictory {
                field: "protein_change",
            };
        }
    }
    if let Some(value) = requested.coding_change.as_deref() {
        let wanted = coding_key(value);
        let usable = source
            .coding_changes
            .iter()
            .filter_map(|alias| coding_key(alias).map(|key| (alias, key)))
            .collect::<Vec<_>>();
        if let Some(wanted) = wanted.filter(|_| !usable.is_empty()) {
            if let Some((alias, _)) = usable
                .iter()
                .find(|(alias, _)| alias.trim() == value.trim())
                .or_else(|| usable.iter().find(|(_, key)| key == &wanted))
            {
                matched_alias.get_or_insert_with(|| (*alias).clone());
            } else {
                return VariantIdentityComparison::Contradictory {
                    field: "coding_change",
                };
            }
        } else {
            indeterminate = Some("coding_change");
        }
    }
    if let Some(value) = requested.transcript.as_deref() {
        let transcripts = source
            .coding_changes
            .iter()
            .chain(&source.protein_changes)
            .filter_map(|alias| transcript_prefix(alias).map(str::to_string))
            .collect::<Vec<_>>();
        if transcripts.is_empty() {
            indeterminate = Some("transcript");
        } else if !transcripts
            .iter()
            .any(|alias| alias.eq_ignore_ascii_case(value))
        {
            return VariantIdentityComparison::Contradictory {
                field: "transcript",
            };
        }
    }
    let assertion = genomic_assertion::genomic_assertion(
        &source.genomic_id,
        genomic_assertion::Admission::Source,
        true,
    );
    let source_genomic = assertion.source_assertion();
    let source_genomic = source_genomic
        .as_ref()
        .map(|source| source.comparison_fields())
        .unwrap_or(genomic_assertion::LegacyComparisonFields {
            build: assertion.build,
            ..Default::default()
        });
    macro_rules! compare_genomic_field {
        ($requested:expr, $source:expr, $field:literal, $matches:expr) => {
            if let Some(wanted) = $requested {
                match $source {
                    None => indeterminate = Some($field),
                    Some(actual) if ($matches)(wanted, actual) => {}
                    Some(_) => return VariantIdentityComparison::Contradictory { field: $field },
                }
            }
        };
    }
    compare_genomic_field!(
        requested.genome_build.as_deref(),
        source_genomic.build,
        "genome_build",
        |wanted: &str, actual: &str| wanted.eq_ignore_ascii_case(actual)
    );
    compare_genomic_field!(
        requested.genomic_accession.as_deref(),
        source_genomic.accession,
        "genomic_accession",
        |wanted: &str, actual: &str| wanted.eq_ignore_ascii_case(actual)
    );
    compare_genomic_field!(
        requested.position,
        source_genomic.position,
        "position",
        |wanted: u64, actual: u64| wanted == actual
    );
    compare_genomic_field!(
        requested.reference.as_deref(),
        source_genomic.reference,
        "reference",
        |wanted: &str, actual: &str| wanted.eq_ignore_ascii_case(actual)
    );
    compare_genomic_field!(
        requested.alternate.as_deref(),
        source_genomic.alternate,
        "alternate",
        |wanted: &str, actual: &str| wanted.eq_ignore_ascii_case(actual)
    );
    if requested.genomic_accession.is_some()
        || requested.position.is_some()
        || requested.reference.is_some()
        || requested.alternate.is_some()
    {
        matched_alias.get_or_insert(source.genomic_id.clone());
    }
    if let Some(value) = requested.rsid() {
        if source.rsids.is_empty() {
            indeterminate = Some("rsid");
        } else if let Some(alias) = source.rsids.iter().find(|v| v.eq_ignore_ascii_case(value)) {
            matched_alias.get_or_insert(alias.clone());
        } else {
            return VariantIdentityComparison::Contradictory { field: "rsid" };
        }
    }
    if let Some(field) = indeterminate {
        VariantIdentityComparison::Indeterminate { field }
    } else {
        VariantIdentityComparison::Compatible {
            matched_alias: matched_alias.unwrap_or_else(|| source.genomic_id.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum VariantResolutionStatus {
    Resolved,
    Ambiguous,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct VariantSearchResolution {
    pub status: VariantResolutionStatus,
    pub normalized_aliases: NormalizedVariantAliases,
    pub exhaustive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VariantArticleResolutionBasis {
    CallerSupplied,
    ProviderConfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VariantProviderValidationStatus {
    Confirmed,
    NotFound,
    Indeterminate,
    Contradictory,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct VariantProviderValidation {
    pub source: String,
    pub status: VariantProviderValidationStatus,
    pub matched_alias: Option<String>,
    pub contradictory_field: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct VariantArticleResolution {
    pub status: VariantResolutionStatus,
    pub normalized_aliases: NormalizedVariantAliases,
    pub exhaustive: bool,
    pub basis: Option<VariantArticleResolutionBasis>,
    pub provider_validation: VariantProviderValidation,
}

#[derive(Debug, Clone)]
pub(crate) struct VariantArticleResolutionContext {
    pub requested: RequestedVariantIdentity,
    pub resolution: VariantArticleResolution,
    pub source_id: Option<String>,
    pub source_identity: Option<SourceVariantIdentity>,
    pub source_hit: Option<crate::sources::myvariant::MyVariantHit>,
    pub fallback_source_identities: Vec<SourceVariantIdentity>,
    pub available: bool,
}

#[cfg(test)]
pub(super) mod tests;
