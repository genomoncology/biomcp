use std::collections::HashSet;

use time::Month;

use crate::entities::drug::{Drug, DrugInteraction, DrugSearchResult};
use crate::sources::mychem::{MyChemHit, MyChemNdcField, MyChemPharmClass};

fn normalize_name(value: &str) -> String {
    value.trim().trim_matches('.').to_ascii_lowercase()
}

fn ndc_nonproprietaryname(hit: &MyChemHit) -> Option<&str> {
    let ndc = hit.ndc.as_ref()?;
    match ndc {
        MyChemNdcField::One(v) => v.nonproprietaryname.as_deref(),
        MyChemNdcField::Many(v) => v.iter().find_map(|n| n.nonproprietaryname.as_deref()),
    }
}

fn ndc_pharm_classes(hit: &MyChemHit) -> Vec<&str> {
    let Some(ndc) = hit.ndc.as_ref() else {
        return Vec::new();
    };
    match ndc {
        MyChemNdcField::One(v) => v
            .pharm_classes
            .iter()
            .filter_map(MyChemPharmClass::as_str)
            .collect(),
        MyChemNdcField::Many(v) => v
            .iter()
            .flat_map(|n| n.pharm_classes.iter().filter_map(MyChemPharmClass::as_str))
            .collect(),
    }
}

fn clean_moa_class(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let trimmed = trimmed.strip_suffix("[MoA]").unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix("[EPC]").unwrap_or(trimmed);
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn moa_pharm_classes(hit: &MyChemHit) -> Vec<String> {
    ndc_pharm_classes(hit)
        .into_iter()
        .filter(|v| v.contains("[MoA]"))
        .filter_map(clean_moa_class)
        .collect()
}

fn chebi_name(hit: &MyChemHit) -> Option<&str> {
    hit.chebi.as_ref().and_then(|c| c.name())
}

fn unii_display_name(hit: &MyChemHit) -> Option<&str> {
    hit.unii.as_ref().and_then(|u| u.display_name())
}

fn unii_id(hit: &MyChemHit) -> Option<&str> {
    hit.unii.as_ref().and_then(|u| u.unii())
}

fn openfda_generic_name(hit: &MyChemHit) -> Option<&str> {
    hit.openfda.as_ref().and_then(|o| o.generic_name.first())
}

fn openfda_brand_name(hit: &MyChemHit) -> Option<&str> {
    hit.openfda.as_ref().and_then(|o| o.brand_name.first())
}

fn best_name_from_hit(hit: &MyChemHit) -> Option<String> {
    let candidates: [Option<&str>; 8] = [
        ndc_nonproprietaryname(hit),
        openfda_generic_name(hit),
        openfda_brand_name(hit),
        hit.drugbank.as_ref().and_then(|d| d.name.as_deref()),
        hit.chembl.as_ref().and_then(|c| c.pref_name.as_deref()),
        hit.gtopdb.as_ref().and_then(|g| g.name.as_deref()),
        unii_display_name(hit),
        chebi_name(hit),
    ];

    candidates
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(normalize_name)
}

fn hit_all_names(hit: &MyChemHit) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(v) = ndc_nonproprietaryname(hit) {
        out.push(normalize_name(v));
    }
    if let Some(v) = openfda_generic_name(hit) {
        out.push(normalize_name(v));
    }
    if let Some(v) = openfda_brand_name(hit) {
        out.push(normalize_name(v));
    }
    if let Some(v) = hit.drugbank.as_ref().and_then(|d| d.name.as_deref()) {
        out.push(normalize_name(v));
    }
    if let Some(drugbank) = hit.drugbank.as_ref() {
        for synonym in &drugbank.synonyms {
            out.push(normalize_name(synonym));
        }
    }
    if let Some(v) = hit.chembl.as_ref().and_then(|c| c.pref_name.as_deref()) {
        out.push(normalize_name(v));
    }
    if let Some(v) = hit.gtopdb.as_ref().and_then(|g| g.name.as_deref()) {
        out.push(normalize_name(v));
    }
    if let Some(v) = unii_display_name(hit) {
        out.push(normalize_name(v));
    }
    if let Some(v) = chebi_name(hit) {
        out.push(normalize_name(v));
    }
    out
}

