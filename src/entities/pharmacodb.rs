//! Published PharmacoDB drug-response numbers, shared by the drug and cell line
//! entities.
//!
//! Both sections report how many experiments each dataset holds and list no
//! rows, because two measured inputs (K-562 and doxorubicin) are over 18 MB.
//! Rows come from a drug-and-cell-line pair or from one side with a `--dataset`
//! filter. BioMCP prints every metric as PharmacoDB published it: no units are
//! added, no value is rounded in JSON, repeated experiments stay separate, and
//! nothing is ranked, thresholded, averaged, or labelled sensitive or resistant.

use serde::{Deserialize, Serialize};

use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::pharmacodb::{
    PHARMACODB_BODY_LIMIT_MESSAGE, PHARMACODB_DATASET_NAMES, PharmacoCellLine,
    PharmacoDatasetCount, PharmacoExperiment,
};

pub(crate) const PHARMACODB_SOURCE: &str = "PharmacoDB";
/// What the cell line section says when PharmacoDB holds no such line.
pub(crate) const PHARMACODB_CELL_LINE_EMPTY_MESSAGE: &str =
    "no PharmacoDB cell line for this accession";
/// What the cell line section says when a candidate claims another accession.
pub(crate) const PHARMACODB_MISMATCH_MESSAGE: &str = "PharmacoDB accession does not match";
/// What the drug section says when PharmacoDB holds no such compound.
pub(crate) const PHARMACODB_COMPOUND_EMPTY_MESSAGE: &str = "no PharmacoDB compound with this name";
/// What either section says when the lookup failed.
const PHARMACODB_UNAVAILABLE_MESSAGE: &str = "PharmacoDB did not answer the drug response lookup";

/// What BioMCP can say about PharmacoDB's terms, verified on 2026-09-18.
///
/// PharmacoDB serves the same 2,258-byte single-page shell on every path and its
/// application bundle carries no licence string, so the provider publishes no
/// licence or terms page. Its source code is GPL-3.0
/// (<https://github.com/bhklab/PharmacoDB/blob/master/LICENSE>) and the paper
/// that describes the database is CC BY-NC 4.0 (doi:10.1093/nar/gkab1084). The
/// provider states nothing about the data itself, so BioMCP says exactly that
/// and signals the non-commercial term a user cannot see in the numbers. The
/// same sentence is in `docs/reference/source-licensing.md` and
/// `docs/reference/sources.json`, and a test pins that the three agree.
pub(crate) const PHARMACODB_LICENSE_SUMMARY: &str = "PharmacoDB publishes no licence or terms page; its source code is GPL-3.0 and the describing paper is CC BY-NC 4.0, and the terms for the data itself are unstated by the provider";

/// The one attribution line every PharmacoDB output carries.
pub(crate) fn pharmacodb_attribution(data_as_of: &str) -> String {
    format!(
        "Values as published by PharmacoDB. {PHARMACODB_LICENSE_SUMMARY}; treat reuse as \
non-commercial. PharmacoDB publishes no version; retrieved {data_as_of}. PharmacoDB gives no \
units; BioMCP does not interpret sensitivity."
    )
}

/// The counts one side of PharmacoDB holds. Neither section lists rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PharmacoDbCounts {
    pub source: String,
    pub pharmacodb_id: i64,
    pub total: usize,
    #[serde(default)]
    pub datasets: Vec<PharmacoDatasetCount>,
    pub data_as_of: String,
    pub data_as_of_kind: String,
}

/// Which rows a helper asked for.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PharmacoDbRowFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<String>,
}

/// One page of rows for a pair or for one dataset of one side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PharmacoDbRows {
    pub source: String,
    pub subject: String,
    pub pharmacodb_id: i64,
    pub total: usize,
    #[serde(default)]
    pub datasets: Vec<PharmacoDatasetCount>,
    pub filter: PharmacoDbRowFilter,
    /// How many rows the filter selected, before this page.
    pub matched: usize,
    #[serde(default)]
    pub rows: Vec<PharmacoDbRow>,
    pub data_as_of: String,
    pub data_as_of_kind: String,
}

