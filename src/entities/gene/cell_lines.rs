//! HPA RNA expression of one gene across the cell lines of one cancer group.
//!
//! HPA answers one gene at a time and names its cell lines, so the join to
//! Cellosaurus goes by name. Every name goes through the one cell line matcher
//! in [`crate::entities::cell_line`], which accepts an identifier match and
//! never a synonym. BioMCP reports the nTPM values as HPA published them and
//! adds no labels and no thresholds.

use serde::{Deserialize, Serialize};

use crate::entities::cell_line::unique_identifier_accession;
use crate::error::BioMcpError;
use crate::sources::cellosaurus::{CellosaurusClient, IDENTIFIER_BATCH_SIZE};
use crate::sources::hpa::{HPA_CELL_LINE_FILE_DATE, HpaCellLineExpression, HpaClient};
use crate::sources::mygene::MyGeneClient;

pub(crate) const GENE_CELL_LINES_SOURCE: &str = "Human Protein Atlas";
/// What a row says when no single human Cellosaurus record carries the name.
pub(crate) const GENE_CELL_LINES_NO_MATCH_NOTE: &str = "no unique Cellosaurus match";
/// What the payload says when Cellosaurus did not answer at all.
pub(crate) const GENE_CELL_LINES_UNAVAILABLE_NOTE: &str =
    "Cellosaurus did not answer the name lookup, so no row carries an accession";
/// The license the Human Protein Atlas publishes for the copyrightable parts of
/// its database, read from <https://www.proteinatlas.org/about/licence> on
/// 2026-09-18. The same value is in `docs/reference/source-licensing.md` and
/// `docs/reference/sources.json`, and a test pins that the three agree.
pub(crate) const GENE_CELL_LINES_LICENSE: &str = "CC BY 4.0";

/// One gene across the cell lines of one HPA cancer group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneCellLines {
    pub source: String,
    pub gene: String,
    pub ensembl_id: String,
    pub group: String,
    pub total: usize,
    pub data_as_of: String,
    pub data_as_of_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(default)]
    pub rows: Vec<GeneCellLineRow>,
}

/// One cell line row, in the order HPA published its columns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneCellLineRow {
    pub name: String,
    pub accession: Option<String>,
    pub ntpm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl GeneCellLines {
    /// Keep one window of the rows. `total` still counts the whole group.
    pub(crate) fn page(mut self, offset: usize, limit: usize) -> Self {
        self.rows = self.rows.into_iter().skip(offset).take(limit).collect();
        self
    }
}

/// The attribution line every output carries. The date is the recorded file
/// date, and the license is the one the Human Protein Atlas publishes.
pub(crate) fn gene_cell_lines_attribution(data_as_of: &str) -> String {
    format!(
        "Human Protein Atlas, files dated {data_as_of}, {GENE_CELL_LINES_LICENSE}. proteinatlas.org"
    )
}

/// Split the HPA names into the batches one Cellosaurus request each answers.
pub(crate) fn identifier_batches(names: &[String]) -> Vec<Vec<String>> {
    names
        .chunks(IDENTIFIER_BATCH_SIZE)
        .map(<[String]>::to_vec)
        .collect()
}

/// Pair each HPA row with the accession its name resolved to.
///
/// `accessions` holds one entry per name. A `None` entry is a name that no
/// single human record claimed, and the row says so.
pub(crate) fn build_rows(
    expressions: &[HpaCellLineExpression],
    accessions: &[Option<String>],
) -> Vec<GeneCellLineRow> {
    expressions
        .iter()
        .enumerate()
        .map(|(index, expression)| {
            let accession = accessions.get(index).cloned().flatten();
            GeneCellLineRow {
                name: expression.name.clone(),
                note: accession
                    .is_none()
                    .then(|| GENE_CELL_LINES_NO_MATCH_NOTE.to_string()),
                accession,
                ntpm: expression.ntpm,
            }
        })
        .collect()
}

/// What a Cellosaurus failure leaves behind: no accession anywhere and one
/// note. The expression rows still print.
pub(crate) fn unresolved_accessions(count: usize) -> (Vec<Option<String>>, Vec<String>) {
    (
        vec![None; count],
        vec![GENE_CELL_LINES_UNAVAILABLE_NOTE.to_string()],
    )
}

/// Resolve every HPA name to an accession, one request per batch.
///
/// A Cellosaurus failure leaves every accession `None` and adds one note. An
/// empty window is a normal answer and adds no note.
async fn resolve_accessions(names: &[String]) -> (Vec<Option<String>>, Vec<String>) {
    let client = match CellosaurusClient::new() {
        Ok(client) => client,
        Err(error) => {
            tracing::debug!("the Cellosaurus client is unavailable: {error}");
            return unresolved_accessions(names.len());
        }
    };

    let mut accessions = Vec::with_capacity(names.len());
    for batch in identifier_batches(names) {
        let page = match client.search_identifiers(&batch).await {
            Ok(page) => page,
            Err(error) => {
                tracing::debug!("the Cellosaurus identifier batch failed: {error}");
                return unresolved_accessions(names.len());
            }
        };
        for name in &batch {
            accessions.push(unique_identifier_accession(&page.records, name));
        }
    }
    (accessions, Vec::new())
}

/// The Ensembl gene ID behind one symbol, through the existing gene lookup.
async fn resolve_ensembl_id(symbol: &str) -> Result<(String, String), BioMcpError> {
    let hit = MyGeneClient::new()?.get(symbol, false).await?;
    let resolved = hit
        .symbol
        .clone()
        .unwrap_or_else(|| symbol.trim().to_string());
    let ensembl_id = hit
        .ensembl
        .as_ref()
        .and_then(|field| field.gene())
        .map(String::to_string)
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(format!(
                "`{resolved}` has no Ensembl gene ID, and HPA cell line expression is keyed by it"
            ))
        })?;
    Ok((resolved, ensembl_id))
}

/// Read one gene across one HPA cancer group.
pub(crate) async fn load(symbol: &str, group: &str) -> Result<GeneCellLines, BioMcpError> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "A gene symbol is required. Example: biomcp gene cell-lines FLT3 --group leukemia"
                .into(),
        ));
    }
    // The group is checked before the symbol lookup, so an unknown group makes
    // no request at all.
    let plan_group = HpaClient::cell_line_group(group)?;
    let (gene, ensembl_id) = resolve_ensembl_id(symbol).await?;

    let expressions = HpaClient::new()?
        .cell_line_rna(&ensembl_id, &plan_group)
        .await?;
    let names: Vec<String> = expressions
        .iter()
        .map(|expression| expression.name.clone())
        .collect();
    let (accessions, notes) = resolve_accessions(&names).await;

    Ok(GeneCellLines {
        source: GENE_CELL_LINES_SOURCE.to_string(),
        gene,
        ensembl_id,
        group: plan_group,
        total: expressions.len(),
        data_as_of: HPA_CELL_LINE_FILE_DATE.to_string(),
        data_as_of_kind: "release".to_string(),
        notes,
        rows: build_rows(&expressions, &accessions),
    })
}

#[cfg(test)]
mod tests;
