//! Tests for the ORCID source client.

use super::*;

#[test]
fn request_plans_carry_exact_paths_headers_and_bearer_mode_only() {
    let person = OrcidRequest::Person {
        id: "0000-0002-1825-0097".into(),
    };
    let works = OrcidRequest::Works {
        id: "0000-0002-1825-0097".into(),
    };
    let plan = person.plan();
    assert_eq!(plan.path, "0000-0002-1825-0097/person");
    assert_eq!(plan.header_value("Accept"), Some(ORCID_ACCEPT));
    assert!(
        !plan
            .headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("authorization"))
    );
    let plan = works.plan();
    assert_eq!(plan.path, "0000-0002-1825-0097/works");
}

#[test]
fn token_classification_matches_the_frozen_three_states() {
    assert_eq!(classify_token(None), OrcidToken::Missing);
    assert_eq!(classify_token(Some("   ".into())), OrcidToken::Missing);
    assert_eq!(
        classify_token(Some("secret-token".into())),
        OrcidToken::Valid("secret-token".into())
    );
    assert_eq!(
        classify_token(Some(" bad token".into())),
        OrcidToken::Invalid
    );
    assert_eq!(
        classify_token(Some("tab\ttoken".into())),
        OrcidToken::Invalid
    );
    assert_eq!(classify_token(Some("é".into())), OrcidToken::Invalid);
    assert_eq!(classify_token(Some("x".repeat(4097))), OrcidToken::Invalid);
    assert_eq!(
        classify_token(Some("x".repeat(4096))),
        OrcidToken::Valid("x".repeat(4096))
    );
}

#[test]
fn person_path_and_public_name_precedence() {
    let response: OrcidPersonResponse = serde_json::from_str(
        r#"{"path":"/0000-0002-1825-0097/person","name":{"visibility":"PUBLIC","given-names":{"value":" Josiah "},"family-name":{"value":"Carberry"}}}"#,
    )
    .unwrap();
    let response = response.validate("0000-0002-1825-0097").unwrap();
    assert_eq!(response.public_display_name().unwrap(), "Josiah Carberry");
    let wrong: OrcidPersonResponse =
        serde_json::from_str(r#"{"path":"/other/person","name":null}"#).unwrap();
    assert!(wrong.validate("0000-0002-1825-0097").is_err());
}

#[test]
fn works_validation_selects_representatives_and_rejects_bad_shapes() {
    let body = r#"{"path":"/i/works","group":[{"work-summary":[
        {"visibility":"PUBLIC","put-code":7,"display-index":"2","title":{"title":{"value":"Second"}}},
        {"visibility":"PUBLIC","put-code":9,"display-index":"10","title":{"title":{"value":"First"}}},
        {"visibility":"PRIVATE","put-code":99,"display-index":"99","title":{"title":{"value":"private"}}}
    ]}]}"#;
    let response: OrcidWorksResponse = serde_json::from_str(body).unwrap();
    let selected = response.validate("i").unwrap().selected_works().unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].put_code, 9, "greatest display-index wins");
    assert_eq!(selected[0].title, "First");

    let zero_code = r#"{"path":"/i/works","group":[{"work-summary":[
        {"visibility":"PUBLIC","put-code":0,"display-index":"1","title":{"title":{"value":"x"}}}
    ]}]}"#;
    let response: OrcidWorksResponse = serde_json::from_str(zero_code).unwrap();
    assert!(response.validate("i").unwrap().selected_works().is_err());

    let leading_zero_index = r#"{"path":"/i/works","group":[{"work-summary":[
        {"visibility":"PUBLIC","put-code":1,"display-index":"01","title":{"title":{"value":"x"}}}
    ]}]}"#;
    let response: OrcidWorksResponse = serde_json::from_str(leading_zero_index).unwrap();
    assert!(response.validate("i").unwrap().selected_works().is_err());
}

