//! Variant detail retrieval, section gating, and enrichment orchestration.

use std::time::Duration;

use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
#[cfg(feature = "alphagenome")]
use crate::sources::alphagenome::AlphaGenomeClient;
use crate::sources::cancerhotspots::CancerHotspotsClient;
use crate::sources::cbioportal::CBioPortalClient;
use crate::sources::civic::CivicClient;
use crate::sources::dbsnp::DbSnpClient;
use crate::sources::gnomad::{GnomadClient, GnomadVariantPopulation};
#[cfg(feature = "alphagenome")]
use crate::sources::mygene::MyGeneClient;
use crate::sources::myvariant::MyVariantClient;
use crate::sources::oncokb::{OncoKBAnnotation, OncoKBClient};
use crate::transform;

use super::gwas::add_gwas_section;
#[cfg(test)]
use super::gwas::mark_gwas_unavailable;
#[cfg(feature = "alphagenome")]
use super::resolution::hgvs_coords_re;
use super::resolution::parse_variant_id;
use super::{
    ClinvarRecord, GenomeBuild, GnomadPopulationResult, GnomadPopulationStatus,
    ResolvedPopulationCoordinate, TreatmentImplication, Variant, VariantCivicSection,
    VariantIdFormat, VariantInputKind, VariantNormalizationResponse, VariantNormalizationStatus,
    VariantOncoKbResult, classify_variant_input, gnomad_variant_slug, normalize_variant,
    protein_changes_equivalent,
};

const VARIANT_SECTION_PREDICT: &str = "predict";
const VARIANT_SECTION_PREDICTIONS: &str = "predictions";
const VARIANT_SECTION_CLINVAR: &str = "clinvar";
const VARIANT_SECTION_POPULATION: &str = "population";
const VARIANT_SECTION_POPULATION_DETAILS: &str = "population-details";
const VARIANT_SECTION_CONSERVATION: &str = "conservation";
const VARIANT_SECTION_COSMIC: &str = "cosmic";
const VARIANT_SECTION_CGI: &str = "cgi";
const VARIANT_SECTION_CIVIC: &str = "civic";
const VARIANT_SECTION_CBIOPORTAL: &str = "cbioportal";
const VARIANT_SECTION_GWAS: &str = "gwas";
const VARIANT_SECTION_ALL: &str = "all";

pub const VARIANT_SECTION_NAMES: &[&str] = &[
    VARIANT_SECTION_PREDICT,
    VARIANT_SECTION_PREDICTIONS,
    VARIANT_SECTION_CLINVAR,
    VARIANT_SECTION_POPULATION,
    VARIANT_SECTION_POPULATION_DETAILS,
    VARIANT_SECTION_CONSERVATION,
    VARIANT_SECTION_COSMIC,
    VARIANT_SECTION_CGI,
    VARIANT_SECTION_CIVIC,
    VARIANT_SECTION_CBIOPORTAL,
    VARIANT_SECTION_GWAS,
    VARIANT_SECTION_ALL,
];

const OPTIONAL_ENRICHMENT_TIMEOUT: Duration = Duration::from_secs(8);
const TRACED_TEST_ENRICHMENT_TIMEOUT: Duration = Duration::from_secs(300);

fn optional_enrichment_timeout() -> Duration {
    if std::env::var_os("NEXTEST_EXECUTION_MODE").is_some() {
        TRACED_TEST_ENRICHMENT_TIMEOUT
    } else {
        OPTIONAL_ENRICHMENT_TIMEOUT
    }
}

const GNOMAD_DATASET: &str = "gnomad_r4";
const GNOMAD_RELEASE: &str = "gnomAD v4";
const GNOMAD_FAF_CAVEAT: &str =
    "gnomAD excludes bottlenecked genetic ancestry groups when selecting grpmax FAF.";
const GNOMAD_GRCH38_REQUIRED: &str =
    "Direct gnomAD v4 population data requires a trustworthy GRCh38 coordinate.";
const GNOMAD_DBSNP_GRCH38_REQUIRED: &str = "Direct gnomAD v4 population data requires a trustworthy GRCh38 coordinate; dbSNP could not provide a compatible coordinate.";
const DBSNP_PROVIDER_FAILURE: &str =
    "dbSNP population coordinate resolution is temporarily unavailable.";
const GNOMAD_PROVIDER_FAILURE: &str = "gnomAD population data is temporarily unavailable.";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VariantWorkflowSignals {
    pub has_clinvar_signal: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct VariantSections {
    include_prediction: bool,
    include_expanded_predictions: bool,
    include_clinvar: bool,
    include_population: bool,
    include_conservation: bool,
    include_cosmic: bool,
    include_cgi: bool,
    include_civic: bool,
    include_cbioportal: bool,
    include_cancerhotspots: bool,
    include_gwas: bool,
}

fn parse_sections(sections: &[String]) -> Result<VariantSections, BioMcpError> {
    let mut out = VariantSections::default();
    let mut include_all = false;

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }
        match section.as_str() {
            VARIANT_SECTION_PREDICT => out.include_prediction = true,
            VARIANT_SECTION_PREDICTIONS => out.include_expanded_predictions = true,
            VARIANT_SECTION_CLINVAR => out.include_clinvar = true,
            VARIANT_SECTION_POPULATION | VARIANT_SECTION_POPULATION_DETAILS => {
                out.include_population = true
            }
            VARIANT_SECTION_CONSERVATION => out.include_conservation = true,
            VARIANT_SECTION_COSMIC => out.include_cosmic = true,
            VARIANT_SECTION_CGI => out.include_cgi = true,
            VARIANT_SECTION_CIVIC => out.include_civic = true,
            VARIANT_SECTION_CBIOPORTAL => out.include_cbioportal = true,
            VARIANT_SECTION_GWAS => out.include_gwas = true,
            VARIANT_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for variant. Available: {}",
                    VARIANT_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.include_expanded_predictions = true;
        out.include_clinvar = true;
        out.include_population = true;
        out.include_conservation = true;
        out.include_cosmic = true;
        out.include_cgi = true;
        out.include_civic = true;
        out.include_cbioportal = true;
        out.include_cancerhotspots = true;
        out.include_gwas = true;
    }

    Ok(out)
}

fn score_myvariant_hit(hit: &crate::sources::myvariant::MyVariantHit) -> i32 {
    let mut score = 0;
    if let Some(clinvar) = hit.clinvar.as_ref() {
        if !clinvar.rcv.is_empty() {
            score += 100;
            score += clinvar.rcv.len().min(50) as i32;
        }
        if clinvar.variant_id.is_some() {
            score += 5;
        }
    }
    if hit.dbnsfp.as_ref().and_then(|d| d.hgvsp.first()).is_some() {
        score += 10;
    }
    if hit.dbsnp.as_ref().and_then(|d| d.rsid.as_ref()).is_some() {
        score += 5;
    }
    score
}

fn best_hit(
    hits: &[crate::sources::myvariant::MyVariantHit],
) -> Option<&crate::sources::myvariant::MyVariantHit> {
    hits.iter().max_by_key(|h| score_myvariant_hit(h))
}

/// A hit carries the ClinVar record for its own genomic variant. A protein
/// change that resolves to such a hit resolves to the variant ClinVar
/// cataloged, not a different transcript's spelling of the same alias.
fn hit_carries_clinvar_record(hit: &crate::sources::myvariant::MyVariantHit) -> bool {
    hit.clinvar
        .as_ref()
        .is_some_and(|clinvar| clinvar.variant_id.is_some() || !clinvar.rcv.is_empty())
}

fn protein_change_candidate(hit: &crate::sources::myvariant::MyVariantHit) -> String {
    let clinvar_id = hit
        .clinvar
        .as_ref()
        .and_then(|clinvar| clinvar.variant_id)
        .map(|variant_id| format!("ClinVar VariationID {variant_id}"));
    let rsid = hit
        .dbsnp
        .as_ref()
        .and_then(|dbsnp| dbsnp.rsid.clone())
        .filter(|rsid| !rsid.trim().is_empty());
    let details = [clinvar_id, rsid]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("; ");
    if details.is_empty() {
        hit.id.trim().to_string()
    } else {
        format!("{} ({details})", hit.id.trim())
    }
}

/// The gene+protein arm's per-query transcript facts: the MANE transcript
/// ClinVar's preferred names or the canonical record's MANE-Select
/// cross-reference mark (None when neither source carries one), and the
/// numbering note for a hit whose headline protein change does not spell
/// the request (tickets 2016 and 2036).
struct ProteinChangeContext {
    mane_transcript: Option<String>,
    numbering_note: Option<String>,
}

