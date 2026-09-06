use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::{
    ArticleBatchMode, BatchArgs, CvxCommand, DdinterCommand, EmaCommand, EnrichArgs, GtrCommand,
    VersionArgs, WhoCommand, WhoIvdCommand,
};
use super::{settle_batch, validate_batch_args, validate_batch_ids};
use crate::cli::CommandOutcome;

pub(crate) async fn handle_batch(args: BatchArgs, json: bool) -> anyhow::Result<CommandOutcome> {
    validate_batch_args(&args)?;
    let entity = args.entity.trim().to_ascii_lowercase();
    let parsed_ids = validate_batch_ids(&args, &entity)?;
    let batch_sections = parse_batch_sections(args.sections.as_deref());

    match entity.as_str() {
        "gene" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::gene::get(id, &batch_sections));
            return settle_batch(
                "gene",
                &parsed_ids,
                futs,
                json,
                |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::gene_evidence_urls(item),
                        crate::render::markdown::related_gene(item),
                        crate::render::provenance::gene_section_sources(item),
                    )
                },
                |item| crate::render::markdown::gene_markdown(item, &batch_sections),
            )
            .await;
        }
        "variant" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::variant::get(id, &batch_sections));
            return settle_batch(
                "variant",
                &parsed_ids,
                futs,
                json,
                |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::variant_evidence_urls(item),
                        crate::render::markdown::related_variant(item),
                        crate::render::provenance::variant_section_sources(item),
                    )
                },
                |item| crate::render::markdown::variant_markdown(item, &batch_sections),
            )
            .await;
        }
        "article" => {
            if args.mode == Some(ArticleBatchMode::Compact) {
                let futs = parsed_ids
                    .iter()
                    .map(|id| crate::entities::article::get_compact(id));
                return settle_batch(
                    "article",
                    &parsed_ids,
                    futs,
                    json,
                    |item| serde_json::to_value(item).map_err(crate::error::BioMcpError::Json),
                    |item| {
                        crate::render::markdown::article_batch_markdown(std::slice::from_ref(item))
                    },
                )
                .await;
            }
            let futs = parsed_ids.iter().map(|id| {
                crate::entities::article::get(
                    id,
                    &batch_sections,
                    crate::entities::article::ArticleGetOptions::default(),
                )
            });
            return settle_batch(
                "article",
                &parsed_ids,
                futs,
                json,
                |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::article_evidence_urls(item),
                        crate::render::markdown::related_article(item),
                        crate::render::provenance::article_section_sources(item),
                    )
                },
                |item| crate::render::markdown::article_markdown(item, &batch_sections),
            )
            .await;
        }
        "trial" => {
            let trial_source = crate::entities::trial::TrialSource::from_flag(
                args.source.as_deref().unwrap_or("ctgov"),
            )?;
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::trial::get(id, &batch_sections, trial_source));
            return settle_batch(
                "trial",
                &parsed_ids,
                futs,
                json,
                |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::trial_evidence_urls(item),
                        crate::render::markdown::related_trial(item),
                        crate::render::provenance::trial_section_sources(item),
                    )
                },
                |item| crate::render::markdown::trial_markdown(item, &batch_sections),
            )
            .await;
        }
        "drug" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::drug::get(id, &batch_sections));
            return settle_batch(
                "drug",
                &parsed_ids,
                futs,
                json,
                |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::drug_evidence_urls(item),
                        crate::render::markdown::related_drug(item),
                        crate::render::provenance::drug_section_sources(item),
                    )
                },
                |item| crate::render::markdown::drug_markdown(item, &batch_sections),
            )
            .await;
        }
        "disease" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::disease::get(id, &batch_sections));
            return settle_batch(
                "disease",
                &parsed_ids,
                futs,
                json,
                |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::disease_evidence_urls(item),
                        crate::render::markdown::related_disease(item),
                        crate::render::provenance::disease_section_sources(item),
                    )
                },
                |item| crate::render::markdown::disease_markdown(item, &batch_sections),
            )
            .await;
        }
        "pgx" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::pgx::get(id, &batch_sections));
            return settle_batch(
                "pgx",
                &parsed_ids,
                futs,
                json,
                |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::pgx_evidence_urls(item),
                        crate::render::markdown::related_pgx(item),
                        crate::render::provenance::pgx_section_sources(item),
                    )
                },
                |item| crate::render::markdown::pgx_markdown(item, &batch_sections),
            )
            .await;
        }
        "pathway" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::pathway::get(id, &batch_sections));
            return settle_batch(
                "pathway",
                &parsed_ids,
                futs,
                json,
                |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::pathway_evidence_urls(item),
                        crate::render::markdown::related_pathway(item),
                        crate::render::provenance::pathway_section_sources(item),
                    )
                },
                |item| crate::render::markdown::pathway_markdown(item, &batch_sections),
            )
            .await;
        }
        "protein" => {
            let futs = parsed_ids
                .iter()
                .map(|id| crate::entities::protein::get(id, &batch_sections));
            return settle_batch(
                "protein",
                &parsed_ids,
                futs,
                json,
                |item| {
                    crate::render::json::to_entity_json_value(
                        item,
                        crate::render::markdown::protein_evidence_urls(item),
                        crate::render::markdown::related_protein(item, &batch_sections),
                        crate::render::provenance::protein_section_sources(item),
                    )
                },
                |item| crate::render::markdown::protein_markdown(item, &batch_sections),
            )
            .await;
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
            return settle_batch(
                "adverse-event",
                &parsed_ids,
                futs,
                json,
                |item| match item {
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
                },
                |item| match item {
                    crate::entities::adverse_event::AdverseEventReport::Faers(report) => {
                        crate::render::markdown::adverse_event_markdown(
                            report,
                            super::super::empty_sections(),
                        )
                    }
                    crate::entities::adverse_event::AdverseEventReport::Device(report) => {
                        crate::render::markdown::device_event_markdown(report)
                    }
                },
            )
            .await;
        }
        other => {
            Err(crate::error::BioMcpError::InvalidArgument(format!(
                "Unknown batch entity '{other}'. Expected one of: gene, variant, article, trial, drug, disease, pgx, pathway, protein, adverse-event"
            ))
            .into())
        }
    }
}

