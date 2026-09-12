//! Tests for CTGov trial search helpers.

use super::super::super::test_support::*;
use super::super::COUNT_TRAVERSAL_PAGE_CAP;
use super::super::{prepare_ctgov_search_context, validate_trial_search};
use super::*;
use crate::entities::trial::ClinicalTrialSearchUnknownReason;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
        provider_total: biodata::ClinicalTrialProviderTotal::Present(1),
        studies,
        provider_cursor: biodata::ClinicalTrialProviderCursor::Absent,
        raw_study_count: 1,
        verification_incomplete: false,
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
        cursor_unusable: false,
        exhausted: false,
        pages_fetched: 1,
        provider_total_seen: false,
    };
    let page = CtGovRawPage {
        provider_total: biodata::ClinicalTrialProviderTotal::Absent,
        studies: Vec::new(),
        provider_cursor: biodata::ClinicalTrialProviderCursor::Present(SENTINEL.into()),
        raw_study_count: 0,
        verification_incomplete: false,
    };
    assert!(!format!("{worker:?} {page:?}").contains(SENTINEL));
}

fn ctgov_studies(
    values: Vec<serde_json::Value>,
) -> Vec<biodata::ClinicalTrialsGovApiV2SearchResult> {
    let bytes = serde_json::to_vec(&serde_json::json!({"studies": values})).unwrap();
    let filters = biodata::ClinicalTrialSearchFilters::new(
        biodata::ClinicalTrialSearchFilterFields {
            condition: Some("fixture".into()),
            ..Default::default()
        },
        Default::default(),
    )
    .unwrap();
    let plan = biodata::ClinicalTrialsGovApiV2SearchPlan::new(&filters, 50, None, true).unwrap();
    biodata::ClinicalTrialsGovApiV2SearchPage::parse(&plan, &bytes, &Default::default())
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
        provider_total: total_count.map_or(biodata::ClinicalTrialProviderTotal::Absent, |value| {
            biodata::ClinicalTrialProviderTotal::Present(value as u64)
        }),
        studies: ctgov_studies(studies),
        provider_cursor: next_page_token
            .map_or(biodata::ClinicalTrialProviderCursor::Absent, |value| {
                biodata::ClinicalTrialProviderCursor::Present(value.to_string())
            }),
        raw_study_count,
        verification_incomplete: false,
    }
}

async fn alias_stability_fixture() -> (
    String,
    std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind alias stability fixture");
    let base = format!("http://{}", listener.local_addr().unwrap());
    let requests = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = captured.clone();
            tokio::spawn(async move {
                let mut bytes = vec![0_u8; 16 * 1024];
                let len = stream.read(&mut bytes).await.unwrap();
                let request = String::from_utf8_lossy(&bytes[..len]).into_owned();
                captured.lock().unwrap().push(request.clone());
                let expanded = request.contains("expanded");
                let later = request.contains("pageToken=");
                let (id, status, cursor) = match (expanded, later) {
                    (false, false) => ("NCT00000001", "COMPLETED", Some("requested-2")),
                    (false, true) => ("NCT00000002", "RECRUITING", None),
                    (true, false) => ("NCT00000003", "ACTIVE_NOT_RECRUITING", Some("expanded-2")),
                    (true, true) => ("NCT00000004", "NOT_YET_RECRUITING", None),
                };
                let mut body = serde_json::json!({
                    "studies": [{
                        "protocolSection": {
                            "identificationModule": {"nctId": id, "briefTitle": id},
                            "statusModule": {"overallStatus": status},
                            "eligibilityModule": {
                                "minimumAge": "18 Years",
                                "maximumAge": "75 Years"
                            }
                        }
                    }],
                    "totalCount": 2
                });
                if let Some(cursor) = cursor {
                    body["nextPageToken"] = serde_json::Value::String(cursor.into());
                }
                let body = body.to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            });
        }
    });
    (base, requests, task)
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
        let page = finish_ctgov_single_page(state, &context, limit, 0).unwrap();

        assert_eq!(page.total.value(), Some(200));
        assert_eq!(page.total.precision(), "approximate");
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
    let page = finish_ctgov_single_page(state, &context, 5, 0).unwrap();

    assert_eq!(page.total.value(), None);
    assert_eq!(page.total.reason(), Some("provider_omitted_total"));
    assert_eq!(page.continuation.cursor_value(), Some("p2"));
}