/// A gene+protein query names a protein change, not a genomic variant: the
/// same alias can sit on several genomic variants (`DICER1 p.Met1483Ile`
/// spans three alternate bases at chr14:g.95562808), and dbNSFP merges other
/// isoforms' protein names onto other variants, so the alias search also
/// returns lookalikes (`TP53 C124Y` matches the `p.Cys135Tyr` variant
/// chr17:g.7578526C>T through a shorter isoform). A hit carries the query
/// only when the transcript BioMCP headlines for it — the ClinVar-named,
/// MANE, change-naming, or first-NM_ SnpEff annotation — spells the
/// requested change; ClinVar presence breaks ties among those hits (ticket
/// 1297) but never outranks the named change itself (ticket 2016). Resolve
/// a single provider hit, a single carrying hit, or the one ClinVar record
/// names among the true matches; otherwise refuse with every candidate and
/// a working input form.
///
/// `mane_transcript` is the query's MANE marker: the transcript stem
/// ClinVar's preferred names agree on, or the canonical record's
/// MANE-Select cross-reference when the response carries no ClinVar name
/// (ticket 2036). The marker decides which annotation headlines a hit, so
/// it must reach this resolver before any hit is accepted: a ClinVar-free
/// cohort where exactly one candidate names the change on MANE
/// (`BRCA1 S1551Y`) resolves here, while the change-naming tier alone would
/// treat every isoform spelling as a named match and refuse.
fn resolve_protein_change_hit(
    id: &str,
    gene: &str,
    change: &str,
    mut hits: Vec<crate::sources::myvariant::MyVariantHit>,
    mane_transcript: Option<&str>,
) -> Result<crate::sources::myvariant::MyVariantHit, BioMcpError> {
    if hits.is_empty() {
        return Err(BioMcpError::NotFound {
            entity: "variant".into(),
            id: id.to_string(),
            suggestion: format!("Try searching: biomcp search variant -g {gene} --hgvsp {change}"),
        });
    }
    let names_change = |hit: &crate::sources::myvariant::MyVariantHit| {
        hit_names_requested_change(hit, change, mane_transcript)
    };
    if hits.len() == 1 {
        return Ok(hits.into_iter().next().expect("one compatible hit"));
    }
    let named = hits
        .iter()
        .enumerate()
        .filter(|(_, hit)| names_change(hit))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if named.len() == 1 {
        return Ok(hits.swap_remove(named[0]));
    }
    let refusal = |count: usize, reason: &str, candidates: &str| {
        BioMcpError::InvalidArgument(format!(
            "Ambiguous protein change '{id}': {count} variants match and {reason}; \
BioMCP refuses rather than return the wrong variant.\n\
Candidates:\n{candidates}\
Retry `biomcp get variant` with one candidate's exact form: its genomic HGVS, ClinVar VariationID, rsID, or a transcript-qualified HGVS.",
        ))
    };
    if named.is_empty() {
        let candidates = hits
            .iter()
            .map(|hit| format!("- {}\n", protein_change_candidate(hit)))
            .collect::<String>();
        return Err(refusal(
            hits.len(),
            "none of them names that change on its canonical (or ClinVar-named) transcript",
            &candidates,
        ));
    }
    let clinvar_named = named
        .iter()
        .filter(|index| hit_carries_clinvar_record(&hits[**index]))
        .count();
    if clinvar_named == 1 {
        let index = named
            .into_iter()
            .find(|index| hit_carries_clinvar_record(&hits[*index]))
            .expect("one ClinVar-named hit");
        return Ok(hits.swap_remove(index));
    }
    let reason = if clinvar_named == 0 {
        "none of them carries a ClinVar record that names one".to_string()
    } else {
        format!("{clinvar_named} of them carry conflicting ClinVar records")
    };
    let candidates = named
        .iter()
        .map(|index| format!("- {}\n", protein_change_candidate(&hits[*index])))
        .collect::<String>();
    Err(refusal(named.len(), &reason, &candidates))
}

/// The requested protein change confirms a hit only when the transcript
/// BioMCP headlines for it spells that change. The dbNSFP alias list that
/// matched the query merges every isoform's name, so it cannot tell the
/// named variant from a lookalike.
fn hit_names_requested_change(
    hit: &crate::sources::myvariant::MyVariantHit,
    change: &str,
    mane_transcript: Option<&str>,
) -> bool {
    transform::variant::canonical_protein_change(hit, mane_transcript, Some(change))
        .is_some_and(|protein| protein_changes_equivalent(change, &protein))
}

/// A gene+protein query can resolve to a hit whose headline protein change
/// does not spell the request (a unique provider hit, or the only named
/// match): `TP53 R116Q` reaches the `p.Arg248Gln` variant through a shorter
/// isoform's numbering. The answer must say which numbering matched instead
/// of silently returning another transcript's spelling (ticket 2016). One
/// line, in the ticket's voice: name the transcript, its spelling, and that
/// the request follows another transcript's numbering.
fn protein_change_numbering_note(
    hit: &crate::sources::myvariant::MyVariantHit,
    change: &str,
    mane_transcript: Option<&str>,
) -> Option<String> {
    super::normalize_protein_change(change)?;
    let protein = transform::variant::canonical_protein_change(hit, mane_transcript, Some(change))?;
    if protein_changes_equivalent(change, &protein) {
        return None;
    }
    let transcript = transform::variant::canonical_transcript(hit, mane_transcript, Some(change))?;
    Some(format!(
        "Numbering note: resolved on {transcript} as {protein}; the requested \
         {change} follows another transcript's numbering."
    ))
}

/// The requested reference residue and position of a compact substitution
/// (`R209Q` → `('R', 209)`), when the request names one.
fn requested_reference_residue(change: &str) -> Option<(char, u32)> {
    let normalized = super::normalize_protein_substitution(change)?;
    let bytes = normalized.as_bytes();
    let digits = bytes.iter().position(|b| b.is_ascii_digit())?;
    let end = bytes[digits..]
        .iter()
        .position(|b| !b.is_ascii_digit())
        .map(|idx| digits + idx)?;
    if digits != 1 || end < 2 || end + 1 != bytes.len() {
        return None;
    }
    let position = normalized[digits..end].parse::<u32>().ok()?;
    Some((normalized.as_bytes()[0] as char, position))
}

/// The gene's canonical (MANE Select) protein facts from UniProt (tickets
/// 2033 finding 3 and 2036): the accession, the reference residue at the
/// requested position (None when the request names no position or the
/// position falls outside the sequence), and the MANE Select transcript
/// with its current RefSeq version from the record's MANE-Select
/// cross-reference. Any lookup failure returns None and the answer prints
/// no numbering note: the residue check narrows a note that could be false,
/// and an unavailable sequence cannot prove the request's numbering either
/// way (ticket 2035 finding 21).
struct CanonicalProteinFacts {
    accession: String,
    residue: Option<char>,
    /// The canonical sequence's length, so a requested position beyond it
    /// (`BRCA1 Y1866D` against the 1863-residue P38398) is distinguishable
    /// from an unreadable sequence: a position the protein cannot hold
    /// proves the request follows another numbering by itself (ticket 2042).
    sequence_length: Option<u32>,
    mane_transcript: Option<String>,
}

async fn canonical_protein_facts(
    gene: &str,
    position: Option<u32>,
) -> Option<CanonicalProteinFacts> {
    let accession = crate::entities::protein::resolve_accession(gene)
        .await
        .ok()?;
    let record = crate::sources::uniprot::UniProtClient::new()
        .ok()?
        .get_record(&accession)
        .await
        .ok()?;
    let sequence = record
        .sequence
        .as_ref()
        .and_then(|sequence| sequence.value.as_deref())?;
    let residue = position
        .filter(|position| *position > 0)
        .and_then(|position| sequence.chars().nth(position as usize - 1));
    let sequence_length = u32::try_from(sequence.chars().count()).ok();
    let mane_transcript = record.mane_select_transcript();
    Some(CanonicalProteinFacts {
        accession,
        residue,
        sequence_length,
        mane_transcript,
    })
}

