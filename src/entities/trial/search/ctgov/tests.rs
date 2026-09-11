//! Tests for CTGov trial search helpers.

use super::super::super::test_support::*;
use super::super::{prepare_ctgov_search_context, validate_trial_search};
use super::*;
use crate::entities::trial::TrialCountUnknownReason;

fn trial_alias(label: &str, source: TrialAliasSource) -> TrialAlias {
    TrialAlias {
        label: label.into(),
        source,
    }
}

#[test]
fn raw_page_debug_redacts_ignored_untrusted_values() {
    const SENTINEL: &str = "RAW-PAGE-PRIVACY-SENTINEL-0115";
    let studies = ctgov_studies(vec![serde_json::json!({
        "protocolSection": {
            "identificationModule": {"nctId": "NCT00000001", "briefTitle": "Fixture"},
            "statusModule": {"overallStatus": "RECRUITING"},
            "contactsLocationsModule": {
                "overallOfficials": [{"name": SENTINEL}],
                "locations": [{"facility": SENTINEL, "contacts": [{"name": SENTINEL}]}]
            }
        }
    })]);
    let page = CtGovRawPage {
        total_count: Some(1),
        studies,
        next_page_token: None,
        raw_study_count: 1,
    };
    assert!(!format!("{page:?}").contains(SENTINEL));
}

#[test]
fn paging_wrapper_debug_redacts_condition_alias_label_and_cursor() {
    const SENTINEL: &str = "CTGOV-PAGING-PRIVATE-SENTINEL-0117";
    let worker = CtGovWorkerState {
        condition_query: Some(SENTINEL.into()),
        intervention_query: Some(SENTINEL.into()),
        intervention_source: SENTINEL,
        matched_intervention_label: Some(SENTINEL.into()),
        next_page_token: Some(SENTINEL.into()),
        exhausted: false,
        pages_fetched: 1,
    };
    let page = CtGovRawPage {
        total_count: None,
        studies: Vec::new(),
        next_page_token: Some(SENTINEL.into()),
        raw_study_count: 0,
    };
    assert!(!format!("{worker:?} {page:?}").contains(SENTINEL));
}

fn ctgov_studies(
    values: Vec<serde_json::Value>,
) -> Vec<biodata::ClinicalTrialsGovApiV2SearchResult> {
    let bytes = serde_json::to_vec(&serde_json::json!({"studies": values})).unwrap();
    biodata::ClinicalTrialsGovApiV2SearchPage::parse(&bytes, &Default::default())
        .expect("valid CTGov search page")
        .results()
        .unwrap_or_default()
        .to_vec()
}

fn filtered_page(
    studies: Vec<serde_json::Value>,
    next_page_token: Option<&str>,
    total_count: Option<usize>,
) -> CtGovRawPage {
    let raw_study_count = studies.len();
    CtGovRawPage {
        total_count,
        studies: ctgov_studies(studies),
        next_page_token: next_page_token.map(str::to_string),
        raw_study_count,
    }
}

fn single_ctgov_context_and_worker(
    filters: &TrialSearchFilters,
) -> (CtGovSearchContext, CtGovWorkerState) {
    let normalized = validate_trial_search(filters).expect("filters should validate");
    let context = prepare_ctgov_search_context(&normalized).expect("context should build");
    let worker = ctgov_workers(
        raw_condition_query(filters),
        &raw_intervention_query(filters)
            .map(|value| vec![trial_alias(value, TrialAliasSource::Requested)])
            .unwrap_or_default(),
    )
    .into_iter()
    .next()
    .expect("single worker");
    (context, worker)
}

#[test]
fn trial_location_requires_a_positive_distance() {
    let valid = TrialSearchFilters {
        lat: Some(42.36),
        lon: Some(-71.06),
        distance: Some(1),
        ..Default::default()
    };
    super::super::biodata_filters(&valid, None)
        .expect("a positive distance should validate locally");

    let zero_distance = TrialSearchFilters {
        distance: Some(0),
        ..valid
    };
    let err = super::super::biodata_filters(&zero_distance, None)
        .expect_err("zero distance must fail before any provider work");
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
}

#[test]
fn trial_search_rejects_absurd_offset_before_provider_setup() {
    let err = validate_search_page_args(5, 100_001, None)
        .expect_err("an absurd offset must fail before CTGov client construction");

    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("--offset"));
    validate_search_page_args(5, 100_000, None)
        .expect("the maximum trial offset must remain valid");
}

