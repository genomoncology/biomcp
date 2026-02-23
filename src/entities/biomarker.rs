use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::nci_cts::{NciBiomarkerSearchParams, NciCtsClient};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomarkerSearchResult {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biomarker_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assay_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trial_count: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct BiomarkerSearchFilters {
    pub query: Option<String>,
    pub biomarker_type: Option<String>,
    pub eligibility_criterion: Option<String>,
    pub assay_purpose: Option<String>,
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

fn json_get_u32(value: &serde_json::Value, keys: &[&str]) -> Option<u32> {
    for key in keys {
        if let Some(v) = value.get(*key) {
            if let Some(n) = v.as_u64() {
                return u32::try_from(n).ok();
            }
            if let Some(raw) = v.as_str() {
                let raw = raw.trim();
                if let Ok(n) = raw.parse::<u32>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

fn has_any_filter(filters: &BiomarkerSearchFilters) -> bool {
    filters
        .query
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
        || filters
            .biomarker_type
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .eligibility_criterion
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .assay_purpose
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .code
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
}

fn from_nci_row(row: &serde_json::Value) -> Option<BiomarkerSearchResult> {
    let id = json_get_string(row, &["id", "biomarker_id", "biomarkerId"]).unwrap_or_default();
    let name = json_get_string(row, &["name", "biomarker_name", "preferred_name"])
        .unwrap_or_else(|| id.clone());
    if name.is_empty() {
        return None;
    }

    Some(BiomarkerSearchResult {
        id,
        name,
        gene: clean_opt(
            json_get_string(
                row,
                &["gene", "gene_symbol", "geneSymbol", "reference_gene"],
            )
            .as_deref(),
        ),
        biomarker_type: clean_opt(
            json_get_string(row, &["type", "biomarker_type", "category"]).as_deref(),
        ),
        assay_type: clean_opt(
            json_get_string(row, &["assay_type", "assay", "assayType"]).as_deref(),
        ),
        trial_count: json_get_u32(row, &["trial_count", "trialCount", "count"]),
    })
}

pub fn search_query_summary(filters: &BiomarkerSearchFilters) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = clean_opt(filters.query.as_deref()) {
        parts.push(format!("query={v}"));
    }
    if let Some(v) = clean_opt(filters.biomarker_type.as_deref()) {
        parts.push(format!("type={v}"));
    }
    if let Some(v) = clean_opt(filters.eligibility_criterion.as_deref()) {
        parts.push(format!("eligibility={v}"));
    }
    if let Some(v) = clean_opt(filters.assay_purpose.as_deref()) {
        parts.push(format!("assay_purpose={v}"));
    }
    if let Some(v) = clean_opt(filters.code.as_deref()) {
        parts.push(format!("code={v}"));
    }
    parts.join(", ")
}

#[allow(dead_code)]
pub async fn search(
    filters: &BiomarkerSearchFilters,
    limit: usize,
) -> Result<Vec<BiomarkerSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub async fn search_page(
    filters: &BiomarkerSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<BiomarkerSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }
    if !has_any_filter(filters) {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter is required. Example: biomcp search biomarker -q \"PD-L1\"".into(),
        ));
    }

    let client = NciCtsClient::new()?;
    let page = client
        .search_biomarkers_page(&NciBiomarkerSearchParams {
            name: filters.query.clone(),
            biomarker_type: filters.biomarker_type.clone(),
            eligibility_criterion: filters.eligibility_criterion.clone(),
            assay_purpose: filters.assay_purpose.clone(),
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
        let summary = search_query_summary(&BiomarkerSearchFilters {
            query: Some("PD-L1".into()),
            biomarker_type: Some("reference_gene".into()),
            eligibility_criterion: Some("positive".into()),
            assay_purpose: Some("eligibility".into()),
            code: Some("C104743".into()),
        });
        assert_eq!(
            summary,
            "query=PD-L1, type=reference_gene, eligibility=positive, assay_purpose=eligibility, code=C104743"
        );
    }

    #[test]
    fn row_normalization_uses_alias_fields() {
        let row = json!({
            "biomarker_id": "BIO-1",
            "biomarker_name": "PD-L1",
            "gene_symbol": "CD274",
            "category": "reference_gene",
            "assay": "IHC",
            "trialCount": "12"
        });
        let result = from_nci_row(&row).expect("normalized row");
        assert_eq!(result.id, "BIO-1");
        assert_eq!(result.name, "PD-L1");
        assert_eq!(result.gene.as_deref(), Some("CD274"));
        assert_eq!(result.biomarker_type.as_deref(), Some("reference_gene"));
        assert_eq!(result.assay_type.as_deref(), Some("IHC"));
        assert_eq!(result.trial_count, Some(12));
    }

    #[test]
    fn has_any_filter_requires_non_empty_value() {
        assert!(!has_any_filter(&BiomarkerSearchFilters::default()));
        assert!(has_any_filter(&BiomarkerSearchFilters {
            query: Some("PD-L1".into()),
            ..Default::default()
        }));
    }
}
