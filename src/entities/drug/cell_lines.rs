//! The PharmacoDB cell line section and row helper for one drug.
//!
//! `Drug` carries no PubChem ID and PharmacoDB publishes no ChEMBL or DrugBank
//! key on its compound records, so the join goes by name. PharmacoDB matches a
//! compound name ignoring case, and a candidate is accepted only when the name
//! it returns is the name that was asked for.

use crate::entities::drug::Drug;
use crate::entities::pharmacodb::{
    PHARMACODB_COMPOUND_EMPTY_MESSAGE, PHARMACODB_SOURCE, PharmacoDbCounts, PharmacoDbRowFilter,
    PharmacoDbRows, PharmacoRowPageInput, PharmacoSide, build_counts, build_row_page,
    compound_name_matches, failure_outcome, normalize_dataset, retrieved_now,
};
use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::pharmacodb::{PharmacoCompound, PharmacoDbClient, PharmacoFilter};

use super::DRUG_SECTION_CELL_LINES;

/// What one compound join and count settled on.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DrugCellLines {
    Counts(PharmacoDbCounts),
    /// PharmacoDB holds no compound with this name.
    Missing,
}

/// Resolve one drug name to its PharmacoDB compound.
///
/// The resolved name is tried first. A different requested name is tried once
/// after a miss, so a brand name the resolver rewrote still has a route.
pub(crate) async fn resolve_compound(
    client: &PharmacoDbClient,
    resolved_name: &str,
    requested_name: &str,
) -> Result<Option<PharmacoCompound>, BioMcpError> {
    let mut queries: Vec<&str> = Vec::new();
    for query in [resolved_name.trim(), requested_name.trim()] {
        if query.is_empty() || queries.iter().any(|seen| seen.eq_ignore_ascii_case(query)) {
            continue;
        }
        queries.push(query);
    }
    for query in queries {
        if let Some(record) = client.compound_by_name(query).await?
            && compound_name_matches(query, &record.name)
        {
            return Ok(Some(record));
        }
    }
    Ok(None)
}

/// Read the section for one drug name.
pub(crate) async fn load_cell_lines_section(
    resolved_name: &str,
    requested_name: &str,
) -> Result<DrugCellLines, BioMcpError> {
    let client = PharmacoDbClient::new()?;
    let Some(record) = resolve_compound(&client, resolved_name, requested_name).await? else {
        return Ok(DrugCellLines::Missing);
    };
    let datasets = client
        .experiment_counts(PharmacoFilter::Compound(record.id))
        .await?;
    Ok(DrugCellLines::Counts(build_counts(
        record.id,
        datasets,
        retrieved_now(),
    )))
}

/// Attach the loaded section and complete its outcome.
pub(crate) fn attach_cell_lines_section(
    drug: &mut Drug,
    section: Result<DrugCellLines, BioMcpError>,
) {
    let outcome = match &section {
        Ok(DrugCellLines::Counts(counts)) if counts.total > 0 => {
            SectionOutcome::data(PHARMACODB_SOURCE)
        }
        Ok(DrugCellLines::Counts(_) | DrugCellLines::Missing) => {
            SectionOutcome::empty(PHARMACODB_SOURCE)
        }
        Err(error) => {
            tracing::debug!("the PharmacoDB compound lookup failed: {error}");
            failure_outcome(error)
        }
    };
    drug.section_outcomes
        .complete(DRUG_SECTION_CELL_LINES, outcome);
    if let Ok(DrugCellLines::Counts(counts)) = section {
        drug.cell_lines = Some(counts);
    }
}

/// What the empty section prints.
pub(crate) fn cell_lines_empty_message() -> &'static str {
    PHARMACODB_COMPOUND_EMPTY_MESSAGE
}

