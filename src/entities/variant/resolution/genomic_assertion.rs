//! Complete checked genomic source facts with explicit point admission.
use biodata::{
    HgvsEdit, HgvsEnvelope, HgvsLocation, HgvsMarker, HgvsMolecule, ParsedHgvsNucleotide,
    parse_hgvs_nucleotide_21_1_4,
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

// This is a comparison policy, never the authoritative source representation.
#[derive(Default)]
pub(super) struct LegacyComparisonFields<'a> {
    pub(super) build: Option<&'a str>,
    pub(super) accession: Option<&'a str>,
    pub(super) position: Option<u64>,
    pub(super) reference: Option<&'a str>,
    pub(super) alternate: Option<&'a str>,
}

pub(super) enum SourceGenomicAssertion<'s, 'a> {
    Absent {
        build: Option<&'a str>,
    },
    Checked {
        assertion: &'s GenomicAssertion<'a>,
        parsed: &'s ParsedHgvsNucleotide,
    },
    Compatibility {
        fields: LegacyComparisonFields<'a>,
        diagnostic: Option<&'static str>,
    },
}
impl<'a> SourceGenomicAssertion<'_, 'a> {
    pub(super) fn comparison_fields(&self) -> LegacyComparisonFields<'a> {
        match self {
            Self::Absent { build } => LegacyComparisonFields {
                build: *build,
                ..Default::default()
            },
            Self::Compatibility { fields, .. } => LegacyComparisonFields { ..*fields },
            Self::Checked { assertion, parsed } => {
                if let Ok(c) = assertion.checked_components() {
                    return LegacyComparisonFields {
                        build: assertion.build,
                        accession: Some(c.accession),
                        position: c.position_lexeme.parse().ok(),
                        reference: Some(c.reference),
                        alternate: Some(c.alternate),
                    };
                }
                // Delete this lexical policy only after separately reviewed changes
                // to unknown/uncertain substitution comparison decisions.
                if matches!(parsed.edit(), Some(HgvsEdit::Substitution { .. })) {
                    return legacy_source_field_policy(assertion.candidate, assertion.build);
                }
                LegacyComparisonFields {
                    build: assertion.build,
                    accession: assertion.reference_prefix,
                    ..Default::default()
                }
            }
        }
    }
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
    let mut route = if !admitted {
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
    let envelope = if route == "checked"
        || (admitted && matches!(admission, Admission::Source) && route == "body_compatibility")
    {
        let envelope = parse_hgvs_nucleotide_21_1_4(candidate);
        if envelope.disposition().parsed().is_some() {
            route = "checked";
        }
        Some(envelope)
    } else {
        None
    };
    GenomicAssertion {
        source,
        candidate,
        candidate_offset_bytes: candidate.as_ptr() as usize - source.as_ptr() as usize,
        build,
        envelope,
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
    fn checked_parsed(&self) -> Result<&ParsedHgvsNucleotide, &'static str> {
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
        let reference = parsed
            .reference_span()
            .and_then(|s| s.slice(self.candidate));
        let location = parsed.location_span().and_then(|s| s.slice(self.candidate));
        if parsed.molecule() != HgvsMolecule::Genomic
            || parsed.span().start() != 0
            || parsed.span().end() != self.candidate.len()
            || parsed.span().slice(self.candidate) != Some(self.candidate)
            || reference.is_none()
            || reference != parsed.reference()
            || reference != self.reference_prefix
            || parsed.location().is_none()
            || parsed.edit().is_none()
            || location.is_none()
            || parsed.location_span().map(|s| s.start()) != reference.map(|r| r.len() + 3)
            || parsed.render_constructed() != self.candidate
        {
            return Err("genomic_substitution_unrepresentable");
        }
        Ok(parsed)
    }
    fn checked_components(&self) -> Result<Components<'a>, &'static str> {
        let parsed = self.checked_parsed()?;
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
            match self.source_assertion().ok()? {
                SourceGenomicAssertion::Compatibility { diagnostic, .. } => diagnostic,
                _ => None,
            }
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
    pub(super) fn source_assertion(&self) -> Result<SourceGenomicAssertion<'_, 'a>, &'static str> {
        if !self.admitted {
            return Ok(SourceGenomicAssertion::Absent { build: self.build });
        }
        if self.route == "checked" {
            return Ok(SourceGenomicAssertion::Checked {
                assertion: self,
                parsed: self.checked_parsed()?,
            });
        }
        Ok(SourceGenomicAssertion::Compatibility {
            fields: legacy_source_field_policy(self.candidate, self.build),
            diagnostic: self.envelope.as_ref().map(|e| e.disposition().code()),
        })
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
fn legacy_source_field_policy<'a>(
    candidate: &'a str,
    build: Option<&'a str>,
) -> LegacyComparisonFields<'a> {
    let Some((accession, change)) = candidate.split_once(":g.") else {
        return LegacyComparisonFields {
            build,
            ..Default::default()
        };
    };
    let Some((left, alternate)) = change.split_once('>') else {
        return LegacyComparisonFields {
            build,
            accession: Some(accession),
            ..Default::default()
        };
    };
    let (position, reference) = left
        .find(|ch: char| !ch.is_ascii_digit())
        .map(|i| (left[..i].parse().ok(), Some(&left[i..])))
        .unwrap_or((None, None));
    LegacyComparisonFields {
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
