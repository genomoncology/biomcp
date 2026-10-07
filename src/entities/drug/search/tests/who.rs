//! WHO structured-search bridge coverage.

use super::*;

#[test]
fn all_region_search_degrades_when_who_pq_data_is_absent_but_explicit_who_fails() {
    let unavailable = || BioMcpError::SourceUnavailable {
        source_name: "WHO Prequalification".to_string(),
        reason: "Could not prepare WHO Prequalification data.".to_string(),
        suggestion: "Run `biomcp who sync`.".to_string(),
    };

    // Region-less searches keep going with an empty WHO bucket and a
    // visible warning (ticket 1304, issue #288).
    let (client, degraded) = who_ready_for_region("imatinib", DrugRegion::All, Err(unavailable()))
        .expect("all-region search should degrade");
    assert!(client.is_none());
    assert!(degraded);
    let warning = who_pq_degradation_warning(&unavailable());
    assert!(warning.contains("omits the WHO section"));
    assert!(warning.contains("biomcp who sync"));
    assert!(
        warning.contains("WHO Prequalification"),
        "the warning should name the missing source: {warning}"
    );
    assert_eq!(empty_who_search_page().total, Some(0));
    assert!(empty_who_search_page().results.is_empty());

    // An explicit `--region who` search still fails loudly instead of
    // silently returning nothing.
    assert!(
        who_ready_for_region("imatinib", DrugRegion::Who, Err(unavailable())).is_err(),
        "explicit WHO search must not degrade"
    );

    // A ready client passes through untouched.
    let root = crate::test_support::TempDirGuard::new("who-degrade-ready");
    let (client, degraded) = who_ready_for_region(
        "imatinib",
        DrugRegion::Who,
        Ok(crate::sources::who_pq::WhoPqClient::from_root(root.path())),
    )
    .expect("ready client should pass through");
    assert!(client.is_some());
    assert!(!degraded);
}

#[test]
fn explicit_who_vaccine_search_skips_drug_identity_resolution() {
    assert!(!should_resolve_drug_identity(
        DrugRegion::Who,
        crate::sources::who_pq::WhoProductTypeFilter::Vaccine
    ));
    assert!(should_resolve_drug_identity(
        DrugRegion::Who,
        crate::sources::who_pq::WhoProductTypeFilter::Api
    ));
    assert!(should_resolve_drug_identity(
        DrugRegion::Eu,
        crate::sources::who_pq::WhoProductTypeFilter::Vaccine
    ));
    assert!(should_resolve_drug_identity(
        DrugRegion::All,
        crate::sources::who_pq::WhoProductTypeFilter::Vaccine
    ));
}

#[tokio::test]
async fn structured_who_search_stops_after_one_extra_match_and_reports_unknown_total() {
    let filters = DrugSearchFilters {
        indication: Some("malaria".into()),
        ..Default::default()
    };
    let fetch_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let fetch_count_for_closure = fetch_count.clone();

    let page = search_structured_who_page_with(
        &filters,
        2,
        0,
        crate::sources::who_pq::WhoProductTypeFilter::Both,
        move |_, _, page_offset| {
            let fetch_count = fetch_count_for_closure.clone();
            async move {
                fetch_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                match page_offset {
                    0 => Ok(SearchPage::offset(
                        vec![mychem_row("candidate-a"), mychem_row("candidate-b")],
                        Some(100),
                    )),
                    _ => Ok(SearchPage::offset(Vec::new(), Some(100))),
                }
            }
        },
        |name| match name {
            "candidate-a" => vec![who_row("W1", "Artemether"), who_row("W2", "Lumefantrine")],
            "candidate-b" => vec![who_row("W3", "Artesunate")],
            _ => Vec::new(),
        },
    )
    .await
    .expect("structured WHO search");

    assert_eq!(fetch_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(page.total, None);
    assert_eq!(page.results.len(), 2);
    assert_eq!(page.results[0].who_reference_number.as_deref(), Some("W1"));
    assert_eq!(page.results[1].who_reference_number.as_deref(), Some("W2"));
}

#[tokio::test]
async fn structured_who_search_reports_exact_total_when_mychem_is_exhausted() {
    let filters = DrugSearchFilters {
        indication: Some("malaria".into()),
        ..Default::default()
    };

    let page = search_structured_who_page_with(
        &filters,
        5,
        0,
        crate::sources::who_pq::WhoProductTypeFilter::Both,
        |_, _, page_offset| async move {
            match page_offset {
                0 => Ok(SearchPage::offset(vec![mychem_row("candidate-a")], Some(1))),
                _ => Ok(SearchPage::offset(Vec::new(), Some(1))),
            }
        },
        |name| match name {
            "candidate-a" => vec![who_row("W1", "Artemether/Lumefantrine")],
            _ => Vec::new(),
        },
    )
    .await
    .expect("structured WHO search");

    assert_eq!(page.total, Some(1));
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.results[0].who_reference_number.as_deref(), Some("W1"));
}

#[tokio::test]
async fn structured_who_search_with_api_filter_keeps_only_api_rows() {
    let filters = DrugSearchFilters {
        indication: Some("malaria".into()),
        ..Default::default()
    };

    let page = search_structured_who_page_with(
        &filters,
        5,
        0,
        crate::sources::who_pq::WhoProductTypeFilter::Api,
        |_, _, _| async { Ok(SearchPage::offset(vec![mychem_row("candidate-a")], Some(1))) },
        |name| match name {
            "candidate-a" => vec![
                who_row("W1", "Artemether"),
                who_api_row("WHOAPI-001", "Artesunate"),
            ],
            _ => Vec::new(),
        },
    )
    .await
    .expect("structured WHO API search");

    assert_eq!(page.total, Some(1));
    assert_eq!(page.results.len(), 1);
    let api_rows = page
        .results
        .into_iter()
        .filter(|row| row.who_product_id.is_some())
        .collect::<Vec<_>>();
    assert_eq!(api_rows.len(), 1);
    assert_eq!(api_rows[0].who_product_id.as_deref(), Some("WHOAPI-001"));
}

#[tokio::test]
async fn structured_who_search_with_finished_filter_keeps_only_finished_rows() {
    let filters = DrugSearchFilters {
        indication: Some("malaria".into()),
        ..Default::default()
    };

    let page = search_structured_who_page_with(
        &filters,
        5,
        0,
        crate::sources::who_pq::WhoProductTypeFilter::FinishedPharma,
        |_, _, _| async { Ok(SearchPage::offset(vec![mychem_row("candidate-a")], Some(1))) },
        |name| match name {
            "candidate-a" => vec![
                who_row("W1", "Artemether"),
                who_api_row("WHOAPI-001", "Artesunate"),
            ],
            _ => Vec::new(),
        },
    )
    .await
    .expect("structured WHO finished search");

    assert_eq!(page.total, Some(1));
    assert_eq!(page.results.len(), 1);
    let finished_rows = page
        .results
        .into_iter()
        .filter(|row| row.who_reference_number.is_some())
        .collect::<Vec<_>>();
    assert_eq!(finished_rows.len(), 1);
    assert_eq!(finished_rows[0].who_reference_number.as_deref(), Some("W1"));
}