/// Which rows `drug cell-lines` was asked for. One of the two is required,
/// because an unscoped drug listing is up to 18.6 MB of body for one command.
#[derive(Debug)]
pub(crate) enum DrugCellLinesRequest<'a> {
    /// One drug on one cell line: 0 to 3 rows in practice.
    Pair { cell_line: &'a str },
    /// One drug across one dataset.
    Dataset { dataset: &'a str },
}

impl<'a> DrugCellLinesRequest<'a> {
    /// Read the two helper arguments. Neither one is a failure before any
    /// request is made, and the message names the command that prints counts.
    pub(crate) fn from_args(
        cell_line: Option<&'a str>,
        dataset: Option<&'a str>,
    ) -> Result<Self, BioMcpError> {
        match (cell_line, dataset) {
            (Some(cell_line), None) => Ok(Self::Pair { cell_line }),
            (None, Some(dataset)) => Ok(Self::Dataset { dataset }),
            (Some(_), Some(_)) => Err(BioMcpError::InvalidArgument(
                "Use --cell-line or --dataset for drug cell-lines, not both.".into(),
            )),
            (None, None) => Err(BioMcpError::InvalidArgument(
                "drug cell-lines requires --cell-line <id> or --dataset <name>. \
For the counts per dataset run `biomcp get drug <name> cell_lines`."
                    .into(),
            )),
        }
    }
}

/// Read one page of rows for one drug.
pub(crate) async fn load_cell_lines_rows(
    name: &str,
    request: DrugCellLinesRequest<'_>,
    offset: usize,
    limit: usize,
) -> Result<PharmacoDbRows, BioMcpError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "A drug name is required.".into(),
        ));
    }
    // Both arguments are checked before any request is made.
    let filter = match &request {
        DrugCellLinesRequest::Dataset { dataset } => PharmacoDbRowFilter {
            cell_line: None,
            dataset: Some(normalize_dataset(dataset)?),
        },
        DrugCellLinesRequest::Pair { cell_line } => PharmacoDbRowFilter {
            cell_line: Some((*cell_line).to_string()),
            dataset: None,
        },
    };

    let client = PharmacoDbClient::new()?;
    let Some(compound) = resolve_compound(&client, name, name).await? else {
        return Err(BioMcpError::NotFound {
            entity: "pharmacodb compound".to_string(),
            id: name.to_string(),
            suggestion: format!("biomcp get drug {name}"),
        });
    };

    let (experiments, filter) = match request {
        DrugCellLinesRequest::Dataset { .. } => (
            client
                .experiments(PharmacoFilter::Compound(compound.id))
                .await?,
            filter,
        ),
        DrugCellLinesRequest::Pair { cell_line } => {
            let identity = crate::entities::cell_line::load_pharmacodb_identity(cell_line).await?;
            let record = match crate::entities::cell_line::pharmacodb::resolve_cell_line(
                &client,
                &identity.accession,
                &identity.pharmacodb_ids,
                &identity.name,
            )
            .await?
            {
                crate::entities::pharmacodb::PharmacoCellLineJoin::Found(record) => record,
                crate::entities::pharmacodb::PharmacoCellLineJoin::Missing => {
                    return Err(BioMcpError::NotFound {
                        entity: "pharmacodb cell line".to_string(),
                        id: identity.accession.clone(),
                        suggestion: format!("biomcp get cell-line {} xrefs", identity.accession),
                    });
                }
                crate::entities::pharmacodb::PharmacoCellLineJoin::Mismatch => {
                    return Err(BioMcpError::Api {
                        api: "pharmacodb".to_string(),
                        message: crate::entities::pharmacodb::PHARMACODB_MISMATCH_MESSAGE
                            .to_string(),
                    });
                }
            };
            (
                client
                    .experiments(PharmacoFilter::Pair {
                        cell_line_id: record.id,
                        compound_id: compound.id,
                    })
                    .await?,
                PharmacoDbRowFilter {
                    cell_line: Some(identity.accession),
                    dataset: None,
                },
            )
        }
    };

    Ok(build_row_page(
        &experiments,
        PharmacoRowPageInput {
            subject: compound.name.clone(),
            pharmacodb_id: compound.id,
            side: PharmacoSide::Drug,
            filter,
            offset,
            limit,
            data_as_of: retrieved_now(),
        },
    ))
}

#[cfg(test)]
mod tests;
