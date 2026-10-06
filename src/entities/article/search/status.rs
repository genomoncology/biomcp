//! Source-status assembly for article search: the Semantic Scholar
//! enrichment tracker and the shared degraded/timeout status builders.

use super::FEDERATED_ARTICLE_SOURCE_TIMEOUT;
use super::{ArticleSource, ArticleSourceAvailability, ArticleSourceStatus};

#[derive(Default)]
pub(super) struct SemanticScholarStatusTracker {
    auth_mode: Option<crate::sources::semantic_scholar::SemanticScholarAuthMode>,
    succeeded: bool,
    failed: bool,
    message: Option<String>,
}

impl SemanticScholarStatusTracker {
    pub(super) fn record(&mut self, status: ArticleSourceStatus) {
        if self.auth_mode.is_none() {
            self.auth_mode = status.auth_mode;
        }
        match status.status {
            Some(ArticleSourceAvailability::Ok) => self.succeeded = true,
            Some(ArticleSourceAvailability::Degraded) => {
                self.succeeded = true;
                self.failed = true;
            }
            Some(ArticleSourceAvailability::Unavailable) => self.failed = true,
            Some(ArticleSourceAvailability::Skipped) | None => {}
        }
        if status.message.is_some() {
            self.message = status.message;
        }
    }

    pub(super) fn finish(self) -> Vec<ArticleSourceStatus> {
        let status = if self.failed && self.succeeded {
            ArticleSourceAvailability::Degraded
        } else if self.failed {
            ArticleSourceAvailability::Unavailable
        } else {
            ArticleSourceAvailability::Ok
        };
        vec![ArticleSourceStatus {
            source: ArticleSource::SemanticScholar,
            enabled: true,
            auth_mode: self.auth_mode,
            status: Some(status),
            message: self.failed.then_some(
                self.message
                    .unwrap_or_else(|| "Semantic Scholar unavailable".to_string()),
            ),
        }]
    }
}

pub(super) fn source_provider(source: ArticleSource) -> crate::error::SourceProvider {
    match source {
        ArticleSource::PubTator => crate::error::SourceProvider::PUBTATOR3,
        ArticleSource::EuropePmc => crate::error::SourceProvider::EUROPE_PMC,
        ArticleSource::PubMed => crate::error::SourceProvider::PUBMED,
        ArticleSource::SemanticScholar => crate::error::SourceProvider::SEMANTIC_SCHOLAR,
        ArticleSource::LitSense2 => crate::error::SourceProvider::LITSENSE2,
    }
}

pub(super) fn source_degraded_status(
    source: ArticleSource,
    message: String,
) -> ArticleSourceStatus {
    ArticleSourceStatus {
        source,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Degraded),
        message: Some(message),
    }
}

pub(super) fn timed_out_source_status(source: ArticleSource) -> ArticleSourceStatus {
    source_degraded_status(
        source,
        format!(
            "{} timed out after {}s",
            source.display_name(),
            FEDERATED_ARTICLE_SOURCE_TIMEOUT.as_secs()
        ),
    )
}
