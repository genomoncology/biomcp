//! Bound continuations only slice their immutable pool.
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

pub(super) const ORDERING_VERSION: u32 = super::rank::RANKING_VERSION;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Continuation {
    pub(super) snapshot: String,
    pub(super) query: String,
    pub(super) ordering: u32,
    pub(super) offset: usize,
}

impl Continuation {
    pub(super) fn encode(snapshot: &str, query: &str, offset: usize) -> String {
        let value = Self {
            snapshot: snapshot.into(),
            query: query.into(),
            ordering: ORDERING_VERSION,
            offset,
        };
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&value).expect("continuation is serializable"))
    }

    pub(super) fn decode(token: &str, query: &str, offset: usize) -> Result<Self, BioMcpError> {
        let invalid = || {
            BioMcpError::InvalidArgument(
                "Invalid article page token; restart at offset 0 without a token".into(),
            )
        };
        if token.len() > 2048 {
            return Err(invalid());
        }
        let bytes = URL_SAFE_NO_PAD.decode(token).map_err(|_| invalid())?;
        let value: Self = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        if value.ordering != ORDERING_VERSION || value.query != query || value.snapshot.is_empty() {
            return Err(BioMcpError::InvalidArgument("Article page token conflicts with the request options; restart at offset 0 without a token".into()));
        }
        if offset != 0 && offset != value.offset {
            return Err(BioMcpError::InvalidArgument(
                "--offset conflicts with the article page token".into(),
            ));
        }
        Ok(value)
    }
}

pub(super) fn unavailable() -> BioMcpError {
    BioMcpError::InvalidArgument(
        "Article continuation expired or is unavailable; restart at offset 0 without a token"
            .into(),
    )
}

/// Display hydration happens after slicing and cannot change rank or membership.
pub(super) async fn enrich_visible(
    mut outcome: super::VariantArticleOutcome,
    execution: &super::VariantArticleExecutionContext,
) -> super::VariantArticleOutcome {
    let mut candidates = outcome
        .response
        .results
        .iter()
        .map(|row| {
            let mut candidate =
                super::super::candidates::article_candidate_from_row(row.article.clone());
            candidate.identity = row.identity.clone();
            candidate.variant_provenance = row.provenance.clone();
            candidate
        })
        .collect::<Vec<_>>();
    let mut statuses = super::enrich_candidates(&mut candidates, execution).await;
    for (row, candidate) in outcome.response.results.iter_mut().zip(candidates) {
        row.article = candidate.row;
        row.metadata_status = statuses.remove(&row.article.pmid);
    }
    outcome
}
