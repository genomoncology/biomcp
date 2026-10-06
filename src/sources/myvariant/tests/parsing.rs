//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json` and
//! the response types, plus the pure post-processing helper `select_get_hit_value` and
//! the shared ClinVar / dbNSFP carrier shapes. No network, no server.

use crate::error::BioMcpError;
use crate::sources::decode_json;
use crate::sources::myvariant::{MyVariantClient, MyVariantHit, MyVariantSearchResponse};
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde_json::json;

#[test]
fn snpeff_embedding_preserves_siblings_encoding_and_private_debug() {
    let exact = |ann: serde_json::Value| {
        serde_json::from_value::<MyVariantHit>(json!({
            "_id": "chr5:g.118860951A>G",
            "clinvar": {"rcv": {"preferred_name": "NM_000414.3(HSD17B4):c.1544A>G (p.His515Arg)"}},
            "dbnsfp": {"genename": "HSD17B4", "hgvsp": "p.His540Arg"},
            "snpeff": {"ann": ann}
        }))
        .expect("malformed SnpEff must not discard valid siblings")
    };

    let valid = exact(json!({
        "feature_id": " NM_001199291.2 ",
        "genename": " HSD17B4 ",
        "hgvs_c": " c.1619A>G ",
        "hgvs_p": " p.His540Arg "
    }));
    assert!(!format!("{:?}", valid.snpeff).contains("NM_001199291.2"));
    assert_eq!(
        serde_json::to_value(&valid).unwrap()["snpeff"],
        json!({"ann":[{
            "feature_id":"NM_001199291.2", "genename":"HSD17B4",
            "hgvs_c":"c.1619A>G", "hgvs_p":"p.His540Arg"
        }]})
    );
    let partial = exact(json!({"feature_id":"NM_001199291.2"}));
    assert_eq!(
        serde_json::to_value(&partial).unwrap()["snpeff"],
        json!({"ann":[{
            "feature_id":"NM_001199291.2", "genename":null, "hgvs_c":null, "hgvs_p":null
        }]})
    );
    let malformed = exact(json!([{"feature_id":7}]));
    assert_eq!(
        serde_json::to_value(&malformed).unwrap()["snpeff"],
        json!({"ann":[]})
    );
    assert!(malformed.clinvar.is_some());
    assert!(malformed.dbnsfp.is_some());

    for value in [json!({"_id":"x"}), json!({"_id":"x", "snpeff":null})] {
        let hit: MyVariantHit = serde_json::from_value(value).unwrap();
        assert!(hit.snpeff.is_none());
        assert_eq!(serde_json::to_value(&hit).unwrap()["snpeff"], json!(null));
    }
}

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/myvariant/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

#[test]
fn parses_search_response_total_and_hits_from_real_fixture() {
    let resp: MyVariantSearchResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("search_braf.json"),
        true,
    )
    .unwrap();
    assert_eq!(resp.total, Some(1));
    assert!(!resp.hits.is_empty());
    assert!(resp.hits[0].id.starts_with("chr7"));
    assert_eq!(
        resp.hits[0]
            .dbnsfp
            .as_ref()
            .and_then(|d| d.genename().first()),
        Some("BRAF")
    );
}

#[test]
fn parses_receipted_braf_filter_searches() {
    let missense: MyVariantSearchResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("search_braf_missense_20260805.json"),
        true,
    )
    .unwrap();
    assert!(missense.hits.iter().any(|hit| {
        hit.dbnsfp
            .as_ref()
            .and_then(|dbnsfp| dbnsfp.genename().first())
            == Some("BRAF")
            && hit
                .cadd
                .as_ref()
                .and_then(|cadd| cadd.consequence.as_ref())
                .and_then(crate::utils::serde::StringOrVec::first)
                == Some("NON_SYNONYMOUS")
    }));

    let revel: MyVariantSearchResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("search_braf_revel_20260805.json"),
        true,
    )
    .unwrap();
    assert!(revel.hits.iter().any(|hit| {
        hit.dbnsfp
            .as_ref()
            .and_then(|dbnsfp| dbnsfp.genename().first())
            == Some("BRAF")
            && hit
                .dbnsfp
                .as_ref()
                .and_then(|dbnsfp| dbnsfp.revel())
                .and_then(|revel| revel.score())
                .and_then(biodata::MyVariantDbnsfpNumber::first)
                .is_some()
    }));
}

#[test]
fn parses_receipted_variant_identity_searches() {
    for (bytes, gene, protein_change) in [
        (
            &fixture!("search_braf_v600e_20260806.json")[..],
            "BRAF",
            "p.V600E",
        ),
        (
            &fixture!("search_myd88_l265p_20260806.json")[..],
            "MYD88",
            "p.L265P",
        ),
    ] {
        let response: MyVariantSearchResponse = decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
            StatusCode::OK,
            Some(&json_ct()),
            bytes,
            true,
        )
        .unwrap();

        assert!(response.hits.iter().any(|hit| {
            hit.dbnsfp
                .as_ref()
                .and_then(|dbnsfp| dbnsfp.genename().first())
                == Some(gene)
                && hit.dbnsfp.as_ref().is_some_and(|dbnsfp| {
                    dbnsfp
                        .hgvsp()
                        .values()
                        .iter()
                        .any(|value| value == protein_change)
                })
        }));
    }
}

