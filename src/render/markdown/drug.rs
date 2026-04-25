//! Drug markdown renderers and drug-specific empty-state helpers.

use std::fmt::Write as _;

use super::drug_regulatory::{
    render_regulatory_block, render_safety_block, render_shortage_block, render_us_approvals_block,
};
use super::*;

#[cfg(test)]
mod tests;

pub fn drug_markdown_with_region(
    drug: &Drug,
    requested_sections: &[String],
    region: DrugRegion,
    raw_label: bool,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("drug.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_label_section = !section_only || include_all || has_requested("label");
    let show_targets_section = !section_only || include_all || has_requested("targets");
    let show_indications_section = !section_only || include_all || has_requested("indications");
    let show_interactions_section = include_all || has_requested("interactions");
    let show_civic_section = include_all || has_requested("civic");
    let show_regulatory_section = include_all || has_requested("regulatory");
    let show_safety_section =
        !matches!(region, DrugRegion::Who) && (include_all || has_requested("safety"));
    let show_shortage_section = !matches!(region, DrugRegion::Who)
        && (!section_only || include_all || has_requested("shortage"));
    let show_approvals_section = has_requested("approvals");
    // Suppress US-only header facts when rendering a full card (not section_only) for EU region.
    let show_us_header = section_only || region.includes_us();
    let approval_date_display: Option<&str> = if show_us_header {
        drug.approval_date_display.as_deref()
    } else {
        None
    };
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(&drug.name, requested_sections),
        drug_interactions_heading => crate::render::provenance::drug_interaction_heading_label(drug),
        drug_interactions_note => crate::render::provenance::drug_interaction_note(drug),
        name => &drug.name,
        drugbank_id => &drug.drugbank_id,
        chembl_id => &drug.chembl_id,
        unii => &drug.unii,
        drug_type => &drug.drug_type,
        mechanism => &drug.mechanism,
        mechanisms => &drug.mechanisms,
        approval_date => &drug.approval_date,
        approval_date_display => approval_date_display,
        brand_names => &drug.brand_names,
        route => &drug.route,
        show_us_header => show_us_header,
        top_adverse_events => &drug.top_adverse_events,
        targets => &drug.targets,
        variant_targets => &drug.variant_targets,
        target_family => &drug.target_family,
        target_family_name => &drug.target_family_name,
        indications => &drug.indications,
        interactions => &drug.interactions,
        interaction_text => &drug.interaction_text,
        interaction_class_summaries => crate::entities::drug::interaction_class_summaries(&drug.interactions),
        label => &drug.label,
        raw_label => raw_label,
        civic => &drug.civic,
        show_label_section => show_label_section,
        show_targets_section => show_targets_section,
        show_indications_section => show_indications_section,
        show_interactions_section => show_interactions_section,
        show_civic_section => show_civic_section,
        regulatory_block => if show_regulatory_section { render_regulatory_block(drug, region) } else { String::new() },
        safety_block => if show_safety_section { render_safety_block(drug, region) } else { String::new() },
        shortage_block => if show_shortage_section { render_shortage_block(drug, region) } else { String::new() },
        approvals_block => if show_approvals_section {
            render_us_approvals_block("## Drugs@FDA Approvals", drug.approvals.as_deref())
        } else {
            String::new()
        },
        sections_block => format_sections_block("drug", &drug.name, sections_drug(drug, requested_sections)),
        related_block => format_related_block(related_drug(drug)),
    })?;
    Ok(append_evidence_urls(body, drug_evidence_urls(drug)))
}

pub fn drug_markdown(drug: &Drug, requested_sections: &[String]) -> Result<String, BioMcpError> {
    drug_markdown_with_region(drug, requested_sections, DrugRegion::Us, false)
}

