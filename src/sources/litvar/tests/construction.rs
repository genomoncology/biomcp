//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the
//! exact method / path / query that would be sent. Nothing is sent.

use crate::error::BioMcpError;
use crate::sources::HttpMethod;
use crate::sources::litvar::LitVarClient;

#[test]
fn autocomplete_plan_sets_query_path_and_parameter() {
    let plan = LitVarClient::autocomplete_plan(" CDKN2A A57V ").unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "variant/autocomplete/");
    assert_eq!(plan.query_value("query"), Some("CDKN2A A57V"));
}

#[test]
fn autocomplete_plan_rejects_empty_and_oversized_queries() {
    let empty = LitVarClient::autocomplete_plan("   ").unwrap_err();
    assert!(matches!(empty, BioMcpError::InvalidArgument(_)));
    assert!(empty.to_string().contains("required"));

    let oversized = LitVarClient::autocomplete_plan(&"v".repeat(257)).unwrap_err();
    assert!(matches!(oversized, BioMcpError::InvalidArgument(_)));
    assert!(oversized.to_string().contains("too long"));
}

#[test]
fn publications_plan_percent_encodes_the_variant_id() {
    let plan = LitVarClient::publications_plan("litvar@rs113488022##").unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    // A raw `#` would read as a URL fragment, so the id travels encoded the
    // way the provider's own examples spell it.
    assert_eq!(
        plan.path,
        "variant/get/litvar%40rs113488022%23%23/publications"
    );
    assert!(plan.query.is_empty());
}

#[test]
fn publications_plan_encodes_the_non_rsid_variant_id_shape() {
    let plan = LitVarClient::publications_plan("litvar@#673#p.V600F").unwrap();
    assert_eq!(
        plan.path,
        "variant/get/litvar%40%23673%23p.V600F/publications"
    );
}

#[test]
fn publications_plan_rejects_an_empty_variant_id() {
    let err = LitVarClient::publications_plan("  ").unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("required"));
}
