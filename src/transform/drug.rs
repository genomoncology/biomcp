use std::collections::HashSet;

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

fn best_name_from_hit(hit: &MyChemHit) -> Option<String> {
    let candidates: [Option<&str>; 6] = [
        ndc_nonproprietaryname(hit),
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
    if let Some(v) = hit.drugbank.as_ref().and_then(|d| d.name.as_deref()) {
        out.push(normalize_name(v));
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

fn drug_type_from_hit(hit: &MyChemHit) -> Option<String> {
    let v = hit
        .chembl
        .as_ref()
        .and_then(|c| c.molecule_type.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_ascii_lowercase());

    match v.as_deref() {
        Some("antibody") => Some("biologic".into()),
        Some("small molecule") => Some("small-molecule".into()),
        Some(other) => Some(other.to_string()),
        None => None,
    }
}

fn title_case_words(value: &str) -> String {
    value
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let first = first.to_uppercase().collect::<String>();
            let rest = chars.as_str().to_ascii_lowercase();
            format!("{first}{rest}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_action_type(action: &str) -> String {
    let v = action.trim().replace('_', " ");
    if v.is_empty() {
        return String::new();
    }
    if v.chars().any(|c| c.is_ascii_lowercase()) {
        return v;
    }
    title_case_words(&v.to_ascii_lowercase())
}

fn chembl_mechanisms_from_hit(hit: &MyChemHit) -> Vec<String> {
    let Some(chembl) = hit.chembl.as_ref() else {
        return Vec::new();
    };

    let mut out: Vec<String> = Vec::new();
    for mech in &chembl.drug_mechanisms {
        if let Some(mechanism) = mech
            .mechanism_of_action
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
        {
            out.push(mechanism);
            continue;
        }

        let action = mech
            .action_type
            .as_deref()
            .map(normalize_action_type)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let target = mech
            .target_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty());
        if action.is_none() && target.is_none() {
            continue;
        }

        let mechanism = match (action, target) {
            (Some(a), Some(t)) => format!("{a} of {t}"),
            (Some(a), None) => a,
            (None, Some(t)) => t.to_string(),
            (None, None) => continue,
        };
        out.push(mechanism);
    }
    out
}

fn fallback_mechanism_from_hit(hit: &MyChemHit) -> Option<String> {
    moa_pharm_classes(hit).into_iter().next()
}

fn normalize_approval_date(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    if v.len() == 10 {
        return Some(v.to_string());
    }
    if v.len() == 8 && v.chars().all(|c| c.is_ascii_digit()) {
        return Some(format!("{}-{}-{}", &v[0..4], &v[4..6], &v[6..8]));
    }
    None
}

fn approval_date_from_hit(hit: &MyChemHit) -> Option<String> {
    let approvals = hit.drugcentral.as_ref().map(|d| &d.approval)?;
    let mut fda_dates: Vec<String> = approvals
        .iter()
        .filter(|a| {
            a.agency
                .as_deref()
                .map(str::trim)
                .is_some_and(|v| v.eq_ignore_ascii_case("FDA"))
        })
        .filter_map(|a| a.date.as_deref())
        .filter_map(normalize_approval_date)
        .collect();

    if !fda_dates.is_empty() {
        fda_dates.sort();
        return fda_dates.first().cloned();
    }

    let mut any_dates = approvals
        .iter()
        .filter_map(|a| a.date.as_deref())
        .filter_map(normalize_approval_date)
        .collect::<Vec<_>>();
    any_dates.sort();
    any_dates.first().cloned()
}

fn first_target_from_hit(hit: &MyChemHit) -> Option<String> {
    if let Some(gtopdb) = hit.gtopdb.as_ref() {
        for target in &gtopdb.interaction_targets {
            let Some(symbol) = target
                .symbol
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            return Some(symbol.to_string());
        }
    }

    let chembl = hit.chembl.as_ref()?;
    for mechanism in &chembl.drug_mechanisms {
        let Some(target) = mechanism
            .target_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            continue;
        };
        return Some(target.to_string());
    }

    None
}

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
    for row in &drugbank.drug_interactions {
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
        out.push(DrugInteraction { drug, description });
    }

    out
}

pub fn from_mychem_search_hit(hit: &MyChemHit) -> Option<DrugSearchResult> {
    let name = best_name_from_hit(hit)?;
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

    Some(DrugSearchResult {
        name,
        drugbank_id,
        drug_type: drug_type_from_hit(hit),
        mechanism,
        target,
    })
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

    out
}

pub fn merge_mychem_hits(hits: &[&MyChemHit], requested_name: &str) -> Drug {
    let mut name = normalize_name(requested_name);
    let mut drugbank_id: Option<String> = None;
    let mut chembl_id: Option<String> = None;
    let mut unii: Option<String> = None;
    let mut drug_type: Option<String> = None;
    let mut mechanisms: Vec<String> = Vec::new();
    let mut mechanisms_seen: HashSet<String> = HashSet::new();
    let mut brand_names: Vec<String> = Vec::new();
    let mut brand_names_seen: HashSet<String> = HashSet::new();

    let mut targets: Vec<String> = Vec::new();
    let mut indications: Vec<String> = Vec::new();
    let mut pharm_classes: Vec<String> = Vec::new();
    let mut interactions: Vec<DrugInteraction> = Vec::new();

    let mut targets_seen: HashSet<String> = HashSet::new();
    let mut indications_seen: HashSet<String> = HashSet::new();
    let mut classes_seen: HashSet<String> = HashSet::new();
    let mut interactions_seen: HashSet<String> = HashSet::new();
    let mut approval_date: Option<String> = None;

    for hit in hits {
        if name.is_empty()
            && let Some(n) = best_name_from_hit(hit)
        {
            name = n;
        }

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

        if let Some(drugbank) = hit.drugbank.as_ref() {
            for synonym in &drugbank.synonyms {
                let synonym = synonym.trim();
                if synonym.is_empty() {
                    continue;
                }
                if synonym.eq_ignore_ascii_case(&name)
                    || synonym.eq_ignore_ascii_case(requested_name)
                {
                    continue;
                }
                let key = synonym.to_ascii_lowercase();
                if !brand_names_seen.insert(key) {
                    continue;
                }
                brand_names.push(synonym.to_string());
                if brand_names.len() >= 3 {
                    break;
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
            for t in &gtopdb.interaction_targets {
                let Some(sym) = t.symbol.as_deref().map(str::trim).filter(|v| !v.is_empty()) else {
                    continue;
                };
                let sym = sym.to_string();
                if targets_seen.insert(sym.clone()) {
                    targets.push(sym);
                }
            }
        }

        if let Some(dc) = hit.drugcentral.as_ref()
            && let Some(use_) = dc.drug_use.as_ref()
        {
            for ind in &use_.indication {
                let Some(name) = ind
                    .concept_name
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                else {
                    continue;
                };
                let key = name.to_ascii_lowercase();
                if indications_seen.insert(key) {
                    indications.push(name.to_string());
                }
            }
        }

        for cls in moa_pharm_classes(hit) {
            let key = cls.to_ascii_lowercase();
            if classes_seen.insert(key) {
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

    Drug {
        name,
        drugbank_id,
        chembl_id,
        unii,
        drug_type,
        mechanism,
        mechanisms,
        approval_date,
        brand_names,
        route: None,
        targets,
        indications,
        interactions,
        pharm_classes,
        top_adverse_events: Vec::new(),
        label: None,
        shortage: None,
        approvals: None,
        civic: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_mychem_hits_collects_deduped_mechanisms() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "1",
            "_score": 1.0,
            "chembl": {
                "molecule_chembl_id": "CHEMBL1",
                "molecule_type": "Small molecule",
                "pref_name": "test",
                "drug_mechanisms": [
                    {"action_type": "INHIBITOR", "target_name": "BRAF"},
                    {"action_type": "INHIBITOR", "target_name": "BRAF"},
                    {"action_type": "AGONIST", "target_name": "TP53"},
                    {"action_type": "ANTAGONIST", "target_name": "EGFR"},
                    {"action_type": "BLOCKER", "target_name": "ALK"}
                ]
            }
        }))
        .expect("valid JSON");

        let drug = merge_mychem_hits(&[&hit], "test");
        assert_eq!(drug.mechanisms.len(), 3, "mechanisms should be limited");
        assert_eq!(drug.mechanisms[0], "Inhibitor of BRAF");
        assert_eq!(drug.mechanisms[1], "Agonist of TP53");
        assert_eq!(drug.mechanisms[2], "Antagonist of EGFR");
        assert_eq!(drug.mechanism.as_deref(), Some("Inhibitor of BRAF"));
    }

    #[test]
    fn select_hits_for_name_matches_salt_forms() {
        let base: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "1",
            "_score": 1.0,
            "drugbank": {"name": "Dabrafenib"}
        }))
        .expect("valid base hit");

        let salt: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "2",
            "_score": 1.0,
            "chembl": {
                "pref_name": "DABRAFENIB MESYLATE",
                "molecule_chembl_id": "CHEMBL2105729",
                "drug_mechanisms": [
                    {"action_type": "INHIBITOR", "target_name": "BRAF"}
                ]
            }
        }))
        .expect("valid salt hit");

        let hits = [base, salt];
        let selected = select_hits_for_name(&hits, "dabrafenib");
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn merge_mychem_hits_collects_drug_interactions() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "1",
            "_score": 1.0,
            "drugbank": {
                "id": "DB0001",
                "name": "warfarin",
                "drug_interactions": [
                    {"name": "Aspirin", "description": "May increase bleeding risk."},
                    {"name": "Clopidogrel", "description": "Monitor for bleeding."}
                ]
            }
        }))
        .expect("valid JSON");

        assert_eq!(
            hit.drugbank
                .as_ref()
                .map(|d| d.drug_interactions.len())
                .unwrap_or_default(),
            2
        );
        let drug = merge_mychem_hits(&[&hit], "warfarin");
        assert_eq!(drug.interactions.len(), 2);
        assert_eq!(drug.interactions[0].drug, "Aspirin");
    }

    #[test]
    fn drug_sections_maps_osimertinib() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "DB09330",
            "_score": 1.0,
            "drugbank": {"id": "DB09330", "name": "osimertinib"},
            "chembl": {
                "molecule_chembl_id": "CHEMBL3353410",
                "molecule_type": "Small molecule",
                "pref_name": "OSIMERTINIB",
                "drug_mechanisms": [
                    {"action_type": "INHIBITOR", "target_name": "EGFR"}
                ]
            },
            "gtopdb": {
                "interaction_targets": [{"symbol": "EGFR"}]
            },
            "drugcentral": {
                "approval": [{"agency": "FDA", "date": "20151113"}],
                "drug_use": {"indication": [{"concept_name": "Non-small cell lung cancer"}]}
            }
        }))
        .expect("valid osimertinib hit");

        let drug = merge_mychem_hits(&[&hit], "osimertinib");
        assert_eq!(drug.name, "osimertinib");
        assert_eq!(drug.targets.first().map(String::as_str), Some("EGFR"));
        assert_eq!(drug.drug_type.as_deref(), Some("small-molecule"));
        assert_eq!(drug.approval_date.as_deref(), Some("2015-11-13"));
    }

    #[test]
    fn drug_sections_maps_imatinib() {
        let hit: MyChemHit = serde_json::from_value(serde_json::json!({
            "_id": "DB00619",
            "_score": 1.0,
            "drugbank": {"id": "DB00619", "name": "imatinib"},
            "chembl": {
                "molecule_chembl_id": "CHEMBL941",
                "molecule_type": "Small molecule",
                "pref_name": "IMATINIB",
                "drug_mechanisms": [
                    {"action_type": "INHIBITOR", "target_name": "ABL1"}
                ]
            },
            "gtopdb": {
                "interaction_targets": [{"symbol": "ABL1"}]
            }
        }))
        .expect("valid imatinib hit");

        let drug = merge_mychem_hits(&[&hit], "imatinib");
        assert_eq!(drug.name, "imatinib");
        assert_eq!(drug.targets.first().map(String::as_str), Some("ABL1"));
        assert!(
            drug.mechanism
                .as_deref()
                .is_some_and(|v| v.to_ascii_lowercase().contains("inhibitor"))
        );
    }
}