pub fn drug_interaction_report_markdown(
    report: &crate::entities::drug::DrugInteractionReport,
) -> Result<String, BioMcpError> {
    let mut out = String::new();
    let _ = writeln!(out, "# {}\n", report.name);
    if let Some(note) = report.source_note.as_deref() {
        let _ = writeln!(out, "{note}\n");
    }
    if !report.class_summaries.is_empty() {
        out.push_str("## Interacting Drug Classes\n\n");
        out.push_str("| Class | Interactions | Highest Level |\n");
        out.push_str("|---|---|---|\n");
        for row in &report.class_summaries {
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                markdown_cell(&row.class_name),
                row.interaction_count,
                row.highest_level
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
            );
        }
        out.push('\n');
    }
    if !report.interactions.is_empty() {
        out.push_str("## Interacting Drugs\n\n");
        out.push_str("| Interacting Drug | Level | Classes | Description |\n");
        out.push_str("|---|---|---|---|\n");
        for row in &report.interactions {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} |",
                markdown_cell(&row.drug),
                row.level
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                if row.partner_classes.is_empty() {
                    "-".to_string()
                } else {
                    markdown_cell(&row.partner_classes.join(", "))
                },
                row.description
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
            );
        }
        out.push('\n');
    }
    if let Some(label_text) = report.label_interaction_text.as_deref() {
        out.push_str("## Additive Label Text (OpenFDA)\n\n");
        let _ = writeln!(out, "{label_text}\n");
    }
    out.push_str(&format_related_block(related_drug_interactions(
        &report.name,
    )));
    Ok(append_evidence_urls(
        out,
        crate::render::markdown::drug_interaction_report_evidence_urls(report),
    ))
}

pub fn drug_search_markdown(
    query: &str,
    results: &[DrugSearchResult],
) -> Result<String, BioMcpError> {
    drug_search_markdown_with_footer(query, results, None, "")
}

