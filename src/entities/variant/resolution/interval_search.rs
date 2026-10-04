//! Preserve written positional assertions beside one complete checked interval parse.
use biodata::{
    HgvsProteinIntervalDisposition, HgvsProteinIntervalEnvelope, ParsedHgvsProteinInterval,
    parse_hgvs_protein_interval_21_1_4,
};
use std::fmt;

const INPUT_LIMIT: usize = 1_048_576;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntervalSearchPreparation {
    None,
    Identity,
    Trim,
    UppercasePrefix,
    BarePrefix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntervalSearchDisposition {
    NotSelected,
    Checked,
    ReferenceRefused,
    Invalid,
    Unsupported,
    ResourceLimitExceeded,
}

pub(crate) struct IntervalSearchAssertion<'a> {
    source: &'a str,
    trimmed: &'a str,
    trim_offset: usize,
    preparation: IntervalSearchPreparation,
    candidate_bytes: Option<usize>,
    candidate: Option<String>,
    envelope: Option<HgvsProteinIntervalEnvelope>,
    disposition: IntervalSearchDisposition,
    code: Option<&'static str>,
    diagnostic_span: Option<(usize, usize)>,
}

impl<'a> IntervalSearchAssertion<'a> {
    pub(crate) fn selects(source: &str) -> bool {
        selected(source)
    }
    pub(crate) fn starts_change(source: &str) -> bool {
        shaped(source)
    }
    pub(crate) fn disposition(&self) -> IntervalSearchDisposition {
        self.disposition
    }
    pub(crate) fn query_spelling(&self) -> Option<&'a str> {
        (self.disposition == IntervalSearchDisposition::Checked).then_some(self.trimmed)
    }
    pub(crate) fn parsed(&self) -> Option<&ParsedHgvsProteinInterval> {
        self.envelope
            .as_ref()
            .and_then(|e| e.disposition().parsed())
    }
    #[cfg(test)]
    pub(crate) fn source(&self) -> &'a str {
        self.source
    }
    #[cfg(test)]
    pub(crate) fn trimmed(&self) -> &'a str {
        self.trimmed
    }
    #[cfg(test)]
    pub(crate) fn trim_offset(&self) -> usize {
        self.trim_offset
    }
    #[cfg(test)]
    pub(crate) fn preparation(&self) -> IntervalSearchPreparation {
        self.preparation
    }
    #[cfg(test)]
    pub(crate) fn candidate_bytes(&self) -> Option<usize> {
        self.candidate_bytes
    }
    #[cfg(test)]
    pub(crate) fn candidate(&self) -> Option<&str> {
        self.candidate.as_deref()
    }
    #[cfg(test)]
    pub(crate) fn envelope(&self) -> Option<&HgvsProteinIntervalEnvelope> {
        self.envelope.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn code(&self) -> Option<&'static str> {
        self.code
    }
    #[cfg(test)]
    pub(crate) fn diagnostic_span(&self) -> Option<(usize, usize)> {
        self.diagnostic_span
    }
    /// Inserted prefix bytes have no source range; written prefixes retain their ranges.
    #[cfg(test)]
    pub(crate) fn source_span(&self, span: biodata::HgvsSpan) -> Option<(usize, usize)> {
        let candidate = self.candidate.as_ref()?;
        if span.start() > span.end() || span.end() > candidate.len() {
            return None;
        }
        let written = self.trimmed.starts_with("p.") || self.trimmed.starts_with("P.");
        let offset = if written {
            self.trim_offset
        } else {
            self.trim_offset.checked_add(span.start().checked_sub(2)?)?
        };
        if written {
            Some((
                offset.checked_add(span.start())?,
                offset.checked_add(span.end())?,
            ))
        } else {
            Some((
                offset,
                self.trim_offset.checked_add(span.end().checked_sub(2)?)?,
            ))
        }
    }
}

impl fmt::Debug for IntervalSearchAssertion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntervalSearchAssertion")
            .field("disposition", &self.disposition)
            .field("preparation", &self.preparation)
            .field("source_bytes", &self.source.len())
            .field("trim_offset", &self.trim_offset)
            .field("candidate_bytes", &self.candidate_bytes)
            .field("prediction", &self.parsed().map(|p| p.is_predicted()))
            .field("code", &self.code)
            .field("diagnostic_span", &self.diagnostic_span)
            .finish()
    }
}

/// Borrowed resemblance only. The producer proves grammar after resource preflight.
fn selected(source: &str) -> bool {
    let text = source.trim();
    if !["del", "dup", "ins"]
        .iter()
        .any(|marker| text.contains(marker))
    {
        return false;
    }
    shaped(text)
}

