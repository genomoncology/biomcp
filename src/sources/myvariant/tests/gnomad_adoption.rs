//! Whole-hit codecs retain canonical source encoding and sanitize owned errors.
use super::super::{MyVariantClient, MyVariantHit};
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::error::BioMcpError;
use serde_json::json;

const AF: &str = r#"{"af":0.01,"af_afr":0.02,"af_eas":0.03,"af_nfe":0.04,"af_sas":0.05,"af_amr":0.06,"af_asj":0.07,"af_fin":0.08,"af_afr_female":0.09,"af_afr_male":0.1,"af_amr_female":0.11,"af_amr_male":0.12,"af_eas_jpn":0.13,"af_eas_kor":0.14,"af_nfe_bgr":0.15,"af_nfe_est":0.16,"af_nfe_nwe":0.17,"af_nfe_onf":0.18,"af_nfe_seu":0.19,"af_nfe_swe":0.2,"af_oth":0.21}"#;
const EXOME: &str = r#"{"af":0.2,"af_afr":null,"af_eas":null,"af_nfe":null,"af_sas":null,"af_amr":null,"af_asj":null,"af_fin":null,"af_afr_female":null,"af_afr_male":null,"af_amr_female":null,"af_amr_male":null,"af_eas_jpn":null,"af_eas_kor":null,"af_nfe_bgr":null,"af_nfe_est":null,"af_nfe_nwe":null,"af_nfe_onf":null,"af_nfe_seu":null,"af_nfe_swe":null,"af_oth":null}"#;
const GENOME: &str = r#"{"af":0.3,"af_afr":null,"af_eas":null,"af_nfe":null,"af_sas":null,"af_amr":null,"af_asj":null,"af_fin":null,"af_afr_female":null,"af_afr_male":null,"af_amr_female":null,"af_amr_male":null,"af_eas_jpn":null,"af_eas_kor":null,"af_nfe_bgr":null,"af_nfe_est":null,"af_nfe_nwe":null,"af_nfe_onf":null,"af_nfe_seu":null,"af_nfe_swe":null,"af_oth":null}"#;

#[test]
fn gnomad_hit_encodes_complete_canonical_objects_from_bytes_and_value() {
    for consequence in [r#""missense""#, r#"["missense","synonymous"]"#] {
        let prefix = format!(
            r#"{{"_id":"safe-id","cadd":{{"phred":32.0,"consequence":{consequence}}},"clinvar":null,"dbnsfp":null,"dbsnp":{{"rsid":"rs123"}},"gnomad_exome":{{"af":{AF}}},"gnomad":{{"exomes":{{"af":{EXOME}}},"genomes":{{"af":{GENOME}}}}},"exac":null,"exac_nontcga":null,"cosmic":null,"cgi":null,"civic":null,"snpeff":{{"ann":[{{"feature_id":null,"genename":"BRAF","hgvs_c":null,"hgvs_p":"p.Val600Glu"}}]}}}}"#
        );
        let positional = format!(
            r#"{{"_id":"safe-id","cadd":{{"phred":32.0,"consequence":{consequence}}},"dbsnp":{{"rsid":"rs123"}},"snpeff":{{"ann":{{"genename":"BRAF","hgvs_p":"p.Val600Glu"}}}},"gnomad_exome":[[0.01,0.02,0.03,0.04,0.05,0.06,0.07,0.08,0.09,0.1,0.11,0.12,0.13,0.14,0.15,0.16,0.17,0.18,0.19,0.2,0.21]],"gnomad":[[{{"af":0.2}}],[{{"af":0.3}}]]}}"#
        );
        for raw in [&prefix, &positional] {
            for hit in [
                serde_json::from_str::<MyVariantHit>(raw).unwrap(),
                MyVariantHit::from_value(serde_json::from_str(raw).unwrap()).unwrap(),
            ] {
                assert_eq!(serde_json::to_string(&hit.clone()).unwrap(), prefix);
            }
        }
    }
    let hit: MyVariantHit = serde_json::from_str(r#"{"_id":"safe-id"}"#).unwrap();
    assert_eq!(
        serde_json::to_value(hit).unwrap(),
        json!({"_id":"safe-id","cadd":null,"clinvar":null,"dbnsfp":null,"dbsnp":null,"gnomad_exome":null,"gnomad":null,"exac":null,"exac_nontcga":null,"cosmic":null,"cgi":null,"civic":null,"snpeff":null})
    );
    assert!(
        serde_json::from_str::<MyVariantHit>(
            r#"{"_id":"safe-id","gnomad_exome":{"af":null,"af":{}}}"#
        )
        .is_err()
    );
}

fn private_error(error: &BioMcpError) {
    let underlying = match error {
        BioMcpError::WithSourceContext { context, source } => {
            assert_eq!(context.provider().label(), "MyVariant.info");
            source.as_ref()
        }
        error => error,
    };
    assert!(
        matches!(underlying, BioMcpError::ApiJson { api, .. } if api.eq_ignore_ascii_case("myvariant.info"))
    );
    let mut cause: Option<&dyn std::error::Error> = Some(error);
    while let Some(error) = cause {
        let diagnostic = format!("{error:?} {error}");
        assert!(!diagnostic.contains("PRIVATE_AF_MARKER"), "{diagnostic}");
        assert!(!diagnostic.contains("987654321"), "{diagnostic}");
        cause = error.source();
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn gnomad_owned_failures_are_private_through_bytes_get_and_get_all() {
    for raw in [
        r#"{"_id":"safe-id","gnomad_exome":{"af":{"af":"PRIVATE_AF_MARKER"}}}"#,
        r#"{"_id":"safe-id","gnomad":{"exomes":{"af":{"af":"PRIVATE_AF_MARKER"}}}}"#,
        r#"{"_id":"safe-id","gnomad_exome":987654321}"#,
        r#"{"_id":"safe-id","gnomad":987654321}"#,
    ] {
        let error = crate::sources::decode_json::<MyVariantHit>(
            crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
            reqwest::StatusCode::OK,
            Some(&reqwest::header::HeaderValue::from_static(
                "application/json",
            )),
            raw.as_bytes(),
            true,
        )
        .unwrap_err();
        private_error(&error);
        let fixture = TestHttpFixture::spawn(move |_| {
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                raw.as_bytes(),
            ))
        })
        .await;
        let mut env = TestEnv::new();
        env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set("BIOMCP_CACHE_MODE", "off");
        let client = MyVariantClient::new().unwrap();
        private_error(&client.get("safe-id", None).await.unwrap_err());
        private_error(&client.get_all("safe-id").await.unwrap_err());
    }
}
