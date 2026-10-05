//! Independent MIT lookup policy and real caller contracts. No provider qualification.
use super::super::genomic_lookup::{LookupRoute, genomic_lookup};
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use biodata::{HgvsEdit, HgvsLocation, HgvsMarker, HgvsMolecule};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::{ServiceExt, model::CallToolRequestParams};
use serde_json::{Value, json};
use std::{
    process::Stdio,
    sync::{Arc, Mutex},
};
use tokio::io::AsyncReadExt;

fn oracle() -> Value {
    serde_json::from_str(include_str!("genomic_lookup_oracles.json")).unwrap()
}
fn facts(p: &biodata::ParsedHgvsNucleotide) -> Value {
    assert_eq!(p.molecule(), HgvsMolecule::Genomic);
    assert!(!p.is_predicted());
    let location = p.location().unwrap();
    let kind = match location {
        HgvsLocation::Point(_) => "Point",
        HgvsLocation::Range(..) => "Range",
        HgvsLocation::InsertionFlanks(..) => "InsertionFlanks",
        _ => panic!("unexpected location"),
    };
    let digits: Vec<_> = location
        .positions()
        .iter()
        .map(|p| {
            assert_eq!(p.marker(), HgvsMarker::Ordinary);
            assert!(p.offset().is_none());
            p.digits().unwrap()
        })
        .collect();
    let edit = match p.edit().unwrap() {
        HgvsEdit::Substitution {
            reference,
            alternate,
        } => {
            json!({"kind":"Substitution","reference":reference.to_string(),"alternate":alternate.to_string()})
        }
        HgvsEdit::Deletion { deleted } => json!({"kind":"Deletion","deleted":deleted}),
        HgvsEdit::Duplication { duplicated } => {
            json!({"kind":"Duplication","duplicated":duplicated})
        }
        HgvsEdit::Insertion { inserted } => json!({"kind":"Insertion","inserted":inserted}),
        HgvsEdit::Delins { inserted } => json!({"kind":"Delins","inserted":inserted}),
        HgvsEdit::Inversion => json!({"kind":"Inversion"}),
        _ => panic!("unexpected edit"),
    };
    json!({"reference":p.reference(),"location":kind,"digits":digits,"edit":edit})
}
fn coordinate_view(c: Option<NormalizedGenomicCoordinate>) -> Value {
    c.map(|c| json!({"id":c.id,"genome_build":c.genome_build,"requires_comparison":c.requires_comparison})).unwrap_or(Value::Null)
}
fn adapter_contract() {
    let all = oracle();
    for row in all["coordinates"].as_array().unwrap() {
        let input = row["input"].as_str().unwrap();
        let a = genomic_lookup(input, LookupRoute::Coordinate)
            .unwrap()
            .unwrap();
        assert_eq!(a.original, input);
        assert_eq!(a.candidate, row["candidate"]);
        assert_eq!(json!(a.build), row["coordinate"]["genome_build"]);
        assert_eq!(a.envelope.source(), a.candidate);
        assert_eq!(a.envelope.disposition().code(), row["code"]);
        let p = a.envelope.disposition().parsed().unwrap();
        assert_eq!(facts(p), row["facts"], "{input}");
        assert_eq!(p.span().slice(&a.candidate), Some(a.candidate.as_str()));
        assert_eq!(p.render_constructed(), a.candidate);
        assert_eq!(
            p.reference_span().unwrap().slice(&a.candidate),
            p.reference()
        );
        assert_eq!(
            coordinate_view(a.coordinate().unwrap()),
            row["coordinate"],
            "{input}"
        );
        assert_eq!(
            coordinate_view(normalize_genomic_coordinate(input).unwrap()),
            row["coordinate"]
        );
    }
    for row in all["direct"].as_array().unwrap() {
        let input = row["input"].as_str().unwrap();
        let a = genomic_lookup(input, LookupRoute::Direct).unwrap().unwrap();
        assert_eq!(a.original, input);
        assert_eq!(a.candidate, input.trim());
        assert_eq!(a.envelope.source(), input.trim());
        assert_eq!(a.envelope.disposition().code(), row["code"], "{input}");
        assert_eq!(
            a.compatibility_code(),
            if row["facts"].is_null() {
                row["code"].as_str()
            } else {
                None
            }
        );
        let observed = a
            .envelope
            .disposition()
            .parsed()
            .map(facts)
            .unwrap_or(Value::Null);
        assert_eq!(observed, row["facts"], "{input}");
        assert!(
            matches!(classify_variant_input(input), VariantInputKind::Exact(VariantIdFormat::HgvsGenomic(id)) if id == input.trim())
        );
        assert!(
            matches!(parse_variant_id(input).unwrap(), VariantIdFormat::HgvsGenomic(id) if id == input.trim())
        );
        if input != "chr1:g.0C>T" && input != "chr1:g.١C>T" {
            assert!(normalize_genomic_coordinate(input).unwrap().is_none());
        }
    }
    for row in all["errors"].as_array().unwrap() {
        assert_eq!(
            normalize_genomic_coordinate(row["input"].as_str().unwrap())
                .unwrap_err()
                .to_string(),
            row["error"]
        );
    }
    for input in all["refused"].as_array().unwrap() {
        let input = input.as_str().unwrap();
        assert!(
            genomic_lookup(input, LookupRoute::Direct)
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            classify_variant_input(input),
            VariantInputKind::Unsupported
        ));
    }
    let mut a = genomic_lookup("chr1:g.19C>T", LookupRoute::Coordinate)
        .unwrap()
        .unwrap();
    a.envelope = biodata::parse_hgvs_nucleotide_21_1_4("chr1:g.20C>T");
    assert!(matches!(
        a.coordinate(),
        Err(BioMcpError::InternalProcessing)
    ));
    let mut a = genomic_lookup("chr2:g.17_19del", LookupRoute::Direct)
        .unwrap()
        .unwrap();
    a.envelope = biodata::parse_hgvs_nucleotide_21_1_4("chr2:g.18_19del");
    assert!(matches!(a.exact_id(), Err(BioMcpError::InternalProcessing)));
    for (bytes, code) in [
        (1024 * 1024, "hgvs_parsed"),
        (1024 * 1024 + 1, "hgvs_input_limit"),
    ] {
        let input = format!("chr2:g.{}17_19del", "0".repeat(bytes - 15));
        assert_eq!(input.len(), bytes);
        let a = genomic_lookup(&input, LookupRoute::Direct)
            .unwrap()
            .unwrap();
        assert_eq!(a.envelope.disposition().code(), code);
        assert!(a.exact_id().unwrap().is_some());
        let debug = format!("{a:?}");
        assert!(!debug.contains("chr2") && !debug.contains(&input));
        let error = crate::sources::myvariant::MyVariantClient::get_plan(
            a.exact_id().unwrap().unwrap(),
            None,
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Invalid argument: Variant ID is too long."
        );
    }
}
fn decoded_requests(requests: &Arc<Mutex<Vec<String>>>) -> Value {
    json!(requests.lock().unwrap().iter().map(|request| {
        let words: Vec<_> = request.lines().next().unwrap().split_whitespace().collect();
        assert_eq!(words.len(), 3);
        assert_eq!(words[2], "HTTP/1.1");
        let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
        json!({"method":words[0],"path":url.path(),"query_pairs":url.query_pairs().into_owned().collect::<Vec<_>>(),"body":request.split_once("\r\n\r\n").unwrap().1})
    }).collect::<Vec<_>>())
}
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn genomic_lookup_contract_and_caller_table() {
    adapter_contract();
    let all = oracle();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/debug/biomcp"));
    let harness = ContractHarness::new(binary, root.clone());
    let refusals = Arc::new(Mutex::new(Vec::new()));
    let captured = refusals.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        captured.lock().unwrap().push(request.to_owned());
        TestHttpReply::Bytes(test_http_response(
            "404 Not Found",
            "application/json",
            b"{}",
        ))
    })
    .await;
    {
        let cache = tempfile::tempdir().unwrap();
        let mut guard = TestEnv::new();
        guard.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
        guard.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        guard.set("BIOMCP_CACHE_MODE", "off");
        guard.set("BIOMCP_CACHE_DIR", cache.path());
        for input in all["refused"].as_array().unwrap() {
            assert!(matches!(
                crate::entities::variant::get(input.as_str().unwrap(), &[]).await,
                Err(BioMcpError::InvalidArgument(_))
            ));
        }
        for row in all["errors"].as_array().unwrap() {
            assert_eq!(
                crate::entities::variant::get(row["input"].as_str().unwrap(), &[])
                    .await
                    .unwrap_err()
                    .to_string(),
                row["error"]
            );
        }
        assert!(refusals.lock().unwrap().is_empty());
    }
    let hit =
        std::fs::read(root.join(all["provenance"]["retained_hit_file"].as_str().unwrap())).unwrap();
    for source_row in all["transports"].as_array().unwrap() {
        let mut row = source_row.clone();
        if let Some(file) = row["payload_file"].as_str() {
            let mut payload: Value =
                serde_json::from_slice(&std::fs::read(root.join(file)).unwrap()).unwrap();
            for field in row["omit_payload_fields"].as_array().unwrap() {
                payload
                    .as_object_mut()
                    .unwrap()
                    .remove(field.as_str().unwrap());
            }
            payload
                .as_object_mut()
                .unwrap()
                .extend(row["payload_overrides"].as_object().unwrap().clone());
            row["payload"] = payload;
        }
        for request in row["requests"].as_array_mut().unwrap() {
            let file = request["query_pairs"][0][1]["literal_file"]
                .as_str()
                .unwrap();
            request["query_pairs"][0][1] =
                json!(std::fs::read_to_string(root.join(file)).unwrap().trim_end());
        }
        let replies = row["replies"].as_array().unwrap().clone();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let hit = hit.clone();
        let fixture = TestHttpFixture::spawn(move |request| {
            let mut requests = captured.lock().unwrap();
            let index = requests.len();
            requests.push(request.to_owned());
            let reply = replies
                .get(index)
                .expect("unexpected request")
                .as_str()
                .unwrap();
            let (status, body) = match reply {
                "hit" => ("200 OK", hit.clone()),
                "distinct" => {
                    let mut v: Value = serde_json::from_slice(&hit).unwrap();
                    v["dbsnp"] = json!({"rsid":"rs1"});
                    ("200 OK", serde_json::to_vec(&v).unwrap())
                }
                "404" => ("404 Not Found", b"{}".to_vec()),
                _ => panic!("unknown reply"),
            };
            TestHttpReply::Bytes(test_http_response(status, "application/json", &body))
        })
        .await;
        let cache = tempfile::tempdir().unwrap();
        let env = [
            ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
            ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
            ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
            ("BIOMCP_CACHE_MODE", "off".into()),
            ("BIOMCP_DEFAULT_ASSEMBLY", "grch38".into()),
            ("RUST_LOG", "off,reqwest_retry=error".into()),
        ];
        if row["channel"] == "get" {
            let mut guard = TestEnv::new();
            for (key, value) in &env {
                guard.set(key, value);
            }
            let v = crate::entities::variant::get(row["input"].as_str().unwrap(), &[])
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_value(v).unwrap(),
                row["payload"],
                "{}",
                row["name"]
            );
        } else if row["channel"] == "cli" {
            let args: Vec<_> = row["argv"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();
            let out = tokio::process::Command::new(&harness.biomcp_bin)
                .args(args)
                .envs(env.clone())
                .env_remove("NCI_API_KEY")
                .output()
                .await
                .unwrap();
            assert_eq!(
                out.status.code(),
                Some(row["exit"].as_i64().unwrap() as i32)
            );
            assert_eq!(String::from_utf8(out.stderr).unwrap(), row["stderr"]);
            assert_eq!(
                serde_json::from_slice::<Value>(&out.stdout).unwrap(),
                row["payload"]
            );
        } else {
            let mut child = tokio::process::Command::new(&harness.biomcp_bin)
                .arg("serve")
                .current_dir(&harness.repo_root)
                .envs(env.clone())
                .env_remove("NCI_API_KEY")
                .env("UMLS_API_KEY", "")
                .env_remove("RUST_MIN_STACK")
                .env_remove("BIOMCP_TEST_PANIC_TOOL")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true)
                .spawn()
                .unwrap();
            let mut stderr = child.stderr.take().unwrap();
            let stderr = tokio::spawn(async move {
                let mut s = String::new();
                stderr.read_to_string(&mut s).await.unwrap();
                s
            });
            let client =
                ().serve((child.stdout.take().unwrap(), child.stdin.take().unwrap()))
                    .await
                    .unwrap();
            let result = client
                .peer()
                .call_tool(
                    CallToolRequestParams::new(row["tool"].as_str().unwrap().to_owned())
                        .with_arguments(row["arguments"].as_object().unwrap().clone()),
                )
                .await
                .unwrap();
            let payload = first_text(&result.content);
            assert_eq!(payload, row["payload"].as_str().unwrap());
            assert_eq!(serde_json::to_value(result).unwrap(), row["wrapper"]);
            client.cancel().await.unwrap();
            let status = tokio::time::timeout(std::time::Duration::from_secs(5), child.wait())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(status.code(), Some(row["exit"].as_i64().unwrap() as i32));
            assert_eq!(stderr.await.unwrap(), row["stderr"]);
        }
        assert_eq!(
            decoded_requests(&requests),
            row["requests"],
            "{}",
            row["name"]
        );
    }
}