fn sync_outcome(source: &str, message: String, json: bool) -> anyhow::Result<CommandOutcome> {
    let text = if json {
        crate::render::json::to_pretty(&serde_json::json!({
            "kind": "data_sync",
            "source": source,
            "status": "synchronized",
            "changed": true,
        }))?
    } else {
        message
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_ema(cmd: EmaCommand, json: bool) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        EmaCommand::Sync => {
            crate::sources::ema::EmaClient::sync(crate::sources::ema::EmaSyncMode::Force).await?;
            "EMA data synchronized successfully.\n".to_string()
        }
    };
    sync_outcome("ema", text, json)
}

pub(crate) async fn handle_who(cmd: WhoCommand, json: bool) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        WhoCommand::Sync => {
            crate::sources::who_pq::WhoPqClient::sync(crate::sources::who_pq::WhoPqSyncMode::Force)
                .await?;
            "WHO Prequalification data synchronized successfully.\n".to_string()
        }
    };
    sync_outcome("who", text, json)
}

pub(crate) async fn handle_cvx(cmd: CvxCommand, json: bool) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        CvxCommand::Sync => {
            crate::sources::cvx::CvxClient::sync(crate::sources::cvx::CvxSyncMode::Force).await?;
            "CDC CVX/MVX local data bundle synchronized successfully.\n".to_string()
        }
    };
    sync_outcome("cvx", text, json)
}

pub(crate) async fn handle_ddinter(
    cmd: DdinterCommand,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        DdinterCommand::Sync => {
            crate::sources::ddinter::DdinterClient::sync(
                crate::sources::ddinter::DdinterSyncMode::Force,
            )
            .await?;
            "DDInter local interaction data synchronized successfully.\n".to_string()
        }
    };
    sync_outcome("ddinter", text, json)
}

pub(crate) async fn handle_gtr(cmd: GtrCommand, json: bool) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        GtrCommand::Sync => {
            crate::sources::gtr::GtrClient::sync(crate::sources::gtr::GtrSyncMode::Force).await?;
            "GTR local diagnostic data synchronized successfully.\n".to_string()
        }
    };
    sync_outcome("gtr", text, json)
}

pub(crate) async fn handle_who_ivd(
    cmd: WhoIvdCommand,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        WhoIvdCommand::Sync => {
            crate::sources::who_ivd::WhoIvdClient::sync(
                crate::sources::who_ivd::WhoIvdSyncMode::Force,
            )
            .await?;
            "WHO IVD local diagnostic data synchronized successfully.\n".to_string()
        }
    };
    sync_outcome("who_ivd", text, json)
}