/// A resolved hit whose headline does not spell the request on the MANE
/// transcript. Decide the numbering story before anything prints (ticket
/// 2035 finding 4): when the gene's canonical protein carries the requested
/// reference residue at the requested position and no annotation on the
/// marked MANE transcript spells the request, the request's own numbering
/// is valid on MANE while the hit names a different change — the
/// other-transcript note would be false here (`TP53 R209Q` matching
/// `p.Arg248Gln` through a shorter isoform; `TP53 S183Y` whose only alias
/// match spells the request on the shorter isoform while MANE names
/// `p.Ser315Tyr`), so `get variant` refuses rather than return the
/// lookalike (tickets 2033 finding 3 and 2036). When the check instead
/// finds a different residue at the position, the request really does
/// follow another transcript's numbering and the answer says so (ticket
/// 2016). When the request names no residue, or the sequence lookup is
/// unavailable, nothing is proven either way — and an unverified
/// other-transcript claim is exactly the false note this rule removes
/// (ticket 2035 finding 21) — so the answer stays silent.
///
/// `prefetched` carries facts an earlier step already fetched (a
/// ClinVar-free response needs them before resolution, ticket 2036); a
/// `None` with no MANE marker means the lookup already ran and failed, so
/// the answer does not retry the dead source — and an answer with no
/// facts at all must still say so instead of silently resolving another
/// numbering (ticket 2042): the headline spells the request under a note
/// that says plainly the numbering could not be checked, or a headline
/// naming a different change refuses.
async fn refuse_or_note_numbering_mismatch(
    id: &str,
    gene: &str,
    change: &str,
    hit: &crate::sources::myvariant::MyVariantHit,
    mane_transcript: Option<&str>,
    prefetched: Option<CanonicalProteinFacts>,
) -> Result<Option<String>, BioMcpError> {
    if transform::variant::mane_annotation_names_change(hit, change, mane_transcript) {
        return Ok(None);
    }
    let Some((_, position)) = requested_reference_residue(change) else {
        return Ok(None);
    };
    let facts = match prefetched {
        Some(facts) => facts,
        None if mane_transcript.is_some() => {
            match canonical_protein_facts(gene, Some(position)).await {
                Some(facts) => facts,
                None => {
                    return refuse_or_note_without_facts(id, gene, change, hit, mane_transcript);
                }
            }
        }
        None => return refuse_or_note_without_facts(id, gene, change, hit, mane_transcript),
    };
    refuse_or_note_with_facts(id, gene, change, hit, mane_transcript, &facts)
}

/// The numbering story when the canonical facts are unavailable — the
/// gene-to-accession lookup failed, UniProt could not be reached, or the
/// record carried no sequence (ticket 2042). Nothing can be proven either
/// way, so no other-numbering claim prints: a headline that spells the
/// request resolves under a note saying plainly the numbering could not be
/// checked against MANE, and a headline naming a different change refuses
/// rather than answer a renumbered lookalike with no note at all.
fn refuse_or_note_without_facts(
    id: &str,
    gene: &str,
    change: &str,
    hit: &crate::sources::myvariant::MyVariantHit,
    mane_transcript: Option<&str>,
) -> Result<Option<String>, BioMcpError> {
    let candidate = protein_change_candidate(hit);
    let Some(protein) =
        transform::variant::canonical_protein_change(hit, mane_transcript, Some(change))
    else {
        return Err(protein_change_absent_refusal_message(
            id, gene, change, &candidate,
        ));
    };
    let Some(transcript) =
        transform::variant::canonical_transcript(hit, mane_transcript, Some(change))
    else {
        return Err(protein_change_absent_refusal_message(
            id, gene, change, &candidate,
        ));
    };
    if protein_changes_equivalent(change, &protein) {
        return Ok(Some(unchecked_numbering_note(
            &transcript,
            &protein,
            change,
        )));
    }
    Err(unchecked_numbering_refusal_message(
        id,
        gene,
        change,
        &protein,
        &transcript,
        &candidate,
    ))
}

/// The note for a resolved answer whose numbering could not be checked
/// against MANE: the canonical record was unreadable, or it named no MANE
/// transcript (ticket 2042). One spelling for both reasons — the answer
/// says plainly what it could not do, and claims nothing else.
fn unchecked_numbering_note(transcript: &str, protein: &str, change: &str) -> String {
    format!(
        "Numbering note: resolved on {transcript} as {protein}; the requested \
         {change} could not be checked against MANE numbering."
    )
}

/// The numbering decision once the canonical protein facts are in hand —
/// pure, so the recorded shapes pin it offline (ticket 2036).
fn refuse_or_note_with_facts(
    id: &str,
    gene: &str,
    change: &str,
    hit: &crate::sources::myvariant::MyVariantHit,
    mane_transcript: Option<&str>,
    facts: &CanonicalProteinFacts,
) -> Result<Option<String>, BioMcpError> {
    let Some((from, position)) = requested_reference_residue(change) else {
        return Ok(None);
    };
    let Some(residue) = facts.residue else {
        // No residue to compare: a requested position the canonical protein
        // cannot hold proves the request follows another numbering without
        // one (`BRCA1 Y1866D` past the 1863 residues of P38398, ticket 2042),
        // so the other-transcript note prints; a sequence the record never
        // carried proves nothing either way (ticket 2035 finding 21) and the
        // answer stays silent.
        if facts
            .sequence_length
            .is_some_and(|length| position > length)
        {
            // The note names the headline's own transcript and spelling; a
            // record that cannot name one refuses like the no-protein-change
            // case below instead of resolving silently.
            if let Some(note) = protein_change_numbering_note(hit, change, mane_transcript) {
                return Ok(Some(note));
            }
            return Err(protein_change_absent_refusal_message(
                id,
                gene,
                change,
                &protein_change_candidate(hit),
            ));
        }
        return Ok(None);
    };
    if residue != from {
        return Ok(protein_change_numbering_note(hit, change, mane_transcript));
    }
    let Some(reference) = super::amino_acid_three_letter(residue) else {
        return Ok(None);
    };
    let Some(protein) =
        transform::variant::canonical_protein_change(hit, mane_transcript, Some(change))
    else {
        // The request's numbering is valid on the canonical protein, but the
        // record names no protein change on any headline transcript: a
        // protein-change request must not resolve to a bare genomic variant
        // (ticket 2042).
        return Err(protein_change_absent_refusal_message(
            id,
            gene,
            change,
            &protein_change_candidate(hit),
        ));
    };
    if protein_changes_equivalent(change, &protein) {
        // The headline spells the request, so no other-numbering claim can
        // print. With a known MANE transcript the headline is that
        // transcript's own spelling (the caller already returned for a
        // MANE-named change); without one the headline may still be another
        // isoform's spelling of the request, so the answer says plainly the
        // numbering could not be checked against MANE (ticket 2042).
        if mane_transcript.is_some() {
            return Ok(None);
        }
        return Ok(
            transform::variant::canonical_transcript(hit, mane_transcript, Some(change))
                .map(|transcript| unchecked_numbering_note(&transcript, &protein, change)),
        );
    }
    let Some(transcript) =
        transform::variant::canonical_transcript(hit, mane_transcript, Some(change))
    else {
        return Ok(None);
    };
    Err(mane_numbering_refusal_message(
        id,
        gene,
        change,
        &facts.accession,
        reference,
        position,
        &protein,
        &transcript,
        &protein_change_candidate(hit),
    ))
}

/// The refusal text for a request whose numbering is valid on the MANE
/// protein while no matching record names it there (tickets 2033 finding 3
/// and 2036). The retry line names the requested change, not the candidate:
/// the candidate is already known to be a different change, so retrying its
/// exact form cannot answer the request (ticket 2035 finding 21).
#[allow(clippy::too_many_arguments)]
fn mane_numbering_refusal_message(
    id: &str,
    gene: &str,
    change: &str,
    accession: &str,
    reference: &str,
    position: u32,
    protein: &str,
    transcript: &str,
    candidate: &str,
) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "No MANE-numbered variant matches '{id}': the gene's canonical protein \
         (UniProt {accession}) has {reference} at {position}, so the requested \
         numbering is valid there, but no matching record names that change; \
         the only alias match is {protein} on {transcript} — a different \
         change. BioMCP refuses rather than return the wrong variant.\n\
Candidates:\n- {candidate}\n\
Retry `biomcp get variant` with a transcript-qualified HGVS naming \
'{change}', or search the spelling: biomcp search variant -g {gene} \
--hgvsp {change}.",
    ))
}

