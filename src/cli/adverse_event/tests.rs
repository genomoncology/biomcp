use clap::Parser;

use crate::cli::{Cli, Commands, GetEntity, SearchEntity};

#[test]
fn search_adverse_event_parses_serious_default_and_limit() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "adverse-event",
        "-d",
        "ibuprofen",
        "--serious",
        "--limit",
        "2",
    ])
    .expect("adverse-event search should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::AdverseEvent(crate::cli::adverse_event::AdverseEventSearchArgs {
                        drug,
                        serious,
                        r#type,
                        limit,
                        offset,
                        ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected adverse-event search command");
    };

    assert_eq!(drug.as_deref(), Some("ibuprofen"));
    assert_eq!(serious.as_deref(), Some("any"));
    assert_eq!(r#type, "faers");
    assert_eq!(limit, 2);
    assert_eq!(offset, 0);
}

#[test]
fn search_adverse_event_parses_source_filter() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "adverse-event",
        "MMR vaccine",
        "--source",
        "vaers",
    ])
    .expect("adverse-event search should parse source filter");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::AdverseEvent(crate::cli::adverse_event::AdverseEventSearchArgs {
                        positional_query,
                        r#type,
                        source,
                        ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected adverse-event search command");
    };

    assert_eq!(positional_query.as_deref(), Some("MMR vaccine"));
    assert_eq!(r#type, "faers");
    assert_eq!(source, "vaers");
}

#[test]
fn get_adverse_event_parses_sections() {
    let cli = Cli::try_parse_from(["biomcp", "get", "adverse-event", "10222779", "reactions"])
        .expect("adverse-event get should parse");

    let Cli {
        command:
            Commands::Get {
                entity:
                    GetEntity::AdverseEvent(crate::cli::adverse_event::AdverseEventGetArgs {
                        report_id,
                        sections,
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected adverse-event get command");
    };

    assert_eq!(report_id, "10222779");
    assert_eq!(sections, vec!["reactions".to_string()]);
}

#[test]
fn resolved_device_report_rejects_every_named_section_including_all() {
    let report = crate::entities::adverse_event::AdverseEventReport::Device(
        crate::entities::adverse_event::DeviceEvent {
            report_id: "123".into(),
            report_number: None,
            device: "pump".into(),
            manufacturer: None,
            event_type: None,
            date: None,
            description: None,
        },
    );

    for section in ["reactions", "all"] {
        crate::entities::adverse_event::parse_sections(&[section.into()])
            .expect("syntactically valid section");
        let error = super::dispatch::validate_resolved_sections(&report, true)
            .expect_err("device sections must be rejected after resolution");
        assert!(error.to_string().contains("resolved to a device report"));
    }
    super::dispatch::validate_resolved_sections(&report, false)
        .expect("unsectioned device report remains valid");
}

#[test]
fn search_plan_rejects_positional_drug_alias_for_device() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "adverse-event",
        "pembrolizumab",
        "--type",
        "device",
    ])
    .expect("adverse-event device search should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::AdverseEvent(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected adverse-event search command");
    };

    assert!(!json);
    let err = super::dispatch::search_plan_from_args(&args)
        .expect_err("device query should reject positional drug alias");
    assert!(
        err.to_string()
            .contains("--drug cannot be used with --type device")
    );
}

#[test]
fn search_adverse_event_device_rejects_positional_drug_alias() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "adverse-event",
        "pembrolizumab",
        "--type",
        "device",
    ])
    .expect("adverse-event device search should parse");
    let Cli {
        command: Commands::Search {
            entity: SearchEntity::AdverseEvent(args),
        },
        ..
    } = cli
    else {
        panic!("expected adverse-event search command");
    };
    let err = super::dispatch::search_plan_from_args(&args)
        .expect_err("device query should reject positional drug alias");
    assert!(
        err.to_string()
            .contains("--drug cannot be used with --type device")
    );
}

