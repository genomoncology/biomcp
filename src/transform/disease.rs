use std::collections::{HashMap, HashSet};

use crate::entities::disease::{Disease, DiseasePhenotype, DiseaseSearchResult};
use crate::sources::mydisease::MyDiseaseHit;

fn clean_definition(value: &str) -> String {
    let mut s = value.trim().to_string();
    if s.starts_with('\"') {
        s = s.trim_start_matches('\"').to_string();
    }
    if let Some(idx) = s.find("\" [") {
        s = s[..idx].to_string();
    }
    if s.ends_with('\"') {
        s = s.trim_end_matches('\"').to_string();
    }
    s.trim().to_string()
}

fn first_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.trim().to_string()),
        serde_json::Value::Array(arr) => arr
            .iter()
            .find_map(|v| v.as_str())
            .map(|s| s.trim().to_string()),
        _ => None,
    }
    .filter(|s| !s.is_empty())
}

fn push_unique(out: &mut Vec<String>, seen: &mut HashSet<String>, raw: &str, max: usize) {
    if out.len() >= max {
        return;
    }
    let v = raw.trim();
    if v.is_empty() {
        return;
    }
    let key = v.to_ascii_lowercase();
    if seen.insert(key) {
        out.push(v.to_string());
    }
}

fn collect_strings(
    value: &serde_json::Value,
    out: &mut Vec<String>,
    seen: &mut HashSet<String>,
    max: usize,
) {
    if out.len() >= max {
        return;
    }
    match value {
        serde_json::Value::String(s) => push_unique(out, seen, s, max),
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_strings(v, out, seen, max);
                if out.len() >= max {
                    break;
                }
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                collect_strings(v, out, seen, max);
                if out.len() >= max {
                    break;
                }
            }
        }
        _ => {}
    }
}

fn collect_synonyms(
    mondo: Option<&serde_json::Value>,
    disease_ontology: Option<&serde_json::Value>,
    name: &str,
    max: usize,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let name_key = name.trim().to_ascii_lowercase();

    let mut add_synonym_obj = |value: &serde_json::Value| {
        let Some(obj) = value.as_object() else {
            collect_strings(value, &mut out, &mut seen, max);
            return;
        };

        for key in ["exact", "related", "narrow", "broad"] {
            if let Some(v) = obj.get(key) {
                collect_strings(v, &mut out, &mut seen, max);
            }
        }
        for (k, v) in obj {
            if matches!(k.as_str(), "exact" | "related" | "narrow" | "broad") {
                continue;
            }
            collect_strings(v, &mut out, &mut seen, max);
        }
    };

    if let Some(mondo) = mondo
        && let Some(syn) = mondo.get("synonym")
    {
        add_synonym_obj(syn);
    }

    if let Some(do_term) = disease_ontology
        && let Some(syn) = do_term.get("synonyms")
    {
        add_synonym_obj(syn);
    }

    out.retain(|s| s.trim().to_ascii_lowercase() != name_key);
    out
}

fn collect_parents(
    mondo: Option<&serde_json::Value>,
    disease_ontology: Option<&serde_json::Value>,
    max: usize,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let mut add_parents = |value: &serde_json::Value| {
        let Some(arr) = value.as_array() else { return };
        for v in arr {
            if out.len() >= max {
                break;
            }
            let Some(s) = v.as_str() else { continue };
            push_unique(&mut out, &mut seen, s, max);
        }
    };

    if let Some(mondo) = mondo
        && let Some(parents) = mondo.get("parents")
    {
        add_parents(parents);
    }
    if let Some(do_term) = disease_ontology
        && let Some(parents) = do_term.get("parents")
    {
        add_parents(parents);
    }

    out
}