#[test]
fn empty_provider_page_with_nonblank_cursor_remains_continuable() {
    let filters = TrialSearchFilters {
        condition: Some("melanoma".into()),
        ..Default::default()
    };
    let (context, worker) = single_ctgov_context_and_worker(&filters);
    let mut state = CtGovSinglePageState::new(None, 0, true);
    apply_ctgov_single_page(
        &mut state,
        &context,
        &worker,
        5,
        filtered_page(Vec::new(), Some("p2"), Some(2)),
    );
    let page = finish_ctgov_single_page(state, &context, 5, 0).unwrap();
    assert_eq!(page.continuation.cursor_value(), Some("p2"));
    assert_eq!(
        (page.total.value(), page.total.precision()),
        (Some(2), "exact")
    );
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
    let page = finish_ctgov_single_page(state, &context, 3, 1).unwrap();

    assert_eq!(page.results.len(), 2);
    assert_eq!(page.continuation.cursor_value(), Some("p2"));
}

#[test]
fn direct_total_precedence_keeps_fail_open_ahead_of_traversal_cap() {
    let filters = TrialSearchFilters {
        condition: Some("melanoma".into()),
        criteria: Some("BRAF".into()),
        age: Some(50.0),
        ..Default::default()
    };
    let (context, _) = single_ctgov_context_and_worker(&filters);
    let mut state = CtGovSinglePageState::new(None, 0, true);
    state.provider_total = Some(biodata::ClinicalTrialProviderTotal::Present(100));
    state.verification_incomplete = true;
    state.traversal_capped = true;
    let page = finish_ctgov_single_page(state, &context, 5, 0).unwrap();
    assert_eq!(page.total.reason(), Some("incomplete_local_verification"));
}

#[test]
fn unrequested_total_stays_unknown_during_local_filtering() {
    let filters = age_filtered_ctgov_filters();
    let (context, _) = single_ctgov_context_and_worker(&filters);
    let mut state = CtGovSinglePageState::new(None, 0, false);
    state.provider_total = Some(biodata::ClinicalTrialProviderTotal::NotRequested);
    state.page_token = Some("next".into());
    let page = finish_ctgov_single_page(state, &context, 5, 0).unwrap();
    assert_eq!(page.total.reason(), Some("total_not_requested"));
    assert_eq!(page.continuation.cursor_value(), Some("next"));
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
        let page = finish_ctgov_single_page(state, &context, limit, 0).unwrap();

        if limit == 10 {
            assert_eq!(page.total.value(), Some(20));
            assert_eq!(page.total.precision(), "approximate");
        } else {
            assert_eq!(page.total.value(), Some(20));
            assert_eq!(page.total.precision(), "exact");
        }
    }
}

#[test]
fn count_all_returns_approximate_for_age_only_filters() {
    assert_eq!(
        ctgov_count_from_native_total(Some(250), true)
            .unwrap()
            .precision(),
        "approximate"
    );
}

