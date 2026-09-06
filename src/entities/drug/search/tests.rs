//! Search-module tests split out from the legacy drug facade.

use super::super::test_support::*;
use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod fallback;
mod mechanism;
mod who;

fn provider_hits(value: serde_json::Value) -> Vec<MyChemHit> {
    serde_json::from_value(value).expect("provider-shaped MyChem hits")
}

#[test]
fn ema_identity_admits_only_hits_with_exact_allowed_fields_and_preserves_field_order() {
    let hits = provider_hits(serde_json::json!([
        {
            "openfda": {
                "generic_name": ["other", " eflornithine. "],
                "brand_name": ["Vaniqa", "Second brand"]
            },
            "ndc": [
                {"nonproprietaryname": "not it"},
                {"nonproprietaryname": "Eflornithine"}
            ],
            "drugbank": {
                "name": "Eflornithine",
                "synonyms": ["2,5-diamino-2-(difluoromethyl)pentanoic acid", "acid"]
            },
            "chembl": {"pref_name": "DFMO"},
            "gtopdb": {"name": "excluded gtopdb"},
            "unii": {"display_name": "excluded unii"},
            "chebi": {"name": "excluded chebi"}
        },
        {
            "drugbank": {"name": "irrelevant", "synonyms": ["eflornithine"]},
            "openfda": {"brand_name": "must not leak"}
        }
    ]));

    let identity = ema_identity_from_mychem_hits("  EFLORNITHINE.. ", &hits)
        .expect("first hit should resolve through an allowed exact field");
    assert_eq!(
        identity.terms_for_test(),
        vec![
            ("EFLORNITHINE", "query"),
            ("other", "openfda.generic_name"),
            ("not it", "ndc.nonproprietaryname"),
            ("DFMO", "chembl.pref_name"),
            ("Vaniqa", "openfda.brand_name"),
            ("Second brand", "openfda.brand_name"),
        ]
    );
}

#[test]
fn ema_identity_has_no_all_hits_or_excluded_field_fallback() {
    for value in [
        serde_json::json!([]),
        serde_json::json!([{"drugbank": {"name": "unrelated", "synonyms": ["query"]}}]),
        serde_json::json!([{"gtopdb": {"name": "query"}}]),
        serde_json::json!([{"unii": {"display_name": "query"}}]),
        serde_json::json!([{"chebi": {"name": "query"}}]),
    ] {
        assert!(ema_identity_from_mychem_hits("query", &provider_hits(value)).is_none());
    }
}

#[test]
fn every_allowed_ema_field_independently_admits_a_hit() {
    let cases = [
        (
            serde_json::json!({"openfda": {"generic_name": ["query"]}}),
            "openfda.generic_name",
        ),
        (
            serde_json::json!({"ndc": [
                {"nonproprietaryname": "other"},
                {"nonproprietaryname": "query"}
            ]}),
            "ndc.nonproprietaryname",
        ),
        (
            serde_json::json!({"drugbank": {"name": "query"}}),
            "drugbank.name",
        ),
        (
            serde_json::json!({"chembl": {"pref_name": "query"}}),
            "chembl.pref_name",
        ),
        (
            serde_json::json!({"openfda": {"brand_name": ["other", "query"]}}),
            "openfda.brand_name",
        ),
    ];

    for (hit, expected_source) in cases {
        assert!(
            ema_identity_from_mychem_hits("query", &provider_hits(serde_json::json!([hit])))
                .is_some(),
            "{expected_source} should independently admit its hit"
        );
    }
}

struct EmaSearchFixtureEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl EmaSearchFixtureEnv {
    fn set(&mut self, name: &'static str, value: &str) {
        self.0.push((name, std::env::var_os(name)));
        // SAFETY: the test holds serial_test's process-wide environment lock.
        unsafe { std::env::set_var(name, value) };
    }
}