fn collect_xrefs(
    mondo: Option<&serde_json::Value>,
    disease_ontology: Option<&serde_json::Value>,
    umls: Option<&serde_json::Value>,
) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = HashMap::new();

    if let Some(do_term) = disease_ontology {
        if let Some(doid) = do_term
            .get("doid")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            out.insert("doid".into(), doid.to_string());
        }

        if let Some(xrefs) = do_term.get("xrefs").and_then(|v| v.as_object()) {
            if !out.contains_key("MeSH")
                && let Some(v) = xrefs.get("mesh").and_then(first_string)
            {
                out.insert("MeSH".into(), v);
            }
            if !out.contains_key("NCI")
                && let Some(v) = xrefs
                    .get("ncit")
                    .or_else(|| xrefs.get("nci"))
                    .and_then(first_string)
            {
                out.insert("NCI".into(), v);
            }
            if !out.contains_key("SNOMED")
                && let Some(v) = xrefs
                    .get("snomedct")
                    .or_else(|| xrefs.get("snomedct_us_2023_03_01"))
                    .and_then(first_string)
            {
                out.insert("SNOMED".into(), v);
            }
            if !out.contains_key("ICD-10")
                && let Some(v) = xrefs.get("icd10").and_then(first_string)
            {
                out.insert("ICD-10".into(), v);
            }
        }
    }

    if let Some(mondo) = mondo
        && let Some(xrefs) = mondo.get("xrefs").and_then(|v| v.as_object())
    {
        if !out.contains_key("MeSH")
            && let Some(v) = xrefs.get("mesh").and_then(first_string)
        {
            out.insert("MeSH".into(), v);
        }
        if !out.contains_key("NCI")
            && let Some(v) = xrefs
                .get("ncit")
                .or_else(|| xrefs.get("nci"))
                .and_then(first_string)
        {
            out.insert("NCI".into(), v);
        }
        if !out.contains_key("SNOMED")
            && let Some(v) = xrefs
                .get("sctid")
                .or_else(|| xrefs.get("snomedct"))
                .and_then(first_string)
        {
            out.insert("SNOMED".into(), v);
        }
        if !out.contains_key("ICD-10")
            && let Some(v) = xrefs.get("icd10").and_then(first_string)
        {
            out.insert("ICD-10".into(), v);
        }
        if !out.contains_key("umls_cui")
            && let Some(v) = xrefs.get("umls").and_then(first_string)
        {
            out.insert("umls_cui".into(), v);
        }
    }

    if let Some(umls) = umls {
        if !out.contains_key("MeSH")
            && let Some(v) = umls.get("mesh").and_then(first_string)
        {
            out.insert("MeSH".into(), v);
        }
        if !out.contains_key("NCI")
            && let Some(v) = umls.get("nci").and_then(first_string)
        {
            out.insert("NCI".into(), v);
        }
        if !out.contains_key("SNOMED")
            && let Some(v) = umls.get("snomed").and_then(first_string)
        {
            out.insert("SNOMED".into(), v);
        }
        if !out.contains_key("ICD-10")
            && let Some(v) = umls.get("icd10am").and_then(first_string)
        {
            out.insert("ICD-10".into(), v);
        }
    }

    out
}