#[derive(serde::Serialize)]
pub(super) struct EnrichResponse {
    pub(super) genes: Vec<String>,
    pub(super) unresolved_genes: Vec<String>,
    pub(super) count: usize,
    pub(super) results: Vec<crate::sources::gprofiler::GProfilerTerm>,
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
    let enrichment = crate::sources::gprofiler::GProfilerClient::new()?
        .enrich_genes(&genes, args.limit)
        .await?;
    let text = if json {
        crate::render::json::to_pretty(&EnrichResponse {
            genes,
            unresolved_genes: enrichment.unresolved_genes,
            count: enrichment.terms.len(),
            results: enrichment.terms,
        })?
    } else {
        enrich_markdown(&genes, &enrichment.terms, &enrichment.unresolved_genes)
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_uninstall(json: bool) -> anyhow::Result<CommandOutcome> {
    let message = uninstall_self()?;
    let text = if json {
        crate::render::json::to_pretty(&serde_json::json!({
            "kind": "uninstall",
            "status": "uninstalled",
            "changed": true,
            "message": message,
        }))?
    } else {
        message
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_version(
    args: VersionArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let text = if json {
        version_identity_json()?
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
    let identity = crate::build_identity::current();
    VersionIdentity {
        version: identity.version,
        git_revision: identity.git_revision,
        build_timestamp: identity.build_date,
    }
}

pub(crate) fn version_identity_json() -> Result<String, crate::error::BioMcpError> {
    crate::render::json::to_pretty(&version_identity())
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
    let current = std::fs::canonicalize(std::env::current_exe()?)?;
    #[cfg(windows)]
    {
        return Err(crate::error::BioMcpError::PackageManagedInstall {
            guidance: format!(
                "Automatic uninstall is unsupported on Windows. Remove this standalone installation manually: del \"{}\" && del \"{}\"",
                current.display(),
                crate::cli::install::receipt_path(&current)?.display()
            ),
        });
    }
    #[cfg(unix)]
    {
        uninstall_owned_at(&current)
    }
}

#[cfg(unix)]
pub(super) fn uninstall_owned_at(current: &Path) -> Result<String, crate::error::BioMcpError> {
    let owned = crate::cli::install::validate_owned(current)?;
    let revalidated = crate::cli::install::validate_owned(current)?;
    if owned.receipt != revalidated.receipt {
        return Err(crate::error::BioMcpError::PackageManagedInstall {
            guidance: "The installer receipt changed during uninstall; no files were removed."
                .into(),
        });
    }
    std::fs::remove_file(&owned.executable).map_err(|error| {
        crate::error::BioMcpError::Io(std::io::Error::new(
            error.kind(),
            format!(
                "could not remove owned executable {}; receipt remains at {}: {error}",
                owned.executable.display(),
                owned.receipt_path.display()
            ),
        ))
    })?;
    if let Err(error) = std::fs::remove_file(&owned.receipt_path) {
        return Err(crate::error::BioMcpError::Io(std::io::Error::new(
            error.kind(),
            format!(
                "executable was removed, but receipt remains at {}: {error}",
                owned.receipt_path.display()
            ),
        )));
    }
    Ok(format!(
        "Uninstalled biomcp from {} and removed {}",
        owned.executable.display(),
        owned.receipt_path.display()
    ))
}

pub(super) fn enrich_markdown(
    genes: &[String],
    terms: &[crate::sources::gprofiler::GProfilerTerm],
    unresolved_genes: &[String],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Enrichment: {}\n\n", genes.join(", ")));
    let unresolved = if unresolved_genes.is_empty() {
        "None".to_string()
    } else {
        unresolved_genes.join(", ")
    };
    out.push_str(&format!("Unresolved genes: {unresolved}\n\n"));
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

#[cfg(test)]
mod batch_settlement_tests {
    use super::*;
    use futures::future::BoxFuture;

    #[tokio::test]
    async fn mixed_batch_preserves_input_order_and_emits_every_result() {
        let inputs = ["slow", "bad", "fast"];
        let futures: Vec<BoxFuture<'_, Result<&str, crate::error::BioMcpError>>> = vec![
            Box::pin(async {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                Ok("slow-result")
            }),
            Box::pin(async {
                Err(crate::error::BioMcpError::NotFound {
                    entity: "gene".into(),
                    id: "bad".into(),
                    suggestion: "check the identifier".into(),
                })
            }),
            Box::pin(async { Ok("fast-result") }),
        ];
        let outcome = settle_batch(
            "gene",
            &inputs,
            futures,
            true,
            |value| Ok(serde_json::json!({"value": value})),
            |value| Ok((*value).to_string()),
        )
        .await
        .expect("settled batch");

        assert_eq!(outcome.exit_code, 1);
        assert_eq!(outcome.stream, crate::cli::OutputStream::Stdout);
        let value: serde_json::Value = serde_json::from_str(&outcome.text).expect("batch json");
        assert_eq!(
            value["summary"],
            serde_json::json!({"total":3,"succeeded":2,"failed":1})
        );
        assert_eq!(value["items"][0]["input"], "slow");
        assert_eq!(value["items"][1]["status"], "error");
        assert_eq!(value["items"][2]["input"], "fast");
    }
}
