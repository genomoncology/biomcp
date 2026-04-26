//! Extraction and shell-sensitive command-anchor tests for suggestions.

use clap::Parser;

use super::super::*;
use crate::cli::Cli;

fn parse_cmd(cmd: &str) {
    let args = shlex::split(cmd).unwrap_or_else(|| panic!("shlex failed on: {cmd}"));
    Cli::try_parse_from(args).unwrap_or_else(|err| panic!("failed to parse {cmd}: {err}"));
}

#[test]
fn generated_commands_quote_user_derived_multiword_and_shell_metacharacter_anchors() {
    let response = suggest_question("What drugs treat lung cancer?");
    assert_eq!(response.matched_skill.as_deref(), Some("treatment-lookup"));
    assert!(response.first_commands[0].contains("\"lung cancer\""));
    assert!(response.first_commands[1].contains("\"lung cancer\""));

    let response = suggest_question("What drugs treat lung cancer; rm -rf /?");
    assert_eq!(response.matched_skill.as_deref(), Some("treatment-lookup"));
    assert!(response.first_commands[0].contains("\"lung cancer; rm -rf /\""));
    parse_cmd(&response.first_commands[0]);
}

#[test]
fn mechanism_resistance_to_drug_quotes_shell_sensitive_anchor() {
    let response = suggest_question("What is the mechanism of resistance to imatinib; rm -rf /?");
    assert_eq!(response.matched_skill.as_deref(), Some("mechanism-pathway"));
    assert_eq!(
        response.first_commands,
        vec![
            format!(
                "biomcp search drug {} --limit 5",
                extract::quote("imatinib; rm -rf /")
            ),
            format!(
                "biomcp get drug {} targets regulatory",
                extract::quote("imatinib; rm -rf /")
            ),
        ]
    );
    parse_cmd(&response.first_commands[0]);
}
