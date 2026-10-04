//! One private owner for checked coding substitutions and retained lexical compatibility.
use biodata::{
    HgvsEdit, HgvsEnvelope, HgvsLocation, HgvsMarker, HgvsMolecule, parse_hgvs_nucleotide_21_1_4,
};
use std::fmt;

const INPUT_LIMIT: usize = 1024 * 1024;
const REFERENCE_LIMIT: usize = 4 * 1024;

pub(super) enum CodingRoute {
    Absent,
    CompatibilityResource,
    CompatibilityReference,
    CompatibilityZero,
    CompatibilityCase,
    CompatibilityBody,
    Checked(HgvsEnvelope),
}

pub(super) struct CodingAssertion<'a> {
    pub(super) source: &'a str,
    pub(super) segment: &'a str,
    pub(super) reference_prefix: Option<&'a str>,
    pub(super) outer_whitespace: bool,
    pub(super) route: CodingRoute,
}

impl CodingAssertion<'_> {
    pub(super) fn route_name(&self) -> &'static str {
        match self.route {
            CodingRoute::Absent => "absent",
            CodingRoute::CompatibilityResource => "compatibility_resource",
            CodingRoute::CompatibilityReference => "compatibility_reference",
            CodingRoute::CompatibilityZero => "compatibility_zero",
            CodingRoute::CompatibilityCase => "compatibility_case",
            CodingRoute::CompatibilityBody => "compatibility_body",
            CodingRoute::Checked(_) => "checked",
        }
    }

    fn checked_key(envelope: &HgvsEnvelope) -> Result<String, &'static str> {
        let parsed = envelope
            .disposition()
            .parsed()
            .ok_or(envelope.disposition().code())?;
        if parsed.molecule() != HgvsMolecule::Coding
            || !matches!(parsed.location(), Some(HgvsLocation::Point(p))
                if p.marker() == HgvsMarker::Ordinary && p.offset().is_none() && p.digits().is_some())
            || !matches!(parsed.edit(), Some(HgvsEdit::Substitution { .. }))
        {
            return Err("coding_alias_unrepresentable");
        }
        envelope
            .render_source()
            .ok_or("coding_alias_unrepresentable")?;
        let rendered = parsed.render_constructed();
        let segment = match parsed.reference() {
            Some(reference) => rendered
                .strip_prefix(reference)
                .and_then(|v| v.strip_prefix(':'))
                .ok_or("coding_alias_unrepresentable")?,
            None => rendered.as_str(),
        };
        if !segment.starts_with("c.") {
            return Err("coding_alias_unrepresentable");
        }
        Ok(segment.to_ascii_uppercase())
    }

    pub(super) fn diagnostic(&self) -> Option<&'static str> {
        match &self.route {
            CodingRoute::Checked(envelope) => Self::checked_key(envelope).err(),
            _ => None,
        }
    }

    pub(super) fn key(&self) -> Option<String> {
        match &self.route {
            CodingRoute::Absent => None,
            CodingRoute::Checked(envelope) => Self::checked_key(envelope).ok(),
            _ => Some(self.segment.to_ascii_uppercase()),
        }
    }
}

impl fmt::Debug for CodingAssertion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prediction = match &self.route {
            CodingRoute::Checked(envelope) => {
                envelope.disposition().parsed().map(|p| p.is_predicted())
            }
            _ => None,
        };
        f.debug_struct("CodingAssertion")
            .field("source_bytes", &self.source.len())
            .field("reference_bytes", &self.reference_prefix.map(str::len))
            .field("outer_whitespace", &self.outer_whitespace)
            .field("route", &self.route_name())
            .field("prediction", &prediction)
            .field("diagnostic", &self.diagnostic())
            .finish()
    }
}

/// Select only finite complete spellings. BioData owns their syntax verification.
enum Spelling {
    Checked,
    Zero,
    Case,
}

fn spelling_class(segment: &str) -> Option<Spelling> {
    let bytes = segment.as_bytes();
    let (prefix, mut body) = bytes.split_at_checked(2)?;
    if !matches!(prefix, b"c." | b"C.") {
        return None;
    }
    if body.first() == Some(&b'(') && body.last() == Some(&b')') {
        body = body.get(1..body.len().checked_sub(1)?)?;
    }
    let (digits, edit) = body.split_at_checked(body.len().checked_sub(3)?)?;
    if digits.is_empty()
        || !digits.iter().all(u8::is_ascii_digit)
        || edit[1] != b'>'
        || !matches!(
            edit[0],
            b'A' | b'C' | b'G' | b'T' | b'a' | b'c' | b'g' | b't'
        )
        || !matches!(
            edit[2],
            b'A' | b'C' | b'G' | b'T' | b'a' | b'c' | b'g' | b't'
        )
    {
        return None;
    }
    if digits.iter().all(|b| *b == b'0') {
        Some(Spelling::Zero)
    } else if prefix[0] != b'c' || edit[0].is_ascii_lowercase() || edit[2].is_ascii_lowercase() {
        Some(Spelling::Case)
    } else {
        Some(Spelling::Checked)
    }
}

pub(super) fn coding_assertion(source: &str) -> CodingAssertion<'_> {
    let candidate = source.trim();
    let segment = super::coding_change_segment(source);
    let reference_prefix = candidate
        .rsplit_once(':')
        .filter(|(_, body)| {
            body.as_bytes()
                .get(..2)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"c."))
        })
        .map(|(prefix, _)| prefix);
    let route = if segment.is_empty() {
        CodingRoute::Absent
    } else if candidate.len() > INPUT_LIMIT
        || reference_prefix.is_some_and(|v| v.len() > REFERENCE_LIMIT)
    {
        CodingRoute::CompatibilityResource
    } else if reference_prefix.is_some_and(|prefix| {
        prefix.is_empty() || !prefix.bytes().all(|b| b.is_ascii_graphic() && b != b':')
    }) {
        CodingRoute::CompatibilityReference
    } else {
        match spelling_class(segment) {
            Some(Spelling::Zero) => CodingRoute::CompatibilityZero,
            Some(Spelling::Case) => CodingRoute::CompatibilityCase,
            Some(Spelling::Checked) => {
                CodingRoute::Checked(parse_hgvs_nucleotide_21_1_4(candidate))
            }
            None => CodingRoute::CompatibilityBody,
        }
    };
    CodingAssertion {
        source,
        segment,
        reference_prefix,
        outer_whitespace: candidate != source,
        route,
    }
}

pub(super) fn coding_key(source: &str) -> Option<String> {
    coding_assertion(source).key()
}

pub(crate) fn coding_changes_equivalent(left: &str, right: &str) -> bool {
    match (coding_key(left), coding_key(right)) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}
