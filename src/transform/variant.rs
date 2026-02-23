use std::collections::HashMap;

use crate::entities::variant::{
    ConditionReportCount, PopulationFrequency, Variant, VariantCgiAssociation, VariantCivicSection,
    VariantConservationScores, VariantCosmicContext, VariantPopulationBreakdown, VariantPrediction,
    VariantPredictionScore, VariantSearchResult,
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

fn pick_hgvs(values: Vec<StringOrVec>) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    for v in values {
        out.extend(v.into_vec());
    }

    // Prefer compact HGVS protein forms like `p.V600E`.
    for s in &out {
        let t = s.trim();
        if !t.starts_with("p.") {
            continue;
        }
        let rest = &t[2..];
        let mut chars = rest.chars();
        let Some(first) = chars.next() else { continue };
        if !first.is_ascii_uppercase() {
            continue;
        }
        let mut seen_digit = false;
        let mut digits = 0;
        let mut last: Option<char> = None;
        for ch in chars {
            if ch.is_ascii_digit() {
                seen_digit = true;
                digits += 1;
                last = Some(ch);
                continue;
            }
            last = Some(ch);
        }
        let Some(last) = last else { continue };
        if !seen_digit || digits == 0 {
            continue;
        }
        if last.is_ascii_uppercase() || last == '*' {
            // Ensure there are no extra letters (e.g., Val600Glu).
            if rest.len() >= 3 && rest.chars().all(|c| c.is_ascii_alphanumeric() || c == '*') {
                // This is still a heuristic; accept.
                if rest.len() <= 12 {
                    return Some(t.to_string());
                }
            }
        }
    }

    // Fall back to any `p.` form.
    for s in &out {
        let t = s.trim();
        if t.starts_with("p.") && !t.is_empty() {
            return Some(t.to_string());
        }
    }

    out.into_iter()
        .map(|s| s.trim().to_string())
        .find(|s| !s.is_empty())
}

fn pick_gene(dbnsfp: &crate::sources::myvariant::MyVariantDbnsfp) -> String {
    dbnsfp
        .genename
        .first()
        .and_then(normalize_gene)
        .unwrap_or_default()
}

fn pick_hgvsp(dbnsfp: &crate::sources::myvariant::MyVariantDbnsfp) -> Option<String> {
    pick_hgvs(vec![dbnsfp.hgvsp.clone()])
}

fn pick_hgvsc(dbnsfp: &crate::sources::myvariant::MyVariantDbnsfp) -> Option<String> {
    pick_hgvs(vec![dbnsfp.hgvsc.clone()])
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

fn gnomad_subpopulations(hit: &MyVariantHit) -> Vec<PopulationFrequency> {
    let Some(af) = best_gnomad_af(hit) else {
        return Vec::new();
    };

    let mut out: Vec<PopulationFrequency> = Vec::new();
    for (label, value) in [
        ("African/African American", af.af_afr),
        ("East Asian", af.af_eas),
        ("Non-Finnish European", af.af_nfe),
        ("South Asian", af.af_sas),
        ("Latino/Admixed American", af.af_amr),
        ("Ashkenazi Jewish", af.af_asj),
        ("Finnish", af.af_fin),
    ] {
        let Some(freq) = value else { continue };
        out.push(PopulationFrequency {
            population: label.to_string(),
            af: freq,
        });
    }

    out
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
        "BayesDel addAF",
        dbnsfp
            .bayesdel_addaf
            .as_ref()
            .and_then(|v| first_score(v.score.as_ref())),
        normalize_prediction(
            dbnsfp
                .bayesdel_addaf
                .as_ref()
                .and_then(|v| v.pred.as_ref())
                .and_then(first_nonempty),
            "bayesdel_addaf",
        ),
    );

    out
}

fn push_population(out: &mut Vec<PopulationFrequency>, label: &str, af: Option<f64>) {
    let Some(af) = af else {
        return;
    };
    out.push(PopulationFrequency {
        population: label.to_string(),
        af,
    });
}

