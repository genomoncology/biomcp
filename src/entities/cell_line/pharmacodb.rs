//! The PharmacoDB drug-response section and row helper for one cell line.
//!
//! The join starts at the PharmacoDB cross-reference Cellosaurus publishes and
//! falls back to the Cellosaurus name, because Cellosaurus carries no PharmacoDB
//! link for every line PharmacoDB holds: HL-60(TB) (CVCL_A794) is reached by
//! name alone. Either route is accepted only when the PharmacoDB record claims
//! the accession that was asked for.

use crate::entities::cell_line::{CELL_LINE_SECTION_DRUG_RESPONSE, CellLine};
use crate::entities::pharmacodb::{
    PHARMACODB_CELL_LINE_EMPTY_MESSAGE, PHARMACODB_MISMATCH_MESSAGE, PHARMACODB_SOURCE,
    PharmacoCellLineJoin, PharmacoDbCounts, PharmacoDbRowFilter, PharmacoDbRows,
    PharmacoRowPageInput, PharmacoSide, accept_cell_line, build_counts, build_row_page,
    failure_outcome, normalize_dataset, retrieved_now,
};
use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::pharmacodb::{PharmacoCellLine, PharmacoDbClient, PharmacoFilter};

/// What one cell line join and count settled on.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CellLineDrugResponse {
    Counts(PharmacoDbCounts),
    /// PharmacoDB holds no cell line for the accession.
    Missing,
    /// A candidate exists but claims another accession.
    Mismatch,
}

/// Resolve one accession to its PharmacoDB cell line.
///
/// The cross-reference is tried first. A name fallback runs only when no
/// cross-reference answered, and both routes go through the accession check.
pub(crate) async fn resolve_cell_line(
    client: &PharmacoDbClient,
    accession: &str,
    pharmacodb_ids: &[String],
    name: &str,
) -> Result<PharmacoCellLineJoin, BioMcpError> {
    for uid in pharmacodb_ids {
        let candidate = client.cell_line_by_uid(uid).await?;
        match accept_cell_line(accession, candidate) {
            PharmacoCellLineJoin::Missing => continue,
            settled => return Ok(settled),
        }
    }
    if name.trim().is_empty() {
        return Ok(PharmacoCellLineJoin::Missing);
    }
    let candidate = client.cell_line_by_name(name).await?;
    Ok(accept_cell_line(accession, candidate))
}

/// Read the section for one accession. A mismatch makes no experiment request.
pub(crate) async fn load_drug_response_section(
    accession: &str,
    pharmacodb_ids: &[String],
    name: &str,
) -> Result<CellLineDrugResponse, BioMcpError> {
    let client = PharmacoDbClient::new()?;
    let record = match resolve_cell_line(&client, accession, pharmacodb_ids, name).await? {
        PharmacoCellLineJoin::Found(record) => record,
        PharmacoCellLineJoin::Missing => return Ok(CellLineDrugResponse::Missing),
        PharmacoCellLineJoin::Mismatch => return Ok(CellLineDrugResponse::Mismatch),
    };
    let datasets = client
        .experiment_counts(PharmacoFilter::CellLine(record.id))
        .await?;
    Ok(CellLineDrugResponse::Counts(build_counts(
        record.id,
        datasets,
        retrieved_now(),
    )))
}

/// Attach the loaded section and complete its outcome.
pub(crate) fn attach_drug_response_section(
    cell_line: &mut CellLine,
    section: Result<CellLineDrugResponse, BioMcpError>,
) {
    let outcome = match &section {
        Ok(CellLineDrugResponse::Counts(counts)) if counts.total > 0 => {
            SectionOutcome::data(PHARMACODB_SOURCE)
        }
        Ok(CellLineDrugResponse::Counts(_) | CellLineDrugResponse::Missing) => {
            SectionOutcome::empty(PHARMACODB_SOURCE)
        }
        Ok(CellLineDrugResponse::Mismatch) => {
            SectionOutcome::unavailable(PHARMACODB_MISMATCH_MESSAGE)
        }
        Err(error) => {
            tracing::debug!("the PharmacoDB cell line lookup failed: {error}");
            failure_outcome(error)
        }
    };
    cell_line
        .section_outcomes
        .complete(CELL_LINE_SECTION_DRUG_RESPONSE, outcome);
    if let Ok(CellLineDrugResponse::Counts(counts)) = section {
        cell_line.drug_response = Some(counts);
    }
}

/// What the empty section prints.
pub(crate) fn drug_response_empty_message() -> &'static str {
    PHARMACODB_CELL_LINE_EMPTY_MESSAGE
}

/// Read one page of rows for one cell line and one dataset.
///
/// `--dataset` is required, because an unscoped cell line listing is up to 19.4
/// MB of body for one card.
pub(crate) async fn load_drug_response_rows(
    id: &str,
    dataset: &str,
    offset: usize,
    limit: usize,
) -> Result<PharmacoDbRows, BioMcpError> {
    let dataset = normalize_dataset(dataset)?;
    let identity = super::load_pharmacodb_identity(id).await?;
    let client = PharmacoDbClient::new()?;
    let record: PharmacoCellLine = match resolve_cell_line(
        &client,
        &identity.accession,
        &identity.pharmacodb_ids,
        &identity.name,
    )
    .await?
    {
        PharmacoCellLineJoin::Found(record) => record,
        PharmacoCellLineJoin::Missing => {
            return Err(BioMcpError::NotFound {
                entity: "pharmacodb cell line".to_string(),
                id: identity.accession.clone(),
                suggestion: format!("biomcp get cell-line {} xrefs", identity.accession),
            });
        }
        PharmacoCellLineJoin::Mismatch => {
            return Err(BioMcpError::Api {
                api: "pharmacodb".to_string(),
                message: PHARMACODB_MISMATCH_MESSAGE.to_string(),
            });
        }
    };
    let experiments = client
        .experiments(PharmacoFilter::CellLine(record.id))
        .await?;
    Ok(build_row_page(
        &experiments,
        PharmacoRowPageInput {
            subject: identity.accession,
            pharmacodb_id: record.id,
            side: PharmacoSide::CellLine,
            filter: PharmacoDbRowFilter {
                cell_line: None,
                dataset: Some(dataset),
            },
            offset,
            limit,
            data_as_of: retrieved_now(),
        },
    ))
}

#[cfg(test)]
mod tests;
