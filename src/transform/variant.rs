use std::collections::HashMap;

#[cfg(feature = "alphagenome")]
use crate::entities::variant::VariantPrediction;
use crate::entities::variant::{
    ConditionReportCount, Variant, VariantCgiAssociation, VariantCivicSection,
    VariantConservationScores, VariantCosmicContext, VariantPredictionScore, VariantSearchResult,
    normalize_protein_change,
};
use crate::sources::cbioportal::CBioMutationSummary;
use crate::sources::civic::CivicEvidenceItem;
use crate::sources::myvariant::{FloatOrVec, MyVariantClinVarRcv, MyVariantGnomadAf, MyVariantHit};
use crate::utils::serde::StringOrVec;

fn normalize_gene(gene: &str) -> Option<String> {
    let g = gene.trim();
    if g.is_empty() {
        return None;
    }
    Some(g.to_uppercase())
}

fn pick_gene(dbnsfp: &crate::sources::myvariant::MyVariantDbnsfp) -> String {
    dbnsfp
        .genename
        .first()
        .and_then(normalize_gene)
        .unwrap_or_default()
}

#[derive(Default)]
struct TranscriptAnnotation {
    gene: Option<String>,
    transcript: Option<String>,
    coding: Option<String>,
    protein: Option<String>,
}

fn accession_stem(value: &str) -> &str {
    value.trim().split('.').next().unwrap_or(value.trim())
}

fn clinvar_preferred_annotation(hit: &MyVariantHit) -> Option<TranscriptAnnotation> {
    hit.clinvar.as_ref()?.rcv.iter().find_map(|rcv| {
        let name = rcv.preferred_name.as_deref()?.trim();
        let (left, rest) = name.split_once(":c.")?;
        let transcript = left.split_once('(').map_or(left, |(value, _)| value).trim();
        if !transcript.starts_with("NM_") {
            return None;
        }
        let gene = left
            .split_once('(')
            .and_then(|(_, value)| value.strip_suffix(')'))
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let coding_tail = format!("c.{rest}");
        let coding = coding_tail
            .split_whitespace()
            .next()
            .map(str::to_string)
            .filter(|value| value.len() > 2);
        let protein = name
            .rfind("(p.")
            .and_then(|start| name[start + 1..].strip_suffix(')'))
            .map(str::to_string);
        Some(TranscriptAnnotation {
            gene: gene.map(str::to_string),
            transcript: Some(transcript.to_string()),
            coding,
            protein,
        })
    })
}

/// The MANE marker this data carries: the versioned accession ClinVar's
/// preferred names agree on. MyVariant.info's SnpEff and dbNSFP sections
/// ship no MANE status of their own, but ClinVar names each variant on
/// the gene's MANE Select transcript when one exists, so the transcript
/// stem ClinVar's preferred names agree on marks the MANE transcript for
/// the gene, and the accession's version is the current RefSeq one where
/// the SnpEff build lags it (NM_000546.5 against NM_000546.6, ticket 2035
/// finding 21). A cohort whose ClinVar-preferred stems disagree carries no
/// marker (ticket 2016).
pub(crate) fn clinvar_mane_transcript(hits: &[MyVariantHit]) -> Option<String> {
    let mut marker: Option<(String, String)> = None;
    for hit in hits {
        let Some(annotation) = clinvar_preferred_annotation(hit) else {
            continue;
        };
        let Some(transcript) = annotation.transcript else {
            continue;
        };
        let stem = accession_stem(&transcript).to_string();
        match &marker {
            Some((known, _)) if *known != stem => return None,
            _ => marker = Some((stem, transcript)),
        }
    }
    marker.map(|(_, transcript)| transcript)
}

/// Rank the SnpEff annotations: the hit's own ClinVar-preferred transcript
/// first (ClinVar names on MANE Select), then the gene's MANE transcript
/// when the query response marks one, then the annotation that spells the
/// requested protein change (a response with no ClinVar name anywhere
/// still names the request on the transcript it follows — the only MANE
/// signal a ClinVar-free response carries, ticket 2035 finding 4), then
/// the first NM_ transcript (yesterday's rule), then anything else.
pub(crate) fn selected_snpeff_annotation_index(
    hit: &MyVariantHit,
    mane_transcript: Option<&str>,
    requested_change: Option<&str>,
) -> Option<usize> {
    let clinvar = clinvar_preferred_annotation(hit);
    let preferred_stem = clinvar
        .as_ref()
        .and_then(|value| value.transcript.as_deref())
        .map(accession_stem);
    let mane_stem = mane_transcript.map(accession_stem);
    let annotations = &hit.snpeff.as_ref()?.ann;
    annotations
        .iter()
        .enumerate()
        .filter(|ann| {
            ann.1
                .hgvs_c
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
        })
        .min_by_key(|ann| {
            let feature = ann.1.feature_id.as_deref().unwrap_or_default();
            let feature_stem = accession_stem(feature);
            if preferred_stem.is_some_and(|preferred| feature_stem == preferred) {
                0
            } else if mane_stem.is_some_and(|mane| feature_stem == mane) {
                1
            } else if requested_change.is_some_and(|change| {
                ann.1
                    .hgvs_p
                    .as_deref()
                    .and_then(crate::entities::variant::normalize_protein_change)
                    .is_some_and(|protein| {
                        crate::entities::variant::protein_changes_equivalent(change, &protein)
                    })
            }) {
                2
            } else if feature.starts_with("NM_") {
                3
            } else {
                4
            }
        })
        .map(|(index, _)| index)
}