fn extract_population_breakdown(hit: &MyVariantHit) -> Option<VariantPopulationBreakdown> {
    let af = best_gnomad_af(hit);
    let mut populations: Vec<PopulationFrequency> = Vec::new();
    if let Some(af) = af {
        push_population(&mut populations, "African/African American", af.af_afr);
        push_population(
            &mut populations,
            "African/African American (female)",
            af.af_afr_female,
        );
        push_population(
            &mut populations,
            "African/African American (male)",
            af.af_afr_male,
        );
        push_population(&mut populations, "Latino/Admixed American", af.af_amr);
        push_population(
            &mut populations,
            "Latino/Admixed American (female)",
            af.af_amr_female,
        );
        push_population(
            &mut populations,
            "Latino/Admixed American (male)",
            af.af_amr_male,
        );
        push_population(&mut populations, "East Asian", af.af_eas);
        push_population(&mut populations, "East Asian (Japanese)", af.af_eas_jpn);
        push_population(&mut populations, "East Asian (Korean)", af.af_eas_kor);
        push_population(&mut populations, "Non-Finnish European", af.af_nfe);
        push_population(
            &mut populations,
            "Non-Finnish European (Bulgarian)",
            af.af_nfe_bgr,
        );
        push_population(
            &mut populations,
            "Non-Finnish European (Estonian)",
            af.af_nfe_est,
        );
        push_population(
            &mut populations,
            "Non-Finnish European (Northwestern)",
            af.af_nfe_nwe,
        );
        push_population(
            &mut populations,
            "Non-Finnish European (Other)",
            af.af_nfe_onf,
        );
        push_population(
            &mut populations,
            "Non-Finnish European (Southeastern)",
            af.af_nfe_seu,
        );
        push_population(
            &mut populations,
            "Non-Finnish European (Swedish)",
            af.af_nfe_swe,
        );
        push_population(&mut populations, "South Asian", af.af_sas);
        push_population(&mut populations, "Ashkenazi Jewish", af.af_asj);
        push_population(&mut populations, "Finnish", af.af_fin);
        push_population(&mut populations, "Other", af.af_oth);
    }

    let exac_af = hit.exac.as_ref().and_then(|e| e.af);
    let exac_nontcga_af = hit.exac_nontcga.as_ref().and_then(|e| e.af);

    if populations.is_empty() && exac_af.is_none() && exac_nontcga_af.is_none() {
        return None;
    }

    Some(VariantPopulationBreakdown {
        populations,
        exac_af,
        exac_nontcga_af,
    })
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

fn clinvar_review_stars(review_status: &str) -> Option<u8> {
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
    let mut gene = String::new();
    let mut hgvs_p: Option<String> = None;
    let mut hgvs_c: Option<String> = None;
    let mut sift_pred: Option<String> = None;
    let mut polyphen_pred: Option<String> = None;

    if let Some(dbnsfp) = hit.dbnsfp.as_ref() {
        gene = pick_gene(dbnsfp);
        hgvs_p = pick_hgvsp(dbnsfp);
        hgvs_c = pick_hgvsc(dbnsfp);

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

    let gnomad_af = best_gnomad_af(hit).and_then(|a| a.af);
    let gnomad_subpopulations = gnomad_subpopulations(hit);

    let cadd_score = hit.cadd.as_ref().and_then(|c| c.phred);
    let consequence = pick_consequence(hit);
    let cached_civic = extract_civic_cached_evidence(hit);

    Variant {
        id: hit.id.clone(),
        gene,
        hgvs_p,
        hgvs_c,
        rsid,
        cosmic_id,
        significance,
        clinvar_id,
        clinvar_review_status,
        clinvar_review_stars,
        conditions,
        clinvar_conditions,
        clinvar_condition_reports,
        gnomad_af,
        gnomad_subpopulations,
        population: None,
        consequence,
        cadd_score,
        sift_pred,
        polyphen_pred,
        conservation: extract_conservation(hit),
        expanded_predictions: extract_expanded_predictions(hit),
        population_breakdown: extract_population_breakdown(hit),
        cosmic_context: extract_cosmic_details(hit),
        cgi_associations: extract_cgi_associations(hit),
        civic: (!cached_civic.is_empty()).then_some(VariantCivicSection {
            cached_evidence: cached_civic,
            graphql: None,
        }),
        cancer_frequencies: Vec::new(),
        cancer_frequency_source: None,
        gwas: Vec::new(),
        prediction: None,
    }
}

pub fn from_myvariant_search_hit(hit: &MyVariantHit) -> VariantSearchResult {
    let gene = hit.dbnsfp.as_ref().map(pick_gene).unwrap_or_default();
    let hgvs_p = hit.dbnsfp.as_ref().and_then(pick_hgvsp);

    let significance = hit.clinvar.as_ref().and_then(|c| pick_significance(&c.rcv));
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
        gene,
        hgvs_p,
        significance,
        clinvar_stars,
        gnomad_af,
        revel,
        gerp,
    }
}

pub fn merge_cbioportal(variant: &mut Variant, summary: &CBioMutationSummary) {
    variant.cancer_frequencies = summary.cancer_distribution.clone();
    variant.cancer_frequency_source = Some(format!(
        "study={}, sample_list={}, profile={} (override with BIOMCP_CBIOPORTAL_STUDY/BIOMCP_CBIOPORTAL_SAMPLE_LIST/BIOMCP_CBIOPORTAL_MUTATION_PROFILE)",
        summary.study_id, summary.sample_list_id, summary.mutation_profile_id
    ));
}

pub fn merge_prediction(variant: &mut Variant, prediction: VariantPrediction) {
    variant.prediction = Some(prediction);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn significance_rank_prefers_pathogenic_over_benign() {
        assert!(significance_rank("Pathogenic") > significance_rank("Benign"));
        assert!(
            significance_rank("Likely pathogenic") > significance_rank("Uncertain significance")
        );
    }

    #[test]
    fn normalize_gene_uppercases() {
        assert_eq!(normalize_gene("egfr").as_deref(), Some("EGFR"));
        assert_eq!(normalize_gene("  tP53 ").as_deref(), Some("TP53"));
    }

    #[test]
    fn normalize_polyphen_codes() {
        assert_eq!(normalize_polyphen("D"), "Probably damaging");
        assert_eq!(normalize_polyphen("P"), "Possibly damaging");
        assert_eq!(normalize_polyphen("B"), "Benign");
    }

    #[test]
    fn clinvar_review_stars_known_statuses() {
        assert_eq!(
            clinvar_review_stars("no assertion criteria provided"),
            Some(0)
        );
        assert_eq!(
            clinvar_review_stars("criteria provided, single submitter"),
            Some(1)
        );
        assert_eq!(
            clinvar_review_stars("criteria provided, multiple submitters, no conflicts"),
            Some(2)
        );
        assert_eq!(clinvar_review_stars("reviewed by expert panel"), Some(3));
        assert_eq!(clinvar_review_stars("practice guideline"), Some(4));
    }

    #[test]
    fn pick_review_status_prefers_highest_star_rating() {
        let rcvs = vec![
            MyVariantClinVarRcv {
                clinical_significance: None,
                review_status: Some("criteria provided, single submitter".into()),
                conditions: None,
            },
            MyVariantClinVarRcv {
                clinical_significance: None,
                review_status: Some("reviewed by expert panel".into()),
                conditions: None,
            },
        ];

        let (status, stars) = pick_review_status(&rcvs);
        assert_eq!(stars, Some(3));
        assert_eq!(status.as_deref(), Some("reviewed by expert panel"));
    }

    #[test]
    fn pick_significance_handles_empty_and_partial_rcvs() {
        let empty: Vec<MyVariantClinVarRcv> = Vec::new();
        assert_eq!(pick_significance(&empty), None);

        let partial = vec![MyVariantClinVarRcv {
            clinical_significance: None,
            review_status: Some("criteria provided, single submitter".into()),
            conditions: None,
        }];
        assert_eq!(pick_significance(&partial), None);
    }

    #[test]
    fn pick_significance_with_brca1_rcvs() {
        let rcvs = vec![
            MyVariantClinVarRcv {
                clinical_significance: Some("Likely benign".into()),
                review_status: Some("criteria provided, single submitter".into()),
                conditions: Some(serde_json::json!({"name": "Breast-ovarian cancer"})),
            },
            MyVariantClinVarRcv {
                clinical_significance: Some("Pathogenic".into()),
                review_status: Some("reviewed by expert panel".into()),
                conditions: Some(serde_json::json!({"name": "Hereditary breast cancer"})),
            },
        ];

        assert_eq!(pick_significance(&rcvs).as_deref(), Some("Pathogenic"));
    }

    #[test]
    fn pick_significance_with_kras_rcvs() {
        let rcvs = vec![
            MyVariantClinVarRcv {
                clinical_significance: Some("Uncertain significance".into()),
                review_status: None,
                conditions: Some(serde_json::json!({"name": "Colorectal carcinoma"})),
            },
            MyVariantClinVarRcv {
                clinical_significance: Some("Likely pathogenic".into()),
                review_status: Some("criteria provided, single submitter".into()),
                conditions: Some(serde_json::json!({"name": "Lung adenocarcinoma"})),
            },
        ];

        assert_eq!(
            pick_significance(&rcvs).as_deref(),
            Some("Likely pathogenic")
        );
    }

    #[test]
    fn aggregate_clinvar_conditions_counts_reports() {
        let rcvs = vec![
            MyVariantClinVarRcv {
                clinical_significance: None,
                review_status: None,
                conditions: Some(serde_json::json!([
                    {"name": "Melanoma"},
                    {"name": "Lung cancer"}
                ])),
            },
            MyVariantClinVarRcv {
                clinical_significance: None,
                review_status: None,
                conditions: Some(serde_json::json!({"name": "Melanoma"})),
            },
        ];

        let (names, rows, reports) = aggregate_clinvar_conditions(&rcvs);
        assert_eq!(reports, Some(3));
        assert_eq!(names.first().map(String::as_str), Some("Melanoma"));
        assert_eq!(rows.first().map(|r| r.reports), Some(2));
    }

    #[test]
    fn extracts_expanded_variant_sections() {
        let hit: MyVariantHit = serde_json::from_value(serde_json::json!({
            "_id": "chr7:g.140453136A>T",
            "dbnsfp": {
                "genename": "BRAF",
                "hgvsp": "p.V600E",
                "sift": {"pred": "D", "score": 0.01},
                "revel": {"score": 0.94},
                "alphamissense": {"score": 0.99, "pred": "P"},
                "phylop": {"100way_vertebrate": {"rankscore": 0.92}},
                "phastcons": {"100way_vertebrate": {"rankscore": 0.88}},
                "gerp++": {"rs": 5.6}
            },
            "gnomad_exome": {"af": {"af": 0.0001, "af_afr": 0.0002, "af_eas_jpn": 0.0003}},
            "exac": {"af": 0.0004},
            "exac_nontcga": {"af": 0.0005},
            "cosmic": {"cosmic_id": "COSM476", "mut_freq": 2.8, "tumor_site": "skin"},
            "cgi": [{"drug": "vemurafenib", "association": "Responsive", "evidence_level": "FDA"}],
            "civic": {
                "molecularProfiles": [{
                    "name": "BRAF V600E",
                    "evidenceItems": [{
                        "id": 1,
                        "name": "EID1",
                        "evidenceType": "PREDICTIVE",
                        "evidenceLevel": "A",
                        "significance": "SENSITIVITYRESPONSE",
                        "status": "ACCEPTED",
                        "disease": {"displayName": "Melanoma"},
                        "therapies": [{"name": "Vemurafenib"}]
                    }]
                }]
            }
        }))
        .expect("variant payload should parse");

        let variant = from_myvariant_hit(&hit);
        assert!(variant.conservation.is_some());
        assert!(!variant.expanded_predictions.is_empty());
        assert!(variant.population_breakdown.is_some());
        assert!(variant.cosmic_context.is_some());
        assert_eq!(variant.cgi_associations.len(), 1);
        assert_eq!(
            variant
                .civic
                .as_ref()
                .map(|v| v.cached_evidence.len())
                .unwrap_or_default(),
            1
        );
    }
}
