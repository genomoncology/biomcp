use biodata::{Doi, Pmcid, Pmid, PublicationIdentifier};

use crate::sources::HttpMethod;
use crate::sources::europepmc::{EuropePmcClient, EuropePmcDetail, parse_publication_detail};

fn pmid(value: &str) -> PublicationIdentifier {
    PublicationIdentifier::Pmid(Pmid::new(value).unwrap())
}

fn pmcid(value: &str) -> PublicationIdentifier {
    PublicationIdentifier::Pmcid(Pmcid::new(value).unwrap())
}

fn doi(value: &str) -> PublicationIdentifier {
    PublicationIdentifier::Doi(Doi::new(value).unwrap())
}

fn legacy(_query: &str, row: &str, request: &str, hit_count: u64) -> Vec<u8> {
    format!(
        r#"{{"version":"wide","hitCount":{hit_count},{request}"resultList":{{"result":[{row}]}}}}"#
    )
    .into_bytes()
}

#[test]
fn detail_plan_adapts_exact_biodata_pairs_for_each_identity() {
    for (identity, query, result_type) in [
        (pmid("20516115"), "EXT_ID:20516115 AND SRC:MED", None),
        (pmcid("PMC3040717"), "PMCID:PMC3040717", Some("lite")),
        (
            doi("10.1158/0008-5472.CAN-09-4563"),
            "DOI:10.1158/0008-5472.can-09-4563",
            Some("lite"),
        ),
    ] {
        let plan = EuropePmcClient::publication_detail_plan(identity).unwrap();
        assert_eq!(plan.method, HttpMethod::Get);
        assert_eq!(plan.path, "search");
        assert_eq!(plan.query_value("query"), Some(query));
        assert_eq!(plan.query_value("format"), Some("json"));
        assert_eq!(plan.query_value("page"), Some("1"));
        assert_eq!(plan.query_value("pageSize"), Some("1"));
        assert_eq!(plan.query_value("resultType"), result_type);
    }
}

#[test]
fn admitted_detail_keeps_original_fixture_bytes_and_selected_source() {
    let bytes = include_bytes!("../../../../testdata/sources/europepmc/search_pmid_20516115.json");
    let detail = parse_publication_detail(pmid("20516115"), bytes)
        .unwrap()
        .expect("admitted detail");
    let EuropePmcDetail::Adopted(response) = detail else {
        panic!("expected adopted detail")
    };
    assert_eq!(response.response_bytes(), bytes);
    assert_eq!(response.selected_index(), 0);
    assert_eq!(
        response.provider_record().unwrap().pmcid(),
        Some("PMC3040717")
    );
}

#[test]
fn planned_detail_never_downgrades_strict_failures() {
    let valid_row = r#"{"id":"7","source":"MED","pmid":"7","title":"one"}"#;
    let exact_request = r#""request":{"queryString":"EXT_ID:7 AND SRC:MED","internalQuery":"x","resultType":"LITE","cursorMark":"*","pageSize":1,"sort":"","synonym":false},"#;
    let cases = [
        legacy("ignored", valid_row, exact_request, 2),
        legacy(
            "ignored",
            r#"{"id":"8","source":"MED","pmid":"8","unknown":"x"}"#,
            exact_request,
            1,
        ),
        legacy(
            "ignored",
            r#"{"id":"7","source":"MED","pmid":"8","unknown":"x"}"#,
            exact_request,
            1,
        ),
        legacy(
            "ignored",
            r#"{"id":"7","source":"MED","pmid":"7","title":"one","title":"two"}"#,
            exact_request,
            1,
        ),
        b"{".to_vec(),
    ];
    for bytes in cases {
        assert!(parse_publication_detail(pmid("7"), &bytes).is_err());
    }
    let oversized = legacy(
        "ignored",
        &format!(
            r#"{{"id":"7","source":"MED","pmid":"7","title":"{}"}}"#,
            "x".repeat(262_145)
        ),
        exact_request,
        1,
    );
    assert!(parse_publication_detail(pmid("7"), &oversized).is_err());
}

