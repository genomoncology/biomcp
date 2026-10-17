//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to
//! `decode_json` and response types. No network, no server.

use crate::entities::variant::RequestedVariantIdentity;
use crate::sources::decode_json;
use crate::sources::litvar::{LitVarAutocompleteEntry, LitVarPublications};
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/litvar/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

#[test]
fn parses_the_cdkn2a_autocomplete_capture() {
    let entries: Vec<LitVarAutocompleteEntry> = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::LITVAR2),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("autocomplete_cdkn2a_a57v.json"),
        true,
    )
    .unwrap();

    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.id.as_deref(), Some("litvar@rs372266620##"));
    assert_eq!(entry.rsid.as_deref(), Some("rs372266620"));
    assert_eq!(entry.gene, vec!["CDKN2A".to_string()]);
    assert_eq!(entry.name.as_deref(), Some("p.A57V"));
    assert_eq!(entry.hgvs.as_deref(), Some("p.A57V"));
    assert_eq!(entry.pmids_count, Some(39));

    let identity = entry.source_identity().expect("identity");
    assert_eq!(identity.genes, vec!["CDKN2A".to_string()]);
    assert_eq!(identity.protein_changes, vec!["p.A57V".to_string()]);
    assert_eq!(identity.rsids, vec!["rs372266620".to_string()]);
    let requested =
        RequestedVariantIdentity::from_variant_input("CDKN2A p.A57V").expect("requested");
    assert!(matches!(
        crate::entities::variant::compare_variant_identity(&requested, &identity),
        crate::entities::variant::VariantIdentityComparison::Compatible { .. }
    ));
}

#[test]
fn parses_the_rsid_autocomplete_capture() {
    let entries: Vec<LitVarAutocompleteEntry> = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::LITVAR2),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("autocomplete_rsid_rs113488022.json"),
        true,
    )
    .unwrap();

    let entry = &entries[0];
    assert_eq!(entry.id.as_deref(), Some("litvar@rs113488022##"));
    assert_eq!(entry.gene, vec!["BRAF".to_string()]);
    assert_eq!(entry.name.as_deref(), Some("p.V600E"));
    assert_eq!(entry.pmids_count, Some(32_227));

    let identity = entry.source_identity().expect("identity");
    assert_eq!(identity.rsids, vec!["rs113488022".to_string()]);
    let requested = RequestedVariantIdentity::from_variant_input("rs113488022").expect("requested");
    assert!(matches!(
        crate::entities::variant::compare_variant_identity(&requested, &identity),
        crate::entities::variant::VariantIdentityComparison::Compatible { matched_alias }
            if matched_alias == "rs113488022"
    ));
}

#[test]
fn an_entry_without_an_id_resolves_no_identity() {
    let entry: LitVarAutocompleteEntry = serde_json::from_value(serde_json::json!({
        "rsid": "rs113488022",
        "gene": ["BRAF"],
        "name": "p.V600E"
    }))
    .unwrap();
    assert!(entry.source_identity().is_none());
}

#[test]
fn a_coding_change_entry_keeps_its_change_out_of_the_protein_list() {
    let entry: LitVarAutocompleteEntry = serde_json::from_value(serde_json::json!({
        "_id": "litvar@rs3088440##",
        "rsid": "rs3088440",
        "gene": ["CDKN2A", "CDKN2A-DT"],
        "name": "c.540C>T",
        "hgvs": "c.540C>T"
    }))
    .unwrap();
    let identity = entry.source_identity().expect("identity");
    assert!(identity.protein_changes.is_empty());
    assert_eq!(identity.coding_changes, vec!["c.540C>T".to_string()]);
}

#[test]
fn parses_the_publications_captures() {
    let cdkn2a: LitVarPublications = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::LITVAR2),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("publications_rs372266620.json"),
        true,
    )
    .unwrap();
    assert_eq!(cdkn2a.pmids.len(), 39);
    assert_eq!(cdkn2a.pmids[0], 29_936_259);
    assert_eq!(cdkn2a.pmids_count, Some(39));
    assert_eq!(cdkn2a.pmcids.len(), 34);

    let braf: LitVarPublications = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::LITVAR2),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("publications_rs113488022.json"),
        true,
    )
    .unwrap();
    assert_eq!(braf.pmids.len(), 100);
    assert_eq!(braf.pmids_count, Some(32_227));
    assert_eq!(braf.pmcids.len(), 100);
}

#[test]
fn an_empty_publications_reply_is_a_complete_answer() {
    let reply: LitVarPublications = serde_json::from_value(serde_json::json!({
        "pmids": [],
        "pmcids": [],
        "pmids_count": 0
    }))
    .unwrap();
    assert!(reply.pmids.is_empty());
    assert!(reply.pmcids.is_empty());
    assert_eq!(reply.pmids_count, Some(0));
}
