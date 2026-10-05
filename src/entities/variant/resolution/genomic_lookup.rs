//! Complete lookup input ownership; lexical guards retain product admission.
use super::{NormalizedGenomicCoordinate, REFSEQ_GENOMIC_BUILDS};
use crate::{entities::variant::GenomeBuild, error::BioMcpError};
use biodata::{
    HgvsEdit, HgvsEnvelope, HgvsLocation, HgvsMarker, HgvsMolecule, ParsedHgvsNucleotide,
    parse_hgvs_nucleotide_21_1_4,
};
use regex::Regex;
use std::{fmt, sync::OnceLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LookupRoute {
    Direct,
    Coordinate,
}

pub(super) struct GenomicLookup<'a> {
    pub(super) original: &'a str,
    pub(super) candidate: String,
    pub(super) build: Option<GenomeBuild>,
    pub(super) envelope: HgvsEnvelope,
    route: LookupRoute,
    // Only preflight compatibility may use wrapper fields. Checked projection
    // has no route back to these fields after a refusal.
    compatibility_coordinate: Option<NormalizedGenomicCoordinate>,
}
fn position_error() -> BioMcpError {
    BioMcpError::InvalidArgument("genomic coordinate position must be positive".into())
}
fn projection_error() -> BioMcpError {
    BioMcpError::InternalProcessing
}

const CHROMOSOME_PATTERN: &str = r"chr(?:[1-9]|1[0-9]|2[0-2]|X|Y)";
const GENOMIC_CHANGE_PATTERN: &str = concat!(
    r"(?:[ACGT]>[ACGT]",
    r"|(?:_\d+)?del(?:[ACGT]+)?",
    r"|(?:_\d+)?dup(?:[ACGT]+)?",
    r"|_\d+ins[ACGT]+",
    r"|_\d+inv",
    r"|(?:_\d+)?delins[ACGT]+",
    r"|(?:_\d+)?[ACGT]+\[[1-9]\d*\])",
);

fn hgvs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(&format!(
            r"^{CHROMOSOME_PATTERN}:g\.\d+{GENOMIC_CHANGE_PATTERN}$"
        ))
        .expect("valid regex")
    })
}

fn coordinate_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(&format!(
            r"(?i)^(?:(GRCh37|GRCh38|hg19|hg38):)?({CHROMOSOME_PATTERN}):g\.(\d+)([ACGT]>[ACGT]|del)$"
        ))
        .expect("valid regex")
    })
}

fn vcf_coordinate_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^(chr(?:[1-9]|1[0-9]|2[0-2]|X|Y)):(\d+):([ACGT]):([ACGT])$")
            .expect("valid regex")
    })
}

fn refseq_coordinate_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^(NC_\d+\.\d+):g\.(\d+)([ACGT]>[ACGT]|del)$").expect("valid regex")
    })
}

fn spdi_coordinate_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^(NC_\d+\.\d+):(\d+):([ACGT]):([ACGT])$").expect("valid regex")
    })
}

fn coordinate_candidate(input: &str) -> Result<Option<(String, Option<GenomeBuild>)>, BioMcpError> {
    let construct = |chromosome: &str, position: &str, change: &str, build| {
        let change = if change.eq_ignore_ascii_case("del") {
            "del".to_string()
        } else {
            change.to_ascii_uppercase()
        };
        (
            format!(
                "chr{}:g.{position}{change}",
                chromosome[3..].to_ascii_uppercase()
            ),
            build,
        )
    };
    if let Some(c) = coordinate_re().captures(input) {
        let build = c
            .get(1)
            .map(|v| v.as_str().parse())
            .transpose()
            .map_err(BioMcpError::InvalidArgument)?;
        return Ok(Some(construct(&c[2], &c[3], &c[4], build)));
    }
    if let Some(c) = vcf_coordinate_re().captures(input) {
        return Ok(Some(construct(
            &c[1],
            &c[2],
            &format!("{}>{}", &c[3], &c[4]),
            None,
        )));
    }
    let spdi = spdi_coordinate_re().captures(input);
    if let Some(c) = spdi.as_ref() {
        let (chromosome, build) = accession(&c[1])?;
        let position = c[2]
            .parse::<u64>()
            .ok()
            .and_then(|v| v.checked_add(1))
            .ok_or_else(position_error)?;
        return Ok(Some(construct(
            chromosome,
            &position.to_string(),
            &format!("{}>{}", &c[3], &c[4]),
            Some(build),
        )));
    }
    if let Some(c) = refseq_coordinate_re().captures(input) {
        let (chromosome, build) = accession(&c[1])?;
        return Ok(Some(construct(chromosome, &c[2], &c[3], Some(build))));
    }
    if input.to_ascii_uppercase().starts_with("NC_") {
        return Err(BioMcpError::InvalidArgument(
            "RefSeq genomic accessions must include a supported version".into(),
        ));
    }
    Ok(None)
}
fn accession(value: &str) -> Result<(&'static str, GenomeBuild), BioMcpError> {
    REFSEQ_GENOMIC_BUILDS
        .iter()
        .find(|(a, _, _)| a.eq_ignore_ascii_case(value))
        .map(|(_, chromosome, build)| (*chromosome, *build))
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(format!("unsupported RefSeq genomic accession: {value}"))
        })
}