fn select_transcript_annotation(
    hit: &MyVariantHit,
    mane_transcript: Option<&str>,
    requested_change: Option<&str>,
) -> Option<TranscriptAnnotation> {
    let selected = hit
        .snpeff
        .as_ref()?
        .ann
        .get(selected_snpeff_annotation_index(
            hit,
            mane_transcript,
            requested_change,
        )?)?;
    let feature = selected.feature_id.clone()?;
    // ClinVar's preferred names carry the current RefSeq accession for the
    // stem while the SnpEff build lags it (ticket 2035 finding 21), so the
    // headline shows the current accession; the coding and protein
    // spellings stay the selected annotation's own.
    let transcript = clinvar_preferred_annotation(hit)
        .and_then(|annotation| annotation.transcript)
        .filter(|preferred| accession_stem(preferred) == accession_stem(&feature))
        .or_else(|| {
            mane_transcript
                .filter(|mane| accession_stem(mane) == accession_stem(&feature))
                .map(str::to_string)
        })
        .unwrap_or(feature);
    Some(TranscriptAnnotation {
        gene: selected.genename.clone(),
        transcript: Some(transcript),
        coding: selected.hgvs_c.clone(),
        protein: selected.hgvs_p.clone(),
    })
}

fn paired_annotation(
    hit: &MyVariantHit,
    mane_transcript: Option<&str>,
    requested_change: Option<&str>,
) -> Option<TranscriptAnnotation> {
    select_transcript_annotation(hit, mane_transcript, requested_change)
        .or_else(|| clinvar_preferred_annotation(hit))
}

/// The protein change on the transcript BioMCP headlines for a hit: the
/// ClinVar-named, MANE, change-naming, or first-NM_ SnpEff annotation.
/// dbNSFP's merged alias list (`dbnsfp.hgvsp`) carries other isoforms'
/// spellings on other genomic variants, so only this value confirms a
/// gene+protein query names the hit (ticket 2016).
pub(crate) fn canonical_protein_change(
    hit: &MyVariantHit,
    mane_transcript: Option<&str>,
    requested_change: Option<&str>,
) -> Option<String> {
    paired_annotation(hit, mane_transcript, requested_change)
        .and_then(|annotation| annotation.protein)
}

/// The transcript whose numbering the headline protein change uses (ticket
/// 2016's numbering note).
pub(crate) fn canonical_transcript(
    hit: &MyVariantHit,
    mane_transcript: Option<&str>,
    requested_change: Option<&str>,
) -> Option<String> {
    paired_annotation(hit, mane_transcript, requested_change)
        .and_then(|annotation| annotation.transcript)
}

/// True when any SnpEff annotation on the given transcript stem — or the
/// hit's ClinVar-preferred annotation when it names that stem — spells the
/// requested change. The headline annotation picks one isoform; this says
/// whether the MANE transcript itself names the request, so the numbering
/// rules never refuse a hit that does name it (ticket 2033 finding 3).
pub(crate) fn mane_annotation_names_change(
    hit: &MyVariantHit,
    change: &str,
    mane_transcript: Option<&str>,
) -> bool {
    let Some(mane_stem) = mane_transcript.map(accession_stem) else {
        return false;
    };
    let snpeff_names = hit.snpeff.as_ref().is_some_and(|snpeff| {
        snpeff.ann.iter().any(|ann| {
            ann.feature_id
                .as_deref()
                .is_some_and(|feature| accession_stem(feature) == mane_stem)
                && ann
                    .hgvs_p
                    .as_deref()
                    .and_then(crate::entities::variant::normalize_protein_change)
                    .is_some_and(|protein| {
                        crate::entities::variant::protein_changes_equivalent(change, &protein)
                    })
        })
    });
    snpeff_names
        || clinvar_preferred_annotation(hit).is_some_and(|annotation| {
            annotation
                .transcript
                .as_deref()
                .is_some_and(|transcript| accession_stem(transcript) == mane_stem)
                && annotation
                    .protein
                    .as_deref()
                    .and_then(crate::entities::variant::normalize_protein_change)
                    .is_some_and(|protein| {
                        crate::entities::variant::protein_changes_equivalent(change, &protein)
                    })
        })
}