#[test]
fn works_root_path_mismatch_fails_the_contract() {
    // A works document rooted at another ORCID record must never project: the
    // root path is part of the identity contract.
    let body = r#"{"path":"/0000-0002-1825-0097/works","group":[]}"#;
    let response: OrcidWorksResponse = serde_json::from_str(body).unwrap();
    assert!(
        response.validate("0000-0002-1825-0098").is_err(),
        "a mismatched works root must fail"
    );
}

#[test]
fn an_absent_group_key_fails_decoding() {
    // A present `group` array is required: absence is malformed, not empty.
    let body = r#"{"path":"/0000-0002-1825-0097/works"}"#;
    assert!(serde_json::from_str::<OrcidWorksResponse>(body).is_err());
}

// ---- Ticket 1142 closing pass: loopback transport and works bounds ----

mod closing {
    use super::super::*;
    use std::sync::Arc;
    use std::sync::Mutex as StdMutex;

    const VALID_ID: &str = "0000-0002-1825-0097";

    fn person_body() -> String {
        format!(
            r#"{{"path":"/{VALID_ID}/person","name":{{"visibility":"PUBLIC","given-names":{{"value":"Josiah"}},"family-name":{{"value":"Carberry"}}}}}}"#
        )
    }

    /// Scripted loopback: each accepted connection consumes the next
    /// `(status, retry_after, body)` response; every request line is
    /// logged.
    async fn spawn_scripted_orcid(
        responses: Vec<(u16, Option<&'static str>, String)>,
    ) -> (String, Arc<StdMutex<Vec<String>>>) {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
        let requests = Arc::new(StdMutex::new(Vec::new()));
        let logged = requests.clone();
        let counter = Arc::new(StdMutex::new(0usize));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind orcid fixture");
        let address = listener.local_addr().expect("orcid fixture address");
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let logged = logged.clone();
                let responses = responses.clone();
                let counter = counter.clone();
                tokio::spawn(async move {
                    let mut request = vec![0_u8; 16 * 1024];
                    let length = stream.read(&mut request).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&request[..length]);
                    if let Some(target) = request.split_whitespace().nth(1) {
                        logged.lock().unwrap().push(target.to_string());
                    }
                    let index = {
                        let mut guard = counter.lock().unwrap();
                        let index = *guard;
                        *guard += 1;
                        index
                    };
                    let (status, retry_after, body) =
                        responses
                            .get(index)
                            .cloned()
                            .unwrap_or((500, None, String::new()));
                    let retry = retry_after
                        .map(|value| format!("Retry-After: {value}\r\n"))
                        .unwrap_or_default();
                    let response = format!(
                        "HTTP/1.1 {status} X\r\nContent-Type: application/vnd.orcid+json\r\n{retry}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                });
            }
        });
        (format!("http://{address}"), requests)
    }

    struct OrcidEnv {
        previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }
    impl OrcidEnv {
        fn new(base: &str) -> Self {
            let mut env = Self {
                previous: Vec::new(),
            };
            for (key, value) in [
                (ORCID_BASE_ENV, base.to_string()),
                (ORCID_TOKEN_ENV, "fixture-public-read-token".to_string()),
                ("BIOMCP_TEST_UNPACED_ORIGIN", base.to_string()),
            ] {
                env.previous.push((key, std::env::var_os(key)));
                // SAFETY: serial-guarded test environment mutation.
                unsafe { std::env::set_var(key, value) };
            }
            env
        }
    }
    impl Drop for OrcidEnv {
        fn drop(&mut self) {
            for (key, value) in self.previous.drain(..) {
                // SAFETY: restoring the serial-guarded environment.
                unsafe {
                    match value {
                        Some(value) => std::env::set_var(key, value),
                        None => std::env::remove_var(key),
                    }
                }
            }
        }
    }

