use crate::cli::CommandOutcome;
use futures::stream::{self, StreamExt};

use super::{ArticleBatchMode, BatchArgs};

pub(crate) const BATCH_MAX_IN_FLIGHT: usize = 10;

pub(crate) struct PreparedBatch<'a> {
    pub(crate) entity: String,
    pub(crate) ids: Vec<&'a str>,
    pub(crate) sections: Vec<String>,
    pub(crate) trial_source: Option<crate::entities::trial::TrialSource>,
}

pub(crate) fn preflight_batch(
    args: &BatchArgs,
) -> Result<PreparedBatch<'_>, crate::error::BioMcpError> {
    validate_batch_args(args)?;
    let entity = args.entity.trim().to_ascii_lowercase();
    let supported = matches!(
        entity.as_str(),
        "gene"
            | "variant"
            | "article"
            | "trial"
            | "drug"
            | "disease"
            | "pgx"
            | "pathway"
            | "protein"
            | "adverse-event"
            | "adverse_event"
            | "adverseevent"
    );
    if !supported {
        return Err(crate::error::BioMcpError::InvalidArgument(format!(
            "Unknown batch entity '{entity}'. Expected one of: gene, variant, article, trial, drug, disease, pgx, pathway, protein, adverse-event"
        )));
    }
    let ids = validate_batch_ids(args, &entity)?;
    let sections = super::dispatch::parse_batch_sections(args.sections.as_deref());
    if matches!(
        entity.as_str(),
        "adverse-event" | "adverse_event" | "adverseevent"
    ) && !sections.is_empty()
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "Batch sections are not supported for adverse-event".into(),
        ));
    }
    let trial_source = (entity == "trial")
        .then(|| {
            crate::entities::trial::TrialSource::from_flag(
                args.source.as_deref().unwrap_or("ctgov"),
            )
        })
        .transpose()?;
    Ok(PreparedBatch {
        entity,
        ids,
        sections,
        trial_source,
    })
}

pub(crate) fn validate_batch_args(args: &BatchArgs) -> Result<(), crate::error::BioMcpError> {
    let entity = args.entity.trim().to_ascii_lowercase();
    if entity != "trial" && args.source.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--source is only supported for trial batches".into(),
        ));
    }
    if entity != "article" && args.mode.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--mode is only supported for article batches".into(),
        ));
    }
    if entity != "article" && args.pdf {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--pdf is only supported for article batches".into(),
        ));
    }
    if entity == "article" && args.pdf && args.mode == Some(ArticleBatchMode::Compact) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--pdf is not supported for compact article batches".into(),
        ));
    }
    if entity == "article"
        && args.mode == Some(ArticleBatchMode::Compact)
        && args.sections.is_some()
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--sections is not supported for compact article batches".into(),
        ));
    }
    if entity == "article"
        && args.mode != Some(ArticleBatchMode::Compact)
        && let Some(sections) = args.sections.as_deref()
    {
        validate_article_batch_sections(sections)?;
    }
    if entity == "article"
        && args.pdf
        && args.mode != Some(ArticleBatchMode::Compact)
        && !args
            .sections
            .as_deref()
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .any(|section| matches!(section.to_ascii_lowercase().as_str(), "fulltext" | "all"))
    {
        // The shared detail path refuses --pdf without the fulltext section;
        // preflight mirrors it so the refusal happens before any provider
        // request, like every other batch argument check.
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--pdf requires the fulltext section (example: biomcp batch article 22663011,22663012 --mode detail --sections fulltext --pdf)"
                .into(),
        ));
    }
    Ok(())
}

