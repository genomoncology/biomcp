use super::*;

fn eligibility_fixture() -> biodata::ClinicalTrialEligibility {
    let parse =
        |source, bound| match biodata::TemporalParser::default().parse_duration(source, bound) {
            biodata::ParseOutcome::Parsed(value) => value,
            other => panic!("age fixture did not parse: {other:?}"),
        };
    let minimum =
        biodata::ClinicalTrialAgeBound::limited(parse("2 Years", biodata::Bound::Minimum)).unwrap();
    let maximum =
        biodata::ClinicalTrialAgeBound::limited(parse("18 Years", biodata::Bound::Maximum))
            .unwrap();
    biodata::ClinicalTrialEligibility::new(
        Some("Key inclusion.".into()),
        Some(biodata::ClinicalTrialAgeRange::new(Some(minimum), Some(maximum)).unwrap()),
        Some(vec![
            biodata::ExtensibleCode::new(
                "clinicaltrials.gov",
                "FEMALE",
                Some("Female"),
                None::<String>,
                None::<String>,
            )
            .unwrap(),
        ]),
        Some(false),
        None,
    )
    .unwrap()
}

fn criterion(
    id: u64,
    description: &str,
    classification: biodata::ClinicalTrialEligibilityClassification,
) -> biodata::ClinicalTrialEligibilityCriterion {
    biodata::ClinicalTrialEligibilityCriterion::new(
        biodata::ClinicalTrialEligibilityCriterionId::new(id).unwrap(),
        description,
        classification,
    )
    .unwrap()
}

#[test]
fn eligibility_markdown_preserves_linear_transitions_and_bounds_all_content() {
    use biodata::ClinicalTrialEligibilityClassification::{Exclusion, Inclusion, Other};

    let future = biodata::ExtensibleCode::new(
        "future.registry",
        "MAYBE",
        None::<String>,
        None::<String>,
        None::<String>,
    )
    .unwrap();
    let eligibility = biodata::ClinicalTrialEligibility::new(
        Some(format!("Registry β {}", "x".repeat(12_100))),
        None,
        None,
        Some(false),
        Some(vec![
            criterion(1, "include one", Inclusion),
            criterion(2, "exclude one", Exclusion),
            criterion(3, "include two", Inclusion),
            criterion(4, "other one", Other(future)),
        ]),
    )
    .unwrap();
    let markdown = eligibility_markdown(&eligibility);
    assert!(markdown.contains("Healthy Subjects: No"));
    assert!(markdown.contains("Registry β"));
    assert!(markdown.contains("(truncated,"));
    assert!(!markdown.contains("include one"));

    let without_long_text = biodata::ClinicalTrialEligibility::new(
        None,
        None,
        None,
        eligibility.includes_healthy_subjects(),
        eligibility.criteria().map(<[_]>::to_vec),
    )
    .unwrap();
    let markdown = eligibility_markdown(&without_long_text);
    let expected = [
        "### Inclusion Criteria",
        "- include one",
        "### Exclusion Criteria",
        "- exclude one",
        "### Inclusion Criteria",
        "- include two",
        "### Other Criteria (future.registry: MAYBE)",
        "- other one",
    ];
    let mut prior = 0;
    for part in expected {
        let index = markdown[prior..].find(part).expect(part) + prior;
        prior = index + part.len();
    }
}

#[test]
fn eligibility_markdown_accepts_a_present_empty_aggregate_without_claims() {
    let empty = biodata::ClinicalTrialEligibility::new(None, None, None, None, None).unwrap();
    assert_eq!(eligibility_markdown(&empty), "");
}

#[test]
fn eligibility_markdown_makes_source_sex_codes_readable_without_changing_json() {
    let female = biodata::ExtensibleCode::new(
        "clinicaltrials.gov",
        "FEMALE",
        None::<String>,
        None::<String>,
        None::<String>,
    )
    .unwrap();
    let eligibility =
        biodata::ClinicalTrialEligibility::new(None, None, Some(vec![female]), None, None).unwrap();

    assert_eq!(eligibility_markdown(&eligibility), "Sex: Female");
}

