//! Synthetic contracts exercise the callable get boundary and complete request ledger.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::entities::variant::{self, GenomeBuild, VariantIdFormat};
use crate::error::BioMcpError;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub(super) fn oracle() -> Value {
    serde_json::from_str(include_str!("protein_lookup_oracles.json")).unwrap()
}
pub(super) fn ledger(requests: &Arc<Mutex<Vec<String>>>, query: &str, offsets: &[usize]) {
    let actual = requests.lock().unwrap();
    assert_eq!(actual.len(), offsets.len(), "{actual:?}");
    for (request, offset) in actual.iter().zip(offsets) {
        let words: Vec<_> = request.lines().next().unwrap().split_whitespace().collect();
        assert_eq!(words[0], "GET");
        assert_eq!(words[2], "HTTP/1.1");
        let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
        assert_eq!(url.path(), "/query");
        assert_eq!(
            url.query_pairs().into_owned().collect::<Vec<_>>(),
            vec![
                ("q".into(), query.into()),
                ("size".into(), "50".into()),
                ("from".into(), offset.to_string()),
                ("fields".into(), oracle()["fields"].as_str().unwrap().into())
            ]
        );
        assert_eq!(request.split_once("\r\n\r\n").unwrap().1, "");
    }
}
pub(super) async fn fixture(pages: Vec<Value>) -> (TestHttpFixture, Arc<Mutex<Vec<String>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        let mut log = captured.lock().unwrap();
        let index = log.len();
        log.push(request.into());
        let page = &pages[index.min(pages.len() - 1)];
        let (status, raw) = if page == "malformed" {
            ("200 OK", b"{".to_vec())
        } else if page == "failure" {
            ("400 Bad Request", b"{}".to_vec())
        } else {
            ("200 OK", serde_json::to_vec(page).unwrap())
        };
        TestHttpReply::Bytes(test_http_response(status, "application/json", &raw))
    })
    .await;
    (fixture, requests)
}
async fn exercise(input: &str, pages: Vec<Value>, expected: &str, offsets: &[usize], row: &Value) {
    let (fixture, requests) = fixture(pages).await;
    let cache = tempfile::tempdir().unwrap();
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_MODE", "off");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let result = if expected == "success" {
        super::super::resolve_base_with_hit(input, None).await
    } else {
        variant::get(input, &[])
            .await
            .map(|_| unreachable!("negative case returned a card"))
    };
    match expected {
        "success" => {
            let (detail, format, hit) = result.unwrap();
            assert_eq!(serde_json::to_value(hit).unwrap(), row["decoded_hit"]);
            assert_eq!(
                format,
                VariantIdFormat::GeneProteinChange {
                    gene: "GENE".into(),
                    change: row["change"].as_str().unwrap().into()
                }
            );
            assert_eq!(detail.genome_build, Some(GenomeBuild::Grch37));
            assert_eq!(serde_json::to_value(detail).unwrap(), row["detail"]);
            ledger(&requests, row["query"].as_str().unwrap(), offsets);
            requests.lock().unwrap().clear();
            assert_eq!(
                serde_json::to_value(variant::get(input, &[]).await.unwrap()).unwrap(),
                row["detail"]
            );
        }
        "absent" => match result.unwrap_err() {
            BioMcpError::NotFound {
                entity,
                id,
                suggestion,
            } => {
                assert_eq!(entity, "variant");
                assert_eq!(id, input);
                assert_eq!(
                    suggestion,
                    format!(
                        "Try searching: biomcp search variant -g GENE --hgvsp {}",
                        row["change"].as_str().unwrap()
                    )
                );
            }
            other => panic!("{other:?}"),
        },
        "malformed" | "failure" => {
            let error = result.unwrap_err();
            assert_eq!(
                serde_json::from_str::<Value>(&crate::render::json::to_error_json(&error).unwrap())
                    .unwrap(),
                oracle()[if expected == "malformed" {
                    "malformed_payload"
                } else {
                    "source_failure_payload"
                }]
            );
        }
        error => match result.unwrap_err() {
            BioMcpError::InvalidArgument(message) => assert_eq!(
                message,
                row.get("errors").unwrap_or(&oracle()["errors"])[error]
            ),
            other => panic!("{other:?}"),
        },
    }
    ledger(&requests, row["query"].as_str().unwrap(), offsets);
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn interval_edits_reach_detail_with_written_source_and_two_query_styles() {
    for row in oracle()["admission"].as_array().unwrap() {
        exercise(
            row["input"].as_str().unwrap(),
            vec![json!({"total":1,"hits":[row["hit"]]})],
            "success",
            &[0],
            row,
        )
        .await;
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn interval_get_finishes_scan_before_resolving_identity() {
    let row = &oracle()["admission"][1];
    let h1 = row["hit"].clone();
    let mut h2 = h1.clone();
    h2["_id"] = json!("chr1:g.102A>T");
    h2["dbsnp"]["rsid"] = json!("rs102");
    let mut wrong = h1.clone();
    wrong["dbnsfp"]["genename"] = json!("OTHER");
    let mut missing = h1.clone();
    missing["dbnsfp"].as_object_mut().unwrap().remove("hgvsp");
    let mut coding = h1.clone();
    coding["dbnsfp"]["hgvsc"] = json!("NM_000001.1:c.2A>T");
    let mut extra = h1.clone();
    extra["dbnsfp"]["hgvsp"] = json!(["p.Ala11_Gly12del", "p.Ser20dup"]);
    let mut single = h1.clone();
    single["dbnsfp"]["hgvsp"] = json!(["p.Ala11_Gly12del"]);
    let mut multiplicity = h1.clone();
    multiplicity["dbnsfp"]["hgvsp"] = json!(["p.Ala11_Gly12del", "p.Ala11_Gly12del"]);
    for (hits, outcome) in [
        (vec![wrong.clone(), h1.clone()], "success"),
        (vec![h1.clone(), h1.clone()], "success"),
        (vec![h1.clone(), h2.clone()], "ambiguous"),
        (vec![h1.clone(), h2.clone(), missing.clone()], "ambiguous"),
        (vec![h1.clone(), coding], "ambiguous"),
        (vec![single, extra], "ambiguous"),
        (vec![h1.clone(), multiplicity], "ambiguous"),
        (vec![missing.clone()], "evidence"),
        (vec![h1.clone(), missing], "evidence"),
        (vec![wrong.clone()], "absent"),
        (vec![], "absent"),
    ] {
        exercise(
            "GENE p.Ala11_Gly12del",
            vec![json!({"total":hits.len(),"hits":hits})],
            outcome,
            &[0],
            row,
        )
        .await;
    }
    let mut asserted = row.clone();
    asserted["hit"]["dbnsfp"]["hgvsp"] = json!(["p.Ala11_Gly12del", "p.Ser20dup"]);
    asserted["decoded_hit"]["dbnsfp"]["hgvsp"] = asserted["hit"]["dbnsfp"]["hgvsp"].clone();
    let mut reordered = asserted["hit"].clone();
    reordered["dbnsfp"]["hgvsp"] = json!(["p.Ser20dup", "p.Ala11_Gly12del"]);
    exercise(
        "GENE p.Ala11_Gly12del",
        vec![json!({"total":2,"hits":[asserted["hit"],reordered]})],
        "success",
        &[0],
        &asserted,
    )
    .await;
    for protein in [
        "p.(Ala11_Gly12del)",
        "p.Ala10_Gly12del",
        "p.Ala11_Ser12del",
        "p.Ala11_Gly12dup",
        "p.Ala11_Gly12delinsSerVal",
    ] {
        let mut contradicted = h1.clone();
        contradicted["dbnsfp"]["hgvsp"] = json!(protein);
        exercise(
            "GENE p.Ala11_Gly12del",
            vec![json!({"total":1,"hits":[contradicted]})],
            "absent",
            &[0],
            row,
        )
        .await;
    }
    for (pages, outcome, offsets) in [
        (
            vec![
                json!({"total":51,"hits":vec![wrong.clone();50]}),
                json!({"total":51,"hits":[h1.clone()]}),
            ],
            "success",
            vec![0, 50],
        ),
        (
            vec![
                json!({"total":51,"hits":vec![h1.clone();50]}),
                json!({"total":51,"hits":[h2]}),
            ],
            "ambiguous",
            vec![0, 50],
        ),
        (
            vec![json!({"hits":[h1.clone()]}), json!({"hits":[]})],
            "success",
            vec![0, 1],
        ),
        (
            vec![json!({"total":2,"hits":[h1.clone()]}), json!("failure")],
            "failure",
            vec![0, 1],
        ),
        (vec![json!("malformed")], "malformed", vec![0]),
    ] {
        exercise("GENE p.Ala11_Gly12del", pages, outcome, &offsets, row).await;
    }
    for (total, outcome) in [
        (Some(1000), "success"),
        (Some(1001), "incomplete"),
        (None, "incomplete"),
    ] {
        let mut pages = Vec::new();
        for index in 0..20 {
            let mut hits = vec![wrong.clone(); 50];
            if index == 0 {
                hits[0] = h1.clone();
            }
            let mut page = json!({"hits":hits});
            if let Some(total) = total {
                page["total"] = json!(total);
            }
            pages.push(page);
        }
        exercise(
            "GENE p.Ala11_Gly12del",
            pages,
            outcome,
            &(0..20).map(|n| n * 50).collect::<Vec<_>>(),
            row,
        )
        .await;
    }
    let mut partial = vec![wrong.clone(); 1000];
    partial[0] = h1.clone();
    partial[1] = row["hit"].clone();
    partial[1]["_id"] = json!("chr1:g.102A>T");
    exercise(
        "GENE p.Ala11_Gly12del",
        vec![json!({"total":1001,"hits":partial})],
        "incomplete",
        &[0],
        row,
    )
    .await;
    let mut oversized = vec![wrong; 1001];
    oversized[0] = h1;
    oversized[1000] = row["hit"].clone();
    oversized[1000]["_id"] = json!("chr1:g.103A>T");
    exercise(
        "GENE p.Ala11_Gly12del",
        vec![json!({"total":1001,"hits":oversized})],
        "incomplete",
        &[0],
        row,
    )
    .await;
    let mut displayed = row.clone();
    displayed["hit"]["snpeff"]["ann"][0]["hgvs_p"] = json!("p.Ser20dup");
    displayed["decoded_hit"]["snpeff"]["ann"][0]["hgvs_p"] = json!("p.Ser20dup");
    displayed["detail"]["hgvs_p"] = json!("p.Ser20dup");
    exercise(
        "GENE p.Ala11_Gly12del",
        vec![json!({"total":1,"hits":[displayed["hit"]]})],
        "success",
        &[0],
        &displayed,
    )
    .await;
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn interval_get_refuses_invalid_inputs_and_original_or_rendered_limits_without_requests() {
    let (fixture, requests) = fixture(vec![json!({"total":0,"hits":[]})]).await;
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    for row in oracle()["refusals"].as_array().unwrap() {
        let input = row["input"].as_str().unwrap();
        match variant::get(input, &[]).await.unwrap_err() {
            BioMcpError::InvalidArgument(message) => assert_eq!(message, row["message"]),
            error => panic!("{error:?}"),
        }
    }
    for input in [
        format!("{}GENE p.A11del", " ".repeat(500)),
        format!("GENE p.A11_G12ins{}", "V".repeat(180)),
    ] {
        match variant::get(&input, &[]).await.unwrap_err() {
            BioMcpError::InvalidArgument(message) => {
                assert_eq!(message, oracle()["errors"]["limit"])
            }
            error => panic!("{error:?}"),
        }
        assert!(matches!(
            variant::classify_variant_input(&input),
            variant::VariantInputKind::Unsupported
        ));
    }
    assert!(requests.lock().unwrap().is_empty());
    let padded = format!("{}GENE p.Ala11_Gly12del", " ".repeat(491));
    assert_eq!(padded.len(), 512);
    let row = &oracle()["admission"][1];
    exercise(
        &padded,
        vec![json!({"total":1,"hits":[row["hit"]]})],
        "success",
        &[0],
        row,
    )
    .await;
    let prepared =
        crate::entities::variant::resolution::protein_get::prepare("GENE p.Ala11_Gly12del")
            .unwrap()
            .unwrap();
    assert_eq!(
        format!("{prepared:?}"),
        "ProteinGet { source_bytes: 21, gene_bytes: 4, change_bytes: 16, query_bytes: [12, 16], disposition: Checked }"
    );
}

pub(super) fn point_pages(case: &Value, point: &Value) -> Vec<Value> {
    case["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|page| {
            if page.is_string() {
                return page.clone();
            }
            let mut hits = Vec::new();
            for group in page["rows"].as_array().unwrap() {
                hits.extend(std::iter::repeat_n(
                    point["records"][group[0].as_str().unwrap()].clone(),
                    group[1].as_u64().unwrap() as usize,
                ));
            }
            let mut value = json!({"hits":hits});
            if !page["total"].is_null() {
                value["total"] = page["total"].clone();
            }
            value
        })
        .collect()
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn point_get_finishes_scan_before_resolving_complete_assertion_identity() {
    let data = oracle();
    let point = &data["point"];
    for case in point["cases"].as_array().unwrap() {
        let mut row = point.clone();
        row["decoded_hit"] = point["decoded"][case["selected"].as_str().unwrap()].clone();
        let offsets = case["offsets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        exercise(
            point["input"].as_str().unwrap(),
            point_pages(case, point),
            case["outcome"].as_str().unwrap(),
            &offsets,
            &row,
        )
        .await;
    }
}
