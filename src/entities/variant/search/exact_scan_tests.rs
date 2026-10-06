//! Original MIT synthetic JSON. No provider payload or reference assertion is used.
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub(super) fn hit() -> Value {
    json!({"_id":"chr7:g.101A>T","dbnsfp":{"genename":"BRAF","hgvsp":"p.Val600Glu"}})
}
fn wrong() -> Value {
    let mut row = hit();
    row["dbnsfp"]["hgvsp"] = json!("p.Val601Glu");
    row
}
pub(super) fn filters() -> VariantSearchFilters {
    VariantSearchFilters {
        gene: Some("BRAF".into()),
        hgvsp: Some("p.Val600Glu".into()),
        requested_identity: Some(RequestedVariantIdentity::for_search(
            Some("BRAF".into()),
            Some("p.Val600Glu".into()),
            None,
            None,
        )),
        ..Default::default()
    }
}
pub(super) async fn fixture(pages: Vec<Value>) -> (TestHttpFixture, Arc<Mutex<Vec<String>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        let mut log = captured.lock().unwrap();
        let index = log.len();
        log.push(request.into());
        let (status, bytes) = match pages.get(index).or_else(|| pages.last()) {
            Some(page) if page == "failure" => (
                "503 Service Unavailable",
                b"authored-provider-secret".to_vec(),
            ),
            Some(page) => ("200 OK", serde_json::to_vec(page).unwrap()),
            None => ("400 Bad Request", b"unexpected-request".to_vec()),
        };
        TestHttpReply::Bytes(test_http_response(status, "application/json", &bytes))
    })
    .await;
    (fixture, requests)
}
pub(super) fn ledger(requests: &Arc<Mutex<Vec<String>>>, offsets: &[usize], query: &str) {
    let log = requests.lock().unwrap();
    assert_eq!(log.len(), offsets.len(), "{log:?}");
    for (request, offset) in log.iter().zip(offsets) {
        let words: Vec<_> = request.lines().next().unwrap().split_whitespace().collect();
        assert_eq!((words[0], words[2]), ("GET", "HTTP/1.1"));
        let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
        assert_eq!(url.path(), "/query");
        let pairs: std::collections::BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["size"], "50");
        assert_eq!(pairs["from"], offset.to_string());
        assert_eq!(pairs["q"], query);
        assert_eq!(
            pairs["fields"],
            "_id,dbnsfp.genename,dbnsfp.hgvsp,dbnsfp.hgvsc,dbnsfp.revel.score,dbnsfp.gerp*.rs,clinvar.gene.symbol,clinvar.rcv.clinical_significance,clinvar.rcv.review_status,clinvar.rcv.preferred_name,clinvar.variant_id,snpeff.ann.feature_id,snpeff.ann.genename,snpeff.ann.hgvs_c,snpeff.ann.hgvs_p,dbsnp.rsid,gnomad_exome.af.af,gnomad.exomes.af.af,gnomad.genomes.af.af,cadd.consequence"
        );
        assert_eq!(pairs.len(), 4);
        assert_eq!(request.split_once("\r\n\r\n").unwrap().1, "");
    }
}
async fn exercise(
    pages: Vec<Value>,
    filters: VariantSearchFilters,
    offset: usize,
    ids: &[&str],
    status: VariantResolutionStatus,
    completion: (bool, Option<usize>),
    offsets: &[usize],
) {
    let (complete, total) = completion;
    let (fixture, requests) = fixture(pages).await;
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_MODE", "off");
    let page = search_page(&filters, 1, offset).await.unwrap();
    // Outcome precedes the ledger: baseline red must be the scan defect.
    let resolution = page.resolution.unwrap();
    assert_eq!(resolution.status, status);
    assert_eq!(resolution.exhaustive, complete);
    assert_eq!(
        page.results
            .iter()
            .map(|r| r.id.as_str())
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(page.total, total);
    assert_eq!(page.has_more, Some(!complete));
    assert_eq!(page.requested_variant, filters.requested_identity);
    let query = match (
        filters.hgvsp.as_deref(),
        filters.hgvsc.as_deref(),
        filters.rsid.as_deref(),
    ) {
        (Some("p.Val600Glu"), _, _) => "dbnsfp.genename:BRAF AND dbnsfp.hgvsp:\"p.Val600Glu\"",
        (Some("p.Glu746_Ala750del"), _, _) => {
            "dbnsfp.genename:EGFR AND dbnsfp.hgvsp:\"p.Glu746_Ala750del\""
        }
        (_, Some("c.1799T>A"), _) => "dbnsfp.genename:BRAF AND dbnsfp.hgvsc:\"c.1799T>A\"",
        (_, _, Some("rs113488022")) => "dbsnp.rsid:\"rs113488022\"",
        _ => panic!("unknown authored family"),
    };
    for row in &page.results {
        let source = row.source_identity.as_ref().unwrap();
        if let Some(protein) = &filters.hgvsp {
            assert_eq!(source.protein_changes, [protein.clone()]);
        }
        if let Some(coding) = &filters.hgvsc {
            assert_eq!(source.coding_changes, [coding.clone()]);
        }
        if let Some(rsid) = &filters.rsid {
            assert_eq!(source.rsids, [rsid.clone()]);
        }
    }
    ledger(&requests, offsets, query);
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn outstanding_total_survives_empty_or_omitted_total() {
    for end in [json!({"total":2,"hits":[]}), json!({"hits":[]})] {
        exercise(
            vec![json!({"total":2,"hits":[hit()]}), end],
            filters(),
            0,
            &["chr7:g.101A>T"],
            VariantResolutionStatus::Ambiguous,
            (false, None),
            &[0, 1],
        )
        .await;
    }
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn maximum_total_survives_decrease() {
    exercise(
        vec![
            json!({"total":60,"hits":[hit()]}),
            json!({"total":1,"hits":[hit()]}),
            json!({"hits":[]}),
        ],
        filters(),
        0,
        &["chr7:g.101A>T"],
        VariantResolutionStatus::Ambiguous,
        (false, None),
        &[0, 1, 2],
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn received_tail_prevents_false_completion() {
    let mut rows = vec![hit(); 1000];
    let mut other = hit();
    other["_id"] = json!("chr7:g.104A>T");
    rows.push(other);
    exercise(
        vec![json!({"total":1000,"hits":rows})],
        filters(),
        0,
        &["chr7:g.101A>T"],
        VariantResolutionStatus::Ambiguous,
        (false, None),
        &[0],
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn completion_controls_count_raw_rows_and_keep_unknown_totals() {
    for (pages, complete, offsets) in [
        (
            vec![
                json!({"hits":[hit()]}),
                json!({"hits":[wrong()]}),
                json!({"hits":[]}),
            ],
            true,
            vec![0, 1, 2],
        ),
        (vec![json!({"total":2,"hits":[hit(),hit()]})], true, vec![0]),
        (
            vec![
                json!({"total":2,"hits":[hit()]}),
                json!({"total":3,"hits":[wrong()]}),
                json!({"hits":[]}),
            ],
            false,
            vec![0, 1, 2],
        ),
    ] {
        exercise(
            pages,
            filters(),
            0,
            &["chr7:g.101A>T"],
            if complete {
                VariantResolutionStatus::Resolved
            } else {
                VariantResolutionStatus::Ambiguous
            },
            (complete, complete.then_some(1)),
            &offsets,
        )
        .await;
    }
    for total in [Some(1000), Some(1001), None] {
        let mut page = json!({"hits":vec![hit();50]});
        if let Some(total) = total {
            page["total"] = json!(total);
        }
        let complete = total == Some(1000);
        exercise(
            vec![page; 20],
            filters(),
            0,
            &["chr7:g.101A>T"],
            if complete {
                VariantResolutionStatus::Resolved
            } else {
                VariantResolutionStatus::Ambiguous
            },
            (complete, complete.then_some(1)),
            &(0..1000).step_by(50).collect::<Vec<_>>(),
        )
        .await;
    }
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn filtering_and_offsets_preserve_exact_page_truth() {
    let mut other = hit();
    other["_id"] = json!("chr7:g.104A>T");
    let mut alias = hit();
    alias["dbnsfp"]["hgvsp"] = json!("p.V600E");
    exercise(
        vec![json!({"total":5,"hits":[wrong(),hit(),hit(),alias,other]})],
        filters(),
        1,
        &["chr7:g.104A>T"],
        VariantResolutionStatus::Ambiguous,
        (true, Some(2)),
        &[0],
    )
    .await;
    exercise(
        vec![json!({"total":2,"hits":[hit(),{"_id":"chr7:g.103A>T"}]})],
        filters(),
        0,
        &["chr7:g.101A>T"],
        VariantResolutionStatus::Ambiguous,
        (true, Some(1)),
        &[0],
    )
    .await;
    exercise(
        vec![json!({"total":1,"hits":[{"_id":"chr7:g.103A>T"}]})],
        filters(),
        0,
        &[],
        VariantResolutionStatus::Ambiguous,
        (true, Some(0)),
        &[0],
    )
    .await;
    exercise(
        vec![json!({"total":1,"hits":[wrong()]})],
        filters(),
        0,
        &[],
        VariantResolutionStatus::Unresolved,
        (true, Some(0)),
        &[0],
    )
    .await;
    exercise(
        vec![
            json!({"total":2,"hits":[wrong()]}),
            json!({"total":2,"hits":[]}),
        ],
        filters(),
        0,
        &[],
        VariantResolutionStatus::Ambiguous,
        (false, None),
        &[0, 1],
    )
    .await;
    exercise(
        vec![json!({"total":3,"hits":[hit()]}), json!({"hits":[]})],
        filters(),
        2,
        &[],
        VariantResolutionStatus::Ambiguous,
        (false, None),
        &[0, 1],
    )
    .await;
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn exact_families_preserve_requests_and_source_spellings() {
    for (gene, protein, coding, rsid, row) in [
        (Some("BRAF"), Some("p.Val600Glu"), None, None, hit()),
        (
            Some("BRAF"),
            None,
            Some("c.1799T>A"),
            None,
            json!({"_id":"chr7:g.101A>T","dbnsfp":{"genename":"BRAF","hgvsc":"c.1799T>A"}}),
        ),
        (
            Some("EGFR"),
            Some("p.Glu746_Ala750del"),
            None,
            None,
            json!({"_id":"chr7:g.101A>T","dbnsfp":{"genename":"EGFR","hgvsp":"p.Glu746_Ala750del"}}),
        ),
        (
            None,
            None,
            None,
            Some("rs113488022"),
            json!({"_id":"chr7:g.101A>T","dbsnp":{"rsid":"rs113488022"}}),
        ),
    ] {
        let owned = |s: Option<&str>| s.map(str::to_owned);
        let filters = VariantSearchFilters {
            gene: owned(gene),
            hgvsp: owned(protein),
            hgvsc: owned(coding),
            rsid: owned(rsid),
            requested_identity: Some(RequestedVariantIdentity::for_search(
                owned(gene),
                owned(protein),
                owned(coding),
                owned(rsid),
            )),
            ..Default::default()
        };
        exercise(
            vec![json!({"total":2,"hits":[row]}), json!({"hits":[]})],
            filters,
            0,
            &["chr7:g.101A>T"],
            VariantResolutionStatus::Ambiguous,
            (false, None),
            &[0, 1],
        )
        .await;
    }
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn broad_search_and_limit_validation_keep_their_boundaries() {
    let (fixture, requests) = fixture(vec![json!({"total":2,"hits":[hit(),hit()]})]).await;
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_MODE", "off");
    let filters = VariantSearchFilters {
        gene: Some("BRAF".into()),
        ..Default::default()
    };
    for limit in [0, 51] {
        assert!(matches!(
            search_page(&filters, limit, 0).await,
            Err(BioMcpError::InvalidArgument(_))
        ));
    }
    assert!(requests.lock().unwrap().is_empty());
    let page = search_page(&filters, 1, 0).await.unwrap();
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.total, Some(2));
    assert!(page.resolution.is_none());
    assert!(page.requested_variant.is_none());
    assert_eq!(requests.lock().unwrap().len(), 1);
}
#[tokio::test]
#[serial_test::serial(source_env)]
async fn later_source_failure_returns_sanitized_error() {
    let (fixture, requests) =
        fixture(vec![json!({"total":2,"hits":[hit()]}), json!("failure")]).await;
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_MODE", "off");
    let error = search_page(&filters(), 1, 0).await.unwrap_err();
    assert_eq!(
        serde_json::from_str::<Value>(&crate::render::json::to_error_json(&error).unwrap())
            .unwrap(),
        json!({"error":{"code":"api","message":"API request to MyVariant.info failed.","source":"MyVariant.info","recovery":"Retry the remote source."},"_meta":{"not_found":false}})
    );
    assert!(
        !crate::render::json::to_error_json(&error)
            .unwrap()
            .contains("authored-provider-secret")
    );
    let log = requests.lock().unwrap();
    assert!(log.len() >= 2);
    for request in log.iter().skip(1) {
        assert!(request.contains("from=1"));
    }
}
