use http_cache_reqwest::CacheMode;
use serde_json::json;

use super::test_server::{FixtureServer, Reply, condition, condition_bundle};
use super::*;

const ID: &str = "SYNTH-PT-7Q.42";

fn id() -> PatientId {
    PatientId::parse(ID).expect("valid synthetic ID")
}

fn assert_names_nothing(message: &str, servers: &[&FixtureServer]) {
    assert!(!message.contains(ID), "message names the ID: {message}");
    assert!(!message.contains("://"), "message names a URL: {message}");
    for server in servers {
        let authority = server.base.trim_start_matches("http://");
        assert!(
            !message.contains(authority),
            "message names the server: {message}"
        );
    }
}

#[test]
fn patient_ids_outside_the_fhir_id_rule_are_refused_without_echo() {
    for raw in [
        "",
        "a/b",
        "../Patient",
        "a?b",
        "a#b",
        "a&b",
        "a b",
        "a%2Fb",
        "é",
        ".",
        "..",
        "...",
        &"a".repeat(65),
    ] {
        let error = PatientId::parse(raw).expect_err(raw);
        let message = error.to_string();
        if !raw.is_empty() {
            assert!(!message.contains(raw), "echoed {raw}: {message}");
        }
    }
    assert!(PatientId::parse(&"a".repeat(64)).is_ok());
    assert!(PatientId::parse("Ab-9.z").is_ok());
    assert!(PatientId::parse("a.").is_ok());
    assert!(PatientId::parse(".a").is_ok());
}

#[test]
fn query_values_and_path_segments_are_url_encoded() {
    let base = FhirBase::parse("http://fhir.test/r4/").expect("base");
    let url = base.resource_url(&["Condition"], &[("patient", "a b&c=d/e?f#g")]);
    assert_eq!(url.path(), "/r4/Condition");
    assert_eq!(url.query(), Some("patient=a+b%26c%3Dd%2Fe%3Ff%23g"));
    let url = base.resource_url(&["Patient", "x/y?z"], &[]);
    assert_eq!(url.path(), "/r4/Patient/x%2Fy%3Fz");
    assert_eq!(url.query(), None);
}

#[test]
fn the_base_contains_only_its_origin_and_path() {
    let base = FhirBase::parse("http://fhir.test:8080/fhir").expect("base");
    for inside in [
        "http://fhir.test:8080/fhir",
        "http://fhir.test:8080/fhir?_getpages=abc",
        "http://FHIR.test:8080/fhir/Condition?patient=x",
    ] {
        assert!(base.contains(&Url::parse(inside).unwrap()), "{inside}");
    }
    for outside in [
        "https://fhir.test:8080/fhir/Condition",
        "http://fhir.test:8081/fhir/Condition",
        "http://other.test:8080/fhir/Condition",
        "http://fhir.test:8080/fhirx/Condition",
        "http://fhir.test:8080/other/Condition",
        "http://user:pw@fhir.test:8080/fhir/Condition",
    ] {
        assert!(!base.contains(&Url::parse(outside).unwrap()), "{outside}");
    }
    for bad in [
        "ftp://fhir.test/fhir",
        "http://user:pw@fhir.test/fhir",
        "http://fhir.test/fhir?x=1",
        "not a url",
    ] {
        assert_eq!(FhirBase::parse(bad), Err(FhirError::BadBase), "{bad}");
    }
}

#[tokio::test]
async fn the_request_function_sets_no_store() {
    let client = FhirClient::new("http://fhir.test/fhir").expect("client");
    for url in [
        client.base().resource_url(&["Patient", ID], &[]),
        client
            .base()
            .resource_url(&["Condition"], &[("patient", ID), ("_count", "100")]),
    ] {
        let mut request = client.request_for_test(url);
        assert_eq!(
            request.extensions().get::<CacheMode>(),
            Some(&no_store_mode())
        );
        let built = request.build().expect("request");
        assert_eq!(
            built
                .headers()
                .get(CACHE_CONTROL)
                .and_then(|v| v.to_str().ok()),
            Some("no-store")
        );
    }
}

#[tokio::test]
async fn a_two_page_bundle_returns_both_pages_and_every_request_is_no_store() {
    let server = FixtureServer::start(|target| {
        if target.starts_with("/fhir/Condition?") {
            let next = "/fhir?_getpages=synthetic&_getpagesoffset=1";
            return Reply::json(200, condition_bundle(&[condition("c1", "one")], Some(next)));
        }
        if target.starts_with("/fhir?_getpages=synthetic") {
            return Reply::json(200, condition_bundle(&[condition("c2", "two")], None));
        }
        Reply::json(404, json!({}))
    })
    .await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let walk = client.search_conditions(&id()).await.expect("walk");
    assert_eq!(walk.matches.len(), 2);
    assert_eq!(walk.stop, None);
    let seen = server.seen();
    assert_eq!(seen.len(), 2);
    assert_eq!(
        seen[0].target,
        format!("/fhir/Condition?patient={ID}&_count=100")
    );
    for request in &seen {
        assert_eq!(request.cache_control.as_deref(), Some("no-store"));
    }
}

#[tokio::test]
async fn a_repeated_next_link_stops_the_walk() {
    let server = FixtureServer::start(|target| {
        // Page 1 links to page 2, and page 2 links to itself.
        let page = if target.contains("page=2") { "2" } else { "1" };
        let next = "/fhir?page=2";
        Reply::json(
            200,
            condition_bundle(&[condition(&format!("c{page}"), page)], Some(next)),
        )
    })
    .await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let walk = client.search_conditions(&id()).await.expect("walk");
    assert_eq!(walk.stop, Some(WalkStop::RepeatedLink));
    assert_eq!(walk.matches.len(), 2);
    assert_eq!(server.seen().len(), 2);
}