fn summary_trial(summary: Option<&str>) -> crate::entities::trial::Trial {
    crate::entities::trial::Trial {
        identities: Vec::new(),
        nct_id: "NCT00000001".to_string(),
        source: Some("ClinicalTrials.gov".to_string()),
        title: "Summary trial".to_string(),
        official_title: None,
        status: "Recruiting".to_string(),
        why_stopped: None,
        phase: None,
        phases: Vec::new(),
        study_type: None,
        conditions: vec![],
        design: crate::entities::trial::TrialDesign::default(),
        sponsor: None,
        enrollment: None,
        summary: summary.map(str::to_string),
        start_date: None,
        completion_date: None,
        eligibility: None,
        eligibility_provenance: None,
        site_directory: None,
        site_offset: 0,
        site_limit: None,
        outcomes: None,
        references: None,
    }
}

#[test]
fn bounded_trial_summary_keeps_supported_mid_sentence_abbreviations() {
    let cases = [
        (
            "pts.",
            "This study enrolls 40 pts. with relapsed disease and compares two regimens.",
        ),
        (
            "vs.",
            "This study compares treatment vs. placebo in relapsed disease.",
        ),
        (
            "approx.",
            "This study enrolls approx. 40 participants with relapsed disease.",
        ),
        (
            "e.g.",
            "This study includes tumors, e.g. relapsed melanoma, in two regimens.",
        ),
        (
            "i.v.",
            "This study compares i.v. therapy with an oral regimen.",
        ),
        (
            "Dr.",
            "This study is led by Dr. Smith and compares two regimens.",
        ),
    ];

    for (abbreviation, second_sentence) in cases {
        let summary =
            format!("Background is established. {second_sentence} The endpoint is survival.");
        let expected = format!(
            "Background is established. {}...",
            &second_sentence[..second_sentence.len() - 1]
        );
        let bounded = bounded_trial_summary(&summary);

        assert_eq!(bounded, expected, "abbreviation: {abbreviation}");
        assert!(!bounded.contains("The endpoint is survival"));
        assert!(bounded.ends_with("..."));
        assert!(!bounded.ends_with("...."));
        assert_eq!(bounded.matches("...").count(), 1);
    }
}

#[test]
fn bounded_trial_summary_marks_sentence_omission_but_not_complete_input() {
    assert_eq!(
        bounded_trial_summary("  Sentence one. Sentence two. Sentence three.  "),
        "Sentence one. Sentence two..."
    );
    assert_eq!(
        bounded_trial_summary("  Sentence one. Sentence two.  "),
        "Sentence one. Sentence two."
    );
    assert_eq!(
        bounded_trial_summary("  One sentence only.  "),
        "One sentence only."
    );
}

#[test]
fn bounded_trial_summary_is_utf8_safe_when_both_limits_omit_content() {
    let summary = format!("{}. Second sentence. Third sentence.", "€".repeat(167));
    let bounded = bounded_trial_summary(&summary);

    assert!(bounded.is_char_boundary(bounded.len()));
    assert!(bounded.ends_with("..."));
    assert!(!bounded.ends_with("...."));
    assert!(bounded.strip_suffix("...").expect("marker").len() <= 500);
    assert_eq!(bounded, format!("{}...", "€".repeat(166)));
}

#[test]
fn bounded_trial_summary_distinguishes_sentence_final_and_suffix_collisions() {
    assert_eq!(
        bounded_trial_summary("Two attempts. Participants recovered. Follow-up continued."),
        "Two attempts. Participants recovered..."
    );
    assert_eq!(
        bounded_trial_summary("Route was i.v. Participants recovered. Follow-up continued."),
        "Route was i.v. Participants recovered..."
    );
    assert_eq!(
        bounded_trial_summary(
            "Background is established. The study enrolls 40 PTS. with disease. Tail omitted."
        ),
        "Background is established. The study enrolls 40 PTS. with disease..."
    );
}

