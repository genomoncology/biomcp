use clap::Parser;
use serde::Serialize;

use crate::cli::Cli;

mod diagnostic;
mod disease_trial;
mod gene_article;
mod pathway_adverse_event;
mod variant_drug;

fn collect_next_commands(json: &str) -> Vec<String> {
    let value: serde_json::Value = serde_json::from_str(json).expect("valid json");
    value["_meta"]["next_commands"]
        .as_array()
        .expect("next_commands array")
        .iter()
        .map(|cmd| cmd.as_str().expect("command string").to_string())
        .collect()
}

fn collect_suggestions(json: &str) -> Vec<String> {
    let value: serde_json::Value = serde_json::from_str(json).expect("valid json");
    value["_meta"]["suggestions"]
        .as_array()
        .expect("suggestions array")
        .iter()
        .map(|cmd| cmd.as_str().expect("command string").to_string())
        .collect()
}

fn assert_json_next_commands_parse(label: &str, json: &str) {
    let value: serde_json::Value =
        serde_json::from_str(json).unwrap_or_else(|e| panic!("{label}: invalid json: {e}"));
    let cmds = value["_meta"]["next_commands"]
        .as_array()
        .unwrap_or_else(|| panic!("{label}: missing _meta.next_commands"));
    assert!(
        !cmds.is_empty(),
        "{label}: expected at least one next_command"
    );
    for cmd in cmds {
        let cmd = cmd
            .as_str()
            .unwrap_or_else(|| panic!("{label}: next_command was not a string"));
        let argv = shlex::split(cmd).unwrap_or_else(|| panic!("{label}: shlex failed on: {cmd}"));
        Cli::try_parse_from(argv)
            .unwrap_or_else(|e| panic!("{label}: failed to parse '{cmd}': {e}"));
    }
}

fn assert_entity_json_next_commands<T, L>(
    label: &str,
    entity: &T,
    evidence_urls: Vec<(L, String)>,
    next_commands: Vec<String>,
    section_sources: Vec<crate::render::provenance::SectionSource>,
) where
    T: Serialize,
    L: AsRef<str>,
{
    let json =
        crate::render::json::to_entity_json(entity, evidence_urls, next_commands, section_sources)
            .unwrap_or_else(|e| panic!("{label}: failed to render entity json: {e}"));
    assert_json_next_commands_parse(label, &json);
}
