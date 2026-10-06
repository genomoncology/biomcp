//! Callable detail keeps card trimming and ambiguity source spelling distinct.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::entities::variant;
use crate::error::BioMcpError;
use serde_json::json;
use std::sync::{Arc, Mutex};

#[tokio::test]
#[serial_test::serial(source_env)]
async fn dbsnp_detail_trims_card_but_keeps_nonblank_ambiguity_text() {
    for (input, body, expected, path) in [
        (
            "chr1:g.101A>T",
            json!({"_id":"chr1:g.101A>T","dbnsfp":{"genename":"GENE"},"dbsnp":{"rsid":" RS101 "}}),
            json!({"gene":"GENE","id":"chr1:g.101A>T","genome_build":"GRCh38","rsid":"RS101",
                "section_outcomes":{
                    "cancerhotspots":{"outcome":"not_requested","sources":[]},
                    "cbioportal":{"outcome":"not_requested","sources":[]},
                    "civic":{"outcome":"not_requested","sources":[]},
                    "clinvar":{"outcome":"not_requested","sources":[]},
                    "gwas":{"outcome":"not_requested","sources":[]},
                    "population":{"outcome":"not_requested","sources":[]},
                    "predict":{"outcome":"not_requested","sources":[]}}}),
            "/variant/chr1:g.101A%3ET",
        ),
        (
            "GENE A11V",
            json!({"total":2,"hits":[
            {"_id":"chr1:g.101A>T","dbnsfp":{"genename":"GENE","hgvsp":"p.Ala11Val"},"dbsnp":{"rsid":" RS101 "}},
            {"_id":"chr1:g.102A>T","dbnsfp":{"genename":"GENE","hgvsp":"p.Ala11Val"},"dbsnp":{"rsid":"   "}}]}),
            json!(
                "Ambiguous protein change 'GENE A11V': 2 variants match and none of them carries a ClinVar record that names one; BioMCP refuses rather than return the wrong variant.\nCandidates:\n- chr1:g.101A>T ( RS101 )\n- chr1:g.102A>T\nRetry `biomcp get variant` with one candidate's exact form: its genomic HGVS, ClinVar VariationID, rsID, or a transcript-qualified HGVS."
            ),
            "/query",
        ),
    ] {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let fixture = TestHttpFixture::spawn(move |request| {
            captured.lock().unwrap().push(request.to_string());
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                &serde_json::to_vec(&body).unwrap(),
            ))
        })
        .await;
        let mut env = TestEnv::new();
        env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set("BIOMCP_CACHE_MODE", "off");
        let actual = match variant::get(input, &[]).await {
            Ok(card) => serde_json::to_value(card).unwrap(),
            Err(BioMcpError::InvalidArgument(message)) => json!(message),
            Err(error) => panic!("{error:?}"),
        };
        assert_eq!(actual, expected);
        let requests = requests.lock().unwrap();
        assert_eq!(
            requests.len(),
            if path == "/query" { 1 } else { 2 },
            "{requests:?}"
        );
        for (index, request) in requests.iter().enumerate() {
            let target = request
                .lines()
                .next()
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap();
            let url = reqwest::Url::parse(&format!("http://localhost{target}")).unwrap();
            assert_eq!(url.path(), path);
            let pairs = url.query_pairs().into_owned().collect::<Vec<_>>();
            let fields = crate::sources::myvariant::MYVARIANT_FIELDS_GET;
            let expected_pairs = if path == "/query" {
                vec![
                    (
                        "q".into(),
                        "dbnsfp.genename:GENE AND dbnsfp.hgvsp:\"p.A11V\"".into(),
                    ),
                    ("size".into(), "50".into()),
                    ("from".into(), "0".into()),
                    ("fields".into(), fields.into()),
                ]
            } else {
                vec![
                    ("fields".into(), fields.into()),
                    (
                        "assembly".into(),
                        if index == 0 { "hg38" } else { "hg19" }.into(),
                    ),
                ]
            };
            assert_eq!(pairs, expected_pairs);
            assert_eq!(request.split_once("\r\n\r\n").unwrap().1, "");
        }
    }
}
