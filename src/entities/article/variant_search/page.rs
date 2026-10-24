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
    verification: super::VariantArticleVerificationOptions,
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
    if verification.verify_identity
        && !verification.confirmed_only
        && candidates.iter().any(|candidate| {
            candidate
                .identity
                .as_ref()
                .is_none_or(|identity| identity.incomplete)
        })
    {
        let canonical = outcome
            .response
            .canonical_equivalence
            .clone()
            .unwrap_or_else(super::unattempted_canonical_equivalence);
        super::materialize_identity_verification_plan(
            &candidates,
            &outcome.response.requested_variant,
            0,
            candidates.len(),
            false,
            execution,
        );
        execution.reserve_identity_verification_through(candidates.len());
        let verified = verify_candidates(
            &mut candidates,
            &outcome.response.requested_variant,
            &canonical,
            execution,
        )
        .await;
        if verified.incomplete {
            outcome.response.complete = false;
            outcome.response.truncated = true;
        }
        let routes = [
            "identity_verification",
            "clingen_ldh_medium",
            "clingen_ldh_direct",
        ];
        let statuses = super::terminal_reconciled_statuses(execution, &execution.events());
        outcome
            .response
            .source_status
            .retain(|status| !routes.contains(&status.route.as_str()));
        outcome.response.source_status.extend(
            statuses
                .into_iter()
                .filter(|status| routes.contains(&status.route.as_str())),
        );
        if let Some(plan) = outcome.response.debug_plan.as_mut() {
            for candidate in &candidates {
                let identifier =
                    super::super::candidates::stable_article_identifier(&candidate.row);
                for trace in plan
                    .candidate_trace
                    .candidates
                    .iter_mut()
                    .filter(|trace| trace.identifier == identifier)
                {
                    trace.verification_disposition = candidate
                        .identity
                        .as_ref()
                        .map_or("unverified", |identity| identity.status)
                        .into();
                }
            }
            plan.verification = Some(super::verification_plan(
                &outcome.response.requested_variant,
                super::PUBTATOR_EXPORT_FULLTEXT_TEMPLATE_VERSION,
                &verified.responses,
                &verified.contents,
            ));
        }
    }
    let mut statuses = super::enrich_candidates(&mut candidates, execution).await;
    for (row, candidate) in outcome.response.results.iter_mut().zip(candidates) {
        row.article = candidate.row;
        row.identity = candidate.identity;
        if row.match_reason == "unverified_alias_match"
            && row
                .identity
                .as_ref()
                .is_some_and(|identity| identity.status == "confirmed")
        {
            row.match_reason = "exact_variant";
        }
        row.metadata_status = statuses.remove(&row.article.pmid);
    }
    outcome
}

pub(super) struct VerificationCapture {
    pub(super) incomplete: bool,
    pub(super) responses: Vec<String>,
    pub(super) contents: Vec<String>,
}

pub(super) async fn verify_candidates(
    candidates: &mut [super::ArticleCandidate],
    requested: &crate::entities::variant::RequestedVariantIdentity,
    canonical: &super::CanonicalEquivalence,
    execution: &super::VariantArticleExecutionContext,
) -> VerificationCapture {
    use super::{
        canonical_content_subset, canonical_response_subset, combine_identities,
        verify_captured_abstract, verify_paper_anchors, verify_pubtator,
    };
    let pending = candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            candidate
                .identity
                .as_ref()
                .is_none_or(|identity| identity.incomplete)
                .then_some(index)
        })
        .collect::<Vec<_>>();
    let mut verification_incomplete = false;
    let mut verification_response_subsets = Vec::new();
    let mut verification_content_subsets = Vec::new();
    for candidate in candidates.iter_mut() {
        if candidate
            .identity
            .as_ref()
            .is_some_and(|identity| !identity.incomplete)
        {
            continue;
        }
        let captured = verify_captured_abstract(
            requested,
            candidate
                .row
                .abstract_snippet
                .as_deref()
                .unwrap_or(&candidate.row.normalized_abstract),
        );
        if captured.status != "unverified" {
            verification_content_subsets.push(canonical_content_subset(&captured));
            candidate.identity = Some(captured);
            continue;
        }
        let incomplete = match candidate.row.pmid.parse::<u64>() {
            Ok(pmid) => {
                let Some(unit) = execution
                    .begin_provider_unit("identity_verification", "pubtator")
                    .await
                else {
                    verification_incomplete = true;
                    candidate.identity = Some(combine_identities(
                        captured.clone(),
                        verify_pubtator(
                            requested,
                            &pmid.to_string(),
                            &crate::sources::pubtator::PubTatorExportResponse {
                                documents: Vec::new(),
                            },
                            true,
                        ),
                    ));
                    continue;
                };
                let response = match crate::sources::pubtator::PubTatorClient::new_with_deadline(
                    execution.deadline(),
                )
                .await
                {
                    Ok(client) => client.export_biocjson_fulltext(pmid).await,
                    Err(error) => Err(error),
                };
                match response {
                    Ok(response) => {
                        verification_response_subsets.push(canonical_response_subset(&response));
                        let mut fetched =
                            verify_pubtator(requested, &pmid.to_string(), &response, false);
                        unit.commit("ok", 1, || {});
                        verification_incomplete |= verify_paper_anchors(
                            requested,
                            &pmid.to_string(),
                            &response,
                            canonical,
                            execution,
                            &mut fetched,
                        )
                        .await;
                        let identity = combine_identities(captured.clone(), fetched);
                        verification_content_subsets.push(canonical_content_subset(&identity));
                        candidate.identity = Some(identity);
                        false
                    }
                    Err(error) => {
                        unit.record_error(&error);
                        true
                    }
                }
            }
            Err(_) => true,
        };
        verification_incomplete |= incomplete;
        if candidate.identity.is_none() {
            candidate.identity = Some(combine_identities(
                captured.clone(),
                verify_pubtator(
                    requested,
                    &candidate.row.pmid,
                    &crate::sources::pubtator::PubTatorExportResponse {
                        documents: Vec::new(),
                    },
                    incomplete,
                ),
            ));
        }
    }

    let mut pending_candidates = pending
        .iter()
        .map(|index| candidates[*index].clone())
        .collect::<Vec<_>>();
    if !pending_candidates.is_empty() {
        verification_incomplete |=
            super::add_ldh_observations(&mut pending_candidates, requested, canonical, execution)
                .await;
        for (index, candidate) in pending.into_iter().zip(pending_candidates) {
            candidates[index] = candidate;
        }
    }
    VerificationCapture {
        incomplete: verification_incomplete,
        responses: verification_response_subsets,
        contents: verification_content_subsets,
    }
}
