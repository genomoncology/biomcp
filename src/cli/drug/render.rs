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
        Ok(drug) => {
            let text = if json_output {
                let workflow = drug_pharmacogene_workflow(&drug, name).await?;
                crate::render::json::to_entity_json_with_workflow(
                    &drug,
                    crate::render::markdown::drug_evidence_urls(&drug),
                    crate::render::markdown::related_drug(&drug),
                    crate::render::provenance::drug_section_sources(&drug),
                    workflow,
                )?
            } else {
                crate::render::markdown::drug_markdown_with_region(
                    &drug,
                    sections,
                    effective_region,
                    raw_label,
                )?
            };
            Ok(CommandOutcome::stdout(text))
        }
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

async fn drug_pharmacogene_workflow(
    drug: &crate::entities::drug::Drug,
    requested_name: &str,
) -> Result<Option<crate::workflow_ladders::WorkflowMeta>, crate::error::BioMcpError> {
    const CPIC_GENE_THRESHOLD: usize = 3;

    let has_signal = match crate::workflow_ladders::probe_workflow(
        crate::workflow_ladders::Workflow::PharmacogeneCumulative,
        Box::pin(async {
            let primary_count = crate::entities::pgx::distinct_actionable_cpic_gene_count_for_drug(
                &drug.name,
                CPIC_GENE_THRESHOLD,
            )
            .await?;
            if primary_count >= CPIC_GENE_THRESHOLD {
                return Ok(true);
            }

            let requested = requested_name.trim();
            if requested.is_empty() || requested.eq_ignore_ascii_case(drug.name.trim()) {
                return Ok(false);
            }

            let requested_count =
                crate::entities::pgx::distinct_actionable_cpic_gene_count_for_drug(
                    requested,
                    CPIC_GENE_THRESHOLD,
                )
                .await?;
            Ok(requested_count >= CPIC_GENE_THRESHOLD)
        }),
    )
    .await?
    {
        crate::workflow_ladders::WorkflowProbeOutcome::Triggered(meta) => Some(meta),
        crate::workflow_ladders::WorkflowProbeOutcome::NotTriggered
        | crate::workflow_ladders::WorkflowProbeOutcome::Unavailable => None,
    };

    Ok(has_signal)
}

#[derive(serde::Serialize)]
pub(super) struct DrugSearchRegionBucket<T: serde::Serialize> {
    pagination: crate::cli::PaginationMeta,
    count: usize,
    results: Vec<T>,
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
    page: crate::entities::SearchPage<T>,
    offset: usize,
    limit: usize,
) -> DrugSearchRegionBucket<T> {
    let count = page.results.len();
    DrugSearchRegionBucket {
        pagination: crate::cli::PaginationMeta::offset(offset, limit, count, page.total),
        count,
        results: page.results,
    }
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
                    us: Some(bucket_from_page(page, offset, limit)),
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
                    eu: Some(bucket_from_page(page, offset, limit)),
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
                    who: Some(bucket_from_page(page, offset, limit)),
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
                    us: Some(bucket_from_page(us, offset, limit)),
                    eu: Some(bucket_from_page(eu, offset, limit)),
                    who: Some(bucket_from_page(who, offset, limit)),
                },
                _meta: crate::cli::search_meta_with_workflow(next_commands, None, workflow),
            }
        }
    };

    crate::render::json::to_pretty(&response).map_err(Into::into)
}
