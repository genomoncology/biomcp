use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Semaphore;

#[derive(Clone)]
struct OperationCounts {
    active: Arc<AtomicUsize>,
    maximum: Arc<AtomicUsize>,
    started: Arc<AtomicUsize>,
    cancelled: Arc<AtomicUsize>,
}

impl OperationCounts {
    fn new() -> Self {
        Self {
            active: Arc::new(AtomicUsize::new(0)),
            maximum: Arc::new(AtomicUsize::new(0)),
            started: Arc::new(AtomicUsize::new(0)),
            cancelled: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn enter(&self) -> OperationGuard {
        self.started.fetch_add(1, Ordering::SeqCst);
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.maximum.fetch_max(active, Ordering::SeqCst);
        OperationGuard {
            counts: self.clone(),
            completed: false,
        }
    }
}

struct OperationGuard {
    counts: OperationCounts,
    completed: bool,
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        self.counts.active.fetch_sub(1, Ordering::SeqCst);
        if !self.completed {
            self.counts.cancelled.fetch_add(1, Ordering::SeqCst);
        }
    }
}

struct GatedHpo {
    permits: Arc<Semaphore>,
    counts: OperationCounts,
}

impl HpoResolutionSource for GatedHpo {
    async fn term(&self, id: &str) -> Result<HpoTerm, BioMcpError> {
        let mut guard = self.counts.enter();
        self.permits
            .acquire()
            .await
            .expect("test semaphore open")
            .forget();
        guard.completed = true;
        Ok(HpoTerm {
            id: id.into(),
            name: format!("label {id}"),
        })
    }

    async fn search_terms(&self, query: &str) -> Result<Vec<HpoResolvedTerm>, BioMcpError> {
        let mut guard = self.counts.enter();
        self.permits
            .acquire()
            .await
            .expect("test semaphore open")
            .forget();
        guard.completed = true;
        let index = query.trim_start_matches('p').parse::<usize>().unwrap_or(1);
        Ok(vec![HpoResolvedTerm {
            id: format!("HP:{index:07}"),
            label: format!("label {index}"),
        }])
    }
}

struct ImmediateHpo;

impl HpoResolutionSource for ImmediateHpo {
    async fn term(&self, id: &str) -> Result<HpoTerm, BioMcpError> {
        Ok(HpoTerm {
            id: id.into(),
            name: "label".into(),
        })
    }

    async fn search_terms(&self, _: &str) -> Result<Vec<HpoResolvedTerm>, BioMcpError> {
        unreachable!("deadline tests use direct IDs")
    }
}

struct PhaseMonarch {
    similarity_counts: OperationCounts,
    support_counts: OperationCounts,
    block_similarity: bool,
    block_support: bool,
}

#[derive(Clone, Copy)]
enum SupportFailure {
    Transport,
    Status,
    ContentType,
    BodyLimit,
    Decode,
}

struct FailingSupportMonarch(SupportFailure);

impl PhenotypeMonarchSource for FailingSupportMonarch {
    async fn similarity(
        &self,
        _: &[String],
    ) -> Result<crate::sources::monarch::MonarchPhenotypeSearchResponse, BioMcpError> {
        Ok(crate::sources::monarch::MonarchPhenotypeSearchResponse {
            matches: vec![MonarchPhenotypeMatch {
                disease_id: "MONDO:0000001".into(),
                disease_name: "candidate".into(),
                score: 1.0,
            }],
            raw_row_count: 1,
            provider_window_exhausted: false,
        })
    }

    async fn direct_support(
        &self,
        _: &[String],
        _: &[String],
    ) -> Result<MonarchDirectSupportLookup, BioMcpError> {
        match self.0 {
            SupportFailure::BodyLimit => Err(BioMcpError::BodyLimit {
                source_name: "Monarch Initiative".into(),
                max_bytes: 8 * 1024 * 1024,
            }),
            kind => Err(BioMcpError::Api {
                api: "monarch".into(),
                message: match kind {
                    SupportFailure::Transport => "transport",
                    SupportFailure::Status => "status",
                    SupportFailure::ContentType => "content-type",
                    SupportFailure::Decode => "decode",
                    SupportFailure::BodyLimit => unreachable!(),
                }
                .into(),
            }),
        }
    }
}

impl PhenotypeMonarchSource for PhaseMonarch {
    async fn similarity(
        &self,
        _: &[String],
    ) -> Result<crate::sources::monarch::MonarchPhenotypeSearchResponse, BioMcpError> {
        let mut guard = self.similarity_counts.enter();
        if self.block_similarity {
            std::future::pending::<()>().await;
        }
        guard.completed = true;
        Ok(crate::sources::monarch::MonarchPhenotypeSearchResponse {
            matches: vec![MonarchPhenotypeMatch {
                disease_id: "MONDO:0000001".into(),
                disease_name: "candidate".into(),
                score: 1.0,
            }],
            raw_row_count: 1,
            provider_window_exhausted: false,
        })
    }