#[tokio::test]
async fn a_next_link_to_another_origin_is_never_followed() {
    let other = FixtureServer::start(|_| {
        Reply::json(200, condition_bundle(&[condition("x", "other")], None))
    })
    .await;
    let other_next = format!("{}/fhir?page=2", other.base);
    let server = FixtureServer::start(move |_| {
        Reply::json(
            200,
            condition_bundle(&[condition("c1", "one")], Some(&other_next)),
        )
    })
    .await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let walk = client.search_conditions(&id()).await.expect("walk");
    assert_eq!(walk.stop, Some(WalkStop::OffBaseLink));
    assert_eq!(walk.matches.len(), 1);
    assert!(other.seen().is_empty(), "followed a next link off origin");
}

#[tokio::test]
async fn a_next_link_off_the_base_path_is_never_followed() {
    let server = FixtureServer::start(|target| {
        if target.starts_with("/elsewhere") {
            return Reply::json(200, condition_bundle(&[condition("x", "x")], None));
        }
        Reply::json(
            200,
            condition_bundle(&[condition("c1", "one")], Some("/elsewhere?page=2")),
        )
    })
    .await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let walk = client.search_conditions(&id()).await.expect("walk");
    assert_eq!(walk.stop, Some(WalkStop::OffBaseLink));
    assert_eq!(server.seen().len(), 1);
}

#[tokio::test]
async fn a_redirect_to_another_origin_during_the_walk_is_refused() {
    let other = FixtureServer::start(|_| {
        Reply::json(200, condition_bundle(&[condition("x", "other")], None))
    })
    .await;
    let other_base = other.base.clone();
    let server = FixtureServer::start(move |target| {
        if target.starts_with("/fhir?page=2") {
            return Reply::redirect(format!("{other_base}/fhir?page=2&patient={ID}"));
        }
        Reply::json(
            200,
            condition_bundle(&[condition("c1", "one")], Some("/fhir?page=2")),
        )
    })
    .await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let walk = client.search_conditions(&id()).await.expect("walk");
    assert_eq!(walk.stop, Some(WalkStop::OffBaseRedirect));
    assert_eq!(walk.matches.len(), 1);
    assert!(other.seen().is_empty(), "followed a redirect off origin");
}

#[tokio::test]
async fn a_redirect_within_the_base_is_followed() {
    let server = FixtureServer::start(|target| {
        if target.starts_with("/fhir/Condition?") {
            return Reply::redirect("/fhir/moved?page=1");
        }
        Reply::json(200, condition_bundle(&[condition("c1", "one")], None))
    })
    .await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let walk = client.search_conditions(&id()).await.expect("walk");
    assert_eq!(walk.stop, None);
    assert_eq!(walk.matches.len(), 1);
    let seen = server.seen();
    assert_eq!(seen.len(), 2);
    assert!(
        seen.iter()
            .all(|request| request.cache_control.as_deref() == Some("no-store"))
    );
}

#[tokio::test]
async fn the_walk_stops_at_twenty_pages() {
    let server = FixtureServer::start(|target| {
        let page = target
            .split("page=")
            .nth(1)
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(0);
        let next = format!("/fhir?page={}", page + 1);
        Reply::json(
            200,
            condition_bundle(&[condition(&format!("c{page}"), "x")], Some(&next)),
        )
    })
    .await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let walk = client.search_conditions(&id()).await.expect("walk");
    assert_eq!(walk.stop, Some(WalkStop::PageCap));
    assert_eq!(walk.matches.len(), MAX_PAGES);
    assert_eq!(server.seen().len(), MAX_PAGES);
}

#[tokio::test]
async fn a_server_error_is_retried_by_exactly_one_retry_layer() {
    let server =
        FixtureServer::start(|_| Reply::json(500, json!({"resourceType": "OperationOutcome"})))
            .await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let error = client
        .search_conditions(&id())
        .await
        .expect_err("server error");
    assert_eq!(error, FhirError::Status(500));
    assert_names_nothing(&error.to_string(), &[&server]);
    // One initial request plus the shared layer's three retries.
    assert_eq!(server.seen().len(), 4);
}

#[tokio::test]
async fn a_patient_redirect_to_another_host_fails_naming_no_url() {
    let other =
        FixtureServer::start(|_| Reply::json(200, json!({"resourceType": "Patient", "id": ID})))
            .await;
    let other_base = other.base.clone();
    let server =
        FixtureServer::start(move |target| Reply::redirect(format!("{other_base}{target}"))).await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let error = client
        .read_patient(&id())
        .await
        .expect_err("redirect refused");
    assert_eq!(error, FhirError::Redirect);
    let rendered = BioMcpError::Fhir(error).to_string();
    assert_names_nothing(&rendered, &[&server, &other]);
    assert!(other.seen().is_empty(), "followed a redirect off origin");
}

#[tokio::test]
async fn a_missing_patient_is_not_found_naming_no_id() {
    let server = FixtureServer::start(|target| {
        Reply::json(
            404,
            json!({"resourceType": "OperationOutcome", "issue": [{"severity": "error", "code": "not-found", "diagnostics": format!("Resource {target} is not known")}]}),
        )
    })
    .await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let error = client.read_patient(&id()).await.expect_err("not found");
    assert_eq!(error, FhirError::NotFound);
    assert_names_nothing(&BioMcpError::Fhir(error).to_string(), &[&server]);
}

#[test]
fn an_unset_base_names_the_variable() {
    let message = BioMcpError::Fhir(FhirError::NotConfigured).to_string();
    assert!(message.contains(FHIR_BASE_ENV), "{message}");
}
