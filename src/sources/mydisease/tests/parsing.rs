//! Authored synthetic controls use no provider captures. MIT source-code license.
use crate::error::BioMcpError;
use crate::sources::mydisease::{
    MyDiseaseClient,
    projection::{decode_get, decode_search},
};
use reqwest::{StatusCode, header::HeaderValue};
use sha2::{Digest, Sha256};
fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}
#[test]
fn decode_get_hit_maps_not_found_status() {
    let error = MyDiseaseClient::decode_get_hit(
        StatusCode::NOT_FOUND,
        Some(&json_ct()),
        b"{}",
        "MONDO:missing",
    )
    .unwrap_err();
    assert!(matches!(error, BioMcpError::NotFound { .. }));
}
#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let error = MyDiseaseClient::decode_get_hit(
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"SOURCE-ONLY-CANARY",
        "MONDO:1",
    )
    .unwrap_err();
    assert_eq!(error.code(), "api");
    assert!(format!("{error:?}").contains("500"));
    assert!(!error.to_string().contains("SOURCE-ONLY-CANARY"));
}
#[test]
fn disease_identity_transport_table() {
    for case in crate::entities::disease::identity_surface_tests::controls::cases() {
        let get = decode_get(&case.get);
        let search = decode_search(&case.search);
        assert_eq!(get.is_err(), case.error, "{} get", case.label);
        assert_eq!(search.is_err(), case.error, "{} search", case.label);
        if let Ok(hit) = get {
            assert_eq!(
                hit.page.digest(),
                format!("sha256:{:x}", Sha256::digest(&case.get))
            );
            assert_eq!(hit.row.source().ordinal(), 0);
            assert_eq!(hit.row.identity().provider_id(), "MONDO:1");
            assert_eq!(
                crate::transform::disease::name_from_mydisease_hit(&hit),
                case.name
            );
        }
        if let Ok(page) = search {
            assert_eq!(page.total, 1);
            assert_eq!(page.hits[0].row.source().ordinal(), 0);
            assert_eq!(
                page.hits[0].page.digest(),
                format!("sha256:{:x}", Sha256::digest(&case.search))
            );
        }
    }
    for hpo in [
        r#"{"inheritance":{"hpo_id":"HP:0000006"}}"#,
        r#"{"inheritance":[{"hpo_id":"HP:0000006"}]}"#,
        r#"{"inheritance":null}"#,
    ] {
        let bytes = format!(r#"{{"_id":"MONDO:1","hpo":{hpo}}}"#);
        let hit = decode_get(bytes.as_bytes()).unwrap();
        assert_eq!(
            hit.hpo.unwrap().inheritance.len(),
            usize::from(!hpo.contains("null"))
        );
    }
    let page = decode_search(br#"{"total":0,"hits":[]}"#).unwrap();
    assert!(page.hits.is_empty());
    assert_eq!(page.total, 0);
    for body in [
        br#"{"hits":[]}"#.as_slice(),
        br#"{"total":18446744073709551616,"hits":[]}"#.as_slice(),
        br#"{"total":2,"hits":[{"_id":"MONDO:1"},{"_id":"bad","mondo":{"name":false}}]}"#
            .as_slice(),
    ] {
        assert!(decode_search(body).is_err());
    }
    let too_large = vec![b' '; 1_048_577];
    assert!(decode_get(&too_large).is_err());
    assert!(
        MyDiseaseClient::decode_get_hit(
            StatusCode::OK,
            Some(&HeaderValue::from_static("text/html")),
            b"{}",
            "MONDO:1"
        )
        .is_err()
    );
}
