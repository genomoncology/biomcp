//! The ChEMBL cell line section.
//!
//! ChEMBL names the Cellosaurus accession on its own cell line record, so the
//! join is an exact identifier lookup. The section reports the ChEMBL ID, the
//! EFO and CLO IDs, and how many ChEMBL assays name the line. BioMCP lists no
//! assays and no activity values.

use serde::{Deserialize, Serialize};

use crate::entities::cell_line::{CELL_LINE_SECTION_CHEMBL, CellLine, CellLineChemblRecord};
use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::chembl::{ChemblCellLine, ChemblClient, ChemblRelease};

pub(crate) const CELL_LINE_CHEMBL_SOURCE: &str = "ChEMBL";
/// What the section says when ChEMBL lists no cell line for the accession.
pub(crate) const CELL_LINE_CHEMBL_EMPTY_MESSAGE: &str = "no ChEMBL cell line lists this accession";
const CELL_LINE_CHEMBL_UNAVAILABLE_MESSAGE: &str = "ChEMBL did not answer the cell line lookup";

/// The ChEMBL cell line records for one accession, with the release they came
/// from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellLineChembl {
    pub source: String,
    pub data_as_of: String,
    pub data_as_of_kind: String,
    #[serde(default)]
    pub records: Vec<CellLineChemblRecord>,
}

/// The attribution line the section carries.
pub(crate) fn chembl_attribution(data_as_of: &str, data_as_of_kind: &str) -> String {
    if data_as_of_kind == "release" {
        format!("{data_as_of}, CC BY-SA 3.0. ChEMBL is produced by EMBL-EBI.")
    } else {
        format!("ChEMBL, retrieved {data_as_of}, CC BY-SA 3.0. ChEMBL is produced by EMBL-EBI.")
    }
}

/// The ChEMBL IDs whose assays are counted. No record asks for no count, so an
/// accession ChEMBL does not list makes one request and no more.
pub(crate) fn assay_count_requests(records: &[ChemblCellLine]) -> Vec<String> {
    records
        .iter()
        .map(|record| record.chembl_id.clone())
        .collect()
}

/// Turn the ChEMBL release into the `data_as_of` pair. A failed status read
/// degrades to the retrieval time rather than failing the section.
pub(crate) fn data_as_of_from_release(
    release: Result<ChemblRelease, BioMcpError>,
) -> (String, String) {
    match release {
        Ok(release) => (
            format!("{} ({})", release.version, release.released),
            "release".to_string(),
        ),
        Err(error) => {
            tracing::debug!("ChEMBL status is unavailable: {error}");
            (chrono::Utc::now().to_rfc3339(), "retrieved".to_string())
        }
    }
}

/// Pair each record with its assay count, in ChEMBL order.
pub(crate) fn build_chembl_section(
    records: Vec<ChemblCellLine>,
    counts: &[u64],
    data_as_of: String,
    data_as_of_kind: String,
) -> CellLineChembl {
    let records = records
        .into_iter()
        .zip(counts.iter().copied())
        .map(|(record, assay_count)| CellLineChemblRecord {
            chembl_id: record.chembl_id,
            name: record.name,
            efo_id: record.efo_id,
            clo_id: record.clo_id,
            assay_count,
        })
        .collect();
    CellLineChembl {
        source: CELL_LINE_CHEMBL_SOURCE.to_string(),
        data_as_of,
        data_as_of_kind,
        records,
    }
}

/// Attach the loaded section and complete its outcome. A transport or decode
/// failure completes `unavailable` and renders no row.
pub(crate) fn attach_chembl_section(
    cell_line: &mut CellLine,
    section: Result<CellLineChembl, BioMcpError>,
) {
    match section {
        Ok(section) => {
            let outcome = if section.records.is_empty() {
                SectionOutcome::empty(CELL_LINE_CHEMBL_SOURCE)
            } else {
                SectionOutcome::data(CELL_LINE_CHEMBL_SOURCE)
            };
            cell_line
                .section_outcomes
                .complete(CELL_LINE_SECTION_CHEMBL, outcome);
            cell_line.chembl = Some(section);
        }
        Err(error) => {
            tracing::debug!("the ChEMBL cell line lookup failed: {error}");
            cell_line.section_outcomes.complete(
                CELL_LINE_SECTION_CHEMBL,
                SectionOutcome::unavailable(CELL_LINE_CHEMBL_UNAVAILABLE_MESSAGE),
            );
        }
    }
}

/// Read the section for one Cellosaurus accession.
pub(crate) async fn load_chembl_section(accession: &str) -> Result<CellLineChembl, BioMcpError> {
    let client = ChemblClient::new()?;
    let (data_as_of, data_as_of_kind) = data_as_of_from_release(client.status().await);
    let records = client.cell_line_by_cellosaurus(accession).await?;
    let mut counts = Vec::with_capacity(records.len());
    for chembl_id in assay_count_requests(&records) {
        counts.push(client.cell_line_assay_count(&chembl_id).await?);
    }
    Ok(build_chembl_section(
        records,
        &counts,
        data_as_of,
        data_as_of_kind,
    ))
}

#[cfg(test)]
mod tests;
