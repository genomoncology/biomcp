use crate::cli::CommandOutcome;
use futures::future::join_all;

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
    let settled = join_all(futures).await;
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
    use futures::future::BoxFuture;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn ten_items_start_together_and_each_owns_its_deadline() {
        let inputs = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
        let started = Arc::new(AtomicUsize::new(0));
        let futures: Vec<BoxFuture<'_, Result<usize, crate::error::BioMcpError>>> = (0usize..10)
            .map(|index| {
                let started = Arc::clone(&started);
                Box::pin(async move {
                    started.fetch_add(1, Ordering::SeqCst);
                    while started.load(Ordering::SeqCst) != 10 {
                        tokio::task::yield_now().await;
                    }
                    tokio::time::timeout(Duration::from_millis(30), async {
                        tokio::time::sleep(Duration::from_millis(if index == 0 { 80 } else { 2 }))
                            .await;
                        index
                    })
                    .await
                    .map_err(|_| {
                        crate::error::BioMcpError::SourceUnavailable {
                            source_name: "GenCC".into(),
                            reason: "per-gene deadline".into(),
                            suggestion: "retry".into(),
                        }
                    })
                }) as BoxFuture<'_, _>
            })
            .collect();
        let began = Instant::now();
        let outcome = settle_batch(
            "gene",
            &inputs,
            futures,
            true,
            |value| Ok((*value).into()),
            |value| Ok(value.to_string()),
        )
        .await
        .unwrap();
        assert!(began.elapsed() < Duration::from_millis(100));
        let value: serde_json::Value = serde_json::from_str(&outcome.text).unwrap();
        assert_eq!(
            value["summary"],
            serde_json::json!({"total":10,"succeeded":9,"failed":1})
        );
        assert_eq!(value["items"][9]["result"], 9);
    }
}
