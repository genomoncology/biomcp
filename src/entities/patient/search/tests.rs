use serde_json::json;

use super::*;
use crate::sources::fhir::test_server::{FixtureServer, Reply, capability};

const ALL_PARAMS: &[&str] = &["gender", "birthdate", "_has"];
const SNOMED: &str = "http://snomed.info/sct|44054006";

fn filters(
    gender: Option<&str>,
    born_after: Option<&str>,
    born_before: Option<&str>,
    condition: Option<&str>,
) -> PatientSearchFilters {
    PatientSearchFilters {
        gender: gender.map(Into::into),
        born_after: born_after.map(Into::into),
        born_before: born_before.map(Into::into),
        condition: condition.map(Into::into),
    }
}

fn query(filters: &PatientSearchFilters) -> PatientQuery {
    PatientQuery::parse(filters).expect("valid filters")
}

/// A server that lists `params` in `metadata` and answers every Patient
/// search with `page`.
async fn server(params: &'static [&'static str], page: serde_json::Value) -> FixtureServer {
    FixtureServer::start(move |target| {
        if target == "/fhir/metadata" {
            Reply::json(200, capability(params))
        } else {
            Reply::json(200, page.clone())
        }
    })
    .await
}

fn client(server: &FixtureServer) -> FhirClient {
    FhirClient::new(&format!("{}/fhir", server.base)).expect("client")
}

fn empty_bundle() -> serde_json::Value {
    json!({"resourceType": "Bundle", "type": "searchset", "total": 0, "entry": []})
}

#[tokio::test]
async fn each_filter_combination_sends_the_exact_strict_query() {
    let elements = "_elements=id%2Cgender%2CbirthDate";
    let has = "_has%3ACondition%3Apatient%3Acode=http%3A%2F%2Fsnomed.info%2Fsct%7C44054006";
    let cases = [
        (
            filters(Some("female"), None, None, None),
            format!("gender=female&{elements}&_count=10"),
        ),
        (
            filters(None, Some("1950-01-01"), None, None),
            format!("birthdate=gt1950-01-01&{elements}&_count=10"),
        ),
        (
            filters(None, None, Some("1960"), None),
            format!("birthdate=lt1960&{elements}&_count=10"),
        ),
        (
            filters(None, Some("1950-01"), Some("1960-12-31"), None),
            format!("birthdate=gt1950-01&birthdate=lt1960-12-31&{elements}&_count=10"),
        ),
        (
            filters(None, None, None, Some(SNOMED)),
            format!("{has}&{elements}&_count=10"),
        ),
        (
            filters(Some("male"), Some("1950"), Some("1960"), Some(SNOMED)),
            format!(
                "gender=male&birthdate=gt1950&birthdate=lt1960&{has}&{elements}&_count=10"
            ),
        ),
    ];
    for (filters, expected) in cases {
        let server = server(ALL_PARAMS, empty_bundle()).await;
        search_with_client(&client(&server), &query(&filters), 10)
            .await
            .expect("search");
        let seen = server.seen();
        assert_eq!(seen.len(), 2, "{expected}");
        assert_eq!(seen[0].target, "/fhir/metadata");
        assert_eq!(seen[1].target, format!("/fhir/Patient?{expected}"));
        assert_eq!(seen[1].prefer.as_deref(), Some("handling=strict"));
        assert_eq!(seen[1].cache_control.as_deref(), Some("no-store"));
    }
}

#[tokio::test]
async fn a_count_sends_the_filters_with_summary_count_and_strict_handling() {
    let server = server(ALL_PARAMS, json!({"resourceType": "Bundle", "total": 42})).await;
    let filters = filters(Some("female"), None, None, Some(SNOMED));
    let total = count_with_client(&client(&server), &query(&filters))
        .await
        .expect("count");
    assert_eq!(total, Some(42));
    let seen = server.seen();
    assert_eq!(seen.len(), 2);
    assert_eq!(
        seen[1].target,
        "/fhir/Patient?gender=female&_has%3ACondition%3Apatient%3Acode=http%3A%2F%2Fsnomed.info%2Fsct%7C44054006&_summary=count"
    );
    assert_eq!(seen[1].prefer.as_deref(), Some("handling=strict"));
}

#[tokio::test]
async fn a_count_with_no_total_reports_none_and_never_counts_entries() {
    let page = json!({"resourceType": "Bundle", "entry": [
        {"resource": {"resourceType": "Patient", "id": "p1"}},
        {"resource": {"resourceType": "Patient", "id": "p2"}}
    ]});
    let server = server(ALL_PARAMS, page).await;
    let total = count_with_client(&client(&server), &query(&filters(Some("female"), None, None, None)))
        .await
        .expect("count");
    assert_eq!(total, None);
}

