use super::*;

pub(crate) fn search_next_commands_trial(results: &[TrialSearchHit]) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    if let Some(nct_id) = results
        .first()
        .map(|result| quote_arg(result.nct_id()))
        .filter(|nct_id| !nct_id.is_empty())
    {
        out.push(format!("biomcp get trial {nct_id}"));
    }
    out.push("biomcp list trial".to_string());
    dedupe_markdown_commands(out)
}

pub(crate) fn related_trial(trial: &TrialResponse) -> Vec<String> {
    let mut out = Vec::new();
    if is_completed_or_terminated_trial_status(trial.trial().overall_status().code())
        && let Some(command) = trial_results_search_command(trial)
    {
        out.push(command);
    }
    if let Some(condition) = trial.trial().conditions().first().map(String::as_str) {
        let condition = quote_arg(condition);
        if !condition.is_empty() {
            out.push(format!("biomcp search disease --query {condition}"));
            out.push(format!("biomcp search article -d {condition}"));
            out.push(format!("biomcp search trial -c {condition}"));
        }
    }
    if let Some(detail) = trial
        .trial()
        .interventions()
        .unwrap_or_default()
        .iter()
        .find(|detail| {
            detail
                .other_names()
                .is_some_and(|names| names.iter().any(|name| !name.trim().is_empty()))
        })
    {
        if let Some(alias) = detail
            .other_names()
            .unwrap_or_default()
            .iter()
            .find(|name| !name.trim().is_empty())
        {
            let alias = force_quote_arg(alias);
            if !alias.is_empty() {
                out.push(format!("biomcp search drug -q {alias}"));
            }
        }
    } else if let Some(intervention) = trial.trial().interventions().unwrap_or_default().first() {
        let name = quote_arg(intervention.name());
        if !name.is_empty() {
            out.push(format!("biomcp search drug -q {name}"));
            out.push(format!("biomcp drug trials {name}"));
        }
    }
    dedupe_markdown_commands(out)
}
