//! Private checked genomic substitutions with caller admission and named compatibility.
use biodata::{
    HgvsEdit, HgvsEnvelope, HgvsLocation, HgvsMarker, HgvsMolecule, parse_hgvs_nucleotide_21_1_4,
};
use regex::Regex;
use std::{fmt, sync::OnceLock};

const INPUT_LIMIT: usize = 1024 * 1024;
const REFERENCE_LIMIT: usize = 4 * 1024;

#[derive(Clone, Copy)]
pub(in crate::entities::variant) enum Admission {
    Chromosome,
    Structured,
    Source,
}

pub(in crate::entities::variant) struct Components<'a> {
    pub(in crate::entities::variant) accession: &'a str,
    pub(in crate::entities::variant) position_lexeme: &'a str,
    pub(in crate::entities::variant) reference: &'a str,
    pub(in crate::entities::variant) alternate: &'a str,
}

pub(in crate::entities::variant) struct GenomicAssertion<'a> {
    pub(in crate::entities::variant) source: &'a str,
    pub(in crate::entities::variant) candidate: &'a str,
    pub(in crate::entities::variant) candidate_offset_bytes: usize,
    pub(in crate::entities::variant) build: Option<&'a str>,
    pub(in crate::entities::variant) envelope: Option<HgvsEnvelope>,
    route: &'static str,
    reference_prefix: Option<&'a str>,
    admission: Admission,
    admitted: bool,
}

fn chromosome(reference: &str) -> bool {
    reference.strip_prefix("chr").is_some_and(|v| {
        !v.is_empty()
            && v.bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'X' | b'Y' | b'M'))
    })
}
fn structured(reference: &str) -> bool {
    chromosome(reference)
        || reference
            .strip_prefix("NC_")
            .and_then(|v| v.split_once('.'))
            .is_some_and(|(a, b)| {
                !a.is_empty()
                    && !b.is_empty()
                    && a.bytes().all(|v| v.is_ascii_digit())
                    && b.bytes().all(|v| v.is_ascii_digit())
            })
}
fn decimal(value: &str) -> bool {
    static RE: OnceLock<Option<Regex>> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\d+$").ok())
        .as_ref()
        .is_some_and(|re| re.is_match(value))
}
// Complete compatibility captures use the old Unicode decimal class. This selects
// ownership only; selected syntax is verified by BioData on the complete candidate.
fn substitution(candidate: &str) -> Option<Components<'_>> {
    let (accession, body) = candidate.split_once(":g.")?;
    let cut = body.len().checked_sub(3)?;
    let digits = body.get(..cut)?;
    let edit = body.get(cut..)?.as_bytes();
    if digits.is_empty()
        || edit[1] != b'>'
        || !matches!(edit[0], b'A' | b'C' | b'G' | b'T')
        || !matches!(edit[2], b'A' | b'C' | b'G' | b'T')
    {
        return None;
    }
    Some(Components {
        accession,
        position_lexeme: digits,
        reference: body.get(cut..cut + 1)?,
        alternate: body.get(cut + 2..)?,
    })
}

pub(in crate::entities::variant) fn genomic_assertion(
    source: &str,
    admission: Admission,
    trim: bool,
) -> GenomicAssertion<'_> {
    let mut candidate = if trim { source.trim() } else { source };
    let mut build = None;
    if matches!(admission, Admission::Source) {
        if let Some((prefix, rest)) = candidate.split_once(':')
            && (prefix.eq_ignore_ascii_case("GRCh37") || prefix.eq_ignore_ascii_case("GRCh38"))
        {
            build = Some(prefix);
            candidate = rest;
        }
    }
    let reference_prefix = candidate.split_once(":g.").map(|(v, _)| v);
    let complete = substitution(candidate);
    let admitted = match admission {
        Admission::Source => reference_prefix.is_some(),
        Admission::Chromosome => complete
            .as_ref()
            .is_some_and(|v| chromosome(v.accession) && decimal(v.position_lexeme)),
        Admission::Structured => complete
            .as_ref()
            .is_some_and(|v| structured(v.accession) && decimal(v.position_lexeme)),
    };
    let route = if !admitted {
        "body_compatibility"
    } else if candidate.len() > INPUT_LIMIT
        || reference_prefix.is_some_and(|v| v.len() > REFERENCE_LIMIT)
    {
        "resource_compatibility"
    } else if reference_prefix
        .is_some_and(|v| v.is_empty() || !v.bytes().all(|b| b.is_ascii_graphic() && b != b':'))
    {
        "reference_compatibility"
    } else if let Some(c) = &complete {
        if c.position_lexeme.bytes().all(|b| b.is_ascii_digit()) {
            if c.position_lexeme.bytes().all(|b| b == b'0') {
                "zero_compatibility"
            } else {
                "checked"
            }
        } else if decimal(c.position_lexeme) {
            "decimal_compatibility"
        } else {
            "body_compatibility"
        }
    } else {
        "body_compatibility"
    };
    GenomicAssertion {
        source,
        candidate,
        candidate_offset_bytes: candidate.as_ptr() as usize - source.as_ptr() as usize,
        build,
        envelope: (route == "checked").then(|| parse_hgvs_nucleotide_21_1_4(candidate)),
        route,
        reference_prefix,
        admission,
        admitted,
    }
}