#[tokio::test]
async fn a_parameter_missing_from_metadata_is_refused_before_the_search() {
    let cases: [(&'static [&'static str], PatientSearchFilters, &str); 3] = [
        (&["gender", "birthdate"], filters(None, None, None, Some(SNOMED)), "_has"),
        (&["gender", "_has"], filters(None, Some("1950"), None, None), "birthdate"),
        (&["birthdate", "_has"], filters(Some("female"), None, None, None), "gender"),
    ];
    for (params, filters, missing) in cases {
        for counting in [false, true] {
            let server = server(params, empty_bundle()).await;
            let client = client(&server);
            let query = query(&filters);
            let error = if counting {
                count_with_client(&client, &query).await.expect_err(missing)
            } else {
                search_with_client(&client, &query, 10).await.expect_err(missing)
            };
            assert!(
                matches!(
                    error,
                    BioMcpError::Fhir(FhirError::UnsupportedSearchParam(name)) if name == missing
                ),
                "{missing}: {error}"
            );
            assert!(error.to_string().contains(missing), "{error}");
            let targets = server
                .seen()
                .into_iter()
                .map(|seen| seen.target)
                .collect::<Vec<_>>();
            assert_eq!(targets, vec!["/fhir/metadata"], "{missing}");
        }
    }
}

#[test]
fn bad_values_are_refused() {
    let cases = [
        filters(Some("F"), None, None, None),
        filters(Some("male,female"), None, None, None),
        filters(None, Some("1950-13-01"), None, None),
        filters(None, Some("1950-02-30"), None, None),
        filters(None, Some("ge1950"), None, None),
        filters(None, Some("0000"), None, None),
        filters(None, Some("50-01-01"), None, None),
        filters(None, None, Some("1950,1960"), None),
        filters(None, None, Some("1950-1"), None),
        filters(None, None, None, Some("44054006")),
        filters(None, None, None, Some("|44054006")),
        filters(None, None, None, Some("http://snomed.info/sct|")),
        filters(None, None, None, Some("http://snomed.info/sct|1,2")),
        filters(None, None, None, Some("a|b|c")),
        PatientSearchFilters::default(),
    ];
    for filters in cases {
        assert!(PatientQuery::parse(&filters).is_err(), "{filters:?}");
    }
    for good in ["1950", "1950-12", "2000-02-29"] {
        assert!(
            PatientQuery::parse(&filters(None, Some(good), None, None)).is_ok(),
            "{good}"
        );
    }
}

#[test]
fn the_limit_runs_one_to_fifty() {
    assert!(check_limit(0).is_err());
    assert!(check_limit(51).is_err());
    assert!(check_limit(1).is_ok());
    assert!(check_limit(50).is_ok());
}

#[tokio::test]
async fn output_keeps_only_patient_id_gender_and_birth_date_up_to_the_limit() {
    let full = |id: &str| {
        json!({
            "resourceType": "Patient",
            "id": id,
            "gender": "female",
            "birthDate": "1970-01-01",
            "name": [{"family": "Synthleak", "given": ["Nameleak"]}],
            "address": [{"line": ["1 Addressleak Way"], "city": "Cityleak"}],
            "telecom": [{"system": "phone", "value": "555-0100-leak"}]
        })
    };
    let page = json!({"resourceType": "Bundle", "type": "searchset", "entry": [
        {"resource": {"resourceType": "Condition", "id": "cond-1", "code": {"text": "Conditionleak"}}},
        {"resource": full("a/b"), "search": {"mode": "match"}},
        {"resource": full("p1"), "search": {"mode": "match"}},
        {"resource": full("p2"), "search": {"mode": "include"}},
        {"resource": full("p3")},
        {"resource": full("p4")}
    ]});
    let server = server(ALL_PARAMS, page).await;
    let rows = search_with_client(&client(&server), &query(&filters(Some("female"), None, None, None)), 3)
        .await
        .expect("search");
    assert_eq!(
        rows.iter().map(|row| row.id.as_deref()).collect::<Vec<_>>(),
        vec![None, Some("p1"), Some("p3")]
    );
    let rendered = serde_json::to_string(&rows).expect("json");
    for leak in ["leak", "Condition", "a/b", "cond-1"] {
        assert!(!rendered.contains(leak), "{leak}: {rendered}");
    }
    assert_eq!(rows[1].gender.as_deref(), Some("female"));
    assert_eq!(rows[1].birth_date.as_deref(), Some("1970-01-01"));
}