#[test]
fn parses_receipted_braf_get_hit() {
    let hit: MyVariantHit = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("get_braf_v600e_20260805.json"),
        true,
    )
    .unwrap();

    assert_eq!(hit.id, "chr7:g.140453136A>T");
    assert_eq!(
        hit.dbnsfp
            .as_ref()
            .and_then(|dbnsfp| dbnsfp.genename().first()),
        Some("BRAF")
    );
    assert!(hit.dbnsfp.as_ref().is_some_and(|dbnsfp| {
        dbnsfp
            .hgvsp()
            .values()
            .iter()
            .any(|hgvsp| hgvsp == "p.V600E")
    }));
}

#[test]
fn parses_get_hit_nested_fields_from_real_fixture() {
    let hit: MyVariantHit = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("get_braf_v600e.json"),
        true,
    )
    .unwrap();

    assert_eq!(hit.id, "chr7:g.140453136A>T");
    assert_eq!(hit.cadd.as_ref().and_then(|c| c.phred), Some(32.0));
    assert_eq!(
        hit.dbsnp.as_ref().and_then(|d| d.rsid().map(str::to_owned)),
        Some("rs113488022".into())
    );
    assert_eq!(
        hit.dbnsfp
            .as_ref()
            .and_then(|d| d.revel())
            .and_then(|r| r.score())
            .and_then(biodata::MyVariantDbnsfpNumber::first),
        Some(0.931)
    );
    assert_eq!(
        hit.dbnsfp.as_ref().and_then(|d| d.genename().first()),
        Some("BRAF")
    );
    let bayesdel = hit.dbnsfp.as_ref().and_then(|d| d.bayesdel());
    assert_eq!(
        bayesdel
            .and_then(|b| b.add_af())
            .and_then(|v| v.score())
            .and_then(biodata::MyVariantDbnsfpNumber::first),
        Some(0.399079)
    );
    assert_eq!(
        bayesdel
            .and_then(|b| b.no_af())
            .and_then(|v| v.score())
            .and_then(biodata::MyVariantDbnsfpNumber::first),
        Some(0.335473)
    );
    assert_eq!(hit.cosmic.as_ref().and_then(|c| c.mut_freq), Some(2.83));
    assert!(hit.exac.as_ref().and_then(|e| e.af).is_some());
    assert_eq!(
        hit.clinvar.as_ref().and_then(|c| c.variant_id()),
        Some(13961)
    );
    assert!(
        hit.clinvar
            .as_ref()
            .map(|c| !c.rcv().is_empty())
            .unwrap_or(false)
    );
    assert!(
        hit.gnomad_exome
            .as_ref()
            .and_then(|g| g.af())
            .and_then(|a| a.af())
            .is_some()
    );
    assert!(hit.civic.is_some());
    assert!(hit.cgi.is_some());
}

#[test]
fn select_get_hit_value_passes_object_through() {
    let value = json!({"_id": "chr1:g.1A>T"});
    let out = MyVariantClient::select_get_hit_value(value.clone(), "chr1:g.1A>T").unwrap();
    assert_eq!(out, value);
}

#[test]
fn select_get_hit_values_preserves_object_array_and_empty_shapes() {
    let object = json!({"_id": "one"});
    assert_eq!(
        MyVariantClient::select_get_hit_values(object.clone()).unwrap(),
        vec![object]
    );
    let array = json!([{"_id": "first"}, {"_id": "second"}]);
    let hits = MyVariantClient::select_get_hit_values(array).unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[1]["_id"], "second");
    assert!(
        MyVariantClient::select_get_hit_values(json!([]))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn select_get_hit_value_takes_first_array_element() {
    let value = json!([{"_id": "first"}, {"_id": "second"}]);
    let out = MyVariantClient::select_get_hit_value(value, "x").unwrap();
    assert_eq!(out.get("_id").and_then(|v| v.as_str()), Some("first"));
}

#[test]
fn select_get_hit_value_empty_array_is_not_found() {
    let err = MyVariantClient::select_get_hit_value(json!([]), "rs999").unwrap_err();
    match err {
        BioMcpError::NotFound { entity, id, .. } => {
            assert_eq!(entity, "variant");
            assert_eq!(id, "rs999");
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn select_get_hit_value_scalar_is_api_error() {
    let err = MyVariantClient::select_get_hit_value(json!("nope"), "x").unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("Unexpected response type"));
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<MyVariantSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("MyVariant.info"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}

#[test]
fn decode_json_rejects_html_content_type() {
    let html = HeaderValue::from_static("text/html");
    let err = decode_json::<MyVariantSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::OK,
        Some(&html),
        b"<html><body>error</body></html>",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("MyVariant.info"), "got: {msg}");
    assert!(msg.contains("HTML"), "got: {msg}");
}