#[test]
fn trial_markdown_keeps_the_post_abbreviation_clause_and_json_stays_full() {
    let full = "Background is established. This study enrolls 40 pts. with relapsed disease and compares two regimens. The endpoint is survival.";
    let trial = summary_trial(Some(full));
    let markdown = trial_markdown(&trial, &[]).expect("trial markdown");
    let rendered_summary = markdown
        .split_once("## Summary (ClinicalTrials.gov)\n\n")
        .expect("summary section")
        .1
        .split_once("\nMore:\n")
        .expect("end of summary section")
        .0;
    assert_eq!(
        rendered_summary,
        "Background is established. This study enrolls 40 pts. with relapsed disease and compares two regimens..."
    );
    assert!(!markdown.contains("The endpoint is survival."));
    assert_eq!(
        serde_json::to_value(&trial).expect("trial JSON")["summary"],
        full
    );
}

#[test]
fn response_markdown_explains_selected_section_states() {
    use crate::entities::trial::{TrialResponse, TrialSectionState, TrialSectionStates};

    let response = TrialResponse {
        trial: summary_trial(None),
        section_states: TrialSectionStates {
            arms: TrialSectionState::Present,
            eligibility: TrialSectionState::Absent,
            outcomes: TrialSectionState::NotRequested,
            references: TrialSectionState::Unavailable,
            contacts: TrialSectionState::NotRequested,
            locations: TrialSectionState::NotRequested,
        },
    };
    let markdown = trial_response_markdown(&response, &["all".to_owned()]).unwrap();
    assert!(markdown.contains("No arms found."));
    assert!(markdown.contains("The provider omitted eligibility."));
    assert!(markdown.contains("The selected provider does not support references"));

    let mut omitted = response;
    omitted.section_states.references = TrialSectionState::NotRequested;
    let markdown = trial_response_markdown(&omitted, &["references".to_owned()]).unwrap();
    assert!(!markdown.contains("## References"));
}

#[test]
fn planned_outcome_markdown_preserves_groups_order_text_and_states() {
    use crate::entities::trial::{TrialResponse, TrialSectionState, TrialSectionStates};

    let planned = |classification: &str, measure: &str, description: &str, time_frame: &str| {
        biodata::ClinicalTrialPlannedOutcome::new(
            measure,
            Some(description),
            Some(time_frame),
            biodata::ExtensibleCode::new(
                "clinicaltrials.gov",
                classification,
                None::<String>,
                None::<String>,
                None::<String>,
            )
            .unwrap(),
        )
        .unwrap()
    };
    let mut trial = summary_trial(None);
    trial.outcomes = Some(vec![
        planned(
            "primaryOutcomes",
            "Primary measure α",
            "Complete primary description.",
            "From baseline through week 10",
        ),
        planned(
            "primaryOutcomes",
            "Primary measure β",
            "Second complete primary description.",
            "At week 12",
        ),
        planned(
            "secondaryOutcomes",
            "Secondary measure",
            "Complete secondary description.",
            "During follow-up",
        ),
        planned(
            "otherOutcomes",
            "Other measure",
            "Complete other description.",
            "At end of treatment",
        ),
    ]);
    let states = TrialSectionStates {
        arms: TrialSectionState::NotRequested,
        eligibility: TrialSectionState::NotRequested,
        outcomes: TrialSectionState::Present,
        references: TrialSectionState::NotRequested,
        contacts: TrialSectionState::NotRequested,
        locations: TrialSectionState::NotRequested,
    };
    let response = TrialResponse {
        trial,
        section_states: states.clone(),
    };
    let markdown = trial_response_markdown(&response, &["outcomes".to_owned()]).unwrap();
    for text in [
        "Complete primary description.",
        "From baseline through week 10",
        "Second complete primary description.",
        "At week 12",
        "Complete secondary description.",
        "During follow-up",
        "Complete other description.",
        "At end of treatment",
    ] {
        assert!(markdown.contains(text), "missing complete text: {text}");
    }
    let positions = [
        "### Primary",
        "Primary measure α",
        "Primary measure β",
        "### Secondary",
        "Secondary measure",
        "### Other",
        "Other measure",
    ]
    .map(|text| markdown.find(text).unwrap());
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));

    let mut empty = summary_trial(None);
    empty.outcomes = Some(Vec::new());
    let markdown = trial_response_markdown(
        &TrialResponse {
            trial: empty,
            section_states: states.clone(),
        },
        &["outcomes".to_owned()],
    )
    .unwrap();
    assert!(markdown.contains("The provider returned no planned outcomes."));

    for (state, expected) in [
        (
            TrialSectionState::Absent,
            Some("The provider omitted planned outcomes."),
        ),
        (
            TrialSectionState::Unavailable,
            Some("The selected provider does not support planned outcomes."),
        ),
        (TrialSectionState::NotRequested, None),
    ] {
        let mut section_states = states.clone();
        section_states.outcomes = state;
        let markdown = trial_response_markdown(
            &TrialResponse {
                trial: summary_trial(None),
                section_states,
            },
            &["outcomes".to_owned()],
        )
        .unwrap();
        if let Some(expected) = expected {
            assert!(markdown.contains(expected));
        } else {
            assert!(!markdown.contains("## Outcomes"));
        }
    }
}

