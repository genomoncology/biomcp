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
    if bytes.len() == READ_LIMIT as usize {
        return Err(BioMcpError::InvalidArgument(
            "CAR input exceeds the supported size limit".into(),
        ));
    }
    Ok(bytes)
}

pub(super) async fn handle_single(input: &str, json: bool) -> anyhow::Result<CommandOutcome> {
    let item = crate::entities::variant::normalize_car(input).await?;
    match item.status {
        crate::entities::variant::CarNormalizationStatus::Invalid => {
            return Err(BioMcpError::InvalidArgument(
                item.error
                    .unwrap_or_else(|| "CAR rejected the HGVS input".into()),
            )
            .into());
        }
        crate::entities::variant::CarNormalizationStatus::Indeterminate
        | crate::entities::variant::CarNormalizationStatus::Unavailable => {
            return Err(BioMcpError::SourceUnavailable {
                source_name: "ClinGen Allele Registry".into(),
                reason: item
                    .error
                    .unwrap_or_else(|| "normalization was incomplete".into()),
                suggestion: "Retry the read-only CAR lookup.".into(),
            }
            .into());
        }
        crate::entities::variant::CarNormalizationStatus::Resolved
        | crate::entities::variant::CarNormalizationStatus::NotFound => {}
    }
    if json {
        Ok(CommandOutcome::stdout(crate::render::json::to_pretty(
            &item,
        )?))
    } else {
        Ok(CommandOutcome::stdout(format!(
            "# ClinGen Allele Registry normalization\n\nInput: {}\nStatus: {:?}\nCAid: {}\n",
            item.input,
            item.status,
            item.caid.as_deref().unwrap_or("-")
        )))
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn input_reader_rejects_a_file_larger_than_its_supported_size() {
        let root = crate::test_support::TempDirGuard::new("car-input-limit");
        let path = root.path().join("input.json");
        tokio::fs::write(&path, vec![b' '; 64 * 1024 + 1])
            .await
            .expect("write oversized CAR input");

        let result = read_input(path.to_str().expect("UTF-8 temporary path")).await;

        assert!(
            matches!(result, Err(BioMcpError::InvalidArgument(message)) if message.contains("exceeds"))
        );
    }
}
