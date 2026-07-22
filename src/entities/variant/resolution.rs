//! rsID, HGVS, and protein change parsing and classification.

use regex::Regex;
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::error::BioMcpError;

use super::{
    VariantGuidance, VariantGuidanceKind, VariantIdFormat, VariantInputKind, VariantProteinAlias,
    VariantShorthand, transcript_coding_hgvs_re,
};

fn rsid_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^(rs\d+)$").expect("valid regex"))
}

pub(crate) fn is_rsid(value: &str) -> bool {
    rsid_re().is_match(value.trim())
}

fn hgvs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(chr[0-9XYM]+:g\.\d+[ACGT]>[ACGT])$").expect("valid regex"))
}

pub(in crate::entities::variant) fn hgvs_coords_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(chr[0-9XYM]+):g\.(\d+)([ACGT])>([ACGT])$").expect("valid regex")
    })
}

fn structured_genomic_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^((?:chr[0-9XYM]+)|(?:NC_[0-9]+\.[0-9]+)):g\.(\d+)([ACGT])>([ACGT])$")
            .expect("valid regex")
    })
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

fn is_exact_gene_token(token: &str) -> bool {
    let mut chars = token.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_uppercase())
        && chars.clone().next().is_some()
        && chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
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

fn parse_exact_gene_protein_change(input: &str) -> Option<VariantIdFormat> {
    let (gene, change) = split_gene_change_tokens(input)?;
    if !is_exact_gene_token(gene) {
        return None;
    }
    let change = normalize_protein_change(change)?;
    Some(VariantIdFormat::GeneProteinChange {
        gene: gene.to_string(),
        change,
    })
}

pub fn classify_variant_input(input: &str) -> VariantInputKind {
    let input = input.trim();
    if input.is_empty() {
        return VariantInputKind::Unsupported;
    }

    if let Some(caps) = rsid_re().captures(input) {
        return VariantInputKind::Exact(VariantIdFormat::RsId(caps[1].to_ascii_lowercase()));
    }
    if let Some(caps) = hgvs_re().captures(input) {
        return VariantInputKind::Exact(VariantIdFormat::HgvsGenomic(caps[1].to_string()));
    }
    if transcript_coding_hgvs_re().is_match(input) {
        return VariantInputKind::TranscriptCodingHgvs(input.to_string());
    }
    if let Some(caps) = gene_protein_re().captures(input) {
        return VariantInputKind::Exact(VariantIdFormat::GeneProteinChange {
            gene: caps[1].to_string(),
            change: caps[2].to_string(),
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
- HGVS genomic: chr7:g.140453136A>T\n\
- Transcript HGVS: NM_004333.6:c.1799T>A\n\
- Gene + protein: BRAF V600E, BRAF p.Val600Glu"
    )))
}

pub(crate) fn gnomad_variant_slug(id: &str) -> Option<String> {
    let VariantIdFormat::HgvsGenomic(hgvs) = parse_variant_id(id).ok()? else {
        return None;
    };
    let caps = hgvs_coords_re().captures(&hgvs)?;
    Some(format!(
        "{}-{}-{}-{}",
        &caps[1][3..],
        &caps[2],
        &caps[3],
        &caps[4]
    ))
}

fn amino_acid_one_letter(token: &str) -> Option<char> {
    match token.trim().to_ascii_uppercase().as_str() {
        "A" | "ALA" => Some('A'),
        "R" | "ARG" => Some('R'),
        "N" | "ASN" => Some('N'),
        "D" | "ASP" => Some('D'),
        "C" | "CYS" => Some('C'),
        "Q" | "GLN" => Some('Q'),
        "E" | "GLU" => Some('E'),
        "G" | "GLY" => Some('G'),
        "H" | "HIS" => Some('H'),
        "I" | "ILE" => Some('I'),
        "L" | "LEU" => Some('L'),
        "K" | "LYS" => Some('K'),
        "M" | "MET" => Some('M'),
        "F" | "PHE" => Some('F'),
        "P" | "PRO" => Some('P'),
        "S" | "SER" => Some('S'),
        "T" | "THR" => Some('T'),
        "W" | "TRP" => Some('W'),
        "Y" | "TYR" => Some('Y'),
        "V" | "VAL" => Some('V'),
        "*" | "TER" | "STOP" | "X" => Some('*'),
        _ => None,
    }
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

pub(crate) fn normalize_protein_change(value: &str) -> Option<String> {
    let trimmed = protein_alias_body(value);
    if trimmed.is_empty() {
        return None;
    }

    let bytes = trimmed.as_bytes();
    let start_digits = bytes.iter().position(|b| b.is_ascii_digit())?;
    let end_digits = bytes[start_digits..]
        .iter()
        .position(|b| !b.is_ascii_digit())
        .map(|idx| start_digits + idx)
        .unwrap_or(bytes.len());
    if start_digits == 0 || end_digits <= start_digits || end_digits >= bytes.len() {
        return None;
    }

    let from = amino_acid_one_letter(&trimmed[..start_digits])?;
    let pos = trimmed[start_digits..end_digits].trim();
    let to = amino_acid_one_letter(&trimmed[end_digits..])?;
    if pos.is_empty() {
        return None;
    }

    Some(format!("{from}{pos}{to}"))
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
                || genomic_alias(&identity)
                    .as_deref()
                    .is_none_or(|value| !structured_genomic_re().is_match(value)))
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
            VariantInputKind::TranscriptCodingHgvs(value) => {
                let (transcript, coding) = value.split_once(':').unwrap_or(("", value.as_str()));
                Ok(Self {
                    transcript: (!transcript.is_empty()).then(|| transcript.to_string()),
                    coding_change: Some(coding.to_string()),
                    ..Self::default()
                })
            }
            _ => Err(BioMcpError::InvalidArgument(format!(
                "Unrecognized variant format: '{supplied}'"
            ))),
        }
    }

    pub(crate) fn populate_genomic(&mut self, value: &str) {
        let Some(caps) = hgvs_coords_re().captures(value.trim()) else {
            return;
        };
        self.populate_genomic_captures(&caps);
    }

    fn populate_structured_genomic(&mut self, value: &str) {
        let Some(caps) = structured_genomic_re().captures(value.trim()) else {
            return;
        };
        self.populate_genomic_captures(&caps);
    }

    fn populate_genomic_captures(&mut self, caps: &regex::Captures<'_>) {
        self.genomic_accession = Some(caps[1].to_string());
        self.position = caps[2].parse().ok();
        self.reference = Some(caps[3].to_string());
        self.alternate = Some(caps[4].to_string());
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

#[derive(Default)]
struct GenomicComponents<'a> {
    build: Option<&'a str>,
    accession: Option<&'a str>,
    position: Option<u64>,
    reference: Option<&'a str>,
    alternate: Option<&'a str>,
}

