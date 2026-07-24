use tokio::io::AsyncReadExt;

use crate::cli::CommandOutcome;
use crate::entities::variant::normalize_car_batch;
use crate::error::BioMcpError;

async fn read_input(path: &str) -> Result<Vec<u8>, BioMcpError> {
    const READ_LIMIT: u64 = 64 * 1024 + 1;
    let mut bytes = Vec::new();
    let read = if path == "-" {
        tokio::io::stdin()
            .take(READ_LIMIT)
            .read_to_end(&mut bytes)
            .await
    } else {
        tokio::fs::File::open(path)
            .await
            .map_err(|_| BioMcpError::InvalidArgument("unable to read CAR input file".into()))?
            .take(READ_LIMIT)
            .read_to_end(&mut bytes)
            .await
    };
    read.map_err(|_| BioMcpError::InvalidArgument("unable to read CAR input".into()))?;
    Ok(bytes)
}

pub(super) async fn handle_batch(input: &str, json: bool) -> anyhow::Result<CommandOutcome> {
    if !json {
        return Err(BioMcpError::InvalidArgument(
            "variant normalize car --input requires --json".into(),
        )
        .into());
    }
    let inputs =
        serde_json::from_slice::<Vec<String>>(&read_input(input).await?).map_err(|_| {
            BioMcpError::InvalidArgument(
                "CAR input must be a bare JSON array of 1-50 HGVS strings".into(),
            )
        })?;
    let response = normalize_car_batch(inputs).await?;
    Ok(CommandOutcome::stdout(crate::render::json::to_pretty(
        &response,
    )?))
}
