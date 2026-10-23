//! Lossless private storage for article rows shared by acquisition and pools.
use super::super::ArticleSearchResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredArticleRow {
    article: ArticleSearchResult,
    normalized_title: String,
    normalized_abstract: String,
    publication_type: Option<String>,
    source_local_position: usize,
}
impl StoredArticleRow {
    pub(crate) fn capture(article: &ArticleSearchResult) -> Self {
        Self {
            article: article.clone(),
            normalized_title: article.normalized_title.clone(),
            normalized_abstract: article.normalized_abstract.clone(),
            publication_type: article.publication_type.clone(),
            source_local_position: article.source_local_position,
        }
    }
    pub(crate) fn article(&self) -> &ArticleSearchResult {
        &self.article
    }
    pub(crate) fn restore(self) -> ArticleSearchResult {
        let mut article = self.article;
        article.normalized_title = self.normalized_title;
        article.normalized_abstract = self.normalized_abstract;
        article.publication_type = self.publication_type;
        article.source_local_position = self.source_local_position;
        article
    }
}
