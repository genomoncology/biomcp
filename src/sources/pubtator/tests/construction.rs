//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::HttpMethod;

fn client_with_api_key(api_key: Option<&str>) -> PubTatorClient {
    PubTatorClient {
        client: crate::sources::shared_client().expect("shared client"),
        base: std::borrow::Cow::Borrowed("http://127.0.0.1"),
        api_key: api_key.map(str::to_string),
    }
}

#[test]
fn export_biocjson_plan_sets_pmids_and_optional_api_key() {
    let plan = PubTatorClient::export_biocjson_plan(22663011, None);

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "publications/export/biocjson");
    assert_eq!(plan.query_value("pmids"), Some("22663011"));
    assert!(!plan.has_query("api_key"));

    let keyed = PubTatorClient::export_biocjson_plan(22663011, Some(" test-key "));
    assert_eq!(keyed.query_value("api_key"), Some("test-key"));
}

#[test]
fn pubtator_detail_plan_adapts_the_biodata_path_and_adds_auth_only_in_transport() {
    let plan = PubTatorClient::publication_detail_plan(22663011, None).unwrap();
    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "publications/export/biocjson");
    assert_eq!(plan.query_value("pmids"), Some("22663011"));
    assert!(!plan.has_query("api_key"));

    let keyed = PubTatorClient::publication_detail_plan(22663011, Some(" test-key ")).unwrap();
    assert_eq!(keyed.query_value("pmids"), Some("22663011"));
    assert_eq!(keyed.query_value("api_key"), Some("test-key"));
    assert!(PubTatorClient::publication_detail_plan(0, None).is_err());
}

#[test]
fn autocomplete_plan_sets_query_and_validates_input() {
    let plan = PubTatorClient::entity_autocomplete_plan(" BRAF ", None).unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "entity/autocomplete/");
    assert_eq!(plan.query_value("query"), Some("BRAF"));

    assert!(matches!(
        PubTatorClient::entity_autocomplete_plan("   ", None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        PubTatorClient::entity_autocomplete_plan(&"x".repeat(257), None),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

#[test]
fn search_plan_sets_text_paging_sort_and_auth() {
    let plan = PubTatorClient::search_plan(" @GENE_BRAF ", 2, 25, Some(" date desc "), Some("key"))
        .unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "search/");
    assert_eq!(plan.query_value("text"), Some("@GENE_BRAF"));
    assert_eq!(plan.query_value("page"), Some("2"));
    assert_eq!(plan.query_value("size"), Some("25"));
    assert_eq!(plan.query_value("sort"), Some("date desc"));
    assert_eq!(plan.query_value("api_key"), Some("key"));
}

#[test]
fn search_plan_validates_query_and_page_size() {
    assert!(matches!(
        PubTatorClient::search_plan("   ", 1, 25, None, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        PubTatorClient::search_plan("BRAF", 0, 25, None, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        PubTatorClient::search_plan("BRAF", 1, 0, None, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        PubTatorClient::search_plan("BRAF", 1, 101, None, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

#[test]
fn legacy_request_plans_keep_article_contract_shape() {
    let client = client_with_api_key(Some("secret-ncbi-key"));

    let search: PubTatorSearchRequestPlan = client
        .search_request_plan("BRAF annotations", 1, 10, Some("date"))
        .expect("PubTatorSearchRequestPlan");
    assert_eq!(search.method, "GET");
    assert_eq!(search.path, "/search/");
    assert!(
        search
            .query_params
            .contains(&("text", "BRAF annotations".to_string()))
    );
    assert_eq!(search.cache_mode, "auth");
    assert_eq!(search.auth_mode, "authenticated");
    assert!(
        !search
            .query_params
            .iter()
            .any(|(_, value)| value.contains("secret-ncbi"))
    );

    let export: PubTatorExportRequestPlan = client.export_biocjson_request_plan(12345);
    assert_eq!(export.path, "/publications/export/biocjson");
    assert!(
        export
            .query_params
            .contains(&("pmids", "12345".to_string()))
    );

    let autocomplete: PubTatorAutocompleteRequestPlan = client
        .entity_autocomplete_request_plan("BRAF")
        .expect("PubTatorAutocompleteRequestPlan");
    assert_eq!(autocomplete.path, "/entity/autocomplete/");
    assert!(
        autocomplete
            .query_params
            .contains(&("query", "BRAF".to_string()))
    );

    let keyless = client_with_api_key(None);
    let keyless_plan = keyless.export_biocjson_request_plan(22663011);
    assert_eq!(keyless_plan.cache_mode, "default");
    assert_eq!(keyless_plan.auth_mode, "keyless");
}
