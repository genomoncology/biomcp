use super::*;

fn discovery_drug() -> Drug {
    serde_json::from_value(serde_json::json!({
        "name": "eflornithine",
        "targets": ["ODC1"]
    }))
    .expect("drug fixture")
}

#[test]
fn drug_command_discovery_matches_default_projection_order() {
    let drug = discovery_drug();
    let discovery = drug_command_discovery(&drug, &[], DrugRegion::Us);
    assert_eq!(
        discovery.next_commands,
        vec![
            "biomcp get drug eflornithine approvals",
            "biomcp get drug eflornithine label",
            "biomcp get drug eflornithine regulatory --region us",
            "biomcp get drug eflornithine all --region us",
            "biomcp search article --drug eflornithine --type review --limit 5",
            "biomcp drug trials eflornithine",
            "biomcp drug adverse-events eflornithine",
            "biomcp search pgx -d eflornithine",
            "biomcp get gene ODC1",
        ]
    );
    assert_eq!(
        discovery
            .sections
            .iter()
            .map(|entry| entry.section.as_str())
            .collect::<Vec<_>>(),
        vec!["approvals", "label", "regulatory"]
    );
    assert_eq!(
        discovery.all.as_deref(),
        Some("biomcp get drug eflornithine all --region us")
    );
}

#[test]
fn drug_command_discovery_uses_loaded_plan_and_region_safe_recovery() {
    let mut drug = discovery_drug();
    drug.section_outcomes.complete(
        "safety",
        crate::entities::section_outcome::SectionOutcome::unavailable("test outage"),
    );
    let discovery = drug_command_discovery(
        &drug,
        &[
            " safety ".to_string(),
            "-j".to_string(),
            "SAFETY".to_string(),
        ],
        DrugRegion::Eu,
    );
    assert_eq!(
        discovery
            .recovery
            .iter()
            .map(|entry| entry.command.as_str())
            .collect::<Vec<_>>(),
        vec!["biomcp get drug eflornithine safety --region eu"]
    );
    assert_eq!(
        discovery
            .sections
            .iter()
            .map(|entry| entry.command.as_str())
            .collect::<Vec<_>>(),
        vec![
            "biomcp get drug eflornithine approvals",
            "biomcp get drug eflornithine label",
            "biomcp get drug eflornithine regulatory --region eu",
        ]
    );
    assert!(
        !discovery
            .next_commands
            .iter()
            .any(|command| command == "biomcp get drug eflornithine safety")
    );
}

#[test]
fn drug_command_discovery_table_covers_each_section_and_mixed_plans() {
    let cases = [
        (vec![], vec!["approvals", "label", "regulatory"], true),
        (
            vec!["label"],
            vec!["approvals", "regulatory", "safety"],
            true,
        ),
        (
            vec!["regulatory"],
            vec!["approvals", "label", "safety"],
            true,
        ),
        (
            vec!["safety"],
            vec!["approvals", "label", "regulatory"],
            true,
        ),
        (
            vec!["shortage"],
            vec!["approvals", "label", "regulatory"],
            true,
        ),
        (
            vec!["targets"],
            vec!["approvals", "label", "regulatory"],
            true,
        ),
        (
            vec!["indications"],
            vec!["approvals", "label", "regulatory"],
            true,
        ),
        (
            vec!["interactions"],
            vec!["approvals", "label", "regulatory"],
            true,
        ),
        (
            vec!["civic"],
            vec!["approvals", "label", "regulatory"],
            true,
        ),
        (vec!["all"], vec!["approvals"], false),
        (
            vec![" approvals ", "LABEL", "approvals"],
            vec!["regulatory", "safety", "shortage"],
            true,
        ),
        (vec!["all", "approvals", "label"], vec![], false),
    ];
    for (requested, expected_sections, expect_all) in cases {
        let discovery = drug_command_discovery(
            &discovery_drug(),
            &requested
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>(),
            DrugRegion::Us,
        );
        assert_eq!(
            discovery
                .sections
                .iter()
                .map(|entry| entry.section.as_str())
                .collect::<Vec<_>>(),
            expected_sections,
        );
        assert_eq!(discovery.all.is_some(), expect_all);
        let mut expected_flat = expected_sections
            .iter()
            .map(|section| {
                let region = if matches!(*section, "regulatory" | "safety" | "shortage") {
                    " --region us"
                } else {
                    ""
                };
                format!("biomcp get drug eflornithine {section}{region}")
            })
            .collect::<Vec<_>>();
        if expect_all {
            expected_flat.push("biomcp get drug eflornithine all --region us".to_string());
        }
        expected_flat.extend([
            "biomcp search article --drug eflornithine --type review --limit 5".to_string(),
            "biomcp drug trials eflornithine".to_string(),
            "biomcp drug adverse-events eflornithine".to_string(),
            "biomcp search pgx -d eflornithine".to_string(),
            "biomcp get gene ODC1".to_string(),
        ]);
        assert_eq!(discovery.next_commands, expected_flat);
        assert!(
            discovery
                .related
                .first()
                .is_some_and(|command| command.contains("type review"))
        );
    }
}

