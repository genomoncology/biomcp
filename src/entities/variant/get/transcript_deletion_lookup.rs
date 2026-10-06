//! Exact complete source tuples, retained before legacy identity deduplication.
use super::coding_lookup;
use crate::entities::variant::SourceVariantIdentity;
use crate::entities::variant::resolution::transcript_deletion_get::{
    TranscriptDeletion, coding_valid, prepare, tuple, versioned_transcript,
};
use crate::error::BioMcpError;
use crate::sources::myvariant::{MyVariantClient, MyVariantHit};
use biodata::parse_hgvs_protein_interval_21_1_4;
use std::collections::{HashMap, HashSet};

const AMBIGUOUS: &str =
    "Transcript gene coding lookup is ambiguous. Use an exact rsID or genomic HGVS ID.";
const EVIDENCE: &str = "Transcript gene coding lookup lacks complete identity evidence. Use an exact rsID or genomic HGVS ID.";
const INCOMPLETE: &str = "Transcript gene coding lookup did not complete its candidate scan. Use an exact rsID or genomic HGVS ID.";

// Source kind and original location remain attached to the complete matched assertion.
pub(in crate::entities::variant) enum TupleLocation {
    Rcv(usize),
    Snpeff(usize),
}
pub(in crate::entities::variant) struct MatchedTuple {
    pub location: TupleLocation,
    pub transcript: String,
    pub gene: String,
    pub coding: String,
    pub protein: Option<String>,
}
fn protein_valid(value: &str) -> bool {
    if !value.starts_with("p.") || value.len() > 256 {
        return false;
    }
    if crate::entities::variant::resolution::complete_source_protein_point(value) {
        return true;
    }
    let interval = parse_hgvs_protein_interval_21_1_4(value);
    interval
        .disposition()
        .parsed()
        .is_some_and(|parsed| parsed.reference().is_none())
        && interval.render_source() == value
}
fn rcv_tuple(value: &str, index: usize) -> Option<MatchedTuple> {
    if value.len() > 512 || value.trim() != value {
        return None;
    }
    let (coding, protein) = if let Some((coding, suffix)) = value.split_once(" (p.") {
        let protein = suffix.strip_suffix(')')?;
        let protein = format!("p.{protein}");
        if !protein_valid(&protein) {
            return None;
        }
        (coding, Some(protein))
    } else {
        (value, None)
    };
    // Source preferred names use exactly the colon wrapper, never a prefix extractor.
    let tuple = tuple(coding).ok()?;
    if coding != format!("{}({}):{}", tuple.transcript, tuple.gene, tuple.change) {
        return None;
    }
    Some(MatchedTuple {
        location: TupleLocation::Rcv(index),
        transcript: tuple.transcript.into(),
        gene: tuple.gene.into(),
        coding: tuple.change.into(),
        protein,
    })
}
fn snpeff_tuple(ann: &biodata::MyVariantSnpEffAnnotation, index: usize) -> Option<MatchedTuple> {
    let transcript = ann.feature_id()?;
    let gene = ann.genename()?;
    let coding = ann.hgvs_c()?;
    let coding = if let Some((prefix, change)) = coding.split_once(':') {
        if prefix != transcript {
            return None;
        }
        change
    } else {
        coding
    };
    if !versioned_transcript(transcript)
        || !crate::entities::variant::is_exact_gene_token(gene)
        || !coding_valid(coding)
        || ann.hgvs_p().is_some_and(|p| !protein_valid(p))
    {
        return None;
    }
    Some(MatchedTuple {
        location: TupleLocation::Snpeff(index),
        transcript: transcript.into(),
        gene: gene.into(),
        coding: coding.into(),
        protein: ann.hgvs_p().map(str::to_owned),
    })
}
fn matches(tuple: &MatchedTuple, transcript: &str, gene: Option<&str>, change: &str) -> bool {
    tuple.transcript == transcript
        && gene.is_none_or(|gene| tuple.gene == gene)
        && tuple.coding == change
}
fn matched_tuples(
    hit: &MyVariantHit,
    transcript: &str,
    gene: Option<&str>,
    change: &str,
) -> (Vec<MatchedTuple>, bool) {
    let mut tuples = Vec::new();
    let mut incomplete = hit.snpeff.as_ref().is_some_and(|s| !s.is_complete());
    let mut assertions = 0;
    if let Some(snpeff) = &hit.snpeff {
        for (index, ann) in snpeff.annotations().iter().enumerate() {
            assertions += 1;
            match snpeff_tuple(ann, index) {
                Some(tuple) if matches(&tuple, transcript, gene, change) => tuples.push(tuple),
                Some(_) => {}
                None => incomplete = true,
            }
        }
    }
    if let Some(clinvar) = &hit.clinvar {
        for (index, rcv) in clinvar.rcv.iter().enumerate() {
            assertions += 1;
            match rcv
                .preferred_name
                .as_deref()
                .and_then(|value| rcv_tuple(value, index))
            {
                Some(tuple) if matches(&tuple, transcript, gene, change) => tuples.push(tuple),
                Some(_) => {}
                None => incomplete = true,
            }
        }
    }
    (tuples, incomplete || assertions == 0)
}
pub(super) async fn lookup(
    client: &MyVariantClient,
    id: &str,
    requested: &TranscriptDeletion<'_>,
) -> Result<(MyVariantHit, MatchedTuple), BioMcpError> {
    lookup_source(
        client,
        id,
        requested.transcript,
        Some(requested.gene),
        requested.change,
        false,
    )
    .await
}

