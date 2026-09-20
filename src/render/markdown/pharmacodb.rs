//! PharmacoDB row renderers for the two drug-response helpers.

use super::*;
use crate::entities::pharmacodb::{
    PharmacoDbRows, counts_line, format_metric, pharmacodb_attribution,
};

/// One page of published PharmacoDB rows, seen from the drug side or the cell
/// line side. The counterpart column is named by the caller, because the rows
/// name compounds on a cell line page and cell lines on a drug page.
pub fn pharmacodb_rows_markdown(
    page: &PharmacoDbRows,
    heading: &str,
    counterpart_column: &str,
    show_tissue: bool,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("pharmacodb_rows.md.j2")?;
    let filter_line = match (&page.filter.cell_line, &page.filter.dataset) {
        (Some(cell_line), _) => format!("Filter: cell line {cell_line}"),
        (None, Some(dataset)) => format!("Filter: dataset {dataset}"),
        (None, None) => String::new(),
    };
    let rows: Vec<serde_json::Value> = page
        .rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "experiment_id": row.experiment_id,
                "dataset": row.dataset,
                "name": row.name,
                "tissue": row.tissue.clone().unwrap_or_else(|| "-".to_string()),
                "aac": format_metric(row.aac),
                "ic50": format_metric(row.ic50),
                "ec50": format_metric(row.ec50),
                "einf": format_metric(row.einf),
                "hs": format_metric(row.hs),
                "dss1": format_metric(row.dss1),
            })
        })
        .collect();
    let body = tmpl.render(context! {
        heading => heading,
        subject => &page.subject,
        counts_line => counts_line(page.total, &page.datasets),
        filter_line => filter_line,
        counterpart_column => counterpart_column,
        show_tissue => show_tissue,
        rows => rows,
        truncated => page.rows.len() < page.matched,
        shown => page.rows.len(),
        matched => page.matched,
        attribution => pharmacodb_attribution(&page.data_as_of),
    })?;
    Ok(body)
}
