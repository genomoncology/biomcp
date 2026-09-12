//! Shared test-only helpers for decomposed trial module sidecars.

#[allow(unused_imports)]
pub(super) use super::{ClinicalTrialSearchTotal, TrialSearchFilters, TrialSource};
#[allow(unused_imports)]
pub(super) use crate::error::BioMcpError;
#[allow(unused_imports)]
pub(super) use crate::sources::clinicaltrials::ClinicalTrialsClient;
#[allow(unused_imports)]
pub(super) use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub(super) struct CtGovFixtureEnv {
    previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    _cache: tempfile::TempDir,
}

impl CtGovFixtureEnv {
    pub(super) fn set(base: &str) -> Self {
        let cache = tempfile::tempdir().expect("CTGov fixture cache");
        let mut prior = Vec::new();
        for (key, value) in [
            ("BIOMCP_CTGOV_BASE", base.to_owned()),
            ("BIOMCP_TEST_UNPACED_ORIGIN", base.to_owned()),
            (
                "BIOMCP_CACHE_DIR",
                cache.path().to_string_lossy().into_owned(),
            ),
        ] {
            prior.push((key, std::env::var_os(key)));
            // SAFETY: callers hold the serial-test process-wide environment lock.
            unsafe { std::env::set_var(key, value) };
        }
        Self {
            previous: prior,
            _cache: cache,
        }
    }
}

impl Drop for CtGovFixtureEnv {
    fn drop(&mut self) {
        for (key, previous) in self.previous.drain(..).rev() {
            // SAFETY: callers hold the serial-test process-wide environment lock.
            unsafe {
                if let Some(previous) = previous {
                    std::env::set_var(key, previous);
                } else {
                    std::env::remove_var(key);
                }
            }
        }
    }
}

pub(super) async fn ctgov_json_fixture(
    body: impl Into<String>,
) -> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let body = Arc::new(body.into());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind synthetic CTGov fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = captured.clone();
            let body = body.clone();
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let len = stream.read(&mut request).await.expect("read CTGov request");
                captured
                    .lock()
                    .expect("lock fixture requests")
                    .push(String::from_utf8_lossy(&request[..len]).into_owned());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write CTGov response");
            });
        }
    });
    (base, requests, task)
}

pub(super) fn ctgov_search_study_fixture(
    nct_id: &str,
    min_age: &str,
    max_age: &str,
) -> serde_json::Value {
    json!({
        "protocolSection": {
            "identificationModule": {
                "nctId": nct_id,
                "briefTitle": format!("Trial {nct_id}")
            },
            "statusModule": {
                "overallStatus": "RECRUITING"
            },
            "eligibilityModule": {
                "minimumAge": min_age,
                "maximumAge": max_age
            }
        }
    })
}

pub(super) fn ctgov_search_results(
    values: Vec<serde_json::Value>,
) -> Vec<biodata::ClinicalTrialsGovApiV2SearchResult> {
    let bytes = serde_json::to_vec(&json!({"studies": values})).unwrap();
    let filters = biodata::ClinicalTrialSearchFilters::new(
        biodata::ClinicalTrialSearchFilterFields {
            condition: Some("fixture".into()),
            ..Default::default()
        },
        Default::default(),
    )
    .unwrap();
    let plan = biodata::ClinicalTrialsGovApiV2SearchPlan::new(&filters, 50, None, true).unwrap();
    biodata::ClinicalTrialsGovApiV2SearchPage::parse(&plan, &bytes, &Default::default())
        .expect("valid CTGov search page")
        .results()
        .unwrap_or_default()
        .to_vec()
}

pub(super) fn age_filtered_ctgov_filters() -> TrialSearchFilters {
    TrialSearchFilters {
        condition: Some("melanoma".into()),
        status: Some("recruiting".into()),
        age: Some(51.0),
        ..Default::default()
    }
}

pub(super) fn studies_with_age_matches(
    total: usize,
    eligible: usize,
    prefix: &str,
) -> Vec<serde_json::Value> {
    (0..total)
        .map(|index| {
            let group = prefix.parse::<usize>().unwrap_or_default();
            let nct_id = format!(
                "NCT{:08}",
                group.saturating_mul(1_000).saturating_add(index)
            );
            if index < eligible {
                ctgov_search_study_fixture(&nct_id, "18 Years", "75 Years")
            } else {
                ctgov_search_study_fixture(&nct_id, "18 Years", "50 Years")
            }
        })
        .collect()
}