pub(super) async fn lookup_alias(
    client: &MyVariantClient,
    id: &str,
) -> Result<(MyVariantHit, MatchedTuple), BioMcpError> {
    let (transcript, change) = id
        .split_once(':')
        .ok_or_else(|| BioMcpError::InvalidArgument(EVIDENCE.into()))?;
    lookup_source(client, id, transcript, None, change, true).await
}

pub(super) fn alias_tuple(hit: &MyVariantHit, id: &str) -> Result<MatchedTuple, BioMcpError> {
    let (transcript, change) = id
        .split_once(':')
        .ok_or_else(|| BioMcpError::InvalidArgument(EVIDENCE.into()))?;
    let (tuples, incomplete) = matched_tuples(hit, transcript, None, change);
    let assertions: HashSet<_> = tuples
        .iter()
        .map(|tuple| (&tuple.gene, &tuple.protein))
        .collect();
    if incomplete || assertions.len() != 1 {
        return Err(BioMcpError::InvalidArgument(EVIDENCE.into()));
    }
    tuples
        .into_iter()
        .min_by_key(|tuple| match tuple.location {
            TupleLocation::Rcv(index) => (0, index),
            TupleLocation::Snpeff(index) => (1, index),
        })
        .ok_or_else(|| BioMcpError::InvalidArgument(EVIDENCE.into()))
}

async fn lookup_source(
    client: &MyVariantClient,
    id: &str,
    transcript: &str,
    gene: Option<&str>,
    change: &str,
    require_alias: bool,
) -> Result<(MyVariantHit, MatchedTuple), BioMcpError> {
    let query = query(&format!("{transcript}:{change}"));
    let mut selected = None;
    let mut seen = HashSet::new();
    let mut proteins: HashMap<String, (String, Option<String>)> = HashMap::new();
    let mut indeterminate = false;
    coding_lookup::scan(client, &query, INCOMPLETE, |hit| {
        if require_alias {
            match hit
                .clinvar
                .as_ref()
                .and_then(|clinvar| clinvar.hgvs.as_ref())
            {
                Some(hgvs) if hgvs.coding_contains(id) => {}
                Some(_) => return,
                None => {
                    indeterminate = true;
                    return;
                }
            }
        }
        let (tuples, incomplete) = matched_tuples(&hit, transcript, gene, change);
        indeterminate |= incomplete || hit.id.trim().is_empty();
        if tuples.is_empty() || hit.id.trim().is_empty() {
            return;
        }
        for tuple in &tuples {
            if let Some(prior) = proteins.get(&hit.id) {
                indeterminate |= prior != &(tuple.gene.clone(), tuple.protein.clone());
            } else {
                proteins.insert(hit.id.clone(), (tuple.gene.clone(), tuple.protein.clone()));
            }
        }
        let source = SourceVariantIdentity::from_myvariant_hit(&hit);
        let mut protein = source.protein_changes.clone();
        let mut coding = source.coding_changes.clone();
        protein.sort();
        coding.sort();
        if seen.insert((hit.id.clone(), source.normalized_key(), protein, coding))
            && selected.is_none()
        {
            // RCV and snpEff are complete assertions. Do not combine their fields.
            let tuple = tuples.into_iter().min_by_key(|tuple| match tuple.location {
                TupleLocation::Rcv(index) => (0, index),
                TupleLocation::Snpeff(index) => (1, index),
            });
            selected = tuple.map(|tuple| (hit, tuple));
        }
    })
    .await?;
    if indeterminate {
        return Err(BioMcpError::InvalidArgument(EVIDENCE.into()));
    }
    if seen.len() > 1 {
        return Err(BioMcpError::InvalidArgument(AMBIGUOUS.into()));
    }
    selected.ok_or_else(|| BioMcpError::NotFound {
        entity: "variant".into(),
        id: id.into(),
        suggestion: "Try searching: biomcp search variant".into(),
    })
}

pub(super) fn query(id: &str) -> String {
    format!(
        "clinvar.hgvs.coding:\"{}\"",
        MyVariantClient::escape_query_value(id)
    )
}

pub(super) async fn resolve(
    id: &str,
    genome_build: Option<crate::entities::variant::GenomeBuild>,
) -> Result<
    Option<(
        crate::entities::variant::Variant,
        crate::entities::variant::VariantIdFormat,
        MyVariantHit,
    )>,
    BioMcpError,
> {
    use crate::entities::variant::GenomeBuild;
    let Some(requested) = prepare(id)? else {
        return Ok(None);
    };
    if genome_build.is_some() {
        return Err(BioMcpError::InvalidArgument(
            "--assembly is only supported for genomic variant IDs".into(),
        ));
    }
    let (hit, tuple) = lookup(&MyVariantClient::new()?, id.trim(), &requested).await?;
    let mut variant = crate::transform::variant::from_myvariant_hit_with_tuple(
        &hit,
        &tuple.gene,
        &tuple.transcript,
        &tuple.coding,
        tuple.protein.as_deref(),
    );
    variant.genome_build = Some(GenomeBuild::Grch37);
    variant.genome_build_provenance = Some("MyVariant.info provider default".into());
    Ok(Some((variant, requested.format(), hit)))
}
#[cfg(test)]
#[path = "transcript_deletion_lookup_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "transcript_deletion_transport_tests.rs"]
mod transport_tests;
