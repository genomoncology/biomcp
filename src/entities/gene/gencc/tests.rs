use std::sync::atomic::AtomicBool;

use super::*;
use crate::sources::gencc::model::GenCcDataset;

fn dataset() -> GenCcDataset {
    GenCcDataset::parse(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/gencc/submissions-new-odc1.csv"
        )),
        &AtomicBool::new(false),
    )
    .unwrap()
}

fn data() -> GenCcData {
    GenCcData {
        dataset: Some(dataset()),
        status: GenCcStatus {
            freshness: GenCcFreshness::Fresh,
            result: GenCcResult::Data,
            operation: GenCcOperation::LocalQuery,
            checked_at: Some("2026-09-05T22:51:21Z".into()),
            retrieved_at: Some("2026-09-05T22:51:21Z".into()),
            attempted_at: Some("2026-09-05T22:51:21Z".into()),
            etag: Some("\"fixture\"".into()),
            last_modified: Some("Sun, 30 Aug 2026 06:00:29 GMT".into()),
            upstream_version: None,
            message: None,
        },
    }
}

#[test]
fn canonical_symbol_and_hgnc_return_three_submission_rows() {
    let (section, outcome) = project("ODC1", Some("HGNC:8109"), data());
    assert_eq!(section.assertions.len(), 3);
    assert_eq!(section.total_matching_assertions, 3);
    assert!(!section.truncated);
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Data
    );
}

#[test]
fn missing_hgnc_uses_unique_symbol_identity() {
    let (section, _) = project("odc1", None, data());
    assert_eq!(section.assertions.len(), 3);
}

#[test]
fn one_sided_identity_match_is_inconclusive() {
    for (symbol, hgnc) in [("ODC1", "HGNC:42"), ("OTHER", "HGNC:8109")] {
        let (section, outcome) = project(symbol, Some(hgnc), data());
        assert_eq!(section.status.operation, GenCcOperation::IdentityMatch);
        assert_eq!(section.status.result, GenCcResult::Unknown);
        assert!(section.assertions.is_empty());
        assert_eq!(
            outcome.outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Unavailable
        );
    }
}