fn validate_article_batch_sections(raw: &str) -> Result<(), crate::error::BioMcpError> {
    let sections = raw.split(',').map(str::trim).collect::<Vec<_>>();
    if sections.iter().any(|section| section.is_empty()) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "Article batch sections must be a comma-separated list of nonempty section names"
                .into(),
        ));
    }
    let mut index = 0;
    while index < sections.len() {
        let section = sections[index].to_ascii_lowercase();
        index += 1;
        if section == "assets" || section == "asset" {
            // Batch sections resolve identically for every requested ID; the
            // asset surface is per-article (keys, destinations, view paging),
            // so it stays on `get article` only.
            return Err(crate::error::BioMcpError::InvalidArgument(format!(
                "Article batch does not support the \"{section}\" section; asset lists and downloads need get article (example: biomcp get article 22663011 assets). Supported article batch sections: annotations, fulltext, indexing, tldr, all"
            )));
        }
        if !crate::entities::article::ARTICLE_SECTION_NAMES.contains(&section.as_str()) {
            return Err(crate::error::BioMcpError::InvalidArgument(format!(
                "Unknown section \"{section}\" for article. Available: {}",
                crate::entities::article::ARTICLE_SECTION_NAMES.join(", ")
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_batch_ids<'a>(
    args: &'a BatchArgs,
    entity: &str,
) -> Result<Vec<&'a str>, crate::error::BioMcpError> {
    let parsed_ids = args
        .ids
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if parsed_ids.is_empty() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "Batch IDs are required. Example: biomcp batch gene BRAF,TP53".into(),
        ));
    }
    let max_ids = if entity == "article" && args.mode == Some(ArticleBatchMode::Compact) {
        crate::entities::article::ARTICLE_BATCH_MAX_IDS
    } else {
        10
    };
    if parsed_ids.len() > max_ids {
        return Err(crate::error::BioMcpError::InvalidArgument(format!(
            "Batch is limited to {max_ids} IDs"
        )));
    }
    validate_batch_id_lengths(parsed_ids.iter().copied(), "Batch")?;
    Ok(parsed_ids)
}

pub(crate) fn validate_batch_id_lengths<'a>(
    ids: impl IntoIterator<Item = &'a str>,
    label: &str,
) -> Result<(), crate::error::BioMcpError> {
    if let Some(id) = ids.into_iter().find(|id| id.len() > 512) {
        return Err(crate::error::BioMcpError::InvalidArgument(format!(
            "{label} ID is limited to 512 UTF-8 bytes (received {})",
            id.len()
        )));
    }
    Ok(())
}

pub(crate) fn validate_compatibility_article_batch_ids(
    ids: &[String],
) -> Result<(), crate::error::BioMcpError> {
    if ids.len() > crate::entities::article::ARTICLE_BATCH_MAX_IDS {
        return Err(crate::error::BioMcpError::InvalidArgument(format!(
            "Article batch is limited to {} IDs",
            crate::entities::article::ARTICLE_BATCH_MAX_IDS
        )));
    }
    validate_batch_id_lengths(ids.iter().map(String::as_str), "Article batch")
}

async fn settle_ordered<Fut, T>(
    futures: impl IntoIterator<Item = Fut>,
) -> Vec<Result<T, crate::error::BioMcpError>>
where
    Fut: std::future::Future<Output = Result<T, crate::error::BioMcpError>>,
{
    let mut settled = stream::iter(futures.into_iter().enumerate())
        .map(|(index, future)| async move { (index, future.await) })
        .buffer_unordered(BATCH_MAX_IN_FLIGHT)
        .collect::<Vec<_>>()
        .await;
    settled.sort_by_key(|(index, _)| *index);
    settled.into_iter().map(|(_, result)| result).collect()
}