#[test]
fn trial_search_markdown_with_footer_shows_scoped_zero_result_nickname_hint() {
    let markdown = trial_search_markdown_with_footer(
        "condition=CodeBreaK 300",
        &[],
        Some(0),
        "",
        true,
        Some("CodeBreaK 300"),
    )
    .expect("markdown");

    assert!(markdown.contains("ClinicalTrials.gov does not index trial nicknames."));
    assert!(markdown.contains("biomcp search trial -i \"<drug>\" -c \"<condition>\""));
    assert!(markdown.contains("biomcp search article \"CodeBreaK 300\" to find the NCT ID"));
}

#[test]
fn trial_search_markdown_with_footer_omits_zero_result_nickname_hint_without_flag() {
    let markdown =
        trial_search_markdown_with_footer("condition=melanoma", &[], Some(0), "", false, None)
            .expect("markdown");

    assert!(!markdown.contains("ClinicalTrials.gov does not index trial nicknames."));
}

#[test]
fn trial_search_markdown_with_footer_shows_filtered_zero_result_broadening_hints() {
    let hints = vec![
        "loosen or drop `--mutation`; it is an exact free-text boolean search".to_string(),
        "widen `--distance` or remove the geo filter".to_string(),
        "relax `--status` to include non-recruiting or not-yet-recruiting trials".to_string(),
        "try `--biomarker <gene>`".to_string(),
    ];
    let markdown = trial_search_markdown_with_footer_and_hints(
        "condition=melanoma, mutation=BRAF V600E, status=recruiting, distance=100",
        &[],
        Some(0),
        "",
        false,
        None,
        &hints,
    )
    .expect("markdown");

    assert!(markdown.contains("Try broadening the filtered search:"));
    assert!(markdown.contains("loosen or drop `--mutation`"));
    assert!(markdown.contains("exact free-text boolean search"));
    assert!(markdown.contains("widen `--distance`"));
    assert!(markdown.contains("relax `--status`"));
    assert!(markdown.contains("try `--biomarker <gene>`"));
}

#[test]
fn trial_search_markdown_shows_matched_intervention_column_when_present() {
    let markdown = trial_search_markdown(
        "intervention=daraxonrasib",
        &[crate::entities::trial::TrialSearchResult {
            nct_id: "NCT00000001".to_string(),
            title: "Example daraxonrasib trial".to_string(),
            status: "Recruiting".to_string(),
            phase: Some("Phase 1".to_string()),
            conditions: vec!["pancreatic cancer".to_string()],
            sponsor: Some("Example Sponsor".to_string()),
            matched_intervention_label: Some("RMC-6236".to_string()),
        }],
        Some(1),
    )
    .expect("markdown");

    assert!(markdown.contains("Matched Intervention"));
    assert!(markdown.contains("RMC-6236"));
}