    fn deadline(seconds: u64) -> Instant {
        Instant::now() + Duration::from_secs(seconds)
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn transient_429_retries_once_under_the_pacing_gap_then_succeeds() {
        let (base, requests) = spawn_scripted_orcid(vec![
            (429, Some("1"), String::new()),
            (200, None, person_body()),
        ])
        .await;
        let _env = OrcidEnv::new(&base);
        let started = std::time::Instant::now();
        let person = OrcidClient::new()
            .expect("client")
            .person(VALID_ID, deadline(30))
            .await
            .expect("second attempt succeeds");
        assert_eq!(
            person.public_display_name().expect("public name"),
            "Josiah Carberry"
        );
        let elapsed = started.elapsed();
        assert!(
            elapsed >= Duration::from_secs(1),
            "paced retry took {elapsed:?}"
        );
        let logged = requests.lock().unwrap().clone();
        assert_eq!(logged, vec!["/0000-0002-1825-0097/person".to_string(); 2]);
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn persistent_server_errors_stop_at_four_physical_gets_with_a_sanitized_error() {
        let (base, requests) = spawn_scripted_orcid(vec![(500, None, String::new()); 4]).await;
        let _env = OrcidEnv::new(&base);
        let error = OrcidClient::new()
            .expect("client")
            .person(VALID_ID, deadline(30))
            .await
            .expect_err("cap exhausted");
        assert_eq!(error.code(), "api");
        let logged = requests.lock().unwrap().clone();
        assert_eq!(logged.len(), 4, "exactly four physical GETs");
    }

    /// Raw scripted loopback: each connection writes the exact response
    /// bytes supplied, so headers can lie about the body the way hostile or
    /// buggy servers do.
    async fn spawn_raw_orcid(responses: Vec<String>) -> (String, Arc<StdMutex<Vec<String>>>) {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
        let requests = Arc::new(StdMutex::new(Vec::new()));
        let logged = requests.clone();
        let counter = Arc::new(StdMutex::new(0usize));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind orcid raw fixture");
        let address = listener.local_addr().expect("orcid raw fixture address");
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let logged = logged.clone();
                let responses = responses.clone();
                let counter = counter.clone();
                tokio::spawn(async move {
                    let mut request = vec![0_u8; 16 * 1024];
                    let length = stream.read(&mut request).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&request[..length]);
                    if let Some(target) = request.split_whitespace().nth(1) {
                        logged.lock().unwrap().push(target.to_string());
                    }
                    let index = {
                        let mut guard = counter.lock().unwrap();
                        let index = *guard;
                        *guard += 1;
                        index
                    };
                    if let Some(response) = responses.get(index) {
                        let _ = stream.write_all(response.as_bytes()).await;
                    }
                });
            }
        });
        (format!("http://{address}"), requests)
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn a_declared_oversize_person_body_fails_the_pre_read_check_in_one_get() {
        // Content-Length declares 600 KiB, above the 512 KiB person cap: the
        // response-body policy rejects it before any body byte is read, as a
        // bounded hard failure rather than a retry that burns the budget.
        let response = format!(
            "HTTP/1.1 200 X\r\nContent-Type: application/vnd.orcid+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            600 * 1024
        );
        let (base, requests) = spawn_raw_orcid(vec![response]).await;
        let _env = OrcidEnv::new(&base);
        let error = OrcidClient::new()
            .expect("client")
            .person(VALID_ID, deadline(30))
            .await
            .expect_err("declared oversize body");
        assert!(is_body_limit(&error), "{error:?}");
        assert_eq!(
            requests.lock().unwrap().len(),
            1,
            "an oversize body is never retried"
        );
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn a_streamed_oversize_person_body_fails_once_without_retry() {
        // The declared length sits exactly at the cap, but the stream
        // continues past it: the read-side limit fires mid-body as the same
        // bounded hard failure.
        let declared = ORCID_PERSON_BODY_LIMIT;
        let mut body = String::new();
        body.push_str(r#"{"pad":""#);
        for _ in 0..(declared + 64 * 1024) {
            body.push('x');
        }
        body.push('}');
        let response = format!(
            "HTTP/1.1 200 X\r\nContent-Type: application/vnd.orcid+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let (base, requests) = spawn_raw_orcid(vec![response]).await;
        let _env = OrcidEnv::new(&base);
        let error = OrcidClient::new()
            .expect("client")
            .person(VALID_ID, deadline(30))
            .await
            .expect_err("streamed oversize body");
        assert!(is_body_limit(&error), "{error:?}");
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn rejected_credentials_fail_immediately_without_retry() {
        let (base, requests) = spawn_scripted_orcid(vec![(401, None, String::new())]).await;
        let _env = OrcidEnv::new(&base);
        let error = OrcidClient::new()
            .expect("client")
            .person(VALID_ID, deadline(30))
            .await
            .expect_err("401");
        assert!(matches!(error, BioMcpError::ApiKeyRejected { .. }));
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn not_found_is_a_sanitized_single_get() {
        let (base, requests) = spawn_scripted_orcid(vec![(404, None, String::new())]).await;
        let _env = OrcidEnv::new(&base);
        let error = OrcidClient::new()
            .expect("client")
            .person(VALID_ID, deadline(30))
            .await
            .expect_err("404");
        assert_eq!(error.code(), "api");
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn client_errors_are_never_retried() {
        let (base, requests) = spawn_scripted_orcid(vec![(400, None, String::new())]).await;
        let _env = OrcidEnv::new(&base);
        let error = OrcidClient::new()
            .expect("client")
            .person(VALID_ID, deadline(30))
            .await
            .expect_err("400");
        assert_eq!(error.code(), "api");
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn an_expired_deadline_admits_no_second_attempt_and_paces_the_first() {
        let (base, requests) = spawn_scripted_orcid(vec![(200, None, person_body()); 2]).await;
        let _env = OrcidEnv::new(&base);
        let client = OrcidClient::new().expect("client");
        client
            .person(VALID_ID, deadline(30))
            .await
            .expect("first call succeeds");
        // The second logical call cannot admit a physical attempt before
        // its absolute deadline: the one-second pacing gap is longer than
        // the remaining budget, so the bounded error returns with no new
        // GET.
        let error = client
            .person(VALID_ID, Instant::now() + Duration::from_millis(100))
            .await
            .expect_err("deadline");
        assert_eq!(error.code(), "api");
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    fn summary(
        put_code: u64,
        display_index: &str,
        title: &str,
        pmid: Option<&str>,
        visibility: &str,
    ) -> serde_json::Value {
        let mut summary = serde_json::json!({
            "visibility": visibility,
            "put-code": put_code,
            "display-index": display_index,
            "title": {"title": {"value": title}},
        });
        if let Some(pmid) = pmid {
            summary["external-ids"] = serde_json::json!({
                "external-id": [
                    {"external-id-type": "pmid", "external-id-value": pmid, "external-id-relationship": "SELF"}
                ]
            });
        }
        summary
    }

    fn works_body(groups: &[serde_json::Value]) -> String {
        serde_json::json!({"path": "/i/works", "group": groups}).to_string()
    }

    fn selected(body: &str) -> Result<Vec<OrcidSelectedWork>, BioMcpError> {
        let response: OrcidWorksResponse = serde_json::from_str(body).unwrap();
        response.validate("i")?.selected_works()
    }

    #[test]
    fn a_group_without_summaries_is_a_contract_error() {
        let body = works_body(&[serde_json::json!({"work-summary": []})]);
        assert!(
            selected(&body).is_err(),
            "1-64 summaries per group is the bound"
        );
    }

    #[test]
    fn one_and_sixty_four_summaries_are_the_group_bound_edges() {
        for count in [1_usize, 64] {
            let summaries: Vec<_> = (0..count)
                .map(|index| summary(1000 + index as u64, "1", "Title", None, "PUBLIC"))
                .collect();
            let body = works_body(&[serde_json::json!({"work-summary": summaries})]);
            let works = selected(&body).expect("inside the bound");
            assert_eq!(works.len(), 1);
        }
        let summaries: Vec<_> = (0..65)
            .map(|index| summary(index as u64 + 1, "1", "Title", None, "PUBLIC"))
            .collect();
        let body = works_body(&[serde_json::json!({"work-summary": summaries})]);
        assert!(
            selected(&body).is_err(),
            "65 summaries per group is a contract error"
        );
    }

    #[test]
    fn ten_thousand_groups_is_the_hard_bound() {
        let group = serde_json::json!({"work-summary": [summary(7, "1", "Only", None, "PUBLIC")]});
        for count in [10_000_usize, 10_001] {
            let body = works_body(&vec![group.clone(); count]);
            let result = selected(&body);
            if count == 10_000 {
                result.expect("at the bound");
            } else {
                result.expect_err("past the bound");
            }
        }
    }

    #[test]
    fn representative_ties_prefer_the_lowest_put_code_then_original_order() {
        let body = works_body(&[serde_json::json!({"work-summary": [
            summary(5, "2", "Higher code", None, "PUBLIC"),
            summary(3, "2", "Lower code wins", None, "PUBLIC")
        ]})]);
        let works = selected(&body).expect("valid");
        assert_eq!(works[0].title, "Lower code wins");

        let body = works_body(&[serde_json::json!({"work-summary": [
            summary(7, "2", "First in order wins ties", None, "PUBLIC"),
            summary(7, "2", "Second", None, "PUBLIC")
        ]})]);
        let works = selected(&body).expect("valid");
        assert_eq!(works[0].title, "First in order wins ties");
    }

    #[test]
    fn a_present_year_outside_1000_9999_is_a_contract_error() {
        let mut bad = summary(7, "1", "Bad year", None, "PUBLIC");
        bad["publication-date"] = serde_json::json!({"year": {"value": "0999"}});
        let body = works_body(&[serde_json::json!({"work-summary": [bad]})]);
        assert!(selected(&body).is_err(), "0999 is outside 1000-9999");
        let mut good = summary(8, "1", "Good year", None, "PUBLIC");
        good["publication-date"] = serde_json::json!({"year": {"value": "2024"}});
        let body = works_body(&[serde_json::json!({"work-summary": [good]})]);
        assert_eq!(selected(&body).expect("valid")[0].year, Some(2024));
    }

    #[test]
    fn identifiers_from_excluded_locations_never_reach_the_selected_work() {
        let hostile = serde_json::json!({
            "external-ids": {"external-id": [
                {"external-id-type": "pmid", "external-id-value": "9991", "external-id-relationship": "SELF"}
            ]},
            "work-summary": [
                {"visibility": "PRIVATE", "put-code": 91, "display-index": "50",
                 "title": {"title": {"value": "Private summary"}},
                 "external-ids": {"external-id": [
                    {"external-id-type": "pmid", "external-id-value": "9992", "external-id-relationship": "SELF"}
                 ]}},
                {"visibility": "PUBLIC", "put-code": 92, "display-index": "1",
                 "title": {"title": {"value": "Public but not selected"}},
                 "external-ids": {"external-id": [
                    {"external-id-type": "pmid", "external-id-value": "9993", "external-id-relationship": "SELF"}
                 ]}},
                summary(42, "2", "Selected representative", Some("42"), "PUBLIC")
            ]
        });
        let body = works_body(&[hostile]);
        let works = selected(&body).expect("valid");
        assert_eq!(works.len(), 1);
        assert_eq!(works[0].put_code, 42);
        let flattened: Vec<(String, String)> = works[0].external_ids.clone();
        assert_eq!(flattened, vec![("pmid".into(), "42".into())]);
        for (_, value) in flattened {
            assert!(
                !value.contains("999"),
                "excluded identifier leaked: {value}"
            );
        }
    }
}

#[test]
fn content_type_gate_accepts_only_the_two_orcid_media_types() {
    let header = |raw: &str| reqwest::header::HeaderValue::from_str(raw).unwrap();
    assert!(ensure_orcid_content_type(Some(&header("application/vnd.orcid+json"))).is_ok());
    assert!(ensure_orcid_content_type(Some(&header("application/json; charset=utf-8"))).is_ok());
    assert!(ensure_orcid_content_type(Some(&header("APPLICATION/JSON"))).is_ok());
    assert!(ensure_orcid_content_type(Some(&header("text/html"))).is_err());
    assert!(ensure_orcid_content_type(None).is_err());
}
