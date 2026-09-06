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
        lease: None,
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

#[test]
fn stale_positive_and_zero_outcomes_preserve_the_lifecycle_message() {
    let mut positive = data();
    positive.status.freshness = GenCcFreshness::Stale;
    positive.status.operation = GenCcOperation::RefreshDeferred;
    positive.status.message = Some(
        "GenCC refresh is still in progress; results come from the last validated dataset.".into(),
    );
    let (section, outcome) = project("ODC1", Some("HGNC:8109"), positive);
    assert_eq!(outcome.message(), section.status.message.as_deref());
    assert_eq!(outcome.sources(), &["GenCC"]);

    let mut zero = data();
    zero.status.freshness = GenCcFreshness::Stale;
    zero.status.message =
        Some("GenCC refresh failed; results come from the last validated dataset.".into());
    let (section, outcome) = project("NOTFOUND", None, zero);
    assert_eq!(section.status.result, GenCcResult::Empty);
    assert_eq!(outcome.message(), section.status.message.as_deref());
    assert!(outcome.sources().is_empty());
}

#[test]
fn assertion_cap_is_separate_from_the_total() {
    let mut reader = csv::Reader::from_reader(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/gencc/submissions-new-odc1.csv"
        ))
        .as_slice(),
    );
    let header = reader.headers().unwrap().clone();
    let base = reader.records().next().unwrap().unwrap();
    let mut bytes = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut bytes);
        writer.write_record(&header).unwrap();
        for index in 1..=101 {
            let mut fields = base.iter().map(str::to_string).collect::<Vec<_>>();
            fields[0] = format!("SGC-{index}");
            writer.write_record(fields).unwrap();
        }
        writer.flush().unwrap();
    }
    let data = GenCcData {
        dataset: Some(GenCcDataset::parse(&bytes, &AtomicBool::new(false)).unwrap()),
        status: data().status,
        lease: None,
    };
    let (section, _) = project("ODC1", Some("HGNC:8109"), data);
    assert_eq!(section.assertions.len(), 100);
    assert_eq!(section.total_matching_assertions, 101);
    assert!(section.truncated);
}
