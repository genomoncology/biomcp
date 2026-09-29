use super::dispatch::{PARTIAL_COUNT_REASON_TEXT, TrialPaginationMeta, render_count_only};
use crate::entities::trial::TrialCount;

#[test]
fn json_and_text_mark_a_partial_count_with_its_reason() {
    let json = render_count_only(TrialCount::partial(7), true).expect("partial count JSON");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).expect("count JSON"),
        serde_json::json!({
            "total": 7,
            "total_precision": "partial",
            "total_reason": "incomplete_local_verification",
            "partial": true,
            "partial_reason": PARTIAL_COUNT_REASON_TEXT
        })
    );
    let text = render_count_only(TrialCount::partial(7), false).expect("partial count text");
    assert_eq!(
        text,
        format!("Total: 7 (partial, {PARTIAL_COUNT_REASON_TEXT})")
    );
}

#[test]
fn search_json_carries_the_partial_note_in_meta_notes() {
    use super::dispatch::search_json_with_meta_and_upstream_total;
    let payload = search_json_with_meta_and_upstream_total(
        vec![serde_json::json!({"nct_id": "NCT1"})],
        TrialPaginationMeta::new(
            0, 10, 1,
            &biodata::ClinicalTrialSearchTotal::exact(1).unwrap(),
            &biodata::ClinicalTrialSearchContinuation::terminal(),
        ),
        Vec::new(),
        None,
        Some("The count may be too high: we could not check 1 of the kept trials (NCT1), because the detail fetch failed, the eligibility text was missing, or the trial had no NCT ID."),
    )
    .expect("search JSON");
    let value: serde_json::Value = serde_json::from_str(&payload).expect("valid JSON");
    assert_eq!(
        value["_meta"]["notes"][0],
        "The count may be too high: we could not check 1 of the kept trials (NCT1), because the detail fetch failed, the eligibility text was missing, or the trial had no NCT ID."
    );
}

#[test]
fn search_json_omits_meta_notes_when_the_page_is_fully_verified() {
    use super::dispatch::search_json_with_meta_and_upstream_total;
    let payload = search_json_with_meta_and_upstream_total(
        vec![serde_json::json!({"nct_id": "NCT1"})],
        TrialPaginationMeta::new(
            0,
            10,
            1,
            &biodata::ClinicalTrialSearchTotal::exact(1).unwrap(),
            &biodata::ClinicalTrialSearchContinuation::terminal(),
        ),
        Vec::new(),
        None,
        None,
    )
    .expect("search JSON");
    let value: serde_json::Value = serde_json::from_str(&payload).expect("valid JSON");
    assert!(value.get("_meta").is_none() || value["_meta"].get("notes").is_none());
}
