//! Variant detail retrieval, section gating, and enrichment orchestration.

use std::time::Duration;

use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
#[cfg(feature = "alphagenome")]
use crate::sources::alphagenome::AlphaGenomeClient;
use crate::sources::cancerhotspots::CancerHotspotsClient;
use crate::sources::cbioportal::CBioPortalClient;
use crate::sources::civic::CivicClient;
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
    GenomeBuild, TreatmentImplication, Variant, VariantCivicSection, VariantIdFormat,
    VariantInputKind, VariantNormalizationResponse, VariantNormalizationStatus,
    VariantOncoKbResult, classify_variant_input, normalize_variant,
};

const VARIANT_SECTION_PREDICT: &str = "predict";
const VARIANT_SECTION_PREDICTIONS: &str = "predictions";
const VARIANT_SECTION_CLINVAR: &str = "clinvar";
const VARIANT_SECTION_POPULATION: &str = "population";
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
    VARIANT_SECTION_CONSERVATION,
    VARIANT_SECTION_COSMIC,
    VARIANT_SECTION_CGI,
    VARIANT_SECTION_CIVIC,
    VARIANT_SECTION_CBIOPORTAL,
    VARIANT_SECTION_GWAS,
    VARIANT_SECTION_ALL,
];

const OPTIONAL_ENRICHMENT_TIMEOUT: Duration = Duration::from_secs(8);

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
            VARIANT_SECTION_POPULATION => out.include_population = true,
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
            normalize_transcript_hgvs_for_get(id).await?
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
    let (hit, answering_build, build_candidates) = match &id_format {
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
                        (preferred_hit, Some(preferred), candidates)
                    }
                    (Ok(hit), Err(error)) if error.is_not_found() => {
                        (hit, Some(preferred), Vec::new())
                    }
                    (Err(error), Ok(hit)) if error.is_not_found() => (hit, Some(other), Vec::new()),
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
                    let q = transcript_hgvs_clinvar_query(id);
                    let resp = myvariant
                        .query_with_fields(
                            &q,
                            10,
                            0,
                            crate::sources::myvariant::MYVARIANT_FIELDS_GET,
                        )
                        .await?;
                    let compatible_hits = resp
                        .hits
                        .into_iter()
                        .filter(&compatible)
                        .collect::<Vec<_>>();
                    (
                        best_hit(&compatible_hits).cloned().ok_or_else(|| {
                            BioMcpError::NotFound {
                                entity: "variant".into(),
                                id: id.to_string(),
                                suggestion: format!("Try first: biomcp variant normalize all {id}"),
                            }
                        })?,
                        effective_build.or(Some(GenomeBuild::Grch37)),
                        Vec::new(),
                    )
                } else {
                    let hit = direct.map_err(|error| match effective_build {
                        Some(build) => build_aware_not_found(hgvs, build, error),
                        None => error,
                    })?;
                    if !compatible(&hit) {
                        return Err(BioMcpError::NotFound {
                            entity: "variant".into(),
                            id: id.to_string(),
                            suggestion: format!("Try searching: biomcp search variant -g \"{id}\""),
                        });
                    }
                    (hit, effective_build, Vec::new())
                }
            }
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
            (
                resp.hits
                    .into_iter()
                    .find(&compatible)
                    .ok_or_else(|| BioMcpError::NotFound {
                        entity: "variant".into(),
                        id: id.to_string(),
                        suggestion: format!(
                            "Try searching: biomcp search variant -g {gene} --hgvsp {change}"
                        ),
                    })?,
                Some(GenomeBuild::Grch37),
                Vec::new(),
            )
        }
    };

    let mut variant = transform::variant::from_myvariant_hit(&hit);
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
        rsid: Some(rsid.to_string()),
        cosmic_id: None,
        significance: None,
        clinvar_id: None,
        clinvar_review_status: None,
        clinvar_review_stars: None,
        conditions: Vec::new(),
        gnomad_af: None,
        allele_frequency_raw: None,
        allele_frequency_percent: None,
        consequence: None,
        cadd_score: None,
        sift_pred: None,
        polyphen_pred: None,
        conservation: None,
        expanded_predictions: Vec::new(),
        population_breakdown: None,
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

fn strip_clinvar_details(variant: &mut Variant) {
    variant.conditions.clear();
    variant.clinvar_conditions.clear();
    variant.clinvar_condition_reports = None;
    variant.top_disease = None;
    variant.clinvar_id = None;
    variant.clinvar_review_status = None;
    variant.clinvar_review_stars = None;
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

    let (mut variant, id_format) = resolve_base(id, genome_build).await?;
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
        variant.population_breakdown = None;
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
mod tests;
