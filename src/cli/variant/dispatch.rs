pub(super) use super::VariantSearchPlan;
use super::guidance::variant_guidance_outcome;
use super::query::{
    GeneFirstNote, apply_gene_first_routing, confirm_gene_first_candidate, gene_first_working_form,
    resolve_variant_query,
};
use super::{VariantCommand, VariantGetArgs, VariantSearchArgs};
use crate::cli::CommandOutcome;
use crate::cli::{PaginationMeta, pagination_footer_offset, search_json_with_meta};
use crate::error::BioMcpError;

pub(crate) async fn handle_get(
    args: VariantGetArgs,
    json: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    render_variant_card_outcome(args, json, alias_suggestions_as_json).await
}

pub(crate) async fn handle_search(
    args: VariantSearchArgs,
    json: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    render_variant_search_outcome(
        json,
        alias_suggestions_as_json,
        VariantSearchRequest {
            gene: args.gene,
            positional_query: args.positional_query,
            hgvsp: args.hgvsp,
            significance: args.significance,
            max_frequency: args.max_frequency,
            min_cadd: args.min_cadd,
            consequence: args.consequence,
            review_status: args.review_status,
            population: args.population,
            revel_min: args.revel_min,
            gerp_min: args.gerp_min,
            tumor_site: args.tumor_site,
            condition: args.condition,
            impact: args.impact,
            lof: args.lof,
            has: args.has,
            missing: args.missing,
            therapy: args.therapy,
            limit: args.limit,
            offset: args.offset,
        },
    )
    .await
}

pub(crate) async fn handle_command(
    cmd: VariantCommand,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        VariantCommand::Trials {
            id,
            limit,
            offset,
            source,
        } => {
            let _ = crate::entities::variant::parse_variant_id(&id)?;
            let mutation_query = super::trial::variant_trial_mutation_query(&id).await;
            let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
            let filters = crate::entities::trial::TrialSearchFilters {
                mutation: Some(mutation_query.clone()),
                source: trial_source,
                ..Default::default()
            };
            let (results, total) = crate::entities::trial::search(&filters, limit, offset).await?;
            if let Some(total) = total {
                super::super::log_pagination_truncation(total as usize, offset, results.len());
            }
            if json {
                #[derive(serde::Serialize)]
                struct SearchResponse {
                    count: usize,
                    total: Option<u32>,
                    results: Vec<crate::entities::trial::TrialSearchResult>,
                }

                crate::render::json::to_pretty(&SearchResponse {
                    count: results.len(),
                    total,
                    results,
                })?
            } else {
                let mut query_parts = vec![format!("mutation={mutation_query}")];
                if matches!(trial_source, crate::entities::trial::TrialSource::NciCts) {
                    query_parts.push("source=nci".to_string());
                }
                if offset > 0 {
                    query_parts.push(format!("offset={offset}"));
                }
                let query = query_parts.join(", ");
                crate::render::markdown::trial_search_markdown(&query, &results, total)?
            }
        }
        VariantCommand::Articles {
            id,
            input,
            debug_plan,
            verify_identity,
            confirmed_only,
            strategy,
            limit,
            offset,
        } => {
            return Box::pin(super::articles::handle(
                id,
                input,
                debug_plan,
                verify_identity,
                confirmed_only,
                strategy,
                limit,
                offset,
                json,
            ))
            .await;
        }
        VariantCommand::Structure { id } => {
            let mut result = crate::entities::variant::structure(&id).await?;
            result.meta.next_commands =
                crate::render::markdown::variant_structure_recovery_commands(&result);
            if json {
                crate::render::json::to_pretty(&result)?
            } else {
                crate::render::markdown::variant_structure_markdown(&result)
            }
        }
        VariantCommand::Oncokb { id } => {
            let result = crate::entities::variant::oncokb(&id).await?;
            if json {
                crate::render::json::to_pretty(&result)?
            } else {
                crate::render::markdown::variant_oncokb_markdown(&result)
            }
        }
        VariantCommand::Erepo {
            caid,
            input,
            gene,
            limit,
            offset,
            detail,
            assertion,
            version,
        } => {
            return Box::pin(super::erepo::handle(
                super::erepo::Request {
                    caid,
                    input,
                    gene,
                    limit,
                    offset,
                    detail,
                    assertion,
                    version,
                },
                json,
            ))
            .await;
        }
        VariantCommand::Normalize {
            service,
            variant,
            input,
        } => {
            if input.is_some() && !service.eq_ignore_ascii_case("car") {
                return Err(BioMcpError::InvalidArgument(
                    "variant normalize --input is available only for car".into(),
                )
                .into());
            }
            if let Some(input) = input {
                return Box::pin(super::car::handle_batch(&input, json)).await;
            }
            let variant = variant.ok_or_else(|| {
                BioMcpError::InvalidArgument(
                    "variant normalize requires HGVS input or --input".into(),
                )
            })?;
            if service.eq_ignore_ascii_case("car") {
                return Box::pin(super::car::handle_single(&variant, json)).await;
            } else {
                let result =
                    crate::entities::variant::normalize_variant(&service, &variant).await?;
                if json {
                    super::normalization_json::render(&result)?
                } else {
                    crate::render::markdown::variant_normalization_markdown(&result)
                }
            }
        }
        VariantCommand::External(args) => {
            let id = args.join(" ");
            let variant =
                crate::entities::variant::get(&id, super::super::empty_sections()).await?;
            if json {
                crate::render::json::to_entity_json(
                    &variant,
                    crate::render::markdown::variant_evidence_urls(&variant),
                    crate::render::markdown::related_variant_with_recovery(&variant),
                    crate::render::provenance::variant_section_sources(&variant),
                )?
            } else {
                crate::render::markdown::variant_markdown(&variant, super::super::empty_sections())?
            }
        }
    };

    Ok(CommandOutcome::stdout(text))
}

