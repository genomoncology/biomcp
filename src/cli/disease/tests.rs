use clap::{CommandFactory, Parser};

use super::DiseaseCommand;
use super::dispatch::disease_search_json;
use crate::cli::{Cli, Commands, GetEntity, PaginationMeta};

fn render_disease_get_long_help() -> String {
    let mut command = Cli::command();
    let get = command
        .find_subcommand_mut("get")
        .expect("get subcommand should exist");
    let disease = get
        .find_subcommand_mut("disease")
        .expect("disease get subcommand should exist");
    let mut help = Vec::new();
    disease
        .write_long_help(&mut help)
        .expect("disease help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

#[test]
fn get_disease_help_includes_when_to_use_guidance() {
    let help = render_disease_get_long_help();

    assert!(help.contains("When to use:"));
    assert!(help.contains("normalized disease card"));
    assert!(help.contains("diagnostics, funding, survival, or clinical_features"));
    assert!(help.contains("MedlinePlus clinical-summary section for configured diseases"));
    assert!(help.contains("MedlinePlus clinical-summary rows for configured diseases"));
    assert!(help.contains("remains excluded from all"));
    assert!(!help.contains("currently empty until extraction support is wired"));
    assert!(!help.contains("accepted MedlinePlus clinical-feature foundation section"));
    assert!(help.contains("tuberculosis diagnostics"));
    assert!(help.contains("melanoma clinical_features"));
    assert!(help.contains("--name <NAME_OR_ID>"));
    assert!(help.contains("biomcp get disease --name \"chronic myeloid leukemia\" survival"));
    assert!(help.contains("search article -d"));
}

#[test]
fn get_disease_accepts_explicit_multi_word_name() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "get",
        "disease",
        "--name",
        "chronic myeloid leukemia",
        "survival",
    ])
    .expect("explicit multi-word disease name should parse");

    let Cli {
        command:
            Commands::Get {
                entity: GetEntity::Disease(crate::cli::disease::DiseaseGetArgs { name_or_id, args }),
            },
        ..
    } = cli
    else {
        panic!("expected get disease command");
    };

    assert_eq!(name_or_id.as_deref(), Some("chronic myeloid leukemia"));
    assert_eq!(args, vec!["survival".to_string()]);
}

