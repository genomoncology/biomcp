//! Offline question-to-skill routing for the `biomcp suggest` first move.

use std::collections::HashSet;

mod extract;
mod patterns;
mod routes;

use routes::ROUTES;

#[derive(Debug, Clone)]
pub(crate) struct SuggestArgs {
    pub question: String,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub(crate) struct SuggestResponse {
    pub matched_skill: Option<String>,
    pub summary: String,
    pub first_commands: Vec<String>,
    pub full_skill: Option<String>,
}

pub(super) struct QuestionContext<'a> {
    pub(super) original: &'a str,
    normalized: String,
}

/// Render a suggestion for a free-text biomedical question.
///
/// # Errors
///
/// Returns an error only when JSON serialization fails.
pub(crate) fn run(args: SuggestArgs, json: bool) -> anyhow::Result<String> {
    let response = suggest_question(&args.question);
    if json {
        Ok(crate::render::json::to_pretty(&response)?)
    } else {
        Ok(render_markdown(&response))
    }
}

pub(crate) fn suggest_question(question: &str) -> SuggestResponse {
    let context = QuestionContext::new(question);
    if context.normalized.is_empty() {
        return no_match_response();
    }

    for route in ROUTES {
        if let Some(commands) = (route.matcher)(&context) {
            return matched_response(route.slug, route.summary, commands);
        }
    }

    no_match_response()
}

#[cfg(test)]
pub(crate) fn route_examples() -> &'static [routes::SuggestRouteExample] {
    routes::route_examples()
}

impl QuestionContext<'_> {
    fn new(question: &str) -> QuestionContext<'_> {
        QuestionContext {
            original: question.trim(),
            normalized: extract::normalize_text(question),
        }
    }

    pub(super) fn has_any(&self, phrases: &[&str]) -> bool {
        phrases
            .iter()
            .any(|phrase| extract::contains_phrase(&self.normalized, phrase))
    }
}

fn matched_response(slug: &str, summary: &str, commands: Vec<String>) -> SuggestResponse {
    let mut seen = HashSet::new();
    let mut first_commands = Vec::new();
    for command in commands {
        if seen.insert(command.to_ascii_lowercase()) {
            first_commands.push(command);
        }
    }
    assert_eq!(
        first_commands.len(),
        2,
        "suggest route {slug} must produce exactly two starter commands",
    );

    SuggestResponse {
        matched_skill: Some(slug.to_string()),
        summary: summary.to_string(),
        first_commands,
        full_skill: Some(format!("biomcp skill {slug}")),
    }
}

fn no_match_response() -> SuggestResponse {
    SuggestResponse {
        matched_skill: None,
        summary: "No confident BioMCP skill match.".to_string(),
        first_commands: Vec::new(),
        full_skill: None,
    }
}

fn render_markdown(response: &SuggestResponse) -> String {
    let matched_skill = response.matched_skill.as_deref().unwrap_or("no match");
    let full_skill = response.full_skill.as_deref().unwrap_or("none");

    let mut out = String::new();
    out.push_str("# BioMCP Suggestion\n\n");
    if response.matched_skill.is_some() {
        out.push_str(&format!("- matched_skill: `{matched_skill}`\n"));
    } else {
        out.push_str("- matched_skill: no match\n");
    }
    out.push_str(&format!("- summary: {}\n", response.summary));
    out.push_str("- first_commands:\n");
    if response.first_commands.is_empty() {
        out.push_str("  none\n");
    } else {
        for (index, command) in response.first_commands.iter().enumerate() {
            out.push_str(&format!("  {}. `{command}`\n", index + 1));
        }
    }
    if response.full_skill.is_some() {
        out.push_str(&format!("- full_skill: `{full_skill}`\n"));
    } else {
        out.push_str("- full_skill: none\n");
    }

    if response.matched_skill.is_none() {
        out.push_str(
            "\nTry `biomcp skill list` to browse playbooks or `biomcp discover \"<question>\"` \
             when you need entity resolution instead of playbook selection.\n",
        );
    }
    out
}

#[cfg(test)]
mod tests {
    mod extract;
    mod render;
    mod routes;
}
