//! Drug regulatory, safety, and shortage block renderers.

use super::*;
use crate::sources::fda_orphan::{FdaOrphanDesignations, FdaOrphanOutcome};

fn orphan_text(value: &str) -> String {
    let value = crate::render::human::sanitize_inline(value);
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if value.is_empty() {
        return "-".into();
    }
    value.chars().fold(String::new(), |mut out, ch| {
        if matches!(
            ch,
            '\\' | '|' | '[' | ']' | '(' | ')' | '`' | '<' | '>' | '#'
        ) {
            out.push('\\');
        }
        out.push(ch);
        out
    })
}

fn orphan_value(value: Option<&str>) -> String {
    value.map(orphan_text).unwrap_or_else(|| "-".into())
}

fn render_fda_orphan_block(name: &str, value: Option<&FdaOrphanDesignations>) -> String {
    let Some(value) = value else {
        return String::new();
    };
    let mut out = String::from("### FDA orphan designations\n\n");
    if let Some(message) = value.message.as_deref() {
        let _ = writeln!(out, "{}\n", orphan_text(message));
    }
    match value.outcome {
        FdaOrphanOutcome::Empty => out.push_str("No matching FDA orphan designations found.\n"),
        FdaOrphanOutcome::Unavailable => {
            let command = format!(
                "biomcp get drug {} regulatory --region us",
                shell_quote_arg(name)
            );
            let _ = writeln!(out, "Retry: {}\n", markdown_code_span(&command));
        }
        FdaOrphanOutcome::Data | FdaOrphanOutcome::Degraded => {
            for row in &value.records {
                let _ = writeln!(out, "#### {}\n", orphan_text(&row.generic_name));
                let _ = writeln!(out, "- Designated: {}", orphan_text(&row.designation_date));
                let _ = writeln!(
                    out,
                    "- Designation status: {}",
                    orphan_value(row.designation_status.as_deref())
                );
                let _ = writeln!(
                    out,
                    "- Orphan approval: {}",
                    orphan_text(match row.orphan_approval {
                        crate::sources::fda_orphan::OrphanApproval::Approved => "approved",
                        crate::sources::fda_orphan::OrphanApproval::NotApproved => "not_approved",
                        crate::sources::fda_orphan::OrphanApproval::Unknown => "unknown",
                    })
                );
                let _ = writeln!(
                    out,
                    "- Marketing approval: {}",
                    orphan_value(row.marketing_approval_date.as_deref())
                );
                let _ = writeln!(
                    out,
                    "- Exclusivity end: {}",
                    orphan_value(row.exclusivity_end_date.as_deref())
                );
                let _ = writeln!(out, "- Sponsor: {}", orphan_value(row.sponsor.as_deref()));
                let _ = writeln!(out, "- Indication: {}", orphan_text(&row.designation));
                let _ = writeln!(out, "- FDA record: [FDA record]({})\n", row.source_url);
            }
        }
    }
    out
}

pub(super) fn render_us_approvals_block(
    heading: &str,
    approvals: Option<&[DrugApproval]>,
) -> String {
    let Some(approvals) = approvals else {
        return String::new();
    };

    let mut out = String::new();
    let _ = writeln!(out, "{heading}\n");
    if approvals.is_empty() {
        out.push_str("No approvals found in Drugs@FDA for this query.\n");
        return out;
    }

    for app in approvals {
        let _ = writeln!(out, "### {}\n", markdown_cell(&app.application_number));
        if let Some(sponsor_name) = app.sponsor_name.as_deref() {
            let _ = writeln!(out, "- Sponsor: {}", markdown_cell(sponsor_name));
        }
        if !app.openfda_brand_names.is_empty() {
            let brands = app
                .openfda_brand_names
                .iter()
                .map(|value| markdown_cell(value))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(out, "- Brands: {brands}");
        }
        if !app.openfda_generic_names.is_empty() {
            let generics = app
                .openfda_generic_names
                .iter()
                .map(|value| markdown_cell(value))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(out, "- Generic Names: {generics}");
        }
        if !app.products.is_empty() {
            out.push_str("| Product | Dosage Form | Route | Marketing Status |\n");
            out.push_str("|---|---|---|---|\n");
            for product in &app.products {
                let _ = writeln!(
                    out,
                    "| {} | {} | {} | {} |",
                    product
                        .brand_name
                        .as_deref()
                        .map(markdown_cell)
                        .unwrap_or_else(|| "-".to_string()),
                    product
                        .dosage_form
                        .as_deref()
                        .map(markdown_cell)
                        .unwrap_or_else(|| "-".to_string()),
                    product
                        .route
                        .as_deref()
                        .map(markdown_cell)
                        .unwrap_or_else(|| "-".to_string()),
                    product
                        .marketing_status
                        .as_deref()
                        .map(markdown_cell)
                        .unwrap_or_else(|| "-".to_string()),
                );
            }
        }
        if !app.submissions.is_empty() {
            out.push_str("| Submission Type | Number | Status | Date |\n");
            out.push_str("|---|---|---|---|\n");
            for submission in &app.submissions {
                let _ = writeln!(
                    out,
                    "| {} | {} | {} | {} |",
                    submission
                        .submission_type
                        .as_deref()
                        .map(markdown_cell)
                        .unwrap_or_else(|| "-".to_string()),
                    submission
                        .submission_number
                        .as_deref()
                        .map(markdown_cell)
                        .unwrap_or_else(|| "-".to_string()),
                    submission
                        .status
                        .as_deref()
                        .map(markdown_cell)
                        .unwrap_or_else(|| "-".to_string()),
                    submission
                        .status_date
                        .as_deref()
                        .map(markdown_cell)
                        .unwrap_or_else(|| "-".to_string()),
                );
            }
        }
        out.push('\n');
    }

    out
}