/// One published experiment, seen from the side that was asked for. The
/// counterpart is the compound on a cell line row and the cell line on a drug
/// row. A null metric stays null.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PharmacoDbRow {
    pub experiment_id: i64,
    pub dataset: String,
    pub name: String,
    pub uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tissue: Option<String>,
    pub aac: Option<f64>,
    pub ic50: Option<f64>,
    pub ec50: Option<f64>,
    pub einf: Option<f64>,
    pub hs: Option<f64>,
    pub dss1: Option<f64>,
}

/// Which side of an experiment a row reports on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PharmacoSide {
    /// A cell line was asked for, so each row names its compound.
    CellLine,
    /// A drug was asked for, so each row names its cell line and tissue.
    Drug,
}

/// What a cell line join settled on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PharmacoCellLineJoin {
    Found(PharmacoCellLine),
    Missing,
    Mismatch,
}

/// Accept a candidate only when it claims the accession that was asked for.
///
/// The accession check is what stops a name fallback from joining another line's
/// numbers onto this card.
pub(crate) fn accept_cell_line(
    accession: &str,
    candidate: Option<PharmacoCellLine>,
) -> PharmacoCellLineJoin {
    let Some(candidate) = candidate else {
        return PharmacoCellLineJoin::Missing;
    };
    match candidate.accession_id.as_deref() {
        Some(found) if found.trim().eq_ignore_ascii_case(accession.trim()) => {
            PharmacoCellLineJoin::Found(candidate)
        }
        _ => PharmacoCellLineJoin::Mismatch,
    }
}

/// Accept a compound only when PharmacoDB spells its name the way it was asked
/// for, ignoring ASCII case.
pub(crate) fn compound_name_matches(query: &str, name: &str) -> bool {
    query.trim().eq_ignore_ascii_case(name.trim())
}

/// Turn one dataset argument into the name PharmacoDB publishes.
pub(crate) fn normalize_dataset(dataset: &str) -> Result<String, BioMcpError> {
    let dataset = dataset.trim();
    PHARMACODB_DATASET_NAMES
        .iter()
        .find(|name| name.eq_ignore_ascii_case(dataset))
        .map(|name| (*name).to_string())
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(format!(
                "Unknown PharmacoDB dataset \"{dataset}\". Available: {}",
                PHARMACODB_DATASET_NAMES.join(", ")
            ))
        })
}

/// The counts payload for one side.
pub(crate) fn build_counts(
    pharmacodb_id: i64,
    datasets: Vec<PharmacoDatasetCount>,
    data_as_of: String,
) -> PharmacoDbCounts {
    PharmacoDbCounts {
        source: PHARMACODB_SOURCE.to_string(),
        pharmacodb_id,
        total: datasets.iter().map(|row| row.count).sum(),
        datasets,
        data_as_of,
        data_as_of_kind: retrieved_kind(),
    }
}

/// `retrieved`, always. PharmacoDB publishes no version, no release name, and no
/// release date, so a repeat query can return different numbers with no way to
/// tell.
pub(crate) fn retrieved_kind() -> String {
    "retrieved".to_string()
}

/// The retrieval time every output carries.
pub(crate) fn retrieved_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// The counts per dataset the same rows imply, in name order.
pub(crate) fn counts_from_rows(experiments: &[PharmacoExperiment]) -> Vec<PharmacoDatasetCount> {
    let mut counts: Vec<PharmacoDatasetCount> = Vec::new();
    for row in experiments {
        match counts
            .iter_mut()
            .find(|entry| entry.name.eq_ignore_ascii_case(&row.dataset))
        {
            Some(entry) => entry.count += 1,
            None => counts.push(PharmacoDatasetCount {
                name: row.dataset.clone(),
                count: 1,
            }),
        }
    }
    counts.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.name.cmp(&right.name))
    });
    counts
}

