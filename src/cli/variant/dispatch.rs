use super::guidance::variant_guidance_outcome;
pub(super) use super::{ResolvedVariantQuery, VariantSearchPlan};
use super::{VariantCommand, VariantGetArgs, VariantSearchArgs};
use crate::cli::CommandOutcome;
use crate::cli::{
    PaginationMeta, empty_sections, normalize_cli_query, pagination_footer_offset,
    search_json_with_meta,
};
use crate::entities::variant::{VariantIdFormat, VariantInputKind};
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
            let mutation_query = variant_trial_mutation_query(&id).await;
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
            let result = crate::entities::variant::structure(&id).await?;
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
            detail,
            assertion,
            version,
        } => {
            return Box::pin(super::erepo::handle(
                caid, input, detail, assertion, version, json,
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
                    crate::render::markdown::related_variant(&variant),
                    crate::render::provenance::variant_section_sources(&variant),
                )?
            } else {
                crate::render::markdown::variant_markdown(&variant, super::super::empty_sections())?
            }
        }
    };

    Ok(CommandOutcome::stdout(text))
}

pub(super) fn parse_simple_gene_change(query: &str) -> Option<(String, String)> {
    let parts = query.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 2 {
        return None;
    }

    let gene = parts[0].trim();
    let change = parts[1]
        .trim()
        .trim_start_matches("p.")
        .trim_start_matches("P.");
    if gene.is_empty() || change.is_empty() {
        return None;
    }

    let candidate = format!("{gene} {change}");
    match crate::entities::variant::parse_variant_id(&candidate).ok()? {
        crate::entities::variant::VariantIdFormat::GeneProteinChange { gene, change } => {
            Some((gene, change))
        }
        _ => None,
    }
}

pub(super) fn parse_gene_c_hgvs(query: &str) -> Option<(String, String)> {
    let parts = query.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 2 {
        return None;
    }

    let gene = parts[0].trim();
    let change = parts[1].trim();
    if gene.is_empty() || change.is_empty() || !crate::sources::is_valid_gene_symbol(gene) {
        return None;
    }
    if !change.starts_with("c.") && !change.starts_with("C.") {
        return None;
    }
    Some((gene.to_string(), format!("c.{}", change[2..].trim())))
}