/// The record's annotation that spells the requested change on a transcript
/// other than the MANE one — the isoform a refusal must name so the reader
/// sees where the request is actually spelled (tickets 2042 and 2047).
/// Protein-structure rows (`feature_type: interaction`) carry no transcript
/// of their own, so only transcript-shaped features answer. Returns the
/// transcript and its spelling of the request.
pub(crate) fn isoform_annotation_naming_change(
    hit: &MyVariantHit,
    change: &str,
    mane_transcript: Option<&str>,
) -> Option<(String, String)> {
    let mane_stem = mane_transcript.map(accession_stem);
    hit.snpeff.as_ref()?.ann.iter().find_map(|ann| {
        let feature = ann.feature_id.as_deref()?.trim();
        if feature.is_empty() || !looks_like_transcript(feature) {
            return None;
        }
        if mane_stem.is_some_and(|mane| accession_stem(feature) == mane) {
            return None;
        }
        let raw = ann
            .hgvs_p
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())?;
        let normalized = crate::entities::variant::normalize_protein_change(raw)?;
        crate::entities::variant::protein_changes_equivalent(change, &normalized)
            .then(|| (feature.to_string(), raw.to_string()))
    })
}

/// A SnpEff feature that names a transcript: RefSeq accessions (`NM_`,
/// `NR_`, `XM_`) and Ensembl transcripts (`ENST`). Protein-structure
/// features such as `1JNX:X_1820-X_1857:NM_007294.3` do not count.
fn looks_like_transcript(feature: &str) -> bool {
    let stem = accession_stem(feature);
    stem.starts_with("NM_")
        || stem.starts_with("NR_")
        || stem.starts_with("XM_")
        || stem.starts_with("ENST")
}

fn legacy_name(gene: &str, protein: Option<&str>) -> Option<String> {
    let gene = gene.trim();
    let normalized = normalize_protein_change(protein?)?;
    if gene.is_empty() {
        return None;
    }
    let legacy = normalized
        .strip_suffix('*')
        .map(|prefix| format!("{prefix}stop"))
        .unwrap_or(normalized);
    Some(format!("{gene} {legacy}"))
}

fn normalize_consequence(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "non_synonymous" | "nonsynonymous" | "non-synonymous" => "missense_variant".into(),
        "synonymous" => "synonymous_variant".into(),
        other => other.replace(' ', "_"),
    }
}

fn pick_consequence(hit: &MyVariantHit) -> Option<String> {
    hit.cadd
        .as_ref()
        .and_then(|c| c.consequence.as_ref())
        .and_then(StringOrVec::first)
        .map(normalize_consequence)
        .filter(|v| !v.is_empty())
}

fn best_gnomad_af(hit: &MyVariantHit) -> Option<&MyVariantGnomadAf> {
    hit.gnomad_exome
        .as_ref()
        .and_then(|v| v.af.as_ref())
        .or_else(|| {
            hit.gnomad
                .as_ref()
                .and_then(|g| g.exomes.as_ref())
                .and_then(|v| v.af.as_ref())
        })
        .or_else(|| {
            hit.gnomad
                .as_ref()
                .and_then(|g| g.genomes.as_ref())
                .and_then(|v| v.af.as_ref())
        })
}

fn first_score(value: Option<&FloatOrVec>) -> Option<f64> {
    value.and_then(FloatOrVec::first)
}