#[test]
fn drug_command_discovery_covers_all_regions_and_who_exclusions() {
    for (region, label, safety_and_shortage) in [
        (DrugRegion::Us, "us", true),
        (DrugRegion::Eu, "eu", true),
        (DrugRegion::Who, "who", false),
        (DrugRegion::All, "all", true),
    ] {
        let discovery = drug_command_discovery(&discovery_drug(), &["targets".to_string()], region);
        let mut expected = vec![
            format!("biomcp get drug eflornithine approvals"),
            format!("biomcp get drug eflornithine label"),
            format!("biomcp get drug eflornithine regulatory --region {label}"),
            format!("biomcp get drug eflornithine all --region {label}"),
            "biomcp search article --drug eflornithine --type review --limit 5".to_string(),
            "biomcp drug trials eflornithine".to_string(),
            "biomcp drug adverse-events eflornithine".to_string(),
            "biomcp search pgx -d eflornithine".to_string(),
            "biomcp get gene ODC1".to_string(),
        ];
        if !safety_and_shortage {
            expected
                .retain(|command| !command.contains(" safety ") && !command.contains(" shortage "));
        }
        assert_eq!(discovery.next_commands, expected);
        let expected_sections = vec![
            "biomcp get drug eflornithine approvals".to_string(),
            "biomcp get drug eflornithine label".to_string(),
            format!("biomcp get drug eflornithine regulatory --region {label}"),
        ];
        assert_eq!(
            discovery
                .sections
                .iter()
                .map(|entry| entry.command.clone())
                .collect::<Vec<_>>(),
            expected_sections,
        );
        let expected_all = format!("biomcp get drug eflornithine all --region {label}");
        assert_eq!(discovery.all.as_deref(), Some(expected_all.as_str()));
    }
}

#[test]
fn drug_command_discovery_recovery_matrix_covers_registered_states() {
    let recovery_sections = [
        "approvals",
        "safety",
        "targets",
        "indications",
        "interactions",
        "civic",
    ];
    for (state, outcome) in [
        (
            "unavailable",
            crate::entities::section_outcome::SectionOutcome::unavailable("fixture outage"),
        ),
        (
            "degraded",
            crate::entities::section_outcome::SectionOutcome::degraded(
                ["fixture"],
                "fixture degradation",
            ),
        ),
    ] {
        for section in recovery_sections {
            let mut drug = discovery_drug();
            drug.section_outcomes.complete(section, outcome.clone());
            let discovery = drug_command_discovery(&drug, &[section.to_string()], DrugRegion::Eu);
            assert_eq!(
                discovery.recovery,
                vec![DrugCommand {
                    section: section.to_string(),
                    command: format!(
                        "biomcp get drug eflornithine {section}{}",
                        if matches!(section, "safety") {
                            " --region eu"
                        } else {
                            ""
                        }
                    ),
                }],
                "missing {state} recovery for {section}",
            );
            assert_eq!(
                discovery.next_commands.first(),
                Some(&discovery.recovery[0].command),
                "{state} recovery must remain first for {section}",
            );
            assert_eq!(
                discovery
                    .next_commands
                    .iter()
                    .filter(|command| *command == &discovery.recovery[0].command)
                    .count(),
                1,
            );
        }
    }

    for state in [
        SectionOutcomeState::Data,
        SectionOutcomeState::Empty,
        SectionOutcomeState::Inapplicable,
        SectionOutcomeState::NotRequested,
    ] {
        for section in recovery_sections {
            let mut drug = discovery_drug();
            if state != SectionOutcomeState::NotRequested {
                let outcome = match state {
                    SectionOutcomeState::Data => {
                        crate::entities::section_outcome::SectionOutcome::data("fixture")
                    }
                    SectionOutcomeState::Empty => {
                        crate::entities::section_outcome::SectionOutcome::empty("fixture")
                    }
                    SectionOutcomeState::Inapplicable => {
                        crate::entities::section_outcome::SectionOutcome::inapplicable(
                            "not applicable",
                        )
                    }
                    _ => unreachable!(),
                };
                drug.section_outcomes.complete(section, outcome);
            }
            let discovery = drug_command_discovery(&drug, &[section.to_string()], DrugRegion::Us);
            assert!(
                discovery.recovery.is_empty(),
                "{state:?} must not recover {section}"
            );
        }
    }
}

