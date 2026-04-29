use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::{
    BatchArgs, CvxCommand, DdinterCommand, EmaCommand, EnrichArgs, GtrCommand, VersionArgs,
    WhoCommand, WhoIvdCommand,
};
use crate::cli::CommandOutcome;
use futures::future::try_join_all;

pub(crate) async fn handle_batch(args: BatchArgs, json: bool) -> anyhow::Result<CommandOutcome> {
    let entity = args.entity.trim().to_ascii_lowercase();
    let parsed_ids = args
        .ids
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let batch_sections = parse_batch_sections(args.sections.as_deref());

    if parsed_ids.is_empty() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "Batch IDs are required. Example: biomcp batch gene BRAF,TP53".into(),
        )
        .into());
    }
    if parsed_ids.len() > 10 {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "Batch is limited to 10 IDs".into(),
        )
        .into());
    }

    let text = match entity.as_str() {
        "gene" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::gene::get(id, &batch_sections));
            let results = try_join_all(futs).await?;
            if json {
                super::super::render_batch_json(&results, |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::gene_evidence_urls(item),
                        crate::render::markdown::related_gene(item),
                        crate::render::provenance::gene_section_sources(item),
                    )
                })?
            } else {
                let mut out = String::new();
                out.push_str(&format!("# Batch: gene ({})\n\n", results.len()));
                for (idx, item) in results.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("\n\n---\n\n");
                    }
                    out.push_str(&crate::render::markdown::gene_markdown(
                        item,
                        &batch_sections,
                    )?);
                }
                out
            }
        }
        "variant" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::variant::get(id, &batch_sections));
            let results = try_join_all(futs).await?;
            if json {
                super::super::render_batch_json(&results, |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::variant_evidence_urls(item),
                        crate::render::markdown::related_variant(item),
                        crate::render::provenance::variant_section_sources(item),
                    )
                })?
            } else {
                let mut out = String::new();
                out.push_str(&format!("# Batch: variant ({})\n\n", results.len()));
                for (idx, item) in results.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("\n\n---\n\n");
                    }
                    out.push_str(&crate::render::markdown::variant_markdown(
                        item,
                        &batch_sections,
                    )?);
                }
                out
            }
        }
        "article" => {
            let futs = parsed_ids.iter().map(|id| {
                crate::entities::article::get(
                    id,
                    &batch_sections,
                    crate::entities::article::ArticleGetOptions::default(),
                )
            });
            let results = try_join_all(futs).await?;
            if json {
                super::super::render_batch_json(&results, |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::article_evidence_urls(item),
                        crate::render::markdown::related_article(item),
                        crate::render::provenance::article_section_sources(item),
                    )
                })?
            } else {
                let mut out = String::new();
                out.push_str(&format!("# Batch: article ({})\n\n", results.len()));
                for (idx, item) in results.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("\n\n---\n\n");
                    }
                    out.push_str(&crate::render::markdown::article_markdown(
                        item,
                        &batch_sections,
                    )?);
                }
                out
            }
        }
        "trial" => {
            let trial_source = crate::entities::trial::TrialSource::from_flag(&args.source)?;
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::trial::get(id, &batch_sections, trial_source));
            let results = try_join_all(futs).await?;
            if json {
                super::super::render_batch_json(&results, |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::trial_evidence_urls(item),
                        crate::render::markdown::related_trial(item),
                        crate::render::provenance::trial_section_sources(item),
                    )
                })?
            } else {
                let mut out = String::new();
                out.push_str(&format!("# Batch: trial ({})\n\n", results.len()));
                for (idx, item) in results.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("\n\n---\n\n");
                    }
                    out.push_str(&crate::render::markdown::trial_markdown(
                        item,
                        &batch_sections,
                    )?);
                }
                out
            }
        }
        "drug" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::drug::get(id, &batch_sections));
            let results = try_join_all(futs).await?;
            if json {
                super::super::render_batch_json(&results, |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::drug_evidence_urls(item),
                        crate::render::markdown::related_drug(item),
                        crate::render::provenance::drug_section_sources(item),
                    )
                })?
            } else {
                let mut out = String::new();
                out.push_str(&format!("# Batch: drug ({})\n\n", results.len()));
                for (idx, item) in results.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("\n\n---\n\n");
                    }
                    out.push_str(&crate::render::markdown::drug_markdown(
                        item,
                        &batch_sections,
                    )?);
                }
                out
            }
        }
        "disease" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::disease::get(id, &batch_sections));
            let results = try_join_all(futs).await?;
            if json {
                super::super::render_batch_json(&results, |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::disease_evidence_urls(item),
                        crate::render::markdown::related_disease(item),
                        crate::render::provenance::disease_section_sources(item),
                    )
                })?
            } else {
                let mut out = String::new();
                out.push_str(&format!("# Batch: disease ({})\n\n", results.len()));
                for (idx, item) in results.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("\n\n---\n\n");
                    }
                    out.push_str(&crate::render::markdown::disease_markdown(
                        item,
                        &batch_sections,
                    )?);
                }
                out
            }
        }
        "pgx" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::pgx::get(id, &batch_sections));
            let results = try_join_all(futs).await?;
            if json {
                super::super::render_batch_json(&results, |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::pgx_evidence_urls(item),
                        crate::render::markdown::related_pgx(item),
                        crate::render::provenance::pgx_section_sources(item),
                    )
                })?
            } else {
                let mut out = String::new();
                out.push_str(&format!("# Batch: pgx ({})\n\n", results.len()));
                for (idx, item) in results.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("\n\n---\n\n");
                    }
                    out.push_str(&crate::render::markdown::pgx_markdown(
                        item,
                        &batch_sections,
                    )?);
                }
                out
            }
        }
        "pathway" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::pathway::get(id, &batch_sections));
            let results = try_join_all(futs).await?;
            if json {
                super::super::render_batch_json(&results, |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::pathway_evidence_urls(item),
                        crate::render::markdown::related_pathway(item),
                        crate::render::provenance::pathway_section_sources(item),
                    )
                })?
            } else {
                let mut out = String::new();
                out.push_str(&format!("# Batch: pathway ({})\n\n", results.len()));
                for (idx, item) in results.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("\n\n---\n\n");
                    }
                    out.push_str(&crate::render::markdown::pathway_markdown(
                        item,
                        &batch_sections,
                    )?);
                }
                out
            }
        }
        "protein" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::protein::get(id, &batch_sections));
            let results = try_join_all(futs).await?;
            if json {
                super::super::render_batch_json(&results, |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::protein_evidence_urls(item),
                        crate::render::markdown::related_protein(item, &batch_sections),
                        crate::render::provenance::protein_section_sources(item),
                    )
                })?
            } else {
                let mut out = String::new();
                out.push_str(&format!("# Batch: protein ({})\n\n", results.len()));
                for (idx, item) in results.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("\n\n---\n\n");
                    }
                    out.push_str(&crate::render::markdown::protein_markdown(
                        item,
                        &batch_sections,
                    )?);
                }
                out
            }
        }
        "adverse-event" | "adverse_event" | "adverseevent" => {
            if !batch_sections.is_empty() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "Batch sections are not supported for adverse-event".into(),
                )
                .into());
            }
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::adverse_event::get(id));
            let results = try_join_all(futs).await?;
            if json {
                super::super::render_batch_json(&results, |item| match item {
                    crate::entities::adverse_event::AdverseEventReport::Faers(report) => {
                        crate::render::json::to_entity_json_value(
                            item,
                            crate::render::markdown::adverse_event_evidence_urls(report),
                            crate::render::markdown::related_adverse_event(report),
                            crate::render::provenance::adverse_event_report_section_sources(item),
                        )
                    }
                    crate::entities::adverse_event::AdverseEventReport::Device(report) => {
                        crate::render::json::to_entity_json_value(
                            item,
                            crate::render::markdown::device_event_evidence_urls(report),
                            crate::render::markdown::related_device_event(report),
                            crate::render::provenance::adverse_event_report_section_sources(item),
                        )
                    }
                })?
            } else {
                let mut out = String::new();
                out.push_str(&format!("# Batch: adverse-event ({})\n\n", results.len()));
                for (idx, item) in results.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("\n\n---\n\n");
                    }
                    match item {
                        crate::entities::adverse_event::AdverseEventReport::Faers(report) => {
                            out.push_str(&crate::render::markdown::adverse_event_markdown(
                                report,
                                super::super::empty_sections(),
                            )?);
                        }
                        crate::entities::adverse_event::AdverseEventReport::Device(report) => {
                            out.push_str(&crate::render::markdown::device_event_markdown(report)?);
                        }
                    }
                }
                out
            }
        }
        other => {
            return Err(crate::error::BioMcpError::InvalidArgument(format!(
                "Unknown batch entity '{other}'. Expected one of: gene, variant, article, trial, drug, disease, pgx, pathway, protein, adverse-event"
            ))
            .into());
        }
    };

    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_ema(cmd: EmaCommand) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        EmaCommand::Sync => {
            crate::sources::ema::EmaClient::sync(crate::sources::ema::EmaSyncMode::Force).await?;
            "EMA data synchronized successfully.\n".to_string()
        }
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_who(cmd: WhoCommand) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        WhoCommand::Sync => {
            crate::sources::who_pq::WhoPqClient::sync(crate::sources::who_pq::WhoPqSyncMode::Force)
                .await?;
            "WHO Prequalification data synchronized successfully.\n".to_string()
        }
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_cvx(cmd: CvxCommand) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        CvxCommand::Sync => {
            crate::sources::cvx::CvxClient::sync(crate::sources::cvx::CvxSyncMode::Force).await?;
            "CDC CVX/MVX local data bundle synchronized successfully.\n".to_string()
        }
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_ddinter(cmd: DdinterCommand) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        DdinterCommand::Sync => {
            crate::sources::ddinter::DdinterClient::sync(
                crate::sources::ddinter::DdinterSyncMode::Force,
            )
            .await?;
            "DDInter local interaction data synchronized successfully.\n".to_string()
        }
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_gtr(cmd: GtrCommand) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        GtrCommand::Sync => {
            crate::sources::gtr::GtrClient::sync(crate::sources::gtr::GtrSyncMode::Force).await?;
            "GTR local diagnostic data synchronized successfully.\n".to_string()
        }
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_who_ivd(cmd: WhoIvdCommand) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        WhoIvdCommand::Sync => {
            crate::sources::who_ivd::WhoIvdClient::sync(
                crate::sources::who_ivd::WhoIvdSyncMode::Force,
            )
            .await?;
            "WHO IVD local diagnostic data synchronized successfully.\n".to_string()
        }
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_enrich(args: EnrichArgs, json: bool) -> anyhow::Result<CommandOutcome> {
    const MAX_ENRICH_LIMIT: usize = 50;
    if args.limit == 0 || args.limit > MAX_ENRICH_LIMIT {
        return Err(crate::error::BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_ENRICH_LIMIT}"
        ))
        .into());
    }
    let genes = args
        .genes
        .split(',')
        .map(str::trim)
        .filter(|gene| !gene.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if genes.is_empty() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "At least one gene is required. Example: biomcp enrich BRAF,KRAS".into(),
        )
        .into());
    }
    let terms = crate::sources::gprofiler::GProfilerClient::new()?
        .enrich_genes(&genes, args.limit)
        .await?;
    let text = if json {
        #[derive(serde::Serialize)]
        struct EnrichResponse {
            genes: Vec<String>,
            count: usize,
            results: Vec<crate::sources::gprofiler::GProfilerTerm>,
        }

        crate::render::json::to_pretty(&EnrichResponse {
            genes,
            count: terms.len(),
            results: terms,
        })?
    } else {
        enrich_markdown(&genes, &terms)
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_uninstall() -> anyhow::Result<CommandOutcome> {
    Ok(CommandOutcome::stdout(uninstall_self()?))
}

pub(crate) async fn handle_version(
    args: VersionArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let text = if json {
        crate::render::json::to_pretty(&version_identity())?
    } else {
        version_output(args.verbose)
    };
    Ok(CommandOutcome::stdout(text))
}

pub(super) fn parse_batch_sections(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[derive(serde::Serialize)]
pub(super) struct VersionIdentity {
    version: &'static str,
    git_revision: &'static str,
    build_timestamp: &'static str,
}

pub(super) fn version_identity() -> VersionIdentity {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let git_tag = option_env!("BIOMCP_BUILD_GIT_TAG");
    let version = git_tag
        .filter(|t| t.starts_with('v') && !t.contains('-'))
        .map(|t| &t[1..])
        .unwrap_or(cargo_version);
    VersionIdentity {
        version,
        git_revision: option_env!("BIOMCP_BUILD_GIT_SHA").unwrap_or("unknown"),
        build_timestamp: option_env!("BIOMCP_BUILD_DATE").unwrap_or("unknown"),
    }
}

pub(super) fn version_output(verbose: bool) -> String {
    let identity = version_identity();
    let version = identity.version;
    let git = identity.git_revision;
    let build = identity.build_timestamp;
    let base = format!("biomcp {version} (git {git}, build {build})");
    if !verbose {
        return base;
    }

    let executable = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let path_hits = find_biomcp_on_path();
    let active = std::env::current_exe()
        .ok()
        .as_deref()
        .and_then(canonical_for_compare);
    let mut out = Vec::new();
    out.push(base);
    out.push(format!("Executable: {executable}"));
    out.push(format!("Build: version={version}, git={git}, date={build}"));
    out.push("PATH:".to_string());
    if path_hits.is_empty() {
        out.push("- (no biomcp binaries found on PATH)".to_string());
    } else {
        for hit in &path_hits {
            let canonical = canonical_for_compare(hit);
            let marker = if active.is_some() && active == canonical {
                " (active)"
            } else {
                ""
            };
            out.push(format!("- {}{}", hit.display(), marker));
        }
    }
    if executable.contains("/.venv/") || executable.contains("\\.venv\\") {
        out.push("Warning: active executable appears to come from a virtualenv path.".to_string());
    }
    if path_hits.len() > 1 {
        out.push(format!(
            "Warning: multiple biomcp binaries found on PATH ({}).",
            path_hits.len()
        ));
    }
    out.join("\n")
}

pub(super) fn find_biomcp_on_path() -> Vec<PathBuf> {
    #[cfg(windows)]
    let binary_name = "biomcp.exe";
    #[cfg(not(windows))]
    let binary_name = "biomcp";

    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let Some(path_var) = std::env::var_os("PATH") else {
        return out;
    };
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary_name);
        if !candidate.is_file() {
            continue;
        }
        let canonical = canonical_for_compare(&candidate);
        let key = canonical
            .as_deref()
            .unwrap_or(candidate.as_path())
            .display()
            .to_string();
        if seen.insert(key) {
            out.push(candidate);
        }
    }
    out
}

pub(super) fn canonical_for_compare(path: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(path).ok()
}

pub(super) fn uninstall_self() -> Result<String, crate::error::BioMcpError> {
    let current = std::env::current_exe()?;
    match std::fs::remove_file(&current) {
        Ok(()) => Ok(format!("Uninstalled biomcp from {}", current.display())),
        Err(err) => Ok(format!(
            "Unable to remove running binary automatically ({err}).\nRemove manually:\n  rm {}",
            current.display()
        )),
    }
}

pub(super) fn enrich_markdown(
    genes: &[String],
    terms: &[crate::sources::gprofiler::GProfilerTerm],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Enrichment: {}\n\n", genes.join(", ")));
    if terms.is_empty() {
        out.push_str("No enriched terms found.\n");
        return out;
    }

    out.push_str("| Source | ID | Name | p-value |\n");
    out.push_str("|--------|----|------|---------|\n");
    for row in terms {
        let source = row.source.as_deref().unwrap_or("-");
        let id = row.native.as_deref().unwrap_or("-");
        let name = row.name.as_deref().unwrap_or("-");
        let p = row
            .p_value
            .map(|v| format!("{v:.3e}"))
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!("| {source} | {id} | {name} | {p} |\n"));
    }
    out
}
