use reqwest::StatusCode;

use super::super::{
    ScriptedResponse,
    clingen_allele_registry::ClinGenAlleleRegistryClient,
    clingen_cspec::CspecClient,
    clingen_erepo::ERepoClient,
    clingen_ldh::{ClinGenLdhClient, DIRECT_BODY_LIMIT},
    scripted_client,
};
use crate::entities::variant::CarNormalizationStatus;
use crate::error::SourceProvider;

const CAR_HOST: &str = "reg.genome.network";
const CSPEC_HOST: &str = "cspec.clinicalgenome.org";
const EREPO_HOST: &str = "erepo.clinicalgenome.org";
const LDH_HOST: &str = "ldh.genome.network";

#[tokio::test]
async fn car_outage_does_not_overwrite_erepo_healthy_empty_or_version() {
    let client = scripted_client([
        (
            CAR_HOST,
            ScriptedResponse::Http {
                status: StatusCode::SERVICE_UNAVAILABLE,
                headers: vec![("X-CAR-Version", "car-test-version")],
                body: "upstream CAR diagnostic",
            },
        ),
        (
            EREPO_HOST,
            ScriptedResponse::Http {
                status: StatusCode::NOT_FOUND,
                headers: vec![("content-type", "application/json")],
                body: r#"{"status":{"code":404,"message":"No records found"}}"#,
            },
        ),
    ])
    .expect("scripted client");
    let car =
        ClinGenAlleleRegistryClient::with_test_client(client.clone(), "https://reg.genome.network");
    let erepo = ERepoClient::with_test_client(client, "https://erepo.clinicalgenome.org");

    let car = car
        .normalize("NM_004333.6(BRAF):c.1799T>A")
        .await
        .expect("CAR result");
    let erepo = erepo.summary("CAID:CA000001").await.expect("ERepo result");

    assert_eq!(car.status, CarNormalizationStatus::Unavailable);
    assert_eq!(
        car.provenance.car_version.as_deref(),
        Some("car-test-version")
    );
    assert_eq!(
        erepo
            .pointer("/status/code")
            .and_then(serde_json::Value::as_i64),
        Some(200)
    );
    assert_eq!(
        erepo
            .get("data")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(0)
    );
}

#[tokio::test]
async fn receipt_backed_ldh_captures_reach_the_production_medium_and_direct_clients() {
    let medium_client = scripted_client([(
        LDH_HOST,
        ScriptedResponse::Http {
            status: StatusCode::OK,
            headers: vec![("content-type", "application/json")],
            body: include_str!("../../../testdata/sources/clingen_ldh/ca288251-medium.json"),
        },
    )])
    .expect("scripted client");
    let medium = ClinGenLdhClient::with_test_client(medium_client)
        .medium("CA288251")
        .await
        .expect("recorded medium response");
    assert!(
        medium
            .pointer("/data/VariantsInLiterature")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|rows| rows.iter().any(|row| {
                row.get("entId").and_then(serde_json::Value::as_str) == Some("PMC8710334")
            }))
    );

    let empty_medium_client = scripted_client([(
        LDH_HOST,
        ScriptedResponse::Http {
            status: StatusCode::OK,
            headers: vec![("content-type", "application/json")],
            body: include_str!("../../../testdata/sources/clingen_ldh/ca288251-medium-empty.json"),
        },
    )])
    .expect("scripted client");
    let empty_medium = ClinGenLdhClient::with_test_client(empty_medium_client)
        .medium("CA999999")
        .await
        .expect("recorded empty medium response");
    assert!(empty_medium.pointer("/data/VariantsInLiterature").is_none());

    let direct_client = scripted_client([(
        LDH_HOST,
        ScriptedResponse::Http {
            status: StatusCode::OK,
            headers: vec![("content-type", "application/json")],
            body: include_str!(
                "../../../testdata/sources/clingen_ldh/ca288251-pmc8710334-direct.json"
            ),
        },
    )])
    .expect("scripted client");
    let (direct, body_bytes) = ClinGenLdhClient::with_test_client(direct_client)
        .direct(
            "https://ldh.genome.network/ldh/dss/cg/ns/ldh/set/variants_in_literature/id/PMC8710334/data",
            DIRECT_BODY_LIMIT,
        )
        .await
        .expect("recorded direct response");
    assert_eq!(
        body_bytes,
        include_bytes!("../../../testdata/sources/clingen_ldh/ca288251-pmc8710334-direct.json")
            .len()
    );
    assert!(
        direct
            .get("annotations")
            .is_some_and(serde_json::Value::is_array)
    );
}

#[tokio::test]
async fn cspec_and_ldh_transport_failures_are_source_labelled_and_do_not_erase_peer_results() {
    let cspec_failure_client = scripted_client([
        (
            CSPEC_HOST,
            ScriptedResponse::TransportError("cspec-secret-body"),
        ),
        (
            LDH_HOST,
            ScriptedResponse::Http {
                status: StatusCode::OK,
                headers: vec![("content-type", "application/json")],
                body: r#"{"ldh":"independent"}"#,
            },
        ),
    ])
    .expect("scripted client");
    let cspec = CspecClient::with_test_client(cspec_failure_client.clone());
    let ldh = ClinGenLdhClient::with_test_client(cspec_failure_client);

    let cspec_error = cspec.manifest("BRAF").await.expect_err("CSpec outage");
    let ldh_value = ldh.medium("CA000001").await.expect("LDH result");
    let cspec_projection = cspec_error.public_projection();
    assert_eq!(
        cspec_projection.source,
        Some(SourceProvider::CLINGEN_CSPEC.label())
    );
    assert!(!cspec_projection.message.contains("cspec-secret-body"));
    assert_eq!(
        ldh_value.get("ldh").and_then(serde_json::Value::as_str),
        Some("independent")
    );

    let ldh_failure_client = scripted_client([
        (
            CSPEC_HOST,
            ScriptedResponse::Http {
                status: StatusCode::OK,
                headers: vec![("content-type", "application/json")],
                body: r#"{"status":{"code":200},"metadata":{},"data":[]}"#,
            },
        ),
        (
            LDH_HOST,
            ScriptedResponse::TransportError("ldh-secret-body"),
        ),
    ])
    .expect("scripted client");
    let cspec = CspecClient::with_test_client(ldh_failure_client.clone());
    let ldh = ClinGenLdhClient::with_test_client(ldh_failure_client);

    let cspec_value = cspec.manifest("BRAF").await.expect("CSpec result");
    let ldh_error = ldh.medium("CA000001").await.expect_err("LDH outage");
    let ldh_projection = ldh_error.public_projection();
    assert_eq!(
        ldh_projection.source,
        Some(SourceProvider::CLINGEN_LDH.label())
    );
    assert!(!ldh_projection.message.contains("ldh-secret-body"));
    assert_eq!(
        cspec_value
            .pointer("/status/code")
            .and_then(serde_json::Value::as_i64),
        Some(200)
    );
}
