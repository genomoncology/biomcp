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
    let discovery = drug_command_discovery(&discovery_drug(), &[], DrugRegion::Us);
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
fn drug_command_discovery_respects_loaded_sections_regions_and_who_exclusions() {
    let requests = [
        (vec!["label"], vec!["approvals", "regulatory", "safety"]),
        (vec!["all"], vec!["approvals"]),
        (
            vec![" approvals ", "LABEL", "approvals"],
            vec!["regulatory", "safety", "shortage"],
        ),
    ];
    for (requested, expected) in requests {
        let requested = requested
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let discovery = drug_command_discovery(&discovery_drug(), &requested, DrugRegion::Us);
        assert_eq!(
            discovery
                .sections
                .iter()
                .map(|entry| entry.section.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            discovery.all.is_some(),
            requested.iter().all(|s| s != "all")
        );
    }

    for (region, label) in [
        (DrugRegion::Us, "us"),
        (DrugRegion::Eu, "eu"),
        (DrugRegion::All, "all"),
    ] {
        let discovery = drug_command_discovery(&discovery_drug(), &["targets".to_string()], region);
        assert!(discovery.next_commands.contains(&format!(
            "biomcp get drug eflornithine regulatory --region {label}"
        )));
        assert!(discovery.next_commands.contains(&format!(
            "biomcp get drug eflornithine all --region {label}"
        )));
    }
    let who = drug_command_discovery(&discovery_drug(), &["targets".to_string()], DrugRegion::Who);
    assert!(who.next_commands.iter().all(|command| {
        !command.contains(" get drug eflornithine safety")
            && !command.contains(" get drug eflornithine shortage")
    }));
}

#[test]
fn drug_command_discovery_prioritizes_recovery_and_caps_exact_commands() {
    let mut drug = discovery_drug();
    for key in [
        "approvals",
        "safety",
        "targets",
        "indications",
        "interactions",
        "civic",
    ] {
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
    assert_eq!(discovery.recovery.len(), 6);
    assert_eq!(discovery.next_commands.len(), 10);
    assert_eq!(
        discovery.next_commands[..6],
        [
            "biomcp get drug eflornithine approvals",
            "biomcp get drug eflornithine safety --region us",
            "biomcp get drug eflornithine targets",
            "biomcp get drug eflornithine indications",
            "biomcp get drug eflornithine interactions",
            "biomcp get drug eflornithine civic",
        ]
    );
    assert!(
        !discovery
            .next_commands
            .iter()
            .any(|command| command.contains("get gene"))
    );
}

#[test]
fn drug_command_discovery_quotes_identity_and_recovery_is_rendered_once() {
    use clap::Parser;

    let mut blank = discovery_drug();
    blank.name = "   ".to_string();
    assert!(
        drug_command_discovery(&blank, &[], DrugRegion::Us)
            .next_commands
            .is_empty()
    );

    let mut drug = discovery_drug();
    drug.name = " Dose $x `rm` ; & path ".to_string();
    let discovery = drug_command_discovery(&drug, &[], DrugRegion::Us);
    let command = &discovery.next_commands[0];
    let argv = shlex::split(command).expect("shell command parses");
    let parsed = crate::cli::Cli::try_parse_from(argv).expect("command parses with CLI");
    let crate::cli::Commands::Get {
        entity: crate::cli::GetEntity::Drug(args),
    } = parsed.command
    else {
        panic!("expected get drug command");
    };
    assert_eq!(
        args.args.first().map(String::as_str),
        Some(drug.name.trim())
    );
    assert!(command.contains("\\$x") && command.contains("\\`rm\\`"));

    let mut failed = discovery_drug();
    failed.section_outcomes.complete(
        "safety",
        crate::entities::section_outcome::SectionOutcome::unavailable("fixture outage"),
    );
    let markdown = crate::render::markdown::drug_markdown_with_region(
        &failed,
        &["safety".to_string()],
        DrugRegion::Eu,
        false,
    )
    .expect("drug markdown");
    assert_eq!(
        markdown
            .matches("Retry: `biomcp get drug eflornithine safety --region eu`")
            .count(),
        1
    );
}

fn expected_drug_command(name: &str, section: &str, region: DrugRegion) -> String {
    let suffix = matches!(section, "regulatory" | "safety" | "shortage")
        .then(|| format!(" --region {}", region.as_str()))
        .unwrap_or_default();
    format!("biomcp get drug {name} {section}{suffix}")
}

fn expected_sections(name: &str, sections: &[&str], region: DrugRegion) -> Vec<DrugCommand> {
    sections
        .iter()
        .map(|section| DrugCommand {
            section: (*section).to_string(),
            command: expected_drug_command(name, section, region),
        })
        .collect()
}

fn expected_related(name: &str, review: bool, target: Option<&str>) -> Vec<String> {
    let mut related = Vec::new();
    if review {
        related.push(format!(
            "biomcp search article --drug {name} --type review --limit 5"
        ));
    }
    related.extend([
        format!("biomcp drug trials {name}"),
        format!("biomcp drug adverse-events {name}"),
        format!("biomcp search pgx -d {name}"),
    ]);
    if let Some(target) = target {
        related.push(format!("biomcp get gene {target}"));
    }
    related
}

fn expected_flattened(
    recovery: &[DrugCommand],
    sections: &[DrugCommand],
    all: Option<&str>,
    related: &[String],
) -> Vec<String> {
    recovery
        .iter()
        .chain(sections)
        .map(|entry| entry.command.clone())
        .chain(all.into_iter().map(str::to_string))
        .chain(related.iter().cloned())
        .collect()
}

fn assert_projection(
    discovery: &DrugCommandDiscovery,
    recovery: Vec<DrugCommand>,
    sections: Vec<DrugCommand>,
    all: Option<&str>,
    related: Vec<String>,
) {
    let flattened = expected_flattened(&recovery, &sections, all, &related);
    assert_eq!(discovery.recovery, recovery);
    assert_eq!(discovery.sections, sections);
    assert_eq!(discovery.all.as_deref(), all);
    assert_eq!(discovery.related, related);
    assert_eq!(discovery.next_commands, flattened);
}

#[test]
fn push_exact_capped_keeps_first_exact_bytes_before_the_ten_item_cap() {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for command in ["a", "a", "A", "b", "c", "d", "e", "f", "g", "h", "i"] {
        push_exact_capped(command.to_string(), &mut seen, &mut output);
    }
    assert_eq!(
        output,
        vec!["a", "A", "b", "c", "d", "e", "f", "g", "h", "i"]
    );
}

#[test]
fn drug_command_discovery_asserts_every_explicit_single_section_projection() {
    let cases = [
        ("approvals", vec!["label", "regulatory", "safety"]),
        ("label", vec!["approvals", "regulatory", "safety"]),
        ("regulatory", vec!["approvals", "label", "safety"]),
        ("safety", vec!["approvals", "label", "regulatory"]),
        ("shortage", vec!["approvals", "label", "regulatory"]),
        ("interactions", vec!["approvals", "label", "regulatory"]),
        ("indications", vec!["approvals", "label", "regulatory"]),
        ("targets", vec!["approvals", "label", "regulatory"]),
        ("civic", vec!["approvals", "label", "regulatory"]),
    ];
    let related = expected_related("eflornithine", true, Some("ODC1"));
    for (requested, remaining) in cases {
        let discovery =
            drug_command_discovery(&discovery_drug(), &[requested.to_string()], DrugRegion::Us);
        let sections = expected_sections("eflornithine", &remaining, DrugRegion::Us);
        assert_projection(
            &discovery,
            Vec::new(),
            sections,
            Some("biomcp get drug eflornithine all --region us"),
            related.clone(),
        );
    }
}

#[test]
fn drug_command_discovery_asserts_all_and_every_recovery_state() {
    let all_related = expected_related("eflornithine", true, Some("ODC1"));
    let all_discovery =
        drug_command_discovery(&discovery_drug(), &["all".to_string()], DrugRegion::Us);
    assert_projection(
        &all_discovery,
        Vec::new(),
        expected_sections("eflornithine", &["approvals"], DrugRegion::Us),
        None,
        all_related,
    );

    for outcome in [
        crate::entities::section_outcome::SectionOutcome::unavailable("fixture outage"),
        crate::entities::section_outcome::SectionOutcome::degraded(
            ["fixture"],
            "fixture degradation",
        ),
    ] {
        for section in [
            "approvals",
            "safety",
            "targets",
            "indications",
            "interactions",
            "civic",
        ] {
            let mut drug = discovery_drug();
            drug.section_outcomes.complete(section, outcome.clone());
            let discovery = drug_command_discovery(&drug, &[section.to_string()], DrugRegion::Eu);
            let recovery = expected_sections("eflornithine", &[section], DrugRegion::Eu);
            let remaining = match section {
                "approvals" => vec!["label", "regulatory", "safety"],
                "safety" => vec!["approvals", "label", "regulatory"],
                _ => vec!["approvals", "label", "regulatory"],
            };
            let sections = expected_sections("eflornithine", &remaining, DrugRegion::Eu);
            assert_projection(
                &discovery,
                recovery,
                sections,
                Some("biomcp get drug eflornithine all --region eu"),
                expected_related("eflornithine", true, Some("ODC1")),
            );
        }
    }
}

#[test]
fn drug_command_discovery_asserts_complete_region_projections_and_who_exclusions() {
    for (region, label) in [
        (DrugRegion::Us, "us"),
        (DrugRegion::Eu, "eu"),
        (DrugRegion::Who, "who"),
        (DrugRegion::All, "all"),
    ] {
        let discovery = drug_command_discovery(&discovery_drug(), &["targets".to_string()], region);
        let sections = expected_sections(
            "eflornithine",
            &["approvals", "label", "regulatory"],
            region,
        );
        assert_projection(
            &discovery,
            Vec::new(),
            sections,
            Some(&format!(
                "biomcp get drug eflornithine all --region {label}"
            )),
            expected_related("eflornithine", true, Some("ODC1")),
        );
        if region == DrugRegion::Who {
            assert!(discovery.next_commands.iter().all(|command| {
                !command.contains(" get drug eflornithine safety")
                    && !command.contains(" get drug eflornithine shortage")
            }));
        }
    }
}

#[test]
fn drug_command_discovery_asserts_sparse_branches_blank_identity_and_projection_deduplication() {
    let mut complete = discovery_drug();
    complete.label = Some(serde_json::from_value(serde_json::json!({})).unwrap());
    complete.approvals = Some(Vec::new());
    complete.ema_regulatory = Some(Vec::new());
    complete.indications.push("fixture indication".to_string());
    complete.targets.clear();
    let complete_discovery = drug_command_discovery(&complete, &[], DrugRegion::Us);
    assert_projection(
        &complete_discovery,
        Vec::new(),
        expected_sections(
            "eflornithine",
            &["approvals", "label", "regulatory"],
            DrugRegion::Us,
        ),
        Some("biomcp get drug eflornithine all --region us"),
        expected_related("eflornithine", false, None),
    );

    let mut sparse_blank_target = discovery_drug();
    sparse_blank_target.targets = vec!["  ".to_string()];
    let sparse_discovery = drug_command_discovery(&sparse_blank_target, &[], DrugRegion::Us);
    assert_projection(
        &sparse_discovery,
        Vec::new(),
        expected_sections(
            "eflornithine",
            &["approvals", "label", "regulatory"],
            DrugRegion::Us,
        ),
        Some("biomcp get drug eflornithine all --region us"),
        expected_related("eflornithine", true, None),
    );

    let mut blank = discovery_drug();
    blank.name = "   ".to_string();
    let blank_discovery = drug_command_discovery(&blank, &[], DrugRegion::Us);
    assert_projection(&blank_discovery, Vec::new(), Vec::new(), None, Vec::new());

    let projection = drug_command_discovery(&discovery_drug(), &[], DrugRegion::Us);
    let categorized = expected_flattened(
        &projection.recovery,
        &projection.sections,
        projection.all.as_deref(),
        &projection.related,
    );
    assert_eq!(projection.next_commands, categorized);
    assert_eq!(
        projection
            .next_commands
            .iter()
            .collect::<HashSet<_>>()
            .len(),
        projection.next_commands.len()
    );
}

#[test]
fn drug_command_discovery_asserts_complete_9_10_and_over_10_candidate_arrays() {
    let recovery_order = [
        "approvals",
        "safety",
        "targets",
        "indications",
        "interactions",
        "civic",
    ];
    let related = expected_related("eflornithine", true, Some("ODC1"));
    for (count, expected_related_count, expected_len) in [(4, 5, 9), (5, 5, 10), (6, 4, 10)] {
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
        let recovery = recovery_order
            .iter()
            .take(count)
            .map(|section| DrugCommand {
                section: (*section).to_string(),
                command: expected_drug_command("eflornithine", section, DrugRegion::Us),
            })
            .collect::<Vec<_>>();
        let related_surviving = related[..expected_related_count].to_vec();
        assert_projection(&discovery, recovery, Vec::new(), None, related_surviving);
        assert_eq!(discovery.next_commands.len(), expected_len);
    }
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
        gencc: None,
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

    let terminated = crate::entities::trial::TrialResponse::test_ctgov(
        "NCT02576665",
        "Completed trial",
        "TERMINATED",
        "melanoma",
        Some("trametinib"),
    );
    let terminated_sections = sections_trial(&terminated, &[]);
    assert_eq!(terminated_sections[0], "outcomes");
    assert_eq!(terminated_sections[1], "references");
    assert_eq!(terminated_sections[2], "arms");

    let trial_block =
        format_sections_block("trial", terminated.nct_id(), terminated_sections.clone());
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

    let recruiting = crate::entities::trial::TrialResponse::test_ctgov(
        "NCT02576665",
        "Recruiting trial",
        "Recruiting",
        "melanoma",
        Some("trametinib"),
    );
    let recruiting_sections = sections_trial(&recruiting, &[]);
    assert_eq!(recruiting_sections[0], "eligibility");
    assert_eq!(recruiting_sections[1], "contacts");
    assert_eq!(recruiting_sections[2], "locations");
}
