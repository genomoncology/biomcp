use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::nci_cts::{NciCtsClient, NciInterventionSearchParams};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionSearchResult {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intervention_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct InterventionSearchFilters {
    pub query: Option<String>,
    pub intervention_type: Option<String>,
    pub category: Option<String>,
    pub code: Option<String>,
}

fn clean_opt(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn json_get_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(v) = value.get(*key).and_then(|v| v.as_str()) {
            let v = v.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn json_get_string_list(value: &serde_json::Value, keys: &[&str], max: usize) -> Vec<String> {
    for key in keys {
        if let Some(v) = value.get(*key) {
            match v {
                serde_json::Value::Array(values) => {
                    let out = values
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .take(max)
                        .map(str::to_string)
                        .collect::<Vec<_>>();
                    if !out.is_empty() {
                        return out;
                    }
                }
                serde_json::Value::String(raw) => {
                    let out = raw
                        .split(',')
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .take(max)
                        .map(str::to_string)
                        .collect::<Vec<_>>();
                    if !out.is_empty() {
                        return out;
                    }
                }
                _ => {}
            }
        }
    }
    Vec::new()
}

fn has_any_filter(filters: &InterventionSearchFilters) -> bool {
    filters
        .query
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
        || filters
            .intervention_type
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .category
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .code
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
}

fn from_nci_row(row: &serde_json::Value) -> Option<InterventionSearchResult> {
    let id = json_get_string(row, &["id", "intervention_id", "interventionId"]).unwrap_or_default();
    let name = json_get_string(row, &["name", "intervention_name", "preferred_name"])
        .unwrap_or_else(|| id.clone());
    if name.is_empty() {
        return None;
    }

    Some(InterventionSearchResult {
        id,
        name,
        intervention_type: clean_opt(
            json_get_string(row, &["type", "intervention_type", "interventionType"]).as_deref(),
        ),
        category: clean_opt(json_get_string(row, &["category"]).as_deref()),
        synonyms: json_get_string_list(row, &["synonyms", "aliases"], 10),
    })
}

pub fn search_query_summary(filters: &InterventionSearchFilters) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = clean_opt(filters.query.as_deref()) {
        parts.push(format!("query={v}"));
    }
    if let Some(v) = clean_opt(filters.intervention_type.as_deref()) {
        parts.push(format!("type={v}"));
    }
    if let Some(v) = clean_opt(filters.category.as_deref()) {
        parts.push(format!("category={v}"));
    }
    if let Some(v) = clean_opt(filters.code.as_deref()) {
        parts.push(format!("code={v}"));
    }
    parts.join(", ")
}

#[allow(dead_code)]
pub async fn search(
    filters: &InterventionSearchFilters,
    limit: usize,
) -> Result<Vec<InterventionSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub async fn search_page(
    filters: &InterventionSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<InterventionSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }
    if !has_any_filter(filters) {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter is required. Example: biomcp search intervention -q pembrolizumab"
                .into(),
        ));
    }

    let client = NciCtsClient::new()?;
    let page = client
        .search_interventions_page(&NciInterventionSearchParams {
            name: filters.query.clone(),
            intervention_type: filters.intervention_type.clone(),
            category: filters.category.clone(),
            codes: filters.code.clone(),
            size: limit,
            from: offset,
        })
        .await?;

    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for row in page.rows {
        let Some(result) = from_nci_row(&row) else {
            continue;
        };
        let key = if result.id.trim().is_empty() {
            format!("name:{}", result.name.to_ascii_lowercase())
        } else {
            format!("id:{}", result.id.to_ascii_lowercase())
        };
        if seen.insert(key) {
            out.push(result);
        }
        if out.len() >= limit {
            break;
        }
    }

    Ok(SearchPage::offset(out, page.total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn search_query_summary_includes_filters() {
        let summary = search_query_summary(&InterventionSearchFilters {
            query: Some("pembrolizumab".into()),
            intervention_type: Some("drug".into()),
            category: Some("agent".into()),
            code: Some("C82416".into()),
        });
        assert_eq!(
            summary,
            "query=pembrolizumab, type=drug, category=agent, code=C82416"
        );
    }

    #[test]
    fn row_normalization_collects_synonyms() {
        let row = json!({
            "intervention_id": "INT-1",
            "intervention_name": "Pembrolizumab",
            "type": "drug",
            "category": "agent",
            "synonyms": ["Keytruda", "MK-3475"]
        });
        let result = from_nci_row(&row).expect("normalized row");
        assert_eq!(result.id, "INT-1");
        assert_eq!(result.name, "Pembrolizumab");
        assert_eq!(result.intervention_type.as_deref(), Some("drug"));
        assert_eq!(result.category.as_deref(), Some("agent"));
        assert_eq!(
            result.synonyms,
            vec!["Keytruda".to_string(), "MK-3475".to_string()]
        );
    }

    #[test]
    fn row_normalization_handles_comma_separated_synonyms() {
        let row = json!({
            "id": "INT-2",
            "name": "Drug X",
            "aliases": "X-1, X-2, X-3"
        });
        let result = from_nci_row(&row).expect("normalized row");
        assert_eq!(
            result.synonyms,
            vec!["X-1".to_string(), "X-2".to_string(), "X-3".to_string()]
        );
    }
}