fn first_nonempty(values: &StringOrVec) -> Option<String> {
    values
        .first()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn extract_conservation(hit: &MyVariantHit) -> Option<VariantConservationScores> {
    let dbnsfp = hit.dbnsfp.as_ref()?;

    let scores = VariantConservationScores {
        phylop_100way_vertebrate: dbnsfp
            .phylop
            .as_ref()
            .and_then(|p| p.way_100_vertebrate.as_ref())
            .and_then(|v| first_score(v.rankscore.as_ref())),
        phylop_470way_mammalian: dbnsfp
            .phylop
            .as_ref()
            .and_then(|p| p.way_470_mammalian.as_ref())
            .and_then(|v| first_score(v.rankscore.as_ref())),
        phastcons_100way_vertebrate: dbnsfp
            .phastcons
            .as_ref()
            .and_then(|p| p.way_100_vertebrate.as_ref())
            .and_then(|v| first_score(v.rankscore.as_ref())),
        phastcons_470way_mammalian: dbnsfp
            .phastcons
            .as_ref()
            .and_then(|p| p.way_470_mammalian.as_ref())
            .and_then(|v| first_score(v.rankscore.as_ref())),
        gerp_rs: dbnsfp
            .gerp
            .as_ref()
            .and_then(|g| first_score(g.rs.as_ref())),
    };

    if scores.phylop_100way_vertebrate.is_none()
        && scores.phylop_470way_mammalian.is_none()
        && scores.phastcons_100way_vertebrate.is_none()
        && scores.phastcons_470way_mammalian.is_none()
        && scores.gerp_rs.is_none()
    {
        None
    } else {
        Some(scores)
    }
}

fn normalize_prediction(pred: Option<String>, tool: &str) -> Option<String> {
    let pred = pred
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)?;

    let lower = pred.to_ascii_lowercase();
    if tool.eq_ignore_ascii_case("alphamissense") {
        if pred.eq_ignore_ascii_case("p") || lower.contains("pathogenic") {
            return Some("Pathogenic".to_string());
        }
        if pred.eq_ignore_ascii_case("b") || lower.contains("benign") {
            return Some("Benign".to_string());
        }
    }

    Some(pred)
}

fn push_prediction(
    out: &mut Vec<VariantPredictionScore>,
    tool: &str,
    score: Option<f64>,
    prediction: Option<String>,
) {
    if score.is_none() && prediction.is_none() {
        return;
    }
    out.push(VariantPredictionScore {
        tool: tool.to_string(),
        score,
        prediction,
    });
}

fn extract_expanded_predictions(hit: &MyVariantHit) -> Vec<VariantPredictionScore> {
    let Some(dbnsfp) = hit.dbnsfp.as_ref() else {
        return Vec::new();
    };

    let mut out: Vec<VariantPredictionScore> = Vec::new();
    push_prediction(
        &mut out,
        "REVEL",
        dbnsfp
            .revel
            .as_ref()
            .and_then(|v| first_score(v.score.as_ref())),
        None,
    );
    push_prediction(
        &mut out,
        "AlphaMissense",
        dbnsfp
            .alphamissense
            .as_ref()
            .and_then(|v| first_score(v.score.as_ref())),
        normalize_prediction(
            dbnsfp
                .alphamissense
                .as_ref()
                .and_then(|v| v.pred.as_ref())
                .and_then(first_nonempty),
            "alphamissense",
        ),
    );
    push_prediction(
        &mut out,
        "ClinPred",
        dbnsfp
            .clinpred
            .as_ref()
            .and_then(|v| first_score(v.score.as_ref())),
        normalize_prediction(
            dbnsfp
                .clinpred
                .as_ref()
                .and_then(|v| v.pred.as_ref())
                .and_then(first_nonempty),
            "clinpred",
        ),
    );
    push_prediction(
        &mut out,
        "SIFT",
        dbnsfp
            .sift
            .as_ref()
            .and_then(|v| first_score(v.score.as_ref())),
        dbnsfp
            .sift
            .as_ref()
            .and_then(|v| v.pred.as_ref())
            .and_then(StringOrVec::first)
            .map(normalize_sift),
    );
    push_prediction(
        &mut out,
        "MetaRNN",
        dbnsfp
            .metarnn
            .as_ref()
            .and_then(|v| first_score(v.score.as_ref())),
        normalize_prediction(
            dbnsfp
                .metarnn
                .as_ref()
                .and_then(|v| v.pred.as_ref())
                .and_then(first_nonempty),
            "metarnn",
        ),
    );
    push_prediction(
        &mut out,
        "BayesDel add-AF",
        dbnsfp
            .bayesdel
            .as_ref()
            .and_then(|v| v.add_af.as_ref())
            .and_then(|v| first_score(v.score.as_ref())),
        normalize_prediction(
            dbnsfp
                .bayesdel
                .as_ref()
                .and_then(|v| v.add_af.as_ref())
                .and_then(|v| v.pred.as_ref())
                .and_then(first_nonempty),
            "bayesdel_add_af",
        ),
    );
    push_prediction(
        &mut out,
        "BayesDel no-AF",
        dbnsfp
            .bayesdel
            .as_ref()
            .and_then(|v| v.no_af.as_ref())
            .and_then(|v| first_score(v.score.as_ref())),
        normalize_prediction(
            dbnsfp
                .bayesdel
                .as_ref()
                .and_then(|v| v.no_af.as_ref())
                .and_then(|v| v.pred.as_ref())
                .and_then(first_nonempty),
            "bayesdel_no_af",
        ),
    );

    out
}