fn genomic_components(value: &str) -> GenomicComponents<'_> {
    let trimmed = value.trim();
    let (build, hgvs) = match trimmed.split_once(':') {
        Some((prefix, rest))
            if prefix.eq_ignore_ascii_case("GRCh37") || prefix.eq_ignore_ascii_case("GRCh38") =>
        {
            (Some(prefix), rest)
        }
        _ => (None, trimmed),
    };
    let Some((accession, change)) = hgvs.split_once(":g.") else {
        return GenomicComponents {
            build,
            ..Default::default()
        };
    };
    let Some(separator) = change.find('>') else {
        return GenomicComponents {
            build,
            accession: Some(accession),
            ..Default::default()
        };
    };
    let left = &change[..separator];
    let alternate = &change[separator + 1..];
    let first_base = left.find(|ch: char| !ch.is_ascii_digit());
    let (position, reference) = first_base
        .map(|index| (left[..index].parse().ok(), Some(&left[index..])))
        .unwrap_or((None, None));
    GenomicComponents {
        build,
        accession: Some(accession),
        position,
        reference,
        alternate: (!alternate.is_empty()).then_some(alternate),
    }
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

fn coding_change_segment(value: &str) -> &str {
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
                    db.genename.clone().into_vec(),
                    db.hgvsp.clone().into_vec(),
                    db.hgvsc.clone().into_vec(),
                )
            })
            .unwrap_or_default();
        let rsids = hit
            .dbsnp
            .as_ref()
            .and_then(|db| db.rsid.clone())
            .into_iter()
            .collect();
        Self {
            genomic_id: hit.id.clone(),
            genes,
            protein_changes,
            coding_changes,
            rsids,
        }
    }

    pub(crate) fn normalized_key(&self) -> String {
        let mut genes = normalized_set(&self.genes, |v| Some(v.trim().to_ascii_uppercase()));
        let mut proteins = normalized_set(&self.protein_changes, normalize_protein_change);
        let mut coding = normalized_set(&self.coding_changes, |v| {
            Some(coding_change_segment(v).to_ascii_uppercase())
        });
        let mut rsids = normalized_set(&self.rsids, |v| Some(v.trim().to_ascii_lowercase()));
        genes.sort();
        proteins.sort();
        coding.sort();
        rsids.sort();
        format!(
            "{}|{}|{}|{}|{}",
            self.genomic_id.to_ascii_uppercase(),
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
            .find(|alias| protein_alias_body(alias).eq_ignore_ascii_case(protein_alias_body(value)))
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
        let wanted = coding_change_segment(value).to_ascii_uppercase();
        let usable = source
            .coding_changes
            .iter()
            .filter(|v| !coding_change_segment(v).is_empty())
            .collect::<Vec<_>>();
        if usable.is_empty() {
            indeterminate = Some("coding_change");
        } else if let Some(alias) = usable
            .iter()
            .find(|v| v.trim() == value.trim())
            .or_else(|| {
                usable
                    .iter()
                    .find(|v| coding_change_segment(v).to_ascii_uppercase() == wanted)
            })
        {
            matched_alias.get_or_insert_with(|| (*alias).clone());
        } else {
            return VariantIdentityComparison::Contradictory {
                field: "coding_change",
            };
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
    let source_genomic = genomic_components(&source.genomic_id);
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
    if let Some(value) = requested.rsid.as_deref() {
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
mod tests;
