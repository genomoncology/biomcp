//! Route catalog parity, route examples, and route edge-case tests.

use std::collections::BTreeSet;

use clap::Parser;

use super::super::*;
use crate::cli::Cli;

fn parse_cmd(cmd: &str) {
    let args = shlex::split(cmd).unwrap_or_else(|| panic!("shlex failed on: {cmd}"));
    Cli::try_parse_from(args).unwrap_or_else(|err| panic!("failed to parse {cmd}: {err}"));
}

#[test]
fn route_examples_cover_shipped_skill_slugs() {
    let shipped = crate::cli::skill::list_use_case_refs()
        .expect("skills")
        .into_iter()
        .map(|case| case.slug.to_string())
        .collect::<BTreeSet<_>>();
    let routed = routes::ROUTES
        .iter()
        .map(|route| route.slug.to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(routed, shipped);
}

#[test]
fn route_examples_match_expected_skills_commands_and_parse() {
    for example in routes::route_examples() {
        let response = suggest_question(example.question);
        assert_eq!(
            response.matched_skill.as_deref(),
            Some(example.expected_skill),
            "{}",
            example.question
        );
        assert_eq!(response.first_commands.len(), 2, "{}", example.question);
        assert_eq!(
            response.full_skill.as_deref(),
            Some(format!("biomcp skill {}", example.expected_skill).as_str())
        );
        assert_eq!(response.first_commands, example.expected_commands);
        for command in response.first_commands {
            parse_cmd(&command);
        }
    }
}

#[test]
fn guardrails_avoid_common_false_positives() {
    assert_eq!(suggest_question("What is x?").matched_skill, None);
    assert_eq!(
        suggest_question("Which gene is responsible for disease?").matched_skill,
        None
    );
    assert_ne!(
        suggest_question("Find article evidence from 2024 about melanoma")
            .matched_skill
            .as_deref(),
        Some("article-follow-up")
    );
    assert_eq!(
        suggest_question("Tell me about variant rs113488022").matched_skill,
        None
    );
    assert_eq!(
        suggest_question("What does gene brca1 do?")
            .matched_skill
            .as_deref(),
        Some("gene-function-localization")
    );
}

#[test]
fn mechanism_resistance_to_drug_prefers_drug_anchor() {
    let response = suggest_question("What is the mechanism of resistance to imatinib?");
    assert_eq!(response.matched_skill.as_deref(), Some("mechanism-pathway"));
    assert_eq!(
        response.first_commands,
        vec![
            concat!("biomcp search drug ", "imatinib --limit 5").to_string(),
            concat!("biomcp get drug ", "imatinib targets regulatory").to_string(),
        ]
    );
}

#[test]
fn mechanism_resistance_against_prefers_drug_anchor() {
    let response = suggest_question("What is the mechanism of resistance against imatinib?");
    assert_eq!(response.matched_skill.as_deref(), Some("mechanism-pathway"));
    assert_eq!(
        response.first_commands,
        vec![
            concat!("biomcp search drug ", "imatinib --limit 5").to_string(),
            concat!("biomcp get drug ", "imatinib targets regulatory").to_string(),
        ]
    );
}

#[test]
fn mechanism_drug_branch_handles_imatinib_resistance_develop() {
    let response = suggest_question("How does imatinib resistance develop?");
    assert_eq!(response.matched_skill.as_deref(), Some("mechanism-pathway"));
    assert_eq!(
        response.first_commands,
        vec![
            concat!("biomcp search drug ", "imatinib --limit 5").to_string(),
            concat!("biomcp get drug ", "imatinib targets regulatory").to_string(),
        ]
    );
}

#[test]
fn route_specific_contract_edges_match_design() {
    let regulatory = suggest_question("When was imatinib approved by FDA?");
    assert_eq!(
        regulatory.first_commands[0],
        "biomcp get drug imatinib regulatory --region us"
    );

    let intervention = suggest_question("Are there recruiting trials with imatinib?");
    assert_eq!(
        intervention.first_commands,
        [
            "biomcp search trial -i imatinib --status recruiting --limit 5",
            "biomcp search article --drug imatinib --type review --limit 5",
        ]
    );

    let symptom = suggest_question("symptoms include seizure and developmental delay");
    assert_eq!(
        symptom.first_commands,
        [
            "biomcp discover \"seizure and developmental delay\"",
            "biomcp search phenotype \"seizure and developmental delay\" --limit 5",
        ]
    );

    let bare_vs = suggest_question("Goldberg-Shprintzen syndrome vs Shprintzen-Goldberg syndrome");
    assert_eq!(
        bare_vs.matched_skill.as_deref(),
        Some("syndrome-disambiguation")
    );

    let difference = suggest_question(
        "What is the difference between Goldberg-Shprintzen syndrome and Shprintzen-Goldberg syndrome?",
    );
    assert_eq!(
        difference.matched_skill.as_deref(),
        Some("syndrome-disambiguation")
    );

    let no_evidence = suggest_question("No evidence for aspirin and melanoma?");
    assert_eq!(
        no_evidence.matched_skill.as_deref(),
        Some("negative-evidence")
    );
    assert_eq!(
        no_evidence.first_commands,
        [
            "biomcp search article -k \"aspirin melanoma\" --type review --limit 5",
            "biomcp search article -k \"aspirin melanoma association\" --limit 5",
        ]
    );
}