#[derive(Debug, Clone)]
struct VariantSearchRequest {
    gene: Option<String>,
    positional_query: Vec<String>,
    hgvsp: Option<String>,
    significance: Option<String>,
    max_frequency: Option<f64>,
    min_cadd: Option<f64>,
    consequence: Option<String>,
    review_status: Option<String>,
    population: Option<String>,
    revel_min: Option<f64>,
    gerp_min: Option<f64>,
    tumor_site: Option<String>,
    condition: Option<String>,
    impact: Option<String>,
    lof: bool,
    has: Option<String>,
    missing: Option<String>,
    therapy: Option<String>,
    limit: usize,
    offset: usize,
}

impl VariantSearchRequest {
    /// Every explicit filter flag this request carries, in the command
    /// line's own spelling, so the gene-first hints repeat what the caller
    /// typed instead of dropping filters the search applied (ticket 2022's
    /// promise, restored by ticket 2038 finding 7). `--limit` and `--offset`
    /// are pagination, not filters, and stay out.
    fn explicit_filter_flags(&self) -> Vec<(&'static str, String)> {
        let mut flags = Vec::new();
        flag_text(&mut flags, "--significance", self.significance.as_deref());
        flag_number(&mut flags, "--max-frequency", self.max_frequency);
        flag_number(&mut flags, "--min-cadd", self.min_cadd);
        flag_text(&mut flags, "--review-status", self.review_status.as_deref());
        flag_text(&mut flags, "--population", self.population.as_deref());
        flag_number(&mut flags, "--revel-min", self.revel_min);
        flag_number(&mut flags, "--gerp-min", self.gerp_min);
        flag_text(&mut flags, "--tumor-site", self.tumor_site.as_deref());
        flag_text(&mut flags, "--impact", self.impact.as_deref());
        if self.lof {
            flags.push(("--lof", String::new()));
        }
        flag_text(&mut flags, "--has", self.has.as_deref());
        flag_text(&mut flags, "--missing", self.missing.as_deref());
        flag_text(&mut flags, "--therapy", self.therapy.as_deref());
        flags
    }
}

/// Add a text-valued flag when the request carries a non-blank value.
fn flag_text(flags: &mut Vec<(&'static str, String)>, flag: &'static str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        flags.push((flag, value.to_string()));
    }
}

