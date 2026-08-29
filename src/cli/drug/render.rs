//! Drug card rendering and search-JSON serialization helpers.

use super::dispatch::resolve_drug_get_region;
use crate::cli::CommandOutcome;
use crate::entities::drug::DrugRegion;

pub(super) async fn render_drug_card_outcome(
    name: &str,
    sections: &[String],
    region: Option<DrugRegion>,
    raw_label: bool,
    json_output: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    let effective_region = resolve_drug_get_region(sections, region);
    match crate::entities::drug::get_with_region(
        name,
        sections,
        effective_region,
        region.is_some(),
        raw_label,
    )
    .await
    {
        Ok(drug) => Ok(CommandOutcome::stdout(render_loaded_card(
            &drug,
            sections,
            effective_region,
            raw_label,
            json_output,
        )?)),
        Err(err @ crate::error::BioMcpError::NotFound { .. }) => {
            if let Some(outcome) = super::super::try_alias_fallback_outcome(
                name,
                crate::entities::discover::DiscoverType::Drug,
                json_output || alias_suggestions_as_json,
            )
            .await?
            {
                Ok(outcome)
            } else {
                Err(err.into())
            }
        }
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn render_loaded_card(
    drug: &crate::entities::drug::Drug,
    sections: &[String],
    effective_region: DrugRegion,
    raw_label: bool,
    json_output: bool,
) -> anyhow::Result<String> {
    if json_output {
        Ok(crate::render::json::to_entity_json_with_workflow(
            drug,
            crate::render::markdown::drug_evidence_urls(drug),
            crate::render::markdown::related_drug(drug),
            crate::render::provenance::drug_section_sources(drug),
            drug_pharmacogene_workflow(drug)?,
        )?)
    } else {
        Ok(crate::render::markdown::drug_markdown_with_region(
            drug,
            sections,
            effective_region,
            raw_label,
        )?)
    }
}

fn drug_pharmacogene_workflow(
    drug: &crate::entities::drug::Drug,
) -> Result<Option<crate::workflow_ladders::WorkflowMeta>, crate::error::BioMcpError> {
    (!drug.name.trim().is_empty())
        .then(|| {
            crate::workflow_ladders::meta_for(
                crate::workflow_ladders::Workflow::PharmacogeneCumulative,
            )
        })
        .transpose()
}

#[derive(serde::Serialize)]
pub(super) struct DrugSearchRegionBucket<T: serde::Serialize> {
    region: &'static str,
    pagination: crate::cli::PaginationMeta,
    count: usize,
    results: Vec<DrugSearchView<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    continuation_command: Option<String>,
}

#[derive(serde::Serialize)]
struct DrugSearchView<T: serde::Serialize> {
    #[serde(flatten)]
    row: T,
    match_kind: &'static str,
}

#[derive(Default, serde::Serialize)]
pub(super) struct DrugSearchJsonRegions {
    #[serde(skip_serializing_if = "Option::is_none")]
    us: Option<DrugSearchRegionBucket<crate::entities::drug::DrugSearchResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eu: Option<DrugSearchRegionBucket<crate::entities::drug::EmaDrugSearchResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    who: Option<DrugSearchRegionBucket<crate::entities::drug::WhoPrequalificationSearchResult>>,
}

#[derive(serde::Serialize)]
pub(super) struct DrugSearchJsonResponse {
    region: &'static str,
    regions: DrugSearchJsonRegions,
    #[serde(skip_serializing_if = "Option::is_none")]
    _meta: Option<crate::cli::SearchJsonMeta>,
}

pub(super) fn bucket_from_page<T: serde::Serialize>(
    page: crate::entities::drug::RankedDrugSearchPage<T>,
    region: &'static str,
    query: Option<&str>,
    offset: usize,
    limit: usize,
) -> DrugSearchRegionBucket<T> {
    let count = page.results.len();
    let pagination = crate::cli::PaginationMeta::offset(offset, limit, count, page.total);
    let continuation_command =
        drug_region_continuation(query, region, offset, limit, count, pagination.has_more);
    let results = page
        .results
        .into_iter()
        .zip(page.match_kinds)
        .map(|(row, kind)| DrugSearchView {
            match_kind: kind.as_str(),
            row,
        })
        .collect::<Vec<_>>();
    DrugSearchRegionBucket {
        region,
        pagination,
        count,
        results,
        continuation_command,
    }
}

pub(super) fn drug_region_continuation(
    query: Option<&str>,
    region: &'static str,
    offset: usize,
    limit: usize,
    returned: usize,
    has_more: bool,
) -> Option<String> {
    has_more.then(|| {
        format!(
            "biomcp search drug{} --region {region} --limit {limit} --offset {}",
            query
                .map(|value| format!(
                    " --query {}",
                    crate::render::markdown::shell_quote_arg(value)
                ))
                .unwrap_or_default(),
            offset.saturating_add(returned)
        )
    })
}

pub(super) fn drug_search_json(
    page_with_region: crate::entities::drug::DrugSearchPageWithRegion,
    requested_name: Option<&str>,
    offset: usize,
    limit: usize,
    workflow: Option<crate::workflow_ladders::WorkflowMeta>,
) -> anyhow::Result<String> {
    let response = match page_with_region {
        crate::entities::drug::DrugSearchPageWithRegion::Us(page) => {
            let next_commands = crate::render::markdown::search_next_commands_drug_regions(
                requested_name,
                Some(&page.results),
                None,
                None,
            );
            DrugSearchJsonResponse {
                region: crate::entities::drug::DrugRegion::Us.as_str(),
                regions: DrugSearchJsonRegions {
                    us: Some(bucket_from_page(page, "us", requested_name, offset, limit)),
                    ..Default::default()
                },
                _meta: crate::cli::search_meta_with_workflow(next_commands, None, workflow.clone()),
            }
        }
        crate::entities::drug::DrugSearchPageWithRegion::Eu(page) => {
            let next_commands = crate::render::markdown::search_next_commands_drug_regions(
                requested_name,
                None,
                Some(&page.results),
                None,
            );
            DrugSearchJsonResponse {
                region: crate::entities::drug::DrugRegion::Eu.as_str(),
                regions: DrugSearchJsonRegions {
                    eu: Some(bucket_from_page(page, "eu", requested_name, offset, limit)),
                    ..Default::default()
                },
                _meta: crate::cli::search_meta_with_workflow(next_commands, None, workflow.clone()),
            }
        }
        crate::entities::drug::DrugSearchPageWithRegion::Who(page) => {
            let next_commands = crate::render::markdown::search_next_commands_drug_regions(
                requested_name,
                None,
                None,
                Some(&page.results),
            );
            DrugSearchJsonResponse {
                region: crate::entities::drug::DrugRegion::Who.as_str(),
                regions: DrugSearchJsonRegions {
                    who: Some(bucket_from_page(page, "who", requested_name, offset, limit)),
                    ..Default::default()
                },
                _meta: crate::cli::search_meta_with_workflow(next_commands, None, workflow.clone()),
            }
        }
        crate::entities::drug::DrugSearchPageWithRegion::All { us, eu, who } => {
            let next_commands = crate::render::markdown::search_next_commands_drug_regions(
                requested_name,
                Some(&us.results),
                Some(&eu.results),
                Some(&who.results),
            );
            DrugSearchJsonResponse {
                region: crate::entities::drug::DrugRegion::All.as_str(),
                regions: DrugSearchJsonRegions {
                    us: Some(bucket_from_page(us, "us", requested_name, offset, limit)),
                    eu: Some(bucket_from_page(eu, "eu", requested_name, offset, limit)),
                    who: Some(bucket_from_page(who, "who", requested_name, offset, limit)),
                },
                _meta: crate::cli::search_meta_with_workflow(next_commands, None, workflow),
            }
        }
    };

    crate::render::json::to_pretty(&response).map_err(Into::into)
}
