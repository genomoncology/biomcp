//! Keep-path tests for `verify_detail_filters`: a failing detail fetch,
//! missing eligibility text, and a study with no NCT ID each keep the
//! study and land in the verification report. The verified path stays
//! out of it.

use super::*;
use crate::entities::trial::search::DetailVerificationReport;

async fn detail_fixture(
    fail_ids: &[&str],
    empty_criteria_ids: &[&str],
    criteria_text: &str,
) -> (String, tokio::task::JoinHandle<()>) {
    use axum::{Router, http::StatusCode};
    let fail: Vec<String> = fail_ids.iter().map(|v| v.to_string()).collect();
    let empty: Vec<String> = empty_criteria_ids.iter().map(|v| v.to_string()).collect();
    let criteria = criteria_text.to_string();
    let router = Router::new().fallback(|uri: axum::http::Uri| async move {
        let path = uri.path().to_string();
        let id = path.rsplit('/').next().unwrap_or_default().to_string();
        if fail.contains(&id) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "fixture unavailable".to_string(),
            );
        }
        let module = if empty.contains(&id) {
            serde_json::json!({})
        } else {
            serde_json::json!({"eligibilityCriteria": criteria})
        };
        let body = serde_json::json!({
            "protocolSection": {
                "identificationModule": {"nctId": id, "briefTitle": "Fixture"},
                "statusModule": {"overallStatus": "RECRUITING"},
                "eligibilityModule": module,
            }
        })
        .to_string();
        (StatusCode::OK, body)
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind detail fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve detail fixture");
    });
    (base, server)
}

fn study(nct_id: &str) -> biodata::ClinicalTrialsGovApiV2SearchResult {
    let bytes = serde_json::json!({"studies": [{
        "protocolSection": {
            "identificationModule": {"nctId": nct_id, "briefTitle": "Fixture"},
            "statusModule": {"overallStatus": "RECRUITING"}
        }
    }]})
    .to_string();
    let filters = biodata::ClinicalTrialSearchFilters::new(
        biodata::ClinicalTrialSearchFilterFields {
            condition: Some("fixture".into()),
            ..Default::default()
        },
        Default::default(),
    )
    .expect("filters");
    let plan = biodata::ClinicalTrialsGovApiV2SearchPlan::new(&filters, 1, None, true)
        .expect("search plan");
    biodata::ClinicalTrialsGovApiV2SearchPage::parse(&plan, bytes.as_bytes(), &Default::default())
        .expect("search page")
        .results()
        .expect("results")[0]
        .clone()
}

async fn verify_against_fixture(
    fail_ids: &[&str],
    empty_criteria_ids: &[&str],
    criteria_text: &str,
    studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
) -> (
    Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
    DetailVerificationReport,
) {
    let (base, server) = detail_fixture(fail_ids, empty_criteria_ids, criteria_text).await;
    let _env = crate::entities::trial::test_support::CtGovFixtureEnv::set(&base);
    let client = ClinicalTrialsClient::new().expect("CTGov fixture client");
    let keywords = vec!["MSI-H".to_string()];
    let outcome = verify_detail_filters(&client, studies, None, &keywords).await;
    server.abort();
    outcome
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_failed_detail_fetch_keeps_the_study_and_reports_it() {
    let (kept, report) =
        verify_against_fixture(&["NCT00000001"], &[], "", vec![study("NCT00000001")]).await;
    assert_eq!(kept.len(), 1, "the study is kept when its fetch fails");
    assert_eq!(report.unverified_kept, 1);
    assert!(report.unverified_ids.contains(&"NCT00000001".to_string()));
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn missing_eligibility_text_keeps_the_study_and_reports_it() {
    let (kept, report) =
        verify_against_fixture(&[], &["NCT00000002"], "", vec![study("NCT00000002")]).await;
    assert_eq!(
        kept.len(),
        1,
        "the study is kept when its criteria text is missing"
    );
    assert_eq!(report.unverified_kept, 1);
    assert!(report.unverified_ids.contains(&"NCT00000002".to_string()));
}

#[test]
fn a_study_without_an_nct_id_fails_the_biodata_search_boundary() {
    let bytes = serde_json::json!({"studies": [{
        "protocolSection": {
            "identificationModule": {"briefTitle": "Fixture"},
            "statusModule": {"overallStatus": "RECRUITING"}
        }
    }]})
    .to_string();
    let filters = biodata::ClinicalTrialSearchFilters::new(
        biodata::ClinicalTrialSearchFilterFields {
            condition: Some("fixture".into()),
            ..Default::default()
        },
        Default::default(),
    )
    .expect("filters");
    let plan = biodata::ClinicalTrialsGovApiV2SearchPlan::new(&filters, 1, None, true)
        .expect("search plan");
    assert!(
        biodata::ClinicalTrialsGovApiV2SearchPage::parse(
            &plan,
            bytes.as_bytes(),
            &Default::default()
        )
        .is_err()
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_verified_keyword_match_stays_out_of_the_report() {
    let (kept, report) = verify_against_fixture(
        &[],
        &[],
        "Inclusion Criteria: patients with MSI-H tumors",
        vec![study("NCT00000003"), study("NCT00000004")],
    )
    .await;
    assert_eq!(kept.len(), 2, "both studies stay when the criteria match");
    assert_eq!(report.unverified_kept, 0);
    assert!(report.unverified_ids.is_empty());
}