fn collect_associated_genes(disgenet: Option<&serde_json::Value>, max: usize) -> Vec<String> {
    let Some(disgenet) = disgenet else {
        return Vec::new();
    };
    let Some(genes) = disgenet
        .get("genes_related_to_disease")
        .and_then(|v| v.as_array())
    else {
        return Vec::new();
    };

    let mut rows: Vec<(String, f64)> = Vec::new();
    for item in genes {
        let Some(obj) = item.as_object() else {
            continue;
        };
        let symbol = obj
            .get("gene_symbol")
            .or_else(|| obj.get("symbol"))
            .or_else(|| obj.get("gene"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let Some(symbol) = symbol else { continue };
        let score = obj
            .get("score")
            .and_then(|v| v.as_f64())
            .unwrap_or_default();
        rows.push((symbol.to_string(), score));
    }

    rows.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (symbol, _) in rows {
        let key = symbol.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(symbol);
        if out.len() >= max {
            break;
        }
    }
    out
}

fn collect_phenotypes(hit: &MyDiseaseHit, max: usize) -> Vec<DiseasePhenotype> {
    let Some(hpo) = hit.hpo.as_ref() else {
        return Vec::new();
    };

    let mut out: Vec<DiseasePhenotype> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for row in &hpo.phenotype_related_to_disease {
        let Some(hpo_id) = row
            .hpo_id
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
        else {
            continue;
        };
        let key = hpo_id.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(DiseasePhenotype {
            hpo_id,
            name: None,
            evidence: row
                .evidence
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
            frequency: row
                .hp_freq
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
            frequency_qualifier: None,
            onset_qualifier: None,
            sex_qualifier: None,
            stage_qualifier: None,
            qualifiers: Vec::new(),
            source: None,
        });
        if out.len() >= max {
            break;
        }
    }
    out
}

fn synonyms_preview(synonyms: &[String]) -> Option<String> {
    if synonyms.is_empty() {
        return None;
    }
    if synonyms.len() == 1 {
        return Some(synonyms[0].clone());
    }
    if synonyms.len() == 2 {
        return Some(format!("{}, {}", synonyms[0], synonyms[1]));
    }
    Some(format!(
        "{}, {} (and {} more)",
        synonyms[0],
        synonyms[1],
        synonyms.len().saturating_sub(2)
    ))
}

pub fn name_from_mydisease_hit(hit: &MyDiseaseHit) -> String {
    hit.disease_ontology
        .as_ref()
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            hit.mondo
                .as_ref()
                .and_then(|v| v.get("name"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| hit.id.clone())
}

pub fn from_mydisease_hit(hit: MyDiseaseHit) -> Disease {
    let name = name_from_mydisease_hit(&hit);

    let definition = hit
        .mondo
        .as_ref()
        .and_then(|v| v.get("definition"))
        .and_then(|v| v.as_str())
        .map(clean_definition)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            hit.disease_ontology
                .as_ref()
                .and_then(|v| v.get("def"))
                .and_then(|v| v.as_str())
                .map(clean_definition)
                .filter(|s| !s.is_empty())
        });

    let synonyms = collect_synonyms(hit.mondo.as_ref(), hit.disease_ontology.as_ref(), &name, 10);
    let parents = collect_parents(hit.mondo.as_ref(), hit.disease_ontology.as_ref(), 10);
    let associated_genes = collect_associated_genes(hit.disgenet.as_ref(), 5);
    let phenotypes = collect_phenotypes(&hit, 30);
    let xrefs = collect_xrefs(
        hit.mondo.as_ref(),
        hit.disease_ontology.as_ref(),
        hit.umls.as_ref(),
    );

    Disease {
        id: hit.id,
        name,
        definition,
        synonyms,
        parents,
        associated_genes,
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes,
        variants: Vec::new(),
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        civic: None,
        xrefs,
    }
}

pub fn from_mydisease_search_hit(hit: &MyDiseaseHit) -> DiseaseSearchResult {
    let name = name_from_mydisease_hit(hit);

    let synonyms = collect_synonyms(hit.mondo.as_ref(), hit.disease_ontology.as_ref(), &name, 10);
    let synonyms_preview = synonyms_preview(&synonyms);

    DiseaseSearchResult {
        id: hit.id.clone(),
        name,
        synonyms_preview,
    }
}

fn first_doid_like(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => {
            let v = s.trim();
            if v.is_empty() {
                return None;
            }
            if v.to_ascii_uppercase().starts_with("DOID:") {
                Some(format!("DOID:{}", v.split(':').nth(1)?.trim()))
            } else if v.chars().all(|c| c.is_ascii_digit()) {
                Some(format!("DOID:{v}"))
            } else {
                None
            }
        }
        serde_json::Value::Array(arr) => arr.iter().find_map(first_doid_like),
        serde_json::Value::Object(map) => {
            if let Some(v) = map.get("doid").and_then(first_doid_like) {
                return Some(v);
            }
            map.values().find_map(first_doid_like)
        }
        _ => None,
    }
}

