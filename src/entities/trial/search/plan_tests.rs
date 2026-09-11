use super::*;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use crate::sources::nci_cts::NciCtsClient;

#[test]
fn public_search_still_rejects_empty_filters() {
    let error = match validate_trial_search(&TrialSearchFilters::default()) {
        Ok(_) => panic!("empty search unexpectedly passed"),
        Err(error) => error,
    };
    assert!(matches!(error, BioMcpError::InvalidArgument(_)));
}

#[test]
fn clients_execute_exact_biodata_pairs_and_only_nci_adds_a_credential() {
    let product = TrialSearchFilters {
        condition: Some("melanoma".into()),
        phase: Some("III".into()),
        ..Default::default()
    };
    let filters = biodata_filters(&product, None).unwrap();
    let ctgov = biodata::ClinicalTrialsGovApiV2SearchPlan::new(&filters, 5, None, true).unwrap();
    let request = ClinicalTrialsClient::biodata_search_plan(&ctgov);
    assert_eq!(
        request.query,
        ctgov
            .query_pairs()
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect::<Vec<_>>()
    );
    assert!(request.headers.is_empty());

    let nci = biodata::NciCtsV2SearchPlan::new(
        &filters,
        Some(biodata::NciCtsV2DiseaseSelection::Keyword(
            "melanoma".into(),
        )),
        5,
        0,
    )
    .unwrap();
    let request = NciCtsClient::biodata_search_plan("test-key", &nci);
    assert_eq!(
        request.headers,
        vec![("X-API-KEY".into(), "test-key".into())]
    );
    assert_eq!(request.query_value("include"), Some("nct_id"));
    assert!(
        request
            .query
            .iter()
            .any(|pair| pair == &("include".into(), "eligibility".into()))
    );
}

#[test]
fn error_projection_never_exposes_a_filter_value() {
    const SENTINEL: &str = "PRIVATE-FILTER-0118";
    let error = match validate_trial_search(&TrialSearchFilters {
        status: Some(SENTINEL.into()),
        ..Default::default()
    }) {
        Ok(_) => panic!("invalid search unexpectedly passed"),
        Err(error) => error,
    };
    assert!(!format!("{error:?} {error}").contains(SENTINEL));
}

#[test]
fn ambiguous_active_status_returns_only_safe_correction() {
    let error = match validate_trial_search(&TrialSearchFilters {
        status: Some("active".into()),
        ..Default::default()
    }) {
        Ok(_) => panic!("ambiguous status unexpectedly passed"),
        Err(error) => error,
    };
    let public = format!("{error:?} {error}");
    assert!(public.contains("recruiting"));
    assert!(public.contains("active_not_recruiting"));
    assert!(!public.contains("status: active"));
}

#[test]
fn post_filter_context_uses_only_the_validated_biodata_projection() {
    let product = TrialSearchFilters {
        facility: Some("  Cancer Center  ".into()),
        mutation: Some("  BRAF  ".into()),
        criteria: Some("  ECOG 0  ".into()),
        prior_therapies: Some("  platinum  ".into()),
        progression_on: Some("  osimertinib  ".into()),
        age: Some(42.0),
        lat: Some(40.0),
        lon: Some(-80.0),
        distance: Some(25),
        ..Default::default()
    };
    let normalized = validate_trial_search(&product).expect("validated search");
    let context = prepare_ctgov_search_context(&normalized).expect("post-filter context");
    assert_eq!(
        context.eligibility_keywords,
        ["BRAF", "ECOG 0", "platinum", "osimertinib"].map(str::to_owned)
    );
    assert_eq!(
        context.facility_geo_verification,
        Some(("Cancer Center".into(), 40.0, -80.0, 25))
    );
    assert_eq!(context.age_verification, Some(42.0));
}

