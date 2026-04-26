//! Search-all markdown row formatting, JSON conversion, and result refinement.

use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;

use crate::error::BioMcpError;

use super::SearchAllSection;

pub(super) fn markdown_columns(section: &SearchAllSection) -> &'static [&'static str] {
    match section.entity.as_str() {
        "gene" => &["Symbol", "Name", "Entrez"],
        "variant" => &["ID", "Gene", "Protein", "Significance"],
        "disease" => &["ID", "Name", "Synonyms"],
        "drug" => &["Name", "Target", "Mechanism"],
        "trial" => &["NCT", "Title", "Status"],
        "article" => &["PMID", "Title", "Date"],
        "pathway" => &["ID", "Name"],
        "pgx" => &["Gene", "Drug", "CPIC"],
        "gwas" => &["rsID", "Trait", "P-Value"],
        "adverse-event" => &["Reaction", "Count"],
        _ => &[],
    }
}

pub(super) fn markdown_rows(section: &SearchAllSection) -> Vec<Vec<String>> {
    section
        .results
        .iter()
        .filter_map(|row| match section.entity.as_str() {
            "gene" => Some(vec![
                value_str(row, "symbol"),
                value_str(row, "name"),
                value_str(row, "entrez_id"),
            ]),
            "variant" => {
                let rendered = vec![
                    value_str(row, "id"),
                    value_str(row, "gene"),
                    value_str(row, "hgvs_p"),
                    value_str(row, "significance"),
                ];
                let uninformative = rendered.get(1).is_some_and(|cell| is_empty_cell(cell))
                    && rendered.get(2).is_some_and(|cell| is_empty_cell(cell))
                    && rendered.get(3).is_some_and(|cell| is_empty_cell(cell));
                (!uninformative).then_some(rendered)
            }
            "disease" => Some(vec![
                value_str(row, "id"),
                value_str(row, "name"),
                value_str(row, "synonyms_preview"),
            ]),
            "drug" => Some(vec![
                value_str(row, "name"),
                value_str(row, "target"),
                value_str_or(row, "mechanism", value_str(row, "drug_type")),
            ]),
            "trial" => Some(vec![
                value_str(row, "nct_id"),
                value_str(row, "title"),
                value_str(row, "status"),
            ]),
            "article" => Some(vec![
                value_str(row, "pmid"),
                value_str(row, "title"),
                value_str(row, "date"),
            ]),
            "pathway" => Some(vec![value_str(row, "id"), value_str(row, "name")]),
            "pgx" => Some(vec![
                value_str(row, "genesymbol"),
                value_str(row, "drugname"),
                value_str(row, "cpiclevel"),
            ]),
            "gwas" => Some(vec![
                value_str(row, "rsid"),
                value_str(row, "trait_name"),
                value_p_value(row, "p_value"),
            ]),
            "adverse-event" => Some(vec![value_str(row, "reaction"), value_str(row, "count")]),
            _ => Some(vec![format_value(row)]),
        })
        .collect()
}

pub(super) fn to_json_array<T: Serialize>(rows: Vec<T>) -> Result<Vec<Value>, BioMcpError> {
    let value = serde_json::to_value(rows)?;
    Ok(value.as_array().cloned().unwrap_or_default())
}

fn value_str(row: &Value, key: &str) -> String {
    let Some(obj) = row.as_object() else {
        return "-".to_string();
    };
    let Some(value) = obj.get(key) else {
        return "-".to_string();
    };
    format_value(value)
}

fn value_str_or(row: &Value, primary: &str, fallback: String) -> String {
    let value = value_str(row, primary);
    if value == "-" { fallback } else { value }
}

fn value_p_value(row: &Value, key: &str) -> String {
    let Some(obj) = row.as_object() else {
        return "-".to_string();
    };
    let Some(value) = obj.get(key) else {
        return "-".to_string();
    };
    format_search_all_p_value(value)
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                "-".to_string()
            } else {
                trimmed.to_string()
            }
        }
        Value::Array(values) => {
            let joined = values
                .iter()
                .take(3)
                .map(format_value)
                .filter(|v| v != "-")
                .collect::<Vec<_>>()
                .join("; ");
            if joined.is_empty() {
                "-".to_string()
            } else {
                joined
            }
        }
        Value::Object(_) => value.to_string(),
    }
}

fn is_empty_cell(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.is_empty() || trimmed == "-"
}

pub(super) fn format_search_all_p_value(value: &Value) -> String {
    let parsed = match value {
        Value::Number(v) => v.as_f64(),
        Value::String(v) => v.trim().parse::<f64>().ok(),
        _ => None,
    };
    let Some(mut parsed) = parsed else {
        return format_value(value);
    };
    if !parsed.is_finite() {
        return format_value(value);
    }
    if parsed == -0.0 {
        parsed = 0.0;
    }
    if parsed == 0.0 {
        return "0".to_string();
    }
    if parsed.abs() < 0.001 {
        return trim_scientific_notation(parsed);
    }
    if parsed.abs() < 0.01 {
        return trim_trailing_decimal_zeros(format!("{parsed:.4}"));
    }
    trim_trailing_decimal_zeros(format!("{parsed:.3}"))
}