fn extract_cosmic_details(hit: &MyVariantHit) -> Option<VariantCosmicContext> {
    let cosmic = hit.cosmic.as_ref()?;

    let context = VariantCosmicContext {
        mut_freq: cosmic.mut_freq,
        tumor_site: first_nonempty(&cosmic.tumor_site),
        mut_nt: first_nonempty(&cosmic.mut_nt),
    };

    if context.mut_freq.is_none() && context.tumor_site.is_none() && context.mut_nt.is_none() {
        None
    } else {
        Some(context)
    }
}

fn value_first_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.trim().to_string()).filter(|v| !v.is_empty()),
        serde_json::Value::Array(arr) => arr.iter().find_map(value_first_string),
        serde_json::Value::Object(obj) => obj.values().find_map(value_first_string),
        _ => None,
    }
}

fn extract_cgi_associations(hit: &MyVariantHit) -> Vec<VariantCgiAssociation> {
    let Some(cgi) = hit.cgi.as_ref() else {
        return Vec::new();
    };

    let rows: Vec<&serde_json::Value> = match cgi {
        serde_json::Value::Array(arr) => arr.iter().collect(),
        serde_json::Value::Object(_) => vec![cgi],
        _ => Vec::new(),
    };

    let mut out: Vec<VariantCgiAssociation> = Vec::new();
    for row in rows {
        let Some(obj) = row.as_object() else { continue };
        let Some(drug) = obj.get("drug").and_then(value_first_string) else {
            continue;
        };

        let association = obj.get("association").and_then(value_first_string);
        let tumor_type = obj
            .get("primary_tumor_type")
            .or_else(|| obj.get("tumor_type"))
            .and_then(value_first_string);
        let evidence_level = obj
            .get("evidence_level")
            .or_else(|| obj.get("evidence"))
            .and_then(value_first_string);
        let source = obj.get("source").and_then(value_first_string);

        out.push(VariantCgiAssociation {
            drug,
            association,
            tumor_type,
            evidence_level,
            source,
        });
        if out.len() >= 10 {
            break;
        }
    }

    out
}

fn extract_civic_cached_evidence(hit: &MyVariantHit) -> Vec<CivicEvidenceItem> {
    let Some(civic) = hit.civic.as_ref() else {
        return Vec::new();
    };

    let Some(molecular_profiles) = civic
        .get("molecularProfiles")
        .and_then(serde_json::Value::as_array)
    else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for profile in molecular_profiles {
        let profile_name = profile
            .get("name")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or_default()
            .to_string();
        if profile_name.is_empty() {
            continue;
        }

        let Some(items) = profile
            .get("evidenceItems")
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };

        for row in items {
            let id = row
                .get("id")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            let name = row
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("cached")
                .to_string();
            let evidence_type = row
                .get("evidenceType")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("-")
                .to_string();
            let evidence_level = row
                .get("evidenceLevel")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("-")
                .to_string();
            let significance = row
                .get("significance")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("-")
                .to_string();
            let disease = row
                .get("disease")
                .and_then(|v| v.get("displayName").or_else(|| v.get("name")))
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string);
            let therapies = row
                .get("therapies")
                .and_then(serde_json::Value::as_array)
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(|entry| {
                            entry
                                .get("name")
                                .and_then(serde_json::Value::as_str)
                                .map(str::trim)
                                .filter(|v| !v.is_empty())
                                .map(str::to_string)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let status = row
                .get("status")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("-")
                .to_string();

            out.push(CivicEvidenceItem {
                id,
                name,
                molecular_profile: profile_name.clone(),
                evidence_type,
                evidence_level,
                significance,
                disease,
                therapies,
                status,
                citation: None,
                source_type: None,
                publication_year: None,
            });
            if out.len() >= 20 {
                return out;
            }
        }
    }

    out
}

fn dedupe_limit(values: Vec<String>, max: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for v in values {
        let v = v.trim();
        if v.is_empty() {
            continue;
        }
        if out.iter().any(|x| x.eq_ignore_ascii_case(v)) {
            continue;
        }
        out.push(v.to_string());
        if out.len() >= max {
            break;
        }
    }
    out
}

fn clinvar_condition_names(rcv: &MyVariantClinVarRcv) -> Vec<String> {
    let Some(v) = rcv.conditions.as_ref() else {
        return vec![];
    };

    let mut names: Vec<String> = Vec::new();
    match v {
        serde_json::Value::Object(obj) => {
            if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                names.push(name.to_string());
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(name) = item.as_str() {
                    names.push(name.to_string());
                    continue;
                }
                if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                    names.push(name.to_string());
                }
            }
        }
        serde_json::Value::String(s) => names.push(s.to_string()),
        _ => {}
    }
    names
}

