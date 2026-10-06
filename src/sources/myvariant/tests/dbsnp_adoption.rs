//! dbSNP embedding owns canonical positional encoding and outer diagnostic privacy.
use super::super::MyVariantHit;
use serde_json::json;

#[test]
fn dbsnp_positional_embedding_encodes_canonical_object() {
    let raw = br#"{"_id":"safe-id","dbsnp":[" RS101 "]}"#;
    for hit in [
        serde_json::from_slice::<MyVariantHit>(raw).unwrap(),
        MyVariantHit::from_value(serde_json::from_slice(raw).unwrap()).unwrap(),
    ] {
        assert_eq!(
            serde_json::to_value(hit).unwrap(),
            json!({"_id":"safe-id",
            "cadd":null,"clinvar":null,"dbnsfp":null,"dbsnp":{"rsid":" RS101 "},
            "gnomad_exome":null,"gnomad":null,"exac":null,"exac_nontcga":null,
            "cosmic":null,"cgi":null,"civic":null,"snpeff":null})
        );
    }
}

#[test]
fn dbsnp_embedding_keeps_debug_and_outer_errors_private() {
    let hit: MyVariantHit = serde_json::from_str(
        r#"{"_id":"safe-id","dbsnp":{"rsid":"private-dbsnp-marker","private-dbsnp-key":true}}"#,
    )
    .unwrap();
    let debug = format!("{hit:?}");
    for marker in ["private-dbsnp-marker", "private-dbsnp-key"] {
        assert!(!debug.contains(marker), "{debug}");
    }
    for raw in [
        br#"{"_id":"safe-id","dbsnp":{"rsid":null,"rsid":"private-dbsnp-marker"}}"#.as_slice(),
        br#"{"_id":"safe-id","dbsnp":{"rsid":{"private-dbsnp-key":"private-dbsnp-marker"}}}"#
            .as_slice(),
        br#"{"_id":"safe-id","dbsnp":{"rsid":42}}"#.as_slice(),
    ] {
        let error = crate::sources::decode_json::<MyVariantHit>(
            crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
            reqwest::StatusCode::OK,
            Some(&reqwest::header::HeaderValue::from_static(
                "application/json",
            )),
            raw,
            true,
        )
        .unwrap_err();
        let mut cause: Option<&dyn std::error::Error> = Some(&error);
        while let Some(error) = cause {
            for marker in ["private-dbsnp-marker", "private-dbsnp-key"] {
                assert!(!format!("{error:?} {error}").contains(marker));
            }
            cause = error.source();
        }
    }
}
