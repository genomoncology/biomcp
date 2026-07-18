//! Request construction and URL-validation tests. Pure: build request plans
//! and validate URLs without sending. No network.

use crate::sources::{HttpMethod, RequestBody};

use super::*;

#[test]
fn article_plan_uses_article_id_path() {
    let plan = FigshareClient::article_plan(&article_reference());

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "v2/articles/22474820");
}

#[test]
fn search_articles_plan_uses_expected_post_body() {
    let plan = FigshareClient::search_articles_plan(" 10.1000/example ").expect("non-empty search");

    assert_eq!(plan.method, HttpMethod::Post);
    assert_eq!(plan.path, "v2/articles/search");
    match plan.body {
        RequestBody::Json(value) => {
            assert_eq!(value["search_for"], "10.1000/example");
            assert_eq!(value["page_size"], FIGSHARE_SEARCH_PAGE_SIZE);
        }
        other => panic!("expected JSON body, got {other:?}"),
    }
}

#[test]
fn search_articles_plan_skips_empty_query() {
    let plan = FigshareClient::search_articles_plan("   ");

    assert_eq!(plan, None);
}

#[test]
fn parses_aacr_public_article_url_with_file_id() {
    let parsed = parse_figshare_article_url(
        "https://aacr.figshare.com/articles/journal_contribution/Foo/22474820?file=39926318",
    )
    .unwrap();

    assert_eq!(parsed.article_id, 22474820);
    assert_eq!(parsed.file_id, Some(39926318));
}

#[test]
fn parses_public_article_url_with_file_path_id() {
    let parsed = parse_figshare_article_url(
        "https://figshare.com/articles/dataset/Foo/22474820/files/39926318",
    )
    .unwrap();

    assert_eq!(parsed.article_id, 22474820);
    assert_eq!(parsed.file_id, Some(39926318));
}

#[test]
fn parses_versioned_public_article_url_with_file_path_id() {
    let parsed = parse_figshare_article_url(
        "https://aacr.figshare.com/articles/journal_contribution/Foo/22474820/1/files/39926318.pdf",
    )
    .unwrap();

    assert_eq!(parsed.article_id, 22474820);
    assert_eq!(parsed.file_id, Some(39926318));
}

#[test]
fn parses_api_article_url() {
    let parsed =
        parse_figshare_article_url("https://api.figshare.com/v2/articles/22474820").unwrap();

    assert_eq!(parsed.article_id, 22474820);
    assert_eq!(parsed.file_id, None);
}

#[test]
fn rejects_non_figshare_urls_and_unsafe_names() {
    assert!(parse_figshare_article_url("https://example.org/file.pdf").is_none());
    assert!(safe_filename("../secret.pdf").is_none());
    assert!(safe_filename("nested/secret.pdf").is_none());
    assert_eq!(
        safe_filename(" supplement.pdf ").as_deref(),
        Some("supplement.pdf")
    );
}

#[test]
fn production_download_url_validation_rejects_unsafe_targets() {
    let client = production_client();

    for raw in [
        "http://ndownloader.figshare.com/files/1",
        "https://localhost/files/1",
        "https://127.0.0.1/files/1",
        "https://10.0.0.5/files/1",
        "https://169.254.169.254/files/1",
        "file:///tmp/asset.pdf",
        "https://example.org/files/1",
    ] {
        let err = client.validate_download_url(raw).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("BioMCP source"));
        assert!(message.to_ascii_lowercase().contains("retry"));
        assert!(!message.contains(raw));
    }
}

#[test]
fn production_download_url_validation_allows_figshare_https_hosts() {
    let client = production_client();

    for raw in [
        "https://figshare.com/files/1",
        "https://ndownloader.figshare.com/files/1",
        "https://api.figshare.com/v2/articles/1/files/2",
    ] {
        assert!(
            client.validate_download_url(raw).is_ok(),
            "{raw} should be allowed"
        );
    }
}