#[test]
fn trial_numeric_filters_are_validated_before_request_construction() {
    for age in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -1.0,
        -1e-50,
        150.000_001,
        151.0,
    ] {
        let filters = TrialSearchFilters {
            age: Some(age),
            ..Default::default()
        };
        let err = validate_trial_search(&filters)
            .err()
            .expect("invalid age should fail before request construction");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    for age in [0.0, 0.5, 150.0] {
        validate_trial_search(&TrialSearchFilters {
            age: Some(age),
            ..Default::default()
        })
        .expect("valid age boundary should pass");
    }

    for (lat, lon) in [
        (f64::NAN, 0.0),
        (f64::INFINITY, 0.0),
        (f64::NEG_INFINITY, 0.0),
        (-91.0, 0.0),
        (91.0, 0.0),
        (0.0, f64::NAN),
        (0.0, f64::INFINITY),
        (0.0, f64::NEG_INFINITY),
        (0.0, -181.0),
        (0.0, 181.0),
    ] {
        let filters = TrialSearchFilters {
            lat: Some(lat),
            lon: Some(lon),
            distance: Some(1),
            ..Default::default()
        };
        let err = validate_trial_search(&filters)
            .err()
            .expect("invalid coordinates should fail before request construction");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    for (lat, lon) in [
        (Some(f64::NAN), None),
        (Some(91.0), None),
        (None, Some(f64::NAN)),
        (None, Some(181.0)),
    ] {
        let err = validate_trial_search(&TrialSearchFilters {
            lat,
            lon,
            ..Default::default()
        })
        .err()
        .expect("standalone invalid coordinate should report its numeric domain");
        assert!(err.to_string().contains("geography"));
    }

    let nci_invalid_coordinates = TrialSearchFilters {
        lat: Some(91.0),
        lon: Some(0.0),
        distance: Some(1),
        source: TrialSource::NciCts,
        ..Default::default()
    };
    let err = validate_trial_search(&nci_invalid_coordinates)
        .err()
        .expect("the shared coordinate guard should run for NCI");
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));

    for (lat, lon) in [(-90.0, -180.0), (90.0, 180.0)] {
        validate_trial_search(&TrialSearchFilters {
            lat: Some(lat),
            lon: Some(lon),
            distance: Some(1),
            ..Default::default()
        })
        .expect("valid coordinate boundaries should pass");
    }
}

#[test]
fn age_filter_uses_native_total_semantics_across_limits() {
    let filters = age_filtered_ctgov_filters();
    let (context, worker) = single_ctgov_context_and_worker(&filters);

    for limit in [10, 20, 50] {
        let mut state = CtGovSinglePageState::new(None, 0, true);
        apply_ctgov_single_page(
            &mut state,
            &context,
            &worker,
            limit,
            filtered_page(
                studies_with_age_matches(100, 60, &limit.to_string()),
                Some("p2"),
                Some(200),
            ),
        );
        let page = finish_ctgov_single_page(state, &context, limit, 0);

        assert_eq!(page.total, Some(200));
    }
}

#[test]
fn ctgov_cursor_without_a_reported_total_keeps_the_provider_token() {
    let filters = age_filtered_ctgov_filters();
    let (context, worker) = single_ctgov_context_and_worker(&filters);
    let mut state = CtGovSinglePageState::new(Some("p1".into()), 0, true);
    apply_ctgov_single_page(
        &mut state,
        &context,
        &worker,
        5,
        filtered_page(studies_with_age_matches(5, 5, "20"), Some("p2"), None),
    );
    let page = finish_ctgov_single_page(state, &context, 5, 0);

    assert_eq!(page.total, None);
    assert_eq!(page.next_page_token.as_deref(), Some("p2"));
}

#[test]
fn ctgov_cursor_preserves_next_page_token_after_offset_full_page_consumption() {
    let filters = age_filtered_ctgov_filters();
    let (context, worker) = single_ctgov_context_and_worker(&filters);
    let mut state = CtGovSinglePageState::new(None, 1, true);
    apply_ctgov_single_page(
        &mut state,
        &context,
        &worker,
        3,
        filtered_page(studies_with_age_matches(3, 3, "21"), Some("p2"), Some(10)),
    );
    let page = finish_ctgov_single_page(state, &context, 3, 1);

    assert_eq!(page.results.len(), 2);
    assert_eq!(page.next_page_token, Some("p2".into()));
}

