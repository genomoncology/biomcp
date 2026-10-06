//! Product point preparation around the checked BioData parser.
//! Compatibility classes remain until a reviewed contract retires their behavior.
use biodata::{
    HgvsProteinPointEdit, HgvsProteinPointEnvelope, HgvsProteinPointError, HgvsProteinResidue,
    HgvsProteinStyle, parse_hgvs_protein_point_21_1_4,
};
use regex::Regex;
use std::{fmt, sync::OnceLock};

// Existing HGVS 21.1.4 parser ceilings; these do not change public admission.
const INPUT_LIMIT: usize = 1024 * 1024;
const REFERENCE_LIMIT: usize = 4 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Preparation {
    BarePrefix,
    OuterWhitespace,
    UppercasePrefix,
    LegacyResidueSpelling,
    LegacyResidueTokenTrim,
    LegacyEmptyReferencePrefix,
    LegacyUncheckedReferencePrefix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Compatibility {
    EqualSubstitution,
    ZeroPosition,
    InitiationChange,
    TerminationReference,
    OverLimit,
    DirectRegex,
}

pub(super) enum PointRoute {
    Checked(HgvsProteinPointEnvelope),
    Compatibility(Compatibility, String),
    Refused,
}

/// Original text, gene assertion and prepared syntax have separate custody.
pub(super) struct PointAssertion<'a> {
    pub(super) source: &'a str,
    pub(super) gene: Option<&'a str>,
    pub(super) reference_prefix: Option<&'a str>,
    pub(super) preparations: Vec<Preparation>,
    pub(super) route: PointRoute,
}

impl fmt::Debug for PointAssertion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let compatibility = match &self.route {
            PointRoute::Compatibility(class, _) => Some(class),
            _ => None,
        };
        let route = match &self.route {
            PointRoute::Checked(envelope) => envelope.disposition().code(),
            PointRoute::Compatibility(_, _) => "product_point_compatibility",
            PointRoute::Refused => "product_point_refused",
        };
        f.debug_struct("PointAssertion")
            .field("source_bytes", &self.source.len())
            .field("gene_present", &self.gene.is_some())
            .field("reference_bytes", &self.reference_prefix.map(str::len))
            .field("preparations", &self.preparations)
            .field("route", &route)
            .field("compatibility", &compatibility)
            .finish()
    }
}

impl PointAssertion<'_> {
    pub(super) fn alias(&self) -> Option<String> {
        match &self.route {
            PointRoute::Checked(envelope) => {
                let point = envelope.disposition().parsed()?;
                if point.is_predicted()
                    || !matches!(point.edit(), HgvsProteinPointEdit::Substitution { .. })
                {
                    return None;
                }
                let rendered = point.render_constructed(HgvsProteinStyle::OneLetter).ok()?;
                Some(super::protein_alias_body(&rendered).to_owned())
            }
            // Every class is selected before parsing. No refusal retries here.
            PointRoute::Compatibility(_, alias) => Some(alias.clone()),
            PointRoute::Refused => None,
        }
    }
}

fn tokens_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^([A-Za-z*]+)(\s*)(\d+)(\s*)([A-Za-z*]+)$")
            .expect("valid point preparation regex")
    })
}

fn residue(token: &str) -> Option<(&'static str, &'static str)> {
    Some(match token.to_ascii_uppercase().as_str() {
        "A" | "ALA" => ("A", "Ala"),
        "R" | "ARG" => ("R", "Arg"),
        "N" | "ASN" => ("N", "Asn"),
        "D" | "ASP" => ("D", "Asp"),
        "C" | "CYS" => ("C", "Cys"),
        "Q" | "GLN" => ("Q", "Gln"),
        "E" | "GLU" => ("E", "Glu"),
        "G" | "GLY" => ("G", "Gly"),
        "H" | "HIS" => ("H", "His"),
        "I" | "ILE" => ("I", "Ile"),
        "L" | "LEU" => ("L", "Leu"),
        "K" | "LYS" => ("K", "Lys"),
        "M" | "MET" => ("M", "Met"),
        "F" | "PHE" => ("F", "Phe"),
        "P" | "PRO" => ("P", "Pro"),
        "S" | "SER" => ("S", "Ser"),
        "T" | "THR" => ("T", "Thr"),
        "W" | "TRP" => ("W", "Trp"),
        "Y" | "TYR" => ("Y", "Tyr"),
        "V" | "VAL" => ("V", "Val"),
        "*" | "TER" | "STOP" | "X" => ("*", "Ter"),
        _ => return None,
    })
}

fn canonical_token<'a>(written: &'a str, token: (&'static str, &'static str)) -> &'a str {
    if written == token.0 || written == token.1 {
        written
    } else {
        token.1
    }
}

