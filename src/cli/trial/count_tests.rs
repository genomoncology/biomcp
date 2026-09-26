use super::dispatch::{PARTIAL_COUNT_REASON_TEXT, render_count_only};
use crate::entities::trial::{TrialCount, TrialCountPartialReason, TrialCountUnknownReason};

#[test]
fn json_and_text_mark_a_partial_count_with_its_reason() {
    let json = render_count_only(
        TrialCount::Partial {
            total: 7,
            reason: TrialCountPartialReason::DetailVerificationIncomplete,
        },
        true,
    )
    .expect("partial count JSON");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).expect("count JSON"),
        serde_json::json!({
            "total": 7,
            "partial": true,
            "partial_reason": PARTIAL_COUNT_REASON_TEXT
        })
    );
    let text = render_count_only(
        TrialCount::Partial {
            total: 7,
            reason: TrialCountPartialReason::DetailVerificationIncomplete,
        },
        false,
    )
    .expect("partial count text");
    assert_eq!(
        text,
        format!("Total: 7 (partial, {PARTIAL_COUNT_REASON_TEXT})")
    );
}

#[test]
fn json_preserves_precision_and_omits_unknown_approximation() {
    let approximate =
        render_count_only(TrialCount::Approximate(23), true).expect("approximate count JSON");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&approximate).expect("count JSON"),
        serde_json::json!({"total": 23, "approximate": true})
    );
    for reason in [
        TrialCountUnknownReason::ProviderOmittedTotal,
        TrialCountUnknownReason::TraversalLimitReached,
        TrialCountUnknownReason::IncompleteCoverage,
    ] {
        let rendered =
            render_count_only(TrialCount::Unknown(reason), true).expect("unknown count JSON");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&rendered).expect("count JSON"),
            serde_json::json!({"total": null})
        );
    }
}

#[test]
fn text_explains_each_unknown_reason_truthfully() {
    assert_eq!(
        render_count_only(
            TrialCount::Unknown(TrialCountUnknownReason::TraversalLimitReached),
            false,
        )
        .expect("cap count text"),
        "Total: unknown (traversal limit reached)"
    );
    for (reason, expected) in [
        (
            TrialCountUnknownReason::ProviderOmittedTotal,
            "provider omitted the requested total",
        ),
        (
            TrialCountUnknownReason::IncompleteCoverage,
            "expanded CTGov coverage incomplete",
        ),
    ] {
        let rendered =
            render_count_only(TrialCount::Unknown(reason), false).expect("unknown count text");
        assert!(rendered.contains(expected));
        assert!(!rendered.contains("Total: 0"));
        assert!(!rendered.contains("traversal limit reached"));
    }
}

#[test]
fn search_json_carries_the_partial_note_in_meta_notes() {
    use super::dispatch::search_json_with_meta_and_upstream_total;
    let payload = search_json_with_meta_and_upstream_total(
        vec![serde_json::json!({"nct_id": "NCT1"})],
        crate::cli::shared::PaginationMeta::cursor(0, 10, 1, Some(1), None),
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
        crate::cli::shared::PaginationMeta::cursor(0, 10, 1, Some(1), None),
        Vec::new(),
        None,
        None,
    )
    .expect("search JSON");
    let value: serde_json::Value = serde_json::from_str(&payload).expect("valid JSON");
    assert!(value.get("_meta").is_none() || value["_meta"].get("notes").is_none());
}