#[test]
fn age_filter_total_returns_native_total_when_exhausted() {
    let filters = age_filtered_ctgov_filters();
    let (context, worker) = single_ctgov_context_and_worker(&filters);

    for (limit, first_prefix, second_prefix) in [(10, "31", "32"), (50, "41", "42")] {
        let mut state = CtGovSinglePageState::new(None, 0, true);
        apply_ctgov_single_page(
            &mut state,
            &context,
            &worker,
            limit,
            filtered_page(
                studies_with_age_matches(10, 7, first_prefix),
                Some("p2"),
                Some(20),
            ),
        );
        apply_ctgov_single_page(
            &mut state,
            &context,
            &worker,
            limit,
            filtered_page(
                studies_with_age_matches(10, 5, second_prefix),
                None,
                Some(20),
            ),
        );
        let page = finish_ctgov_single_page(state, &context, limit, 0);

        assert_eq!(page.total, Some(20));
    }
}

#[test]
fn count_all_returns_approximate_for_age_only_filters() {
    assert_eq!(
        ctgov_count_from_native_total(Some(250), true),
        TrialCount::Approximate(250)
    );
}

#[test]
fn count_all_returns_exact_for_no_post_filters() {
    assert_eq!(
        ctgov_count_from_native_total(Some(494), false),
        TrialCount::Exact(494)
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn count_all_keeps_an_omitted_provider_total_unknown() {
    // Synthetic first-page envelope reproducing receipted provider optionality.
    let (base, requests, server) = ctgov_json_fixture(r#"{"studies":[]}"#).await;
    let _env = CtGovFixtureEnv::set(&base);
    let client = ClinicalTrialsClient::new().expect("CTGov fixture client");
    for filters in [
        TrialSearchFilters {
            condition: Some("melanoma".into()),
            ..Default::default()
        },
        TrialSearchFilters {
            condition: Some("melanoma".into()),
            age: Some(0.5),
            ..Default::default()
        },
    ] {
        let count = count_all_with_ctgov_client(&client, &filters, COUNT_TRAVERSAL_PAGE_CAP)
            .await
            .expect("synthetic CTGov count response");
        assert_eq!(
            count,
            TrialCount::Unknown(TrialCountUnknownReason::ProviderOmittedTotal)
        );
    }
    server.abort();
    let requests = requests.lock().expect("lock fixture requests");
    assert_eq!(requests.len(), 2);
    assert!(
        requests
            .iter()
            .all(|request| request.contains("countTotal=true") && request.contains("pageSize=1"))
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn trim_empty_provider_cursor_stops_without_repeating_page_one() {
    let body = serde_json::json!({
        "studies": [ctgov_search_study_fixture("NCT00000001", "18 Years", "75 Years")],
        "totalCount": 1,
        "nextPageToken": " \t "
    })
    .to_string();
    let (base, requests, server) = ctgov_json_fixture(body).await;
    let _env = CtGovFixtureEnv::set(&base);
    let client = ClinicalTrialsClient::new().expect("CTGov fixture client");
    let filters = TrialSearchFilters {
        condition: Some("melanoma".into()),
        ..Default::default()
    };

    let page = search_page_with_ctgov_client(&client, &filters, 2, 0, None)
        .await
        .expect("synthetic CTGov search response");
    assert_eq!(page.results.len(), 1);
    assert!(page.next_page_token.is_none());
    assert_eq!(page.results[0].nct_id, "NCT00000001");
    server.abort();
    assert_eq!(requests.lock().expect("lock fixture requests").len(), 1);
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn expensive_single_query_returns_the_traversal_limit_reason_at_its_cap() {
    let (base, requests, server) = ctgov_json_fixture(r#"{"studies":[]}"#).await;
    let _env = CtGovFixtureEnv::set(&base);
    let client = ClinicalTrialsClient::new().expect("CTGov fixture client");
    let filters = TrialSearchFilters {
        condition: Some("melanoma".into()),
        criteria: Some("BRAF V600E".into()),
        ..Default::default()
    };
    let count = count_all_with_ctgov_client(&client, &filters, 0)
        .await
        .expect("bounded expensive CTGov count");
    assert_eq!(
        count,
        TrialCount::Unknown(TrialCountUnknownReason::TraversalLimitReached)
    );
    server.abort();
    assert!(requests.lock().expect("lock fixture requests").is_empty());
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn alias_union_returns_the_traversal_limit_reason_at_its_cap() {
    let (base, requests, server) = ctgov_json_fixture(r#"{"studies":[]}"#).await;
    let _env = CtGovFixtureEnv::set(&base);
    let client = ClinicalTrialsClient::new().expect("CTGov fixture client");
    let filters = TrialSearchFilters {
        condition: Some("melanoma".into()),
        ..Default::default()
    };
    let normalized = validate_trial_search(&filters).expect("valid CTGov filters");
    let context = prepare_ctgov_search_context(&normalized).expect("CTGov context");
    let aliases = [
        trial_alias("requested", TrialAliasSource::Requested),
        trial_alias("expanded", TrialAliasSource::DrugBankSynonym),
    ];
    let count = count_all_with_ctgov_union(
        &client,
        &filters,
        &context,
        raw_condition_query(&filters),
        &aliases,
        1,
    )
    .await
    .expect("bounded alias-union CTGov count");
    assert_eq!(
        count,
        TrialCount::Unknown(TrialCountUnknownReason::TraversalLimitReached)
    );
    server.abort();
    assert!(requests.lock().expect("lock fixture requests").is_empty());
}

#[test]
fn alias_expansion_next_page_error_is_actionable() {
    let err = fanout_next_page_error();
    assert!(err.to_string().contains(
        "--next-page is not supported when CTGov intervention alias expansion uses multiple queries"
    ));
    assert!(err.to_string().contains("--no-alias-expand"));
}

#[test]
fn ctgov_worker_outcome_skips_only_expanded_parser_rejections() {
    let workers = ctgov_workers(
        None,
        &[
            trial_alias("requested", TrialAliasSource::Requested),
            trial_alias("expanded", TrialAliasSource::DrugBankSynonym),
        ],
    );
    let rejection = || BioMcpError::CtGovInterventionQueryRejected;

    assert!(
        handle_ctgov_worker_outcome(1, &workers[1], Err(rejection()))
            .expect("expanded rejection should be tolerated")
            .is_none()
    );
    assert!(matches!(
        handle_ctgov_worker_outcome(0, &workers[0], Err(rejection())),
        Err(BioMcpError::CtGovInterventionQueryRejected)
    ));
    assert!(matches!(
        handle_ctgov_worker_outcome(
            1,
            &workers[1],
            Err(BioMcpError::Api {
                api: "clinicaltrials.gov".into(),
                message: "HTTP 400 unrelated".into(),
            }),
        ),
        Err(BioMcpError::Api { .. })
    ));

    let json_error = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    assert!(matches!(
        handle_ctgov_worker_outcome(
            1,
            &workers[1],
            Err(BioMcpError::ApiJson {
                api: "clinicaltrials.gov".into(),
                source: json_error,
            }),
        ),
        Err(BioMcpError::ApiJson { .. })
    ));

    let transport_error = reqwest::Client::new()
        .get("http://[::1")
        .build()
        .unwrap_err();
    assert!(matches!(
        handle_ctgov_worker_outcome(1, &workers[1], Err(BioMcpError::Http(transport_error)),),
        Err(BioMcpError::Http(_))
    ));
}

#[test]
fn ctgov_workers_keep_literal_condition_during_intervention_fanout() {
    let workers = ctgov_workers(
        Some("Rett Syndrome"),
        &[
            trial_alias("ticket-415-requested", TrialAliasSource::Requested),
            trial_alias("ticket-415-alternate", TrialAliasSource::OpenFdaBrand),
        ],
    );

    assert_eq!(workers.len(), 2);
    assert_eq!(workers[0].condition_query.as_deref(), Some("Rett Syndrome"));
    assert_eq!(
        workers[0].intervention_query.as_deref(),
        Some("ticket-415-requested")
    );
    assert_eq!(
        workers[0].matched_intervention_label.as_deref(),
        Some("ticket-415-requested")
    );
    assert_eq!(workers[1].condition_query.as_deref(), Some("Rett Syndrome"));
    assert_eq!(
        workers[1].matched_intervention_label.as_deref(),
        Some("ticket-415-alternate")
    );
}

#[test]
fn ctgov_workers_do_not_label_literal_single_intervention() {
    let workers = ctgov_workers(
        None,
        &[trial_alias("pembrolizumab", TrialAliasSource::Requested)],
    );

    assert_eq!(workers.len(), 1);
    assert_eq!(workers[0].condition_query, None);
    assert_eq!(
        workers[0].intervention_query.as_deref(),
        Some("pembrolizumab")
    );
    assert_eq!(workers[0].matched_intervention_label, None);
}

#[tokio::test]
async fn no_alias_expand_builds_one_literal_requested_name_worker() {
    let filters = TrialSearchFilters {
        intervention: Some("HRS 4642".into()),
        source: TrialSource::ClinicalTrialsGov,
        no_alias_expand: true,
        ..Default::default()
    };
    let aliases = resolve_ctgov_intervention_aliases(&filters)
        .await
        .expect("no-expand resolution");
    let workers = ctgov_workers(None, &aliases);
    let normalized = validate_trial_search(&filters).expect("filters should validate");
    let context = prepare_ctgov_search_context(&normalized).expect("context should build");
    let plan = build_ctgov_search_plan(
        &filters,
        &context,
        None,
        workers[0].intervention_query.as_deref(),
        None,
        10,
        true,
    )
    .expect("BioData plan");

    assert_eq!(workers.len(), 1);
    assert_eq!(workers[0].intervention_source, "requested");
    assert!(plan.query_pairs().contains(&("query.intr", "\"HRS 4642\"")));
}

#[test]
fn literal_condition_search_still_reports_limit_one_total() {
    let filters = TrialSearchFilters {
        condition: Some("Rett Syndrome".into()),
        ..Default::default()
    };
    let (context, worker) = single_ctgov_context_and_worker(&filters);
    let mut state = CtGovSinglePageState::new(None, 0, !filters.no_count_total);

    apply_ctgov_single_page(
        &mut state,
        &context,
        &worker,
        1,
        filtered_page(
            vec![ctgov_search_study_fixture(
                "NCT00000470",
                "18 Years",
                "75 Years",
            )],
            None,
            None,
        ),
    );
    let page = finish_ctgov_single_page(state, &context, 1, 0);

    assert_eq!(page.results.len(), 1);
    assert_eq!(page.total, Some(1));
}

#[test]
fn search_path_rejects_next_page_when_alias_expansion_uses_multiple_queries() {
    let err = fanout_next_page_error();
    assert!(err.to_string().contains("--next-page is not supported"));
    assert!(err.to_string().contains("--no-alias-expand"));
}

#[test]
fn alias_union_provenance_uses_the_validated_nct_id() {
    let study = ctgov_studies(vec![ctgov_search_study_fixture(
        "NCT00000001",
        "18 Years",
        "75 Years",
    )])
    .remove(0);
    let mut matched_labels = HashMap::new();
    matched_labels.insert("NCT00000001".to_string(), Some("requested".to_string()));
    let mut merged_rows = Vec::new();
    let mut merged_index = HashMap::new();

    push_ctgov_union_rows(
        &mut merged_rows,
        &mut merged_index,
        &matched_labels,
        vec![study],
    );

    assert_eq!(
        merged_rows[0].matched_intervention_label.as_deref(),
        Some("requested")
    );
}

#[test]
fn alias_union_count_returns_exact_unique_total_when_exhausted() {
    let mut unique_nct_ids = std::collections::HashSet::new();

    add_unique_ctgov_nct_ids(
        &mut unique_nct_ids,
        vec![
            ctgov_studies(vec![ctgov_search_study_fixture(
                "NCT00000001",
                "18 Years",
                "75 Years",
            )])
            .remove(0),
            ctgov_studies(vec![ctgov_search_study_fixture(
                "NCT00000002",
                "18 Years",
                "75 Years",
            )])
            .remove(0),
        ],
    );
    add_unique_ctgov_nct_ids(
        &mut unique_nct_ids,
        vec![
            ctgov_studies(vec![ctgov_search_study_fixture(
                "NCT00000001",
                "18 Years",
                "75 Years",
            )])
            .remove(0),
            ctgov_studies(vec![ctgov_search_study_fixture(
                "NCT00000003",
                "18 Years",
                "75 Years",
            )])
            .remove(0),
        ],
    );

    assert_eq!(
        TrialCount::Exact(unique_nct_ids.len()),
        TrialCount::Exact(3)
    );
}

#[test]
fn skipped_expanded_worker_makes_search_and_count_totals_unknown() {
    let mut workers = ctgov_workers(
        None,
        &[
            trial_alias("requested", TrialAliasSource::Requested),
            trial_alias("expanded", TrialAliasSource::DrugBankSynonym),
        ],
    );
    for worker in &mut workers {
        worker.exhausted = true;
    }

    assert_eq!(ctgov_union_total(false, false, &workers, 2), Some(2));
    assert_eq!(ctgov_union_total(true, false, &workers, 2), None);
    assert_eq!(completed_ctgov_union_count(false, 2), TrialCount::Exact(2));
    assert_eq!(
        completed_ctgov_union_count(true, 2),
        TrialCount::Unknown(TrialCountUnknownReason::IncompleteCoverage)
    );
}
