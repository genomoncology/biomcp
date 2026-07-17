use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::gtr::{GtrClient, GtrIndex, GtrRecord, GtrSyncMode};
use crate::sources::who_ivd::{WhoIvdClient, WhoIvdRecord, WhoIvdSyncMode};

#[cfg(test)]
use std::path::Path;

use super::{
    DiagnosticSearchFilters, DiagnosticSearchResult, DiagnosticSourceFilter, search_result,
    who_ivd_search_result,
};

const MAX_SEARCH_LIMIT: usize = 50;
const MIN_DISEASE_MATCH_ALNUM_CHARS: usize = 3;
const ZERO_FILTER_ERROR: &str =
    "diagnostic search requires at least one of --gene, --disease, --type, or --manufacturer";

#[derive(Debug, Clone)]
struct NormalizedSearchFilters {
    source: DiagnosticSourceFilter,
    gene: Option<String>,
    disease: Option<String>,
    test_type: Option<String>,
    manufacturer: Option<String>,
}

impl NormalizedSearchFilters {
    fn from_filters(filters: &DiagnosticSearchFilters) -> Result<Self, BioMcpError> {
        let normalized = Self {
            source: filters.source,
            gene: normalized_exact(filters.gene.as_deref()),
            disease: normalized_contains(filters.disease.as_deref()),
            test_type: normalized_exact(filters.test_type.as_deref()),
            manufacturer: normalized_contains(filters.manufacturer.as_deref()),
        };

        if normalized.gene.is_none()
            && normalized.disease.is_none()
            && normalized.test_type.is_none()
            && normalized.manufacturer.is_none()
        {
            return Err(BioMcpError::InvalidArgument(ZERO_FILTER_ERROR.to_string()));
        }

        if let Some(disease) = normalized.disease.as_deref() {
            validate_disease_filter(disease)?;
        }

        if matches!(normalized.source, DiagnosticSourceFilter::WhoIvd) && normalized.gene.is_some()
        {
            return Err(BioMcpError::InvalidArgument(
                "WHO IVD does not support --gene; use --source gtr or omit --source for gene-first diagnostic searches".to_string(),
            ));
        }

        Ok(normalized)
    }

    fn matches_gtr(&self, record: &GtrRecord, index: &GtrIndex) -> bool {
        if let Some(gene) = self.gene.as_deref()
            && !index
                .merged_genes(&record.accession)
                .iter()
                .any(|candidate| candidate.trim().eq_ignore_ascii_case(gene))
        {
            return false;
        }

        if let Some(disease) = self.disease.as_deref()
            && !index
                .conditions(&record.accession)
                .iter()
                .any(|candidate| disease_phrase_matches(candidate, disease))
        {
            return false;
        }

        if let Some(test_type) = self.test_type.as_deref()
            && !record.test_type.trim().eq_ignore_ascii_case(test_type)
        {
            return false;
        }

        if let Some(manufacturer) = self.manufacturer.as_deref()
            && !manufacturer_matches(record, manufacturer)
        {
            return false;
        }

        true
    }

    fn matches_who_ivd(&self, record: &WhoIvdRecord) -> bool {
        if let Some(disease) = self.disease.as_deref()
            && !disease_phrase_matches(&record.target_marker, disease)
        {
            return false;
        }

        if let Some(test_type) = self.test_type.as_deref()
            && !record.assay_format.trim().eq_ignore_ascii_case(test_type)
        {
            return false;
        }

        if let Some(manufacturer) = self.manufacturer.as_deref()
            && !record
                .manufacturer_name
                .to_ascii_lowercase()
                .contains(manufacturer)
        {
            return false;
        }

        true
    }
}

fn normalized_exact(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn normalized_contains(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
}

fn validate_disease_filter(value: &str) -> Result<(), BioMcpError> {
    let alnum_count = value.chars().filter(|ch| ch.is_alphanumeric()).count();
    if alnum_count < MIN_DISEASE_MATCH_ALNUM_CHARS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--disease must contain at least {MIN_DISEASE_MATCH_ALNUM_CHARS} alphanumeric characters for diagnostic disease matching"
        )));
    }
    Ok(())
}

fn disease_phrase_matches(haystack: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return false;
    }

    let lower = haystack.to_ascii_lowercase();
    lower.match_indices(needle_lower).any(|(pos, matched)| {
        let before_ok = lower[..pos]
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_alphanumeric());
        let after = pos + matched.len();
        let after_ok = lower[after..]
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_alphanumeric());
        before_ok && after_ok
    })
}

fn manufacturer_matches(record: &GtrRecord, needle: &str) -> bool {
    [
        record.manufacturer_test_name.as_str(),
        record.name_of_laboratory.as_str(),
        record.lab_test_name.as_str(),
    ]
    .into_iter()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .any(|value| value.to_ascii_lowercase().contains(needle))
}

fn result_sort_key(result: &DiagnosticSearchResult) -> (String, String) {
    (
        result.name.trim().to_ascii_lowercase(),
        result.accession.clone(),
    )
}