fn aggregate_clinvar_conditions(
    rcvs: &[MyVariantClinVarRcv],
) -> (Vec<String>, Vec<ConditionReportCount>, Option<u32>) {
    let mut counts: HashMap<String, (String, u32)> = HashMap::new();

    for rcv in rcvs {
        for name in clinvar_condition_names(rcv) {
            let cleaned = name.trim();
            if cleaned.is_empty() {
                continue;
            }
            let key = cleaned.to_ascii_lowercase();
            let entry = counts
                .entry(key)
                .or_insert_with(|| (cleaned.to_string(), 0u32));
            entry.1 += 1;
        }
    }

    if counts.is_empty() {
        return (Vec::new(), Vec::new(), None);
    }

    let mut rows = counts
        .into_values()
        .map(|(condition, reports)| ConditionReportCount { condition, reports })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.reports
            .cmp(&a.reports)
            .then_with(|| a.condition.cmp(&b.condition))
    });

    let total_reports = rows.iter().map(|v| v.reports).sum::<u32>();
    let names = rows.iter().map(|v| v.condition.clone()).collect::<Vec<_>>();

    (dedupe_limit(names, 8), rows, Some(total_reports))
}

fn significance_rank(value: &str) -> i32 {
    let v = value.trim().to_ascii_lowercase();
    if v.contains("pathogenic") && !v.contains("likely") {
        return 5;
    }
    if v.contains("likely pathogenic") {
        return 4;
    }
    if v.contains("uncertain") || v.contains("vus") {
        return 3;
    }
    if v.contains("likely benign") {
        return 2;
    }
    if v.contains("benign") {
        return 1;
    }
    0
}

