//! Callable structure mapping retains every matching occurrence and unique positions.
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn dbnsfp_structure_retains_repeated_matches_and_other_positions() {
    let fixture = TestHttpFixture::spawn(|request| {
        let target = request.lines().next().unwrap().split_whitespace().nth(1).unwrap();
        let body = if target.starts_with("/variant/") {
            json!({"_id":"chr7:g.140453136A>T","dbnsfp":{"genename":"BRAF",
                "hgvsp":["p.Val600Glu","p.Val601Glu","p.V600E","p.V600E"]},
                "snpeff":{"ann":{"genename":"BRAF","hgvs_p":"p.Val600Glu"}}})
        } else if target.starts_with("/query") {
            json!({"total":1,"hits":[{"_id":"673","symbol":"BRAF","uniprot":{"Swiss-Prot":"P15056"}}]})
        } else if target.starts_with("/uniprotkb/") {
            json!({"primaryAccession":"P15056","uniProtkbId":"BRAF_HUMAN","sequence":{"length":766}})
        } else if target.starts_with("/entry/interpro/") {json!({"results":[]})}
        else if target.starts_with("/api/hotspots/") {json!([])}
        else {panic!("unexpected request: {target}")};
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &serde_json::to_vec(&body).unwrap()))
    }).await;
    let mut env = TestEnv::new();
    for key in [
        "BIOMCP_MYVARIANT_BASE",
        "BIOMCP_MYGENE_BASE",
        "BIOMCP_UNIPROT_BASE",
        "BIOMCP_INTERPRO_BASE",
        "BIOMCP_CANCERHOTSPOTS_BASE",
        "BIOMCP_TEST_UNPACED_ORIGIN",
    ] {
        env.set(key, &fixture.base);
    }
    env.set("BIOMCP_CACHE_MODE", "off");
    let result = structure("chr7:g.140453136A>T").await.unwrap();
    assert_eq!(result.residue.position, Some(600));
    assert_eq!(
        result.residue.matched_hgvsp,
        ["p.Val600Glu", "p.V600E", "p.V600E"]
    );
    assert_eq!(result.residue.other_source_positions, [601]);
    assert_eq!(result.residue.source, "MyVariant.info/dbNSFP");
    assert_eq!(
        result.residue.position_confidence,
        "requested_hgvsp_exact_match"
    );
    assert_eq!(
        result.warnings,
        [
            "MyVariant.info returned additional transcript/isoform protein positions; mapped domain/structure context uses the requested HGVSp position."
        ]
    );
}
