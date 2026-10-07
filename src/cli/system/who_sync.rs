//! Honest outcome reporting for `biomcp who sync`.
//!
//! Ticket 1304: the command names which export files refreshed and which
//! refreshes failed, never claims bare success after failed refreshes, and
//! exits nonzero when required files are missing after the run. Ticket 2021:
//! a partial run says partial with per-file outcomes and also exits nonzero,
//! because a report that keeps older files is not a synchronized source.
//! Split out of `dispatch.rs` under the CLI line cap (ticket 2010's seam).

use super::WhoCommand;
use crate::cli::CommandOutcome;

/// Renders the per-file sync report as text or JSON. The JSON keeps the
/// shared `data_sync` shape and adds the `refreshed` and `failed` file
/// lists; each `failed` entry carries the file and the reason its refresh
/// failed, so JSON callers see the same detail the terminal does.
pub(super) fn who_sync_outcome(
    report: crate::sources::who_pq::WhoPqSyncReport,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let partial = !report.failed.is_empty();
    let failed = report
        .failed
        .iter()
        .map(|failure| serde_json::json!({"file": failure.file, "reason": failure.reason}))
        .collect::<Vec<_>>();
    let status = if partial { "partial" } else { "synchronized" };
    if json {
        let text = crate::render::json::to_pretty(&serde_json::json!({
            "kind": "data_sync",
            "source": "who",
            "status": status,
            "changed": report.changed,
            "refreshed": report.refreshed,
            "failed": failed,
        }))?;
        return Ok(CommandOutcome::stdout_with_exit(text, u8::from(partial)));
    }

    let failures = report
        .failed
        .iter()
        .map(|failure| format!("{} ({})", failure.file, failure.reason))
        .collect::<Vec<_>>()
        .join(", ");
    let text = if !partial {
        format!(
            "WHO Prequalification data synchronized successfully ({}).\n",
            report.refreshed.join(", ")
        )
    } else if report.refreshed.is_empty() {
        format!(
            "WHO Prequalification data sync incomplete: no files refreshed; refresh failed for {failures}. Existing data files were kept; re-run `biomcp who sync` with network access.\n"
        )
    } else {
        format!(
            "WHO Prequalification data partially synchronized: refreshed {}; refresh failed for {failures}. Existing data files were kept; re-run `biomcp who sync` with network access.\n",
            report.refreshed.join(", ")
        )
    };
    Ok(CommandOutcome::stdout_with_exit(text, u8::from(partial)))
}

pub(crate) async fn handle_who(cmd: WhoCommand, json: bool) -> anyhow::Result<CommandOutcome> {
    let WhoCommand::Sync = cmd;
    let report =
        crate::sources::who_pq::WhoPqClient::sync(crate::sources::who_pq::WhoPqSyncMode::Force)
            .await?;
    who_sync_outcome(report, json)
}
