//! Trial markdown renderers.

use super::*;

#[cfg(test)]
mod tests;

pub fn trial_markdown(trial: &Trial, requested_sections: &[String]) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("trial.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let show_eligibility_section = include_all
        || requested
            .iter()
            .any(|s| s.eq_ignore_ascii_case("eligibility"));
    let show_contacts_section =
        include_all || requested.iter().any(|s| s.eq_ignore_ascii_case("contacts"));
    let show_locations_section = include_all
        || requested
            .iter()
            .any(|s| s.eq_ignore_ascii_case("locations"));
    let show_outcomes_section =
        include_all || requested.iter().any(|s| s.eq_ignore_ascii_case("outcomes"));
    let show_arms_section = include_all || requested.iter().any(|s| s.eq_ignore_ascii_case("arms"));
    let show_references_section = include_all
        || requested
            .iter()
            .any(|s| s.eq_ignore_ascii_case("references"));
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(&trial.nct_id, requested_sections),
        trial_source_label => crate::render::provenance::trial_source_label(trial.source.as_deref()),
        nct_id => &trial.nct_id,
        title => &trial.title,
        status => &trial.status,
        phase => &trial.phase,
        study_type => &trial.study_type,
        age_range => &trial.age_range,
        conditions => &trial.conditions,
        interventions => &trial.interventions,
        intervention_details => &trial.intervention_details,
        sponsor => &trial.sponsor,
        enrollment => &trial.enrollment,
        summary => &trial.summary,
        start_date => &trial.start_date,
        completion_date => &trial.completion_date,
        eligibility_text => &trial.eligibility_text,
        eligibility => &trial.eligibility,
        contacts => &trial.contacts,
        locations => &trial.locations,
        outcomes => &trial.outcomes,
        arms => &trial.arms,
        references => &trial.references,
        show_eligibility_section => show_eligibility_section,
        show_contacts_section => show_contacts_section,
        show_locations_section => show_locations_section,
        show_outcomes_section => show_outcomes_section,
        show_arms_section => show_arms_section,
        show_references_section => show_references_section,
        sections_block => format_sections_block("trial", &trial.nct_id, sections_trial(trial, requested_sections)),
        related_block => format_related_block(related_trial(trial)),
    })?;
    Ok(append_evidence_urls(body, trial_evidence_urls(trial)))
}

pub fn trial_search_markdown(
    query: &str,
    results: &[TrialSearchResult],
    total: Option<u32>,
) -> Result<String, BioMcpError> {
    trial_search_markdown_with_footer(query, results, total, "", false, None)
}

pub fn trial_action_summary_markdown(
    summary: &crate::entities::trial::TrialActionSummary,
) -> Result<String, BioMcpError> {
    let mut out = String::from("# Trial Action Summaries\n\n");
    out.push_str(
        "Uses listed CTGov sites only; BioMCP does not infer pending or unlisted sites.\n",
    );

    for item in &summary.results {
        out.push_str(&format!("\n## {} — {}\n\n", item.nct_id, item.title));
        out.push_str(&format!("- Status: {}\n", item.status));
        if let Some(trial_type) = item.trial_type.as_deref() {
            out.push_str(&format!(
                "- Trial type: {}\n",
                crate::entities::trial::trial_type_label(trial_type)
            ));
        }
        if !item.access_caveats.is_empty() {
            out.push_str("- Access caveats:\n");
            for caveat in &item.access_caveats {
                out.push_str(&format!("  - {}\n", caveat.label));
            }
        }
        if let Some(eligibility) = &item.eligibility {
            if let Some(sex) = eligibility.sex.as_deref() {
                out.push_str(&format!("- Sex: {sex}\n"));
            }
            match (
                eligibility.minimum_age.as_deref(),
                eligibility.maximum_age.as_deref(),
            ) {
                (Some(min), Some(max)) => {
                    out.push_str(&format!("- Eligible Ages: {min} to {max}\n"))
                }
                (Some(min), None) => out.push_str(&format!("- Eligible Ages: {min} to Any age\n")),
                (None, Some(max)) => out.push_str(&format!("- Eligible Ages: Any age to {max}\n")),
                (None, None) => {}
            }
        }
        if !item.contacts.is_empty() {
            out.push_str("- Contacts:\n");
            for contact in &item.contacts {
                out.push_str(&format!("  - {}", contact.name));
                if let Some(email) = contact.email.as_deref() {
                    out.push_str(&format!(" <{email}>"));
                }
                out.push('\n');
            }
        }
        if !item.ranked_sites.is_empty() {
            out.push_str("- Ranked listed sites:\n");
            for site in &item.ranked_sites {
                out.push_str(&format!("  - {}, {}", site.facility, site.city));
                if let Some(state) = site.state.as_deref() {
                    out.push_str(&format!(", {state}"));
                }
                out.push_str(&format!(" ({})", site.match_status));
                if let Some(distance) = site.distance_miles {
                    out.push_str(&format!(" — {:.1} miles", distance));
                }
                if site.match_status == "no_listed_facility_match"
                    && let Some(requested) = site.requested_facility.as_deref()
                {
                    out.push_str(&format!(" — No listed CTGov site matched: {requested}"));
                }
                out.push('\n');
            }
        }
    }

    Ok(out)
}

pub fn trial_search_markdown_with_footer(
    query: &str,
    results: &[TrialSearchResult],
    total: Option<u32>,
    pagination_footer: &str,
    show_zero_result_nickname_hint: bool,
    nickname_query: Option<&str>,
) -> Result<String, BioMcpError> {
    trial_search_markdown_with_footer_and_hints(
        query,
        results,
        total,
        pagination_footer,
        show_zero_result_nickname_hint,
        nickname_query,
        &[],
    )
}

pub fn trial_search_markdown_with_footer_and_hints(
    query: &str,
    results: &[TrialSearchResult],
    total: Option<u32>,
    pagination_footer: &str,
    show_zero_result_nickname_hint: bool,
    nickname_query: Option<&str>,
    zero_result_broadening_hints: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("trial_search.md.j2")?;
    let show_matched_condition_column = results
        .iter()
        .any(|result| result.matched_condition_label.is_some());
    let show_matched_intervention_column = results
        .iter()
        .any(|result| result.matched_intervention_label.is_some());
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        total => total,
        results => results,
        show_matched_condition_column => show_matched_condition_column,
        show_matched_intervention_column => show_matched_intervention_column,
        pagination_footer => pagination_footer,
        show_zero_result_nickname_hint => show_zero_result_nickname_hint,
        nickname_query => nickname_query,
        zero_result_broadening_hints => zero_result_broadening_hints,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}
