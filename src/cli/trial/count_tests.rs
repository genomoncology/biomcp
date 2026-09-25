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