pub(crate) async fn settle_batch<T, Fut, Project, Human>(
    entity: &str,
    inputs: &[&str],
    futures: impl IntoIterator<Item = Fut>,
    json: bool,
    project: Project,
    human: Human,
) -> anyhow::Result<CommandOutcome>
where
    Fut: std::future::Future<Output = Result<T, crate::error::BioMcpError>>,
    Project: Fn(&T) -> Result<serde_json::Value, crate::error::BioMcpError>,
    Human: Fn(&T) -> Result<String, crate::error::BioMcpError>,
{
    let settled = settle_ordered(futures).await;
    let failed = settled.iter().filter(|result| result.is_err()).count();
    let succeeded = settled.len().saturating_sub(failed);
    let text = if json {
        let items = inputs
            .iter()
            .zip(&settled)
            .map(|(input, result)| match result {
                Ok(value) => Ok(serde_json::json!({
                    "input": input, "status": "ok", "result": project(value)?,
                })),
                Err(error) => {
                    let value: serde_json::Value =
                        serde_json::from_str(&crate::render::json::to_error_json(error)?)?;
                    Ok(serde_json::json!({
                        "input": input, "status": "error", "error": value["error"],
                    }))
                }
            })
            .collect::<Result<Vec<_>, crate::error::BioMcpError>>()?;
        crate::render::json::to_pretty(&serde_json::json!({
            "summary": {"total": settled.len(), "succeeded": succeeded, "failed": failed},
            "items": items,
        }))?
    } else {
        let mut out = format!("# Batch: {entity} ({})\n", settled.len());
        for (input, result) in inputs.iter().zip(&settled) {
            out.push_str("\n---\n\n");
            match result {
                Ok(value) => out.push_str(&format!("## {input} — ok\n\n{}", human(value)?)),
                Err(error) => out.push_str(&format!(
                    "## {input} — error\n\n{}\n",
                    error.public_projection().message
                )),
            }
        }
        out.push_str(&format!(
            "\n## Summary\n\nTotal: {}; succeeded: {}; failed: {}.\n",
            settled.len(),
            succeeded,
            failed
        ));
        out
    };
    Ok(CommandOutcome::stdout_with_exit(text, u8::from(failed > 0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Commands};
    use clap::Parser;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Semaphore;

    struct ActiveGuard {
        active: Arc<AtomicUsize>,
        cancelled: Arc<AtomicUsize>,
        completed: bool,
    }

    fn args(
        entity: &str,
        ids: String,
        mode: Option<ArticleBatchMode>,
        sections: Option<&str>,
    ) -> BatchArgs {
        args_with_pdf(entity, ids, mode, sections, false)
    }

    fn args_with_pdf(
        entity: &str,
        ids: String,
        mode: Option<ArticleBatchMode>,
        sections: Option<&str>,
        pdf: bool,
    ) -> BatchArgs {
        BatchArgs {
            entity: entity.into(),
            ids,
            mode,
            sections: sections.map(str::to_string),
            source: None,
            pdf,
        }
    }

    #[test]
    fn batch_command_parses_sections_source_and_closed_article_modes() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "batch",
            "trial",
            "NCT02576665,NCT02693535",
            "--sections",
            "eligibility,locations",
            "--source",
            "nci",
        ])
        .expect("batch should parse");
        let Commands::Batch(parsed) = cli.command else {
            panic!("expected batch command")
        };
        assert_eq!(parsed.mode, None);
        assert_eq!(parsed.sections.as_deref(), Some("eligibility,locations"));
        assert_eq!(parsed.source.as_deref(), Some("nci"));

        for (raw, expected) in [
            ("compact", ArticleBatchMode::Compact),
            ("detail", ArticleBatchMode::Detail),
        ] {
            let cli = Cli::try_parse_from([
                "biomcp",
                "batch",
                "article",
                "22663011,24200969",
                "--mode",
                raw,
            ])
            .expect("article mode should parse");
            let Commands::Batch(parsed) = cli.command else {
                panic!("expected batch command")
            };
            assert_eq!(parsed.mode, Some(expected));
        }
        let error = Cli::try_parse_from([
            "biomcp", "batch", "article", "22663011", "--mode", "summary",
        ])
        .expect_err("unsupported modes must fail in clap");
        assert!(error.to_string().contains("invalid value 'summary'"));

        let cli = Cli::try_parse_from([
            "biomcp",
            "batch",
            "article",
            "22663011,24200969",
            "--mode",
            "detail",
            "--sections",
            "fulltext",
            "--pdf",
        ])
        .expect("article batch --pdf should parse");
        let Commands::Batch(parsed) = cli.command else {
            panic!("expected batch command")
        };
        assert!(parsed.pdf);
        assert_eq!(parsed.sections.as_deref(), Some("fulltext"));
    }

    #[test]
    fn article_batch_preflight_pins_modes_sections_counts_and_id_bytes() {
        for entity in [
            "gene",
            "variant",
            "trial",
            "drug",
            "disease",
            "phenotype",
            "pgx",
            "pathway",
            "protein",
            "adverse-event",
        ] {
            let value = args(entity, "one".into(), Some(ArticleBatchMode::Compact), None);
            assert!(
                validate_batch_args(&value)
                    .expect_err("mode must be article-only")
                    .to_string()
                    .contains("--mode is only supported for article batches")
            );
        }
        let compact_sections = args(
            "article",
            "one".into(),
            Some(ArticleBatchMode::Compact),
            Some(""),
        );
        assert!(
            validate_batch_args(&compact_sections)
                .expect_err("even empty sections fail in compact mode")
                .to_string()
                .contains("--sections is not supported")
        );
        validate_batch_args(&args(
            "article",
            "one".into(),
            Some(ArticleBatchMode::Detail),
            Some("tldr"),
        ))
        .expect("detail accepts sections");
        for sections in ["", " ", "tldr,,annotations", "tldr,unknown", "--pdf"] {
            let value = args(
                "article",
                "one".into(),
                Some(ArticleBatchMode::Detail),
                Some(sections),
            );
            assert!(
                validate_batch_args(&value).is_err(),
                "malformed detail sections must fail preflight: {sections:?}"
            );
        }
        for (mode, sections) in [
            (Some(ArticleBatchMode::Detail), "fulltext,assets"),
            (Some(ArticleBatchMode::Detail), "assets"),
            (Some(ArticleBatchMode::Detail), "asset,traces-s1.csv"),
            (None, "assets"),
        ] {
            let value = args("article", "one".into(), mode, Some(sections));
            let error =
                validate_batch_args(&value).expect_err("batch never supports asset sections");
            let rendered = error.to_string();
            assert!(
                rendered.contains("Article batch does not support"),
                "sections={sections:?}: {rendered}"
            );
            assert!(
                rendered.contains("annotations, fulltext, indexing, tldr, all"),
                "sections={sections:?}: {rendered}"
            );
        }
        validate_batch_args(&args(
            "article",
            "one".into(),
            None,
            Some("tldr,tldr,annotations"),
        ))
        .expect("duplicate known sections retain ordinary article semantics");

        // --pdf rides only on article detail batches that asked for fulltext.
        for entity in ["gene", "variant", "trial"] {
            let value = args_with_pdf(entity, "one".into(), None, None, true);
            assert!(
                validate_batch_args(&value)
                    .expect_err("pdf is article-only")
                    .to_string()
                    .contains("--pdf is only supported for article batches")
            );
        }
        assert!(
            validate_batch_args(&args_with_pdf(
                "article",
                "one".into(),
                Some(ArticleBatchMode::Compact),
                None,
                true,
            ))
            .expect_err("compact refuses pdf")
            .to_string()
            .contains("--pdf is not supported for compact article batches")
        );
        for sections in [None, Some("tldr"), Some("tldr,indexing")] {
            let value = args_with_pdf(
                "article",
                "one".into(),
                Some(ArticleBatchMode::Detail),
                sections,
                true,
            );
            assert!(
                validate_batch_args(&value)
                    .expect_err("pdf needs the fulltext section")
                    .to_string()
                    .contains("--pdf requires the fulltext section"),
                "sections={sections:?}"
            );
        }
        for sections in ["fulltext", "FULLTEXT", "fulltext,tldr", "all"] {
            validate_batch_args(&args_with_pdf(
                "article",
                "one".into(),
                Some(ArticleBatchMode::Detail),
                Some(sections),
                true,
            ))
            .unwrap_or_else(|error| panic!("pdf accepts {sections:?}: {error}"));
            validate_batch_args(&args_with_pdf(
                "article",
                "one".into(),
                None,
                Some(sections),
                true,
            ))
            .unwrap_or_else(|error| panic!("pdf accepts default detail {sections:?}: {error}"));
        }

        for (count, mode, accepted) in [
            (1, None, true),
            (10, None, true),
            (11, None, false),
            (20, Some(ArticleBatchMode::Compact), true),
            (21, Some(ArticleBatchMode::Compact), false),
        ] {
            let value = args(
                "article",
                (0..count)
                    .map(|i| format!("id{i}"))
                    .collect::<Vec<_>>()
                    .join(","),
                mode,
                None,
            );
            assert_eq!(validate_batch_ids(&value, "article").is_ok(), accepted);
        }
        let boundary = args(
            "article",
            format!(" ,{},, ", "é".repeat(256)),
            Some(ArticleBatchMode::Compact),
            None,
        );
        let ids = validate_batch_ids(&boundary, "article").expect("512 bytes accepted");
        assert_eq!((ids.len(), ids[0].len()), (1, 512));
        assert!(
            validate_batch_ids(
                &args(
                    "article",
                    format!("{}x", "é".repeat(256)),
                    Some(ArticleBatchMode::Compact),
                    None,
                ),
                "article"
            )
            .is_err()
        );
        assert!(
            validate_batch_ids(
                &args(
                    "article",
                    " , , ".into(),
                    Some(ArticleBatchMode::Compact),
                    None,
                ),
                "article"
            )
            .is_err()
        );
        for flag in ["--limit", "--offset", "--page", "--cursor"] {
            let error = Cli::try_parse_from(["biomcp", "batch", "article", "22663011", flag, "1"])
                .expect_err("pagination-like flags stay unsupported");
            assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
        }
    }

    impl Drop for ActiveGuard {
        fn drop(&mut self) {
            self.active.fetch_sub(1, Ordering::SeqCst);
            if !self.completed {
                self.cancelled.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    #[tokio::test]
    async fn settlement_caps_live_work_preserves_order_and_settles_after_failure() {
        let permits = Arc::new(Semaphore::new(0));
        let started = Arc::new(AtomicUsize::new(0));
        let started_order = Arc::new(Mutex::new(Vec::new()));
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicUsize::new(0));
        let futures = (0..12).map(|index| {
            let permits = Arc::clone(&permits);
            let started = Arc::clone(&started);
            let started_order = Arc::clone(&started_order);
            let active = Arc::clone(&active);
            let maximum = Arc::clone(&maximum);
            let cancelled = Arc::clone(&cancelled);
            async move {
                started_order.lock().expect("start order lock").push(index);
                started.fetch_add(1, Ordering::SeqCst);
                let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                maximum.fetch_max(now, Ordering::SeqCst);
                let mut guard = ActiveGuard {
                    active,
                    cancelled,
                    completed: false,
                };
                permits
                    .acquire()
                    .await
                    .expect("semaphore remains open")
                    .forget();
                guard.completed = true;
                if index == 3 {
                    Err(crate::error::BioMcpError::InvalidArgument(
                        "fixture failure".into(),
                    ))
                } else {
                    Ok(index)
                }
            }
        });
        let settlement = settle_ordered(futures);
        tokio::pin!(settlement);
        while started.load(Ordering::SeqCst) < BATCH_MAX_IN_FLIGHT {
            tokio::select! {
                biased;
                result = &mut settlement => panic!("settled before permits: {result:?}"),
                _ = tokio::task::yield_now() => {}
            }
        }
        assert_eq!(started.load(Ordering::SeqCst), 10);
        assert_eq!(
            *started_order.lock().expect("start order lock"),
            (0..10).collect::<Vec<_>>()
        );
        assert_eq!(maximum.load(Ordering::SeqCst), 10);
        permits.add_permits(12);
        let results = settlement.await;
        assert_eq!(started.load(Ordering::SeqCst), 12);
        assert_eq!(
            *started_order.lock().expect("start order lock"),
            (0..12).collect::<Vec<_>>()
        );
        assert_eq!(maximum.load(Ordering::SeqCst), 10);
        assert_eq!(cancelled.load(Ordering::SeqCst), 0);
        assert_eq!(results.len(), 12);
        assert!(results[3].is_err());
        assert_eq!(results[11].as_ref().expect("last item settled"), &11);
    }

    #[tokio::test]
    async fn dropping_settlement_cancels_active_and_queued_work() {
        let started = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicUsize::new(0));
        let futures = (0..12).map(|_| {
            let started = Arc::clone(&started);
            let active = Arc::clone(&active);
            let cancelled = Arc::clone(&cancelled);
            async move {
                started.fetch_add(1, Ordering::SeqCst);
                active.fetch_add(1, Ordering::SeqCst);
                let _guard = ActiveGuard {
                    active,
                    cancelled,
                    completed: false,
                };
                tokio::time::sleep(std::time::Duration::from_secs(3_600)).await;
                Ok(())
            }
        });
        let mut settlement = Box::pin(settle_ordered(futures));
        while started.load(Ordering::SeqCst) < BATCH_MAX_IN_FLIGHT {
            tokio::select! {
                biased;
                result = &mut settlement => panic!("pending settlement completed: {result:?}"),
                _ = tokio::task::yield_now() => {}
            }
        }
        assert_eq!(active.load(Ordering::SeqCst), 10);
        drop(settlement);
        assert_eq!(started.load(Ordering::SeqCst), 10);
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert_eq!(cancelled.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn dropping_settlement_cancels_cacheable_article_provider_retry_and_admission() {
        use tokio::io::AsyncWriteExt;

        type EnvRestore = crate::test_support::FixtureEnv;

        let requests = Arc::new(AtomicUsize::new(0));
        let first_response = Arc::new(tokio::sync::Notify::new());
        let active = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicUsize::new(0));
        let admitted = Arc::new(AtomicUsize::new(0));
        let cache_gets = Arc::new(AtomicUsize::new(0));
        let cache_puts = Arc::new(AtomicUsize::new(0));
        let mut server = {
            let requests = Arc::clone(&requests);
            let first_response = Arc::clone(&first_response);
            crate::test_support::TestHttpServer::spawn(move |mut stream| {
let requests = requests.clone();
let first_response = first_response.clone();
async move {

                    let Some(request) = crate::test_support::read_request(&mut stream).await? else { return Ok(()); };
                    requests.fetch_add(1, Ordering::SeqCst);
                    stream
                        .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                        .await?;
                    first_response.notify_one();

Ok(())
}
}).expect("start cancellation fixture")
        };
        let url = server.base.clone();
        let cache = crate::test_support::TempDirGuard::new("article-batch-cancel-cache");
        let mut env = EnvRestore::scratch_cache_only();
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &url);
        env.set("BIOMCP_S2_BASE", &url);
        env.set_cache_root(cache.path());
        let client =
            crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
                &url,
                {
                    let cache_gets = Arc::clone(&cache_gets);
                    move |_, _| {
                        cache_gets.fetch_add(1, Ordering::SeqCst);
                    }
                },
                {
                    let cache_puts = Arc::clone(&cache_puts);
                    move |_, _| {
                        cache_puts.fetch_add(1, Ordering::SeqCst);
                    }
                },
            )
            .expect("article provider retry/cache client");
        let item = {
            let active = Arc::clone(&active);
            let cancelled = Arc::clone(&cancelled);
            let admitted = Arc::clone(&admitted);
            async move {
                admitted.fetch_add(1, Ordering::SeqCst);
                active.fetch_add(1, Ordering::SeqCst);
                let _guard = ActiveGuard {
                    active,
                    cancelled,
                    completed: false,
                };
                client.paper_detail("PMID:22663011").await.map(|_| ())
            }
        };
        let mut settlement = Box::pin(settle_ordered([item]));
        tokio::select! {
            biased;
            () = first_response.notified() => {},
            result = &mut settlement => panic!("retrying item settled early: {result:?}"),
        }
        assert_eq!(admitted.load(Ordering::SeqCst), 1);
        assert_eq!(active.load(Ordering::SeqCst), 1);
        assert_eq!(cache_gets.load(Ordering::SeqCst), 1);
        assert_eq!(cache_puts.load(Ordering::SeqCst), 0);
        drop(settlement);
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert_eq!(cancelled.load(Ordering::SeqCst), 1);
        server.finish().expect("cancellation fixture teardown");
        tokio::time::pause();
        tokio::time::advance(std::time::Duration::from_secs(10)).await;
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }
        assert_eq!(admitted.load(Ordering::SeqCst), 1);
        assert_eq!(requests.load(Ordering::SeqCst), 1);
        assert_eq!(cache_gets.load(Ordering::SeqCst), 1);
        assert_eq!(cache_puts.load(Ordering::SeqCst), 0);
    }
}