#[test]
fn disease_trials_parses_source_and_limit() {
    let cli = Cli::try_parse_from([
        "biomcp", "disease", "trials", "melanoma", "--source", "nci", "--limit", "2",
    ])
    .expect("disease trials should parse");

    match cli.command {
        Commands::Disease {
            cmd:
                DiseaseCommand::Trials {
                    name,
                    limit,
                    offset,
                    source,
                },
        } => {
            assert_eq!(name, "melanoma");
            assert_eq!(limit, 2);
            assert_eq!(offset, 0);
            assert_eq!(source, "nci");
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn disease_trials_limit_rejects_too_big_before_lookup() {
    let cli = Cli::try_parse_from(["biomcp", "disease", "trials", "melanoma", "--limit", "51"])
        .expect("disease trials should parse before validation");

    let Commands::Disease { cmd } = cli.command else {
        panic!("expected disease command");
    };
    let DiseaseCommand::Trials { limit, offset, .. } = cmd else {
        panic!("expected disease trials command");
    };
    let err = super::dispatch::validate_related_limit("disease trials", limit, offset)
        .expect_err("too-large disease trials limit should fail fast");
    assert!(
        err.to_string()
            .contains("--limit for disease trials must be 1-50")
    );
}

#[test]
fn related_limit_rejects_zero_before_lookup() {
    let cli = Cli::try_parse_from(["biomcp", "disease", "articles", "melanoma", "--limit", "0"])
        .expect("disease articles should parse");

    let Cli {
        command: Commands::Disease { cmd },
        json,
        ..
    } = cli
    else {
        panic!("expected disease command");
    };

    assert!(!json);
    let DiseaseCommand::Articles { limit, offset, .. } = cmd else {
        panic!("expected disease articles command");
    };
    let err = super::dispatch::validate_related_limit("disease articles", limit, offset)
        .expect_err("zero disease articles limit should fail fast");
    assert!(
        err.to_string()
            .contains("--limit for disease articles must be 1-50")
    );
}

#[test]
fn disease_search_json_includes_fallback_meta_and_provenance() {
    let pagination = PaginationMeta::offset(0, 10, 1, Some(1));
    let results = vec![crate::entities::disease::DiseaseSearchResult {
        id: "MONDO:0000115".into(),
        name: "Arnold-Chiari malformation".into(),
        synonyms_preview: Some("Chiari malformation".into()),
        resolved_via: Some("MESH crosswalk".into()),
        source_id: Some("MESH:D001139".into()),
    }];
    let next_commands = crate::render::markdown::search_next_commands_disease(&results);
    let json = disease_search_json(results, pagination, true, next_commands, None)
        .expect("disease search json should render");

    let value: serde_json::Value =
        serde_json::from_str(&json).expect("json should parse successfully");
    assert_eq!(value["results"][0]["resolved_via"], "MESH crosswalk");
    assert_eq!(value["results"][0]["source_id"], "MESH:D001139");
    assert_eq!(
        value["_meta"]["next_commands"][0],
        serde_json::Value::String("biomcp get disease MONDO:0000115".into())
    );
    assert_eq!(
        value["_meta"]["next_commands"][1],
        serde_json::Value::String("biomcp list disease".into())
    );
    assert_eq!(value["_meta"]["fallback_used"], true);
}

#[test]
fn disease_search_json_includes_next_commands_for_direct_hits() {
    let pagination = PaginationMeta::offset(0, 10, 1, Some(1));
    let results = vec![crate::entities::disease::DiseaseSearchResult {
        id: "MONDO:0005105".into(),
        name: "melanoma".into(),
        synonyms_preview: Some("malignant melanoma".into()),
        resolved_via: None,
        source_id: None,
    }];
    let next_commands = crate::render::markdown::search_next_commands_disease(&results);
    let json = disease_search_json(results, pagination, false, next_commands, None)
        .expect("disease search json should render");

    let value: serde_json::Value =
        serde_json::from_str(&json).expect("json should parse successfully");
    assert_eq!(
        value["_meta"]["next_commands"][0],
        serde_json::Value::String("biomcp get disease MONDO:0005105".into())
    );
    assert_eq!(
        value["_meta"]["next_commands"][1],
        serde_json::Value::String("biomcp list disease".into())
    );
    assert!(value["_meta"].get("fallback_used").is_none());
    assert!(value["results"][0].get("resolved_via").is_none());
    assert!(value["results"][0].get("source_id").is_none());
}

#[test]
fn disease_search_json_preserves_fallback_with_workflow_meta() {
    let pagination = PaginationMeta::offset(0, 10, 1, Some(1));
    let results = vec![crate::entities::disease::DiseaseSearchResult {
        id: "MONDO:0000115".into(),
        name: "Arnold-Chiari malformation".into(),
        synonyms_preview: Some("Chiari malformation".into()),
        resolved_via: Some("MESH crosswalk".into()),
        source_id: Some("MESH:D001139".into()),
    }];
    let next_commands = crate::render::markdown::search_next_commands_disease(&results);
    let workflow =
        crate::workflow_ladders::meta_for(crate::workflow_ladders::Workflow::MutationCatalog)
            .expect("workflow metadata");
    let expected_first_ladder_command = workflow.ladder[0].command.clone();
    let json = disease_search_json(results, pagination, true, next_commands, Some(workflow))
        .expect("disease search json should render");

    let value: serde_json::Value =
        serde_json::from_str(&json).expect("json should parse successfully");
    assert_eq!(value["_meta"]["fallback_used"], true);
    assert_eq!(value["_meta"]["workflow"], "mutation-catalog");
    assert_eq!(
        value["_meta"]["ladder"][0]["command"],
        serde_json::Value::String(expected_first_ladder_command)
    );
    assert_eq!(
        value["_meta"]["next_commands"][0],
        serde_json::Value::String("biomcp get disease MONDO:0000115".into())
    );
}