#[test]
fn trial_search_markdown_omits_matched_intervention_column_without_labels() {
    let markdown = trial_search_markdown(
        "intervention=daraxonrasib",
        &[crate::entities::trial::TrialSearchResult {
            nct_id: "NCT00000001".to_string(),
            title: "Example daraxonrasib trial".to_string(),
            status: "Recruiting".to_string(),
            phase: Some("Phase 1".to_string()),
            conditions: vec!["pancreatic cancer".to_string()],
            sponsor: Some("Example Sponsor".to_string()),
            matched_intervention_label: None,
        }],
        Some(1),
    )
    .expect("markdown");

    assert!(!markdown.contains("Matched Intervention"));
}

#[test]
fn trial_markdown_includes_source_labeled_sections() {
    let trial = crate::entities::trial::Trial {
        identities: Vec::new(),
        nct_id: "NCT06668103".to_string(),
        source: Some("ClinicalTrials.gov".to_string()),
        title: "Example trial".to_string(),
        official_title: None,
        status: "Recruiting".to_string(),
        why_stopped: None,
        phase: Some("Phase 2".to_string()),
        phases: Vec::new(),
        study_type: Some("Interventional".to_string()),
        conditions: vec!["cystic fibrosis".to_string()],
        design: {
            let intervention_id = biodata::ClinicalTrialInterventionId::new(1).unwrap();
            let arm_id = biodata::ClinicalTrialArmId::new(1).unwrap();
            let code = |value: &str| {
                biodata::ExtensibleCode::new(
                    "test",
                    value,
                    None::<String>,
                    None::<String>,
                    None::<String>,
                )
                .unwrap()
            };
            crate::entities::trial::TrialDesign::new(
                vec![
                    biodata::ClinicalTrialIntervention::new(
                        intervention_id,
                        "ivacaftor",
                        Some(code("BIOLOGICAL")),
                        None,
                        None,
                    )
                    .unwrap(),
                ],
                Some(vec![
                    biodata::ClinicalTrialArm::new(
                        arm_id,
                        "Arm A",
                        Some(code("Experimental")),
                        Some("Description".to_string()),
                    )
                    .unwrap(),
                ]),
                Some(vec![biodata::ClinicalTrialArmInterventionAssignment::new(
                    arm_id,
                    intervention_id,
                )]),
            )
            .unwrap()
        },
        sponsor: Some("Example Sponsor".to_string()),
        enrollment: Some(42),
        summary: Some("Trial summary.".to_string()),
        start_date: Some("2025-01-01".to_string()),
        completion_date: None,
        eligibility: Some(eligibility_fixture()),
        eligibility_provenance: None,
        site_directory: Some(biodata::ClinicalTrialSiteDirectory::new(
            None,
            Some(vec![
                biodata::ClinicalTrialSite::new(biodata::ClinicalTrialSiteFields {
                    facility: Some("Example Hospital".to_string()),
                    status: None,
                    city: Some("Boston".to_string()),
                    state: Some("MA".to_string()),
                    postal_code: None,
                    country: Some("United States".to_string()),
                    coordinates: None,
                    contacts: None,
                })
                .unwrap(),
            ]),
        )),
        site_offset: 0,
        site_limit: None,
        outcomes: Some(vec![
            biodata::ClinicalTrialPlannedOutcome::new(
                "FEV1",
                None::<String>,
                None::<String>,
                biodata::ExtensibleCode::new(
                    "clinicaltrials.gov",
                    "primaryOutcomes",
                    None::<String>,
                    None::<String>,
                    None::<String>,
                )
                .unwrap(),
            )
            .unwrap(),
        ]),
        references: Some(vec![
            biodata::ClinicalTrialReference::new(
                Some("22663011".to_string()),
                Some("Example citation".to_string()),
                Some(
                    biodata::ExtensibleCode::new(
                        "clinicaltrials.gov",
                        "background",
                        None::<String>,
                        None::<String>,
                        None::<String>,
                    )
                    .expect("valid reference type"),
                ),
            )
            .expect("valid reference"),
        ]),
    };

    let markdown = trial_markdown(&trial, &["all".to_string()]).expect("trial");
    assert!(markdown.contains("Source: ClinicalTrials.gov"));
    assert!(markdown.contains("## Conditions (ClinicalTrials.gov)"));
    assert!(markdown.contains("## Interventions (ClinicalTrials.gov)"));
    assert!(markdown.contains("## Summary (ClinicalTrials.gov)"));
    assert!(markdown.contains("## Eligibility (ClinicalTrials.gov)"));
    assert!(markdown.contains("## Locations (ClinicalTrials.gov)"));
    assert!(markdown.contains("## Outcomes (ClinicalTrials.gov)"));
    assert!(markdown.contains("## Arms (ClinicalTrials.gov)"));
    assert!(markdown.contains("## References (ClinicalTrials.gov)"));
    for provider_type in ["BIOLOGICAL", "Experimental", "background"] {
        assert!(
            markdown.contains(provider_type),
            "missing provider type {provider_type} from Markdown"
        );
    }
    assert!(!markdown.contains("Posted trial documents"));
}