fn pick_significance(rcvs: &[MyVariantClinVarRcv]) -> Option<String> {
    let mut best: Option<(&str, i32)> = None;
    for r in rcvs {
        let Some(sig) = r.clinical_significance.as_deref() else {
            continue;
        };
        let rank = significance_rank(sig);
        if best.is_none_or(|b| rank > b.1) {
            best = Some((sig, rank));
        }
    }
    best.map(|(s, _)| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Newest `last_evaluated` day among the records carrying the shown
/// classification (ticket 2022), so the headline date belongs to the printed
/// term instead of a newer, differently classified record. Only strict
/// day-shaped dates surface, under the one shared day-shape rule,
/// `crate::utils::date::is_day_shaped`, which the ClinVar fallback age
/// label (ticket 1291) reads too.
fn newest_rcv_evaluation_date(
    rcvs: &[MyVariantClinVarRcv],
    significance: Option<&str>,
) -> Option<String> {
    let significance = significance?.trim();
    if significance.is_empty() {
        return None;
    }
    rcvs.iter()
        .filter(|rcv| {
            rcv.clinical_significance
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| value.eq_ignore_ascii_case(significance))
        })
        .filter_map(|rcv| rcv.last_evaluated.as_deref().map(str::trim))
        .filter(|value| crate::utils::date::is_day_shaped(value))
        .max()
        .map(str::to_string)
}

fn derived_significance_note(variant_id: &str) -> String {
    format!(
        "Most severe RCV classification in MyVariant.info's cached ClinVar copy; run \
         `biomcp get variant \"{variant_id}\" clinvar` for the current NCBI ClinVar \
         record-level classification."
    )
}

pub(crate) fn clinvar_review_stars(review_status: &str) -> Option<u8> {
    let v = review_status.trim().to_ascii_lowercase();
    if v.is_empty() {
        return None;
    }
    if v.contains("practice guideline") {
        return Some(4);
    }
    if v.contains("reviewed by expert panel") {
        return Some(3);
    }
    if v.contains("multiple submitters") && v.contains("no conflicts") {
        return Some(2);
    }
    if v.contains("single submitter") || v.contains("conflicting interpretations") {
        return Some(1);
    }
    if v.contains("no assertion") {
        return Some(0);
    }
    None
}

fn pick_review_status(rcvs: &[MyVariantClinVarRcv]) -> (Option<String>, Option<u8>) {
    let mut best: Option<(u8, &str)> = None;
    let mut fallback_status: Option<&str> = None;

    for r in rcvs {
        let Some(status) = r.review_status.as_deref().map(str::trim) else {
            continue;
        };
        if status.is_empty() {
            continue;
        }

        if fallback_status.is_none() {
            fallback_status = Some(status);
        }

        let Some(stars) = clinvar_review_stars(status) else {
            continue;
        };

        if best.is_none_or(|b| stars > b.0) {
            best = Some((stars, status));
        }
    }

    if let Some((stars, status)) = best {
        (Some(status.to_string()), Some(stars))
    } else {
        (fallback_status.map(|s| s.to_string()), None)
    }
}

fn normalize_sift(pred: &str) -> String {
    match pred.trim() {
        "D" | "d" => "Deleterious".into(),
        "T" | "t" => "Tolerated".into(),
        other => other.to_string(),
    }
}

fn normalize_polyphen(pred: &str) -> String {
    match pred.trim() {
        "D" | "d" => "Probably damaging".into(),
        "P" | "p" => "Possibly damaging".into(),
        "B" | "b" => "Benign".into(),
        other => other.to_string(),
    }
}

pub(crate) fn normalize_oncokb_level(value: &str) -> String {
    let v = value.trim();
    if let Some(rest) = v.strip_prefix("LEVEL_") {
        return format!("Level {rest}");
    }
    v.to_string()
}

pub fn from_myvariant_hit(hit: &MyVariantHit) -> Variant {
    from_myvariant_hit_with_mane(hit, None, None)
}

/// Same as [`from_myvariant_hit`], with the gene's MANE transcript (the
/// marker ClinVar's preferred names carry) so a hit without its own
/// ClinVar record still headlines the MANE transcript when the query
/// response marks one, and with the requested protein change so a
/// ClinVar-free response still headlines the annotation that names the
/// request (ticket 2016; ticket 2035 finding 4).
pub(crate) fn from_myvariant_hit_with_mane(
    hit: &MyVariantHit,
    mane_transcript: Option<&str>,
    requested_change: Option<&str>,
) -> Variant {
    let mut gene = String::new();
    let mut hgvs_p: Option<String> = None;
    let mut hgvs_c: Option<String> = None;
    let mut sift_pred: Option<String> = None;
    let mut polyphen_pred: Option<String> = None;

    let annotation = paired_annotation(hit, mane_transcript, requested_change);
    if let Some(dbnsfp) = hit.dbnsfp.as_ref() {
        gene = pick_gene(dbnsfp);

        sift_pred = dbnsfp
            .sift
            .as_ref()
            .and_then(|s| s.pred.as_ref())
            .and_then(StringOrVec::first)
            .map(normalize_sift)
            .filter(|s| !s.is_empty());

        polyphen_pred = dbnsfp
            .polyphen2
            .as_ref()
            .and_then(|p| p.hdiv.as_ref())
            .and_then(|h| h.pred.as_ref())
            .and_then(StringOrVec::first)
            .map(normalize_polyphen)
            .filter(|s| !s.is_empty());
    }

    if gene.is_empty() {
        gene = hit
            .clinvar
            .as_ref()
            .and_then(|clinvar| clinvar.gene.as_ref())
            .and_then(|gene| gene.symbol.as_deref())
            .unwrap_or_default()
            .to_string();
    }

    if let Some(annotation) = annotation.as_ref() {
        if let Some(annotation_gene) = annotation.gene.as_deref().and_then(normalize_gene) {
            gene = annotation_gene;
        }
        hgvs_p = annotation.protein.clone();
        hgvs_c = annotation.coding.clone();
    }

    let legacy_name = legacy_name(&gene, hgvs_p.as_deref());

    let rsid = hit
        .dbsnp
        .as_ref()
        .and_then(|d| d.rsid.as_deref())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let cosmic_id = hit
        .cosmic
        .as_ref()
        .map(|c| c.cosmic_id.clone().into_vec())
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.trim().to_string())
        .find(|s| !s.is_empty());

    let clinvar_id = hit
        .clinvar
        .as_ref()
        .and_then(|c| c.variant_id)
        .map(|n| n.to_string());

    let (
        significance,
        clinvar_review_status,
        clinvar_review_stars,
        conditions,
        clinvar_conditions,
        clinvar_condition_reports,
    ) = hit
        .clinvar
        .as_ref()
        .map(|c| {
            let sig = pick_significance(&c.rcv);
            let (review_status, review_stars) = pick_review_status(&c.rcv);
            let (conditions, condition_rows, report_count) = aggregate_clinvar_conditions(&c.rcv);
            (
                sig,
                review_status,
                review_stars,
                conditions,
                condition_rows,
                report_count,
            )
        })
        .unwrap_or((None, None, None, Vec::new(), Vec::new(), None));

    let significance_evaluated = hit
        .clinvar
        .as_ref()
        .and_then(|c| newest_rcv_evaluation_date(&c.rcv, significance.as_deref()));
    let (significance_source, significance_note) = if significance.is_some() {
        (
            Some("MyVariant.info".to_string()),
            Some(derived_significance_note(&hit.id)),
        )
    } else {
        (None, None)
    };

    let cadd_score = hit.cadd.as_ref().and_then(|c| c.phred);
    let consequence = pick_consequence(hit);
    let cached_civic = extract_civic_cached_evidence(hit);
    let top_disease = clinvar_conditions.first().cloned();

    Variant {
        section_outcomes: crate::entities::variant::default_variant_section_outcomes(),
        id: hit.id.clone(),
        genome_build: None,
        genome_build_provenance: None,
        build_ambiguous: None,
        build_candidates: Vec::new(),
        gene,
        hgvs_p,
        legacy_name,
        hgvs_c,
        transcript: annotation
            .as_ref()
            .and_then(|value| value.transcript.clone()),
        protein_numbering_note: None,
        rsid,
        cosmic_id,
        significance,
        significance_source,
        significance_evaluated,
        significance_note,
        clinvar_id,
        clinvar_review_status,
        clinvar_review_stars,
        conditions,
        clinvar: None,
        clinvar_conditions,
        clinvar_condition_reports,
        consequence,
        cadd_score,
        sift_pred,
        polyphen_pred,
        conservation: extract_conservation(hit),
        expanded_predictions: extract_expanded_predictions(hit),
        population: None,
        cosmic_context: extract_cosmic_details(hit),
        cgi_associations: extract_cgi_associations(hit),
        civic: (!cached_civic.is_empty()).then_some(VariantCivicSection {
            cached_evidence: cached_civic,
            graphql: None,
        }),
        top_disease,
        cancerhotspots: None,
        cancer_frequencies: Vec::new(),
        cancer_frequency_source: None,
        gwas: Vec::new(),
        gwas_unavailable_reason: None,
        supporting_pmids: None,
        prediction: None,
    }
}