mod enrichment;
use enrichment::*;

fn name_matches_requested(candidate: &str, requested: &str) -> bool {
    if candidate == requested {
        return true;
    }
    candidate.starts_with(&format!("{requested} "))
        || candidate.ends_with(&format!(" {requested}"))
        || candidate.contains(&format!(" {requested} "))
}

fn json_first_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.trim().to_string()).filter(|v| !v.is_empty()),
        serde_json::Value::Array(arr) => arr.iter().find_map(json_first_string),
        serde_json::Value::Object(obj) => obj.values().find_map(json_first_string),
        _ => None,
    }
}

fn interactions_from_hit(hit: &MyChemHit) -> Vec<DrugInteraction> {
    let Some(drugbank) = hit.drugbank.as_ref() else {
        return Vec::new();
    };

    let mut out: Vec<DrugInteraction> = Vec::new();
    for (index, row) in drugbank.drug_interactions.iter().enumerate() {
        let Some(obj) = row.as_object() else { continue };
        let drug = obj
            .get("name")
            .or_else(|| obj.get("drug"))
            .or_else(|| obj.get("drug_name"))
            .or_else(|| obj.get("drugbank_name"))
            .and_then(json_first_string);
        let Some(drug) = drug else { continue };
        let description = obj
            .get("description")
            .or_else(|| obj.get("interaction"))
            .or_else(|| obj.get("comment"))
            .and_then(json_first_string);
        for (fields, action, reason, target) in [
            (
                &["name", "drug", "drug_name", "drugbank_name"][..],
                "select_interaction_partner",
                "name owns interaction partner under existing field precedence",
                Some(drug.clone()),
            ),
            (
                &["description", "interaction", "comment"][..],
                "select_interaction_description",
                "description owns interaction text",
                description.clone(),
            ),
        ] {
            if let Some(field) = fields.iter().find(|field| obj.contains_key(**field)) {
                hit.record_source(
                    &format!("/drugbank/drug_interactions/{index}/{field}"),
                    "enrichment",
                    action,
                    reason,
                    target,
                );
            }
        }
        out.push(DrugInteraction {
            drug,
            ddinter_id: None,
            level: None,
            description,
            partner_classes: Vec::new(),
        });
    }

    out
}

pub fn from_mychem_search_hit(hit: &MyChemHit) -> Option<DrugSearchResult> {
    let Some(name) = best_name_from_hit(hit) else {
        hit.record_row("search projection", "discard", "row has no display name");
        return None;
    };
    record_display(hit, &name, "search projection");
    if name.is_empty() {
        hit.record_row(
            "search projection",
            "discard",
            "display normalizes to empty",
        );
        return None;
    }
    let mechanisms = chembl_mechanisms_from_hit(hit);
    let mechanism = mechanisms
        .first()
        .cloned()
        .or_else(|| fallback_mechanism_from_hit(hit));
    let target = first_target_from_hit(hit);

    let drugbank_id = hit
        .drugbank
        .as_ref()
        .and_then(|d| d.id.clone())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    hit.record_field(
        "drugbank",
        "id",
        "search projection",
        "select_first_code",
        "first DrugBank identifier",
        drugbank_id.clone(),
    );
    Some(DrugSearchResult {
        name,
        drugbank_id,
        drug_type: drug_type_from_hit(hit),
        mechanism,
        target,
    })
}

fn record_display(hit: &MyChemHit, display: &str, stage: &'static str) {
    let fields = [
        ("ndc", "nonproprietaryname"),
        ("openfda", "generic_name"),
        ("openfda", "brand_name"),
        ("drugbank", "name"),
        ("chembl", "pref_name"),
        ("gtopdb", "name"),
        ("unii", "display_name"),
        ("chebi", "name"),
    ];
    let claims = hit.row.identity().claims();
    let selected = fields.into_iter().find_map(|(section, field)| {
        claims
            .iter()
            .enumerate()
            .find(|(_, claim)| {
                claim.origin().section() == section && claim.origin().field() == field
            })
            .map(|(index, _)| index)
    });
    for (index, claim) in claims.iter().enumerate() {
        if !matches!(claim.value(), biodata::DrugClaimValue::Term(_)) {
            continue;
        }
        let winner = Some(index) == selected;
        hit.record(
            Some(index),
            stage,
            if winner {
                "select_display"
            } else {
                "omit_display"
            },
            if winner {
                "display precedence; trim, strip edge dots and ASCII lowercase"
            } else {
                "first accessor or higher priority source supplies display"
            },
            winner.then(|| display.to_owned()),
        );
    }
}

