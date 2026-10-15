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
use crate::sources::myvariant::MyVariantHit;
use biodata::MyVariantClinVarRcv;
use biodata::MyVariantGnomadAf;

fn normalize_gene(gene: &str) -> Option<String> {
    let g = gene.trim();
    if g.is_empty() {
        return None;
    }
    Some(g.to_uppercase())
}

fn pick_gene(dbnsfp: &biodata::MyVariantDbnsfpProjection) -> String {
    dbnsfp
        .genename()
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
    hit.source().clinvar()?.rcv().iter().find_map(|rcv| {
        let name = rcv.preferred_name()?.trim();
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
/// the SnpEff build lags it (NM_000546.5 against NM_000546.6,
/// finding 21). A cohort whose ClinVar-preferred stems disagree carries no
/// marker.
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
/// signal a ClinVar-free response carries), then
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
    let annotations = hit.source().snpeff()?.annotations();
    annotations
        .iter()
        .enumerate()
        .filter(|ann| ann.1.hgvs_c().is_some_and(|value| !value.trim().is_empty()))
        .min_by_key(|ann| {
            let feature = ann.1.feature_id().unwrap_or_default();
            let feature_stem = accession_stem(feature);
            if preferred_stem.is_some_and(|preferred| feature_stem == preferred) {
                0
            } else if mane_stem.is_some_and(|mane| feature_stem == mane) {
                1
            } else if requested_change.is_some_and(|change| {
                ann.1
                    .hgvs_p()
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
        .source()
        .snpeff()?
        .annotations()
        .get(selected_snpeff_annotation_index(
            hit,
            mane_transcript,
            requested_change,
        )?)?;
    let feature = selected.feature_id().map(str::to_owned)?;
    // ClinVar's preferred names carry the current RefSeq accession for the
    // stem while the SnpEff build lags it, so the
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
        gene: selected.genename().map(str::to_owned),
        transcript: Some(transcript),
        coding: selected.hgvs_c().map(str::to_owned),
        protein: selected.hgvs_p().map(str::to_owned),
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
/// gene+protein query names the hit.
pub(crate) fn canonical_protein_change(
    hit: &MyVariantHit,
    mane_transcript: Option<&str>,
    requested_change: Option<&str>,
) -> Option<String> {
    paired_annotation(hit, mane_transcript, requested_change)
        .and_then(|annotation| annotation.protein)
}

/// The transcript whose numbering the headline protein change uses.
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
/// rules never refuse a hit that does name it.
pub(crate) fn mane_annotation_names_change(
    hit: &MyVariantHit,
    change: &str,
    mane_transcript: Option<&str>,
) -> bool {
    let Some(mane_stem) = mane_transcript.map(accession_stem) else {
        return false;
    };
    let snpeff_names = hit.source().snpeff().is_some_and(|snpeff| {
        snpeff.annotations().iter().any(|ann| {
            ann.feature_id()
                .is_some_and(|feature| accession_stem(feature) == mane_stem)
                && ann
                    .hgvs_p()
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
/// sees where the request is actually spelled.
/// Protein-structure rows (`feature_type: interaction`) carry no transcript
/// of their own, so only transcript-shaped features answer. Returns the
/// transcript and its spelling of the request.
pub(crate) fn isoform_annotation_naming_change(
    hit: &MyVariantHit,
    change: &str,
    mane_transcript: Option<&str>,
) -> Option<(String, String)> {
    let mane_stem = mane_transcript.map(accession_stem);
    hit.source().snpeff()?.annotations().iter().find_map(|ann| {
        let feature = ann.feature_id()?;
        if feature.is_empty() || !looks_like_transcript(feature) {
            return None;
        }
        if mane_stem.is_some_and(|mane| accession_stem(feature) == mane) {
            return None;
        }
        let raw = ann.hgvs_p().map(str::trim).filter(|v| !v.is_empty())?;
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
    hit.source()
        .cadd()
        .and_then(biodata::MyVariantCaddProjection::consequence)
        .and_then(biodata::MyVariantCaddConsequence::first)
        .map(normalize_consequence)
        .filter(|v| !v.is_empty())
}

fn best_gnomad_af(hit: &MyVariantHit) -> Option<&MyVariantGnomadAf> {
    hit.source()
        .gnomad_exome()
        .and_then(|v| v.af())
        .or_else(|| {
            hit.source()
                .gnomad()
                .and_then(|g| g.exomes())
                .and_then(|v| v.af())
        })
        .or_else(|| {
            hit.source()
                .gnomad()
                .and_then(|g| g.genomes())
                .and_then(|v| v.af())
        })
}

fn first_score(value: Option<&biodata::MyVariantDbnsfpNumber>) -> Option<f64> {
    value.and_then(biodata::MyVariantDbnsfpNumber::first)
}

fn first_nonempty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn extract_conservation(hit: &MyVariantHit) -> Option<VariantConservationScores> {
    let dbnsfp = hit.source().dbnsfp()?;

    let scores = VariantConservationScores {
        phylop_100way_vertebrate: dbnsfp
            .phylop()
            .and_then(|p| p.way_100_vertebrate())
            .and_then(|v| first_score(v.rankscore())),
        phylop_470way_mammalian: dbnsfp
            .phylop()
            .and_then(|p| p.way_470_mammalian())
            .and_then(|v| first_score(v.rankscore())),
        phastcons_100way_vertebrate: dbnsfp
            .phastcons()
            .and_then(|p| p.way_100_vertebrate())
            .and_then(|v| first_score(v.rankscore())),
        phastcons_470way_mammalian: dbnsfp
            .phastcons()
            .and_then(|p| p.way_470_mammalian())
            .and_then(|v| first_score(v.rankscore())),
        gerp_rs: dbnsfp.gerp().and_then(|g| first_score(g.rs())),
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
    let Some(dbnsfp) = hit.source().dbnsfp() else {
        return Vec::new();
    };

    let mut out: Vec<VariantPredictionScore> = Vec::new();
    push_prediction(
        &mut out,
        "REVEL",
        dbnsfp.revel().and_then(|v| first_score(v.score())),
        None,
    );
    push_prediction(
        &mut out,
        "AlphaMissense",
        dbnsfp.alphamissense().and_then(|v| first_score(v.score())),
        normalize_prediction(
            dbnsfp
                .alphamissense()
                .and_then(|v| v.pred())
                .and_then(|value| first_nonempty(value.first())),
            "alphamissense",
        ),
    );
    push_prediction(
        &mut out,
        "ClinPred",
        dbnsfp.clinpred().and_then(|v| first_score(v.score())),
        normalize_prediction(
            dbnsfp
                .clinpred()
                .and_then(|v| v.pred())
                .and_then(|value| first_nonempty(value.first())),
            "clinpred",
        ),
    );
    push_prediction(
        &mut out,
        "SIFT",
        dbnsfp.sift().and_then(|v| first_score(v.score())),
        dbnsfp
            .sift()
            .and_then(|v| v.pred())
            .and_then(biodata::MyVariantDbnsfpText::first)
            .map(normalize_sift),
    );
    push_prediction(
        &mut out,
        "MetaRNN",
        dbnsfp.metarnn().and_then(|v| first_score(v.score())),
        normalize_prediction(
            dbnsfp
                .metarnn()
                .and_then(|v| v.pred())
                .and_then(|value| first_nonempty(value.first())),
            "metarnn",
        ),
    );
    push_prediction(
        &mut out,
        "BayesDel add-AF",
        dbnsfp
            .bayesdel()
            .and_then(|v| v.add_af())
            .and_then(|v| first_score(v.score())),
        normalize_prediction(
            dbnsfp
                .bayesdel()
                .and_then(|v| v.add_af())
                .and_then(|v| v.pred())
                .and_then(|value| first_nonempty(value.first())),
            "bayesdel_add_af",
        ),
    );
    push_prediction(
        &mut out,
        "BayesDel no-AF",
        dbnsfp
            .bayesdel()
            .and_then(|v| v.no_af())
            .and_then(|v| first_score(v.score())),
        normalize_prediction(
            dbnsfp
                .bayesdel()
                .and_then(|v| v.no_af())
                .and_then(|v| v.pred())
                .and_then(|value| first_nonempty(value.first())),
            "bayesdel_no_af",
        ),
    );

    out
}

fn extract_cosmic_details(hit: &MyVariantHit) -> Option<VariantCosmicContext> {
    // MyVariant carries these fields from COSMIC's own annotation; a
    // disabled cosmic leaves none of them in variant output.
    if crate::sources::disable::is_disabled("cosmic") {
        return None;
    }
    let cosmic = hit.source().cosmic()?;

    let context = VariantCosmicContext {
        mut_freq: cosmic.mut_freq(),
        tumor_site: first_nonempty(
            cosmic
                .tumor_site()
                .and_then(biodata::MyVariantCosmicText::first),
        ),
        mut_nt: first_nonempty(
            cosmic
                .mut_nt()
                .and_then(biodata::MyVariantCosmicText::first),
        ),
    };

    if context.mut_freq.is_none() && context.tumor_site.is_none() && context.mut_nt.is_none() {
        None
    } else {
        Some(context)
    }
}

fn extract_cgi_associations(hit: &MyVariantHit) -> Vec<VariantCgiAssociation> {
    // MyVariant carries these rows from the Cancer Genome Interpreter's
    // own annotation; a disabled cancer-genome-interpreter drops them.
    if crate::sources::disable::is_disabled("cancer-genome-interpreter") {
        return Vec::new();
    }
    hit.cached_cgi()
        .associations()
        .iter()
        .map(|row| VariantCgiAssociation {
            drug: row.drug().to_owned(),
            association: row.association().map(str::to_owned),
            tumor_type: row.tumor_type().map(str::to_owned),
            evidence_level: row.evidence_level().map(str::to_owned),
            source: row.source().map(str::to_owned),
        })
        .collect()
}

fn extract_civic_cached_evidence(hit: &MyVariantHit) -> Vec<CivicEvidenceItem> {
    hit.cached_civic()
        .evidence_items()
        .iter()
        .map(|row| CivicEvidenceItem {
            id: row.id(),
            name: row.name().to_owned(),
            molecular_profile: row.molecular_profile().to_owned(),
            evidence_type: row.evidence_type().to_owned(),
            evidence_level: row.evidence_level().to_owned(),
            significance: row.significance().to_owned(),
            disease: row.disease().map(str::to_owned),
            therapies: row.therapies().to_vec(),
            status: row.status().to_owned(),
            citation: None,
            source_type: None,
            publication_year: None,
        })
        .collect()
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
    let Some(v) = rcv.conditions() else {
        return vec![];
    };

    let mut names: Vec<String> = Vec::new();
    if let Some(obj) = v.as_object() {
        if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
            names.push(name.to_string());
        }
    } else if let Some(arr) = v.as_array() {
        for item in arr {
            if let Some(name) = item.as_str() {
                names.push(name.to_string());
                continue;
            }
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                names.push(name.to_string());
            }
        }
    } else if let Some(s) = v.as_str() {
        names.push(s.to_string());
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
        let Some(sig) = r.clinical_significance() else {
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
/// classification, so the headline date belongs to the printed
/// term instead of a newer, differently classified record. Only strict
/// day-shaped dates surface, under the one shared day-shape rule,
/// `crate::utils::date::is_day_shaped`, which the ClinVar fallback age
/// label reads too.
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
            rcv.clinical_significance()
                .map(str::trim)
                .is_some_and(|value| value.eq_ignore_ascii_case(significance))
        })
        .filter_map(|rcv| rcv.last_evaluated().map(str::trim))
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
        let Some(status) = r.review_status().map(str::trim) else {
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
    from_myvariant_annotation(hit, paired_annotation(hit, None, None))
}

/// Same as [`from_myvariant_hit`], with the gene's MANE transcript (the
/// marker ClinVar's preferred names carry) so a hit without its own
/// ClinVar record still headlines the MANE transcript when the query
/// response marks one, and with the requested protein change so a
/// ClinVar-free response still headlines the annotation that names the
/// request.
pub(crate) fn from_myvariant_hit_with_mane(
    hit: &MyVariantHit,
    mane_transcript: Option<&str>,
    requested_change: Option<&str>,
) -> Variant {
    from_myvariant_annotation(hit, paired_annotation(hit, mane_transcript, requested_change))
}

/// Render all record facts with the caller-selected complete source assertion.
pub(crate) fn from_myvariant_hit_with_tuple(
    hit: &MyVariantHit,
    gene: &str,
    transcript: &str,
    coding: &str,
    protein: Option<&str>,
) -> Variant {
    from_myvariant_annotation(
        hit,
        Some(TranscriptAnnotation {
            gene: Some(gene.into()),
            transcript: Some(transcript.into()),
            coding: Some(coding.into()),
            protein: protein.map(str::to_string),
        }),
    )
}
fn from_myvariant_annotation(
    hit: &MyVariantHit,
    annotation: Option<TranscriptAnnotation>,
) -> Variant {
    let mut gene = String::new();
    let mut hgvs_p: Option<String> = None;
    let mut hgvs_c: Option<String> = None;
    let mut sift_pred: Option<String> = None;
    let mut polyphen_pred: Option<String> = None;

    if let Some(dbnsfp) = hit.source().dbnsfp() {
        gene = pick_gene(dbnsfp);

        sift_pred = dbnsfp
            .sift()
            .and_then(|s| s.pred())
            .and_then(biodata::MyVariantDbnsfpText::first)
            .map(normalize_sift)
            .filter(|s| !s.is_empty());

        polyphen_pred = dbnsfp
            .polyphen2()
            .and_then(|p| p.hdiv())
            .and_then(|h| h.pred())
            .and_then(biodata::MyVariantDbnsfpText::first)
            .map(normalize_polyphen)
            .filter(|s| !s.is_empty());
    }

    if gene.is_empty() {
        gene = hit
            .source()
            .clinvar()
            .and_then(|clinvar| clinvar.gene())
            .and_then(|gene| gene.symbol())
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
        .source()
        .dbsnp()
        .and_then(|d| d.rsid())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let cosmic_id = if crate::sources::disable::is_disabled("cosmic") {
        None
    } else {
        hit.source()
            .cosmic()
            .and_then(biodata::MyVariantCosmicProjection::cosmic_id)
            .into_iter()
            .flat_map(biodata::MyVariantCosmicText::values)
            .map(|s| s.trim().to_string())
            .find(|s| !s.is_empty())
    };

    let clinvar_id = hit
        .source()
        .clinvar()
        .and_then(|c| c.variant_id())
        .map(|n| n.to_string());

    let (
        significance,
        clinvar_review_status,
        clinvar_review_stars,
        conditions,
        clinvar_conditions,
        clinvar_condition_reports,
    ) = hit
        .source()
        .clinvar()
        .map(|c| {
            let sig = pick_significance(c.rcv());
            let (review_status, review_stars) = pick_review_status(c.rcv());
            let (conditions, condition_rows, report_count) = aggregate_clinvar_conditions(c.rcv());
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
        .source()
        .clinvar()
        .and_then(|c| newest_rcv_evaluation_date(c.rcv(), significance.as_deref()));
    let (significance_source, significance_note) = if significance.is_some() {
        (
            Some("MyVariant.info".to_string()),
            Some(derived_significance_note(hit.source().id())),
        )
    } else {
        (None, None)
    };

    let cadd_score = hit
        .source()
        .cadd()
        .and_then(biodata::MyVariantCaddProjection::phred);
    let consequence = pick_consequence(hit);
    let cached_civic = extract_civic_cached_evidence(hit);
    let top_disease = clinvar_conditions.first().cloned();

    Variant {
        section_outcomes: crate::entities::variant::default_variant_section_outcomes(),
        id: hit.source().id().to_owned(),
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
    let mut gene = hit.source().dbnsfp().map(pick_gene).unwrap_or_default();
    if gene.is_empty() {
        gene = hit
            .source()
            .clinvar()
            .and_then(|value| value.gene())
            .and_then(|value| value.symbol())
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

    let significance = hit
        .source()
        .clinvar()
        .and_then(|c| pick_significance(c.rcv()));
    let significance_source = significance.as_ref().map(|_| "MyVariant.info".to_string());
    let significance_evaluated = hit
        .source()
        .clinvar()
        .and_then(|c| newest_rcv_evaluation_date(c.rcv(), significance.as_deref()));
    let clinvar_stars = hit
        .source()
        .clinvar()
        .and_then(|c| pick_review_status(c.rcv()).1);
    let gnomad_af = best_gnomad_af(hit).and_then(MyVariantGnomadAf::af);
    let revel = hit
        .source()
        .dbnsfp()
        .and_then(|dbnsfp| dbnsfp.revel())
        .and_then(|revel| first_score(revel.score()));
    let gerp = hit
        .source()
        .dbnsfp()
        .and_then(|dbnsfp| dbnsfp.gerp())
        .and_then(|gerp| first_score(gerp.rs()));

    VariantSearchResult {
        id: hit.source().id().to_owned(),
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
#[path = "variant_tests.rs"]
mod tests;
