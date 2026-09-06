//! Shared test-only helpers and re-exports for nested drug module tests.

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[allow(unused_imports)]
pub(super) use crate::entities::SearchPage;
#[allow(unused_imports)]
pub(super) use crate::error::BioMcpError;
#[allow(unused_imports)]
pub(super) use crate::sources::mychem::MyChemHit;

#[allow(unused_imports)]
pub(super) use super::{
    DrugRegion, DrugSearchFilters, DrugSearchResult, WhoPrequalificationEntry,
    WhoPrequalificationKind,
};

pub(super) fn mychem_row(name: &str) -> DrugSearchResult {
    DrugSearchResult {
        name: name.to_string(),
        drugbank_id: None,
        drug_type: None,
        mechanism: None,
        target: None,
    }
}

pub(super) fn who_row(reference: &str, inn: &str) -> WhoPrequalificationEntry {
    WhoPrequalificationEntry {
        kind: WhoPrequalificationKind::FinishedPharma,
        who_reference_number: Some(reference.to_string()),
        inn: inn.to_string(),
        presentation: Some(format!("{inn} Tablet 100mg")),
        dosage_form: Some("Tablet".to_string()),
        product_type: "Finished Pharmaceutical Product".to_string(),
        therapeutic_area: "Malaria".to_string(),
        applicant: "Example Applicant".to_string(),
        listing_basis: Some("Prequalification - Abridged".to_string()),
        alternative_listing_basis: None,
        prequalification_date: Some("2024-01-01".to_string()),
        who_product_id: None,
        grade: None,
        confirmation_document_date: None,
        vaccine_type: None,
        commercial_name: None,
        dose_count: None,
        manufacturer: None,
        responsible_nra: None,
    }
}

pub(super) fn who_api_row(product_id: &str, inn: &str) -> WhoPrequalificationEntry {
    WhoPrequalificationEntry {
        kind: WhoPrequalificationKind::Api,
        who_reference_number: None,
        inn: inn.to_string(),
        presentation: None,
        dosage_form: None,
        product_type: "Active Pharmaceutical Ingredient".to_string(),
        therapeutic_area: "Malaria".to_string(),
        applicant: "Example API Applicant".to_string(),
        listing_basis: None,
        alternative_listing_basis: None,
        prequalification_date: Some("2024-01-01".to_string()),
        who_product_id: Some(product_id.to_string()),
        grade: Some("Standard".to_string()),
        confirmation_document_date: Some("2024-02-01".to_string()),
        vaccine_type: None,
        commercial_name: None,
        dose_count: None,
        manufacturer: None,
        responsible_nra: None,
    }
}

struct RequiredLabelFixtureEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl RequiredLabelFixtureEnv {
    fn set(&mut self, name: &'static str, value: &str) {
        self.0.push((name, std::env::var_os(name)));
        // SAFETY: the test holds serial_test's process-wide environment lock.
        unsafe { std::env::set_var(name, value) };
    }
}

impl Drop for RequiredLabelFixtureEnv {
    fn drop(&mut self) {
        for (name, prior) in self.0.drain(..).rev() {
            // SAFETY: the test holds serial_test's process-wide environment lock.
            unsafe {
                if let Some(value) = prior {
                    std::env::set_var(name, value);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }
}

async fn required_label_failure_server() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind required-label fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut request = vec![0_u8; 32 * 1024];
                let len = stream
                    .read(&mut request)
                    .await
                    .expect("read fixture request");
                let request = String::from_utf8_lossy(&request[..len]);
                let (status, body) = if request.starts_with("GET /v1/query?") {
                    (
                        "200 OK",
                        r#"{"total":1,"hits":[{"_id":"fixture-drug","_score":10.0,"drugbank":{"id":"DBFIXTURE","name":"fixture-drug","synonyms":[],"drug_interactions":[]}}]}"#,
                    )
                } else if request.starts_with("GET /drug/label.json?") {
                    ("400 Bad Request", r#"{"error":"private sentinel"}"#)
                } else {
                    ("404 Not Found", r#"{"error":"unplanned"}"#)
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write fixture response");
            });
        }
    });
    (base, task)
}

#[tokio::test]
#[serial_test::serial]
async fn required_label_failures_make_zero_ddinter_ready_calls() {
    let (base, server) = required_label_failure_server().await;
    let root = crate::test_support::TempDirGuard::new("required-label-ddinter-counter");
    let missing_ddinter = root.path().join("missing-ddinter");
    let mut env = RequiredLabelFixtureEnv(Vec::new());
    env.set("BIOMCP_MYCHEM_BASE", &format!("{base}/v1"));
    env.set("BIOMCP_OPENFDA_BASE", &base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
    env.set("BIOMCP_CACHE_MODE", "off");
    env.set(
        "BIOMCP_DDINTER_DIR",
        missing_ddinter.to_str().expect("UTF-8 fixture path"),
    );

    crate::sources::ddinter::reset_ready_call_count();
    assert!(
        crate::sources::ddinter::DdinterClient::ready()
            .await
            .is_err()
    );
    assert_eq!(crate::sources::ddinter::ready_call_count(), 1);

    crate::sources::ddinter::reset_ready_call_count();
    for sections in [
        vec!["label".to_string(), "interactions".to_string()],
        vec!["all".to_string()],
    ] {
        let error = super::get("fixture-drug", &sections)
            .await
            .expect_err("required OpenFDA label failure must abort the card");
        assert!(error.to_string().contains("OpenFDA"), "{error}");
        assert_eq!(
            crate::sources::ddinter::ready_call_count(),
            0,
            "{sections:?}"
        );
    }
    server.abort();
}
