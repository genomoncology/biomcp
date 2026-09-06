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

#[cfg(test)]
mod reference_wire_tests {
    use super::super::*;
    use biodata::ExtensibleCode;

    #[test]
    fn trial_stores_shared_references_directly() {
        let field: fn(&Trial) -> &Option<Vec<ClinicalTrialReference>> = |trial| &trial.references;
        let _ = field;
    }

    fn shared_reference(
        pmid: Option<&str>,
        citation: Option<&str>,
        source_type: Option<ExtensibleCode>,
    ) -> ClinicalTrialReference {
        ClinicalTrialReference::new(
            pmid.map(str::to_owned),
            citation.map(str::to_owned),
            source_type,
        )
        .expect("shared reference")
    }

    fn trial_wire(references: Option<serde_json::Value>) -> serde_json::Value {
        let mut value = serde_json::json!({
            "nct_id": "NCT00000001",
            "title": "Reference test",
            "status": "RECRUITING",
            "conditions": [],
            "interventions": []
        });
        if let Some(references) = references {
            value["references"] = references;
        }
        value
    }

    #[test]
    fn trial_reference_wire_preserves_section_states_order_and_unicode() {
        let missing: Trial = serde_json::from_value(trial_wire(None)).expect("missing section");
        assert!(missing.references.is_none());
        assert!(
            serde_json::to_value(missing)
                .expect("missing serialization")
                .get("references")
                .is_none()
        );

        let null: Trial = serde_json::from_value(trial_wire(Some(serde_json::Value::Null)))
            .expect("null section");
        assert!(null.references.is_none());

        let empty: Trial =
            serde_json::from_value(trial_wire(Some(serde_json::json!([])))).expect("empty section");
        assert!(empty.references.as_ref().is_some_and(Vec::is_empty));
        assert_eq!(
            serde_json::to_value(empty).expect("empty serialization")["references"],
            serde_json::json!([])
        );

        let populated = serde_json::json!([
            {"pmid":" 123 ", "citation":" Étude α. ", "reference_type":" DERIVED "},
            {"citation":"研究 β", "pmid":" \t ", "reference_type":null}
        ]);
        let trial: Trial =
            serde_json::from_value(trial_wire(Some(populated))).expect("populated section");
        let references = trial.references.as_ref().expect("shared values");
        assert_eq!(references[0].pmid(), Some("123"));
        assert_eq!(references[0].citation(), Some("Étude α."));
        assert_eq!(references[1].citation(), Some("研究 β"));
        assert_eq!(
            serde_json::to_value(trial).expect("populated serialization")["references"],
            serde_json::json!([
                {"pmid":"123", "citation":"Étude α.", "reference_type":"DERIVED"},
                {"citation":"研究 β"}
            ])
        );
    }

    #[test]
    fn trial_reference_wire_rejects_unusable_citations() {
        for citation in [
            None,
            Some(serde_json::Value::Null),
            Some(serde_json::json!("")),
            Some(serde_json::json!(" \t ")),
        ] {
            let mut reference = serde_json::json!({});
            if let Some(citation) = citation {
                reference["citation"] = citation;
            }
            assert!(
                serde_json::from_value::<Trial>(trial_wire(Some(serde_json::json!([reference]))))
                    .is_err()
            );
        }

        let mut trial: Trial = serde_json::from_value(trial_wire(None)).unwrap();
        trial.references = Some(vec![shared_reference(None, Some(" \t "), None)]);
        assert!(serde_json::to_value(trial).is_err());
    }

    #[test]
    fn trial_reference_wire_normalizes_direct_shared_optional_whitespace() {
        let mut trial: Trial = serde_json::from_value(trial_wire(None)).unwrap();
        trial.references = Some(vec![shared_reference(
            Some(" \t "),
            Some(" Citation "),
            Some(
                ExtensibleCode::new(
                    "clinicaltrials.gov",
                    " \t ",
                    None::<String>,
                    None::<String>,
                    None::<String>,
                )
                .unwrap(),
            ),
        )]);
        assert_eq!(
            serde_json::to_value(trial).unwrap()["references"],
            serde_json::json!([{"citation":"Citation"}])
        );
    }

    #[test]
    fn trial_reference_wire_rejects_shared_information_it_cannot_emit() {
        let invalid = [
            shared_reference(None, None, None),
            shared_reference(
                None,
                Some("Citation"),
                Some(
                    ExtensibleCode::new(
                        "nci.nih.gov",
                        " ",
                        None::<String>,
                        None::<String>,
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ),
            shared_reference(
                None,
                Some("Citation"),
                Some(
                    ExtensibleCode::new(
                        "clinicaltrials.gov",
                        " ",
                        Some("Display"),
                        None::<String>,
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ),
            shared_reference(
                None,
                Some("Citation"),
                Some(
                    ExtensibleCode::new(
                        "clinicaltrials.gov",
                        "BACKGROUND",
                        Some("Display"),
                        None::<String>,
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ),
            shared_reference(
                None,
                Some("Citation"),
                Some(
                    ExtensibleCode::new(
                        "clinicaltrials.gov",
                        "BACKGROUND",
                        None::<String>,
                        Some("v1"),
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ),
            shared_reference(
                None,
                Some("Citation"),
                Some(
                    ExtensibleCode::new(
                        "clinicaltrials.gov",
                        "BACKGROUND",
                        None::<String>,
                        None::<String>,
                        Some("meaning"),
                    )
                    .unwrap(),
                ),
            ),
        ];
        for reference in invalid {
            let mut trial: Trial = serde_json::from_value(trial_wire(None)).unwrap();
            trial.references = Some(vec![reference]);
            let error = serde_json::to_value(trial).expect_err("unsupported shared value");
            assert!(error.to_string().contains("Invalid trial reference data"));
            assert!(!error.to_string().contains("nci.nih.gov"));
        }
    }
}
