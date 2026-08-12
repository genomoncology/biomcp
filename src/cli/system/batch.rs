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
