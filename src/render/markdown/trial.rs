//! Trial markdown renderers.

use super::*;

#[cfg(test)]
mod tests;

fn abbreviation_at_period(summary: &str, period_index: usize) -> Option<bool> {
    const ABBREVIATIONS: [(&str, bool); 6] = [
        ("pts.", false),
        ("vs.", false),
        ("approx.", false),
        ("e.g.", false),
        ("i.v.", false),
        ("Dr.", true),
    ];

    let bytes = summary.as_bytes();
    let token_end = period_index + 1;
    ABBREVIATIONS
        .iter()
        .find_map(|(token, continues_before_uppercase)| {
            let token_start = token_end.checked_sub(token.len())?;
            if !bytes[token_start..token_end].eq_ignore_ascii_case(token.as_bytes()) {
                return None;
            }
            let is_complete_token = summary[..token_start]
                .chars()
                .next_back()
                .is_none_or(|character| !character.is_alphanumeric());
            is_complete_token.then_some(*continues_before_uppercase)
        })
}

fn abbreviation_continues_sentence(summary: &str, period_index: usize) -> bool {
    let Some(continues_before_uppercase) = abbreviation_at_period(summary, period_index) else {
        return false;
    };
    let next_lexical_character = summary[period_index + 1..]
        .chars()
        .find(|character| character.is_alphanumeric());

    next_lexical_character.is_some_and(|character| {
        character.is_lowercase()
            || character.is_numeric()
            || (continues_before_uppercase && character.is_uppercase())
    })
}

fn bounded_trial_summary(summary: &str) -> String {
    const MAX_SENTENCES: usize = 2;
    const MAX_BYTES: usize = 500;

    let trimmed = summary.trim();
    let mut sentence_end = None;
    let mut sentence_count = 0;
    let bytes = trimmed.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'.' {
            continue;
        }
        let next = index + 1;
        if next == bytes.len() || bytes.get(next).is_some_and(|b| b.is_ascii_whitespace()) {
            if abbreviation_continues_sentence(trimmed, index) {
                continue;
            }
            sentence_count += 1;
            if sentence_count == MAX_SENTENCES {
                sentence_end = Some(next);
                break;
            }
        }
    }

    let bounded_sentences = &trimmed[..sentence_end.unwrap_or(trimmed.len())];
    let sentence_truncated = bounded_sentences.len() < trimmed.len();
    let byte_truncated = bounded_sentences.len() > MAX_BYTES;
    let mut bounded = if byte_truncated {
        let mut boundary = MAX_BYTES;
        while boundary > 0 && !bounded_sentences.is_char_boundary(boundary) {
            boundary -= 1;
        }
        bounded_sentences[..boundary].trim_end().to_string()
    } else {
        bounded_sentences.to_string()
    };
    if sentence_truncated || byte_truncated {
        if bounded.ends_with('.') {
            bounded.pop();
        }
        bounded.push_str("...");
    }
    bounded
}

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
    let summary = trial
        .summary
        .as_deref()
        .map(bounded_trial_summary)
        .filter(|summary| !summary.is_empty());
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(&trial.nct_id, requested_sections),
        trial_source_label => crate::render::provenance::trial_source_label(trial.source.as_deref()),
        nct_id => &trial.nct_id,
        title => &trial.title,
        status => &trial.status,
        why_stopped_checked => trial.why_stopped.is_some(),
        why_stopped => trial.why_stopped.as_ref().and_then(|reason| reason.as_deref()),
        phase => &trial.phase,
        study_type => &trial.study_type,
        age_range => &trial.age_range,
        conditions => &trial.conditions,
        interventions => &trial.interventions,
        intervention_details => &trial.intervention_details,
        sponsor => &trial.sponsor,
        enrollment => &trial.enrollment,
        summary => &summary,
        start_date => &trial.start_date,
        completion_date => &trial.completion_date,
        eligibility_text => &trial.eligibility_text,
        eligibility => &trial.eligibility,
        eligibility_provenance => &trial.eligibility_provenance,
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
    let show_matched_intervention_column = results
        .iter()
        .any(|result| result.matched_intervention_label.is_some());
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        total => total,
        results => results,
        show_matched_intervention_column => show_matched_intervention_column,
        pagination_footer => pagination_footer,
        show_zero_result_nickname_hint => show_zero_result_nickname_hint,
        nickname_query => nickname_query,
        zero_result_broadening_hints => zero_result_broadening_hints,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}