/// Only an already matched direct gene regex may admit arbitrary uppercase letters.
pub(super) fn point_assertion<'a>(
    source: &'a str,
    gene: Option<&'a str>,
    direct_regex: bool,
) -> PointAssertion<'a> {
    let mut assertion = PointAssertion {
        source,
        gene,
        reference_prefix: None,
        preparations: Vec::new(),
        route: PointRoute::Refused,
    };
    let trimmed = source.trim();
    if trimmed != source {
        assertion.preparations.push(Preparation::OuterWhitespace);
    }
    let (reference, change) = if let Some((prefix, change)) = trimmed.rsplit_once(":p.") {
        assertion.reference_prefix = Some(prefix);
        (Some(prefix), change)
    } else {
        (None, trimmed)
    };
    let body = if let Some(body) = change.strip_prefix("p.") {
        body
    } else if let Some(body) = change.strip_prefix("P.") {
        assertion.preparations.push(Preparation::UppercasePrefix);
        body
    } else {
        if reference.is_none() {
            assertion.preparations.push(Preparation::BarePrefix);
        }
        change
    };
    let token_outer_whitespace = body != body.trim();
    let body = body.trim();
    let Some(tokens) = tokens_re().captures(body) else {
        return assertion;
    };
    let direct_regex = direct_regex
        && gene.is_some()
        && reference.is_none()
        && change == body
        && tokens[1].len() == 1
        && tokens[5].len() == 1
        && tokens[1].bytes().all(|b| b.is_ascii_uppercase())
        && tokens[5]
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b == b'*')
        && tokens[2].is_empty()
        && tokens[4].is_empty();
    // The retained direct gene regex accepts Unicode decimal digits verbatim.
    // Those spellings have no shared point syntax and cannot enter the checked route.
    if !tokens[3].is_ascii() {
        if direct_regex {
            assertion.route =
                PointRoute::Compatibility(Compatibility::DirectRegex, body.to_owned());
        }
        return assertion;
    }
    let (Some(from), Some(to)) = (residue(&tokens[1]), residue(&tokens[5])) else {
        if direct_regex {
            assertion.route =
                PointRoute::Compatibility(Compatibility::DirectRegex, body.to_owned());
        }
        return assertion;
    };
    // Direct regex spelling is its own retained contract, including literal X.
    if direct_regex && (tokens[1] == *"X" || tokens[5] == *"X") {
        assertion.route = PointRoute::Compatibility(Compatibility::DirectRegex, body.to_owned());
        return assertion;
    }
    let position = &tokens[3];
    let first = canonical_token(&tokens[1], from);
    let last = canonical_token(&tokens[5], to);
    if first != &tokens[1] || last != &tokens[5] {
        assertion
            .preparations
            .push(Preparation::LegacyResidueSpelling);
    }
    if token_outer_whitespace || !tokens[2].is_empty() || !tokens[4].is_empty() {
        assertion
            .preparations
            .push(Preparation::LegacyResidueTokenTrim);
    }
    let candidate_bytes = [
        reference.map_or(0, str::len),
        usize::from(reference.is_some()),
        2,
        first.len(),
        position.len(),
        last.len(),
    ]
    .into_iter()
    .try_fold(0_usize, usize::checked_add);
    let compatibility = if candidate_bytes.is_none_or(|n| n > INPUT_LIMIT)
        || reference.is_some_and(|v| v.len() > REFERENCE_LIMIT)
    {
        Some(Compatibility::OverLimit)
    } else if from.0 == to.0 {
        Some(Compatibility::EqualSubstitution)
    } else if position.bytes().all(|b| b == b'0') {
        Some(Compatibility::ZeroPosition)
    } else if from.0 == "M" && position.trim_start_matches('0') == "1" {
        Some(Compatibility::InitiationChange)
    } else if from.0 == "*" {
        Some(Compatibility::TerminationReference)
    } else {
        None
    };
    if let Some(class) = compatibility {
        assertion.route = PointRoute::Compatibility(class, format!("{}{position}{}", from.0, to.0));
        return assertion;
    }
    let Ok(checked_reference) = checked_reference(reference, &mut assertion.preparations) else {
        return assertion;
    };
    let mut candidate = String::new();
    if let Some(reference) = checked_reference {
        candidate.push_str(reference);
        candidate.push(':');
    }
    candidate.push_str("p.");
    candidate.push_str(first);
    candidate.push_str(position);
    candidate.push_str(last);
    // Parsed, Invalid, Unsupported and ResourceLimitExceeded remain exclusive.
    // The selected checked route has no legacy fallback.
    assertion.route = PointRoute::Checked(parse_hgvs_protein_point_21_1_4(&candidate));
    assertion
}

fn checked_reference<'a>(
    reference: Option<&'a str>,
    preparations: &mut Vec<Preparation>,
) -> Result<Option<&'a str>, HgvsProteinPointError> {
    match reference {
        Some("") => {
            preparations.push(Preparation::LegacyEmptyReferencePrefix);
            Ok(None)
        }
        Some(prefix) => {
            // Validate the opaque prefix through the existing checked constructor.
            let probe = HgvsProteinPointEnvelope::construct(
                Some(prefix),
                HgvsProteinResidue::Val,
                "600",
                &HgvsProteinPointEdit::Substitution {
                    alternate: HgvsProteinResidue::Glu,
                },
                false,
                HgvsProteinStyle::OneLetter,
            );
            match probe {
                Ok(_) => Ok(Some(prefix)),
                Err(
                    HgvsProteinPointError::Invalid { .. }
                    | HgvsProteinPointError::ConstructionAgreement,
                ) => {
                    preparations.push(Preparation::LegacyUncheckedReferencePrefix);
                    Ok(None)
                }
                Err(error) => Err(error),
            }
        }
        None => Ok(None),
    }
}

/// Validate one untouched source assertion without compatibility preparation.
pub(in crate::entities::variant) fn complete_source_protein_point(value: &str) -> bool {
    let envelope = parse_hgvs_protein_point_21_1_4(value);
    envelope
        .disposition()
        .parsed()
        .is_some_and(|parsed| parsed.reference().is_none())
        && envelope.render_source() == value
}
