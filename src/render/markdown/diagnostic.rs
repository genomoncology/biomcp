//! Diagnostic markdown renderers.

use super::*;
use serde::Serialize;
use std::fmt::Write as _;

#[cfg(test)]
mod tests;

const DIAGNOSTIC_SEARCH_ROW_LIST_CAP: usize = 5;
const DIAGNOSTIC_ZERO_RESULT_FILTER_HINT: &str =
    "Try adjusting or removing diagnostic filters: --gene, --disease, --type, or --manufacturer.";

#[derive(Debug, Serialize)]
pub(crate) struct DiagnosticSearchRow<'a> {
    accession: &'a str,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    test_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manufacturer_or_lab: Option<&'a str>,
    source: &'a str,
    source_label: String,
    genes_cell: String,
    conditions_cell: String,
}

pub fn diagnostic_markdown(
    diagnostic: &Diagnostic,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("diagnostic.md.j2")?;
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let supported =
        crate::entities::diagnostic::supported_diagnostic_sections_for_source(&diagnostic.source);
    let has_requested = |name: &str| {
        requested
            .iter()
            .any(|section| section.eq_ignore_ascii_case(name))
    };
    let supports = |name: &str| {
        supported
            .iter()
            .any(|section| section.eq_ignore_ascii_case(name))
    };
    let show_genes_section = supports("genes") && (include_all || has_requested("genes"));
    let show_conditions_section =
        supports("conditions") && (include_all || has_requested("conditions"));
    let show_methods_section = supports("methods") && (include_all || has_requested("methods"));
    let show_regulatory_section = supports("regulatory") && has_requested("regulatory");
    let source_states = section_render_contexts("diagnostic", &diagnostic.section_outcomes);
    let regulatory_state = source_states
        .get("regulatory")
        .expect("registered diagnostic regulatory state");
    let regulatory_status = regulatory_state.status.as_deref();
    let regulatory_block = if show_regulatory_section {
        render_regulatory_block(
            regulatory_state
                .payload_allowed
                .then_some(diagnostic.regulatory.as_deref())
                .flatten(),
            regulatory_status,
        )
    } else {
        String::new()
    };

    let body = tmpl.render(context! {
        accession => &diagnostic.accession,
        source_label => crate::entities::diagnostic::diagnostic_source_label(&diagnostic.source),
        source_id => &diagnostic.source_id,
        name => &diagnostic.name,
        test_type_label => if diagnostic.source.eq_ignore_ascii_case(crate::entities::diagnostic::DIAGNOSTIC_SOURCE_WHO_IVD) { "Assay Format" } else { "Type" },
        test_type => &diagnostic.test_type,
        manufacturer => &diagnostic.manufacturer,
        target_marker => &diagnostic.target_marker,
        regulatory_version => &diagnostic.regulatory_version,
        prequalification_year => &diagnostic.prequalification_year,
        laboratory => &diagnostic.laboratory,
        institution => &diagnostic.institution,
        country => &diagnostic.country,
        clia_number => &diagnostic.clia_number,
        state_licenses => &diagnostic.state_licenses,
        current_status => &diagnostic.current_status,
        public_status => &diagnostic.public_status,
        method_categories => &diagnostic.method_categories,
        genes => &diagnostic.genes,
        conditions => &diagnostic.conditions,
        methods => &diagnostic.methods,
        show_genes_section => show_genes_section,
        show_conditions_section => show_conditions_section,
        show_methods_section => show_methods_section,
        regulatory_block => regulatory_block,
        sections_block => format_sections_block(
            "diagnostic",
            &diagnostic.accession,
            sections_diagnostic(diagnostic, requested_sections),
        ),
        related_block => format_related_block(related_diagnostic(diagnostic)),
        source_states => source_states,
    })?;
    Ok(body)
}