pub fn doid_from_mydisease_hit(hit: &MyDiseaseHit) -> Option<String> {
    if let Some(v) = hit
        .disease_ontology
        .as_ref()
        .and_then(|o| o.get("doid"))
        .and_then(first_doid_like)
    {
        return Some(v);
    }

    hit.mondo
        .as_ref()
        .and_then(|m| m.get("xrefs"))
        .and_then(|x| x.get("doid").or_else(|| x.get("DOID")))
        .and_then(first_doid_like)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_definition_strips_wrapping_quotes_and_refs() {
        let raw = "\"Example definition.\" [url:http\\://example.com]";
        assert_eq!(clean_definition(raw), "Example definition.");
    }

    #[test]
    fn synonyms_preview_formats_expected() {
        assert_eq!(synonyms_preview(&[]), None);
        assert_eq!(synonyms_preview(&["a".into()]), Some("a".to_string()));
        assert_eq!(
            synonyms_preview(&["a".into(), "b".into()]),
            Some("a, b".to_string())
        );
        assert_eq!(
            synonyms_preview(&["a".into(), "b".into(), "c".into()]),
            Some("a, b (and 1 more)".to_string())
        );
    }

    #[test]
    fn from_mydisease_hit_collects_hpo_phenotypes() {
        let hit: MyDiseaseHit = serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0017309",
            "hpo": {
                "phenotype_related_to_disease": [
                    {"hpo_id": "HP:0001653", "evidence": "TAS", "hp_freq": "HP:0040280"}
                ]
            }
        }))
        .expect("valid hit");

        let disease = from_mydisease_hit(hit);
        assert_eq!(disease.phenotypes.len(), 1);
        assert_eq!(disease.phenotypes[0].hpo_id, "HP:0001653");
        assert_eq!(disease.phenotypes[0].evidence.as_deref(), Some("TAS"));
    }

    #[test]
    fn disease_sections_maps_lung_adenocarcinoma() {
        let hit: MyDiseaseHit = serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0005233",
            "mondo": {
                "name": "lung adenocarcinoma",
                "definition": "\"A lung carcinoma arising from glandular cells.\" [MONDO:ref]",
                "xrefs": {"umls": "C0152018"}
            },
            "disease_ontology": {
                "doid": "DOID:3910",
                "xrefs": {"mesh": "D000077321"}
            },
            "disgenet": {
                "genes_related_to_disease": [
                    {"gene_symbol": "EGFR", "score": 0.9},
                    {"gene_symbol": "KRAS", "score": 0.8}
                ]
            }
        }))
        .expect("valid disease hit");

        let disease = from_mydisease_hit(hit);
        assert_eq!(disease.id, "MONDO:0005233");
        assert_eq!(disease.name, "lung adenocarcinoma");
        assert!(
            disease
                .definition
                .as_deref()
                .is_some_and(|v| v.contains("glandular cells"))
        );
        assert!(disease.associated_genes.contains(&"EGFR".to_string()));
    }

    #[test]
    fn disease_sections_maps_cml() {
        let hit: MyDiseaseHit = serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0011996",
            "mondo": {
                "name": "chronic myeloid leukemia",
                "parents": ["myeloid neoplasm"]
            },
            "disease_ontology": {
                "xrefs": {"ncit": "C3174"}
            },
            "hpo": {
                "phenotype_related_to_disease": [
                    {"hpo_id": "HP:0001878", "evidence": "IEA"}
                ]
            }
        }))
        .expect("valid disease hit");

        let disease = from_mydisease_hit(hit);
        assert_eq!(disease.name, "chronic myeloid leukemia");
        assert!(disease.parents.contains(&"myeloid neoplasm".to_string()));
        assert_eq!(disease.phenotypes.len(), 1);
        assert_eq!(disease.phenotypes[0].hpo_id, "HP:0001878");
    }
}