fn shaped(text: &str) -> bool {
    if text.starts_with("p.")
        || text.starts_with("P.")
        || text.contains(":p.")
        || text.contains(":P.")
    {
        return true;
    }
    let body = text
        .strip_prefix("p.")
        .or_else(|| text.strip_prefix("P."))
        .unwrap_or(text);
    let body = body.strip_prefix('(').unwrap_or(body);
    const RESIDUES: [&str; 50] = [
        "Ala", "Arg", "Asn", "Asp", "Cys", "Gln", "Glu", "Gly", "His", "Ile", "Leu", "Lys", "Met",
        "Phe", "Pro", "Ser", "Thr", "Trp", "Tyr", "Val", "Asx", "Glx", "Sec", "Xaa", "Ter", "A",
        "R", "N", "D", "C", "Q", "E", "G", "H", "I", "L", "K", "M", "F", "P", "S", "T", "W", "Y",
        "V", "B", "Z", "U", "X", "*",
    ];
    RESIDUES.iter().any(|residue| {
        body.strip_prefix(residue)
            .is_some_and(|rest| rest.as_bytes().first().is_some_and(u8::is_ascii_digit))
    })
}

pub(crate) fn protein_interval_search(source: &str) -> IntervalSearchAssertion<'_> {
    let trimmed = source.trim();
    let mut result = IntervalSearchAssertion {
        source,
        trimmed,
        trim_offset: trimmed.as_ptr() as usize - source.as_ptr() as usize,
        preparation: IntervalSearchPreparation::None,
        candidate_bytes: None,
        candidate: None,
        envelope: None,
        disposition: IntervalSearchDisposition::NotSelected,
        code: None,
        diagnostic_span: None,
    };
    if !selected(source) {
        return result;
    }
    if source.len() > INPUT_LIMIT {
        result.disposition = IntervalSearchDisposition::ResourceLimitExceeded;
        result.code = Some("hgvs_input_limit");
        result.diagnostic_span = Some((0, source.len()));
        return result;
    }
    if trimmed.contains(':') {
        result.disposition = IntervalSearchDisposition::ReferenceRefused;
        result.code = Some("protein_interval_search_reference");
        result.diagnostic_span = Some((0, source.len()));
        return result;
    }
    result.preparation = if trimmed.starts_with("P.") {
        IntervalSearchPreparation::UppercasePrefix
    } else if !trimmed.starts_with("p.") {
        IntervalSearchPreparation::BarePrefix
    } else if trimmed.len() != source.len() {
        IntervalSearchPreparation::Trim
    } else {
        IntervalSearchPreparation::Identity
    };
    result.candidate_bytes = if result.preparation == IntervalSearchPreparation::BarePrefix {
        trimmed.len().checked_add(2)
    } else {
        Some(trimmed.len())
    };
    if result.candidate_bytes.is_none_or(|n| n > INPUT_LIMIT) {
        result.disposition = IntervalSearchDisposition::ResourceLimitExceeded;
        result.code = Some("hgvs_input_limit");
        return result;
    }
    let candidate = match result.preparation {
        IntervalSearchPreparation::UppercasePrefix => format!("p.{}", &trimmed[2..]),
        IntervalSearchPreparation::BarePrefix => format!("p.{trimmed}"),
        _ => trimmed.to_owned(),
    };
    let envelope = parse_hgvs_protein_interval_21_1_4(&candidate);
    result.disposition = match envelope.disposition() {
        HgvsProteinIntervalDisposition::Parsed(p) if p.reference().is_none() => {
            IntervalSearchDisposition::Checked
        }
        HgvsProteinIntervalDisposition::Parsed(_) => IntervalSearchDisposition::ReferenceRefused,
        HgvsProteinIntervalDisposition::Invalid(_) => IntervalSearchDisposition::Invalid,
        HgvsProteinIntervalDisposition::Unsupported(_) => IntervalSearchDisposition::Unsupported,
        HgvsProteinIntervalDisposition::ResourceLimitExceeded(_) => {
            IntervalSearchDisposition::ResourceLimitExceeded
        }
    };
    result.code = Some(envelope.disposition().code());
    result.diagnostic_span = envelope
        .disposition()
        .diagnostic()
        .and_then(|d| d.span())
        .map(|s| (s.start(), s.end()));
    result.candidate = Some(candidate);
    result.envelope = Some(envelope);
    result
}
