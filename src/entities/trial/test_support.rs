//! Shared test-only helpers for decomposed trial module sidecars.

#[allow(unused_imports)]
pub(super) use super::{TrialCount, TrialSearchFilters, TrialSource};
#[allow(unused_imports)]
pub(super) use crate::error::BioMcpError;
#[allow(unused_imports)]
pub(super) use crate::sources::clinicaltrials::{ClinicalTrialsClient, CtGovStudy};
#[allow(unused_imports)]
pub(super) use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub(super) struct CtGovFixtureEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl CtGovFixtureEnv {
    pub(super) fn set(base: &str) -> Self {
        let mut prior = Vec::new();
        for key in ["BIOMCP_CTGOV_BASE", "BIOMCP_TEST_UNPACED_ORIGIN"] {
            prior.push((key, std::env::var_os(key)));
            // SAFETY: callers hold the serial-test process-wide environment lock.
            unsafe { std::env::set_var(key, base) };
        }
        Self(prior)
    }
}

impl Drop for CtGovFixtureEnv {
    fn drop(&mut self) {
        for (key, previous) in self.0.drain(..).rev() {
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
    body: &'static str,
) -> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind synthetic CTGov fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = captured.clone();
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
            let nct_id = format!("NCT{prefix}{index:07}");
            if index < eligible {
                ctgov_search_study_fixture(&nct_id, "18 Years", "75 Years")
            } else {
                ctgov_search_study_fixture(&nct_id, "18 Years", "50 Years")
            }
        })
        .collect()
}
