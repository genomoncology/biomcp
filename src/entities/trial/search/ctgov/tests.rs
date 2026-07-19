//! Tests for CTGov trial search helpers.

use super::super::super::test_support::*;
use super::super::{prepare_ctgov_search_context, validate_trial_search};
use super::*;

fn trial_alias(label: &str, source: TrialAliasSource) -> TrialAlias {
    TrialAlias {
        label: label.into(),
        source,
    }
}

fn ctgov_studies(values: Vec<serde_json::Value>) -> Vec<CtGovStudy> {
    values
        .into_iter()
        .map(|value| serde_json::from_value(value).expect("valid CTGov study"))
        .collect()
}

fn filtered_page(
    studies: Vec<serde_json::Value>,
    next_page_token: Option<&str>,
    total_count: Option<usize>,
) -> CtGovFilteredPage {
    let raw_study_count = studies.len();
    CtGovFilteredPage {
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
    let context = prepare_ctgov_search_context(filters, &normalized).expect("context should build");
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
fn trial_numeric_filters_are_validated_before_request_construction() {
    for age in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 151.0] {
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
fn ctgov_query_term_broadens_mutation_across_discovery_fields() {
    let filters = TrialSearchFilters {
        mutation: Some("dMMR OR MSI-H".into()),
        criteria: Some("mismatch repair deficient".into()),
        ..Default::default()
    };

    let query = ctgov_query_term(&filters, None)
        .expect("query term should build")
        .expect("query term should not be empty");
    assert!(query.contains(
        "(AREA[EligibilityCriteria](\"dMMR\" OR \"MSI\\-H\") OR \
AREA[BriefTitle](\"dMMR\" OR \"MSI\\-H\") OR \
AREA[OfficialTitle](\"dMMR\" OR \"MSI\\-H\") OR \
AREA[BriefSummary](\"dMMR\" OR \"MSI\\-H\") OR \
AREA[Keyword](\"dMMR\" OR \"MSI\\-H\"))"
    ));
    assert!(query.contains("AREA[EligibilityCriteria](\"mismatch repair deficient\")"));
}

#[test]
fn ctgov_query_term_broadens_simple_mutation_across_discovery_fields() {
    let filters = TrialSearchFilters {
        mutation: Some("G12D".into()),
        ..Default::default()
    };

    let query = ctgov_query_term(&filters, None)
        .expect("query term should build")
        .expect("query term should not be empty");
    assert!(query.contains(
        "(AREA[EligibilityCriteria](\"G12D\") OR AREA[BriefTitle](\"G12D\") OR \
AREA[OfficialTitle](\"G12D\") OR AREA[BriefSummary](\"G12D\") OR AREA[Keyword](\"G12D\"))"
    ));
}

#[test]
fn ctgov_query_term_joins_multi_phase_filters_with_and() {
    let filters = TrialSearchFilters {
        condition: Some("melanoma".into()),
        ..Default::default()
    };

    let query = ctgov_query_term(&filters, Some(&["PHASE1".into(), "PHASE2".into()]))
        .expect("query term should build")
        .expect("query term should not be empty");
    assert!(query.contains("(AREA[Phase]PHASE1 AND AREA[Phase]PHASE2)"));
}

#[test]
fn build_ctgov_search_params_maps_all_shared_fields() {
    let filters = TrialSearchFilters {
        condition: Some("melanoma".into()),
        intervention: Some("HRS 4642".into()),
        facility: Some("Mayo Clinic".into()),
        status: Some("active".into()),
        phase: Some("1/2".into()),
        study_type: Some("Interventional".into()),
        sex: Some("female".into()),
        sponsor: Some("Acme Oncology".into()),
        sponsor_type: Some("industry".into()),
        mutation: Some("MSI-H".into()),
        criteria: Some("mismatch repair deficient".into()),
        results_available: true,
        lat: Some(42.3601),
        lon: Some(-71.0589),
        distance: Some(25),
        ..Default::default()
    };
    let normalized = validate_trial_search(&filters).expect("filters should validate");
    let context =
        prepare_ctgov_search_context(&filters, &normalized).expect("context should build");

    let params = build_ctgov_search_params(
        &filters,
        &context,
        raw_condition_query(&filters),
        raw_intervention_query(&filters),
        Some("cursor-1".into()),
        37,
        true,
    );

    assert_eq!(params.condition, filters.condition);
    assert_eq!(params.intervention.as_deref(), Some("\"HRS 4642\""));
    assert_eq!(params.facility, context.facility);
    assert_eq!(params.status, context.normalized_status);
    assert_eq!(params.agg_filters, context.agg_filters);
    assert_eq!(params.query_term, context.query_term);
    assert!(params.count_total);
    assert_eq!(params.page_token.as_deref(), Some("cursor-1"));
    assert_eq!(params.page_size, 37);
    assert_eq!(params.lat, filters.lat);
    assert_eq!(params.lon, filters.lon);
    assert_eq!(params.distance_miles, filters.distance);
}

#[test]
fn build_ctgov_search_params_quotes_interventions_as_single_essie_literals() {
    let filters = TrialSearchFilters {
        intervention: Some("placeholder".into()),
        ..Default::default()
    };
    let normalized = validate_trial_search(&filters).expect("filters should validate");
    let context =
        prepare_ctgov_search_context(&filters, &normalized).expect("context should build");

    for (input, expected) in [
        ("HRS 4642", "\"HRS 4642\""),
        ("name [salt]", "\"name \\[salt\\]\""),
        ("name (free base)", "\"name \\(free base\\)\""),
        ("alpha,beta", "\"alpha,beta\""),
        ("say \"name\"", "\"say \\\"name\\\"\""),
        (r"path\name", r#""path\\name""#),
        ("A+B-C:D/E", "\"A\\+B\\-C\\:D\\/E\""),
        ("AND OR NOT", "\"AND OR NOT\""),
    ] {
        let params =
            build_ctgov_search_params(&filters, &context, None, Some(input), None, 10, true);
        assert_eq!(
            params.intervention.as_deref(),
            Some(expected),
            "input: {input}"
        );
    }
}

#[test]
fn build_ctgov_search_params_preserves_none_values_without_defaults() {
    let filters = TrialSearchFilters {
        condition: Some("melanoma".into()),
        ..Default::default()
    };
    let normalized = validate_trial_search(&filters).expect("filters should validate");
    let context =
        prepare_ctgov_search_context(&filters, &normalized).expect("context should build");

    let params = build_ctgov_search_params(
        &filters,
        &context,
        raw_condition_query(&filters),
        raw_intervention_query(&filters),
        None,
        10,
        true,
    );

    assert_eq!(params.condition, Some("melanoma".into()));
    assert_eq!(params.intervention, None);
    assert_eq!(params.facility, None);
    assert_eq!(params.status, None);
    assert_eq!(params.agg_filters, None);
    assert_eq!(params.query_term, None);
    assert!(params.count_total);
    assert_eq!(params.page_token, None);
    assert_eq!(params.page_size, 10);
    assert_eq!(params.lat, None);
    assert_eq!(params.lon, None);
    assert_eq!(params.distance_miles, None);
}

#[test]
fn build_ctgov_search_params_keeps_search_and_count_call_shapes_aligned() {
    let filters = TrialSearchFilters {
        condition: Some("melanoma".into()),
        intervention: Some("HRS 4642".into()),
        facility: Some("Dana-Farber Cancer Institute".into()),
        status: Some("recruiting".into()),
        phase: Some("2".into()),
        sex: Some("all".into()),
        sponsor_type: Some("nih".into()),
        mutation: Some("BRAF V600E".into()),
        criteria: Some("prior anti-braf therapy".into()),
        lat: Some(42.3355),
        lon: Some(-71.1041),
        distance: Some(15),
        ..Default::default()
    };
    let normalized = validate_trial_search(&filters).expect("filters should validate");
    let context =
        prepare_ctgov_search_context(&filters, &normalized).expect("context should build");

    let search_page_params = build_ctgov_search_params(
        &filters,
        &context,
        raw_condition_query(&filters),
        raw_intervention_query(&filters),
        Some("page-1".into()),
        25,
        true,
    );
    let fast_count_params = build_ctgov_search_params(
        &filters,
        &context,
        raw_condition_query(&filters),
        raw_intervention_query(&filters),
        None,
        1,
        true,
    );
    let slow_count_params = build_ctgov_search_params(
        &filters,
        &context,
        raw_condition_query(&filters),
        raw_intervention_query(&filters),
        Some("page-2".into()),
        CTGOV_COUNT_PAGE_SIZE,
        true,
    );

    assert_eq!(search_page_params.condition, fast_count_params.condition);
    assert_eq!(search_page_params.condition, slow_count_params.condition);
    assert_eq!(
        search_page_params.intervention,
        fast_count_params.intervention
    );
    assert_eq!(
        search_page_params.intervention,
        slow_count_params.intervention
    );
    assert_eq!(search_page_params.facility, fast_count_params.facility);
    assert_eq!(search_page_params.facility, slow_count_params.facility);
    assert_eq!(search_page_params.status, fast_count_params.status);
    assert_eq!(search_page_params.status, slow_count_params.status);
    assert_eq!(
        search_page_params.agg_filters,
        fast_count_params.agg_filters
    );
    assert_eq!(
        search_page_params.agg_filters,
        slow_count_params.agg_filters
    );
    assert_eq!(search_page_params.query_term, fast_count_params.query_term);
    assert_eq!(search_page_params.query_term, slow_count_params.query_term);
    assert_eq!(
        search_page_params.count_total,
        fast_count_params.count_total
    );
    assert_eq!(
        search_page_params.count_total,
        slow_count_params.count_total
    );
    assert_eq!(search_page_params.lat, fast_count_params.lat);
    assert_eq!(search_page_params.lat, slow_count_params.lat);
    assert_eq!(search_page_params.lon, fast_count_params.lon);
    assert_eq!(search_page_params.lon, slow_count_params.lon);
    assert_eq!(
        search_page_params.distance_miles,
        fast_count_params.distance_miles
    );
    assert_eq!(
        search_page_params.distance_miles,
        slow_count_params.distance_miles
    );

    assert_eq!(search_page_params.page_token.as_deref(), Some("page-1"));
    assert_eq!(search_page_params.page_size, 25);
    assert_eq!(fast_count_params.page_token, None);
    assert_eq!(fast_count_params.page_size, 1);
    assert_eq!(slow_count_params.page_token.as_deref(), Some("page-2"));
    assert_eq!(slow_count_params.page_size, CTGOV_COUNT_PAGE_SIZE);
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
        ctgov_count_from_native_total(250, true),
        TrialCount::Approximate(250)
    );
}

#[test]
fn count_all_returns_exact_for_no_post_filters() {
    assert_eq!(
        ctgov_count_from_native_total(494, false),
        TrialCount::Exact(494)
    );
}

#[test]
fn count_all_returns_unknown_when_expensive_post_filter_hits_page_cap() {
    assert!(!ctgov_single_count_page_cap_reached(
        COUNT_TRAVERSAL_PAGE_CAP - 1
    ));
    assert!(ctgov_single_count_page_cap_reached(
        COUNT_TRAVERSAL_PAGE_CAP
    ));
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
    let rejection = || BioMcpError::CtGovInterventionQueryRejected {
        reason: "Error parsing query in Intervention / treatment: invalid expression".into(),
    };

    assert!(
        handle_ctgov_worker_outcome(1, &workers[1], Err(rejection()))
            .expect("expanded rejection should be tolerated")
            .is_none()
    );
    assert!(matches!(
        handle_ctgov_worker_outcome(0, &workers[0], Err(rejection())),
        Err(BioMcpError::CtGovInterventionQueryRejected { .. })
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
    let context =
        prepare_ctgov_search_context(&filters, &normalized).expect("context should build");
    let params = build_ctgov_search_params(
        &filters,
        &context,
        None,
        workers[0].intervention_query.as_deref(),
        None,
        10,
        true,
    );

    assert_eq!(workers.len(), 1);
    assert_eq!(workers[0].intervention_source, "requested");
    assert_eq!(params.intervention.as_deref(), Some("\"HRS 4642\""));
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
fn alias_union_count_returns_exact_unique_total_when_exhausted() {
    let mut unique_nct_ids = std::collections::HashSet::new();

    add_unique_ctgov_nct_ids(
        &mut unique_nct_ids,
        vec![
            serde_json::from_value(ctgov_search_study_fixture(
                "NCT00000001",
                "18 Years",
                "75 Years",
            ))
            .expect("study"),
            serde_json::from_value(ctgov_search_study_fixture(
                "NCT00000002",
                "18 Years",
                "75 Years",
            ))
            .expect("study"),
        ],
    );
    add_unique_ctgov_nct_ids(
        &mut unique_nct_ids,
        vec![
            serde_json::from_value(ctgov_search_study_fixture(
                "NCT00000001",
                "18 Years",
                "75 Years",
            ))
            .expect("study"),
            serde_json::from_value(ctgov_search_study_fixture(
                "NCT00000003",
                "18 Years",
                "75 Years",
            ))
            .expect("study"),
        ],
    );

    assert_eq!(
        TrialCount::Exact(unique_nct_ids.len()),
        TrialCount::Exact(3)
    );
}

#[test]
fn alias_union_count_returns_unknown_when_page_cap_is_hit() {
    assert!(!ctgov_count_page_cap_would_be_exceeded(48, 2));
    assert!(ctgov_count_page_cap_would_be_exceeded(50, 2));
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
    assert_eq!(completed_ctgov_union_count(true, 2), TrialCount::Unknown);
}
