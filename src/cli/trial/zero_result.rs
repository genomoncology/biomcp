use crate::entities::trial::{TrialSearchFilters, TrialSource};

fn has_text(value: Option<&str>) -> bool {
    value.map(str::trim).is_some_and(|value| !value.is_empty())
}

pub(super) fn has_active_trial_filters(filters: &TrialSearchFilters) -> bool {
    has_text(filters.condition.as_deref())
        || has_text(filters.intervention.as_deref())
        || has_text(filters.facility.as_deref())
        || has_text(filters.status.as_deref())
        || has_text(filters.phase.as_deref())
        || has_text(filters.study_type.as_deref())
        || filters.age.is_some()
        || has_text(filters.sex.as_deref())
        || has_text(filters.sponsor.as_deref())
        || has_text(filters.sponsor_type.as_deref())
        || has_text(filters.date_from.as_deref())
        || has_text(filters.date_to.as_deref())
        || has_text(filters.mutation.as_deref())
        || has_text(filters.criteria.as_deref())
        || has_text(filters.biomarker.as_deref())
        || has_text(filters.prior_therapies.as_deref())
        || has_text(filters.progression_on.as_deref())
        || has_text(filters.line_of_therapy.as_deref())
        || filters.results_available
        || filters.lat.is_some()
        || filters.lon.is_some()
        || filters.distance.is_some()
}

pub(super) fn zero_result_trial_broadening_hints(filters: &TrialSearchFilters) -> Vec<String> {
    let mut hints = Vec::new();
    if has_text(filters.mutation.as_deref()) {
        hints.push("loosen or drop `--mutation`; it is an exact free-text boolean search over title, summary, eligibility, and keywords, so specific protein changes can be brittle".to_string());
    } else {
        hints.push(
            "if a protein-change query is too narrow, use a looser `--mutation` term or omit it"
                .to_string(),
        );
    }
    if filters.distance.is_some() || filters.lat.is_some() || filters.lon.is_some() {
        hints.push("widen `--distance` or remove the geo filter".to_string());
    } else {
        hints.push(
            "add or widen `--distance` only after confirming geography is intended".to_string(),
        );
    }
    if has_text(filters.status.as_deref()) {
        hints.push(
            "relax `--status` to include non-recruiting or not-yet-recruiting trials".to_string(),
        );
    } else {
        hints.push(
            "try a broader `--status` set only if recruitment status is the likely blocker"
                .to_string(),
        );
    }
    hints.push("try `--biomarker <gene>` for a phrase search across keyword, intervention, and condition when `--mutation` is too specific".to_string());
    hints
}

pub(super) fn zero_result_trial_next_commands(filters: &TrialSearchFilters) -> Vec<String> {
    let mut commands = Vec::new();
    if has_text(filters.mutation.as_deref()) {
        let mut relaxed = filters.clone();
        relaxed.mutation = None;
        commands.push(trial_search_command(&relaxed));
    }
    if filters.distance.is_some() {
        let mut relaxed = filters.clone();
        relaxed.distance = filters
            .distance
            .and_then(|value| value.checked_mul(2))
            .or(Some(500));
        commands.push(trial_search_command(&relaxed));
    }
    if has_text(filters.status.as_deref()) {
        let mut relaxed = filters.clone();
        relaxed.status = None;
        commands.push(trial_search_command(&relaxed));
    }
    if let Some(gene) = mutation_gene_hint(filters.mutation.as_deref()) {
        let mut relaxed = filters.clone();
        relaxed.mutation = None;
        relaxed.biomarker = Some(gene);
        commands.push(trial_search_command(&relaxed));
    }
    commands.push("biomcp list trial".to_string());
    super::super::normalize_next_commands(commands)
}

fn mutation_gene_hint(mutation: Option<&str>) -> Option<String> {
    mutation
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.split_whitespace().next())
        .map(str::to_string)
}

fn push_text_flag(command: &mut Vec<String>, flag: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        command.push(flag.to_string());
        command.push(crate::render::markdown::quote_arg(value));
    }
}

fn trial_search_command(filters: &TrialSearchFilters) -> String {
    let mut command = vec![
        "biomcp".to_string(),
        "search".to_string(),
        "trial".to_string(),
    ];
    push_text_flag(&mut command, "-c", filters.condition.as_deref());
    push_text_flag(&mut command, "-i", filters.intervention.as_deref());
    if filters.no_alias_expand {
        command.push("--no-alias-expand".to_string());
    }
    push_text_flag(&mut command, "--facility", filters.facility.as_deref());
    push_text_flag(&mut command, "-s", filters.status.as_deref());
    push_text_flag(&mut command, "-p", filters.phase.as_deref());
    push_text_flag(&mut command, "--study-type", filters.study_type.as_deref());
    if let Some(age) = filters.age {
        command.push("--age".to_string());
        command.push(age.to_string());
    }
    push_text_flag(&mut command, "--sex", filters.sex.as_deref());
    push_text_flag(&mut command, "--sponsor", filters.sponsor.as_deref());
    push_text_flag(
        &mut command,
        "--sponsor-type",
        filters.sponsor_type.as_deref(),
    );
    push_text_flag(&mut command, "--date-from", filters.date_from.as_deref());
    push_text_flag(&mut command, "--date-to", filters.date_to.as_deref());
    push_text_flag(&mut command, "--mutation", filters.mutation.as_deref());
    push_text_flag(&mut command, "--criteria", filters.criteria.as_deref());
    push_text_flag(&mut command, "--biomarker", filters.biomarker.as_deref());
    push_text_flag(
        &mut command,
        "--prior-therapies",
        filters.prior_therapies.as_deref(),
    );
    push_text_flag(
        &mut command,
        "--progression-on",
        filters.progression_on.as_deref(),
    );
    push_text_flag(
        &mut command,
        "--line-of-therapy",
        filters.line_of_therapy.as_deref(),
    );
    if let (Some(lat), Some(lon), Some(distance)) = (filters.lat, filters.lon, filters.distance) {
        command.push("--lat".to_string());
        command.push(lat.to_string());
        command.push("--lon".to_string());
        command.push(lon.to_string());
        command.push("--distance".to_string());
        command.push(distance.to_string());
    }
    if filters.results_available {
        command.push("--has-results".to_string());
    }
    if matches!(filters.source, TrialSource::NciCts) {
        command.push("--source".to_string());
        command.push("nci".to_string());
    }
    command.join(" ")
}
