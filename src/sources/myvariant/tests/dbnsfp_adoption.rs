//! Actual hit wiring, intermediate encoding and private observations.
use super::super::MyVariantHit;
use serde_json::json;

#[test]
fn dbnsfp_embedding_preserves_siblings_encoding_and_private_debug() {
    let input = json!({"_id":"chr7:g.140453136A>T", "clinvar":{"variant_id":13961},
        "snpeff":{"ann":{"genename":"BRAF","hgvs_p":"p.Val600Glu"}},
        "dbnsfp":{"genename":"private-dbnsfp-marker","hgvsp":["p.V600E","p.V600E"],"hgvsc":null,
            "revel":{"score":[0.94,0.11]}}});
    let hit: MyVariantHit = serde_json::from_value(input).unwrap();
    assert!(!format!("{hit:?}").contains("private-dbnsfp-marker"));
    let encoded = serde_json::to_value(&hit).unwrap();
    assert_eq!(encoded["_id"], "chr7:g.140453136A>T");
    assert_eq!(encoded["clinvar"]["variant_id"], 13961);
    assert_eq!(encoded["snpeff"]["ann"][0]["genename"], "BRAF");
    assert_eq!(
        encoded["dbnsfp"],
        json!({"genename":"private-dbnsfp-marker",
        "hgvsp":["p.V600E","p.V600E"],"hgvsc":null,"sift":null,"polyphen2":null,
        "revel":{"score":[0.94,0.11],"rankscore":null},"alphamissense":null,
        "clinpred":null,"metarnn":null,"bayesdel":null,"phylop":null,"phastcons":null,"gerp++":null})
    );
    let error = serde_json::from_str::<MyVariantHit>(
        r#"{"_id":"safe-id","dbnsfp":{"revel":{"score":"private-dbnsfp-marker"}}}"#,
    )
    .unwrap_err();
    assert!(!error.to_string().contains("private-dbnsfp-marker"));
}