impl<'a> GenomicAssertion<'a> {
    pub(in crate::entities::variant) fn route_name(&self) -> &'static str {
        self.route
    }
    // The checked seam has no route back to compatibility after a refusal.
    fn checked_components(&self) -> Result<Components<'a>, &'static str> {
        let envelope = self
            .envelope
            .as_ref()
            .ok_or("genomic_substitution_unrepresentable")?;
        if envelope.source() != self.candidate
            || self.source.get(
                self.candidate_offset_bytes..self.candidate_offset_bytes + self.candidate.len(),
            ) != Some(self.candidate)
        {
            return Err("genomic_substitution_unrepresentable");
        }
        let parsed = envelope
            .disposition()
            .parsed()
            .ok_or(envelope.disposition().code())?;
        let Some(HgvsLocation::Point(position)) = parsed.location() else {
            return Err("genomic_substitution_unrepresentable");
        };
        let Some(HgvsEdit::Substitution {
            reference,
            alternate,
        }) = parsed.edit()
        else {
            return Err("genomic_substitution_unrepresentable");
        };
        if parsed.molecule() != HgvsMolecule::Genomic
            || parsed.is_predicted()
            || position.marker() != HgvsMarker::Ordinary
            || position.offset().is_some()
            || position.digits().is_none()
        {
            return Err("genomic_substitution_unrepresentable");
        }
        let span = parsed
            .location_span()
            .ok_or("genomic_substitution_unrepresentable")?;
        let accession = parsed
            .reference_span()
            .and_then(|s| self.candidate.get(s.start()..s.end()))
            .ok_or("genomic_substitution_unrepresentable")?;
        let digits = self
            .candidate
            .get(span.start()..span.end())
            .ok_or("genomic_substitution_unrepresentable")?;
        let ref_text = self
            .candidate
            .get(span.end()..span.end() + 1)
            .ok_or("genomic_substitution_unrepresentable")?;
        let alt_text = self
            .candidate
            .get(span.end() + 2..)
            .ok_or("genomic_substitution_unrepresentable")?;
        if ref_text.chars().next() != Some(*reference)
            || alt_text.chars().next() != Some(*alternate)
            || digits != position.digits().unwrap_or("")
            || Some(accession) != parsed.reference()
        {
            return Err("genomic_substitution_unrepresentable");
        }
        Ok(Components {
            accession,
            position_lexeme: digits,
            reference: ref_text,
            alternate: alt_text,
        })
    }
    pub(in crate::entities::variant) fn diagnostic(&self) -> Option<&'static str> {
        if self.route == "checked" {
            self.checked_components().err()
        } else {
            None
        }
    }
    pub(in crate::entities::variant) fn components(&self) -> Option<Components<'a>> {
        if self.route == "checked" {
            return self.checked_components().ok();
        }
        if !self.admitted || matches!(self.admission, Admission::Source) {
            return None;
        }
        substitution(self.candidate)
    }
    pub(super) fn source_components(&self) -> super::GenomicComponents<'a> {
        if self.route == "checked" {
            return match self.checked_components() {
                Ok(c) => super::GenomicComponents {
                    build: self.build,
                    accession: Some(c.accession),
                    position: c.position_lexeme.parse().ok(),
                    reference: Some(c.reference),
                    alternate: Some(c.alternate),
                },
                Err(_) => super::GenomicComponents {
                    build: self.build,
                    ..Default::default()
                },
            };
        }
        nonselected_source_compatibility(self.candidate, self.build)
    }
    #[cfg(test)]
    pub(in crate::entities::variant) fn from_checked_envelope(
        source: &'a str,
        envelope: HgvsEnvelope,
    ) -> Self {
        Self {
            source,
            candidate: source,
            candidate_offset_bytes: 0,
            build: None,
            reference_prefix: source.split_once(':').map(|(v, _)| v),
            route: "checked",
            envelope: Some(envelope),
            admission: Admission::Source,
            admitted: true,
        }
    }
}
impl fmt::Debug for GenomicAssertion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenomicAssertion")
            .field("source_bytes", &self.source.len())
            .field("reference_bytes", &self.reference_prefix.map(str::len))
            .field("outer_whitespace", &(self.source.trim() != self.source))
            .field("route", &self.route_name())
            .field(
                "prediction",
                &self
                    .envelope
                    .as_ref()
                    .and_then(|e| e.disposition().parsed())
                    .map(|p| p.is_predicted()),
            )
            .field("diagnostic", &self.diagnostic())
            .finish()
    }
}
// Delete only after separate reviewed adoption of every retained nonselected
// source form. Partial components are engineering compatibility, not syntax success.
fn nonselected_source_compatibility<'a>(
    candidate: &'a str,
    build: Option<&'a str>,
) -> super::GenomicComponents<'a> {
    let Some((accession, change)) = candidate.split_once(":g.") else {
        return super::GenomicComponents {
            build,
            ..Default::default()
        };
    };
    let Some((left, alternate)) = change.split_once('>') else {
        return super::GenomicComponents {
            build,
            accession: Some(accession),
            ..Default::default()
        };
    };
    let (position, reference) = left
        .find(|ch: char| !ch.is_ascii_digit())
        .map(|i| (left[..i].parse().ok(), Some(&left[i..])))
        .unwrap_or((None, None));
    super::GenomicComponents {
        build,
        accession: Some(accession),
        position,
        reference,
        alternate: (!alternate.is_empty()).then_some(alternate),
    }
}

// Explicit component requests retain product accession/base admission independently
// of the selected syntax owner; their position is already a represented u64.
pub(super) fn structured_identity_admitted(identity: &super::RequestedVariantIdentity) -> bool {
    identity
        .genomic_accession
        .as_deref()
        .is_some_and(structured)
        && identity.reference.as_deref().is_some_and(single_base)
        && identity.alternate.as_deref().is_some_and(single_base)
}
fn single_base(value: &str) -> bool {
    matches!(value.as_bytes(), b"A" | b"C" | b"G" | b"T")
}