pub fn select_hits_for_name<'a>(hits: &'a [MyChemHit], name: &str) -> Vec<&'a MyChemHit> {
    let target = normalize_name(name);
    let mut out: Vec<&MyChemHit> = hits
        .iter()
        .filter(|h| {
            hit_all_names(h)
                .iter()
                .any(|n| name_matches_requested(n, &target))
        })
        .collect();

    if out.is_empty() {
        out = hits.iter().collect();
    }

    // Prefer richer hits first (more sources).
    out.sort_by_key(|h| {
        let mut score: i32 = 0;
        if h.drugbank.as_ref().and_then(|d| d.id.as_deref()).is_some() {
            score -= 100;
        }
        if h.chembl
            .as_ref()
            .and_then(|c| c.molecule_chembl_id.as_deref())
            .is_some()
        {
            score -= 50;
        }
        if unii_id(h).is_some() {
            score -= 25;
        }
        if ndc_nonproprietaryname(h).is_some() {
            score -= 10;
        }
        score
    });

    for hit in hits {
        hit.record_row(
            "get selection",
            if out.iter().any(|selected| std::ptr::eq(*selected, hit)) {
                "select"
            } else {
                "omit"
            },
            "name matching, all-hit fallback and stable richness order",
        );
    }
    out
}

fn record_codes(hit: &MyChemHit, section: &str, field: &str, available: bool, stage: &'static str) {
    let mut first = available;
    for (index, claim) in hit.row.identity().claims().iter().enumerate() {
        if claim.origin().section() != section || claim.origin().field() != field {
            continue;
        }
        let target = if let biodata::DrugClaimValue::Code(code) = claim.value() {
            first.then(|| code.value().trim().to_owned())
        } else {
            None
        };
        hit.record(
            Some(index),
            stage,
            if first {
                "select_first_code"
            } else {
                "omit_code"
            },
            if first {
                "first identifier; trim lexical code"
            } else {
                "first code already selected; occurrence retained"
            },
            target,
        );
        first = false;
    }
}