#[test]
fn drug_command_discovery_cap_matrix_keeps_exact_categories_and_bytes() {
    let recovery_order = [
        "approvals",
        "safety",
        "targets",
        "indications",
        "interactions",
        "civic",
    ];
    let related = [
        "biomcp search article --drug eflornithine --type review --limit 5",
        "biomcp drug trials eflornithine",
        "biomcp drug adverse-events eflornithine",
        "biomcp search pgx -d eflornithine",
        "biomcp get gene ODC1",
    ];
    for (count, expected) in [
        (4, vec![0, 1, 2, 3, 6, 7, 8, 9, 10]),
        (5, vec![0, 1, 2, 3, 4, 6, 7, 8, 9, 10]),
        (6, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
    ] {
        let mut drug = discovery_drug();
        for section in recovery_order.iter().take(count) {
            drug.section_outcomes.complete(
                section,
                crate::entities::section_outcome::SectionOutcome::unavailable("fixture outage"),
            );
        }
        let discovery = drug_command_discovery(
            &drug,
            &["all".to_string(), "approvals".to_string()],
            DrugRegion::Us,
        );
        let candidates = recovery_order
            .iter()
            .map(|section| format!("biomcp get drug eflornithine {section}"))
            .chain(related.iter().map(|command| (*command).to_string()))
            .collect::<Vec<_>>();
        let expected_flat = expected
            .into_iter()
            .map(|index| {
                let mut command = candidates[index].clone();
                if recovery_order
                    .iter()
                    .any(|section| command == format!("biomcp get drug eflornithine {section}"))
                    && command.contains(" safety")
                {
                    command.push_str(" --region us");
                }
                command
            })
            .collect::<Vec<_>>();
        assert_eq!(discovery.next_commands, expected_flat, "count={count}");
        assert_eq!(
            discovery
                .recovery
                .iter()
                .map(|entry| entry.command.as_str())
                .collect::<Vec<_>>(),
            discovery.next_commands[..count]
        );
        assert_eq!(
            discovery
                .related
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            &related[..(10 - count).min(related.len())],
            "count={count} related displacement",
        );
        let uncapped_category_count = count + related.len();
        assert_eq!(
            uncapped_category_count,
            [9, 10, 11][count - 4],
            "categorized 9/10/11 boundary count={count}",
        );
        assert_eq!(
            discovery.next_commands.len(),
            uncapped_category_count.min(10),
            "flattened cap count={count}",
        );
    }
}

#[test]
fn drug_command_discovery_dedupes_exact_bytes_without_case_folding() {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    push_exact_capped("same bytes".to_string(), &mut seen, &mut output);
    push_exact_capped("same bytes".to_string(), &mut seen, &mut output);
    push_exact_capped("SAME BYTES".to_string(), &mut seen, &mut output);
    assert_eq!(output, vec!["same bytes", "SAME BYTES"]);

    let discovery = drug_command_discovery(&discovery_drug(), &[], DrugRegion::Us);
    let unique = discovery.next_commands.iter().collect::<HashSet<_>>();
    assert_eq!(unique.len(), discovery.next_commands.len());
}

#[test]
fn drug_command_discovery_eu_recovery_markdown_is_adjacent_and_unique() {
    let mut drug = discovery_drug();
    drug.section_outcomes.complete(
        "safety",
        crate::entities::section_outcome::SectionOutcome::unavailable("fixture outage"),
    );
    let markdown = crate::render::markdown::drug_markdown_with_region(
        &drug,
        &["safety".to_string()],
        DrugRegion::Eu,
        false,
    )
    .expect("EU recovery markdown");
    let retry = "Retry: `biomcp get drug eflornithine safety --region eu`";
    assert_eq!(markdown.matches(retry).count(), 1);
    let status = markdown.find("**Safety status").expect("safety status");
    let retry_position = markdown.find(retry).expect("safety retry");
    assert!(retry_position > status);
    assert!(!markdown[status..retry_position].contains("\n##"));
    assert_eq!(
        markdown
            .matches("biomcp get drug eflornithine safety --region eu")
            .count(),
        1
    );
}

#[test]
fn drug_command_discovery_parses_hostile_identity_as_one_cli_argument() {
    use crate::cli::Cli;
    use clap::Parser;

    let mut drug = discovery_drug();
    drug.name = "  Dose 'x' \"q\" \\ $x `rm` ; & path  ".to_string();
    let identity = drug.name.trim();
    let discovery = drug_command_discovery(&drug, &[], DrugRegion::Us);
    let command = &discovery.next_commands[0];
    let argv = shlex::split(command).expect("owner command must round trip");
    let parsed = Cli::try_parse_from(argv).expect("owner command must parse with CLI");
    let crate::cli::Commands::Get {
        entity: crate::cli::GetEntity::Drug(args),
    } = parsed.command
    else {
        panic!("owner command must route to get drug");
    };
    assert_eq!(args.args.first().map(String::as_str), Some(identity));
    assert!(command.contains("\\$x"));
    assert!(command.contains("\\`rm\\`"));
    assert!(command.contains(" ; & "));
}

#[test]
fn drug_markdown_cap_displacement_goldens_only_surviving_related_commands() {
    let mut drug = discovery_drug();
    for section in [
        "approvals",
        "safety",
        "targets",
        "indications",
        "interactions",
        "civic",
    ] {
        drug.section_outcomes.complete(
            section,
            crate::entities::section_outcome::SectionOutcome::unavailable("fixture outage"),
        );
    }
    let markdown = crate::render::markdown::drug_markdown(&drug, &["all".to_string()])
        .expect("capped drug markdown");
    let guidance = markdown
        .split_once("More:\n")
        .map(|(_, suffix)| suffix)
        .expect("More guidance");
    assert_eq!(
        guidance,
        r#"  biomcp get drug eflornithine approvals   - Drugs@FDA approval history
See also:
  biomcp search article --drug eflornithine --type review --limit 5   - supplement sparse structured data with review literature for indication context
  biomcp drug trials eflornithine
  biomcp drug adverse-events eflornithine   - inspect safety reports and adverse-event signal
  biomcp search pgx -d eflornithine   - pharmacogenomics interactions
"#
    );
}

#[test]
fn drug_markdown_batch_item_guidance_matches_shared_owner() {
    let markdown =
        crate::render::markdown::drug_markdown(&discovery_drug(), &["label".to_string()])
            .expect("batch item markdown");
    let guidance = markdown
        .split_once("More:\n")
        .map(|(_, suffix)| suffix)
        .unwrap();
    assert!(guidance.starts_with("  biomcp get drug eflornithine approvals"));
    assert!(guidance.contains("biomcp get drug eflornithine regulatory --region us"));
    assert!(!guidance.contains("biomcp get drug eflornithine label"));
    assert_eq!(
        guidance
            .matches("biomcp get drug eflornithine regulatory --region us")
            .count(),
        1
    );
}

#[test]
fn drug_command_discovery_caps_after_recovery_before_related_candidates() {
    let recovery_order = [
        "approvals",
        "safety",
        "targets",
        "indications",
        "interactions",
        "civic",
    ];
    for count in [4, 5, 6] {
        let mut drug = discovery_drug();
        for key in recovery_order.iter().take(count) {
            drug.section_outcomes.complete(
                key,
                crate::entities::section_outcome::SectionOutcome::unavailable("test outage"),
            );
        }
        let discovery = drug_command_discovery(
            &drug,
            &["all".to_string(), "approvals".to_string()],
            DrugRegion::Us,
        );
        assert_eq!(discovery.sections.len(), 0);
        assert_eq!(
            discovery.next_commands.len(),
            if count == 4 { 9 } else { 10 }
        );
        assert!(discovery.next_commands.len() <= 10);
        assert_eq!(discovery.recovery.len(), count);
        assert_eq!(discovery.related.len(), if count == 6 { 4 } else { 5 });
        if count == 6 {
            assert!(
                !discovery
                    .next_commands
                    .iter()
                    .any(|command| command.contains("get gene"))
            );
        }
    }
}

#[test]
fn drug_command_discovery_shell_quotes_resolved_identity_as_one_argument() {
    let mut drug = discovery_drug();
    drug.name = " Dose $x `rm` ; & \\\"quoted\\\" \\\\ path ".to_string();
    let discovery = drug_command_discovery(&drug, &[], DrugRegion::Us);
    assert_eq!(
        discovery.next_commands[0],
        format!(
            "biomcp get drug {} approvals",
            crate::next_command::shell_quote_arg(drug.name.trim())
        )
    );
    assert!(discovery.next_commands[0].contains("\\$x"));
    assert!(discovery.next_commands[0].contains("\\`rm\\`"));
    assert!(discovery.next_commands[0].contains(" ; & "));
    assert!(
        discovery
            .next_commands
            .iter()
            .all(|command| command.matches("biomcp get drug").count() <= 1)
    );
}

#[test]
fn drug_command_discovery_omits_review_and_blank_target_pivots() {
    let mut drug = discovery_drug();
    drug.ema_regulatory = Some(Vec::new());
    drug.indications.push("test indication".to_string());
    drug.targets.clear();
    let discovery = drug_command_discovery(&drug, &[], DrugRegion::Us);
    assert_eq!(
        discovery.related,
        vec![
            "biomcp drug trials eflornithine",
            "biomcp drug adverse-events eflornithine",
            "biomcp search pgx -d eflornithine",
        ]
    );
    assert!(
        !discovery
            .next_commands
            .iter()
            .any(|command| command.contains("search article --drug"))
    );
    assert!(
        !discovery
            .next_commands
            .iter()
            .any(|command| command.starts_with("biomcp get gene"))
    );
}

#[test]
fn drug_markdown_guidance_categories_are_the_shared_flattened_plan() {
    let markdown =
        crate::render::markdown::drug_markdown(&discovery_drug(), &[]).expect("drug markdown");
    let guidance = markdown
        .split_once("More:\n")
        .map(|(_, suffix)| suffix)
        .expect("More guidance");
    assert_eq!(
        guidance,
        r#"  biomcp get drug eflornithine approvals   - Drugs@FDA approval history
  biomcp get drug eflornithine label   - approved-indication and FDA label detail beyond the base card
  biomcp get drug eflornithine regulatory --region us   - approval and supplement history; use only if the base card lacks approval context

All:
  biomcp get drug eflornithine all --region us
See also:
  biomcp search article --drug eflornithine --type review --limit 5   - supplement sparse structured data with review literature for indication context
  biomcp drug trials eflornithine
  biomcp drug adverse-events eflornithine   - inspect safety reports and adverse-event signal
  biomcp search pgx -d eflornithine   - pharmacogenomics interactions
  biomcp get gene ODC1
"#
    );
}

#[test]
fn sections_pathway_for_kegg_excludes_unsupported_sections() {
    let pathway = Pathway {
        section_outcomes: Default::default(),
        source: "KEGG".to_string(),
        id: "hsa05200".to_string(),
        name: "Pathways in cancer".to_string(),
        species: None,
        summary: None,
        genes: Vec::new(),
        events: Vec::new(),
        enrichment: Vec::new(),
    };

    let sections = sections_pathway(&pathway, &[]);
    assert_eq!(sections, vec!["genes".to_string()]);
}

#[test]
fn sections_diagnostic_omit_requested_section_from_more_block() {
    let diagnostic = Diagnostic {
        section_outcomes: crate::entities::diagnostic::default_diagnostic_section_outcomes(),
        source: "gtr".to_string(),
        source_id: "GTR000000001.1".to_string(),
        accession: "GTR000000001.1".to_string(),
        name: "BRCA1 Hereditary Cancer Panel".to_string(),
        test_type: Some("molecular".to_string()),
        manufacturer: Some("OncoPanel BRCA1".to_string()),
        target_marker: None,
        regulatory_version: None,
        prequalification_year: None,
        laboratory: Some("GenomOncology Lab".to_string()),
        institution: Some("GenomOncology Institute".to_string()),
        country: Some("USA".to_string()),
        clia_number: Some("12D3456789".to_string()),
        state_licenses: Some("NY|CA".to_string()),
        current_status: Some("Current".to_string()),
        public_status: Some("Public".to_string()),
        method_categories: vec!["Molecular genetics".to_string()],
        genes: Some(vec!["BRCA1".to_string()]),
        conditions: Some(vec!["Breast cancer".to_string()]),
        methods: Some(vec!["Sequence analysis".to_string()]),
        regulatory: None,
    };

    let sections = sections_diagnostic(&diagnostic, &["genes".to_string()]);
    assert_eq!(
        sections,
        vec![
            "conditions".to_string(),
            "methods".to_string(),
            "regulatory".to_string()
        ]
    );

    let commands = diagnostic_next_commands(&diagnostic, &["genes".to_string()]);
    assert_eq!(
        commands,
        vec![
            "biomcp get diagnostic GTR000000001.1 conditions".to_string(),
            "biomcp get diagnostic GTR000000001.1 methods".to_string(),
            "biomcp get diagnostic GTR000000001.1 regulatory".to_string(),
            "biomcp list diagnostic".to_string()
        ]
    );
}

#[test]
fn sections_diagnostic_for_who_only_offer_conditions_and_quote_accession() {
    let diagnostic = Diagnostic {
        section_outcomes: crate::entities::diagnostic::default_diagnostic_section_outcomes(),
        source: "who-ivd".to_string(),
        source_id: "ITPW02232- TC40".to_string(),
        accession: "ITPW02232- TC40".to_string(),
        name: "ONE STEP Anti-HIV (1&2) Test".to_string(),
        test_type: Some("Immunochromatographic (lateral flow)".to_string()),
        manufacturer: Some("InTec Products, Inc.".to_string()),
        target_marker: Some("HIV".to_string()),
        regulatory_version: Some("Rest-of-World".to_string()),
        prequalification_year: Some("2019".to_string()),
        laboratory: None,
        institution: None,
        country: None,
        clia_number: None,
        state_licenses: None,
        current_status: None,
        public_status: None,
        method_categories: vec![],
        genes: None,
        conditions: None,
        methods: None,
        regulatory: None,
    };

    assert_eq!(
        sections_diagnostic(&diagnostic, &[]),
        vec!["conditions".to_string(), "regulatory".to_string()]
    );
    assert_eq!(
        diagnostic_next_commands(&diagnostic, &[]),
        vec![
            "biomcp get diagnostic \"ITPW02232- TC40\" conditions".to_string(),
            "biomcp get diagnostic \"ITPW02232- TC40\" regulatory".to_string(),
            "biomcp list diagnostic".to_string()
        ]
    );
}

#[test]
fn diagnostic_more_block_keeps_four_visible_section_commands() {
    let diagnostic = Diagnostic {
        section_outcomes: crate::entities::diagnostic::default_diagnostic_section_outcomes(),
        source: "gtr".to_string(),
        source_id: "GTR000000001.1".to_string(),
        accession: "GTR000000001.1".to_string(),
        name: "BRCA1 Hereditary Cancer Panel".to_string(),
        test_type: Some("molecular".to_string()),
        manufacturer: Some("OncoPanel BRCA1".to_string()),
        target_marker: None,
        regulatory_version: None,
        prequalification_year: None,
        laboratory: Some("GenomOncology Lab".to_string()),
        institution: Some("GenomOncology Institute".to_string()),
        country: Some("USA".to_string()),
        clia_number: Some("12D3456789".to_string()),
        state_licenses: Some("NY|CA".to_string()),
        current_status: Some("Current".to_string()),
        public_status: Some("Public".to_string()),
        method_categories: vec!["Molecular genetics".to_string()],
        genes: None,
        conditions: None,
        methods: None,
        regulatory: None,
    };

    let block = format_sections_block(
        "diagnostic",
        &diagnostic.accession,
        sections_diagnostic(&diagnostic, &[]),
    );
    assert!(block.contains("biomcp get diagnostic GTR000000001.1 genes"));
    assert!(block.contains("biomcp get diagnostic GTR000000001.1 conditions"));
    assert!(block.contains("biomcp get diagnostic GTR000000001.1 methods"));
    assert!(block.contains("biomcp get diagnostic GTR000000001.1 regulatory"));
}

#[test]
fn sections_pathway_for_reactome_keeps_full_supported_set() {
    let pathway = Pathway {
        section_outcomes: Default::default(),
        source: "Reactome".to_string(),
        id: "R-HSA-5673001".to_string(),
        name: "RAF/MAP kinase cascade".to_string(),
        species: None,
        summary: None,
        genes: Vec::new(),
        events: Vec::new(),
        enrichment: Vec::new(),
    };

    let sections = sections_pathway(&pathway, &[]);
    assert_eq!(
        sections,
        vec![
            "genes".to_string(),
            "events".to_string(),
            "enrichment".to_string()
        ]
    );
}

#[test]
fn format_sections_block_renders_described_executable_commands() {
    let block = format_sections_block(
        "gene",
        "TP53",
        vec![
            "pathways".to_string(),
            "hpa".to_string(),
            "diseases".to_string(),
            "protein".to_string(),
        ],
    );

    assert!(block.contains("More:"));
    assert!(block.contains("biomcp get gene TP53 pathways"));
    assert!(block.contains("Reactome/KEGG pathway context"));
    assert!(block.contains("biomcp get gene TP53 hpa"));
    assert!(block.contains("Human Protein Atlas tissue expression and localization"));
    assert!(block.contains("biomcp get gene TP53 diseases"));
    assert!(block.contains("disease associations"));
    assert!(block.contains("All:"));
    assert!(block.contains("biomcp get gene TP53 all"));
}

#[test]
fn format_sections_block_keeps_gene_ontology_in_top_more_entries() {
    let block = format_sections_block(
        "gene",
        "NANOG",
        vec![
            "pathways".to_string(),
            "ontology".to_string(),
            "diseases".to_string(),
            "protein".to_string(),
        ],
    );

    let pathways = block
        .find("biomcp get gene NANOG pathways")
        .expect("pathways command");
    let ontology = block
        .find("biomcp get gene NANOG ontology")
        .expect("ontology command");
    let diseases = block
        .find("biomcp get gene NANOG diseases")
        .expect("diseases command");
    assert!(pathways < ontology);
    assert!(ontology < diseases);
}

#[test]
fn sections_disease_base_card_surfaces_diagnostics_before_optional_sections() {
    let disease = Disease {
        id: "MONDO:0005105".to_string(),
        name: "melanoma".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: Vec::new(),
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        survival: None,
        survival_note: None,
        civic: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: std::collections::HashMap::new(),
    };

    let sections = sections_disease(&disease, &[]);
    assert_eq!(
        sections
            .iter()
            .take(6)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec![
            "genes",
            "pathways",
            "phenotypes",
            "diagnostics",
            "clinical_features",
            "survival"
        ]
    );

    let block = format_sections_block("disease", &disease.id, sections);
    let genes = block
        .find("biomcp get disease MONDO:0005105 genes")
        .expect("genes command");
    let pathways = block
        .find("biomcp get disease MONDO:0005105 pathways")
        .expect("pathways command");
    let phenotypes = block
        .find("biomcp get disease MONDO:0005105 phenotypes")
        .expect("phenotypes command");
    let diagnostics = block
        .find("biomcp get disease MONDO:0005105 diagnostics")
        .expect("diagnostics command");
    let survival = block
        .find("biomcp get disease MONDO:0005105 survival")
        .expect("survival command");
    let clinical_features = block
        .find("biomcp get disease MONDO:0005105 clinical_features")
        .expect("clinical features command");
    assert!(genes < pathways);
    assert!(pathways < phenotypes);
    assert!(phenotypes < diagnostics);
    assert!(diagnostics < clinical_features);
    assert!(clinical_features < survival);
    assert!(block.contains("diagnostic tests for this condition from GTR and WHO IVD"));
    assert!(block.contains("Monarch/HPO phenotype rows as clinical features"));
    assert!(block.contains("SEER Explorer cancer survival rates"));
    assert!(!block.contains("biomcp get disease MONDO:0005105 variants"));
}

#[test]
fn sections_gene_base_card_surfaces_diagnostics_as_fourth_command() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: Some("164757".to_string()),
        uniprot_id: Some("P15056".to_string()),
        summary: None,
        gene_type: None,
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: None,
        go: None,
        interactions: None,
        civic: None,
        expression: None,
        hpa: None,
        druggability: None,
        clingen: None,
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let sections = sections_gene(&gene, &[]);
    assert_eq!(
        sections
            .iter()
            .take(4)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["pathways", "ontology", "diseases", "diagnostics"]
    );

    let block = format_sections_block("gene", &gene.symbol, sections);
    let pathways = block
        .find("biomcp get gene BRAF pathways")
        .expect("pathways command");
    let ontology = block
        .find("biomcp get gene BRAF ontology")
        .expect("ontology command");
    let diseases = block
        .find("biomcp get gene BRAF diseases")
        .expect("diseases command");
    let diagnostics = block
        .find("biomcp get gene BRAF diagnostics")
        .expect("diagnostics command");
    assert!(pathways < ontology);
    assert!(ontology < diseases);
    assert!(diseases < diagnostics);
    assert!(block.contains("diagnostic tests for this gene from GTR"));
    assert!(!block.contains("biomcp get gene BRAF protein"));
}

