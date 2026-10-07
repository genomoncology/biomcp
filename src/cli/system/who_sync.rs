//! Honest outcome reporting for `biomcp who sync`.
//!
//! Ticket 1304: the command names which export files refreshed and which
//! refreshes failed, never claims bare success after failed refreshes, and
//! exits nonzero when required files are missing after the run. Split out
//! of `dispatch.rs` under the CLI line cap (ticket 2010's seam).

use super::WhoCommand;
use crate::cli::CommandOutcome;

/// Renders the per-file sync report as text or JSON. The JSON keeps the
/// shared `data_sync` shape and adds the `refreshed` and `failed` file
/// lists.
pub(super) fn who_sync_outcome(
    report: crate::sources::who_pq::WhoPqSyncReport,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    if json {
        let text = crate::render::json::to_pretty(&serde_json::json!({
            "kind": "data_sync",
            "source": "who",
            "status": "synchronized",
            "changed": report.changed,
            "refreshed": report.refreshed,
            "failed": report.failed,
        }))?;
        return Ok(CommandOutcome::stdout(text));
    }

    let text = if report.failed.is_empty() {
        format!(
            "WHO Prequalification data synchronized successfully ({}).\n",
            report.refreshed.join(", ")
        )
    } else {
        let refreshed = if report.refreshed.is_empty() {
            "no files refreshed".to_string()
        } else {
            format!("refreshed {}", report.refreshed.join(", "))
        };
        format!(
            "WHO Prequalification data synchronized with failures: {refreshed}; refresh failed for {}.\n",
            report.failed.join(", ")
        )
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_who(cmd: WhoCommand, json: bool) -> anyhow::Result<CommandOutcome> {
    let WhoCommand::Sync = cmd;
    let report =
        crate::sources::who_pq::WhoPqClient::sync(crate::sources::who_pq::WhoPqSyncMode::Force)
            .await?;
    who_sync_outcome(report, json)
}