fn render_eu_regulatory_block(heading: &str, rows: Option<&[EmaRegulatoryRow]>) -> String {
    let Some(rows) = rows else {
        return String::new();
    };

    let mut out = String::new();
    let _ = writeln!(out, "{heading}\n");
    if rows.is_empty() {
        out.push_str("No data found (EMA)\n");
        return out;
    }

    out.push_str("| Medicine | Active Substance | EMA Number | Status | Auth Date | Holder |\n");
    out.push_str("|---|---|---|---|---|---|\n");
    for row in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            markdown_cell(&row.medicine_name),
            markdown_cell(&row.active_substance),
            markdown_cell(&row.ema_product_number),
            markdown_cell(&row.status),
            row.marketing_authorisation_date
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.holder
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
        );
    }

    out.push_str("\n### Authorized indications\n");
    let indication_rows = rows
        .iter()
        .filter_map(|row| {
            row.therapeutic_indication
                .as_deref()
                .map(|indication| (row.medicine_name.as_str(), indication))
        })
        .collect::<Vec<_>>();
    if indication_rows.is_empty() {
        out.push_str("No authorized indications found.\n");
    } else {
        for (medicine_name, indication) in indication_rows {
            let _ = writeln!(
                out,
                "- **{}:** {}",
                markdown_cell(medicine_name),
                markdown_cell(indication),
            );
        }
    }

    out.push_str("\n### Recent post-authorisation activity\n");
    let activity_rows = rows
        .iter()
        .flat_map(|row| {
            row.recent_activity.iter().map(move |activity| {
                (
                    row.medicine_name.as_str(),
                    activity.first_published_date.as_str(),
                    activity.last_updated_date.as_deref(),
                )
            })
        })
        .collect::<Vec<_>>();
    if activity_rows.is_empty() {
        out.push_str("No recent post-authorisation activity found.\n");
        return out;
    }

    out.push_str("| Medicine | First Published | Last Updated |\n");
    out.push_str("|---|---|---|\n");
    for (medicine_name, first_published_date, last_updated_date) in activity_rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            markdown_cell(medicine_name),
            markdown_cell(first_published_date),
            last_updated_date
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    out
}

fn render_who_regulatory_block(heading: &str, rows: Option<&[WhoPrequalificationEntry]>) -> String {
    let Some(rows) = rows else {
        return String::new();
    };

    let mut out = String::new();
    let _ = writeln!(out, "{heading}\n");
    if rows.is_empty() {
        out.push_str("Not WHO-prequalified\n");
        return out;
    }

    out.push_str("| WHO ID | Type | Presentation / INN | Dosage Form | Therapeutic Area | Applicant | Listing Basis | Grade | Alternative Basis | Prequalification Date | Confirmation Doc Date |\n");
    out.push_str("|---|---|---|---|---|---|---|---|---|---|---|\n");
    for row in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            markdown_cell(row.display_identifier()),
            markdown_cell(&row.product_type),
            row.presentation
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| markdown_cell(&row.inn)),
            row.dosage_form
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            markdown_cell(&row.therapeutic_area),
            markdown_cell(&row.applicant),
            row.listing_basis
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.grade
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.alternative_listing_basis
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.prequalification_date
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.confirmation_document_date
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
        );
    }

    out
}

fn render_us_safety_block(drug: &Drug, heading: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{heading}\n");

    out.push_str("### Top adverse events (FAERS)\n");
    if drug.top_adverse_events.is_empty() {
        out.push_str("No data found (OpenFDA FAERS)\n");
    } else {
        let _ = writeln!(out, "{}", drug.top_adverse_events.join(", "));
    }

    out.push_str("\n### FDA label warnings\n");
    if let Some(warnings) = drug.us_safety_warnings.as_deref() {
        out.push_str(warnings);
        out.push('\n');
    } else {
        out.push_str("No data found (OpenFDA label)\n");
    }

    out
}