pub async fn search_page(
    filters: &DiagnosticSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<DiagnosticSearchResult>, BioMcpError> {
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let filters = NormalizedSearchFilters::from_filters(filters)?;
    let gtr_index = if filters.source.includes_gtr() {
        let client = GtrClient::ready(GtrSyncMode::Auto).await?;
        Some(client.load_index()?)
    } else {
        None
    };

    let should_query_who_ivd = filters.source.includes_who_ivd() && filters.gene.is_none();
    let who_rows = if should_query_who_ivd {
        let client = WhoIvdClient::ready(WhoIvdSyncMode::Auto).await?;
        Some(client.read_rows()?)
    } else {
        None
    };

    search_page_from_data(filters, limit, offset, gtr_index, who_rows)
}

#[cfg(test)]
pub(super) fn search_page_with_roots(
    filters: &DiagnosticSearchFilters,
    limit: usize,
    offset: usize,
    gtr_root: Option<&Path>,
    who_ivd_root: Option<&Path>,
) -> Result<SearchPage<DiagnosticSearchResult>, BioMcpError> {
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let filters = NormalizedSearchFilters::from_filters(filters)?;
    let gtr_index = if filters.source.includes_gtr() {
        let root = gtr_root.ok_or_else(|| {
            BioMcpError::InvalidArgument("diagnostic GTR test root is required".to_string())
        })?;
        Some(GtrClient::from_root(root).load_index()?)
    } else {
        None
    };

    let should_query_who_ivd = filters.source.includes_who_ivd() && filters.gene.is_none();
    let who_rows = if should_query_who_ivd {
        let root = who_ivd_root.ok_or_else(|| {
            BioMcpError::InvalidArgument("diagnostic WHO IVD test root is required".to_string())
        })?;
        Some(WhoIvdClient::from_root(root).read_rows()?)
    } else {
        None
    };

    search_page_from_data(filters, limit, offset, gtr_index, who_rows)
}

fn search_page_from_data(
    filters: NormalizedSearchFilters,
    limit: usize,
    offset: usize,
    gtr_index: Option<GtrIndex>,
    who_rows: Option<Vec<WhoIvdRecord>>,
) -> Result<SearchPage<DiagnosticSearchResult>, BioMcpError> {
    let mut results = Vec::new();
    let mut matching_sources = 0usize;
    let mut known_total = 0usize;

    if let Some(index) = gtr_index {
        let gtr_results = index
            .records_by_id
            .values()
            .filter(|record| filters.matches_gtr(record, &index))
            .map(|record| search_result(record, &index))
            .collect::<Vec<_>>();
        if !gtr_results.is_empty() {
            matching_sources += 1;
            known_total += gtr_results.len();
        }
        results.extend(gtr_results);
    }

    if let Some(who_rows) = who_rows {
        let who_results = who_rows
            .into_iter()
            .filter(|record| filters.matches_who_ivd(record))
            .map(|record| who_ivd_search_result(&record))
            .collect::<Vec<_>>();
        if !who_results.is_empty() {
            matching_sources += 1;
            known_total += who_results.len();
        }
        results.extend(who_results);
    }

    results.sort_by_key(result_sort_key);

    let total = match filters.source {
        DiagnosticSourceFilter::All if matching_sources > 1 => None,
        DiagnosticSourceFilter::All if matching_sources == 0 => Some(0),
        _ => Some(known_total),
    };
    let results = results.into_iter().skip(offset).take(limit).collect();
    Ok(SearchPage::offset(results, total))
}

pub fn search_query_summary(filters: &DiagnosticSearchFilters) -> String {
    let mut parts = Vec::new();
    if let Some(gene) = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("gene={gene}"));
    }
    if let Some(disease) = filters
        .disease
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("disease={disease}"));
    }
    if let Some(test_type) = filters
        .test_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("type={test_type}"));
    }
    if let Some(manufacturer) = filters
        .manufacturer
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("manufacturer={manufacturer}"));
    }
    if let Some(source) = filters.source.query_summary() {
        parts.push(source);
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disease_phrase_matches_accepts_word_and_phrase_boundaries() {
        assert!(disease_phrase_matches(
            "Mycobacterium tuberculosis complex",
            "tuberculosis"
        ));
        assert!(disease_phrase_matches(
            "Hereditary breast cancer panel",
            "breast cancer"
        ));
    }

    #[test]
    fn disease_phrase_matches_rejects_partial_words_and_keeps_scanning() {
        assert!(!disease_phrase_matches("leukemia", "emia"));
        assert!(disease_phrase_matches(
            "preanemia panel; anemia confirmation",
            "anemia"
        ));
    }

    #[test]
    fn disease_phrase_matches_handles_utf8_boundaries_without_panicking() {
        assert!(disease_phrase_matches(
            "β-thalassemia screening",
            "β-thalassemia"
        ));
        assert!(!disease_phrase_matches("préleukemia", "emia"));
    }

    #[test]
    fn normalized_filters_reject_short_disease_filter() {
        let err = NormalizedSearchFilters::from_filters(&DiagnosticSearchFilters {
            disease: Some("m-a".to_string()),
            ..DiagnosticSearchFilters::default()
        })
        .expect_err("short disease filter should fail before data access");

        assert_eq!(
            err.to_string(),
            "Invalid argument: --disease must contain at least 3 alphanumeric characters for diagnostic disease matching"
        );
    }
}
