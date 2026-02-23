use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::nci_cts::{NciCtsClient, NciOrganizationSearchParams};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSearchResult {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct OrganizationSearchFilters {
    pub query: Option<String>,
    pub organization_type: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
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

fn has_any_filter(filters: &OrganizationSearchFilters) -> bool {
    filters
        .query
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
        || filters
            .organization_type
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .city
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .state
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
}

fn from_nci_row(row: &serde_json::Value) -> Option<OrganizationSearchResult> {
    let id = json_get_string(row, &["id", "org_id", "orgId"]).unwrap_or_default();
    let name = json_get_string(
        row,
        &["name", "org_name", "organization_name", "display_name"],
    )
    .unwrap_or_else(|| id.clone());
    if name.is_empty() {
        return None;
    }

    Some(OrganizationSearchResult {
        id,
        name,
        organization_type: clean_opt(
            json_get_string(row, &["type", "org_type", "category"]).as_deref(),
        ),
        city: clean_opt(json_get_string(row, &["city", "org_city"]).as_deref()),
        state: clean_opt(
            json_get_string(
                row,
                &["state", "state_or_province", "org_state_or_province"],
            )
            .as_deref(),
        ),
        country: clean_opt(json_get_string(row, &["country"]).as_deref()),
    })
}

pub fn search_query_summary(filters: &OrganizationSearchFilters) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = clean_opt(filters.query.as_deref()) {
        parts.push(format!("query={v}"));
    }
    if let Some(v) = clean_opt(filters.organization_type.as_deref()) {
        parts.push(format!("type={v}"));
    }
    if let Some(v) = clean_opt(filters.city.as_deref()) {
        parts.push(format!("city={v}"));
    }
    if let Some(v) = clean_opt(filters.state.as_deref()) {
        parts.push(format!("state={v}"));
    }
    parts.join(", ")
}

#[allow(dead_code)]
pub async fn search(
    filters: &OrganizationSearchFilters,
    limit: usize,
) -> Result<Vec<OrganizationSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub async fn search_page(
    filters: &OrganizationSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<OrganizationSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }
    if !has_any_filter(filters) {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter is required. Example: biomcp search organization -q \"MD Anderson\"".into(),
        ));
    }

    let client = NciCtsClient::new()?;
    let page = client
        .search_organizations_page(&NciOrganizationSearchParams {
            name: filters.query.clone(),
            organization_type: filters.organization_type.clone(),
            city: filters.city.clone(),
            state: filters.state.clone(),
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
    fn search_query_summary_includes_all_filters() {
        let summary = search_query_summary(&OrganizationSearchFilters {
            query: Some("Anderson".into()),
            organization_type: Some("academic".into()),
            city: Some("Houston".into()),
            state: Some("TX".into()),
        });
        assert_eq!(
            summary,
            "query=Anderson, type=academic, city=Houston, state=TX"
        );
    }

    #[test]
    fn row_normalization_uses_aliases() {
        let row = json!({
            "org_id": "ORG-1",
            "organization_name": "Test Center",
            "org_type": "academic",
            "org_city": "Houston",
            "org_state_or_province": "TX",
            "country": "USA"
        });
        let result = from_nci_row(&row).expect("normalized row");
        assert_eq!(result.id, "ORG-1");
        assert_eq!(result.name, "Test Center");
        assert_eq!(result.organization_type.as_deref(), Some("academic"));
        assert_eq!(result.city.as_deref(), Some("Houston"));
        assert_eq!(result.state.as_deref(), Some("TX"));
        assert_eq!(result.country.as_deref(), Some("USA"));
    }

    #[test]
    fn has_any_filter_requires_non_empty_value() {
        assert!(!has_any_filter(&OrganizationSearchFilters::default()));
        assert!(has_any_filter(&OrganizationSearchFilters {
            query: Some("MD Anderson".into()),
            ..Default::default()
        }));
    }
}