fn render_eu_safety_block(heading: &str, safety: Option<&EmaSafetyInfo>) -> String {
    let Some(safety) = safety else {
        return String::new();
    };

    let mut out = String::new();
    let _ = writeln!(out, "{heading}\n");

    out.push_str("### DHPCs\n");
    if safety.dhpcs.is_empty() {
        out.push_str("No data found (EMA)\n");
    } else {
        out.push_str("| Medicine | Type | Outcome | First Published | Last Updated |\n");
        out.push_str("|---|---|---|---|---|\n");
        for row in &safety.dhpcs {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                markdown_cell(&row.medicine_name),
                row.dhpc_type
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.regulatory_outcome
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.first_published_date
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.last_updated_date
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
            );
        }
    }

    out.push_str("\n### Referrals\n");
    if safety.referrals.is_empty() {
        out.push_str("No data found (EMA)\n");
    } else {
        out.push_str("| Referral | Active Substance | Medicines | Status | Type | Start |\n");
        out.push_str("|---|---|---|---|---|---|\n");
        for row in &safety.referrals {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} |",
                markdown_cell(&row.referral_name),
                row.active_substance
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.associated_medicines
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.current_status
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.referral_type
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.procedure_start_date
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
            );
        }
    }

    out.push_str("\n### PSUSAs\n");
    if safety.psusas.is_empty() {
        out.push_str("No data found (EMA)\n");
    } else {
        out.push_str("| Related Medicines | Active Substance | Procedure | Outcome | First Published | Last Updated |\n");
        out.push_str("|---|---|---|---|---|---|\n");
        for row in &safety.psusas {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} |",
                row.related_medicines
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.active_substance
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.procedure_number
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.regulatory_outcome
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.first_published_date
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
                row.last_updated_date
                    .as_deref()
                    .map(markdown_cell)
                    .unwrap_or_else(|| "-".to_string()),
            );
        }
    }

    out
}