fn render_regulatory_block(
    rows: Option<&[DiagnosticRegulatoryRecord]>,
    status: Option<&str>,
) -> String {
    let Some(rows) = rows else {
        return status.map_or_else(String::new, |status| {
            format!("## Regulatory (FDA Device)\n\n{status}\n")
        });
    };

    let mut out = String::from("## Regulatory (FDA Device)\n\n");
    if let Some(status) = status {
        let _ = writeln!(out, "{status}\n");
    }
    if rows.is_empty() {
        out.push_str("No FDA device 510(k) or PMA records matched this diagnostic.\n\n");
        return out;
    }

    out.push_str(
        "| Type | Number | Name | Applicant | Decision Date | Decision | Product Code | Supplements |\n",
    );
    out.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in rows {
        let supplements = row
            .supplement_count
            .filter(|count| *count > 0)
            .map(|count| count.to_string())
            .unwrap_or_else(|| "-".to_string());
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            markdown_cell(&row.submission_type),
            markdown_cell(&row.number),
            markdown_cell(&row.display_name),
            markdown_cell(row.applicant.as_deref().unwrap_or_default()),
            markdown_cell(row.decision_date.as_deref().unwrap_or_default()),
            markdown_cell(row.decision_description.as_deref().unwrap_or_default()),
            markdown_cell(row.product_code.as_deref().unwrap_or_default()),
            markdown_cell(&supplements),
        );
    }
    out.push('\n');
    out
}

// dead-code reason: diagnostic::diagnostic_search_markdown is exercised by native renderer contracts
#[allow(dead_code)]
pub fn diagnostic_search_markdown(
    query: &str,
    results: &[DiagnosticSearchResult],
    total: Option<usize>,
) -> Result<String, BioMcpError> {
    diagnostic_search_markdown_with_footer(query, results, total, "")
}

pub fn diagnostic_search_markdown_with_footer(
    query: &str,
    results: &[DiagnosticSearchResult],
    total: Option<usize>,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("diagnostic_search.md.j2")?;
    let rendered_results = diagnostic_search_rows(results);
    let top_accession = results
        .first()
        .map(|result| result.accession.as_str())
        .unwrap_or("");
    let true_zero_result = results.is_empty() && total == Some(0);
    let recovery_block = if true_zero_result {
        let related_block = format_related_block(diagnostic_zero_result_recovery_commands());
        if related_block.is_empty() {
            DIAGNOSTIC_ZERO_RESULT_FILTER_HINT.to_string()
        } else {
            format!("{DIAGNOSTIC_ZERO_RESULT_FILTER_HINT}\n\n{related_block}")
        }
    } else {
        String::new()
    };
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        total => total,
        top_accession => crate::render::markdown::quote_arg(top_accession),
        results => &rendered_results,
        recovery_block => recovery_block,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub(crate) fn diagnostic_search_rows(
    results: &[DiagnosticSearchResult],
) -> Vec<DiagnosticSearchRow<'_>> {
    results
        .iter()
        .map(|result| DiagnosticSearchRow {
            accession: &result.accession,
            name: &result.name,
            test_type: result.test_type.as_deref(),
            manufacturer_or_lab: result.manufacturer_or_lab.as_deref(),
            source: &result.source,
            source_label: crate::entities::diagnostic::diagnostic_source_label(&result.source)
                .to_string(),
            genes_cell: capped_cell(&result.genes),
            conditions_cell: capped_cell(&result.conditions),
        })
        .collect()
}

fn capped_cell(items: &[String]) -> String {
    if items.is_empty() {
        return "-".to_string();
    }

    let shown = items
        .iter()
        .take(DIAGNOSTIC_SEARCH_ROW_LIST_CAP)
        .map(|item| markdown_cell(item))
        .collect::<Vec<_>>()
        .join(", ");
    let overflow = items.len().saturating_sub(DIAGNOSTIC_SEARCH_ROW_LIST_CAP);

    if overflow == 0 {
        shown
    } else {
        format!("{shown}, +{overflow} more")
    }
}
