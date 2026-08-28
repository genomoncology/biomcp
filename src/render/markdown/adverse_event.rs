//! Adverse-event, device-event, and recall markdown renderers.

use super::*;

#[cfg(test)]
mod tests;

pub fn adverse_event_markdown(
    event: &AdverseEvent,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("adverse_event.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let parsed = crate::entities::adverse_event::parse_sections(requested_sections)?;
    let show_reactions_section = !section_only || parsed.include_reactions;
    let show_outcomes_section = !section_only || parsed.include_outcomes;
    let show_concomitant_section = !section_only || parsed.include_concomitant;
    let show_guidance_section = !section_only || parsed.include_guidance;
    let guidance_commands = adverse_event_guidance_commands(event);
    let guidance_drug = quote_arg(&event.drug);
    let guidance_indication = event
        .indication
        .as_deref()
        .map(quote_arg)
        .unwrap_or_default();
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header("Adverse Event", requested_sections),
        report_id => &event.report_id,
        drug => &event.drug,
        reactions => &event.reactions,
        outcomes => &event.outcomes,
        patient => &event.patient,
        concomitant_medications => &event.concomitant_medications,
        reporter_type => &event.reporter_type,
        reporter_country => &event.reporter_country,
        indication => &event.indication,
        guidance_commands => guidance_commands,
        guidance_drug => guidance_drug,
        guidance_indication => guidance_indication,
        show_reactions_section => show_reactions_section,
        show_outcomes_section => show_outcomes_section,
        show_concomitant_section => show_concomitant_section,
        show_guidance_section => show_guidance_section,
        serious => &event.serious,
        date => &event.date,
        sections_block => if section_only { String::new() } else { format_sections_block("adverse-event", &event.report_id, sections_adverse_event(event, requested_sections)) },
        related_block => if section_only { String::new() } else { format_related_block(related_adverse_event(event)) },
    })?;
    Ok(append_evidence_urls(
        body,
        adverse_event_evidence_urls(event),
    ))
}

pub fn adverse_event_guidance_commands(event: &AdverseEvent) -> Vec<String> {
    let drug = quote_arg(&event.drug);
    if drug.is_empty() {
        return Vec::new();
    }
    let mut commands = vec![
        format!("biomcp drug adverse-events {drug}"),
        format!("biomcp search drug -q {drug}"),
        format!("biomcp drug trials {drug}"),
    ];
    if let Some(indication) = event
        .indication
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        commands.push(format!(
            "biomcp search disease --query {}",
            quote_arg(indication)
        ));
    }
    commands
}

// dead-code reason: adverse_event::adverse_event_search_markdown is exercised by native renderer contracts
#[allow(dead_code)]
pub fn adverse_event_search_markdown(
    query: &str,
    results: &[AdverseEventSearchResult],
    summary: &AdverseEventSearchSummary,
) -> Result<String, BioMcpError> {
    adverse_event_search_markdown_with_footer(query, results, summary, "")
}

// dead-code reason: adverse_event::adverse_event_search_markdown_with_footer is exercised by native renderer contracts
#[allow(dead_code)]
pub fn adverse_event_search_markdown_with_footer(
    query: &str,
    results: &[AdverseEventSearchResult],
    summary: &AdverseEventSearchSummary,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    adverse_event_search_markdown_with_context(
        query,
        results,
        summary,
        pagination_footer,
        None,
        &[],
        None,
    )
}

pub fn adverse_event_search_markdown_with_context(
    query: &str,
    results: &[AdverseEventSearchResult],
    summary: &AdverseEventSearchSummary,
    pagination_footer: &str,
    empty_state_message: Option<&str>,
    trial_adverse_events: &[crate::entities::adverse_event::TrialAdverseEventTerm],
    trial_adverse_event_drug: Option<&str>,
) -> Result<String, BioMcpError> {
    adverse_event_search_markdown_with_source_label(
        query,
        results,
        summary,
        pagination_footer,
        empty_state_message,
        trial_adverse_events,
        trial_adverse_event_drug,
        "OpenFDA FAERS sample",
    )
}

