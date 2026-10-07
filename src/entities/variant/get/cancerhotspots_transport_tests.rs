//! Real callers own transport, section policy and public channels.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub(crate) const CAPTURE: &[u8] =
    include_bytes!("../../../../testdata/sources/cancerhotspots/by_gene_braf_20260805.json");
pub(crate) const PRIVATE: &[u8] = br#"[{"tumorCount":"private-hotspots-marker"}]"#;
pub(crate) const BASES: &[&str] = &[
    "BIOMCP_MYVARIANT_BASE",
    "BIOMCP_MYGENE_BASE",
    "BIOMCP_UNIPROT_BASE",
    "BIOMCP_INTERPRO_BASE",
    "BIOMCP_CANCERHOTSPOTS_BASE",
    "BIOMCP_GNOMAD_BASE",
    "BIOMCP_CLINVAR_BASE",
    "BIOMCP_CBIOPORTAL_BASE",
    "BIOMCP_CIVIC_BASE",
    "BIOMCP_GWAS_BASE",
    "BIOMCP_TEST_UNPACED_ORIGIN",
];
pub(crate) async fn fixture(
    body: Vec<u8>,
    no_change: bool,
) -> (TestHttpFixture, Arc<Mutex<Vec<String>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        captured.lock().unwrap().push(request.to_owned());
        let target = request.lines().next().unwrap().split_whitespace().nth(1).unwrap();
        if target.starts_with("/api/hotspots/") {
            return TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &body));
        }
        let hit = if no_change {
            json!({"_id":"chr7:g.140453136A>T","dbnsfp":{"genename":"BRAF"}})
        } else {
            json!({"_id":"chr7:g.140453136A>T","dbnsfp":{"genename":"BRAF","hgvsp":"p.Val600Glu"},
                "snpeff":{"ann":{"genename":"BRAF","feature_id":"NM_004333.6","hgvs_c":"c.1802T>A","hgvs_p":"p.Val601Glu"}}})
        };
        let value = if target.starts_with("/variant/") { hit }
        else if target.starts_with("/query") && target.contains("dbnsfp") { json!({"total":1,"hits":[hit]}) }
        else if target.starts_with("/query") { json!({"total":1,"hits":[{"_id":"673","symbol":"BRAF","uniprot":{"Swiss-Prot":"P15056"}}]}) }
        else if target.starts_with("/uniprotkb/") { json!({"primaryAccession":"P15056","sequence":{"length":766},"uniProtKBCrossReferences":[{"database":"PDB","id":"1ABC","properties":[{"key":"Chains","value":"A=1-766"}]}]}) }
        else if target.starts_with("/entry/interpro/") { json!({"results":[{"metadata":{"accession":"IPR000719","name":"Protein kinase domain","type":"domain"},"proteins":[{"entry_protein_locations":[{"fragments":[{"start":457,"end":717}]}]}]}]}) }
        else if target.starts_with("/genes?") { json!([{"entrezGeneId":673}]) }
        else if target.starts_with("/studies/") && target.contains("clinical-data") { json!([{"sampleId":"sample-a","value":"Example cancer"}]) }
        else if target.starts_with("/studies/") { json!({"sequencedSampleCount":2}) }
        else if target.starts_with("/molecular-profiles/") { json!([{"sampleId":"sample-a"}]) }
        else if request.starts_with("POST /") { json!({"data":{"variant":null}}) }
        else { json!([]) };
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &serde_json::to_vec(&value).unwrap()))
    }).await;
    (fixture, requests)
}
pub(crate) fn environments(base: &str, cache: &std::path::Path) -> Vec<(&'static str, String)> {
    let mut envs: Vec<_> = BASES.iter().map(|key| (*key, base.to_owned())).collect();
    envs.extend([
        ("BIOMCP_CACHE_MODE", "off".into()),
        ("BIOMCP_CACHE_DIR", cache.display().to_string()),
        ("ONCOKB_TOKEN", "".into()),
        ("RUST_LOG", "warn,reqwest_retry=error".into()),
    ]);
    envs
}
pub(crate) fn expected(data: bool) -> Value {
    if data {
        json!({"source":"cancerhotspots.org","position_count":897,"same_aa_count":833,"matched_transcript":"ENST00000288602"})
    } else {
        json!({"source":"cancerhotspots.org","position_count":null,"same_aa_count":null,"matched_transcript":null})
    }
}
pub(crate) fn hotspot_requests(requests: &Arc<Mutex<Vec<String>>>, count: usize) {
    let requests = requests.lock().unwrap();
    let hits: Vec<_> = requests
        .iter()
        .filter(|r| r.contains("/api/hotspots/"))
        .collect();
    assert_eq!(hits.len(), count, "{requests:?}");
    for request in hits {
        assert_eq!(
            request.lines().next().unwrap(),
            "GET /api/hotspots/single/byGene/BRAF HTTP/1.1"
        );
    }
}
fn check(text: &str, json_mode: bool, state: &str) {
    assert!(!text.contains("private-hotspots-marker"), "{text}");
    if json_mode {
        let card: Value = serde_json::from_str(text).unwrap();
        assert_eq!(card["section_outcomes"]["cancerhotspots"]["outcome"], state);
        let sources = if state == "unavailable" {
            json!([])
        } else {
            json!(["cancerhotspots.org"])
        };
        assert_eq!(
            card["section_outcomes"]["cancerhotspots"]["sources"],
            sources
        );
        if state == "unavailable" {
            assert!(card.get("cancerhotspots").is_none());
        } else {
            assert_eq!(card["cancerhotspots"], expected(state == "data"));
        }
        assert_eq!(card["gene"], "BRAF");
        assert_eq!(
            card["cancer_frequencies"],
            json!([{"cancer_type":"Example cancer","frequency":1.0,"sample_count":1}])
        );
    } else {
        assert!(text.contains("BRAF"), "{text}");
        if state != "unavailable" {
            assert!(text.contains("## Cancerhotspots.org Recurrence"), "{text}");
        }
    }
}
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn hotspots_detail_reaches_native_cli_and_typed_raw_mcp() {
    for (body, state) in [
        (CAPTURE, "data"),
        (&b"[]"[..], "empty"),
        (PRIVATE, "unavailable"),
    ] {
        let (fixture, requests) = fixture(body.to_vec(), false).await;
        let cache = tempfile::tempdir().unwrap();
        let envs = environments(&fixture.base, cache.path());
        let mut env = TestEnv::new();
        for (key, value) in &envs {
            env.set(key, value);
        }
        let native = super::get("BRAF V600E", &["all".into()]).await.unwrap();
        if let Some(section) = &native.cancerhotspots {
            let _: &biodata::CancerHotspotRecurrenceProjection = section.recurrence();
        }
        check(&serde_json::to_string(&native).unwrap(), true, state);
        check(
            &crate::render::markdown::variant_markdown(&native, &["all".into()]).unwrap(),
            false,
            state,
        );
        let harness = ContractHarness::new(
            std::env::var_os("BIOMCP_BIN").unwrap(),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        );
        let client = harness.spawn_stdio_client(&envs).await.unwrap();
        for json_mode in [true, false] {
            let mut args = vec!["get", "variant", "BRAF V600E", "all"];
            if json_mode {
                args.push("--json");
            }
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(args)
                .envs(envs.iter().cloned())
                .output()
                .await
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            assert!(!String::from_utf8_lossy(&output.stderr).contains("private-hotspots-marker"));
            check(
                std::str::from_utf8(&output.stdout).unwrap(),
                json_mode,
                state,
            );
            for (tool, args) in [
                (
                    "get",
                    json!({"entity":"variant","id":"BRAF V600E","sections":["all"],"json":json_mode}),
                ),
                (
                    "biomcp",
                    json!({"command":"biomcp get variant 'BRAF V600E' all","json":json_mode}),
                ),
            ] {
                let result = client
                    .peer()
                    .call_tool(
                        CallToolRequestParams::new(tool)
                            .with_arguments(args.as_object().unwrap().clone()),
                    )
                    .await
                    .unwrap();
                assert!(!result.is_error.unwrap_or_default(), "{result:?}");
                check(first_text(&result.content), json_mode, state);
            }
        }
        client.cancel().await.unwrap();
        hotspot_requests(&requests, 7);
    }
}
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn hotspots_detail_policy_and_private_client_errors() {
    let (fixture, requests) = fixture(PRIVATE.to_vec(), false).await;
    let cache = tempfile::tempdir().unwrap();
    let mut env = TestEnv::new();
    for (key, value) in environments(&fixture.base, cache.path()) {
        env.set(key, value);
    }
    for (id, sections, state) in [
        ("BRAF V600E", vec![], "not_requested"),
        ("chr7:g.140453136A>T", vec!["all".into()], "inapplicable"),
    ] {
        let variant = super::get(id, &sections).await.unwrap();
        assert!(variant.cancerhotspots.is_none());
        assert_eq!(
            serde_json::to_value(&variant).unwrap()["section_outcomes"]["cancerhotspots"]["outcome"],
            state
        );
    }
    hotspot_requests(&requests, 0);
    let error = crate::sources::cancerhotspots::CancerHotspotsClient::new()
        .unwrap()
        .by_gene("BRAF")
        .await
        .unwrap_err();
    for text in [
        format!("{error}"),
        format!("{error:?}"),
        crate::render::json::to_error_json(&error).unwrap(),
    ] {
        assert!(!text.contains("private-hotspots-marker"), "{text}");
    }
    hotspot_requests(&requests, 1);
}

