use super::*;

fn summary_trial(summary: Option<&str>) -> crate::entities::trial::Trial {
    crate::entities::trial::Trial {
        nct_id: "NCT00000001".to_string(),
        source: Some("ClinicalTrials.gov".to_string()),
        title: "Summary trial".to_string(),
        status: "Recruiting".to_string(),
        why_stopped: None,
        phase: None,
        study_type: None,
        age_range: None,
        conditions: vec![],
        interventions: vec![],
        intervention_details: vec![],
        sponsor: None,
        enrollment: None,
        summary: summary.map(str::to_string),
        start_date: None,
        completion_date: None,
        eligibility_text: None,
        eligibility: None,
        eligibility_provenance: None,
        contacts: None,
        locations: None,
        outcomes: None,
        arms: None,
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
        nct_id: "NCT06668103".to_string(),
        source: Some("ClinicalTrials.gov".to_string()),
        title: "Example trial".to_string(),
        status: "Recruiting".to_string(),
        why_stopped: None,
        phase: Some("Phase 2".to_string()),
        study_type: Some("Interventional".to_string()),
        age_range: Some("18 Years and older".to_string()),
        conditions: vec!["cystic fibrosis".to_string()],
        interventions: vec!["ivacaftor".to_string()],
        intervention_details: vec![crate::entities::trial::TrialIntervention {
            name: "ivacaftor".to_string(),
            intervention_type: Some("BIOLOGICAL".to_string()),
            description: None,
            other_names: Vec::new(),
        }],
        sponsor: Some("Example Sponsor".to_string()),
        enrollment: Some(42),
        summary: Some("Trial summary.".to_string()),
        start_date: Some("2025-01-01".to_string()),
        completion_date: None,
        eligibility_text: Some("Eligibility text.".to_string()),
        eligibility: None,
        eligibility_provenance: None,
        contacts: None,
        locations: Some(vec![crate::entities::trial::TrialLocation {
            facility: Some("Example Hospital".to_string()),
            city: Some("Boston".to_string()),
            state: Some("MA".to_string()),
            postal_code: None,
            country: Some("United States".to_string()),
            status: Some("Recruiting".to_string()),
            contacts: Vec::new(),
            contact_name: None,
            contact_role: None,
            contact_phone: None,
            contact_email: None,
            latitude: None,
            longitude: None,
        }]),
        outcomes: Some(crate::entities::trial::TrialOutcomes {
            primary: vec![crate::entities::trial::TrialOutcome {
                measure: "FEV1".to_string(),
                description: None,
                time_frame: None,
            }],
            secondary: Vec::new(),
        }),
        arms: Some(vec![crate::entities::trial::TrialArm {
            label: "Arm A".to_string(),
            arm_type: Some("Experimental".to_string()),
            description: Some("Description".to_string()),
            interventions: vec!["ivacaftor".to_string()],
        }]),
        references: Some(vec![crate::entities::trial::TrialReference {
            pmid: Some("22663011".to_string()),
            citation: "Example citation".to_string(),
            reference_type: Some("background".to_string()),
        }]),
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
fn trial_markdown_renders_contacts_eligibility_and_json_fields() {
    let mut trial = crate::entities::trial::Trial {
        nct_id: "NCT41300001".to_string(),
        source: Some("ClinicalTrials.gov".to_string()),
        title: "Contact trial".to_string(),
        status: "Recruiting".to_string(),
        why_stopped: None,
        phase: None,
        study_type: None,
        age_range: Some("2 Years to 18 Years".to_string()),
        conditions: vec![],
        interventions: vec![],
        intervention_details: Vec::new(),
        sponsor: None,
        enrollment: None,
        summary: None,
        start_date: None,
        completion_date: None,
        eligibility_text: Some("Key inclusion.".to_string()),
        eligibility: Some(crate::entities::trial::TrialEligibility {
            sex: Some("Female".to_string()),
            minimum_age: Some("2 Years".to_string()),
            maximum_age: Some("18 Years".to_string()),
        }),
        eligibility_provenance: Some(crate::entities::trial::TrialEligibilityProvenance {
            source_kind: "registry".to_string(),
            source: "ClinicalTrials.gov registry".to_string(),
            posted_documents_available: true,
            documents_handle: Some("biomcp --json get trial NCT41300001 documents".to_string()),
        }),
        contacts: Some(vec![crate::entities::trial::TrialContact {
            level: "central".to_string(),
            name: "Central Coordinator".to_string(),
            role: Some("CONTACT".to_string()),
            phone: Some("555-0100".to_string()),
            email: Some("central@example.test".to_string()),
            facility: None,
            city: None,
            state: None,
            country: None,
        }]),
        locations: Some(vec![crate::entities::trial::TrialLocation {
            facility: Some("Rare Disease Center".to_string()),
            city: Some("Ann Arbor".to_string()),
            state: Some("Michigan".to_string()),
            postal_code: None,
            country: Some("United States".to_string()),
            status: Some("Recruiting".to_string()),
            contacts: vec![
                crate::entities::trial::TrialSiteContact {
                    name: "Site Coordinator".to_string(),
                    role: Some("CONTACT".to_string()),
                    phone: None,
                    email: Some("site@example.test".to_string()),
                },
                crate::entities::trial::TrialSiteContact {
                    name: "Backup | Coordinator\n\u{7}".to_string(),
                    role: Some("BACK|UP".to_string()),
                    phone: Some("555\n0101".to_string()),
                    email: Some("backup|site@example.test".to_string()),
                },
            ],
            contact_name: Some("Site Coordinator".to_string()),
            contact_role: Some("CONTACT".to_string()),
            contact_phone: None,
            contact_email: Some("site@example.test".to_string()),
            latitude: None,
            longitude: None,
        }]),
        outcomes: None,
        arms: None,
        references: None,
    };

    let markdown = trial_markdown(
        &trial,
        &[
            "contacts".to_string(),
            "eligibility".to_string(),
            "locations".to_string(),
        ],
    )
    .expect("trial markdown");
    let contacts = markdown
        .split_once("## Contacts (ClinicalTrials.gov)\n")
        .expect("contacts section")
        .1
        .split_once("## Eligibility (ClinicalTrials.gov)\n")
        .expect("eligibility section after contacts")
        .0;
    let eligibility = markdown
        .split_once("## Eligibility (ClinicalTrials.gov)\n")
        .expect("eligibility section")
        .1
        .split_once("## Locations (ClinicalTrials.gov)\n")
        .expect("locations section after eligibility")
        .0;
    let locations = markdown
        .split_once("## Locations (ClinicalTrials.gov)\n")
        .expect("locations section")
        .1;

    assert!(contacts.contains(
        "### Central Contact\n- Name: Central Coordinator\n- Role: CONTACT\n- Email: central@example.test\n- Phone: 555-0100"
    ));
    assert!(!contacts.contains("site@example.test"));
    assert!(
        eligibility.contains("Sex: Female\nEligible Ages: 2 Years to 18 Years\nKey inclusion.")
    );
    assert!(eligibility.contains(
        "**Posted trial documents:** Posted trial documents are available and may contain additional eligibility detail: `biomcp --json get trial NCT41300001 documents`"
    ));
    assert!(!eligibility.contains("central@example.test"));
    assert!(!eligibility.contains("site@example.test"));
    assert!(locations.contains(
        "| Facility | City | Postal code | Country | Status | Contact |\n|---|---|---|---|---|---|"
    ));
    assert!(locations.contains(
        "| Rare Disease Center | Ann Arbor, Michigan | - | United States | Recruiting | Site Coordinator (CONTACT) site@example.test<br>Backup \\| Coordinator (BACK\\|UP) 555 0101 backup\\|site@example.test |"
    ));
    assert!(!locations.contains("central@example.test"));

    let json = serde_json::to_value(&trial).expect("trial json");
    assert_eq!(json["contacts"][0]["email"], "central@example.test");
    assert_eq!(json["eligibility"]["sex"], "Female");
    assert_eq!(json["locations"][0]["contact_email"], "site@example.test");

    trial.locations.as_mut().unwrap()[0].contacts.clear();
    let legacy_markdown = trial_markdown(&trial, &["locations".to_string()])
        .expect("legacy-compatible location markdown");
    assert!(legacy_markdown.contains(
        "| Rare Disease Center | Ann Arbor, Michigan | - | United States | Recruiting | Site Coordinator (CONTACT) site@example.test |"
    ));

    trial.eligibility_provenance = Some(crate::entities::trial::TrialEligibilityProvenance {
        source_kind: "registry".to_string(),
        source: "ClinicalTrials.gov registry".to_string(),
        posted_documents_available: false,
        documents_handle: None,
    });
    let markdown = trial_markdown(&trial, &["eligibility".to_string()]).expect("trial markdown");
    assert!(!markdown.contains("Posted trial documents"));
}