fn render_us_shortage_block(
    heading: &str,
    shortage: Option<&[crate::entities::drug::DrugShortageEntry]>,
) -> String {
    let Some(shortage) = shortage else {
        return String::new();
    };

    let mut out = String::new();
    let _ = writeln!(out, "{heading}\n");
    if shortage.is_empty() {
        out.push_str("No shortage entries found\n");
        return out;
    }

    out.push_str("| Status | Availability | Company | Updated | Info |\n");
    out.push_str("|---|---|---|---|---|\n");
    for row in shortage {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            row.status
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.availability
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.company_name
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.update_date
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.related_info
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    out
}

fn render_eu_shortage_block(heading: &str, shortage: Option<&[EmaShortageEntry]>) -> String {
    let Some(shortage) = shortage else {
        return String::new();
    };

    let mut out = String::new();
    let _ = writeln!(out, "{heading}\n");
    if shortage.is_empty() {
        out.push_str("No data found (EMA)\n");
        return out;
    }

    out.push_str("| Medicine | Status | Alternatives | First Published | Last Updated |\n");
    out.push_str("|---|---|---|---|---|\n");
    for row in shortage {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            markdown_cell(&row.medicine_affected),
            row.status
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.availability_of_alternatives
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.first_published_date
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            row.last_updated_date
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    out
}

pub(super) fn render_regulatory_block(drug: &Drug, region: DrugRegion) -> String {
    match region {
        DrugRegion::Us => {
            let approvals = render_us_approvals_block(
                "## Regulatory (US - Drugs@FDA)",
                drug.approvals.as_deref(),
            );
            format!(
                "{approvals}\n{}",
                render_fda_orphan_block(&drug.name, drug.fda_orphan_designations.as_ref())
            )
        }
        DrugRegion::Eu => {
            render_eu_regulatory_block("## Regulatory (EU - EMA)", drug.ema_regulatory.as_deref())
        }
        DrugRegion::Who => render_who_regulatory_block(
            "## Regulatory (WHO Prequalification)",
            drug.who_prequalification.as_deref(),
        ),
        DrugRegion::All => {
            let us = render_us_approvals_block(
                "## Regulatory (US - Drugs@FDA)",
                drug.approvals.as_deref(),
            );
            let orphan = render_fda_orphan_block(&drug.name, drug.fda_orphan_designations.as_ref());
            let eu = render_eu_regulatory_block(
                "## Regulatory (EU - EMA)",
                drug.ema_regulatory.as_deref(),
            );
            let who = render_who_regulatory_block(
                "## Regulatory (WHO Prequalification)",
                drug.who_prequalification.as_deref(),
            );
            [us, orphan, eu, who]
                .into_iter()
                .filter(|block| !block.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

pub(super) fn render_safety_block(
    drug: &Drug,
    region: DrugRegion,
    status: Option<&str>,
    payload_allowed: bool,
) -> String {
    if !payload_allowed {
        let heading = match region {
            DrugRegion::Us | DrugRegion::All => "## Safety (US - OpenFDA)",
            DrugRegion::Eu => "## Safety (EU - EMA)",
            DrugRegion::Who => return String::new(),
        };
        return status.map_or_else(String::new, |status| format!("{heading}\n\n{status}\n"));
    }
    let us_heading = status.map_or_else(
        || "## Safety (US - OpenFDA)".to_string(),
        |status| format!("## Safety (US - OpenFDA)\n\n{status}"),
    );
    let eu_heading = status.map_or_else(
        || "## Safety (EU - EMA)".to_string(),
        |status| format!("## Safety (EU - EMA)\n\n{status}"),
    );
    match region {
        DrugRegion::Us => render_us_safety_block(drug, &us_heading),
        DrugRegion::Eu => render_eu_safety_block(&eu_heading, drug.ema_safety.as_ref()),
        DrugRegion::Who => String::new(),
        DrugRegion::All => {
            let us = render_us_safety_block(drug, &us_heading);
            let eu = render_eu_safety_block("## Safety (EU - EMA)", drug.ema_safety.as_ref());
            [us, eu]
                .into_iter()
                .filter(|block| !block.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

pub(super) fn render_shortage_block(drug: &Drug, region: DrugRegion) -> String {
    match region {
        DrugRegion::Us => render_us_shortage_block(
            "## Shortage (US - OpenFDA Drug Shortages)",
            drug.shortage.as_deref(),
        ),
        DrugRegion::Eu => {
            render_eu_shortage_block("## Shortage (EU - EMA)", drug.ema_shortage.as_deref())
        }
        DrugRegion::Who => String::new(),
        DrugRegion::All => {
            let us = render_us_shortage_block(
                "## Shortage (US - OpenFDA Drug Shortages)",
                drug.shortage.as_deref(),
            );
            let eu =
                render_eu_shortage_block("## Shortage (EU - EMA)", drug.ema_shortage.as_deref());
            [us, eu]
                .into_iter()
                .filter(|block| !block.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

#[cfg(test)]
mod fda_orphan_tests {
    use super::*;
    use crate::sources::fda_orphan::{FdaOrphanRecord, OrphanApproval};

    fn designation(outcome: FdaOrphanOutcome) -> FdaOrphanDesignations {
        FdaOrphanDesignations {
            outcome,
            sources: vec![crate::sources::fda_orphan::FDA_ORPHAN_SOURCE.into()],
            records: vec![FdaOrphanRecord {
                record_id: "992323".into(),
                generic_name: "drug | [link](bad) `tick` <script>\nheading".into(),
                trade_name: None,
                designation_date: "2024-03-11".into(),
                designation: "use | [bad](x) `code` <b>".into(),
                designation_status: None,
                designation_withdrawn_or_revoked_date: None,
                orphan_approval: OrphanApproval::NotApproved,
                orphan_approval_status_text: Some("Not FDA Approved for Orphan Indication".into()),
                approved_labeled_indication: None,
                marketing_approval_date: None,
                exclusivity_end_date: None,
                exclusivity_protected_indication: None,
                sponsor: Some("sponsor\u{1b}[31m | # heading".into()),
                source_url: "https://www.accessdata.fda.gov/scripts/opdlisting/oopd/detailedIndex.cfm?cfgridkey=992323".into(),
            }],
            message: None,
            total_matching: Some(1),
            truncated: false,
        }
    }

    #[test]
    fn hostile_provider_text_cannot_create_markdown_structure() {
        let rendered =
            render_fda_orphan_block("odd `name`", Some(&designation(FdaOrphanOutcome::Data)));
        assert!(rendered.contains("drug \\| \\[link\\]\\(bad\\) \\`tick\\` \\<script\\> heading"));
        assert!(!rendered.contains('\u{1b}'));
        assert_eq!(rendered.matches("[FDA record](https://").count(), 1);
    }

    #[test]
    fn empty_and_unavailable_have_truthful_recovery() {
        let mut empty = designation(FdaOrphanOutcome::Empty);
        empty.records.clear();
        empty.total_matching = Some(0);
        assert!(
            render_fda_orphan_block("drug", Some(&empty))
                .contains("No matching FDA orphan designations found.")
        );
        let mut unavailable = designation(FdaOrphanOutcome::Unavailable);
        unavailable.sources.clear();
        unavailable.records.clear();
        unavailable.total_matching = None;
        unavailable.message =
            Some("FDA orphan-designation data is temporarily unavailable.".into());
        let rendered = render_fda_orphan_block("odd `name`", Some(&unavailable));
        assert!(rendered.contains("FDA orphan-designation data is temporarily unavailable."));
        assert!(rendered.contains("Retry:"));
    }
}