/// The refusal for a request whose numbering could not be checked because
/// the canonical facts never arrived (ticket 2042): the alias match names a
/// different change, and without the residue check BioMCP cannot tell a
/// MANE-numbered request no record names from another transcript's
/// numbering — so it refuses rather than return the lookalike silently.
fn unchecked_numbering_refusal_message(
    id: &str,
    gene: &str,
    change: &str,
    protein: &str,
    transcript: &str,
    candidate: &str,
) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "No MANE-numbered variant matches '{id}': the canonical (MANE Select) \
         protein could not be read, so the requested numbering could not be \
         checked, and the only alias match is {protein} on {transcript} — a \
         different change. BioMCP refuses rather than return the wrong \
         variant.\n\
Candidates:\n- {candidate}\n\
Retry `biomcp get variant` with a transcript-qualified HGVS naming \
'{change}', or search the spelling: biomcp search variant -g {gene} \
--hgvsp {change}.",
    ))
}

/// The refusal for a protein-change request whose resolved record names no
/// protein change on any headline transcript (ticket 2042): a bare genomic
/// variant cannot carry the numbering story the request asked for.
fn protein_change_absent_refusal_message(
    id: &str,
    gene: &str,
    change: &str,
    candidate: &str,
) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "No protein change answers '{id}': the matching record carries no \
         protein change on any transcript, so the requested {change} could \
         not be checked against MANE numbering. BioMCP refuses rather than \
         return a genomic variant with no protein change.\n\
Candidates:\n- {candidate}\n\
Retry `biomcp get variant` with a transcript-qualified HGVS naming \
'{change}', or search the spelling: biomcp search variant -g {gene} \
--hgvsp {change}.",
    ))
}

fn candidate_matches_requested_identity(
    requested: &super::RequestedVariantIdentity,
    hit: &crate::sources::myvariant::MyVariantHit,
) -> bool {
    matches!(
        super::compare_variant_identity(
            requested,
            &super::SourceVariantIdentity::from_myvariant_hit(hit)
        ),
        super::VariantIdentityComparison::Compatible { .. }
    )
}

fn oncokb_alteration_from_variant(
    variant: &Variant,
    id_format: &VariantIdFormat,
) -> Option<String> {
    match id_format {
        VariantIdFormat::GeneProteinChange { change, .. } => {
            super::normalize_protein_change(change).or_else(|| Some(change.clone()))
        }
        _ => variant
            .hgvs_p
            .as_deref()
            .and_then(super::normalize_protein_change)
            .filter(|s| !s.is_empty()),
    }
}

fn therapies_from_oncokb(annotation: &OncoKBAnnotation) -> Vec<TreatmentImplication> {
    let mut implications: Vec<TreatmentImplication> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for treatment in &annotation.treatments {
        let level = treatment
            .level
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(transform::variant::normalize_oncokb_level)
            .unwrap_or_else(|| "Unknown".to_string());
        let mut drugs = treatment
            .drugs
            .iter()
            .filter_map(|d| d.drug_name.as_deref())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        drugs.sort();
        drugs.dedup();
        let cancer_type = treatment
            .cancer_type
            .as_ref()
            .and_then(|c| c.name.as_deref())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        let dedupe_key = format!(
            "{}|{}|{}",
            level,
            drugs.join("+"),
            cancer_type.as_deref().unwrap_or("")
        );
        if !seen.insert(dedupe_key) {
            continue;
        }
        implications.push(TreatmentImplication {
            level,
            drugs,
            cancer_type,
            note: None,
        });
    }

    implications.sort_by(|a, b| a.level.cmp(&b.level));
    let total = implications.len();
    if total > 6 {
        implications.truncate(6);
        if let Some(last) = implications.last_mut() {
            last.note = Some(format!("(and {} more)", total - 6));
        }
    }
    implications
}

fn normalized_genomic_hgvs_for_get(candidate: &str) -> Option<String> {
    if matches!(
        parse_variant_id(candidate),
        Ok(VariantIdFormat::HgvsGenomic(_))
    ) {
        return Some(candidate.to_string());
    }

    let (accession, suffix) = candidate.split_once(":g.")?;
    let digits = accession
        .strip_prefix("NC_")?
        .split_once('.')?
        .0
        .parse::<u32>()
        .ok()?;
    let chromosome = match digits {
        1..=22 => digits.to_string(),
        23 => "X".to_string(),
        24 => "Y".to_string(),
        _ if accession.starts_with("NC_012920.") => "M".to_string(),
        _ => return None,
    };
    let hgvs = format!("chr{chromosome}:g.{suffix}");
    matches!(parse_variant_id(&hgvs), Ok(VariantIdFormat::HgvsGenomic(_))).then_some(hgvs)
}

fn transcript_hgvs_normalization_error(
    id: &str,
    response: Option<&VariantNormalizationResponse>,
) -> BioMcpError {
    let detail = response
        .and_then(|response| {
            response.services.iter().find_map(|service| match service {
                crate::entities::variant::VariantNormalizationAggregate::Legacy(service) => {
                    service.message.as_deref()
                }
                crate::entities::variant::VariantNormalizationAggregate::Car(_) => None,
            })
        })
        .filter(|message| !message.trim().is_empty())
        .map(|message| format!(" Normalization reported: {message}"))
        .unwrap_or_default();

    BioMcpError::InvalidArgument(format!(
        "Could not normalize transcript HGVS for `get variant`: '{id}'.{detail}\n\n\
Try first: biomcp variant normalize all {id}"
    ))
}

pub(crate) fn normalized_get_variant_id(
    response: &VariantNormalizationResponse,
) -> Result<String, BioMcpError> {
    response
        .services
        .iter()
        .filter_map(|service| match service {
            crate::entities::variant::VariantNormalizationAggregate::Legacy(service)
                if service.status == VariantNormalizationStatus::Success =>
            {
                Some(service.genomic_descriptions.iter())
            }
            crate::entities::variant::VariantNormalizationAggregate::Car(_) => None,
            _ => None,
        })
        .flatten()
        .find_map(|candidate| normalized_genomic_hgvs_for_get(&candidate.coordinate))
        .ok_or_else(|| transcript_hgvs_normalization_error(&response.input, Some(response)))
}

fn transcript_hgvs_clinvar_query(id: &str) -> String {
    format!(
        "clinvar.hgvs.coding:\"{}\"",
        MyVariantClient::escape_query_value(id)
    )
}

fn transcript_hgvs_not_found_suggestion(id: &str) -> String {
    format!(
        "Try first: biomcp variant normalize all {id}; the ClinVar VariationID (e.g. biomcp get variant 577152) or rsID (e.g. biomcp get variant rs113488022) also resolves directly"
    )
}

/// A ClinVar alias hit confirms the transcript-qualified query only when the
/// hit carries the exact coding alias the search asked for. Identity
/// comparison alone cannot confirm it: dbNSFP aliases arrive without
/// transcript prefixes, so an unconfirmed hit stays refused.
fn hit_confirms_transcript_alias(hit: &crate::sources::myvariant::MyVariantHit, id: &str) -> bool {
    hit.clinvar
        .as_ref()
        .and_then(|clinvar| clinvar.hgvs.as_ref())
        .is_some_and(|hgvs| hgvs.coding_contains(id))
}

async fn transcript_hgvs_clinvar_alias_hit(
    myvariant: &MyVariantClient,
    id: &str,
) -> Result<Option<crate::sources::myvariant::MyVariantHit>, BioMcpError> {
    let q = transcript_hgvs_clinvar_query(id);
    let resp = myvariant
        .query_with_fields(&q, 10, 0, crate::sources::myvariant::MYVARIANT_FIELDS_GET)
        .await?;
    Ok(best_hit(
        &resp
            .hits
            .into_iter()
            .filter(|hit| hit_confirms_transcript_alias(hit, id))
            .collect::<Vec<_>>(),
    )
    .cloned())
}

async fn resolve_transcript_hgvs_for_get(id: &str) -> Result<VariantIdFormat, BioMcpError> {
    match normalize_transcript_hgvs_for_get(id).await {
        Ok(format) => Ok(format),
        // Transcript normalization services refuse intronic deletion ranges
        // and other aliases they cannot place; ClinVar's own coding alias list
        // still names them, so the alias search resolves what normalization
        // cannot (ticket 1292).
        Err(_) => {
            let myvariant = MyVariantClient::new()?;
            transcript_hgvs_clinvar_alias_hit(&myvariant, id)
                .await?
                .filter(|hit| {
                    matches!(
                        parse_variant_id(hit.id.trim()),
                        Ok(VariantIdFormat::HgvsGenomic(_))
                    )
                })
                .map(|hit| VariantIdFormat::HgvsGenomic(hit.id.trim().to_string()))
                .ok_or_else(|| BioMcpError::NotFound {
                    entity: "variant".into(),
                    id: id.to_string(),
                    suggestion: transcript_hgvs_not_found_suggestion(id),
                })
        }
    }
}