#[allow(clippy::too_many_arguments)]
pub fn adverse_event_search_markdown_with_source_label(
    query: &str,
    results: &[AdverseEventSearchResult],
    summary: &AdverseEventSearchSummary,
    pagination_footer: &str,
    empty_state_message: Option<&str>,
    trial_adverse_events: &[crate::entities::adverse_event::TrialAdverseEventTerm],
    trial_adverse_event_drug: Option<&str>,
    summary_source_label: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("adverse_event_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        summary => summary,
        summary_source_label => summary_source_label,
        results => results,
        empty_state_message => empty_state_message,
        trial_adverse_events => trial_adverse_events,
        trial_adverse_event_drug => trial_adverse_event_drug,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn combined_adverse_event_search_markdown(
    query: &str,
    results: &[AdverseEventSearchResult],
    summary: &AdverseEventSearchSummary,
    pagination_footer: &str,
    empty_state_message: Option<&str>,
    vaers: Option<&crate::entities::adverse_event::VaersSearchPayload>,
) -> Result<String, BioMcpError> {
    let mut body = adverse_event_search_markdown_with_source_label(
        query,
        results,
        summary,
        pagination_footer,
        empty_state_message,
        &[],
        None,
        "OpenFDA FAERS sample",
    )?;

    if let Some(vaers) = vaers.filter(|payload| should_append_vaers_section(payload)) {
        if !body.ends_with('\n') {
            body.push('\n');
        }
        body.push('\n');
        body.push_str(&render_vaers_summary_section(vaers));
    }

    Ok(body)
}

pub fn vaers_only_markdown(
    query: &str,
    vaers: &crate::entities::adverse_event::VaersSearchPayload,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Adverse Events: {query}\n\n"));
    out.push_str(&render_vaers_summary_section(vaers));
    out
}

fn should_append_vaers_section(vaers: &crate::entities::adverse_event::VaersSearchPayload) -> bool {
    matches!(
        vaers.status,
        crate::entities::adverse_event::VaersSearchStatus::Ok
            | crate::entities::adverse_event::VaersSearchStatus::Empty
            | crate::entities::adverse_event::VaersSearchStatus::Unavailable
    )
}

fn vaers_status_label(status: crate::entities::adverse_event::VaersSearchStatus) -> &'static str {
    match status {
        crate::entities::adverse_event::VaersSearchStatus::Ok => "ok",
        crate::entities::adverse_event::VaersSearchStatus::Empty => "empty",
        crate::entities::adverse_event::VaersSearchStatus::QueryNotVaccine => "query_not_vaccine",
        crate::entities::adverse_event::VaersSearchStatus::UnsupportedFilters => {
            "unsupported_filters"
        }
        crate::entities::adverse_event::VaersSearchStatus::UnmappedVaccine => "unmapped_vaccine",
        crate::entities::adverse_event::VaersSearchStatus::Unavailable => "unavailable",
    }
}

fn render_vaers_summary_section(
    vaers: &crate::entities::adverse_event::VaersSearchPayload,
) -> String {
    let mut out = String::new();
    out.push_str("## CDC VAERS Summary\n\n");

    if let Some(matched) = &vaers.matched_vaccine {
        out.push_str(&format!("Matched vaccine: {}\n", matched.display_name));
        out.push_str(&format!("CDC WONDER code: {}\n", matched.wonder_code));
        if !matched.cvx_codes.is_empty() {
            out.push_str(&format!("CVX codes: {}\n", matched.cvx_codes.join(", ")));
        }
        out.push('\n');
    }

    if vaers.status != crate::entities::adverse_event::VaersSearchStatus::Ok {
        out.push_str(&format!("Status: {}\n", vaers_status_label(vaers.status)));
        if let Some(message) = &vaers.message {
            out.push_str(message);
            out.push('\n');
        }
        out.push('\n');
    }

    if let Some(summary) = &vaers.summary {
        out.push_str(&format!("Total reports: {}\n", summary.total_reports));
        out.push_str(&format!("Serious reports: {}\n", summary.serious_reports));
        out.push_str(&format!(
            "Non-serious reports: {}\n\n",
            summary.non_serious_reports
        ));

        out.push_str("### Age distribution\n\n");
        out.push_str("| Age bucket | Reports | Percent |\n");
        out.push_str("|---|---|---|\n");
        for row in &summary.age_distribution {
            out.push_str(&format!(
                "| {} | {} | {:.2}% |\n",
                row.age_bucket, row.reports, row.percentage
            ));
        }
        if summary.age_distribution.is_empty() {
            out.push_str("| - | 0 | 0.00% |\n");
        }
        out.push('\n');

        out.push_str("### Top reactions\n\n");
        out.push_str("| Reaction | Reports | Percent |\n");
        out.push_str("|---|---|---|\n");
        for row in &summary.top_reactions {
            out.push_str(&format!(
                "| {} | {} | {:.2}% |\n",
                row.reaction, row.count, row.percentage
            ));
        }
        if summary.top_reactions.is_empty() {
            out.push_str("| - | 0 | 0.00% |\n");
        }
        out.push('\n');
    }

    if vaers.summary.is_none() && vaers.message.is_none() {
        out.push_str("No CDC VAERS aggregate summary available.\n\n");
    }

    out.push_str("Source: CDC VAERS\n");
    out
}

pub fn adverse_event_count_markdown(
    query: &str,
    count_field: &str,
    buckets: &[AdverseEventCountBucket],
) -> Result<String, BioMcpError> {
    let mut out = String::new();
    out.push_str("# Adverse Event Counts\n");
    out.push_str(&format!("\nQuery: {query}\n"));
    out.push_str(&format!("Count field: {count_field}\n"));
    let counted_total = buckets.iter().map(|bucket| bucket.count).sum::<usize>();
    out.push_str(&format!("Counted rows shown: {counted_total}\n\n"));
    out.push_str("| Value | Count | Percent of Shown Count |\n");
    out.push_str("|---|---|---|\n");
    if buckets.is_empty() {
        out.push_str("| - | 0 | 0.0% |\n");
    } else {
        let denom = counted_total.max(1) as f64;
        for bucket in buckets {
            let percentage = ((bucket.count as f64 * 1000.0) / denom).round() / 10.0;
            out.push_str(&format!(
                "| {} | {} | {:.1}% |\n",
                bucket.value, bucket.count, percentage
            ));
        }
    }
    Ok(out)
}

pub fn device_event_markdown(event: &DeviceEvent) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("device_event.md.j2")?;
    let body = tmpl.render(context! {
        report_id => &event.report_id,
        report_number => &event.report_number,
        device => &event.device,
        manufacturer => &event.manufacturer,
        event_type => &event.event_type,
        date => &event.date,
        description => &event.description,
        related_block => format_related_block(related_device_event(event)),
    })?;
    Ok(append_evidence_urls(
        body,
        device_event_evidence_urls(event),
    ))
}

// dead-code reason: adverse_event::device_event_search_markdown is exercised by native renderer contracts
#[allow(dead_code)]
pub fn device_event_search_markdown(
    query: &str,
    results: &[DeviceEventSearchResult],
) -> Result<String, BioMcpError> {
    device_event_search_markdown_with_footer(query, results, "")
}

pub fn device_event_search_markdown_with_footer(
    query: &str,
    results: &[DeviceEventSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("device_event_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

// dead-code reason: adverse_event::recall_search_markdown is exercised by native renderer contracts
#[allow(dead_code)]
pub fn recall_search_markdown(
    query: &str,
    results: &[RecallSearchResult],
) -> Result<String, BioMcpError> {
    recall_search_markdown_with_footer(query, results, "")
}

pub fn recall_search_markdown_with_footer(
    query: &str,
    results: &[RecallSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("recall_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}
