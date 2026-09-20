use crate::entities::article::Article;
use crate::entities::article::backends::VariantArticleProviderUnit;
use crate::entities::article::variant_search::VariantArticleExecutionContext;
use crate::error::BioMcpError;
use crate::sources::europepmc::{EuropePmcClient, EuropePmcResult, EuropePmcSearchResponse};
use crate::sources::pubtator::PubTatorClient;
use crate::transform;

use super::{article_not_found, is_pubtator_lag_error, resolve_article_from_pmid};

pub(in crate::entities::article) fn first_europepmc_hit(
    search: EuropePmcSearchResponse,
) -> Option<EuropePmcResult> {
    search
        .result_list
        .and_then(|list| list.result.into_iter().next())
}

pub(in crate::entities::article) async fn resolve_article_from_pmid_with_context(
    pmid: u32,
    not_found_id: &str,
    suggestion_id: &str,
    pubtator: &PubTatorClient,
    europe: &EuropePmcClient,
    europe_hint: Option<&EuropePmcResult>,
    execution: Option<&VariantArticleExecutionContext>,
) -> Result<Article, BioMcpError> {
    let Some(execution) = execution else {
        return resolve_article_from_pmid(
            pmid,
            not_found_id,
            suggestion_id,
            pubtator,
            europe,
            None,
        )
        .await;
    };
    resolve_variant_article_from_pmid(pmid, not_found_id, suggestion_id, europe_hint, execution)
        .await
}

pub(in crate::entities::article) async fn resolve_variant_article_from_pmid(
    pmid: u32,
    not_found_id: &str,
    suggestion_id: &str,
    europe_hint: Option<&EuropePmcResult>,
    execution: &VariantArticleExecutionContext,
) -> Result<Article, BioMcpError> {
    let unit = execution
        .begin_provider_unit("enrichment", "pubtator")
        .await
        .ok_or_else(|| BioMcpError::SourceUnavailable {
            source_name: "variant article work budget".into(),
            reason: "item work budget exhausted".into(),
            suggestion: "Retry with a narrower request".into(),
        })?;
    let result = match PubTatorClient::new_with_deadline(execution.deadline()).await {
        Ok(client) => client.export_biocjson(pmid).await,
        Err(error) => Err(error),
    };
    match result {
        Ok(response) => {
            let Some(doc) = response.documents.into_iter().next() else {
                let error = article_not_found(not_found_id, suggestion_id);
                unit.record_error(&error);
                return Err(error);
            };
            let mut article = transform::article::retained_from_pubtator_document(&doc);
            unit.commit("ok", 1, || {
                article.annotations = transform::article::retained_extract_annotations(&doc);
            });
            if let Some(hit) = europe_hint {
                transform::article::retained_merge_europepmc_metadata(&mut article, hit);
            } else {
                execution.add_route_unit("enrichment", "europepmc");
                match variant_europepmc_hit(pmid, execution).await {
                    Ok(Some((hit, europe_unit))) => {
                        europe_unit.commit("ok", 1, || {
                            transform::article::retained_merge_europepmc_metadata(
                                &mut article,
                                &hit,
                            );
                        });
                    }
                    Ok(None) => {}
                    Err(error) => crate::error::warn_external_failure(
                        &error,
                        crate::error::SourceProvider::EUROPE_PMC,
                        "enrich completed PubTator article",
                    ),
                }
            }
            Ok(article)
        }
        Err(error) if is_pubtator_lag_error(&error) => {
            unit.record_error(&error);
            let (hit, europe_unit) = match europe_hint.cloned() {
                Some(hit) => return Ok(article_from_europepmc_fallback(&hit)),
                None => {
                    execution.add_route_unit("enrichment", "europepmc");
                    variant_europepmc_hit(pmid, execution)
                        .await?
                        .ok_or_else(|| article_not_found(not_found_id, suggestion_id))?
                }
            };
            Ok(europe_unit.commit("ok", 1, || article_from_europepmc_fallback(&hit)))
        }
        Err(error) => {
            unit.record_error(&error);
            Err(error)
        }
    }
}

async fn variant_europepmc_hit<'a>(
    pmid: u32,
    execution: &'a VariantArticleExecutionContext,
) -> Result<Option<(EuropePmcResult, VariantArticleProviderUnit<'a>)>, BioMcpError> {
    let Some(unit) = execution
        .begin_provider_unit("enrichment", "europepmc")
        .await
    else {
        return Ok(None);
    };
    let result = match EuropePmcClient::new_with_deadline(execution.deadline()).await {
        Ok(client) => client.search_by_pmid(&pmid.to_string()).await,
        Err(error) => Err(error),
    };
    match result {
        Ok(response) => match first_europepmc_hit(response) {
            Some(hit) => Ok(Some((hit, unit))),
            None => {
                unit.record("ok", 1);
                Ok(None)
            }
        },
        Err(error) => {
            unit.record_error(&error);
            Err(error)
        }
    }
}

pub(in crate::entities::article) fn article_from_europepmc_fallback(
    hit: &EuropePmcResult,
) -> Article {
    let mut article = transform::article::retained_from_europepmc_result(hit);
    article.pubtator_fallback = true;
    article
}