#[test]
fn search_plan_count_without_source_uses_faers() {
    let args = adverse_event_search_args(&["pembrolizumab", "--count", "reaction"]);

    let plan = super::dispatch::search_plan_from_args(&args)
        .expect("a count aggregation should select FAERS");

    assert_eq!(
        plan.source_filter,
        crate::entities::adverse_event::AdverseEventSourceFilter::Faers
    );
}

#[test]
fn search_plan_rejects_count_for_vaers_source() {
    for count in ["reaction", ""] {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "adverse-event",
            "MMR vaccine",
            "--source",
            "vaers",
            "--count",
            count,
        ])
        .expect("adverse-event vaers count query should parse");

        let Cli {
            command:
                Commands::Search {
                    entity: SearchEntity::AdverseEvent(args),
                },
            json,
            ..
        } = cli
        else {
            panic!("expected adverse-event search command");
        };

        assert!(!json);
        let err = super::dispatch::search_plan_from_args(&args)
            .expect_err("vaers search should reject count");
        assert!(
            err.to_string()
                .contains("--source vaers does not support: --count")
        );
    }
}

#[test]
fn search_plan_rejects_nondefault_source_for_recall() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "adverse-event",
        "ibuprofen",
        "--type",
        "recall",
        "--source",
        "vaers",
    ])
    .expect("recall search should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::AdverseEvent(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected adverse-event search command");
    };

    assert!(!json);
    let err = super::dispatch::search_plan_from_args(&args)
        .expect_err("recall query should reject non-default source");
    assert!(
        err.to_string()
            .contains("--source is only supported for --type faers adverse-event search")
    );
}

#[test]
fn search_plan_rejects_nondefault_source_for_device() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "adverse-event",
        "--device",
        "pump",
        "--type",
        "device",
        "--source",
        "faers",
    ])
    .expect("device search should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::AdverseEvent(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected adverse-event search command");
    };

    assert!(!json);
    let err = super::dispatch::search_plan_from_args(&args)
        .expect_err("device query should reject non-default source");
    assert!(
        err.to_string()
            .contains("--source is only supported for --type faers adverse-event search")
    );
}

fn adverse_event_search_args(extra: &[&str]) -> super::AdverseEventSearchArgs {
    let mut argv = vec!["biomcp", "search", "adverse-event"];
    argv.extend_from_slice(extra);
    let cli = Cli::try_parse_from(argv).expect("adverse-event search should parse");
    let Cli {
        command: Commands::Search {
            entity: SearchEntity::AdverseEvent(args),
        },
        ..
    } = cli
    else {
        panic!("expected adverse-event search command");
    };
    args
}