/// Add a number-valued flag when the request carries a value.
fn flag_number(flags: &mut Vec<(&'static str, String)>, flag: &'static str, value: Option<f64>) {
    if let Some(value) = value {
        flags.push((flag, value.to_string()));
    }
}

pub(crate) fn render_loaded_card(
    variant: &crate::entities::variant::Variant,
    has_clinvar_signal: bool,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<String> {
    if json_output {
        let workflow = has_clinvar_signal
            .then(|| {
                crate::workflow_ladders::meta_for(
                    crate::workflow_ladders::Workflow::VariantPathogenicity,
                )
            })
            .transpose()?;
        Ok(crate::render::json::to_entity_json_with_workflow(
            variant,
            crate::render::markdown::variant_evidence_urls(variant),
            crate::render::markdown::related_variant_with_recovery(variant),
            crate::render::provenance::variant_section_sources(variant),
            workflow,
        )?)
    } else {
        Ok(crate::render::markdown::variant_markdown(
            variant, sections,
        )?)
    }
}

async fn render_variant_card_outcome(
    args: VariantGetArgs,
    json: bool,
    guidance_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    if args.assembly.is_some()
        && crate::entities::variant::normalize_genomic_coordinate(&args.id)?.is_none()
    {
        return Err(BioMcpError::InvalidArgument(
            "--assembly only applies to chromosome-prefixed genomic coordinates".into(),
        )
        .into());
    }
    let assembly = if crate::entities::variant::normalize_genomic_coordinate(&args.id)?
        .is_some_and(|coordinate| coordinate.requires_comparison)
    {
        Some(crate::entities::variant::resolved_default_assembly(
            args.assembly,
        )?)
    } else {
        args.assembly
    };
    if let Some(guidance) = crate::entities::variant::variant_guidance(&args.id) {
        return variant_guidance_outcome(&guidance, json_output || guidance_as_json);
    }

    match crate::entities::variant::get_with_workflow_signals(&args.id, &sections, assembly).await {
        Ok((variant, signals)) => Ok(CommandOutcome::stdout(render_loaded_card(
            &variant,
            signals.has_clinvar_signal,
            &sections,
            json_output,
        )?)),
        Err(err) => Err(err.into()),
    }
}

async fn render_variant_search_outcome(
    json_output: bool,
    guidance_as_json: bool,
    request: VariantSearchRequest,
) -> anyhow::Result<CommandOutcome> {
    let explicit_filters = request.explicit_filter_flags();
    let VariantSearchRequest {
        gene,
        positional_query,
        hgvsp,
        significance,
        max_frequency,
        min_cadd,
        consequence,
        review_status,
        population,
        revel_min,
        gerp_min,
        tumor_site,
        condition,
        impact,
        lof,
        has,
        missing,
        therapy,
        limit,
        offset,
    } = request;

    let (resolved, gene_first_note) =
        match resolve_variant_query(gene, hgvsp, consequence, condition, positional_query)? {
            VariantSearchPlan::Standard(resolved) => (resolved, None),
            VariantSearchPlan::Guidance(guidance) => {
                return variant_guidance_outcome(&guidance, json_output || guidance_as_json);
            }
            VariantSearchPlan::GeneFirstCandidate {
                gene,
                protein_change,
                condition,
                hgvsp,
                consequence,
            } => {
                let confirmed = confirm_gene_first_candidate(&gene).await;
                apply_gene_first_routing(
                    gene,
                    protein_change,
                    condition,
                    confirmed,
                    hgvsp,
                    consequence,
                    &explicit_filters,
                )
            }
        };

    let filters = crate::entities::variant::VariantSearchFilters {
        gene: resolved.gene,
        hgvsp: resolved.hgvsp,
        hgvsc: resolved.hgvsc,
        rsid: resolved.rsid,
        protein_alias: resolved.protein_alias,
        significance,
        max_frequency,
        min_cadd,
        consequence: resolved.consequence,
        review_status,
        population,
        revel_min,
        gerp_min,
        tumor_site,
        condition: resolved.condition,
        impact,
        lof,
        has,
        missing,
        therapy,
        requested_identity: resolved.requested_identity.map(|identity| *identity),
    };

    let mut query = crate::entities::variant::search_query_summary(&filters);
    if offset > 0 {
        query = if query.is_empty() {
            format!("offset={offset}")
        } else {
            format!("{query}, offset={offset}")
        };
    }

    let page = crate::entities::variant::search_page(&filters, limit, offset).await?;
    let results = page.results;
    let mut pagination = PaginationMeta::offset(offset, limit, results.len(), page.total);
    pagination.has_more = page.has_more.unwrap_or(pagination.has_more);
    let zero_row_note = gene_first_note.as_ref().filter(|_| results.is_empty());
    let zero_row_command = zero_row_note.map(|note| match note {
        GeneFirstNote::Refused {
            condition,
            hgvsp,
            consequence,
            ..
        } => gene_first_working_form(
            condition,
            hgvsp.as_deref(),
            consequence.as_deref(),
            &explicit_filters,
        ),
        GeneFirstNote::Routed { alternative, .. } => alternative.clone(),
    });
    if json_output {
        let mut next_commands = crate::render::markdown::search_next_commands_variant(
            &results,
            filters.gene.as_deref(),
            filters.condition.as_deref(),
        );
        if let Some(command) = zero_row_command.as_ref() {
            next_commands.push(command.clone());
        }
        let output = search_json_with_meta(results, pagination, next_commands)?;
        return Ok(CommandOutcome::stdout(
            crate::render::json::with_variant_search_resolution(
                output,
                page.requested_variant,
                page.resolution,
                page.filter_evaluation,
                page.diagnostics,
            )?,
        ));
    }

    let footer = pagination_footer_offset(&pagination);
    let body = crate::render::markdown::variant_search_markdown_with_context(
        &query,
        &results,
        &footer,
        filters.gene.as_deref(),
        filters.condition.as_deref(),
        &page.filter_evaluation,
        &page.diagnostics,
    )?;
    let body = match (page.requested_variant, page.resolution) {
        (Some(requested), Some(resolution)) => format!(
            "Requested variant: {}\n\nVariant identity: {}\n\n{body}",
            requested.human_label(),
            format!("{:?}", resolution.status).to_lowercase()
        ),
        _ => body,
    };
    let body = match zero_row_note {
        Some(GeneFirstNote::Refused {
            gene,
            condition,
            hgvsp,
            consequence,
        }) => format!(
            "{body}\n\nNo variants matched the phrase as a condition. Try the condition \
             without {gene}: {}",
            gene_first_working_form(
                condition,
                hgvsp.as_deref(),
                consequence.as_deref(),
                &explicit_filters,
            )
        ),
        Some(GeneFirstNote::Routed {
            parsed,
            alternative,
        }) => format!(
            "{body}\n\nNo variants matched the routed phrase. Read as {parsed}. \
             Try the same filters without the condition: {alternative}"
        ),
        None => body,
    };
    Ok(CommandOutcome::stdout(body))
}

#[cfg(test)]
mod tests {
    use super::VariantSearchRequest;

    #[test]
    fn explicit_filter_flags_state_every_flag_in_command_spelling() {
        let request = VariantSearchRequest {
            gene: None,
            positional_query: Vec::new(),
            hgvsp: None,
            significance: Some("benign".into()),
            max_frequency: Some(0.01),
            min_cadd: Some(20.0),
            consequence: None,
            review_status: Some("2".into()),
            population: Some("nfe".into()),
            revel_min: Some(0.5),
            gerp_min: None,
            tumor_site: Some("skin".into()),
            condition: None,
            impact: None,
            lof: true,
            has: Some("clinvar".into()),
            missing: None,
            therapy: None,
            limit: 10,
            offset: 0,
        };
        assert_eq!(
            request.explicit_filter_flags(),
            vec![
                ("--significance", "benign".to_string()),
                ("--max-frequency", "0.01".to_string()),
                ("--min-cadd", "20".to_string()),
                ("--review-status", "2".to_string()),
                ("--population", "nfe".to_string()),
                ("--revel-min", "0.5".to_string()),
                ("--tumor-site", "skin".to_string()),
                ("--lof", String::new()),
                ("--has", "clinvar".to_string()),
            ]
        );
    }

    #[test]
    fn explicit_filter_flags_skip_unset_and_blank_values() {
        let request = VariantSearchRequest {
            significance: Some("   ".into()),
            lof: false,
            ..empty_request()
        };
        assert!(request.explicit_filter_flags().is_empty());
    }

    fn empty_request() -> VariantSearchRequest {
        VariantSearchRequest {
            gene: None,
            positional_query: Vec::new(),
            hgvsp: None,
            significance: None,
            max_frequency: None,
            min_cadd: None,
            consequence: None,
            review_status: None,
            population: None,
            revel_min: None,
            gerp_min: None,
            tumor_site: None,
            condition: None,
            impact: None,
            lof: false,
            has: None,
            missing: None,
            therapy: None,
            limit: 10,
            offset: 0,
        }
    }
}
