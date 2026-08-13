use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Commands, GetEntity, SearchEntity};
use crate::entities::diagnostic::{Diagnostic, DiagnosticSourceFilter};

fn diagnostic_fixture(accession: &str) -> Diagnostic {
    Diagnostic {
        section_outcomes: crate::entities::diagnostic::default_diagnostic_section_outcomes(),
        source: "gtr".to_string(),
        source_id: accession.to_string(),
        accession: accession.to_string(),
        name: "BRCA1 Hereditary Cancer Panel".to_string(),
        test_type: Some("Molecular genetics".to_string()),
        manufacturer: Some("GenomOncology Lab".to_string()),
        target_marker: None,
        regulatory_version: None,
        prequalification_year: None,
        laboratory: None,
        institution: None,
        country: None,
        clia_number: None,
        state_licenses: None,
        current_status: None,
        public_status: None,
        method_categories: vec!["Molecular genetics".to_string()],
        genes: Some(vec!["BRCA1".to_string(), "BARD1".to_string()]),
        conditions: Some(vec!["Breast cancer".to_string()]),
        methods: None,
        regulatory: None,
    }
}

#[test]
fn search_diagnostic_parses_filter_only_flags() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "diagnostic",
        "--gene",
        "BRCA1",
        "--type",
        "molecular",
        "--limit",
        "2",
    ])
    .expect("search diagnostic should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Diagnostic(crate::cli::diagnostic::DiagnosticSearchArgs {
                        source,
                        gene,
                        disease,
                        test_type,
                        manufacturer,
                        limit,
                        offset,
                        ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search diagnostic command");
    };

    assert!(matches!(
        DiagnosticSourceFilter::from(source),
        DiagnosticSourceFilter::All
    ));
    assert_eq!(gene.as_deref(), Some("BRCA1"));
    assert_eq!(disease, None);
    assert_eq!(test_type.as_deref(), Some("molecular"));
    assert_eq!(manufacturer, None);
    assert_eq!(limit, 2);
    assert_eq!(offset, 0);
}

#[test]
fn get_diagnostic_help_mentions_supported_sections() {
    let mut command = Cli::command();
    let get = command
        .find_subcommand_mut("get")
        .expect("get subcommand should exist");
    let diagnostic = get
        .find_subcommand_mut("diagnostic")
        .expect("diagnostic subcommand should exist");
    let mut help = Vec::new();
    diagnostic
        .write_long_help(&mut help)
        .expect("diagnostic help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("genes, conditions, methods, regulatory, all"));
    assert!(help.contains("biomcp get diagnostic GTR000006692.3"));
    assert!(help.contains("biomcp get diagnostic GTR000006692.3 genes"));
    assert!(help.contains("biomcp get diagnostic GTR000006692.3 regulatory"));
    assert!(help.contains("GTR000006692.3 or \"ITPW02232- TC40\""));
    assert!(help.contains("biomcp get diagnostic \"ITPW02232- TC40\""));
    assert!(help.contains("biomcp get diagnostic \"ITPW02232- TC40\" conditions"));
    assert!(help.contains("biomcp get diagnostic \"ITPW02232- TC40\" regulatory"));
    assert!(!help.contains("GTR000000001.1"));
}

#[test]
fn search_diagnostic_help_mentions_source_aware_examples() {
    let mut command = Cli::command();
    let search = command
        .find_subcommand_mut("search")
        .expect("search subcommand should exist");
    let diagnostic = search
        .find_subcommand_mut("diagnostic")
        .expect("diagnostic subcommand should exist");
    let mut help = Vec::new();
    diagnostic
        .write_long_help(&mut help)
        .expect("diagnostic help should render");
    let help = String::from_utf8(help).expect("help should be utf-8");

    assert!(help.contains("biomcp search diagnostic --disease HIV --source who-ivd --limit 5"));
    assert!(help.contains("Disease filters require at least 3 alphanumeric characters"));
    assert!(
        help.contains(
            "biomcp search diagnostic --gene EGFR --type Clinical --source gtr --limit 5"
        )
    );
    assert!(help.contains("`--source` accepts gtr, who-ivd, or all"));
}

#[test]
fn search_args_reject_zero_limit_before_gtr_lookup() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "diagnostic",
        "--gene",
        "BRCA1",
        "--limit",
        "0",
    ])
    .expect("search diagnostic should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Diagnostic(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected search diagnostic command");
    };

    assert!(!json);
    let err = super::dispatch::validate_search_args(&args)
        .expect_err("zero diagnostic limit should fail fast");
    assert!(err.to_string().contains("--limit must be between 1 and 50"));
}