#[test]
fn search_plan_rejects_every_inapplicable_route_filter() {
    let cases: &[(&[&str], &str)] = &[
        (
            &["aspirin", "--classification", "Class I"],
            "--classification",
        ),
        (
            &["MMR vaccine", "--source", "vaers", "--reaction", "fever"],
            "--reaction",
        ),
        (
            &["MMR vaccine", "--source", "vaers", "--offset", "1"],
            "--offset",
        ),
        (
            &[
                "aspirin", "--source", "faers", "--count", "reaction", "--offset", "1",
            ],
            "--count requires --offset 0",
        ),
        (
            &["aspirin", "--type", "recall", "--reaction", "rash"],
            "--reaction",
        ),
        (
            &["aspirin", "--type", "recall", "--serious", "death"],
            "--serious",
        ),
        (
            &[
                "--type",
                "device",
                "--device",
                "pump",
                "--classification",
                "Class I",
            ],
            "--classification",
        ),
        (
            &[
                "--type",
                "device",
                "--device",
                "pump",
                "--serious",
                "hospitalization",
            ],
            "Expected one of: any, death, injury",
        ),
    ];

    for (argv, expected) in cases {
        let args = adverse_event_search_args(argv);
        let err = match super::dispatch::search_plan_from_args(&args) {
            Ok(_) => panic!("expected rejection for {argv:?}"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains(expected),
            "{argv:?}: expected {expected:?} in {err}"
        );
    }
}

#[test]
fn search_plan_accepts_each_route_specific_contract() {
    let cases: &[&[&str]] = &[
        &[
            "aspirin",
            "--source",
            "faers",
            "--reaction",
            "rash",
            "--outcome",
            "death",
            "--serious",
            "hospitalization",
            "--date-from",
            "2024",
            "--date-to",
            "2025",
            "--suspect-only",
            "--sex",
            "f",
            "--age-min",
            "18",
            "--age-max",
            "65",
            "--reporter",
            "physician",
            "--count",
            "reaction",
            "--limit",
            "5",
        ],
        &["MMR vaccine", "--source", "vaers", "--limit", "3"],
        &[
            "aspirin",
            "--type",
            "recall",
            "--classification",
            "Class I",
            "--limit",
            "5",
            "--offset",
            "2",
        ],
        &[
            "--type",
            "device",
            "--device",
            "pump",
            "--manufacturer",
            "Acme",
            "--product-code",
            "PQP",
            "--date-from",
            "2024",
            "--serious",
            "injury",
            "--limit",
            "5",
            "--offset",
            "2",
        ],
        &[
            "MMR vaccine",
            "--source",
            "all",
            "--reaction",
            "fever",
            "--offset",
            "2",
        ],
    ];

    for argv in cases {
        let args = adverse_event_search_args(argv);
        super::dispatch::search_plan_from_args(&args)
            .unwrap_or_else(|err| panic!("expected accepted plan for {argv:?}: {err}"));
    }
}

fn assert_each_filter_is_rejected(base: &[&str], filters: &[&[&str]]) {
    for filter in filters {
        let mut argv = base.to_vec();
        argv.extend_from_slice(filter);
        let args = adverse_event_search_args(&argv);
        assert!(
            super::dispatch::search_plan_from_args(&args).is_err(),
            "expected rejection for {argv:?}"
        );
    }
}

#[test]
fn search_plan_covers_the_complete_rejected_filter_matrix() {
    assert_each_filter_is_rejected(
        &["aspirin", "--source", "faers"],
        &[
            &["--classification", "Class I"],
            &["--device", "pump"],
            &["--manufacturer", "Acme"],
            &["--product-code", "PQP"],
        ],
    );
    assert_each_filter_is_rejected(
        &["MMR vaccine", "--source", "vaers"],
        &[
            &["--reaction", "rash"],
            &["--outcome", "death"],
            &["--serious", "death"],
            &["--date-from", "2024"],
            &["--date-to", "2025"],
            &["--suspect-only"],
            &["--sex", "f"],
            &["--age-min", "18"],
            &["--age-max", "65"],
            &["--reporter", "physician"],
            &["--count", "reaction"],
            &["--offset", "1"],
            &["--classification", "Class I"],
            &["--device", "pump"],
            &["--manufacturer", "Acme"],
            &["--product-code", "PQP"],
        ],
    );
    assert_each_filter_is_rejected(
        &["aspirin", "--type", "recall"],
        &[
            &["--reaction", "rash"],
            &["--outcome", "death"],
            &["--serious", "death"],
            &["--date-from", "2024"],
            &["--date-to", "2025"],
            &["--suspect-only"],
            &["--sex", "f"],
            &["--age-min", "18"],
            &["--age-max", "65"],
            &["--reporter", "physician"],
            &["--count", "reaction"],
            &["--device", "pump"],
            &["--manufacturer", "Acme"],
            &["--product-code", "PQP"],
        ],
    );
    assert_each_filter_is_rejected(
        &["--type", "device", "--device", "pump"],
        &[
            &["--reaction", "rash"],
            &["--outcome", "death"],
            &["--date-to", "2025"],
            &["--suspect-only"],
            &["--sex", "f"],
            &["--age-min", "18"],
            &["--age-max", "65"],
            &["--reporter", "physician"],
            &["--count", "reaction"],
            &["--classification", "Class I"],
        ],
    );
}
