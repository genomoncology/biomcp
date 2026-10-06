//! Complete hit embedding retains source fields and private observations.
use super::super::MyVariantHit;
use serde_json::{Value, json};

#[test]
fn clinvar_embedding_preserves_complete_hit_and_private_observations() {
    for coding in [json!(" alias "), json!([" alias ", " alias "])] {
        let input = json!({"_id":"safe-id","cadd":{"phred":32.0,"consequence":"missense"},
            "dbsnp":{"rsid":"rs123"},
            "dbnsfp":{"genename":"BRAF","revel":{"score":[0.94,0.11]}},
            "snpeff":{"ann":{"genename":"BRAF","hgvs_p":"p.Val600Glu"}},
            "clinvar":{"variant_id":123,"gene":{"symbol":"private-clinvar-marker"},
                "hgvs":{"coding":coding},"rcv":{"accession":"RCV123","version":2,
                    "clinical_significance":"Pathogenic","review_status":"opaque review",
                    "conditions":{"name":"Alpha","unknown":{"private-condition-key":[true,1.25,null,["nested"]]}},
                    "preferred_name":"opaque name","last_evaluated":"opaque date","number_submitters":0}}});
        let hit: MyVariantHit = serde_json::from_value(input.clone()).unwrap();
        assert!(!format!("{hit:?}").contains("private-clinvar-marker"));
        assert!(!format!("{hit:?}").contains("private-condition-key"));
        let expected = json!({"_id":"safe-id","cadd":{"phred":32.0,"consequence":"missense"},
            "dbsnp":{"rsid":"rs123"},"gnomad_exome":null,"gnomad":null,"exac":null,
            "exac_nontcga":null,"cosmic":null,"cgi":null,"civic":null,
            "dbnsfp":{"genename":"BRAF","hgvsp":null,"hgvsc":null,"sift":null,"polyphen2":null,
                "revel":{"score":[0.94,0.11],"rankscore":null},"alphamissense":null,
                "clinpred":null,"metarnn":null,"bayesdel":null,"phylop":null,"phastcons":null,"gerp++":null},
            "snpeff":{"ann":[{"feature_id":null,"genename":"BRAF","hgvs_c":null,"hgvs_p":"p.Val600Glu"}]},
            "clinvar":{"variant_id":123,"gene":{"symbol":"private-clinvar-marker"},
                "hgvs":{"coding":coding},"rcv":[{"accession":"RCV123","version":2,
                    "clinical_significance":"Pathogenic","review_status":"opaque review",
                    "conditions":{"name":"Alpha","unknown":{"private-condition-key":[true,1.25,null,["nested"]]}},
                    "preferred_name":"opaque name","last_evaluated":"opaque date","number_submitters":0}]}});
        assert_eq!(serde_json::to_value(hit).unwrap(), expected);
    }
    for input in [
        json!({"_id":"safe-id"}),
        json!({"_id":"safe-id","clinvar":null}),
    ] {
        let hit: MyVariantHit = serde_json::from_value(input).unwrap();
        assert!(hit.clinvar.is_none());
        assert_eq!(serde_json::to_value(hit).unwrap()["clinvar"], Value::Null);
    }
    let error = crate::sources::decode_json::<MyVariantHit>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        reqwest::StatusCode::OK,
        Some(&reqwest::header::HeaderValue::from_static(
            "application/json",
        )),
        br#"{"_id":"safe-id","clinvar":{"variant_id":"private-clinvar-marker"}}"#,
        true,
    )
    .unwrap_err();
    assert!(!error.to_string().contains("private-clinvar-marker"));
}