fn trim_scientific_notation(value: f64) -> String {
    let rendered = format!("{value:.2e}");
    let Some((mantissa, exponent)) = rendered.split_once('e') else {
        return rendered;
    };
    let mantissa = trim_trailing_decimal_zeros(mantissa.to_string());
    format!("{mantissa}e{exponent}")
}

fn trim_trailing_decimal_zeros(mut rendered: String) -> String {
    if rendered.contains('.') {
        while rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.pop();
        }
    }
    if rendered.is_empty() {
        "0".to_string()
    } else {
        rendered
    }
}

pub(super) fn is_civic_variant_id(id: &str) -> bool {
    id.trim().to_ascii_uppercase().starts_with("CIVIC_VARIANT:")
}

pub(super) fn first_gettable_variant_id(results: &[Value]) -> Option<String> {
    results
        .iter()
        .filter_map(|row| row.as_object())
        .filter_map(|obj| obj.get("id"))
        .filter_map(Value::as_str)
        .map(str::trim)
        .find(|id| !id.is_empty() && !is_civic_variant_id(id))
        .map(str::to_string)
}

pub(super) fn refine_drug_results(
    mut rows: Vec<crate::entities::drug::DrugSearchResult>,
    preferred: Option<&str>,
    limit: usize,
) -> Vec<crate::entities::drug::DrugSearchResult> {
    let Some(preferred_lower) = preferred
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
    else {
        rows.truncate(limit);
        return rows;
    };

    rows.sort_by_key(|row| {
        crate::render::markdown::drug_parent_match_rank(&row.name, &preferred_lower)
            .unwrap_or(u8::MAX)
    });

    let has_parent_like = rows.iter().any(|row| {
        crate::render::markdown::drug_parent_match_rank(&row.name, &preferred_lower)
            .is_some_and(|rank| rank < 3)
    });

    if has_parent_like {
        rows.retain(|row| {
            let normalized = row.name.trim().to_ascii_lowercase();
            if !normalized.contains(&preferred_lower) {
                return true;
            }
            crate::render::markdown::drug_parent_match_rank(&row.name, &preferred_lower)
                .is_some_and(|rank| rank < 3)
        });
    }

    rows.truncate(limit);
    rows
}

pub(super) fn variant_significance_rank(significance: Option<&str>) -> u8 {
    let Some(significance) = significance.map(str::trim).filter(|v| !v.is_empty()) else {
        return 50;
    };
    let normalized = significance.to_ascii_lowercase();
    if normalized == "pathogenic"
        || (normalized.contains("pathogenic") && !normalized.contains("likely"))
    {
        return 0;
    }
    if normalized.contains("likely pathogenic") {
        return 1;
    }
    if normalized == "vus"
        || normalized.contains("uncertain")
        || normalized.contains("unknown significance")
    {
        return 2;
    }
    if normalized.contains("likely benign") {
        return 3;
    }
    if normalized == "benign" || normalized.contains("benign") {
        return 4;
    }
    5
}

pub(super) fn trait_matches_disease_query(
    row: &crate::entities::variant::VariantGwasAssociation,
    disease: &str,
) -> bool {
    let disease = disease.trim();
    if disease.is_empty() {
        return true;
    }
    row.trait_name
        .as_deref()
        .map(str::trim)
        .filter(|trait_name| !trait_name.is_empty())
        .is_some_and(|trait_name| {
            trait_name
                .to_ascii_lowercase()
                .contains(&disease.to_ascii_lowercase())
        })
}

pub(super) fn dedupe_gwas_rows(
    rows: Vec<crate::entities::variant::VariantGwasAssociation>,
) -> Vec<crate::entities::variant::VariantGwasAssociation> {
    let mut out: Vec<crate::entities::variant::VariantGwasAssociation> = Vec::new();
    let mut index_by_key: HashMap<(String, String), usize> = HashMap::new();

    for row in rows {
        let rsid_key = row.rsid.trim().to_ascii_lowercase();
        let trait_key = row
            .trait_name
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_ascii_lowercase();

        if rsid_key.is_empty() || trait_key.is_empty() {
            out.push(row);
            continue;
        }

        let key = (rsid_key, trait_key);
        if let Some(existing_idx) = index_by_key.get(&key).copied() {
            let existing_p = out[existing_idx].p_value.unwrap_or(f64::INFINITY);
            let candidate_p = row.p_value.unwrap_or(f64::INFINITY);
            if candidate_p < existing_p {
                out[existing_idx] = row;
            }
            continue;
        }

        index_by_key.insert(key, out.len());
        out.push(row);
    }

    out
}