pub fn merge_mychem_hits(hits: &[&MyChemHit], requested_name: &str) -> Drug {
    let mut name = hits
        .iter()
        .find_map(|hit| best_name_from_hit(hit))
        .unwrap_or_else(|| normalize_name(requested_name));
    if let Some(hit) = hits.iter().find(|hit| best_name_from_hit(hit).is_some()) {
        record_display(hit, &name, "get merge");
    }
    let mut drugbank_id: Option<String> = None;
    let mut chembl_id: Option<String> = None;
    let mut unii: Option<String> = None;
    let mut drug_type: Option<String> = None;
    let mut mechanisms: Vec<String> = Vec::new();
    let mut mechanisms_seen: HashSet<String> = HashSet::new();
    let mut brand_names: Vec<String> = Vec::new();
    let mut brand_names_seen: HashSet<String> = HashSet::new();
    let mut ddinter_synonyms: Vec<String> = Vec::new();
    // Only the chosen anchor hit feeds the synonym fold (ticket 1254):
    // a pooled combination-product synonym would widen DDInter identity
    // to a real interaction partner, and interactions aggregation then
    // skips that row silently because both sides match the anchor.
    let anchor_index = hits
        .iter()
        .position(|hit| best_name_from_hit(hit).is_some());

    let mut targets: Vec<String> = Vec::new();
    let mut indications: Vec<String> = Vec::new();
    let mut pharm_classes: Vec<String> = Vec::new();
    let mut interactions: Vec<DrugInteraction> = Vec::new();

    let mut targets_seen: HashSet<String> = HashSet::new();
    let mut indications_seen: HashSet<String> = HashSet::new();
    let mut classes_seen: HashSet<String> = HashSet::new();
    let mut interactions_seen: HashSet<String> = HashSet::new();
    let mut approval_date: Option<String> = None;

    for (hit_index, hit) in hits.iter().enumerate() {
        if let Some(chembl) = &hit.chembl {
            for index in 0..chembl.atc_classifications.clone().into_vec().len() {
                hit.record_source(&format!("/chembl/atc_classifications/{index}"), "enrichment", "omit_get_atc", "ATC remains source-only; Get profile does not request it and no merged Drug field consumes it", None);
            }
        }
        if name.is_empty()
            && let Some(n) = best_name_from_hit(hit)
        {
            name = n;
        }

        record_codes(hit, "drugbank", "id", drugbank_id.is_none(), "get merge");
        record_codes(
            hit,
            "chembl",
            "molecule_chembl_id",
            chembl_id.is_none(),
            "get merge",
        );
        record_codes(hit, "unii", "unii", unii.is_none(), "get merge");
        if drugbank_id.is_none() {
            drugbank_id = hit
                .drugbank
                .as_ref()
                .and_then(|d| d.id.clone())
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty());
        }
        if chembl_id.is_none() {
            chembl_id = hit
                .chembl
                .as_ref()
                .and_then(|c| c.molecule_chembl_id.clone())
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty());
        }
        if unii.is_none() {
            unii = unii_id(hit)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty());
        }
        if drug_type.is_none() {
            drug_type = drug_type_from_hit(hit);
        }
        if approval_date.is_none() {
            approval_date = approval_date_from_hit(hit);
        }

        if Some(hit_index) == anchor_index
            && let Some(drugbank) = hit.drugbank.as_ref()
        {
            for (synonym_index, synonym) in drugbank.synonyms.iter().enumerate() {
                let claim_index = hit
                    .row
                    .identity()
                    .claims()
                    .iter()
                    .enumerate()
                    .filter(|(_, claim)| {
                        claim.origin().section() == "drugbank"
                            && claim.origin().field() == "synonyms"
                    })
                    .nth(synonym_index)
                    .map(|(index, _)| index);
                let synonym = synonym.trim();
                if synonym.is_empty() {
                    continue;
                }
                if synonym.eq_ignore_ascii_case(&name)
                    || synonym.eq_ignore_ascii_case(requested_name)
                {
                    hit.record(
                        claim_index,
                        "get merge synonyms",
                        "omit",
                        "equals requested or displayed name",
                        None,
                    );
                    continue;
                }
                let key = synonym.to_ascii_lowercase();
                if !brand_names_seen.insert(key.clone()) {
                    hit.record(
                        claim_index,
                        "get merge synonyms",
                        "deduplicate",
                        "case insensitive duplicate",
                        None,
                    );
                    continue;
                }
                // The full synonym list feeds DDInter identity matching
                // (ticket 1241); the card keeps the three-brand cap, so
                // the loop does not break here.
                hit.record(
                    claim_index,
                    "DDInter synonyms",
                    if ddinter_synonyms.len() < 32 {
                        "select"
                    } else {
                        "cap"
                    },
                    "anchor-only 32-synonym policy",
                    (ddinter_synonyms.len() < 32).then(|| synonym.to_owned()),
                );
                hit.record(
                    claim_index,
                    "card brands",
                    if brand_names.len() < 3 {
                        "select"
                    } else {
                        "cap"
                    },
                    "three-brand card policy",
                    (brand_names.len() < 3).then(|| synonym.to_owned()),
                );
                if ddinter_synonyms.len() < 32 {
                    ddinter_synonyms.push(synonym.to_string());
                }
                if brand_names.len() < 3 {
                    brand_names.push(synonym.to_string());
                }
            }
        }

        if mechanisms.len() < 3 {
            for mechanism in chembl_mechanisms_from_hit(hit) {
                let key = mechanism.to_ascii_lowercase();
                if !mechanisms_seen.insert(key) {
                    continue;
                }
                mechanisms.push(mechanism);
                if mechanisms.len() >= 3 {
                    break;
                }
            }
        }

        if let Some(gtopdb) = hit.gtopdb.as_ref() {
            for (index, t) in gtopdb.interaction_targets.iter().enumerate() {
                let Some(sym) = t.symbol.as_deref().map(str::trim).filter(|v| !v.is_empty()) else {
                    continue;
                };
                let sym = sym.to_string();
                let selected = targets_seen.insert(sym.clone());
                hit.record_source(
                    &format!("/gtopdb/interaction_targets/{index}/symbol"),
                    "enrichment",
                    if selected {
                        "select_target"
                    } else {
                        "deduplicate_target"
                    },
                    if selected {
                        "first unique trimmed GtoPdb symbol"
                    } else {
                        "duplicate trimmed GtoPdb symbol"
                    },
                    selected.then(|| sym.clone()),
                );
                if selected {
                    targets.push(sym);
                }
            }
        }

        if let Some(dc) = hit.drugcentral.as_ref()
            && let Some(use_) = dc.drug_use.as_ref()
        {
            for (index, ind) in use_.indication.iter().enumerate() {
                let Some(name) = ind
                    .concept_name
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                else {
                    continue;
                };
                let key = name.to_ascii_lowercase();
                let selected = indications_seen.insert(key);
                hit.record_source(
                    &format!("/drugcentral/drug_use/indication/{index}/concept_name"),
                    "enrichment",
                    if selected {
                        "select_indication"
                    } else {
                        "deduplicate_indication"
                    },
                    if selected {
                        "first unique trimmed indication name"
                    } else {
                        "duplicate case-insensitive indication"
                    },
                    selected.then(|| name.to_owned()),
                );
                if selected {
                    indications.push(name.to_string());
                }
            }
        }

        for (pointer, cls) in enrichment::moa_class_occurrences(hit) {
            let key = cls.to_ascii_lowercase();
            let selected = classes_seen.insert(key);
            hit.record_source(
                &pointer,
                "enrichment",
                if selected {
                    "strip_moa_suffix"
                } else {
                    "deduplicate_class"
                },
                if selected {
                    "pharmacologic class keeps text before [MoA]"
                } else {
                    "duplicate normalized pharmacologic class"
                },
                selected.then(|| cls.clone()),
            );
            if selected {
                pharm_classes.push(cls);
            }
        }

        if interactions.len() < 15 {
            for row in interactions_from_hit(hit) {
                let key = row.drug.to_ascii_lowercase();
                if !interactions_seen.insert(key) {
                    continue;
                }
                interactions.push(row);
                if interactions.len() >= 15 {
                    break;
                }
            }
        }
    }

    targets.sort();
    indications.sort();
    brand_names.sort();
    interactions.sort_by(|a, b| a.drug.cmp(&b.drug));
    pharm_classes.truncate(6);
    targets.truncate(8);
    indications.truncate(6);
    brand_names.truncate(3);

    if mechanisms.is_empty() {
        for hit in hits {
            if let Some(mechanism) = fallback_mechanism_from_hit(hit) {
                mechanisms.push(mechanism);
                break;
            }
        }
    }

    let mechanism = mechanisms.first().cloned();
    let approval_date_raw = approval_date.clone();
    let approval_date_display = approval_date_raw
        .as_deref()
        .and_then(approval_date_display)
        .or_else(|| approval_date_raw.clone());
    let approval_summary = approval_summary(approval_date_display.as_deref());

    Drug {
        section_outcomes: crate::entities::drug::default_drug_section_outcomes(),
        name,
        drugbank_id,
        chembl_id,
        unii,
        drug_type,
        mechanism,
        mechanisms,
        approval_date,
        approval_date_raw,
        approval_date_display,
        approval_summary,
        brand_names,
        route: None,
        targets,
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications,
        interactions,
        interaction_text: None,
        interaction_pagination: None,
        interaction_bundle_freshness: None,
        interaction_coverage_status: None,
        ddinter_synonyms,
        pharm_classes,
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        fda_orphan_designations: None,
        us_safety_warnings: None,
        us_boxed_warning: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
        cell_lines: None,
    }
}

#[cfg(test)]
mod tests;
