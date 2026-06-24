//! rsID, HGVS, and protein change parsing and classification.

use regex::Regex;
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

pub(crate) fn normalize_protein_change(value: &str) -> Option<String> {
    let trimmed = value
        .trim()
        .trim_start_matches("p.")
        .trim_start_matches("P.");
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

#[cfg(test)]
mod tests;
