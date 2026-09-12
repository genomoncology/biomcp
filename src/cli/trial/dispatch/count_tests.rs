use super::render_count_only;
use crate::entities::trial::{ClinicalTrialSearchTotal, ClinicalTrialSearchUnknownReason};

#[test]
fn json_preserves_precision_and_omits_unknown_approximation() {
    let exact = render_count_only(ClinicalTrialSearchTotal::exact(23).unwrap(), true)
        .expect("exact count JSON");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&exact).expect("count JSON"),
        serde_json::json!({"total": 23, "total_precision": "exact"})
    );
    let approximate = render_count_only(
        ClinicalTrialSearchTotal::approximate(
            23,
            biodata::ClinicalTrialSearchApproximationReason::BeforeLocalFiltering,
        )
        .unwrap(),
        true,
    )
    .expect("approximate count JSON");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&approximate).expect("count JSON"),
        serde_json::json!({
            "total": 23,
            "total_precision": "approximate",
            "total_reason": "before_local_filtering",
            "approximate": true
        })
    );
    for reason in [
        ClinicalTrialSearchUnknownReason::TotalNotRequested,
        ClinicalTrialSearchUnknownReason::ProviderOmittedTotal,
        ClinicalTrialSearchUnknownReason::TraversalLimitReached,
        ClinicalTrialSearchUnknownReason::IncompleteSourceCoverage,
        ClinicalTrialSearchUnknownReason::IncompleteLocalVerification,
    ] {
        let rendered = render_count_only(ClinicalTrialSearchTotal::unknown(reason), true)
            .expect("unknown count JSON");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&rendered).expect("count JSON"),
            serde_json::json!({
                "total": null,
                "total_precision": "unknown",
                "total_reason": reason.as_str()
            })
        );
    }
}

#[test]
fn text_explains_each_unknown_reason_truthfully() {
    assert_eq!(
        render_count_only(
            ClinicalTrialSearchTotal::unknown(
                ClinicalTrialSearchUnknownReason::TraversalLimitReached
            ),
            false,
        )
        .expect("cap count text"),
        "Total: unknown (traversal_limit_reached)"
    );
    for (reason, expected) in [
        (
            ClinicalTrialSearchUnknownReason::ProviderOmittedTotal,
            "provider_omitted_total",
        ),
        (
            ClinicalTrialSearchUnknownReason::IncompleteSourceCoverage,
            "incomplete_source_coverage",
        ),
    ] {
        let rendered = render_count_only(ClinicalTrialSearchTotal::unknown(reason), false)
            .expect("unknown count text");
        assert!(rendered.contains(expected));
        assert!(!rendered.contains("Total: 0"));
        assert!(!rendered.contains("traversal_limit_reached"));
    }
}