async fn normalize_transcript_hgvs_for_get(id: &str) -> Result<VariantIdFormat, BioMcpError> {
    let response = normalize_variant("all", id)
        .await
        .map_err(|_| transcript_hgvs_normalization_error(id, None))?;
    let normalized_id = normalized_get_variant_id(&response)?;
    parse_variant_id(&normalized_id)
}

fn build_aware_not_found(id: &str, build: GenomeBuild, error: BioMcpError) -> BioMcpError {
    if !error.is_not_found() {
        return error;
    }
    let build = match build {
        GenomeBuild::Grch37 => "GRCh37",
        GenomeBuild::Grch38 => "GRCh38",
    };
    BioMcpError::NotFound {
        entity: "variant".into(),
        id: format!("{id} (attempted {build}; upstream HTTP 404)"),
        suggestion: "Try searching: biomcp search variant".into(),
    }
}

pub(super) async fn resolve_base_with_hit(
    id: &str,
    genome_build: Option<GenomeBuild>,
) -> Result<
    (
        Variant,
        VariantIdFormat,
        crate::sources::myvariant::MyVariantHit,
    ),
    BioMcpError,
> {
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Variant ID is required. Example: biomcp get variant rs113488022".into(),
        ));
    }

    let input_kind = classify_variant_input(id);
    let normalized_coordinate = super::normalize_genomic_coordinate(id)?;
    let mut requested = match normalized_coordinate.as_ref() {
        Some(coordinate) => super::RequestedVariantIdentity::from_variant_input(&coordinate.id)?,
        None => super::RequestedVariantIdentity::from_variant_input(id)?,
    };
    let id_format = match (input_kind.clone(), normalized_coordinate.as_ref()) {
        (_, Some(coordinate)) => VariantIdFormat::HgvsGenomic(coordinate.id.clone()),
        (VariantInputKind::TranscriptCodingHgvs(_), None) => {
            resolve_transcript_hgvs_for_get(id).await?
        }
        _ => parse_variant_id(id)?,
    };
    if let VariantIdFormat::HgvsGenomic(hgvs) = &id_format
        && requested.genomic_accession.is_none()
    {
        requested.populate_genomic(hgvs);
    }
    let inferred_build = normalized_coordinate
        .as_ref()
        .and_then(|coordinate| coordinate.genome_build);
    if let (Some(declared), Some(inferred)) = (genome_build, inferred_build)
        && declared != inferred
    {
        return Err(BioMcpError::InvalidArgument(
            "--assembly conflicts with the genomic coordinate's genome build".into(),
        ));
    }
    let effective_build = inferred_build.or(genome_build);

    let compatible = |hit: &crate::sources::myvariant::MyVariantHit| {
        candidate_matches_requested_identity(&requested, hit)
    };
    let myvariant = MyVariantClient::new()?;
    let (hit, answering_build, build_candidates, protein_change_context) = match &id_format {
        VariantIdFormat::HgvsGenomic(hgvs) => {
            if normalized_coordinate
                .as_ref()
                .is_some_and(|coordinate| coordinate.requires_comparison)
            {
                let preferred = effective_build.unwrap_or(GenomeBuild::Grch38);
                let other = if preferred == GenomeBuild::Grch38 {
                    GenomeBuild::Grch37
                } else {
                    GenomeBuild::Grch38
                };
                let preferred_hit = myvariant.get(hgvs, Some(preferred)).await;
                let other_hit = myvariant.get(hgvs, Some(other)).await;
                match (preferred_hit, other_hit) {
                    (Ok(preferred_hit), Ok(other_hit)) => {
                        let candidates =
                            if super::SourceVariantIdentity::from_myvariant_hit(&preferred_hit)
                                .normalized_key()
                                != super::SourceVariantIdentity::from_myvariant_hit(&other_hit)
                                    .normalized_key()
                            {
                                vec![super::VariantBuildCandidate {
                                    genome_build: other,
                                    id: other_hit.id.clone(),
                                    rsid: transform::variant::from_myvariant_hit(&other_hit).rsid,
                                }]
                            } else {
                                Vec::new()
                            };
                        (preferred_hit, Some(preferred), candidates, None)
                    }
                    (Ok(hit), Err(error)) if error.is_not_found() => {
                        (hit, Some(preferred), Vec::new(), None)
                    }
                    (Err(error), Ok(hit)) if error.is_not_found() => {
                        (hit, Some(other), Vec::new(), None)
                    }
                    (Err(first), Err(second)) if first.is_not_found() && second.is_not_found() => {
                        return Err(BioMcpError::NotFound {
                            entity: "variant".into(),
                            id: format!("{hgvs} (tried GRCh38 and GRCh37; upstream HTTP 404)"),
                            suggestion: "Try searching: biomcp search variant".into(),
                        });
                    }
                    (Err(error), _) | (_, Err(error)) => return Err(error),
                }
            } else {
                let direct = myvariant.get(hgvs, effective_build).await;
                if matches!(input_kind, VariantInputKind::TranscriptCodingHgvs(_))
                    && direct.is_err()
                {
                    let alias_hit = transcript_hgvs_clinvar_alias_hit(&myvariant, id)
                        .await?
                        .ok_or_else(|| BioMcpError::NotFound {
                            entity: "variant".into(),
                            id: id.to_string(),
                            suggestion: transcript_hgvs_not_found_suggestion(id),
                        })?;
                    (
                        alias_hit,
                        effective_build.or(Some(GenomeBuild::Grch37)),
                        Vec::new(),
                        None,
                    )
                } else {
                    let hit = direct.map_err(|error| match effective_build {
                        Some(build) => build_aware_not_found(hgvs, build, error),
                        None => error,
                    })?;
                    let transcript_input =
                        matches!(input_kind, VariantInputKind::TranscriptCodingHgvs(_));
                    let alias_confirmed =
                        transcript_input && hit_confirms_transcript_alias(&hit, id);
                    if !compatible(&hit) && !alias_confirmed {
                        let suggestion = if transcript_input {
                            transcript_hgvs_not_found_suggestion(id)
                        } else {
                            format!("Try searching: biomcp search variant -g \"{id}\"")
                        };
                        return Err(BioMcpError::NotFound {
                            entity: "variant".into(),
                            id: id.to_string(),
                            suggestion,
                        });
                    }
                    (hit, effective_build, Vec::new(), None)
                }
            }
        }
        VariantIdFormat::ClinvarVariationId(variation_id) => {
            let q = format!("clinvar.variant_id:{variation_id}");
            let resp = myvariant
                .query_with_fields(&q, 10, 0, crate::sources::myvariant::MYVARIANT_FIELDS_GET)
                .await?;
            (
                best_hit(&resp.hits)
                    .cloned()
                    .ok_or_else(|| BioMcpError::NotFound {
                        entity: "variant".into(),
                        id: format!("ClinVar VariationID {variation_id}"),
                        suggestion: "Try searching: biomcp search variant".into(),
                    })?,
                Some(GenomeBuild::Grch37),
                Vec::new(),
                None,
            )
        }
        VariantIdFormat::RsId(rsid) => {
            let q = format!("dbsnp.rsid:{rsid}");
            let resp = myvariant
                .query_with_fields(&q, 10, 0, crate::sources::myvariant::MYVARIANT_FIELDS_GET)
                .await?;
            let compatible_hits = resp
                .hits
                .into_iter()
                .filter(&compatible)
                .collect::<Vec<_>>();
            (
                best_hit(&compatible_hits)
                    .cloned()
                    .ok_or_else(|| BioMcpError::NotFound {
                        entity: "variant".into(),
                        id: rsid.to_string(),
                        suggestion: format!("Try searching: biomcp search variant -g \"{id}\""),
                    })?,
                Some(GenomeBuild::Grch37),
                Vec::new(),
                None,
            )
        }
        VariantIdFormat::GeneProteinChange { gene, change } => {
            let q = format!(
                "dbnsfp.genename:{} AND dbnsfp.hgvsp:\"p.{}\"",
                gene,
                MyVariantClient::escape_query_value(change)
            );
            let resp = myvariant
                .query_with_fields(&q, 5, 0, crate::sources::myvariant::MYVARIANT_FIELDS_GET)
                .await?;
            let compatible_hits = resp
                .hits
                .into_iter()
                .filter(&compatible)
                .collect::<Vec<_>>();
            let clinvar_mane = transform::variant::clinvar_mane_transcript(&compatible_hits);
            // The canonical-residue check runs before any annotation that is
            // not on the MANE transcript is accepted (ticket 2036). A
            // ClinVar-free response marks no MANE transcript of its own, so
            // the gene's canonical (MANE Select) protein must be read up
            // front: the record's MANE-Select cross-reference names the MANE
            // transcript with its current version, and its sequence carries
            // the requested position's residue. Without those facts an
            // isoform annotation that spells the request would resolve with
            // no check (`TP53 S183Y` → `p.Ser183Tyr` on NM_001126115.1
            // while MANE names `p.Ser315Tyr`).
            let canonical = if clinvar_mane.is_none() && !compatible_hits.is_empty() {
                canonical_protein_facts(
                    gene,
                    requested_reference_residue(change).map(|(_, position)| position),
                )
                .await
            } else {
                None
            };
            let mane_transcript = clinvar_mane.or_else(|| {
                canonical
                    .as_ref()
                    .and_then(|facts| facts.mane_transcript.clone())
            });
            let hit = resolve_protein_change_hit(
                id,
                gene,
                change,
                compatible_hits,
                mane_transcript.as_deref(),
            )?;
            // The residue check runs before any note prints (ticket 2035
            // finding 4), so a ClinVar-free response never answers a
            // MANE-numbered request with another transcript's spelling and
            // a note claiming the request follows different numbering.
            let numbering_note = match refuse_or_note_numbering_mismatch(
                id,
                gene,
                change,
                &hit,
                mane_transcript.as_deref(),
                canonical,
            )
            .await
            {
                Err(error) => return Err(error),
                Ok(note) => note,
            };
            (
                hit,
                Some(GenomeBuild::Grch37),
                Vec::new(),
                Some(ProteinChangeContext {
                    mane_transcript,
                    numbering_note,
                }),
            )
        }
    };

    let requested_change = match &id_format {
        VariantIdFormat::GeneProteinChange { change, .. } => Some(change.as_str()),
        _ => None,
    };
    let mut variant = match protein_change_context.as_ref() {
        Some(context) => {
            let mut variant = transform::variant::from_myvariant_hit_with_mane(
                &hit,
                context.mane_transcript.as_deref(),
                requested_change,
            );
            variant.protein_numbering_note = context.numbering_note.clone();
            variant
        }
        None => transform::variant::from_myvariant_hit(&hit),
    };
    variant.genome_build = answering_build;
    variant.genome_build_provenance = (answering_build == Some(GenomeBuild::Grch37)
        && effective_build.is_none()
        && (!matches!(id_format, VariantIdFormat::HgvsGenomic(_))
            || matches!(input_kind, VariantInputKind::TranscriptCodingHgvs(_))))
    .then(|| "MyVariant.info provider default".into());
    variant.build_ambiguous = (!build_candidates.is_empty()).then_some(true);
    variant.build_candidates = build_candidates;
    Ok((variant, id_format, hit))
}