    async fn direct_support(
        &self,
        _: &[String],
        _: &[String],
    ) -> Result<MonarchDirectSupportLookup, BioMcpError> {
        let mut guard = self.support_counts.enter();
        if self.block_support {
            std::future::pending::<()>().await;
        }
        guard.completed = true;
        let response = serde_json::from_value(serde_json::json!({"total":0,"items":[]}))
            .expect("complete empty support response");
        MonarchClient::map_direct_support(
            response,
            &["MONDO:0000001".into()],
            &["HP:0001250".into()],
        )
    }
}

#[test]
fn disease_search_request_records_normalized_filters_and_fetch_plan() {
    let filters = DiseaseSearchFilters {
        query: Some(" chronic myeloid leukemia ".into()),
        source: Some(" DOID ".into()),
        inheritance: Some(" autosomal dominant ".into()),
        phenotype: Some(" HP:0001250 ".into()),
        onset: Some(" childhood ".into()),
    };

    let request = DiseaseSearchRequest::new(&filters, 3, 2).expect("request");

    assert_eq!(request.query, "chronic myeloid leukemia");
    assert_eq!(request.source.as_deref(), Some("DOID"));
    assert_eq!(request.inheritance.as_deref(), Some("autosomal dominant"));
    assert_eq!(request.phenotype.as_deref(), Some("HP:0001250"));
    assert_eq!(request.onset.as_deref(), Some("childhood"));
    assert_eq!(request.limit, 3);
    assert_eq!(request.offset, 2);
    assert_eq!(request.fetch_size, 25);
    assert!(
        request
            .resolver_queries
            .iter()
            .any(|value| value == "chronic myeloid leukemia")
    );
    assert!(request.prefer_doid);
}

#[test]
fn ticket_400_request_command_disease_search_fields_drive_source_query_and_pagination() {
    let filters = DiseaseSearchFilters {
        query: Some(" chronic myeloid leukemia ".into()),
        source: Some(" doid ".into()),
        inheritance: Some(" autosomal dominant ".into()),
        phenotype: Some(" HP:0001250 ".into()),
        onset: Some(" childhood ".into()),
    };
    let request = DiseaseSearchRequest::new(&filters, 3, 2).expect("request");
    let client =
        crate::sources::mydisease::MyDiseaseClient::new_for_test("http://127.0.0.1/v1".into())
            .expect("mydisease client");
    let plan = client
        .query_request_plan(
            &request.resolver_queries[0],
            request.fetch_size,
            0,
            request.source.as_deref(),
            request.inheritance.as_deref(),
            request.phenotype.as_deref(),
            request.onset.as_deref(),
        )
        .expect("source query plan");

    assert_eq!(request.limit, 3);
    assert_eq!(request.offset, 2);
    assert_eq!(plan.path, "/query");
    assert!(plan.query_params.contains(&("size", "25".to_string())));
    assert!(plan.query_params.contains(&("from", "0".to_string())));
    assert!(plan.query_params.iter().any(|(key, value)| {
        *key == "q"
            && value.contains("chronic myeloid leukemia")
            && value.contains("disease_ontology.doid:*")
            && value.contains("hpo.inheritance.hpo_name:*autosomal dominant*")
            && value.contains("hpo.phenotype_related_to_disease.hpo_id:*HP\\:0001250*")
            && value.contains("hpo.clinical_course.hpo_name:*childhood*")
    }));
}

#[test]
fn disease_filter_normalizers_accept_supported_values() {
    let inheritance_names = [
        "autosomal dominant",
        "autosomal recessive",
        "x-linked",
        "x-linked dominant",
        "x-linked recessive",
        "y-linked",
        "mitochondrial",
        "multifactorial",
        "oligogenic",
        "polygenic",
        "sporadic",
        "somatic mosaicism",
        "dominant",
        "recessive",
    ];
    for value in inheritance_names {
        assert_eq!(normalize_inheritance(value).unwrap(), value);
    }
    for value in [
        "HP:0000006",
        "HP:0000007",
        "HP:0001417",
        "HP:0001423",
        "HP:0001419",
        "HP:0001450",
        "HP:0001427",
        "HP:0001426",
        "HP:0010983",
        "HP:0010982",
        "HP:0003745",
        "HP:0001442",
    ] {
        assert_eq!(normalize_inheritance(value).unwrap(), value);
    }

    for value in [
        "antenatal",
        "embryonal",
        "fetal",
        "congenital",
        "neonatal",
        "infantile",
        "childhood",
        "juvenile",
        "adolescent",
        "young adult",
        "adult",
        "middle age",
        "late onset",
    ] {
        assert_eq!(normalize_onset(value).unwrap(), value);
    }
    assert_eq!(normalize_onset("infancy").unwrap(), "infantile");
}

#[test]
fn disease_filter_normalizers_handle_case_whitespace_and_unknown_values() {
    assert_eq!(
        normalize_inheritance(" Autosomal Dominant ").unwrap(),
        "autosomal dominant"
    );
    assert_eq!(normalize_inheritance(" hp:0000006 ").unwrap(), "HP:0000006");
    assert_eq!(normalize_onset(" Young Adult ").unwrap(), "young adult");
    for value in ["", "unknown inheritance"] {
        assert!(normalize_inheritance(value).is_err());
    }
    for value in ["", "unknown onset"] {
        assert!(normalize_onset(value).is_err());
    }
}

#[test]
fn disease_search_request_preserves_limit_and_query_validation() {
    let filters = DiseaseSearchFilters::default();
    let err = DiseaseSearchRequest::new(&filters, 0, 0).expect_err("limit should fail");
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));

    let err = DiseaseSearchRequest::new(&filters, 1, 0).expect_err("query should fail");
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
}

