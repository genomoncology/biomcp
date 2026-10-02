//! OpenFDA fallback result selection preserves original label occurrence.
use super::*;
pub(in crate::entities::drug) struct LabelOrigin {
    pub(in crate::entities::drug) result_index: usize,
    pub(in crate::entities::drug) field: &'static str,
    pub(in crate::entities::drug) value_index: Option<usize>,
    pub(in crate::entities::drug) lexical_text: String,
}
pub(in crate::entities::drug) fn search_results_from_openfda_label_response(
    value: &serde_json::Value,
    query: &str,
    max_results: usize,
) -> Vec<DrugSearchResult> {
    search_results_from_openfda_label_response_with_origin(value, query, max_results)
        .into_iter()
        .map(|(row, _)| row)
        .collect()
}

pub(in crate::entities::drug) fn search_results_from_openfda_label_response_with_origin(
    label_response: &serde_json::Value,
    query: &str,
    max_results: usize,
) -> Vec<(DrugSearchResult, LabelOrigin)> {
    let query = query.trim();
    if query.is_empty() || max_results == 0 {
        return Vec::new();
    }

    let Some(results) = label_response.get("results").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let mut exact_matches: Vec<(DrugSearchResult, LabelOrigin)> = Vec::new();
    let mut others: Vec<(DrugSearchResult, LabelOrigin)> = Vec::new();
    for (result_index, result) in results.iter().enumerate() {
        let brand_names = extract_openfda_values_from_result(result, "brand_name");
        let generic_names = extract_openfda_values_from_result(result, "generic_name");
        let Some(name) = generic_names
            .first()
            .cloned()
            .or_else(|| brand_names.first().cloned())
        else {
            continue;
        };
        let field = if generic_names.is_empty() {
            "brand_name"
        } else {
            "generic_name"
        };
        let raw = &result["openfda"][field];
        let (value_index, lexical_text) = if let Some(values) = raw.as_array() {
            let Some((index, value)) = values
                .iter()
                .enumerate()
                .find(|(_, value)| value.as_str().is_some_and(|value| value.trim() == name))
            else {
                continue;
            };
            (Some(index), value.as_str().unwrap_or_default().to_owned())
        } else {
            (None, raw.as_str().unwrap_or_default().to_owned())
        };
        let origin = LabelOrigin {
            result_index,
            field,
            value_index,
            lexical_text,
        };
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }

        let row = DrugSearchResult {
            name,
            drugbank_id: None,
            drug_type: None,
            mechanism: None,
            target: None,
        };
        let is_exact_brand_match = brand_names
            .iter()
            .map(|value| value.trim())
            .any(|value| value.eq_ignore_ascii_case(query));
        if is_exact_brand_match {
            exact_matches.push((row, origin));
        } else {
            others.push((row, origin));
        }
    }

    let mut out: Vec<(DrugSearchResult, LabelOrigin)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (row, origin) in exact_matches.into_iter().chain(others) {
        if !seen.insert(row.name.clone()) {
            continue;
        }
        out.push((row, origin));
        if out.len() >= max_results {
            break;
        }
    }
    out
}
