use super::{TrialPaginationMeta, trial_pagination_footer, trial_search_json};

#[test]
fn json_exposes_exact_total_and_only_the_typed_cursor() {
    let total = biodata::ClinicalTrialSearchTotal::exact(8).unwrap();
    let continuation = biodata::ClinicalTrialSearchContinuation::cursor("page-two").unwrap();
    let pagination = TrialPaginationMeta::new(0, 2, 2, &total, &continuation);
    let rendered = trial_search_json(
        vec![serde_json::json!({"nct_id": "NCT00000001"})],
        pagination,
        Vec::new(),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(value["count"], 1);
    assert_eq!(value["pagination"]["total"], 8);
    assert_eq!(value["pagination"]["total_precision"], "exact");
    assert!(value["pagination"].get("total_reason").is_none());
    assert_eq!(value["pagination"]["continuation_status"], "cursor");
    assert_eq!(value["pagination"]["next_page_token"], "page-two");
    assert!(value["pagination"].get("next_offset").is_none());
    assert_eq!(value["pagination"]["has_more"], true);
}

#[test]
fn unavailable_state_never_invents_a_markdown_continuation() {
    let total = biodata::ClinicalTrialSearchTotal::unknown(
        biodata::ClinicalTrialSearchUnknownReason::IncompleteSourceCoverage,
    );
    let continuation = biodata::ClinicalTrialSearchContinuation::unavailable(
        biodata::ClinicalTrialSearchContinuationUnavailableReason::IncompleteSourceCoverage,
    );
    let pagination = TrialPaginationMeta::new(4, 2, 1, &total, &continuation);
    let footer = trial_pagination_footer(&pagination);
    assert!(footer.contains("Total: unknown (incomplete_source_coverage)."));
    assert!(!footer.contains("Use --offset"));
    assert!(!footer.contains("Use --next-page"));
    let rendered =
        trial_search_json(Vec::<serde_json::Value>::new(), pagination, Vec::new()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(value["pagination"]["continuation_status"], "unavailable");
    assert_eq!(
        value["pagination"]["continuation_reason"],
        "incomplete_source_coverage"
    );
    assert_eq!(
        value["pagination"]["next_page_token"],
        serde_json::Value::Null
    );
    assert!(value["pagination"].get("next_offset").is_none());
    assert_eq!(value["pagination"]["has_more"], false);
}

#[test]
fn offset_state_emits_only_the_offset_command() {
    let total = biodata::ClinicalTrialSearchTotal::approximate(
        40,
        biodata::ClinicalTrialSearchApproximationReason::BeforeLocalFiltering,
    )
    .unwrap();
    let continuation = biodata::ClinicalTrialSearchContinuation::offset(12).unwrap();
    let pagination = TrialPaginationMeta::new(7, 5, 5, &total, &continuation);
    let footer = trial_pagination_footer(&pagination);
    assert!(footer.contains("Total: 40 (approximate: before_local_filtering)."));
    assert!(footer.contains("Use --offset 12 for more."));
    assert!(!footer.contains("--next-page"));
}