#[test]
fn product_mapping_covers_every_ctgov_filter_family() {
    let product = TrialSearchFilters {
        condition: Some("melanoma".into()),
        intervention: Some("drug".into()),
        facility: Some("center".into()),
        status: Some("recruiting".into()),
        phase: Some("III".into()),
        study_type: Some("interventional".into()),
        age: Some(42.0),
        sex: Some("female".into()),
        sponsor: Some("institute".into()),
        sponsor_type: Some("federal".into()),
        date_from: Some("2024".into()),
        date_to: Some("2024-12".into()),
        mutation: Some("BRAF".into()),
        criteria: Some("ECOG 0".into()),
        biomarker: Some("MSI-H".into()),
        prior_therapies: Some("platinum".into()),
        progression_on: Some("osimertinib".into()),
        line_of_therapy: Some("2L".into()),
        results_available: true,
        lat: Some(40.0),
        lon: Some(-80.0),
        distance: Some(25),
        ..Default::default()
    };
    let normalized = validate_trial_search(&product).expect("all CTGov filters");
    let plan = biodata::ClinicalTrialsGovApiV2SearchPlan::new(
        &normalized.biodata,
        5,
        Some("cursor"),
        true,
    )
    .expect("complete CTGov plan");
    let pairs = plan.query_pairs();
    for name in [
        "query.cond",
        "query.intr",
        "query.locn",
        "filter.overallStatus",
        "aggFilters",
        "query.term",
        "filter.geo",
        "countTotal",
        "pageToken",
        "pageSize",
        "fields",
    ] {
        assert!(
            pairs.iter().any(|(actual, _)| *actual == name),
            "missing {name}"
        );
    }
    let term = pairs
        .iter()
        .find(|(name, _)| *name == "query.term")
        .expect("advanced filter query")
        .1;
    for value in [
        "Phase",
        "StudyType",
        "LeadSponsorName",
        "LastUpdatePostDate",
        "ResultsFirstPostDate",
        "BRAF",
        "ECOG 0",
        "MSI\\-H",
        "platinum",
        "osimertinib",
        "second line",
    ] {
        assert!(term.contains(value), "missing mapped value {value}");
    }
    let context = prepare_ctgov_search_context(&normalized).expect("post-filter projection");
    assert_eq!(context.age_verification, Some(42.0));
}

#[test]
fn nci_refuses_every_unsupported_product_selector_before_transport() {
    let cases = [
        TrialSearchFilters {
            study_type: Some("interventional".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            age: Some(42.0),
            ..nci_filters()
        },
        TrialSearchFilters {
            sex: Some("female".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            sponsor: Some("NCI".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            sponsor_type: Some("fed".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            date_from: Some("2024".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            prior_therapies: Some("drug".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            progression_on: Some("drug".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            line_of_therapy: Some("1L".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            results_available: true,
            ..nci_filters()
        },
        TrialSearchFilters {
            phase: Some("early1".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            status: Some("recruiting,completed".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            mutation: Some("BRAF".into()),
            criteria: Some("ECOG".into()),
            ..nci_filters()
        },
        TrialSearchFilters {
            intervention: Some("drug".into()),
            no_alias_expand: true,
            ..nci_filters()
        },
    ];
    let selectors = [
        "study_type",
        "age",
        "sex",
        "sponsor",
        "sponsor_type",
        "date",
        "prior_therapies",
        "progression_on",
        "line_of_therapy",
        "results_available",
        "phase",
        "status",
        "molecular",
        "--no-alias-expand",
    ];
    for (filters, selector) in cases.into_iter().zip(selectors) {
        let error = match validate_trial_search(&filters) {
            Ok(_) => panic!("unsupported selector unexpectedly passed"),
            Err(error) => error,
        };
        assert!(format!("{error}").contains(selector), "missing {selector}");
    }
}

fn nci_filters() -> TrialSearchFilters {
    TrialSearchFilters {
        source: TrialSource::NciCts,
        condition: Some("melanoma".into()),
        ..Default::default()
    }
}