pub(super) fn genomic_lookup(
    original: &str,
    route: LookupRoute,
) -> Result<Option<GenomicLookup<'_>>, BioMcpError> {
    let input = original.trim();
    let (candidate, build) = match route {
        LookupRoute::Direct => {
            if !hgvs_re().is_match(input) {
                return Ok(None);
            }
            (input.to_owned(), None)
        }
        LookupRoute::Coordinate => {
            let Some(candidate) = coordinate_candidate(input)? else {
                return Ok(None);
            };
            candidate
        }
    };
    let envelope = parse_hgvs_nucleotide_21_1_4(&candidate);
    // The old coordinate adapter accepts arbitrary leading zero padding even
    // beyond the producer limit. Preserve only this preflight resource policy.
    let compatibility_coordinate = if route == LookupRoute::Coordinate
        && envelope.disposition().code() == "hgvs_input_limit"
    {
        let c = coordinate_re()
            .captures(&candidate)
            .ok_or_else(projection_error)?;
        let position = c[3]
            .parse::<u64>()
            .ok()
            .filter(|v| *v > 0)
            .ok_or_else(position_error)?;
        Some(NormalizedGenomicCoordinate {
            id: format!("{}:g.{position}{}", &c[2], &c[4]),
            genome_build: build,
            requires_comparison: build.is_none(),
        })
    } else {
        None
    };
    Ok(Some(GenomicLookup {
        original,
        candidate,
        build,
        envelope,
        route,
        compatibility_coordinate,
    }))
}
impl GenomicLookup<'_> {
    pub(super) fn compatibility_code(&self) -> Option<&'static str> {
        self.envelope
            .disposition()
            .parsed()
            .is_none()
            .then(|| self.envelope.disposition().code())
    }
    fn checked(&self) -> Result<&ParsedHgvsNucleotide, BioMcpError> {
        let p = self
            .envelope
            .disposition()
            .parsed()
            .ok_or_else(projection_error)?;
        if self.envelope.source() != self.candidate
            || p.molecule() != HgvsMolecule::Genomic
            || p.is_predicted()
            || p.span().start() != 0
            || p.span().end() != self.candidate.len()
            || p.span().slice(&self.candidate) != Some(self.candidate.as_str())
            || p.reference_span().and_then(|s| s.slice(&self.candidate)) != p.reference()
            || p.reference().is_none()
            || p.location().is_none()
            || p.edit().is_none()
            || p.location_span()
                .and_then(|s| s.slice(&self.candidate))
                .is_none()
            || p.render_constructed() != self.candidate
        {
            return Err(projection_error());
        }
        Ok(p)
    }
    pub(super) fn exact_id(&self) -> Result<Option<&str>, BioMcpError> {
        if self.route != LookupRoute::Direct {
            return Ok(None);
        }
        if self.compatibility_code().is_none() {
            self.checked()?;
        }
        Ok(Some(&self.candidate))
    }
    pub(super) fn coordinate(&self) -> Result<Option<NormalizedGenomicCoordinate>, BioMcpError> {
        if self.route != LookupRoute::Coordinate {
            return Ok(None);
        }
        if let Some(c) = &self.compatibility_coordinate {
            return Ok(Some(c.clone()));
        }
        if self.compatibility_code().is_some() {
            return Err(position_error());
        }
        let p = self.checked()?;
        let Some(HgvsLocation::Point(position)) = p.location() else {
            return Err(projection_error());
        };
        if position.marker() != HgvsMarker::Ordinary || position.offset().is_some() {
            return Err(projection_error());
        }
        let position = position
            .digits()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v > 0)
            .ok_or_else(position_error)?;
        let change = match p.edit() {
            Some(HgvsEdit::Substitution {
                reference,
                alternate,
            }) => format!("{reference}>{alternate}"),
            Some(HgvsEdit::Deletion { deleted: None }) => "del".to_string(),
            _ => return Err(projection_error()),
        };
        Ok(Some(NormalizedGenomicCoordinate {
            id: format!(
                "{}:g.{position}{change}",
                p.reference().ok_or_else(projection_error)?
            ),
            genome_build: self.build,
            requires_comparison: self.build.is_none(),
        }))
    }
}
impl fmt::Debug for GenomicLookup<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenomicLookup")
            .field("original_bytes", &self.original.len())
            .field("candidate_bytes", &self.candidate.len())
            .field("route", &self.route)
            .field("diagnostic", &self.envelope.disposition().code())
            .finish()
    }
}