#[test]
fn unsupported_shape_legacy_requires_complete_single_bound_row_and_exact_optional_echo() {
    let matching_echo = r#""request":{"queryString":"PMCID:PMC7","resultType":"LITE","cursorMark":"*","pageSize":1},"#;
    for request in ["", r#""request":null,"#, matching_echo] {
        let bytes = legacy(
            "PMCID:PMC7",
            r#"{"id":"7","source":"MED","pmid":"7","pmcid":"PMC7","abstractText":"legacy","license":"CC BY"}"#,
            request,
            1,
        );
        let detail = parse_publication_detail(pmcid("PMC7"), &bytes)
            .unwrap()
            .expect("legacy detail");
        assert!(matches!(detail, EuropePmcDetail::Legacy { .. }));
    }

    let invalid = [
        legacy("x", r#"{"pmcid":"PMC7","abstractText":"x"}"#, "", 2),
        legacy("x", r#"{"pmcid":"PMC8","abstractText":"x"}"#, "", 1),
        legacy("x", r#"{"pmid":"7","abstractText":"x"}"#, "", 1),
        legacy(
            "x",
            r#"{"id":"7","source":"MED","pmid":"8","pmcid":"PMC7","abstractText":"x"}"#,
            "",
            1,
        ),
        legacy(
            "x",
            r#"{"pmcid":"PMC7","abstractText":"x"},{"pmcid":"PMC7","abstractText":"y"}"#,
            "",
            2,
        ),
        legacy(
            "x",
            r#"{"pmcid":"PMC7","abstractText":"x"}"#,
            r#""request":{"queryString":"PMCID:PMC8","resultType":"LITE","cursorMark":"*","pageSize":1},"#,
            1,
        ),
        legacy(
            "x",
            r#"{"pmcid":"PMC7","abstractText":"x"}"#,
            r#""request":{"queryString":"PMCID:PMC7","cursorMark":"*","pageSize":1},"#,
            1,
        ),
    ];
    for bytes in invalid {
        assert!(parse_publication_detail(pmcid("PMC7"), &bytes).is_err());
    }
}

#[test]
fn unsupported_profile_doi_retains_legacy_json_behavior_but_not_unguarded_identity() {
    let requested = doi("10.1002/(SICI)1097-0258");
    let valid = legacy(
        "x",
        r#"{"doi":"10.1002/(sici)1097-0258","abstractText":"legacy"}"#,
        "",
        1,
    );
    assert!(matches!(
        parse_publication_detail(requested.clone(), &valid).unwrap(),
        Some(EuropePmcDetail::Legacy { .. })
    ));
    for bytes in [
        legacy(
            "x",
            r#"{"doi":"10.1/wrong","doi":"10.1002/(sici)1097-0258","abstractText":"legacy"}"#,
            "",
            1,
        ),
        legacy("x", r#"{"doi":"10.1/wrong","abstractText":"x"}"#, "", 1),
        legacy(
            "x",
            r#"{"doi":"10.1002/(sici)1097-0258","abstractText":"x"}"#,
            "",
            2,
        ),
        b"{".to_vec(),
    ] {
        assert!(parse_publication_detail(requested.clone(), &bytes).is_err());
    }
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn detail_client_executes_one_exact_plan_and_keeps_acquired_bytes() {
    use crate::entities::article::test_support::{
        TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    let calls = std::sync::Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let bytes = include_bytes!("../../../../testdata/sources/europepmc/search_pmid_20516115.json");
    let fixture = TestHttpFixture::spawn(move |request| {
        seen.fetch_add(1, Ordering::SeqCst);
        assert!(request.starts_with("GET /search?"));
        assert!(request.contains("query=EXT_ID%3A20516115+AND+SRC%3AMED"));
        assert!(request.contains("format=json"));
        assert!(request.contains("page=1"));
        assert!(request.contains("pageSize=1"));
        assert!(!request.contains("resultType="));
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", bytes))
    })
    .await;
    let cache = crate::test_support::TempDirGuard::new("europepmc-detail-source");
    let mut env = TestEnv::new();
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    let detail = EuropePmcClient::new()
        .unwrap()
        .publication_detail(pmid("20516115"))
        .await
        .unwrap()
        .expect("detail");
    let EuropePmcDetail::Adopted(response) = detail else {
        panic!("expected adopted detail")
    };
    assert_eq!(response.response_bytes(), bytes);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[serial_test::serial(source_env)]
#[tokio::test]
async fn unsupported_profile_doi_runtime_uses_one_legacy_request_and_weak_limits() {
    use crate::entities::article::test_support::{
        TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    let calls = std::sync::Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let body = legacy(
        "unused",
        &format!(
            r#"{{"doi":"10.1002/(sici)1097-0258","abstractText":"{}"}}"#,
            "x".repeat(262_145)
        ),
        r#""request":{"queryString":"DOI:10.1002/(sici)1097-0258","resultType":"LITE","cursorMark":"*","pageSize":1},"#,
        1,
    );
    let fixture = TestHttpFixture::spawn(move |request| {
        seen.fetch_add(1, Ordering::SeqCst);
        assert!(request.starts_with("GET /search?"));
        assert!(request.contains("query=DOI%3A10.1002%2F%28sici%291097-0258"));
        assert!(request.contains("pageSize=1"));
        assert!(!request.contains("resultType="));
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &body))
    })
    .await;
    let cache = crate::test_support::TempDirGuard::new("europepmc-unsupported-doi-runtime");
    let mut env = TestEnv::new();
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    let detail = EuropePmcClient::new()
        .unwrap()
        .publication_detail(doi("10.1002/(SICI)1097-0258"))
        .await
        .unwrap()
        .expect("guarded unsupported DOI");
    let EuropePmcDetail::Legacy { requested, result } = detail else {
        panic!("expected legacy detail")
    };
    assert_eq!(requested.to_string(), "10.1002/(sici)1097-0258");
    assert_eq!(result.abstract_text.as_deref().map(str::len), Some(262_145));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn complete_empty_legacy_response_is_absence() {
    let bytes = br#"{"hitCount":0,"resultList":{"result":[]}}"#;
    assert!(
        parse_publication_detail(doi("10.1002/(sici)1097-0258"), bytes)
            .unwrap()
            .is_none()
    );
}