async fn resolve_base(
    id: &str,
    genome_build: Option<GenomeBuild>,
) -> Result<(Variant, VariantIdFormat), BioMcpError> {
    let (variant, id_format, _) = resolve_base_with_hit(id, genome_build).await?;
    Ok((variant, id_format))
}

pub async fn oncokb(id: &str) -> Result<VariantOncoKbResult, BioMcpError> {
    let (variant, id_format) = resolve_base(id, None).await?;
    let gene = variant.gene.trim();
    if gene.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "OncoKB lookup requires a variant that resolves to a gene symbol".into(),
        ));
    }

    let alteration = oncokb_alteration_from_variant(&variant, &id_format)
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(
                "OncoKB lookup requires a protein change (e.g., `BRAF V600E`)".into(),
            )
        })?
        .trim()
        .to_string();
    if alteration.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "OncoKB lookup requires a non-empty protein alteration".into(),
        ));
    }

    let client = OncoKBClient::new()?;
    let annotation = client.annotate_best_effort(gene, &alteration).await?;
    let oncogenic = annotation
        .oncogenic
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let level = annotation
        .highest_sensitive_level
        .as_deref()
        .map(transform::variant::normalize_oncokb_level)
        .filter(|v| !v.is_empty())
        .or_else(|| {
            annotation
                .highest_resistance_level
                .as_deref()
                .map(transform::variant::normalize_oncokb_level)
                .filter(|v| !v.is_empty())
        });
    let effect = annotation
        .mutation_effect
        .as_ref()
        .and_then(|m| m.known_effect.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    Ok(VariantOncoKbResult {
        gene: gene.to_string(),
        alteration,
        oncogenic,
        level,
        effect,
        therapies: therapies_from_oncokb(&annotation),
    })
}

const VARIANT_SOURCE_UNAVAILABLE: &str =
    "Requested variant source data is temporarily unavailable.";
#[cfg(feature = "alphagenome")]
async fn add_prediction(variant: &mut Variant) -> Result<(), BioMcpError> {
    let Some(caps) = hgvs_coords_re().captures(&variant.id) else {
        variant.section_outcomes.complete(
            "predict",
            SectionOutcome::inapplicable("Genomic coordinates are required for prediction."),
        );
        return Ok(());
    };

    let chr = caps[1].to_string();
    let pos: i64 = caps[2]
        .parse()
        .map_err(|_| BioMcpError::InvalidArgument("Invalid HGVS position for prediction".into()))?;
    let reference = caps[3].to_string();
    let alternate = caps[4].to_string();

    let client = match AlphaGenomeClient::new().await {
        Ok(client) => client,
        Err(_) => {
            variant.section_outcomes.complete(
                "predict",
                SectionOutcome::unavailable(VARIANT_SOURCE_UNAVAILABLE),
            );
            return Ok(());
        }
    };
    match client
        .score_variant(&chr, pos, &reference, &alternate)
        .await
    {
        Ok(mut pred) => {
            if let Some(top_gene) = pred.top_gene.as_deref()
                && top_gene.trim().starts_with("ENSG")
            {
                let query = format!("ensembl.gene:\"{}\"", top_gene.trim());
                if let Ok(client) = MyGeneClient::new()
                    && let Ok(resp) = client.search(&query, 1, 0, None).await
                    && let Some(symbol) = resp
                        .hits
                        .first()
                        .and_then(|h| h.symbol.as_deref())
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                {
                    pred.top_gene = Some(symbol.to_string());
                }
            }
            transform::variant::merge_prediction(variant, pred);
            let outcome = if variant.prediction.is_some() {
                SectionOutcome::data("AlphaGenome")
            } else {
                SectionOutcome::empty("AlphaGenome")
            };
            variant.section_outcomes.complete("predict", outcome);
        }
        Err(_) => variant.section_outcomes.complete(
            "predict",
            SectionOutcome::unavailable(VARIANT_SOURCE_UNAVAILABLE),
        ),
    }

    Ok(())
}

#[cfg(not(feature = "alphagenome"))]
async fn add_prediction(variant: &mut Variant) -> Result<(), BioMcpError> {
    variant.section_outcomes.complete(
        "predict",
        SectionOutcome::unavailable("AlphaGenome support was not built into this binary."),
    );
    Ok(())
}

async fn add_cancerhotspots(variant: &mut Variant, id_format: &VariantIdFormat) {
    let inapplicable = || {
        SectionOutcome::inapplicable(
            "A gene and normalizable protein change are required for Cancer Hotspots.",
        )
    };
    let VariantIdFormat::GeneProteinChange { gene, change } = id_format else {
        variant
            .section_outcomes
            .complete("cancerhotspots", inapplicable());
        return;
    };
    let Some(normalized_change) = super::normalize_protein_change(change) else {
        variant
            .section_outcomes
            .complete("cancerhotspots", inapplicable());
        return;
    };
    let gene = gene.trim();
    if gene.is_empty() {
        variant
            .section_outcomes
            .complete("cancerhotspots", inapplicable());
        return;
    }

    let cancerhotspots_fut = async {
        let client = CancerHotspotsClient::new()?;
        let rows = client.by_gene(gene).await?;
        Ok::<_, BioMcpError>(crate::sources::cancerhotspots::recurrence_for_change(
            &rows,
            &normalized_change,
        ))
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, cancerhotspots_fut).await {
        Ok(Ok(recurrence)) => {
            let outcome = cancerhotspots_outcome(&recurrence);
            variant.cancerhotspots = Some(recurrence);
            variant.section_outcomes.complete("cancerhotspots", outcome);
        }
        Ok(Err(_)) | Err(_) => variant.section_outcomes.complete(
            "cancerhotspots",
            SectionOutcome::unavailable(VARIANT_SOURCE_UNAVAILABLE),
        ),
    }
}