#[test]
fn trial_markdown_uses_each_safe_reference_fallback() {
    let mut trial = summary_trial(None);
    trial.references = Some(vec![
        biodata::ClinicalTrialReference::new(
            Some(" 12345 ".to_string()),
            Some(" Citation with identifier ".to_string()),
            Some(
                biodata::ExtensibleCode::new(
                    "example.org",
                    "CODE",
                    Some(" Preferred display ".to_string()),
                    None::<String>,
                    Some("Recognized meaning".to_string()),
                )
                .expect("shared source type"),
            ),
        )
        .expect("shared reference"),
        biodata::ClinicalTrialReference::new(Some("67890".to_string()), None, None)
            .expect("PMID-only reference"),
        biodata::ClinicalTrialReference::new(
            Some("24680".to_string()),
            None,
            Some(
                biodata::ExtensibleCode::new(
                    "example.org",
                    "HIDDEN",
                    Some("Should stay hidden".to_string()),
                    None::<String>,
                    None::<String>,
                )
                .expect("PMID source type"),
            ),
        )
        .expect("PMID and source-type reference"),
        biodata::ClinicalTrialReference::new(
            None,
            None,
            Some(
                biodata::ExtensibleCode::new(
                    "example.org",
                    "CODE-TWO",
                    None::<String>,
                    None::<String>,
                    Some(" Recognized only ".to_string()),
                )
                .expect("recognized source type"),
            ),
        )
        .expect("source-only reference"),
        biodata::ClinicalTrialReference::new(
            None,
            None,
            Some(
                biodata::ExtensibleCode::new(
                    "example.org",
                    " Code only ",
                    None::<String>,
                    None::<String>,
                    None::<String>,
                )
                .expect("code source type"),
            ),
        )
        .expect("source-only code reference"),
        biodata::ClinicalTrialReference::new(None, None, None).expect("all-null reference"),
        biodata::ClinicalTrialReference::new(
            Some(" \t ".to_string()),
            Some(" \t ".to_string()),
            Some(
                biodata::ExtensibleCode::new(
                    "example.org",
                    " \t ",
                    Some(" \t ".to_string()),
                    None::<String>,
                    Some(" \t ".to_string()),
                )
                .expect("whitespace source type"),
            ),
        )
        .expect("whitespace reference"),
    ]);

    let markdown = trial_markdown(&trial, &["references".to_string()]).expect("references");
    for expected in [
        "[PMID: 12345] Citation with identifier *(Preferred display)*",
        "[PMID: 67890]",
        "[PMID: 24680]",
        "Recognized only",
        "Code only",
    ] {
        assert!(markdown.contains(expected), "missing {expected}");
    }
    assert_eq!(
        markdown.matches("Reference details unavailable.").count(),
        2
    );
    assert!(!markdown.contains("Recognized meaning"));
    assert!(!markdown.contains("Should stay hidden"));
}

