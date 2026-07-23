use tokio::io::AsyncReadExt;

use crate::cli::CommandOutcome;
use crate::entities::variant::{ERepoBatchInput, retrieve_erepo};
use crate::error::BioMcpError;

async fn read_input(path: &str) -> Result<Vec<u8>, BioMcpError> {
    const READ_LIMIT: u64 = 64 * 1024 + 1;
    let mut bytes = Vec::new();
    if path == "-" {
        tokio::io::stdin()
            .take(READ_LIMIT)
            .read_to_end(&mut bytes)
            .await
    } else {
        tokio::fs::File::open(path)
            .await
            .map_err(|_| BioMcpError::InvalidArgument("unable to read ERepo input file".into()))?
            .take(READ_LIMIT)
            .read_to_end(&mut bytes)
            .await
    }
    .map_err(|_| BioMcpError::InvalidArgument("unable to read ERepo input".into()))?;
    Ok(bytes)
}

pub(super) async fn handle(
    caid: Option<String>,
    input: Option<String>,
    detail: bool,
    assertion: Option<String>,
    version: Option<String>,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    if caid.is_some() && input.is_some() {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo CAid cannot be combined with --input".into(),
        )
        .into());
    }
    if input.is_some() && !json {
        return Err(
            BioMcpError::InvalidArgument("variant erepo --input requires --json".into()).into(),
        );
    }
    let caids = match (caid, input) {
        (Some(caid), None) => vec![caid],
        (None, Some(path)) => serde_json::from_slice::<ERepoBatchInput>(&read_input(&path).await?)
            .map_err(|_| {
                BioMcpError::InvalidArgument(
                    "variant erepo input must be a JSON array or {\"caids\": [...]}".into(),
                )
            })?
            .into_caids(),
        _ => {
            return Err(BioMcpError::InvalidArgument(
                "variant erepo requires a CAid or --input".into(),
            )
            .into());
        }
    };
    if caids.len() > 1 && (detail || assertion.is_some() || version.is_some()) {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo detail selectors are only available for one CAid".into(),
        )
        .into());
    }
    let response = retrieve_erepo(caids, detail, assertion.as_deref(), version.as_deref()).await?;
    Ok(CommandOutcome::stdout(crate::render::json::to_pretty(
        &response,
    )?))
}