#[test]
fn format_sections_block_describes_guardrailed_drug_and_trial_sections() {
    let drug_block = format_sections_block(
        "drug",
        "pembrolizumab",
        vec![
            "label".to_string(),
            "regulatory".to_string(),
            "safety".to_string(),
        ],
    );

    assert!(drug_block.contains(
            "biomcp get drug pembrolizumab label   - approved-indication and FDA label detail beyond the base card"
        ));
    assert!(drug_block.contains(
            "biomcp get drug pembrolizumab regulatory   - approval and supplement history; use only if the base card lacks approval context"
        ));
    assert!(drug_block.contains(
            "biomcp get drug pembrolizumab safety   - regulatory safety detail; use `biomcp drug adverse-events <name>` first when you want post-marketing signal"
        ));

    let terminated = crate::entities::trial::Trial {
        nct_id: "NCT02576665".to_string(),
        source: None,
        title: "Completed trial".to_string(),
        status: "TERMINATED".to_string(),
        why_stopped: None,
        phase: None,
        study_type: None,
        conditions: vec!["melanoma".to_string()],
        design: crate::entities::trial::TrialDesign::from_names(&["trametinib"]),
        sponsor: None,
        enrollment: None,
        summary: None,
        start_date: None,
        completion_date: None,
        eligibility: None,
        eligibility_provenance: None,
        contacts: None,
        locations: None,
        outcomes: None,
        references: None,
    };
    let terminated_sections = sections_trial(&terminated, &[]);
    assert_eq!(terminated_sections[0], "outcomes");
    assert_eq!(terminated_sections[1], "references");
    assert_eq!(terminated_sections[2], "arms");

    let trial_block =
        format_sections_block("trial", &terminated.nct_id, terminated_sections.clone());
    assert!(
        trial_block.contains(
            "biomcp get trial NCT02576665 outcomes   - endpoint measures and time frames"
        )
    );
    assert!(trial_block.contains(
        "biomcp get trial NCT02576665 references   - linked publications and PMID citations"
    ));
    assert!(
        trial_block.contains(
            "biomcp get trial NCT02576665 arms   - study arms and assigned interventions"
        )
    );

    let recruiting = crate::entities::trial::Trial {
        status: "Recruiting".to_string(),
        ..terminated
    };
    let recruiting_sections = sections_trial(&recruiting, &[]);
    assert_eq!(recruiting_sections[0], "eligibility");
    assert_eq!(recruiting_sections[1], "contacts");
    assert_eq!(recruiting_sections[2], "locations");
}
