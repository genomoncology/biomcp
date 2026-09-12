use super::*;

pub(crate) fn trial_section_sources(trial: &TrialResponse) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let source = trial_source_label(Some(trial.source()));
    let source_ref = [source.as_str()];
    let shared = trial.trial();
    let overview_present = has_text(trial.nct_id())
        || has_text(shared.brief_title())
        || has_text(shared.overall_status().code())
        || !shared.phases().is_empty()
        || has_text(shared.study_type().code())
        || trial.has_eligibility_age()
        || has_text(shared.lead_sponsor_name())
        || shared.enrollment_count().is_some()
        || shared
            .start_date()
            .is_some_and(|value| !value.trim().is_empty())
        || shared
            .completion_date()
            .is_some_and(|value| !value.trim().is_empty());
    push_section(
        &mut out,
        overview_present,
        "overview",
        "Overview",
        source_ref,
    );
    push_section(
        &mut out,
        !shared.conditions().is_empty(),
        "conditions",
        "Conditions",
        source_ref,
    );
    push_section(
        &mut out,
        !shared.interventions().unwrap_or_default().is_empty(),
        "interventions",
        "Interventions",
        source_ref,
    );
    push_section(
        &mut out,
        shared
            .brief_summary()
            .is_some_and(|value| !value.trim().is_empty()),
        "summary",
        "Summary",
        source_ref,
    );
    push_section(
        &mut out,
        shared.eligibility().is_some(),
        "eligibility",
        "Eligibility",
        source_ref,
    );
    push_section(
        &mut out,
        trial.location_count() > 0,
        "locations",
        "Locations",
        source_ref,
    );
    push_section(
        &mut out,
        shared.planned_outcomes().is_some(),
        "outcomes",
        "Outcomes",
        source_ref,
    );
    push_section(&mut out, trial.has_arms(), "arms", "Arms", source_ref);
    push_section(
        &mut out,
        shared.references().is_some(),
        "references",
        "References",
        source_ref,
    );
    out
}