fn cancerhotspots_outcome(
    recurrence: &crate::sources::cancerhotspots::CancerHotspotRecurrence,
) -> SectionOutcome {
    if recurrence.position_count.is_some()
        || recurrence.same_aa_count.is_some()
        || recurrence.matched_transcript.is_some()
    {
        SectionOutcome::data("cancerhotspots.org")
    } else {
        SectionOutcome::empty("cancerhotspots.org")
    }
}

#[cfg(test)]
fn apply_cancerhotspots_result(
    variant: &mut Variant,
    result: Result<crate::sources::cancerhotspots::CancerHotspotRecurrence, BioMcpError>,
) -> Result<(), BioMcpError> {
    match result {
        Ok(recurrence) => {
            variant.cancerhotspots = Some(recurrence);
            Ok(())
        }
        Err(err) => Err(err),
    }
}

async fn add_cbioportal(variant: &mut Variant) {
    let gene = variant.gene.trim();
    if gene.is_empty() {
        variant.section_outcomes.complete(
            "cbioportal",
            SectionOutcome::inapplicable("A gene is required for cancer frequency lookup."),
        );
        return;
    }

    let cbio_fut = async {
        let client = CBioPortalClient::new()?;
        let summary = client.get_mutation_summary(gene).await?;
        Ok::<_, BioMcpError>(summary)
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, cbio_fut).await {
        Ok(Ok(summary)) => {
            transform::variant::merge_cbioportal(variant, &summary);
            let outcome = if variant.cancer_frequencies.is_empty() {
                SectionOutcome::empty("cBioPortal")
            } else {
                SectionOutcome::data("cBioPortal")
            };
            variant.section_outcomes.complete("cbioportal", outcome);
        }
        Ok(Err(_)) | Err(_) => variant.section_outcomes.complete(
            "cbioportal",
            SectionOutcome::unavailable(VARIANT_SOURCE_UNAVAILABLE),
        ),
    }
}

fn civic_molecular_profile_name(variant: &Variant) -> Option<String> {
    let gene = variant.gene.trim();
    if gene.is_empty() {
        return None;
    }

    if let Some(hgvs_p) = variant
        .hgvs_p
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let normalized = hgvs_p.strip_prefix("p.").unwrap_or(hgvs_p).trim();
        if !normalized.is_empty() {
            return Some(format!("{gene} {normalized}"));
        }
    }

    None
}

async fn add_civic(variant: &mut Variant) {
    let Some(molecular_profile_name) = civic_molecular_profile_name(variant) else {
        variant.section_outcomes.complete(
            "civic",
            SectionOutcome::inapplicable(
                "A gene and protein change are required for clinical evidence lookup.",
            ),
        );
        return;
    };

    let civic_fut = async {
        let client = CivicClient::new()?;
        client
            .by_molecular_profile(&molecular_profile_name, 10)
            .await
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, civic_fut).await {
        Ok(Ok(context)) => {
            let has_data = context.evidence_total_count > 0
                || context.assertion_total_count > 0
                || !context.evidence_items.is_empty()
                || !context.assertions.is_empty();
            let section = variant
                .civic
                .get_or_insert_with(VariantCivicSection::default);
            section.graphql = Some(context);
            let outcome = if has_data {
                SectionOutcome::data("CIViC")
            } else {
                SectionOutcome::empty("CIViC")
            };
            variant.section_outcomes.complete("civic", outcome);
        }
        Ok(Err(_)) | Err(_) => variant.section_outcomes.complete(
            "civic",
            SectionOutcome::unavailable(VARIANT_SOURCE_UNAVAILABLE),
        ),
    }
}

fn is_gwas_only_request(flags: &VariantSections) -> bool {
    flags.include_gwas
        && !flags.include_prediction
        && !flags.include_expanded_predictions
        && !flags.include_clinvar
        && !flags.include_population
        && !flags.include_conservation
        && !flags.include_cosmic
        && !flags.include_cgi
        && !flags.include_civic
        && !flags.include_cbioportal
        && !flags.include_cancerhotspots
}

fn gwas_only_variant_stub(rsid: &str) -> Variant {
    Variant {
        section_outcomes: super::default_variant_section_outcomes(),
        gene: String::new(),
        id: rsid.to_string(),
        genome_build: None,
        genome_build_provenance: None,
        build_ambiguous: None,
        build_candidates: Vec::new(),
        hgvs_p: None,
        legacy_name: None,
        hgvs_c: None,
        transcript: None,
        protein_numbering_note: None,
        rsid: Some(rsid.to_string()),
        cosmic_id: None,
        significance: None,
        significance_source: None,
        significance_evaluated: None,
        significance_note: None,
        clinvar_id: None,
        clinvar_review_status: None,
        clinvar_review_stars: None,
        conditions: Vec::new(),
        clinvar: None,
        consequence: None,
        cadd_score: None,
        sift_pred: None,
        polyphen_pred: None,
        conservation: None,
        expanded_predictions: Vec::new(),
        population: None,
        cosmic_context: None,
        cgi_associations: Vec::new(),
        civic: None,
        clinvar_conditions: Vec::new(),
        clinvar_condition_reports: None,
        top_disease: None,
        cancerhotspots: None,
        cancer_frequencies: Vec::new(),
        cancer_frequency_source: None,
        gwas: Vec::new(),
        gwas_unavailable_reason: None,
        supporting_pmids: None,
        prediction: None,
    }
}

pub(super) fn strip_clinvar_details(variant: &mut Variant) {
    variant.conditions.clear();
    variant.clinvar_conditions.clear();
    variant.clinvar_condition_reports = None;
    variant.top_disease = None;
    variant.clinvar_id = None;
    variant.clinvar_review_status = None;
    variant.clinvar_review_stars = None;
    variant.clinvar = None;
}

/// Move the headline significance to the direct record-level germline
/// classification when the NCBI ClinVar VCV record carries one. Without a
/// record-level classification the derived value stays labeled as derived.
pub(super) fn apply_record_level_headline(variant: &mut Variant, record: &ClinvarRecord) {
    let derived = variant.significance.clone();
    let record_level = record.germline_classification.as_ref().and_then(|row| {
        row.classification
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|classification| (classification.to_string(), row))
    });
    let Some((classification, row)) = record_level else {
        if derived.is_some() {
            variant.significance_note = Some(
                "NCBI ClinVar's record has no record-level germline classification; the headline significance remains the most severe RCV classification derived from MyVariant.info's cached ClinVar copy.".into(),
            );
        }
        return;
    };
    variant.significance = Some(classification.clone());
    variant.significance_source = Some(record.source.clone());
    variant.significance_evaluated = row.evaluation_date.clone();
    variant.clinvar_review_status = row.review_status.clone();
    variant.clinvar_review_stars = row
        .review_status
        .as_deref()
        .and_then(crate::transform::variant::clinvar_review_stars);
    variant.significance_note = match derived.as_deref() {
        Some(value) if value != classification => Some(format!(
            "NCBI ClinVar's record-level germline classification ({classification}) disagrees with the most severe RCV classification in MyVariant.info's cached ClinVar copy ({value})."
        )),
        _ => None,
    };
}

fn strip_civic_live_details(variant: &mut Variant) {
    let Some(civic) = variant.civic.as_mut() else {
        return;
    };
    civic.graphql = None;
    if civic.cached_evidence.is_empty() {
        variant.civic = None;
    }
}

fn population_result(
    status: GnomadPopulationStatus,
    message: Option<&str>,
    data: Option<GnomadVariantPopulation>,
    resolved_coordinate: Option<ResolvedPopulationCoordinate>,
) -> GnomadPopulationResult {
    let (exome, genome) = data
        .map(|population| (population.exome, population.genome))
        .unwrap_or_default();
    GnomadPopulationResult {
        status,
        dataset: GNOMAD_DATASET.into(),
        release: GNOMAD_RELEASE.into(),
        resolved_coordinate,
        message: message.map(str::to_string),
        exome,
        genome,
        faf_caveat: GNOMAD_FAF_CAVEAT.into(),
    }
}

fn population_variant_id(variant: &Variant) -> Option<String> {
    (variant.genome_build == Some(GenomeBuild::Grch38))
        .then(|| gnomad_variant_slug(&variant.id))
        .flatten()
}