pub fn from_myvariant_search_hit(hit: &MyVariantHit) -> VariantSearchResult {
    let annotation = paired_annotation(hit, None, None);
    let mut gene = hit.dbnsfp.as_ref().map(pick_gene).unwrap_or_default();
    if gene.is_empty() {
        gene = hit
            .clinvar
            .as_ref()
            .and_then(|value| value.gene.as_ref())
            .and_then(|value| value.symbol.as_deref())
            .and_then(normalize_gene)
            .unwrap_or_default();
    }
    if let Some(annotation_gene) = annotation
        .as_ref()
        .and_then(|value| value.gene.as_deref())
        .and_then(normalize_gene)
    {
        gene = annotation_gene;
    }
    let hgvs_p = annotation.as_ref().and_then(|value| value.protein.clone());
    let legacy_name = legacy_name(&gene, hgvs_p.as_deref());

    let significance = hit.clinvar.as_ref().and_then(|c| pick_significance(&c.rcv));
    let significance_source = significance.as_ref().map(|_| "MyVariant.info".to_string());
    let significance_evaluated = hit
        .clinvar
        .as_ref()
        .and_then(|c| newest_rcv_evaluation_date(&c.rcv, significance.as_deref()));
    let clinvar_stars = hit
        .clinvar
        .as_ref()
        .and_then(|c| pick_review_status(&c.rcv).1);
    let gnomad_af = best_gnomad_af(hit).and_then(|a| a.af);
    let revel = hit
        .dbnsfp
        .as_ref()
        .and_then(|dbnsfp| dbnsfp.revel.as_ref())
        .and_then(|revel| first_score(revel.score.as_ref()));
    let gerp = hit
        .dbnsfp
        .as_ref()
        .and_then(|dbnsfp| dbnsfp.gerp.as_ref())
        .and_then(|gerp| first_score(gerp.rs.as_ref()));

    VariantSearchResult {
        id: hit.id.clone(),
        genome_build: crate::entities::variant::GenomeBuild::Grch37,
        genome_build_provenance: "MyVariant.info provider default".into(),
        gene,
        hgvs_p,
        hgvs_c: annotation.as_ref().and_then(|value| value.coding.clone()),
        transcript: annotation.and_then(|value| value.transcript),
        legacy_name,
        significance,
        significance_source,
        significance_evaluated,
        clinvar_stars,
        gnomad_af,
        revel,
        gerp,
        source_identity: None,
        matched_alias: None,
        transcript_annotations_complete: None,
        transcript_annotations: None,
    }
}

pub fn merge_cbioportal(variant: &mut Variant, summary: &CBioMutationSummary) {
    variant.cancer_frequencies = summary.cancer_distribution.clone();
    variant.cancer_frequency_source = Some(format!(
        "study={}, sample_list={}, profile={} (override with BIOMCP_CBIOPORTAL_STUDY/BIOMCP_CBIOPORTAL_SAMPLE_LIST/BIOMCP_CBIOPORTAL_MUTATION_PROFILE)",
        summary.study_id, summary.sample_list_id, summary.mutation_profile_id
    ));
}

#[cfg(feature = "alphagenome")]
pub fn merge_prediction(variant: &mut Variant, prediction: VariantPrediction) {
    variant.prediction = Some(prediction);
}

#[cfg(test)]
mod tests;