pub(super) fn parse_exon_deletion_phrase(query: &str) -> Option<(String, String)> {
    let parts = query.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 4 {
        return None;
    }

    let gene = parts[0].trim();
    if !crate::sources::is_valid_gene_symbol(gene)
        || !parts[1].eq_ignore_ascii_case("exon")
        || parts[2].parse::<u32>().ok().is_none()
        || !parts[3].eq_ignore_ascii_case("deletion")
    {
        return None;
    }

    Some((gene.to_string(), "inframe_deletion".to_string()))
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

pub(super) fn resolve_variant_query(
    gene_flag: Option<String>,
    hgvsp_flag: Option<String>,
    consequence_flag: Option<String>,
    condition_flag: Option<String>,
    positional_tokens: Vec<String>,
) -> Result<VariantSearchPlan, crate::error::BioMcpError> {
    let gene_flag = normalize_cli_query(gene_flag);
    let hgvsp_flag = normalize_cli_query(hgvsp_flag);
    let consequence_flag = consequence_flag.map(|value| value.trim().to_string());
    let condition_flag = normalize_cli_query(condition_flag);

    let positional = positional_tokens
        .iter()
        .map(|token| token.trim())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let positional = normalize_cli_query(Some(positional));

    let Some(query) = positional else {
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: gene_flag,
            hgvsp: hgvsp_flag,
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    };

    let token_count = query.split_whitespace().count();
    if token_count <= 1 {
        if let Ok(crate::entities::variant::VariantIdFormat::RsId(rsid)) =
            crate::entities::variant::parse_variant_id(&query)
        {
            if gene_flag.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "Use either positional QUERY or --gene, not both".into(),
                ));
            }
            return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
                rsid: Some(rsid),
                hgvsp: hgvsp_flag,
                consequence: consequence_flag,
                condition: condition_flag,
                ..Default::default()
            }));
        }

        if let Some(gene) = gene_flag.clone() {
            if let Some(protein_alias) =
                crate::entities::variant::parse_variant_protein_alias(&query)
            {
                if hgvsp_flag.is_some() {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "Positional residue alias conflicts with --hgvsp".into(),
                    ));
                }
                return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
                    gene: Some(gene),
                    protein_alias: Some(protein_alias),
                    consequence: consequence_flag,
                    condition: condition_flag,
                    ..Default::default()
                }));
            }
            if let crate::entities::variant::VariantInputKind::Shorthand(
                crate::entities::variant::VariantShorthand::ProteinChangeOnly { .. },
            ) = crate::entities::variant::classify_variant_input(&query)
            {
                if hgvsp_flag.is_some() {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "Positional protein change conflicts with --hgvsp".into(),
                    ));
                }
                return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
                    gene: Some(gene),
                    hgvsp: Some(query.clone()),
                    consequence: consequence_flag,
                    condition: condition_flag,
                    ..Default::default()
                }));
            }
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Use either positional QUERY or --gene, not both".into(),
            ));
        }

        if let Some(guidance) = crate::entities::variant::variant_guidance(&query) {
            return Ok(VariantSearchPlan::Guidance(guidance));
        }
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: Some(query),
            hgvsp: hgvsp_flag,
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let Some((gene, change)) = parse_simple_gene_change(&query) {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional \"GENE CHANGE\" conflicts with --gene".into(),
            ));
        }
        if hgvsp_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional \"GENE CHANGE\" conflicts with --hgvsp".into(),
            ));
        }
        let supplied_change = query
            .split_whitespace()
            .nth(1)
            .unwrap_or(&change)
            .to_string();
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: Some(gene),
            hgvsp: Some(supplied_change),
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let crate::entities::variant::VariantInputKind::Shorthand(
        crate::entities::variant::VariantShorthand::GeneResidueAlias {
            gene,
            position,
            residue,
            ..
        },
    ) = crate::entities::variant::classify_variant_input(&query)
    {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional residue alias conflicts with --gene".into(),
            ));
        }
        if hgvsp_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional residue alias conflicts with --hgvsp".into(),
            ));
        }
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: Some(gene),
            protein_alias: Some(crate::entities::variant::VariantProteinAlias {
                position,
                residue,
            }),
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let Some((gene, hgvsc)) = parse_gene_c_hgvs(&query) {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional \"GENE c.HGVS\" conflicts with --gene".into(),
            ));
        }
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: Some(gene),
            hgvsp: hgvsp_flag,
            hgvsc: Some(hgvsc),
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let Some((gene, consequence)) = parse_exon_deletion_phrase(&query) {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional exon-deletion query conflicts with --gene".into(),
            ));
        }
        if consequence_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional exon-deletion query conflicts with --consequence".into(),
            ));
        }
        return Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
            gene: Some(gene),
            hgvsp: hgvsp_flag,
            consequence: Some(consequence),
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if condition_flag.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "Use either positional QUERY or --condition, not both".into(),
        ));
    }
    Ok(VariantSearchPlan::standard(ResolvedVariantQuery {
        gene: gene_flag,
        hgvsp: hgvsp_flag,
        consequence: consequence_flag,
        condition: Some(query),
        ..Default::default()
    }))
}