/// Turn experiments into rows for one side, sorted by counterpart name, then
/// dataset name, then experiment id. No sort is ever by a metric.
pub(crate) fn build_rows(
    experiments: &[PharmacoExperiment],
    side: PharmacoSide,
) -> Vec<PharmacoDbRow> {
    let mut rows: Vec<PharmacoDbRow> = experiments
        .iter()
        .map(|row| PharmacoDbRow {
            experiment_id: row.experiment_id,
            dataset: row.dataset.clone(),
            name: match side {
                PharmacoSide::CellLine => row.compound_name.clone(),
                PharmacoSide::Drug => row.cell_line_name.clone(),
            },
            uid: match side {
                PharmacoSide::CellLine => row.compound_uid.clone(),
                PharmacoSide::Drug => row.cell_line_uid.clone(),
            },
            tissue: match side {
                PharmacoSide::CellLine => None,
                PharmacoSide::Drug => row.tissue.clone(),
            },
            aac: row.aac,
            ic50: row.ic50,
            ec50: row.ec50,
            einf: row.einf,
            hs: row.hs,
            dss1: row.dss1,
        })
        .collect();
    rows.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.dataset.cmp(&right.dataset))
            .then_with(|| left.experiment_id.cmp(&right.experiment_id))
    });
    rows
}

/// Keep the rows of one dataset. PharmacoDB has no dataset argument, so the
/// filter is applied after the fetch.
pub(crate) fn rows_in_dataset(
    rows: Vec<PharmacoDbRow>,
    dataset: Option<&str>,
) -> Vec<PharmacoDbRow> {
    match dataset {
        None => rows,
        Some(dataset) => rows
            .into_iter()
            .filter(|row| row.dataset.eq_ignore_ascii_case(dataset))
            .collect(),
    }
}

/// What one helper page was asked for, apart from the experiments themselves.
pub(crate) struct PharmacoRowPageInput {
    pub(crate) subject: String,
    pub(crate) pharmacodb_id: i64,
    pub(crate) side: PharmacoSide,
    pub(crate) filter: PharmacoDbRowFilter,
    pub(crate) offset: usize,
    pub(crate) limit: usize,
    pub(crate) data_as_of: String,
}

/// One helper payload: the whole-side counts, the filter, and one page of rows.
pub(crate) fn build_row_page(
    experiments: &[PharmacoExperiment],
    input: PharmacoRowPageInput,
) -> PharmacoDbRows {
    let datasets = counts_from_rows(experiments);
    let total = experiments.len();
    let matched = rows_in_dataset(
        build_rows(experiments, input.side),
        input.filter.dataset.as_deref(),
    );
    let rows = matched
        .iter()
        .skip(input.offset)
        .take(input.limit)
        .cloned()
        .collect();
    PharmacoDbRows {
        source: PHARMACODB_SOURCE.to_string(),
        subject: input.subject,
        pharmacodb_id: input.pharmacodb_id,
        total,
        datasets,
        filter: input.filter,
        matched: matched.len(),
        rows,
        data_as_of: input.data_as_of,
        data_as_of_kind: retrieved_kind(),
    }
}

/// The counts line both cards print: `1117 experiments: CTRPv2 416, ...`.
pub(crate) fn counts_line(total: usize, datasets: &[PharmacoDatasetCount]) -> String {
    let datasets = datasets
        .iter()
        .map(|row| format!("{} {}", row.name, row.count))
        .collect::<Vec<_>>()
        .join(", ");
    if datasets.is_empty() {
        format!("{total} experiments")
    } else {
        format!("{total} experiments: {datasets}")
    }
}

/// What a failed lookup completes the section as. A body over the PharmacoDB cap
/// says so plainly rather than looking like a transport failure.
pub(crate) fn failure_outcome(error: &BioMcpError) -> SectionOutcome {
    if matches!(error, BioMcpError::BodyLimit { .. }) {
        return SectionOutcome::unavailable(PHARMACODB_BODY_LIMIT_MESSAGE);
    }
    SectionOutcome::unavailable(PHARMACODB_UNAVAILABLE_MESSAGE)
}

/// Print one published metric with up to four significant digits. JSON keeps the
/// full value; this is the Markdown column only.
pub(crate) fn format_metric(value: Option<f64>) -> String {
    match value {
        None => "-".to_string(),
        Some(0.0) => "0".to_string(),
        Some(value) => {
            let magnitude = value.abs().log10().floor();
            #[expect(
                clippy::cast_possible_truncation,
                reason = "the exponent of a printed metric is small"
            )]
            let decimals = (3 - magnitude as i32).clamp(0, 12) as usize;
            let printed = format!("{value:.decimals$}");
            let trimmed = printed.trim_end_matches('0').trim_end_matches('.');
            if trimmed.is_empty() {
                "0".to_string()
            } else {
                trimmed.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests;