impl Drop for EmaSearchFixtureEnv {
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

fn mychem_fixture_response(status: &str, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

async fn mychem_fallback_fixture_server() -> (String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>)
{
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind MyChem fallback fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(AtomicUsize::new(0));
    let observed = requests.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let observed = observed.clone();
            tokio::spawn(async move {
                let mut request = vec![0_u8; 32 * 1024];
                let len = stream
                    .read(&mut request)
                    .await
                    .expect("read fixture request");
                let request = String::from_utf8_lossy(&request[..len]);
                observed.fetch_add(1, Ordering::SeqCst);
                let response = if request.contains("q=gardasil&") {
                    mychem_fixture_response("200 OK", r#"{"malformed":"fixture"}"#)
                } else if request.contains("q=prevnar&") {
                    mychem_fixture_response("200 OK", r#"{"total":0,"hits":[]}"#)
                } else if request.contains("q=fluzone&") {
                    mychem_fixture_response(
                        "200 OK",
                        r#"{"total":1,"hits":[{"drugbank":{"name":"unrelated"}}]}"#,
                    )
                } else if request.contains("q=GARDASIL&") {
                    mychem_fixture_response(
                        "200 OK",
                        r#"{"total":1,"hits":[{"drugbank":{"synonyms":["GARDASIL"]}}]}"#,
                    )
                } else {
                    mychem_fixture_response("404 Not Found", r#"{"error":"unplanned"}"#)
                };
                stream
                    .write_all(&response)
                    .await
                    .expect("write fixture response");
            });
        }
    });
    (base, requests, task)
}

#[tokio::test]
#[serial_test::serial]
async fn unresolved_mychem_paths_each_request_once_and_preserve_positive_cvx_fallback() {
    let (base, requests, server) = mychem_fallback_fixture_server().await;
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec/fixtures");
    let local_data = crate::test_support::TempDirGuard::new("ema-search-fallback");
    let ema_root = local_data.path().join("ema");
    let cvx_root = local_data.path().join("cvx");
    let cache_root = local_data.path().join("cache");
    std::fs::create_dir_all(&ema_root).expect("create private EMA fixture root");
    std::fs::create_dir_all(&cvx_root).expect("create private CVX fixture root");
    for file in [
        "dhpcs.json",
        "medicines.json",
        "post_authorisation.json",
        "psusas.json",
        "referrals.json",
        "shortages.json",
    ] {
        std::fs::copy(
            fixture_root.join("ema-human").join(file),
            ema_root.join(file),
        )
        .unwrap_or_else(|err| panic!("copy EMA fixture {file}: {err}"));
    }
    for file in ["TRADENAME.txt", "cvx.txt", "mvx.txt"] {
        std::fs::copy(fixture_root.join("cvx").join(file), cvx_root.join(file))
            .unwrap_or_else(|err| panic!("copy CVX fixture {file}: {err}"));
    }
    let mut env = EmaSearchFixtureEnv(Vec::new());
    env.set("BIOMCP_MYCHEM_BASE", &base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
    env.set("BIOMCP_CACHE_DIR", cache_root.to_str().expect("cache path"));
    env.set("BIOMCP_EMA_DIR", ema_root.to_str().expect("EMA path"));
    env.set("BIOMCP_CVX_DIR", cvx_root.to_str().expect("CVX path"));

    for (query, expected_name, expected_source) in [
        ("gardasil", "Silgard", "cvx_full_vaccine_name"),
        ("prevnar", "Prevenar 13", "cvx_full_vaccine_name"),
        ("fluzone", "Flucelvax Tetra", "cvx_short_description"),
        ("GARDASIL", "Silgard", "cvx_full_vaccine_name"),
    ] {
        let before = requests.load(Ordering::SeqCst);
        let page = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            search_name_query_with_region(query, 10, 0, DrugRegion::Eu, WhoProductTypeFilter::Both),
        )
        .await
        .unwrap_or_else(|_| panic!("{query} fixture search was unexpectedly paced"))
        .unwrap_or_else(|err| panic!("{query} fallback search failed: {err}"));
        let after = requests.load(Ordering::SeqCst);
        assert_eq!(after - before, 1, "{query} should make one MyChem request");
        let DrugSearchPageWithRegion::Eu(page) = page else {
            panic!("explicit EU search should return an EU page");
        };
        assert!(
            page.results
                .iter()
                .any(|row| row.name == expected_name && row.source == expected_source),
            "{query} should fall back through CVX to {expected_name}"
        );
    }
    server.abort();
}

#[test]
fn complete_candidate_ranking_moves_a_later_exact_match_before_broad_rows() {
    let mut rows = vec![
        ("broad-page-one", DrugSearchMatchKind::BroadText),
        ("broad-page-two", DrugSearchMatchKind::BroadText),
        ("exact-page-two", DrugSearchMatchKind::ProductName),
        ("active-page-two", DrugSearchMatchKind::ActiveSubstance),
    ];
    rank_drug_candidates(&mut rows);
    assert_eq!(
        rows.into_iter().map(|(row, _)| row).collect::<Vec<_>>(),
        vec![
            "exact-page-two",
            "active-page-two",
            "broad-page-one",
            "broad-page-two"
        ]
    );
}