#[test]
fn handle_search_json_includes_suggestions_for_true_zero_result() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "--json",
        "search",
        "diagnostic",
        "--disease",
        "qzvxxptl",
        "--source",
        "gtr",
        "--limit",
        "5",
    ])
    .expect("search diagnostic should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Diagnostic(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected search diagnostic command");
    };
    super::dispatch::validate_search_args(&args).expect("search args should validate");

    let text = super::dispatch::diagnostic_search_json(
        Vec::new(),
        Some(0),
        args.limit,
        args.offset,
        false,
    )
    .expect("search diagnostic json");
    let value: serde_json::Value = serde_json::from_str(&text).expect("valid json");

    assert!(json);
    assert_eq!(value["count"], 0);
    assert_eq!(value["results"], serde_json::json!([]));
    assert_eq!(value["pagination"]["total"], 0);
    assert_eq!(value["_meta"]["next_commands"], serde_json::json!([]));
    assert!(
        value["_meta"]["suggestions"]
            .as_array()
            .is_some_and(|commands| commands.iter().any(|cmd| cmd == "biomcp list diagnostic"))
    );
}

#[test]
fn handle_search_json_omits_suggestions_for_high_offset_empty_page() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "--json",
        "search",
        "diagnostic",
        "--disease",
        "tuberculosis",
        "--source",
        "gtr",
        "--limit",
        "5",
        "--offset",
        "99",
    ])
    .expect("search diagnostic should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Diagnostic(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected search diagnostic command");
    };
    super::dispatch::validate_search_args(&args).expect("search args should validate");

    let text = super::dispatch::diagnostic_search_json(
        Vec::new(),
        Some(10),
        args.limit,
        args.offset,
        false,
    )
    .expect("search diagnostic json");
    let value: serde_json::Value = serde_json::from_str(&text).expect("valid json");

    assert!(json);
    assert_eq!(value["count"], 0);
    assert_eq!(value["results"], serde_json::json!([]));
    assert!(
        value["pagination"]["total"]
            .as_u64()
            .is_some_and(|total| total > 0)
    );
    assert!(value.get("_meta").is_none());
}

#[test]
fn diagnostic_json_compacts_nested_lists_and_full_restores_them() {
    let row = crate::entities::diagnostic::DiagnosticSearchResult {
        source: "gtr".into(),
        accession: "GTR1.1".into(),
        name: "panel".into(),
        test_type: None,
        manufacturer_or_lab: None,
        genes: (0..7).map(|index| format!("G{index}")).collect(),
        conditions: (0..6).map(|index| format!("C{index}")).collect(),
    };
    let compact: serde_json::Value = serde_json::from_str(
        &super::dispatch::diagnostic_search_json(vec![row.clone()], Some(1), 5, 0, false).unwrap(),
    )
    .unwrap();
    assert_eq!(compact["results"][0]["genes"].as_array().unwrap().len(), 5);
    assert_eq!(compact["results"][0]["genes_total"], 7);
    assert_eq!(compact["results"][0]["genes_has_more"], true);

    let full: serde_json::Value = serde_json::from_str(
        &super::dispatch::diagnostic_search_json(vec![row], Some(1), 5, 0, true).unwrap(),
    )
    .unwrap();
    assert_eq!(full["results"][0]["genes"].as_array().unwrap().len(), 7);
    assert_eq!(full["results"][0]["genes_has_more"], false);
}

#[test]
fn handle_get_honors_trailing_json_flag_after_sections() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "get",
        "diagnostic",
        "GTR000000001.1",
        "genes",
        "--json",
    ])
    .expect("get diagnostic should parse");

    let Cli {
        command: Commands::Get {
            entity: GetEntity::Diagnostic(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected get diagnostic command");
    };
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);

    let text =
        super::dispatch::diagnostic_get_json(&diagnostic_fixture(&args.accession), &sections)
            .expect("get diagnostic json");
    let value: serde_json::Value = serde_json::from_str(&text).expect("valid json");

    assert!(!json);
    assert!(json_override);
    assert_eq!(value["accession"], "GTR000000001.1");
    assert!(
        value.get("genes").is_some(),
        "genes section should be present"
    );
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| commands
                .iter()
                .any(|cmd| cmd == "biomcp get diagnostic GTR000000001.1 conditions"))
    );
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| commands
                .iter()
                .all(|cmd| cmd != "biomcp get diagnostic GTR000000001.1 genes"))
    );
}
