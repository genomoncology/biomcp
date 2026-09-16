use super::*;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn an_expired_entry_refetches() {
    let mut fx = FixtureEnv::new(
        "citation-sidecar-ttl",
        vec![page(0, None, edge(vec!["Provider context"]))],
        Some(JATS_LINKED),
    )
    .await;
    fx.set("BIOMCP_TEST_CITATION_CACHE_TTL_MS", "0");

    with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("first call");
    assert_eq!(sidecar_records(fx.cache.path()).len(), 1);

    fx.cold_http_cache();
    fx.clear_log();

    with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("second call");
    let logged = fx.logged();
    assert!(logged.contains("s2:graph"), "{logged}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_bypassed_cache_reads_and_writes_nothing() {
    let fx = FixtureEnv::new(
        "citation-sidecar-bypass",
        vec![page(0, None, edge(vec!["Provider context"]))],
        Some(JATS_LINKED),
    )
    .await;

    // `with_no_cache` is the CLI `--no-cache` path and the same predicate the
    // environment mode resolves to; the task-local is the deterministic knob.
    let first = crate::sources::with_no_cache(
        true,
        with_test_client(
            fx.client(),
            citation_evidence(CITING_PMID, CITED_PMID, false),
        ),
    )
    .await
    .expect("bypassed call");
    assert_eq!(first.status, CitationEvidenceStatus::ContextFromProvider);
    assert!(sidecar_records(fx.cache.path()).is_empty());

    fx.cold_http_cache();
    fx.clear_log();

    crate::sources::with_no_cache(
        true,
        with_test_client(
            fx.client(),
            citation_evidence(CITING_PMID, CITED_PMID, false),
        ),
    )
    .await
    .expect("second bypassed call");
    let logged = fx.logged();
    assert!(logged.contains("s2:graph"), "{logged}");
    assert!(sidecar_records(fx.cache.path()).is_empty());
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn infinite_mode_serves_an_expired_entry() {
    let mut fx = FixtureEnv::new(
        "citation-sidecar-infinite",
        vec![page(0, None, edge(vec!["Provider context"]))],
        Some(JATS_LINKED),
    )
    .await;
    fx.set("BIOMCP_TEST_CITATION_CACHE_TTL_MS", "0");

    let first = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("first call");
    assert_eq!(sidecar_records(fx.cache.path()).len(), 1);

    // The mode is set after the first call so the HTTP middleware's cached
    // mode resolution never observes it.
    fx.set("BIOMCP_CACHE_MODE", "infinite");
    fx.cold_http_cache();
    fx.clear_log();

    let second = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("second call");
    assert_eq!(json(&second), json(&first));
    let logged = fx.logged();
    assert_eq!(logged.matches("s2:seed").count(), 2, "{logged}");
    assert!(!logged.contains("s2:graph"), "{logged}");
}

async fn populated_provider_cache(
    label: &str,
) -> (
    FixtureEnv,
    ArticleCitationEvidenceResult,
    std::path::PathBuf,
) {
    let fx = FixtureEnv::new(
        label,
        vec![page(0, None, edge(vec!["Provider context"]))],
        Some(JATS_LINKED),
    )
    .await;
    let first = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("first call");
    assert_eq!(first.status, CitationEvidenceStatus::ContextFromProvider);
    let records = sidecar_records(fx.cache.path());
    assert_eq!(records.len(), 1);
    (fx, first, records[0].clone())
}

fn pad_record(path: &std::path::Path, length: usize) {
    let mut bytes = std::fs::read(path).expect("read cached record");
    assert!(bytes.len() < length);
    bytes.resize(length, b' ');
    std::fs::write(path, bytes).expect("pad cached record");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn an_exactly_eight_mib_record_remains_a_hit() {
    let (fx, first, record) = populated_provider_cache("citation-sidecar-eight-mib").await;
    pad_record(&record, 8 * 1024 * 1024);
    fx.cold_http_cache();
    fx.clear_log();

    let second = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("bounded record hit");
    assert_eq!(json(&second), json(&first));
    let logged = fx.logged();
    assert_eq!(logged.matches("s2:seed").count(), 2, "{logged}");
    assert!(!logged.contains("s2:graph"), "{logged}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn an_eight_mib_plus_one_record_is_a_miss() {
    let (fx, _, record) = populated_provider_cache("citation-sidecar-oversize").await;
    pad_record(&record, 8 * 1024 * 1024 + 1);
    fx.cold_http_cache();
    fx.clear_log();

    let result = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("oversize record miss");
    assert_eq!(result.status, CitationEvidenceStatus::ContextFromProvider);
    let logged = fx.logged();
    assert!(logged.contains("s2:graph"), "{logged}");
}

#[cfg(unix)]
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_symlinked_record_is_a_miss() {
    let (fx, _, record) = populated_provider_cache("citation-sidecar-symlink").await;
    let target = fx.cache.path().join("citation-sidecar-link-target");
    std::fs::copy(&record, &target).expect("copy cached record");
    std::fs::remove_file(&record).expect("remove cached record");
    std::os::unix::fs::symlink(&target, &record).expect("link cached record");
    fx.cold_http_cache();
    fx.clear_log();

    let result = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("linked record miss");
    assert_eq!(result.status, CitationEvidenceStatus::ContextFromProvider);
    let logged = fx.logged();
    assert!(logged.contains("s2:graph"), "{logged}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_nonregular_record_is_a_miss() {
    let (fx, _, record) = populated_provider_cache("citation-sidecar-nonregular").await;
    std::fs::remove_file(&record).expect("remove cached record");
    std::fs::create_dir(&record).expect("replace record with directory");
    fx.cold_http_cache();
    fx.clear_log();

    let result = with_test_client(
        fx.client(),
        citation_evidence(CITING_PMID, CITED_PMID, false),
    )
    .await
    .expect("nonregular record miss");
    assert_eq!(result.status, CitationEvidenceStatus::ContextFromProvider);
    let logged = fx.logged();
    assert!(logged.contains("s2:graph"), "{logged}");
}