#[test]
fn parse_hpo_query_terms_requires_valid_ids() {
    let parsed = parse_hpo_query_terms("HP:0001250 HP:0001263").expect("valid terms");
    assert_eq!(parsed, vec!["HP:0001250", "HP:0001263"]);
    let comma_separated = parse_hpo_query_terms("hp:0001250, HP:0001263").expect("comma terms");
    assert_eq!(comma_separated, vec!["HP:0001250", "HP:0001263"]);
    assert!(parse_hpo_query_terms("NOT_AN_HPO").is_err());
}

#[test]
fn phenotype_terms_and_result_window_are_bounded() {
    let eleven = (1..=11)
        .map(|index| format!("HP:{index:07}"))
        .collect::<Vec<_>>()
        .join(" ");
    let err = parse_hpo_query_terms(&eleven).expect_err("eleven HPO terms should fail");
    assert!(err.to_string().contains("at most 10 unique HPO terms"));

    assert_eq!(validate_phenotype_search_window(10, 40).unwrap(), 50);
    for (limit, offset) in [(1, 50), (11, 40), (2, usize::MAX)] {
        let err = validate_phenotype_search_window(limit, offset)
            .expect_err("window beyond Monarch's first 50 rows should fail");
        assert!(err.to_string().contains("--offset + --limit must be <= 50"));
    }
}

#[test]
fn phenotype_continuation_shrinks_to_the_remaining_provider_window() {
    let pagination = PhenotypePagination {
        offset: 40,
        limit: 9,
        returned: 9,
        total: None,
        has_more: true,
        next_page_token: None,
        provider_window_limit: 50,
        provider_raw_row_count: 50,
        provider_window_exhausted: true,
    };

    assert_eq!(pagination.next_window(), Some((1, 49)));
}

