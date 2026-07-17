//! Shared article test helpers used by sidecar test modules.

#[allow(unused_imports)]
pub(super) use super::{
    ARTICLE_BATCH_MAX_IDS, AnnotationCount, Article, ArticleAnnotations, ArticleAuthorCompleteness,
    ArticleBatchEntitySummary, ArticleBatchItem, ArticlePubMedRescueKind, ArticleRankingMode,
    ArticleRankingOptions, ArticleSearchFilters, ArticleSearchResult, ArticleSemanticScholar,
    ArticleSemanticScholarPdf, ArticleSort, ArticleSource, ArticleSourceFilter,
};
#[allow(unused_imports)]
pub(super) use crate::entities::SearchPage;
#[allow(unused_imports)]
pub(super) use crate::error::BioMcpError;
#[allow(unused_imports)]
pub(super) use crate::sources::europepmc::EuropePmcSort;

pub(super) struct TestEnv {
    previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

impl TestEnv {
    pub(super) fn new() -> Self {
        Self {
            previous: Vec::new(),
        }
    }

    pub(super) fn set(&mut self, key: &'static str, value: impl AsRef<std::ffi::OsStr>) {
        if !self.previous.iter().any(|(existing, _)| *existing == key) {
            self.previous.push((key, std::env::var_os(key)));
        }
        // SAFETY: article tests that mutate provider variables use the same serial-test key.
        unsafe { std::env::set_var(key, value) };
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        for (key, previous) in self.previous.drain(..).rev() {
            // SAFETY: article tests that mutate provider variables use the same serial-test key.
            unsafe {
                if let Some(value) = previous {
                    std::env::set_var(key, value);
                } else {
                    std::env::remove_var(key);
                }
            }
        }
    }
}

pub(super) enum TestHttpReply {
    Bytes(Vec<u8>),
}

pub(super) struct TestHttpFixture {
    pub(super) base: String,
    task: tokio::task::JoinHandle<()>,
}

impl TestHttpFixture {
    pub(super) async fn spawn(
        handler: impl Fn(&str) -> TestHttpReply + Send + Sync + 'static,
    ) -> Self {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind article fixture");
        let address = listener.local_addr().expect("article fixture address");
        let handler = std::sync::Arc::new(handler);
        let task = tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let handler = handler.clone();
                tokio::spawn(async move {
                    let mut request = vec![0_u8; 16 * 1024];
                    let length = stream.read(&mut request).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&request[..length]);
                    let TestHttpReply::Bytes(response) = handler(&request);
                    let _ = stream.write_all(&response).await;
                });
            }
        });
        Self {
            base: format!("http://{address}"),
            task,
        }
    }
}

impl Drop for TestHttpFixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(super) fn test_http_response(status: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

pub(super) fn empty_filters() -> ArticleSearchFilters {
    ArticleSearchFilters {
        gene: None,
        gene_anchored: false,
        disease: None,
        drug: None,
        variant: None,
        author: None,
        keyword: None,
        date_from: None,
        date_to: None,
        article_type: None,
        journal: None,
        open_access: false,
        no_preprints: false,
        exclude_retracted: false,
        max_per_source: None,
        sort: ArticleSort::Relevance,
        ranking: ArticleRankingOptions::default(),
    }
}

pub(super) fn row(pmid: &str, source: ArticleSource) -> ArticleSearchResult {
    row_with(pmid, source, Some("2025-01-01"), Some(1), Some(false))
}

pub(super) fn row_with(
    pmid: &str,
    source: ArticleSource,
    date: Option<&str>,
    citation_count: Option<u64>,
    is_retracted: Option<bool>,
) -> ArticleSearchResult {
    ArticleSearchResult {
        pmid: pmid.to_string(),
        pmcid: None,
        doi: None,
        arxiv_id: None,
        semantic_scholar_id: None,
        title: format!("title-{pmid}"),
        journal: Some("Journal".into()),
        date: date.map(str::to_string),
        first_index_date: None,
        citation_count,
        influential_citation_count: None,
        source,
        matched_sources: vec![source],
        score: (source == ArticleSource::PubTator).then_some(42.0),
        is_retracted,
        abstract_snippet: None,
        ranking: None,
        normalized_title: format!("title-{pmid}"),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    }
}