async fn render_variant_card_outcome(
    args: VariantGetArgs,
    json: bool,
    guidance_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    if args.assembly.is_some()
        && !matches!(
            crate::entities::variant::classify_variant_input(&args.id),
            VariantInputKind::Exact(VariantIdFormat::HgvsGenomic(_))
        )
    {
        return Err(BioMcpError::InvalidArgument(
            "--assembly only applies to chromosome-prefixed genomic SNVs".into(),
        )
        .into());
    }
    if let Some(guidance) = crate::entities::variant::variant_guidance(&args.id) {
        return variant_guidance_outcome(&guidance, json_output || guidance_as_json);
    }

    match crate::entities::variant::get_with_workflow_signals(&args.id, &sections, args.assembly)
        .await
    {
        Ok((variant, signals)) => {
            let text = if json_output {
                let workflow = signals
                    .has_clinvar_signal
                    .then(|| {
                        crate::workflow_ladders::meta_for(
                            crate::workflow_ladders::Workflow::VariantPathogenicity,
                        )
                    })
                    .transpose()?;
                crate::render::json::to_entity_json_with_workflow(
                    &variant,
                    crate::render::markdown::variant_evidence_urls(&variant),
                    crate::render::markdown::related_variant(&variant),
                    crate::render::provenance::variant_section_sources(&variant),
                    workflow,
                )?
            } else {
                crate::render::markdown::variant_markdown(&variant, &sections)?
            };
            Ok(CommandOutcome::stdout(text))
        }
        Err(err) => Err(err.into()),
    }
}

async fn render_variant_search_outcome(
    json_output: bool,
    guidance_as_json: bool,
    request: VariantSearchRequest,
) -> anyhow::Result<CommandOutcome> {
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

    let resolved =
        match resolve_variant_query(gene, hgvsp, consequence, condition, positional_query)? {
            VariantSearchPlan::Standard(resolved) => resolved,
            VariantSearchPlan::Guidance(guidance) => {
                return variant_guidance_outcome(&guidance, json_output || guidance_as_json);
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
    if json_output {
        let next_commands = crate::render::markdown::search_next_commands_variant(
            &results,
            filters.gene.as_deref(),
            filters.condition.as_deref(),
        );
        let output = search_json_with_meta(results, pagination, next_commands)?;
        return Ok(CommandOutcome::stdout(
            crate::render::json::with_variant_search_resolution(
                output,
                page.requested_variant,
                page.resolution,
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
    )?;
    let body = match (page.requested_variant, page.resolution) {
        (Some(requested), Some(resolution)) => format!(
            "Requested variant: {}\n\nResolution: {:?}\n\n{body}",
            serde_json::to_string(&requested)?,
            resolution.status
        ),
        _ => body,
    };
    Ok(CommandOutcome::stdout(body))
}

pub(super) fn trim_protein_change_prefix(value: &str) -> &str {
    value
        .trim()
        .trim_start_matches("p.")
        .trim_start_matches("P.")
}

pub(super) fn normalize_search_hgvsp(value: &str) -> String {
    let normalized = crate::entities::variant::normalize_protein_change(value)
        .unwrap_or_else(|| trim_protein_change_prefix(value).to_string());
    normalized
        .strip_suffix('*')
        .map(|prefix| format!("{prefix}X"))
        .unwrap_or(normalized)
}

async fn variant_trial_mutation_query(id: &str) -> String {
    let id = id.trim();
    if id.is_empty() {
        return String::new();
    }

    if let Ok(crate::entities::variant::VariantIdFormat::GeneProteinChange { gene, change }) =
        crate::entities::variant::parse_variant_id(id)
    {
        let normalized = crate::entities::variant::normalize_protein_change(&change)
            .unwrap_or_else(|| trim_protein_change_prefix(&change).to_string());
        if !normalized.is_empty() {
            return format!("{gene} {normalized}");
        }
    }

    if let Ok(variant) = crate::entities::variant::get(id, empty_sections()).await {
        let gene = variant.gene.trim();
        let protein = variant
            .hgvs_p
            .as_deref()
            .map(|value| {
                crate::entities::variant::normalize_protein_change(value)
                    .unwrap_or_else(|| trim_protein_change_prefix(value).to_string())
            })
            .unwrap_or_default();
        if !gene.is_empty() && !protein.is_empty() {
            return format!("{gene} {protein}");
        }
    }

    id.to_string()
}