#[test]
fn hotspots_flattened_detail_restores_required_source_and_literal_target() {
    let base = json!({"id":"fixture","gene":"BRAF"});
    for target in [
        json!({"source":"custom source","position_count":897,"same_aa_count":833,"matched_transcript":"  "}),
        expected(false),
        Value::Null,
    ] {
        let mut card = base.clone();
        card["cancerhotspots"] = target.clone();
        let restored: crate::entities::variant::Variant = serde_json::from_value(card).unwrap();
        let encoded = serde_json::to_value(&restored).unwrap();
        if target.is_null() {
            assert!(encoded.get("cancerhotspots").is_none());
        } else {
            assert_eq!(encoded["cancerhotspots"], target);
            crate::render::markdown::variant_markdown(&restored, &["all".into()]).unwrap();
        }
    }
    let restored: crate::entities::variant::Variant = serde_json::from_value(base.clone()).unwrap();
    assert!(restored.cancerhotspots.is_none());
    let mut card = base;
    card["cancerhotspots"] = json!({"position_count":1});
    assert!(serde_json::from_value::<crate::entities::variant::Variant>(card).is_err());
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn hotspots_detail_timeout_preserves_other_enrichment() {
    let (fixture, _) = fixture(CAPTURE.to_vec(), false).await;
    let (release, receiver) = std::sync::mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let contacted = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let calls = contacted.clone();
    let hung = TestHttpFixture::spawn(move |request| {
        assert_eq!(
            request.lines().next().unwrap(),
            "GET /api/hotspots/single/byGene/BRAF HTTP/1.1"
        );
        calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        TestHttpReply::Hold(receiver.clone())
    })
    .await;
    let cache = tempfile::tempdir().unwrap();
    let mut env = TestEnv::new();
    for (key, value) in environments(&fixture.base, cache.path()) {
        env.set(key, value);
    }
    env.set("BIOMCP_CANCERHOTSPOTS_BASE", &hung.base);
    let started = std::time::Instant::now();
    let native = super::get("BRAF V600E", &["all".into()]).await.unwrap();
    drop(release);
    assert_eq!(contacted.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(started.elapsed() >= std::time::Duration::from_secs(8));
    assert!(started.elapsed() < std::time::Duration::from_secs(16));
    assert!(native.cancerhotspots.is_none());
    let card = serde_json::to_value(&native).unwrap();
    assert_eq!(
        card["section_outcomes"]["cancerhotspots"],
        json!({"outcome":"unavailable","sources":[],"message":"Requested variant source data is temporarily unavailable."})
    );
    assert_eq!(card["gene"], "BRAF");
    assert_eq!(native.hgvs_p.as_deref(), Some("p.Val601Glu"));
    assert_eq!(native.cancer_frequencies.len(), 1);
}
