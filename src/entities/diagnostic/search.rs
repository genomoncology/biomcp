use crate::entities::SearchPage;
use crate::entities::disease::{ExactDiseaseTerms, resolve_exact_disease_terms};
use crate::error::BioMcpError;
use crate::sources::gtr::{GtrClient, GtrIndex, GtrRecord, GtrSyncMode};
use crate::sources::mydisease::MyDiseaseClient;
use crate::sources::who_ivd::{WhoIvdClient, WhoIvdRecord, WhoIvdSyncMode};

#[cfg(test)]
use std::path::Path;

use super::{
    DiagnosticSearchFilters, DiagnosticSearchResult, DiagnosticSourceFilter, DiseaseMatch,
    DiseaseMatchKind, search_result, who_ivd_search_result,
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
        if let Some(disease) = filters
            .disease
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            validate_disease_filter(disease)?;
        }
        let normalized = Self {
            source: filters.source,
            gene: normalized_exact(filters.gene.as_deref()),
            disease: normalized_disease(filters.disease.as_deref()),
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

        if matches!(normalized.source, DiagnosticSourceFilter::WhoIvd) && normalized.gene.is_some()
        {
            return Err(BioMcpError::InvalidArgument(
                "WHO IVD does not support --gene; use --source gtr or omit --source for gene-first diagnostic searches".to_string(),
            ));
        }

        Ok(normalized)
    }

    fn matches_gtr(
        &self,
        record: &GtrRecord,
        index: &GtrIndex,
        disease_terms: Option<&ExactDiseaseTerms>,
    ) -> Option<Option<DiseaseMatch>> {
        if let Some(gene) = self.gene.as_deref()
            && !index
                .merged_genes(&record.accession)
                .iter()
                .any(|candidate| candidate.trim().eq_ignore_ascii_case(gene))
        {
            return None;
        }

        let disease_match = if self.disease.is_some() {
            let terms = disease_terms.expect("disease terms required for disease filter");
            disease_match(&index.conditions(&record.accession), terms)
        } else {
            None
        };
        if self.disease.is_some() && disease_match.is_none() {
            return None;
        }

        if let Some(test_type) = self.test_type.as_deref()
            && !record.test_type.trim().eq_ignore_ascii_case(test_type)
        {
            return None;
        }

        if let Some(manufacturer) = self.manufacturer.as_deref()
            && !manufacturer_matches(record, manufacturer)
        {
            return None;
        }

        Some(disease_match)
    }

    fn matches_who_ivd(&self, record: &WhoIvdRecord) -> Option<Option<DiseaseMatch>> {
        let disease_match = self.disease.as_deref().and_then(|disease| {
            disease_phrase_matches(&record.target_marker, disease).then(|| DiseaseMatch {
                kind: DiseaseMatchKind::Requested,
                term: disease.to_string(),
                resolved_id: None,
            })
        });
        if self.disease.is_some() && disease_match.is_none() {
            return None;
        }

        if let Some(test_type) = self.test_type.as_deref()
            && !record.assay_format.trim().eq_ignore_ascii_case(test_type)
        {
            return None;
        }

        if let Some(manufacturer) = self.manufacturer.as_deref()
            && !record
                .manufacturer_name
                .to_ascii_lowercase()
                .contains(manufacturer)
        {
            return None;
        }

        Some(disease_match)
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

fn normalized_disease(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn collapse_unicode_whitespace(value: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for ch in value.chars() {
        if ch.is_whitespace() {
            pending_space = !out.is_empty();
        } else {
            if pending_space {
                out.push(' ');
            }
            out.push(ch.to_ascii_lowercase());
            pending_space = false;
        }
    }
    out
}

fn validate_disease_filter(value: &str) -> Result<(), BioMcpError> {
    if value.len() > 512 {
        return Err(BioMcpError::InvalidArgument(
            "--disease must be at most 512 UTF-8 bytes".to_string(),
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(BioMcpError::InvalidArgument(
            "--disease must not contain control characters".to_string(),
        ));
    }
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

    let lower = collapse_unicode_whitespace(haystack);
    let needle_lower = collapse_unicode_whitespace(needle_lower);
    lower.match_indices(&needle_lower).any(|(pos, matched)| {
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

fn disease_match(conditions: &[String], terms: &ExactDiseaseTerms) -> Option<DiseaseMatch> {
    if conditions
        .iter()
        .any(|condition| disease_phrase_matches(condition, &terms.requested))
    {
        return Some(DiseaseMatch {
            kind: DiseaseMatchKind::Requested,
            term: terms.requested.clone(),
            resolved_id: terms.canonical_id.clone(),
        });
    }
    if let Some(canonical) = terms.canonical_name.as_deref()
        && conditions
            .iter()
            .any(|condition| disease_phrase_matches(condition, canonical))
    {
        return Some(DiseaseMatch {
            kind: DiseaseMatchKind::Canonical,
            term: canonical.to_string(),
            resolved_id: terms.canonical_id.clone(),
        });
    }
    terms.synonyms.iter().find_map(|synonym| {
        conditions
            .iter()
            .any(|condition| disease_phrase_matches(condition, synonym))
            .then(|| DiseaseMatch {
                kind: DiseaseMatchKind::Synonym,
                term: synonym.clone(),
                resolved_id: terms.canonical_id.clone(),
            })
    })
}

fn disease_match_rank(
    result: &DiagnosticSearchResult,
    terms: Option<&ExactDiseaseTerms>,
) -> (u8, usize) {
    match result.disease_match.as_ref().map(|m| m.kind) {
        Some(DiseaseMatchKind::Requested) => (0, 0),
        Some(DiseaseMatchKind::Canonical) => (1, 0),
        Some(DiseaseMatchKind::Synonym) => (
            2,
            terms
                .and_then(|terms| {
                    result.disease_match.as_ref().and_then(|matched| {
                        terms.synonyms.iter().position(|term| term == &matched.term)
                    })
                })
                .unwrap_or(usize::MAX),
        ),
        None => (3, 0),
    }
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

fn sort_results(
    results: &mut [DiagnosticSearchResult],
    disease_filtered: bool,
    terms: Option<&ExactDiseaseTerms>,
) {
    if disease_filtered {
        results.sort_by(|left, right| {
            disease_match_rank(left, terms)
                .cmp(&disease_match_rank(right, terms))
                .then_with(|| result_sort_key(left).cmp(&result_sort_key(right)))
                .then_with(|| left.source.cmp(&right.source))
        });
    } else {
        results.sort_by_key(result_sort_key);
    }
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
    let disease_terms = if filters.source.includes_gtr() {
        if let Some(disease) = filters.disease.as_deref() {
            let client = MyDiseaseClient::new()?;
            Some(resolve_exact_disease_terms(&client, disease).await?)
        } else {
            None
        }
    } else {
        None
    };
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

    search_page_from_data(
        filters,
        limit,
        offset,
        gtr_index,
        who_rows,
        disease_terms.as_ref(),
    )
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

    let disease_terms = filters.disease.as_ref().map(|requested| ExactDiseaseTerms {
        requested: requested.clone(),
        canonical_id: None,
        canonical_name: None,
        synonyms: Vec::new(),
    });
    search_page_from_data(
        filters,
        limit,
        offset,
        gtr_index,
        who_rows,
        disease_terms.as_ref(),
    )
}

fn search_page_from_data(
    filters: NormalizedSearchFilters,
    limit: usize,
    offset: usize,
    gtr_index: Option<GtrIndex>,
    who_rows: Option<Vec<WhoIvdRecord>>,
    disease_terms: Option<&ExactDiseaseTerms>,
) -> Result<SearchPage<DiagnosticSearchResult>, BioMcpError> {
    let mut results = Vec::new();
    let mut matching_sources = 0usize;
    let mut known_total = 0usize;

    if let Some(index) = gtr_index {
        let gtr_results = index
            .records_by_id
            .values()
            .filter_map(|record| {
                let disease_match = filters.matches_gtr(record, &index, disease_terms)?;
                let mut result = search_result(record, &index);
                result.disease_match = disease_match;
                Some(result)
            })
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
            .filter_map(|record| {
                let disease_match = filters.matches_who_ivd(&record)?;
                let mut result = who_ivd_search_result(&record);
                result.disease_match = disease_match;
                Some(result)
            })
            .collect::<Vec<_>>();
        if !who_results.is_empty() {
            matching_sources += 1;
            known_total += who_results.len();
        }
        results.extend(who_results);
    }

    sort_results(&mut results, filters.disease.is_some(), disease_terms);

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

    #[test]
    fn disease_filter_validation_rejects_controls_and_oversize_before_normalization() {
        let controls = NormalizedSearchFilters::from_filters(&DiagnosticSearchFilters {
            disease: Some("rare\nsyndrome".to_string()),
            ..DiagnosticSearchFilters::default()
        })
        .expect_err("controls must fail before provider or local work");
        assert_eq!(
            controls.to_string(),
            "Invalid argument: --disease must not contain control characters"
        );

        let oversize = NormalizedSearchFilters::from_filters(&DiagnosticSearchFilters {
            disease: Some("x".repeat(513)),
            ..DiagnosticSearchFilters::default()
        })
        .expect_err("oversize disease text must fail before provider or local work");
        assert_eq!(
            oversize.to_string(),
            "Invalid argument: --disease must be at most 512 UTF-8 bytes"
        );
    }

    #[test]
    fn disease_match_prefers_requested_then_canonical_then_synonym() {
        let terms = ExactDiseaseTerms {
            requested: "Bachmann-Bupp syndrome".to_string(),
            canonical_id: Some("MONDO:0033642".to_string()),
            canonical_name: Some("Neurodevelopmental disorder".to_string()),
            synonyms: vec!["BABS".to_string(), "Bachmann-Bupp disease".to_string()],
        };
        assert_eq!(
            disease_match(&["Bachmann-Bupp syndrome".to_string()], &terms)
                .expect("requested match")
                .kind,
            DiseaseMatchKind::Requested
        );
        assert_eq!(
            disease_match(&["Neurodevelopmental disorder".to_string()], &terms)
                .expect("canonical match")
                .kind,
            DiseaseMatchKind::Canonical
        );
        assert_eq!(
            disease_match(&["BABS".to_string()], &terms)
                .expect("synonym match")
                .kind,
            DiseaseMatchKind::Synonym
        );
    }

    #[test]
    fn disease_match_uses_term_priority_across_all_conditions() {
        let terms = ExactDiseaseTerms {
            requested: "requested disease".to_string(),
            canonical_id: Some("MONDO:1".to_string()),
            canonical_name: Some("canonical disease".to_string()),
            synonyms: vec!["first alias".to_string(), "second alias".to_string()],
        };
        let requested = disease_match(
            &["second alias".to_string(), "requested disease".to_string()],
            &terms,
        )
        .expect("requested term across later condition");
        assert_eq!(requested.kind, DiseaseMatchKind::Requested);

        let first_synonym = disease_match(
            &["second alias".to_string(), "first alias".to_string()],
            &terms,
        )
        .expect("first provider synonym across later condition");
        assert_eq!(first_synonym.kind, DiseaseMatchKind::Synonym);
        assert_eq!(first_synonym.term, "first alias");
    }

    #[test]
    fn cross_condition_match_priority_drives_sort_before_paging() {
        let terms = ExactDiseaseTerms {
            requested: "requested disease".to_string(),
            canonical_id: Some("MONDO:1".to_string()),
            canonical_name: Some("canonical disease".to_string()),
            synonyms: vec!["first alias".to_string(), "second alias".to_string()],
        };
        let row = |accession: &str, conditions: &[&str]| DiagnosticSearchResult {
            source: "gtr".to_string(),
            accession: accession.to_string(),
            name: accession.to_string(),
            test_type: None,
            manufacturer_or_lab: None,
            genes: Vec::new(),
            conditions: conditions
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            disease_match: None,
        };
        let mut results = vec![
            row("second", &["second alias"]),
            row("requested", &["second alias", "requested disease"]),
            row("first", &["second alias", "first alias"]),
        ];
        for result in &mut results {
            result.disease_match = disease_match(&result.conditions, &terms);
        }
        sort_results(&mut results, true, Some(&terms));
        assert_eq!(
            results
                .iter()
                .map(|result| result.accession.as_str())
                .collect::<Vec<_>>(),
            vec!["requested", "first", "second"]
        );
        let page = results.into_iter().skip(1).take(1).collect::<Vec<_>>();
        assert_eq!(page[0].accession, "first");
    }

    #[test]
    fn disease_sort_ranks_who_requested_then_canonical_then_provider_synonym_order() {
        let terms = ExactDiseaseTerms {
            requested: "requested disease".to_string(),
            canonical_id: Some("MONDO:1".to_string()),
            canonical_name: Some("canonical disease".to_string()),
            synonyms: vec!["first alias".to_string(), "second alias".to_string()],
        };
        let row = |accession: &str, source: &str, kind, term: &str| DiagnosticSearchResult {
            source: source.to_string(),
            accession: accession.to_string(),
            name: "same name".to_string(),
            test_type: None,
            manufacturer_or_lab: None,
            genes: Vec::new(),
            conditions: Vec::new(),
            disease_match: Some(DiseaseMatch {
                kind,
                term: term.to_string(),
                resolved_id: (source == "gtr").then(|| "MONDO:1".to_string()),
            }),
        };
        let mut rows = vec![
            row("S2", "gtr", DiseaseMatchKind::Synonym, "second alias"),
            row("C", "gtr", DiseaseMatchKind::Canonical, "canonical disease"),
            row("S1", "gtr", DiseaseMatchKind::Synonym, "first alias"),
            row(
                "W",
                "who-ivd",
                DiseaseMatchKind::Requested,
                "requested disease",
            ),
        ];
        sort_results(&mut rows, true, Some(&terms));
        assert_eq!(
            rows.iter()
                .map(|row| row.accession.as_str())
                .collect::<Vec<_>>(),
            vec!["W", "C", "S1", "S2"]
        );
    }
}