#[test]
fn phenotype_pages_slice_one_provider_ordered_deduplicated_window() {
    let provider = crate::sources::monarch::MonarchPhenotypeSearchResponse {
        matches: vec![
            ("MONDO:0000003", "third", 10.0),
            ("MONDO:0000001", "first tie", 9.0),
            ("MONDO:0000002", "second tie", 9.0),
            ("MONDO:0000004", "fourth", 8.0),
            ("MONDO:0000005", "fifth", 7.0),
        ]
        .into_iter()
        .map(|(disease_id, disease_name, score)| MonarchPhenotypeMatch {
            disease_id: disease_id.into(),
            disease_name: disease_name.into(),
            score,
        })
        .collect(),
        raw_row_count: 50,
        provider_window_exhausted: true,
    };

    let first = paginate_phenotype_matches(provider.clone(), 2, 0).unwrap();
    let second = paginate_phenotype_matches(provider.clone(), 3, 2).unwrap();
    let combined = paginate_phenotype_matches(provider, 5, 0).unwrap();
    let paged = first
        .results
        .iter()
        .chain(&second.results)
        .map(|row| row.disease_id.as_str())
        .collect::<Vec<_>>();
    let all = combined
        .results
        .iter()
        .map(|row| row.disease_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(paged, all);
    assert_eq!(
        all,
        [
            "MONDO:0000003",
            "MONDO:0000001",
            "MONDO:0000002",
            "MONDO:0000004",
            "MONDO:0000005"
        ]
    );
    assert!(first.pagination.has_more);
    assert!(first.pagination.provider_window_exhausted);
    assert_eq!(first.pagination.provider_window_limit, 50);
    assert_eq!(first.pagination.provider_raw_row_count, 50);
    assert!(!combined.pagination.has_more);
    assert!(combined.pagination.provider_window_exhausted);
}

#[test]
fn split_phenotype_queries_preserves_single_phrase_and_splits_commas() {
    assert_eq!(
        split_phenotype_queries("developmental delay"),
        vec!["developmental delay"]
    );
    assert_eq!(
        split_phenotype_queries("seizure, developmental delay,  hypotonia "),
        vec!["seizure", "developmental delay", "hypotonia"]
    );
}

#[tokio::test]
async fn resolve_phenotype_query_terms_empty_input_mentions_hpo_ids_and_symptom_phrases() {
    let err = resolve_phenotype_query_terms(
        "   ",
        tokio::time::Instant::now() + std::time::Duration::from_secs(1),
    )
    .await
    .expect_err("empty phenotype query should fail");

    match err {
        BioMcpError::InvalidArgument(message) => {
            assert!(message.contains("Use HPO IDs or symptom phrases"));
            assert!(message.contains("HP:0001250 HP:0001263"));
            assert!(message.contains("seizure, developmental delay"));
        }
        other => panic!("expected InvalidArgument, got: {other}"),
    }
}

#[tokio::test(start_paused = true)]
async fn phenotype_resolution_runs_all_ten_operations_with_at_most_four_in_flight() {
    let source = GatedHpo {
        permits: Arc::new(Semaphore::new(0)),
        counts: OperationCounts::new(),
    };
    let query = (1..=10)
        .map(|index| format!("p{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let future = resolve_phenotype_query_terms_with_source(
        &query,
        Instant::now() + PHENOTYPE_COMMAND_TIMEOUT,
        &source,
    );
    tokio::pin!(future);
    for _ in 0..20 {
        tokio::select! {
            biased;
            result = &mut future => panic!("resolution finished before permits: {result:?}"),
            _ = tokio::task::yield_now() => {}
        }
        if source.counts.started.load(Ordering::SeqCst) == MAX_PHENOTYPE_IN_FLIGHT {
            break;
        }
    }
    assert_eq!(source.counts.started.load(Ordering::SeqCst), 4);
    assert_eq!(source.counts.maximum.load(Ordering::SeqCst), 4);
    source.permits.add_permits(10);
    let terms = future.await.expect("all ten phrases resolve");
    assert_eq!(terms.len(), 10);
    assert_eq!(source.counts.started.load(Ordering::SeqCst), 10);
    assert_eq!(source.counts.maximum.load(Ordering::SeqCst), 4);
    assert_eq!(source.counts.cancelled.load(Ordering::SeqCst), 0);
    assert_eq!(10 + 1 + 1, MAX_PHENOTYPE_LOGICAL_OPERATIONS);
    assert_eq!(
        MAX_PHENOTYPE_LOGICAL_OPERATIONS * 4,
        MAX_PHENOTYPE_PHYSICAL_ATTEMPTS
    );
}

#[tokio::test(start_paused = true)]
async fn phenotype_shared_resolution_deadline_cancels_every_in_flight_operation_at_eight_seconds() {
    let source = GatedHpo {
        permits: Arc::new(Semaphore::new(0)),
        counts: OperationCounts::new(),
    };
    let started_at = Instant::now();
    let result = resolve_phenotype_query_terms_with_source(
        "p1,p2,p3,p4,p5",
        started_at + PHENOTYPE_COMMAND_TIMEOUT,
        &source,
    )
    .await;
    assert!(result.is_err());
    assert_eq!(Instant::now() - started_at, PHENOTYPE_RESOLUTION_TIMEOUT);
    assert_eq!(source.counts.started.load(Ordering::SeqCst), 5);
    assert_eq!(source.counts.active.load(Ordering::SeqCst), 0);
    assert_eq!(source.counts.cancelled.load(Ordering::SeqCst), 5);
}

#[tokio::test(start_paused = true)]
async fn phenotype_whole_command_and_support_deadlines_have_distinct_observable_outcomes() {
    let similarity_blocked = PhaseMonarch {
        similarity_counts: OperationCounts::new(),
        support_counts: OperationCounts::new(),
        block_similarity: true,
        block_support: false,
    };
    let started_at = Instant::now();
    let failure = search_phenotype_page_with_sources(
        "HP:0001250",
        1,
        0,
        started_at + PHENOTYPE_COMMAND_TIMEOUT,
        &ImmediateHpo,
        &similarity_blocked,
    )
    .await
    .expect_err("similarity must fail at the whole-command deadline");
    match failure {
        BioMcpError::SourceUnavailable { reason, .. } => {
            assert!(reason.contains("30-second provider deadline"));
        }
        other => panic!("expected typed Monarch deadline, got {other:?}"),
    }
    assert_eq!(Instant::now() - started_at, PHENOTYPE_COMMAND_TIMEOUT);
    assert_eq!(
        similarity_blocked
            .similarity_counts
            .cancelled
            .load(Ordering::SeqCst),
        1
    );

    let support_blocked = PhaseMonarch {
        similarity_counts: OperationCounts::new(),
        support_counts: OperationCounts::new(),
        block_similarity: false,
        block_support: true,
    };
    let started_at = Instant::now();
    let page = search_phenotype_page_with_sources(
        "HP:0001250",
        1,
        0,
        started_at + PHENOTYPE_COMMAND_TIMEOUT,
        &ImmediateHpo,
        &support_blocked,
    )
    .await
    .expect("support deadline degrades a valid similarity page");
    assert_eq!(Instant::now() - started_at, PHENOTYPE_SUPPORT_TIMEOUT);
    assert_eq!(
        support_blocked
            .support_counts
            .cancelled
            .load(Ordering::SeqCst),
        1
    );
    assert_eq!(
        page.results[0].direct_support[0].status,
        PhenotypeDirectSupportStatus::Unavailable
    );
}

#[tokio::test]
async fn phenotype_support_transport_status_content_body_and_decode_failures_all_degrade() {
    for failure in [
        SupportFailure::Transport,
        SupportFailure::Status,
        SupportFailure::ContentType,
        SupportFailure::BodyLimit,
        SupportFailure::Decode,
    ] {
        let page = search_phenotype_page_with_sources(
            "HP:0001250",
            1,
            0,
            Instant::now() + PHENOTYPE_COMMAND_TIMEOUT,
            &ImmediateHpo,
            &FailingSupportMonarch(failure),
        )
        .await
        .expect("association failures must retain the similarity page");
        assert_eq!(page.results[0].score, 1.0);
        assert_eq!(
            page.results[0].direct_support[0].status,
            PhenotypeDirectSupportStatus::Unavailable
        );
    }
}

#[test]
fn unavailable_support_is_applied_pairwise_without_changing_similarity_order() {
    let mut results = vec![PhenotypeSearchResult {
        disease_id: "MONDO:0000001".into(),
        disease_name: "candidate".into(),
        score: 9.0,
        direct_support: Vec::new(),
    }];
    apply_direct_support(
        &mut results,
        &["HP:0000256".into(), "HP:0001250".into()],
        None,
    );
    assert_eq!(results[0].score, 9.0);
    assert_eq!(
        results[0]
            .direct_support
            .iter()
            .map(|row| row.status)
            .collect::<Vec<_>>(),
        vec![
            PhenotypeDirectSupportStatus::Unavailable,
            PhenotypeDirectSupportStatus::Unavailable,
        ]
    );
}

#[test]
fn phenotype_phrase_rows_flatten_in_phrase_and_provider_order_without_truncating_the_eleventh() {
    let row = |id: usize| HpoResolvedTerm {
        id: format!("HP:{id:07}"),
        label: format!("term {id}"),
    };
    let over = flatten_resolved_phrase_rows(vec![
        ("first phrase".into(), (1..=6).map(row).collect()),
        ("second phrase".into(), (7..=11).map(row).collect()),
    ])
    .expect_err("eleven aggregate unique IDs must be rejected, not truncated");
    assert!(
        over.to_string()
            .contains("resolved more than 10 unique HPO terms")
    );

    let ten = flatten_resolved_phrase_rows(vec![
        ("first phrase".into(), (1..=5).map(row).collect()),
        (
            "second phrase".into(),
            std::iter::once(row(5)).chain((6..=10).map(row)).collect(),
        ),
    ])
    .expect("ten unique IDs with a cross-phrase duplicate should succeed");
    assert_eq!(ten.len(), 10);
    assert_eq!(ten[4].raw, "first phrase");
    assert_eq!(
        ten.iter().map(|term| term.id.as_str()).collect::<Vec<_>>(),
        vec![
            "HP:0000001",
            "HP:0000002",
            "HP:0000003",
            "HP:0000004",
            "HP:0000005",
            "HP:0000006",
            "HP:0000007",
            "HP:0000008",
            "HP:0000009",
            "HP:0000010",
        ]
    );
}