#[test]
fn trial_markdown_renders_coordinates_and_sanitizes_unnamed_contacts() {
    let role = biodata::ExtensibleCode::new(
        "clinicaltrials.gov",
        "INVESTIGATOR\nROLE",
        None::<String>,
        None::<String>,
        None::<String>,
    )
    .unwrap();
    let contact = biodata::ClinicalTrialContact::new(
        None,
        Some(role),
        Some("555\n0100".to_owned()),
        Some("4\u{0007}2".to_owned()),
        Some("site\n@example.test".to_owned()),
    )
    .unwrap();
    let site = biodata::ClinicalTrialSite::new(biodata::ClinicalTrialSiteFields {
        facility: Some("Example\nFacility".to_owned()),
        status: None,
        city: Some("Example\nCity".to_owned()),
        state: Some("E\u{0007}X".to_owned()),
        postal_code: Some("00\n000".to_owned()),
        country: Some("Example\nCountry".to_owned()),
        coordinates: Some(biodata::ClinicalTrialGeographicPoint::new(10.5, -20.25).unwrap()),
        contacts: Some(vec![contact]),
    })
    .unwrap();
    let mut trial = summary_trial(None);
    trial.set_site_directory(Some(biodata::ClinicalTrialSiteDirectory::new(
        None,
        Some(vec![site]),
    )));

    let markdown = trial_markdown(&trial, &["contacts".into(), "locations".into()]).unwrap();
    assert!(markdown.contains("| Latitude | Longitude |"));
    assert!(markdown.contains("| 10.5 | -20.25 |"));
    assert!(markdown.contains("Example Facility"));
    assert!(markdown.contains("Example City, E X"));
    assert!(markdown.contains("INVESTIGATOR ROLE"));
    assert!(markdown.contains("555 0100"));
    assert!(markdown.contains("ext. 4 2"));
    assert!(markdown.contains("site @example.test"));
    assert!(!markdown.contains("- Name:"));
    for forbidden in ['\n', '\r', '\u{0007}'] {
        assert!(
            !markdown
                .lines()
                .any(|line| line.contains(forbidden) && line.starts_with('|'))
        );
    }
}

#[test]
fn arm_rendering_follows_assignment_ids_when_names_do_not_change() {
    use biodata::{
        ClinicalTrialArm, ClinicalTrialArmId, ClinicalTrialArmInterventionAssignment,
        ClinicalTrialIntervention, ClinicalTrialInterventionId,
    };

    let interventions = vec![
        ClinicalTrialIntervention::new(
            ClinicalTrialInterventionId::new(1).unwrap(),
            "alpha",
            None,
            None,
            None,
        )
        .unwrap(),
        ClinicalTrialIntervention::new(
            ClinicalTrialInterventionId::new(2).unwrap(),
            "beta",
            None,
            None,
            None,
        )
        .unwrap(),
    ];
    let arms = vec![
        ClinicalTrialArm::new(ClinicalTrialArmId::new(1).unwrap(), "Arm A", None, None).unwrap(),
        ClinicalTrialArm::new(ClinicalTrialArmId::new(2).unwrap(), "Arm B", None, None).unwrap(),
    ];
    let design = |pairs: [(u64, u64); 2]| {
        crate::entities::trial::TrialDesign::new(
            interventions.clone(),
            Some(arms.clone()),
            Some(
                pairs
                    .map(|(arm, intervention)| {
                        ClinicalTrialArmInterventionAssignment::new(
                            ClinicalTrialArmId::new(arm).unwrap(),
                            ClinicalTrialInterventionId::new(intervention).unwrap(),
                        )
                    })
                    .to_vec(),
            ),
        )
        .unwrap()
    };
    let mut trial = summary_trial(None);
    trial.design = design([(1, 1), (2, 2)]);
    let original_names = trial
        .design
        .interventions()
        .iter()
        .map(|value| value.name().to_string())
        .collect::<Vec<_>>();
    let before = arm_views(&trial);
    assert_eq!(before[0].interventions, ["alpha"]);
    assert_eq!(before[1].interventions, ["beta"]);

    trial.design = design([(1, 2), (2, 1)]);
    assert_eq!(
        trial
            .design
            .interventions()
            .iter()
            .map(|value| value.name().to_string())
            .collect::<Vec<_>>(),
        original_names
    );
    let after = arm_views(&trial);
    assert_eq!(after[0].interventions, ["beta"]);
    assert_eq!(after[1].interventions, ["alpha"]);
}
