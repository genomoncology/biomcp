//! Positively identified transport failures preserve absence policy.
use super::*;
use crate::error::{SourceContext, SourceProvider};

async fn transport(kind: &str) -> BioMcpError {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let error = if kind == "connection" {
        drop(listener);
        reqwest::Client::new().get(url).send().await.unwrap_err()
    } else {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(20))
            .build()
            .unwrap();
        let future = client.get(url).send();
        let task = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            std::future::pending::<()>().await;
        });
        let error = future.await.unwrap_err();
        task.abort();
        error
    };
    assert_eq!(error.is_connect(), kind == "connection");
    assert_eq!(error.is_timeout(), kind == "timeout");
    assert!(!error.is_body() && !error.is_decode());
    assert!(error.status().is_none());
    BioMcpError::Http(error).with_source_context(SourceContext::retry(SourceProvider::MYCHEM))
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn typed_absence_and_uncached_retry_table() {
    for case in table(4)
        .into_iter()
        .filter(|case| case["input"]["operation"] == "trial_alias_resolution_from_lookup_result")
    {
        let alternative = case["alternative"].as_str().unwrap();
        let name = case["input"]["requested_name"]
            .as_str()
            .unwrap_or("sampledrug");
        let error = match alternative {
            "notfound" => BioMcpError::NotFound {
                entity: "drug".into(),
                id: name.into(),
                suggestion: "search drug".into(),
            },
            "connection" | "timeout" => transport(alternative).await,
            "generic-api" => BioMcpError::Api {
                api: "MyChem.info".into(),
                message: "SOURCE_SENTINEL_0224".into(),
            }
            .with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::MYCHEM,
            )),
            "generic-unavailable" => BioMcpError::SourceUnavailable {
                source_name: "MyChem.info".into(),
                reason: "SOURCE_SENTINEL_0224".into(),
                suggestion: "Retry the remote source.".into(),
            }
            .with_source_context(crate::error::SourceContext::new(
                crate::error::SourceProvider::MYCHEM,
                crate::error::RecoveryAction::ReviewSourceConfiguration,
            )),
            _ => panic!("unknown alternative"),
        };
        let result = trial_alias_resolution_from_lookup_result(name, Err(error));
        if case["expected"]["failure"].is_null() {
            let (resolution, cacheable) = result.unwrap();
            assert_eq!(cacheable, case["expected"]["cacheable"].as_bool().unwrap());
            assert_eq!(
                resolution.canonical_name,
                case["expected"]["canonical_name"]
            );
            assert_eq!(json!(resolution.aliases.iter().map(|alias|json!({"label":alias.label,"source":format!("{:?}",alias.source)})).collect::<Vec<_>>()),case["expected"]["aliases"]);
        } else {
            failure(
                result.err().unwrap(),
                &case["expected"]["failure"],
                &case["id"],
            );
        }
    }
    trial_alias_cache().lock().unwrap().clear();
    let initial = resolve_trial_alias_resolution_with_lookup("sampledrug", async {
        Err(transport("connection").await)
    })
    .await
    .unwrap();
    assert_eq!(initial.aliases[0].label, "sampledrug");
    assert!(trial_alias_cache().lock().unwrap().is_empty());
    let response = crate::sources::mychem::projection::decode(
        &std::fs::read(root().join("inputs/surface.json")).unwrap(),
        biodata::MyChemProfile::Get,
    )
    .unwrap();
    let selected = crate::transform::drug::select_hits_for_name(&response.hits, "sampledrug");
    let drug = crate::transform::drug::merge_mychem_hits(&selected, "sampledrug");
    let candidates = trial_alias_candidates_from_hits(&selected);
    let accepted = resolve_trial_alias_resolution_with_lookup("sampledrug", async {
        Ok(TrialAliasLookup {
            canonical_name: drug.name,
            candidates,
        })
    })
    .await
    .unwrap();
    assert_eq!(accepted.canonical_name, "sampledrug");
    assert_eq!(trial_alias_cache().lock().unwrap().len(), 1);
    let cached = resolve_trial_alias_resolution_with_lookup("SAMPLEDRUG", async {
        panic!("cached lookup future polled")
    })
    .await
    .unwrap();
    assert_eq!(cached.aliases[0].label, "SAMPLEDRUG");
    trial_alias_cache().lock().unwrap().clear();
}