pub fn drug_search_markdown_with_footer(
    query: &str,
    results: &[DrugSearchResult],
    total_count: Option<usize>,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("drug_search.md.j2")?;
    let count = total_count.unwrap_or(results.len());
    let discover_hint = discover_try_line(query, "resolve drug trial codes and aliases");
    let body = tmpl.render(context! {
        query => query,
        count => count,
        results => results,
        discover_hint => discover_hint,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

fn who_results_are_vaccines(results: &[WhoPrequalificationSearchResult]) -> bool {
    !results.is_empty()
        && results
            .iter()
            .all(WhoPrequalificationSearchResult::is_vaccine)
}

fn render_who_vaccine_search_table(out: &mut String, rows: &[WhoPrequalificationSearchResult]) {
    out.push_str(
        "|Vaccine Type|Commercial Name|Presentation|Doses|Manufacturer|Responsible NRA|Date|\n",
    );
    out.push_str("|---|---|---|---|---|---|---|\n");
    for row in rows {
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|{}|{}|{}|",
            row.vaccine_type
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| markdown_cell(&row.inn)),
            row.commercial_name
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.presentation
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.dose_count
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.manufacturer
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| markdown_cell(&row.applicant)),
            row.responsible_nra
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.prequalification_date
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn drug_search_markdown_with_region(
    query: &str,
    region: DrugRegion,
    us_results: &[DrugSearchResult],
    us_total: Option<usize>,
    eu_results: &[EmaDrugSearchResult],
    eu_total: Option<usize>,
    who_results: &[WhoPrequalificationSearchResult],
    who_total: Option<usize>,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    match region {
        DrugRegion::Us => {
            let count = us_total.unwrap_or(us_results.len());
            if count == 0 && is_structured_indication_query(query) {
                return Ok(empty_drug_indication_search_message(query, region));
            }
            drug_search_markdown_with_footer(query, us_results, us_total, pagination_footer)
        }
        DrugRegion::Eu => {
            let count = eu_total.unwrap_or(eu_results.len());
            if count == 0 && is_structured_indication_query(query) {
                return Ok(empty_drug_indication_search_message(query, region));
            }
            let mut out = String::new();
            let _ = writeln!(out, "# Drugs: {query}\n");
            if count == 0 {
                out.push_str("No drugs found\n");
                let discover_hint =
                    discover_try_line(query, "resolve drug trial codes and aliases");
                if !discover_hint.is_empty() {
                    let _ = writeln!(out, "\n{discover_hint}");
                }
                return Ok(out);
            }

            let _ = writeln!(
                out,
                "Found {count} drug{}\n",
                if count == 1 { "" } else { "s" }
            );
            out.push_str("|Name|Active Substance|EMA Number|Status|\n");
            out.push_str("|---|---|---|---|\n");
            for row in eu_results {
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{}|",
                    markdown_cell(&row.name),
                    markdown_cell(&row.active_substance),
                    markdown_cell(&row.ema_product_number),
                    markdown_cell(&row.status),
                );
            }
            if !(us_results.is_empty()
                && eu_results.is_empty()
                && who_results_are_vaccines(who_results))
            {
                out.push_str("\nUse `get drug <name>` for full details.\n");
            }
            if !pagination_footer.trim().is_empty() {
                let _ = writeln!(out, "\n{pagination_footer}");
            }
            Ok(out)
        }
        DrugRegion::Who => {
            let count = who_total.unwrap_or(who_results.len());
            if count == 0 && is_structured_indication_query(query) {
                return Ok(empty_drug_indication_search_message(query, region));
            }

            let mut out = String::new();
            let _ = writeln!(out, "# Drugs: {query}\n");
            if count == 0 {
                out.push_str("No WHO-prequalified drugs found\n");
                let discover_hint =
                    discover_try_line(query, "resolve drug trial codes and aliases");
                if !discover_hint.is_empty() {
                    let _ = writeln!(out, "\n{discover_hint}");
                }
                return Ok(out);
            }

            let _ = writeln!(
                out,
                "Found {count} drug{}\n",
                if count == 1 { "" } else { "s" }
            );
            if who_results_are_vaccines(who_results) {
                render_who_vaccine_search_table(&mut out, who_results);
            } else {
                out.push_str(
                    "|INN|Type|Therapeutic Area|Dosage Form|Applicant|WHO ID|Listing Basis|Date|\n",
                );
                out.push_str("|---|---|---|---|---|---|---|---|\n");
                for row in who_results {
                    let _ = writeln!(
                        out,
                        "|{}|{}|{}|{}|{}|{}|{}|{}|",
                        markdown_cell(&row.inn),
                        markdown_cell(&row.product_type),
                        markdown_cell(&row.therapeutic_area),
                        row.dosage_form
                            .as_deref()
                            .map(markdown_cell)
                            .unwrap_or_else(|| "-".to_string()),
                        markdown_cell(&row.applicant),
                        markdown_cell(row.display_identifier()),
                        row.listing_basis
                            .as_deref()
                            .map(markdown_cell)
                            .unwrap_or_else(|| "-".to_string()),
                        row.prequalification_date
                            .as_deref()
                            .map(markdown_cell)
                            .unwrap_or_else(|| "-".to_string()),
                    );
                }
            }
            if !who_results_are_vaccines(who_results) {
                out.push_str("\nUse `get drug <name>` for full details.\n");
            }
            if !pagination_footer.trim().is_empty() {
                let _ = writeln!(out, "\n{pagination_footer}");
            }
            Ok(out)
        }
        DrugRegion::All => {
            let mut out = String::new();
            let _ = writeln!(out, "# Drugs: {query}\n");

            out.push_str("## US (MyChem.info / OpenFDA)\n\n");
            let us_count = us_total.unwrap_or(us_results.len());
            let eu_count = eu_total.unwrap_or(eu_results.len());
            let who_count = who_total.unwrap_or(who_results.len());
            if us_results.is_empty() {
                if us_count == 0 && is_structured_indication_query(query) {
                    out.push_str(&empty_drug_indication_search_body(query, DrugRegion::All));
                    out.push('\n');
                } else {
                    out.push_str("No drugs found\n");
                }
            } else {
                let _ = writeln!(
                    out,
                    "Found {us_count} drug{}\n",
                    if us_count == 1 { "" } else { "s" }
                );
                out.push_str("|Name|Mechanism|Target|\n");
                out.push_str("|---|---|---|\n");
                for row in us_results {
                    let mechanism = row
                        .mechanism
                        .as_deref()
                        .or(row.drug_type.as_deref())
                        .unwrap_or("-");
                    let _ = writeln!(
                        out,
                        "|{}|{}|{}|",
                        markdown_cell(&row.name),
                        markdown_cell(mechanism),
                        row.target
                            .as_deref()
                            .map(markdown_cell)
                            .unwrap_or_else(|| "-".to_string()),
                    );
                }
            }

            out.push_str("\n## EU (EMA)\n\n");
            if eu_results.is_empty() {
                out.push_str("No drugs found\n");
            } else {
                let count = eu_total.unwrap_or(eu_results.len());
                let _ = writeln!(
                    out,
                    "Found {count} drug{}\n",
                    if count == 1 { "" } else { "s" }
                );
                out.push_str("|Name|Active Substance|EMA Number|Status|\n");
                out.push_str("|---|---|---|---|\n");
                for row in eu_results {
                    let _ = writeln!(
                        out,
                        "|{}|{}|{}|{}|",
                        markdown_cell(&row.name),
                        markdown_cell(&row.active_substance),
                        markdown_cell(&row.ema_product_number),
                        markdown_cell(&row.status),
                    );
                }
            }

            out.push_str("\n## WHO (WHO Prequalification)\n\n");
            if who_results.is_empty() {
                out.push_str("No WHO-prequalified drugs found\n");
            } else {
                let _ = writeln!(
                    out,
                    "Found {who_count} drug{}\n",
                    if who_count == 1 { "" } else { "s" }
                );
                if who_results_are_vaccines(who_results) {
                    render_who_vaccine_search_table(&mut out, who_results);
                } else {
                    out.push_str(
                        "|INN|Type|Therapeutic Area|Dosage Form|Applicant|WHO ID|Listing Basis|Date|\n",
                    );
                    out.push_str("|---|---|---|---|---|---|---|---|\n");
                    for row in who_results {
                        let _ = writeln!(
                            out,
                            "|{}|{}|{}|{}|{}|{}|{}|{}|",
                            markdown_cell(&row.inn),
                            markdown_cell(&row.product_type),
                            markdown_cell(&row.therapeutic_area),
                            row.dosage_form
                                .as_deref()
                                .map(markdown_cell)
                                .unwrap_or_else(|| "-".to_string()),
                            markdown_cell(&row.applicant),
                            markdown_cell(row.display_identifier()),
                            row.listing_basis
                                .as_deref()
                                .map(markdown_cell)
                                .unwrap_or_else(|| "-".to_string()),
                            row.prequalification_date
                                .as_deref()
                                .map(markdown_cell)
                                .unwrap_or_else(|| "-".to_string()),
                        );
                    }
                }
            }

            if us_count == 0
                && eu_count == 0
                && who_count == 0
                && !is_structured_indication_query(query)
            {
                let discover_hint =
                    discover_try_line(query, "resolve drug trial codes and aliases");
                if !discover_hint.is_empty() {
                    let _ = writeln!(out, "\n{discover_hint}");
                }
            }

            out.push_str("\nUse `get drug <name>` for full details.\n");
            if !pagination_footer.trim().is_empty() {
                let _ = writeln!(out, "\n{pagination_footer}");
            }
            Ok(out)
        }
    }
}

fn is_structured_indication_query(query: &str) -> bool {
    query
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("indication=")
}

fn indication_query_value(query: &str) -> &str {
    query
        .split_once('=')
        .map(|(_, value)| value.trim())
        .unwrap_or(query.trim())
}

fn empty_drug_indication_search_body(query: &str, region: DrugRegion) -> String {
    let disease = indication_query_value(query);
    let review_query = quote_arg(&format!("{disease} treatment"));
    let discover_hint = discover_try_line(disease, "resolve drug trial codes and aliases");
    match region {
        DrugRegion::Us => format!(
            "No drugs found in U.S. regulatory data for this indication.\nThis absence is informative for approved-drug questions, but it does not rule out investigational or off-label evidence.\nTry `biomcp search article -k {review_query} --type review --limit 5` for broader treatment literature.\n{discover_hint}"
        ),
        DrugRegion::All => format!(
            "No drugs found in U.S. regulatory data for this indication.\nThis absence is informative for approved-drug questions and is specific to the structured regulatory portion of the combined search.\nTry `biomcp search article -k {review_query} --type review --limit 5` for broader treatment literature.\n{discover_hint}"
        ),
        DrugRegion::Eu => format!("No drugs found\n{discover_hint}"),
        DrugRegion::Who => format!(
            "No WHO-prequalified drugs found for this structured search.\nThis absence is informative for WHO-prequalified regulatory coverage, but it does not rule out U.S. approvals or broader investigational evidence.\nTry `biomcp search article -k {review_query} --type review --limit 5` for broader treatment literature.\n{discover_hint}"
        ),
    }
}

fn empty_drug_indication_search_message(query: &str, region: DrugRegion) -> String {
    format!(
        "# Drugs: {query}\n\n{}\n",
        empty_drug_indication_search_body(query, region)
    )
}
