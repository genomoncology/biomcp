//! Alias policy over complete checked, reference-free interval bodies.
use biodata::{
    HgvsProteinCoordinate, HgvsProteinIntervalEdit, HgvsProteinIntervalEnvelope,
    HgvsProteinIntervalLocation, ParsedHgvsProteinInterval, parse_hgvs_protein_interval_21_1_4,
};
use std::fmt;

pub(super) const INPUT_LIMIT: usize = 1_048_576;

pub(super) struct IntervalAssertion<'a> {
    pub(super) source: &'a str,
    #[cfg(test)]
    pub(super) reference: Option<&'a str>,
    pub(super) body: &'a str,
    pub(super) body_offset: usize,
    pub(super) candidate_bytes: Option<usize>,
    pub(super) envelope: Option<HgvsProteinIntervalEnvelope>,
}

impl<'a> IntervalAssertion<'a> {
    pub(super) fn prepare(source: &'a str) -> Self {
        let body = super::protein_alias_body(source);
        Self {
            source,
            #[cfg(test)]
            reference: source
                .trim()
                .rsplit_once(":p.")
                .map(|(reference, _)| reference),
            body,
            body_offset: body.as_ptr() as usize - source.as_ptr() as usize,
            candidate_bytes: body.len().checked_add(2),
            envelope: None,
        }
    }

    fn over_limit(&self) -> bool {
        self.candidate_bytes.is_none_or(|n| n > INPUT_LIMIT)
    }

    pub(super) fn diagnostic(&self) -> Option<&'static str> {
        if self.over_limit() {
            Some("hgvs_input_limit")
        } else {
            self.envelope.as_ref().and_then(|e| {
                e.disposition().diagnostic().map(|diagnostic| diagnostic.code())
            })
        }
    }

    fn parse(&mut self) -> bool {
        let candidate = format!("p.{}", self.body);
        self.envelope = Some(parse_hgvs_protein_interval_21_1_4(&candidate));
        self.envelope
            .as_ref()
            .is_some_and(|e| e.disposition().parsed().is_some())
    }

    /// Translate a candidate body span without assigning it to the removed reference.
    #[cfg(test)]
    pub(super) fn source_span(&self, span: biodata::HgvsSpan) -> Option<(usize, usize)> {
        let start = span.start().checked_sub(2)?;
        let end = span.end().checked_sub(2)?;
        if start > end || end > self.body.len() {
            return None;
        }
        Some((
            self.body_offset.checked_add(start)?,
            self.body_offset.checked_add(end)?,
        ))
    }
}

impl fmt::Debug for IntervalAssertion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntervalAssertion")
            .field(
                "route",
                &if self.over_limit() { "resource" } else { "interval" },
            )
            .field("source_bytes", &self.source.len())
            .field("candidate_bytes", &self.candidate_bytes)
            .field("body_offset", &self.body_offset)
            .field("body_bytes", &self.body.len())
            .field(
                "prediction",
                &self.envelope
                    .as_ref()
                    .and_then(|e| e.disposition().parsed())
                    .map(|p| p.is_predicted()),
            )
            .field("diagnostic", &self.diagnostic())
            .finish()
    }
}

pub(super) struct Comparison<'a> {
    pub(super) equivalent: bool,
    pub(super) left: IntervalAssertion<'a>,
    pub(super) right: IntervalAssertion<'a>,
}

fn hinted(body: &str) -> bool {
    ["del", "dup", "ins"].iter().any(|marker| body.contains(marker))
}

pub(super) fn compare<'a>(left: &'a str, right: &'a str) -> Comparison<'a> {
    let mut result = Comparison {
        equivalent: false,
        left: IntervalAssertion::prepare(left),
        right: IntervalAssertion::prepare(right),
    };
    if !hinted(result.left.body)
        || !hinted(result.right.body)
        || result.left.over_limit()
        || result.right.over_limit()
    {
        return result;
    }
    if !result.left.parse() || !result.right.parse() {
        return result;
    }
    if let (Some(left), Some(right)) = (
        result.left.envelope.as_ref().and_then(|e| e.disposition().parsed()),
        result.right.envelope.as_ref().and_then(|e| e.disposition().parsed()),
    ) {
        result.equivalent = same_interval(left, right);
    }
    result
}

fn same_coordinate(left: &HgvsProteinCoordinate, right: &HgvsProteinCoordinate) -> bool {
    left.residue() == right.residue() && left.position() == right.position()
}

fn same_interval(left: &ParsedHgvsProteinInterval, right: &ParsedHgvsProteinInterval) -> bool {
    use HgvsProteinIntervalEdit::{Deletion, Delins, Duplication, Insertion};
    use HgvsProteinIntervalLocation::{InsertionFlanks, Point, Range};
    let location = match (left.location(), right.location()) {
        (Point(a), Point(b)) => same_coordinate(a, b),
        (Range { start: a, end: b }, Range { start: c, end: d })
        | (InsertionFlanks { left: a, right: b }, InsertionFlanks { left: c, right: d }) => {
            same_coordinate(a, c) && same_coordinate(b, d)
        }
        _ => false,
    };
    let edit = match (left.edit(), right.edit()) {
        (Deletion, Deletion) | (Duplication, Duplication) => true,
        (Insertion { inserted: a }, Insertion { inserted: b })
        | (Delins { inserted: a }, Delins { inserted: b }) => a.residues() == b.residues(),
        _ => false,
    };
    left.is_predicted() == right.is_predicted() && location && edit
}