#[test]
fn count_all_returns_exact_for_no_post_filters() {
    assert_eq!(
        ctgov_count_from_native_total(Some(494), false)
            .unwrap()
            .precision(),
        "exact"
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
        assert_eq!(count.value(), None);
        assert_eq!(
            count.unknown_reason(),
            Some(ClinicalTrialSearchUnknownReason::ProviderOmittedTotal)
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
        "totalCount": 2,
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
    assert_eq!(page.continuation.status(), "unavailable");
    assert_eq!(page.continuation.reason(), Some("unusable_provider_cursor"));
    assert_eq!(page.results[0].nct_id(), "NCT00000001");
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
        count.unknown_reason(),
        Some(ClinicalTrialSearchUnknownReason::TraversalLimitReached)
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
        count.unknown_reason(),
        Some(ClinicalTrialSearchUnknownReason::TraversalLimitReached)
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
    let page = finish_ctgov_single_page(state, &context, 1, 0).unwrap();

    assert_eq!(page.results.len(), 1);
    assert_eq!(page.total.value(), Some(1));
    assert_eq!(page.total.precision(), "exact");
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

    assert_eq!(unique_nct_ids.len(), 3);
}

#[test]
fn unusable_alias_cursor_overrides_another_workers_offset_replay() {
    let mut workers = ctgov_workers(
        None,
        &[
            trial_alias("requested", TrialAliasSource::Requested),
            trial_alias("expanded", TrialAliasSource::DrugBankSynonym),
        ],
    );
    apply_worker_cursor(
        &mut workers[0],
        biodata::ClinicalTrialProviderCursor::Present(" \t ".into()),
    );
    workers[1].next_page_token = Some("usable-for-only-one-worker".into());

    let continuation = ctgov_union_continuation(&workers, false, false, false, 7).unwrap();
    assert_eq!(continuation.status(), "unavailable");
    assert_eq!(continuation.reason(), Some("unusable_provider_cursor"));
    assert_eq!(continuation.offset_value(), None);
}

#[test]
fn buffered_alias_rows_offer_offset_replay_after_workers_exhaust() {
    let mut workers = ctgov_workers(
        None,
        &[trial_alias("requested", TrialAliasSource::Requested)],
    );
    workers[0].exhausted = true;
    let continuation = ctgov_union_continuation(&workers, true, false, false, 5).unwrap();
    assert_eq!(continuation.status(), "offset");
    assert_eq!(continuation.offset_value(), Some(5));
}

#[test]
fn rejected_alias_coverage_overrides_active_worker_and_cap() {
    let workers = ctgov_workers(
        None,
        &[trial_alias("requested", TrialAliasSource::Requested)],
    );
    let with_active = ctgov_union_continuation(&workers, true, true, false, 5).unwrap();
    assert_eq!(with_active.reason(), Some("incomplete_source_coverage"));
    let with_cap = ctgov_union_continuation(&workers, false, true, true, 5).unwrap();
    assert_eq!(with_cap.reason(), Some("incomplete_source_coverage"));
}

#[test]
fn alias_cap_invalidates_a_retained_provider_token() {
    let mut workers = ctgov_workers(
        None,
        &[trial_alias("requested", TrialAliasSource::Requested)],
    );
    workers[0].next_page_token = Some("provider-token".into());
    let continuation = ctgov_union_continuation(&workers, false, false, true, 5).unwrap();
    assert_eq!(continuation.status(), "unavailable");
    assert_eq!(continuation.reason(), Some("traversal_limit_reached"));
    assert_eq!(continuation.cursor_value(), None);
    assert_eq!(continuation.offset_value(), None);
}

#[test]
fn page_twenty_exhaustion_is_terminal_while_a_cursor_is_capped() {
    let aliases = [trial_alias("requested", TrialAliasSource::Requested)];
    let mut exhausted = ctgov_workers(None, &aliases).remove(0);
    exhausted.pages_fetched = CTGOV_MAX_PAGE_FETCHES;
    apply_worker_cursor(&mut exhausted, biodata::ClinicalTrialProviderCursor::Absent);
    let exhausted_capped = cap_continuable_worker(&mut exhausted);
    let terminal =
        ctgov_union_continuation(&[exhausted], false, false, exhausted_capped, 20).unwrap();
    let exact = completed_ctgov_union_count(false, false, 20).unwrap();
    assert_eq!((exact.value(), exact.precision()), (Some(20), "exact"));
    assert_eq!(terminal.status(), "terminal");
    let mut continuable = ctgov_workers(None, &aliases).remove(0);
    continuable.pages_fetched = CTGOV_MAX_PAGE_FETCHES;
    apply_worker_cursor(
        &mut continuable,
        biodata::ClinicalTrialProviderCursor::Present("page-21".into()),
    );
    assert!(cap_continuable_worker(&mut continuable));
    let capped = ctgov_union_continuation(&[continuable], false, false, true, 20).unwrap();
    assert_eq!(capped.reason(), Some("traversal_limit_reached"));
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn bounded_alias_pages_are_stable_across_product_page_invocations() {
    let (base, requests, server) = alias_stability_fixture().await;
    let _env = CtGovFixtureEnv::set(&base);
    let client = ClinicalTrialsClient::new().expect("CTGov fixture client");
    let filters = TrialSearchFilters {
        condition: Some("melanoma".into()),
        ..Default::default()
    };
    let normalized = validate_trial_search(&filters).unwrap();
    let context = prepare_ctgov_search_context(&normalized).unwrap();
    let aliases = [
        trial_alias("requested", TrialAliasSource::Requested),
        trial_alias("expanded", TrialAliasSource::DrugBankSynonym),
    ];

    let first = search_page_with_ctgov_union(
        &client,
        &filters,
        &context,
        Some("melanoma"),
        &aliases,
        2,
        0,
    )
    .await
    .unwrap();
    let second = search_page_with_ctgov_union(
        &client,
        &filters,
        &context,
        Some("melanoma"),
        &aliases,
        2,
        2,
    )
    .await
    .unwrap();
    let combined = search_page_with_ctgov_union(
        &client,
        &filters,
        &context,
        Some("melanoma"),
        &aliases,
        4,
        0,
    )
    .await
    .unwrap();

    let paged = first
        .results
        .iter()
        .chain(&second.results)
        .map(|row| row.nct_id())
        .collect::<Vec<_>>();
    let one_page = combined
        .results
        .iter()
        .map(|row| row.nct_id())
        .collect::<Vec<_>>();
    assert_eq!(paged, one_page);
    assert_eq!(
        one_page,
        ["NCT00000002", "NCT00000003", "NCT00000004", "NCT00000001"]
    );
    assert_eq!(first.continuation.offset_value(), Some(2));
    assert_eq!(second.continuation.status(), "terminal");
    assert_eq!(combined.continuation.status(), "terminal");
    server.abort();

    let requests = requests.lock().unwrap();
    assert!(!requests.is_empty());
    assert!(
        requests
            .iter()
            .all(|request| request.contains("pageSize=100"))
    );
}
