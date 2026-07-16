use super::*;

mod construction;
mod parsing;

fn production_client() -> FigshareClient {
    FigshareClient {
        client: crate::sources::test_client().unwrap(),
        base: Cow::Borrowed(FIGSHARE_BASE),
        provider_policy: ProviderUrlPolicy::for_consumer(
            ProviderUrlConsumer::FigshareDownload,
            None,
        )
        .unwrap(),
    }
}

fn article_reference() -> FigshareArticleRef {
    FigshareArticleRef {
        article_id: 22474820,
        file_id: Some(39926318),
    }
}

fn article_response_bytes() -> Vec<u8> {
    br#"{
        "id": 22474820,
        "url_api": "https://api.figshare.com/v2/articles/22474820",
        "url_public_html": "https://aacr.figshare.com/articles/journal_contribution/Foo/22474820",
        "license": {"name": "CC BY 4.0", "url": "https://creativecommons.org/licenses/by/4.0/"},
        "files": [
            {"id": 1, "name": "other.txt", "size": 5, "download_url": "https://ndownloader.figshare.com/files/1"},
            {"id": 39926318, "name": "figshare-supplement.pdf", "size": 8, "md5": "0123456789abcdef0123456789abcdef", "mimetype": "application/pdf", "download_url": "https://ndownloader.figshare.com/files/39926318"},
            {"id": 3, "name": "../unsafe.pdf", "download_url": "https://ndownloader.figshare.com/files/3"}
        ]
    }"#
    .to_vec()
}

fn search_response_bytes() -> Vec<u8> {
    br#"[
        {
            "id": 22474817,
            "title": " Example ",
            "doi": "10.1000/example",
            "url_api": "https://api.figshare.com/v2/articles/22474817",
            "url_public_html": "https://figshare.com/articles/example/22474817"
        },
        {"title": "missing id"}
    ]"#
    .to_vec()
}