fn dbsnp_population_rsid<'a>(variant: &'a Variant, id_format: &VariantIdFormat) -> Option<&'a str> {
    matches!(
        id_format,
        VariantIdFormat::RsId(_) | VariantIdFormat::GeneProteinChange { .. }
    )
    .then(|| variant.rsid.as_deref())
    .flatten()
    .map(str::trim)
    .filter(|rsid| !rsid.is_empty())
}

async fn add_population(variant: &mut Variant, id_format: &VariantIdFormat) {
    let (variant_id, resolved_coordinate) = match population_variant_id(variant) {
        Some(variant_id) => (variant_id, None),
        None => {
            let Some(rsid) = dbsnp_population_rsid(variant, id_format) else {
                variant.population = Some(population_result(
                    GnomadPopulationStatus::Missing,
                    Some(GNOMAD_GRCH38_REQUIRED),
                    None,
                    None,
                ));
                variant.section_outcomes.complete(
                    VARIANT_SECTION_POPULATION,
                    SectionOutcome::inapplicable(GNOMAD_GRCH38_REQUIRED),
                );
                return;
            };
            let coordinate = match DbSnpClient::new() {
                Ok(client) => match tokio::time::timeout(
                    optional_enrichment_timeout(),
                    client.resolve_grch38_coordinate(rsid, &variant.id),
                )
                .await
                {
                    Ok(Ok(Some(coordinate))) => coordinate,
                    Ok(Ok(None)) => {
                        variant.population = Some(population_result(
                            GnomadPopulationStatus::Missing,
                            Some(GNOMAD_DBSNP_GRCH38_REQUIRED),
                            None,
                            None,
                        ));
                        variant.section_outcomes.complete(
                            VARIANT_SECTION_POPULATION,
                            SectionOutcome::inapplicable(GNOMAD_DBSNP_GRCH38_REQUIRED),
                        );
                        return;
                    }
                    Ok(Err(_)) | Err(_) => {
                        variant.population = Some(population_result(
                            GnomadPopulationStatus::ProviderFailure,
                            Some(DBSNP_PROVIDER_FAILURE),
                            None,
                            None,
                        ));
                        variant.section_outcomes.complete(
                            VARIANT_SECTION_POPULATION,
                            SectionOutcome::unavailable(DBSNP_PROVIDER_FAILURE),
                        );
                        return;
                    }
                },
                Err(_) => {
                    variant.population = Some(population_result(
                        GnomadPopulationStatus::ProviderFailure,
                        Some(DBSNP_PROVIDER_FAILURE),
                        None,
                        None,
                    ));
                    variant.section_outcomes.complete(
                        VARIANT_SECTION_POPULATION,
                        SectionOutcome::unavailable(DBSNP_PROVIDER_FAILURE),
                    );
                    return;
                }
            };
            let Some(variant_id) = gnomad_variant_slug(&coordinate.id) else {
                variant.population = Some(population_result(
                    GnomadPopulationStatus::Missing,
                    Some(GNOMAD_DBSNP_GRCH38_REQUIRED),
                    None,
                    None,
                ));
                variant.section_outcomes.complete(
                    VARIANT_SECTION_POPULATION,
                    SectionOutcome::inapplicable(GNOMAD_DBSNP_GRCH38_REQUIRED),
                );
                return;
            };
            (
                variant_id,
                Some(ResolvedPopulationCoordinate {
                    id: coordinate.id,
                    genome_build: GenomeBuild::Grch38,
                    source: "dbSNP".into(),
                }),
            )
        }
    };

    let response = match GnomadClient::new() {
        Ok(client) => match tokio::time::timeout(
            optional_enrichment_timeout(),
            client.variant_population(&variant_id),
        )
        .await
        {
            Ok(result) => result.map_err(|_| ()),
            Err(_) => Err(()),
        },
        Err(_) => Err(()),
    };
    let response = response.and_then(|data| match data {
        Some(data) if data.variant_id != variant_id => Err(()),
        data => Ok(data),
    });
    let dbsnp_assisted = resolved_coordinate.is_some();
    let sources = || {
        if dbsnp_assisted {
            SectionOutcome::data_sources(["dbSNP", "gnomAD v4"])
        } else {
            SectionOutcome::data("gnomAD v4")
        }
    };
    let empty_sources = || {
        if dbsnp_assisted {
            SectionOutcome::empty_sources(["dbSNP", "gnomAD v4"])
        } else {
            SectionOutcome::empty("gnomAD v4")
        }
    };
    match response {
        Ok(Some(data)) if data.exome.is_some() || data.genome.is_some() => {
            variant.population = Some(population_result(
                GnomadPopulationStatus::Data,
                None,
                Some(data),
                resolved_coordinate,
            ));
            variant
                .section_outcomes
                .complete(VARIANT_SECTION_POPULATION, sources());
        }
        Ok(_) => {
            variant.population = Some(population_result(
                GnomadPopulationStatus::Absent,
                Some("This variant is absent from gnomAD v4."),
                None,
                resolved_coordinate,
            ));
            variant
                .section_outcomes
                .complete(VARIANT_SECTION_POPULATION, empty_sources());
        }
        Err(_) => {
            variant.population = Some(population_result(
                GnomadPopulationStatus::ProviderFailure,
                Some(GNOMAD_PROVIDER_FAILURE),
                None,
                resolved_coordinate,
            ));
            variant.section_outcomes.complete(
                VARIANT_SECTION_POPULATION,
                SectionOutcome::unavailable(GNOMAD_PROVIDER_FAILURE),
            );
        }
    }
}

pub async fn get(id: &str, sections: &[String]) -> Result<Variant, BioMcpError> {
    Ok(get_with_workflow_signals(id, sections, None).await?.0)
}

fn has_clinvar_workflow_signal(variant: &Variant) -> bool {
    variant
        .clinvar_id
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || variant
            .significance
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || !variant.conditions.is_empty()
        || !variant.clinvar_conditions.is_empty()
        || variant.clinvar_condition_reports.is_some()
}

pub async fn get_with_workflow_signals(
    id: &str,
    sections: &[String],
    genome_build: Option<GenomeBuild>,
) -> Result<(Variant, VariantWorkflowSignals), BioMcpError> {
    let section_flags = parse_sections(sections)?;
    if is_gwas_only_request(&section_flags)
        && let VariantIdFormat::RsId(rsid) = parse_variant_id(id)?
    {
        let mut variant = gwas_only_variant_stub(&rsid);
        add_gwas_section(&mut variant, id).await?;
        return Ok((variant, VariantWorkflowSignals::default()));
    }

    let (mut variant, id_format, hit) = resolve_base_with_hit(id, genome_build).await?;
    let signals = VariantWorkflowSignals {
        has_clinvar_signal: has_clinvar_workflow_signal(&variant),
    };

    if !section_flags.include_clinvar {
        strip_clinvar_details(&mut variant);
    }
    if !section_flags.include_conservation {
        variant.conservation = None;
    }
    if !section_flags.include_expanded_predictions {
        variant.expanded_predictions.clear();
    }
    if !section_flags.include_population {
        variant.population = None;
    }
    if !section_flags.include_cosmic {
        variant.cosmic_context = None;
    }
    if !section_flags.include_cgi {
        variant.cgi_associations.clear();
    }
    if !section_flags.include_civic {
        strip_civic_live_details(&mut variant);
    }
    if !section_flags.include_cbioportal {
        variant.cancer_frequencies.clear();
    }
    if !section_flags.include_cancerhotspots {
        variant.cancerhotspots = None;
    }
    if !section_flags.include_gwas {
        variant.gwas.clear();
        variant.gwas_unavailable_reason = None;
        variant.supporting_pmids = None;
    }
    if section_flags.include_prediction {
        add_prediction(&mut variant).await?;
    }
    if section_flags.include_clinvar {
        super::clinvar::add_clinvar(&mut variant, &hit, optional_enrichment_timeout()).await;
    }
    if section_flags.include_population {
        add_population(&mut variant, &id_format).await;
    }
    if section_flags.include_cbioportal {
        add_cbioportal(&mut variant).await;
    }
    if section_flags.include_cancerhotspots {
        add_cancerhotspots(&mut variant, &id_format).await;
    }
    if section_flags.include_civic {
        add_civic(&mut variant).await;
    }
    if section_flags.include_gwas {
        add_gwas_section(&mut variant, id).await?;
    }

    Ok((variant, signals))
}

#[cfg(test)]
mod protein_change_tests;

#[cfg(test)]
mod tests;
