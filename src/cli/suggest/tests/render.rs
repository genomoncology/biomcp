//! Suggest response rendering, JSON shape, and no-match contract tests.

use std::collections::BTreeSet;

use super::super::*;

#[test]
fn ticket_examples_keep_exact_commands_and_response_shape() {
    let treatment = suggest_question("What drugs treat melanoma?");
    assert_eq!(
        treatment,
        SuggestResponse {
            matched_skill: Some("treatment-lookup".to_string()),
            summary: "Use the treatment lookup playbook for therapy or approved-drug questions."
                .to_string(),
            first_commands: vec![
                "biomcp search drug --indication melanoma --limit 5".to_string(),
                "biomcp search article -d melanoma --type review --limit 5".to_string(),
            ],
            full_skill: Some("biomcp skill treatment-lookup".to_string()),
        }
    );

    let variant = suggest_question("Is variant rs113488022 pathogenic in melanoma?");
    assert_eq!(
        variant.first_commands,
        vec![
            "biomcp get variant rs113488022 clinvar predictions population",
            "biomcp get variant rs113488022 civic cgi",
        ]
    );

    let json = crate::render::json::to_pretty(&variant).expect("json");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    let keys = value
        .as_object()
        .expect("object")
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        keys,
        BTreeSet::from([
            "first_commands".to_string(),
            "full_skill".to_string(),
            "matched_skill".to_string(),
            "summary".to_string(),
        ])
    );
}

#[test]
fn no_match_is_successful_with_null_json_fields() {
    let response = suggest_question("What is x?");
    assert_eq!(
        response,
        SuggestResponse {
            matched_skill: None,
            summary: "No confident BioMCP skill match.".to_string(),
            first_commands: vec![],
            full_skill: None,
        }
    );

    let json = crate::render::json::to_pretty(&response).expect("json");
    assert!(json.contains("\"matched_skill\": null"));
    assert!(json.contains("\"full_skill\": null"));
    assert!(json.contains("\"first_commands\": []"));
}

#[test]
fn markdown_exposes_labels_and_no_match_guidance() {
    let matched = render_markdown(&suggest_question("What drugs treat melanoma?"));
    assert!(matched.contains("# BioMCP Suggestion"));
    assert!(matched.contains("- matched_skill: `treatment-lookup`"));
    assert!(matched.contains("- summary: "));
    assert!(matched.contains("- first_commands:"));
    assert!(matched.contains("- full_skill: `biomcp skill treatment-lookup`"));

    let no_match = render_markdown(&suggest_question("What is x?"));
    assert!(no_match.contains("- matched_skill: no match"));
    assert!(no_match.contains("No confident BioMCP skill match."));
    assert!(no_match.contains("biomcp skill list"));
    assert!(no_match.contains("biomcp discover \"<question>\""));
}
