use super::*;

const FULL_SECTION_SUMMARY: &str = "Background is established. This study enrolls 40 pts. with relapsed disease and compares two regimens. The endpoint is survival.";

fn preselected_trial_with_25_locations_from(source: &str) -> TrialResponse {
    let locations = (1..=25)
        .map(|number| {
            serde_json::json!({
                "facility": format!("Fixture Site {number:02}"),
                "city": format!("Fixture City {number:02}"),
                "state": "Michigan",
                "country": "United States",
                "status": "RECRUITING",
                "contacts": [{"name": format!("Person {number:02}"), "role": "CONTACT"}]
            })
        })
        .collect::<Vec<_>>();
    let input = serde_json::json!({
        "protocolSection": {
            "identificationModule": {
                "nctId": "NCT41300001",
                "briefTitle": "Location renderer fixture"
            },
            "statusModule": {"overallStatus": "RECRUITING"},
            "sponsorCollaboratorsModule": {
                "leadSponsor": {"name": "Fixture Sponsor"}
            },
            "conditionsModule": {"conditions": ["Renderer Fixture"]},
            "designModule": {"studyType": "INTERVENTIONAL"},
            "contactsLocationsModule": {
                "centralContacts": [{"name": "Central Person", "role": "CONTACT"}],
                "locations": locations
            }
        }
    })
    .to_string();
    let plan = biodata::ClinicalTrialsGovApiV2DetailPlan::new("NCT41300001", false)
        .expect("test plan")
        .with_locations()
        .with_contacts();
    let response = biodata::ClinicalTrialsGovApiV2Response::parse(
        &plan,
        input.as_bytes(),
        &Default::default(),
    )
    .expect("test response");
    let mut trial = TrialResponse::new(
        response.into_projection().expect("test projection"),
        source,
        None,
        crate::entities::trial::TrialSectionStates {
            arms: crate::entities::trial::TrialSectionState::NotRequested,
            eligibility: crate::entities::trial::TrialSectionState::NotRequested,
            outcomes: crate::entities::trial::TrialSectionState::NotRequested,
            references: crate::entities::trial::TrialSectionState::NotRequested,
            contacts: crate::entities::trial::TrialSectionState::Present,
            locations: crate::entities::trial::TrialSectionState::Present,
        },
    );
    trial.set_site_page(0, 25);
    trial
}

fn preselected_trial_with_25_locations() -> TrialResponse {
    preselected_trial_with_25_locations_from("ClinicalTrials.gov")
}

fn rendered_location_row_count(markdown: &str) -> usize {
    markdown
        .lines()
        .filter(|line| line.starts_with("| Fixture Site "))
        .count()
}

#[test]
fn explicit_preselected_trial_page_renders_all_25_without_cap_disclosure() {
    let markdown = trial_response_page_markdown(
        &preselected_trial_with_25_locations(),
        &["locations".into()],
    )
    .expect("explicit page markdown");

    assert_eq!(rendered_location_row_count(&markdown), 25);
    assert!(markdown.contains("| Fixture Site 25 |"));
    assert!(!markdown.contains("display cap"));
}

#[test]
fn ordinary_trial_response_rendering_keeps_20_location_cap() {
    let markdown = trial_response_markdown(
        &preselected_trial_with_25_locations(),
        &["locations".into()],
    )
    .expect("ordinary trial markdown");

    assert_eq!(rendered_location_row_count(&markdown), 20);
    assert!(markdown.contains("Locations: showing 20 of 25 (display cap 20)."));
    assert!(!markdown.contains("| Fixture Site 21 |"));
}

#[test]
fn capped_trial_locations_keep_contact_rows_aligned() {
    let markdown = trial_response_markdown(
        &preselected_trial_with_25_locations(),
        &["contacts".into(), "locations".into()],
    )
    .expect("contacts and locations markdown");
    assert_eq!(rendered_location_row_count(&markdown), 20);
    assert!(markdown.contains("Central Person"));
    assert!(markdown.contains("Person 20"));
    assert!(!markdown.contains("Person 21"));
    assert!(markdown.contains(
        "Next: `biomcp get trial NCT41300001 --offset 20 --limit 20 contacts locations`"
    ));
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
fn bounded_trial_summary_retains_clause_after_abbreviation() {
    let text = "Background is established. This study enrolls 40 pts. with relapsed disease and compares two regimens. The endpoint is survival.";
    assert_eq!(
        bounded_trial_summary(text),
        "Background is established. This study enrolls 40 pts. with relapsed disease and compares two regimens..."
    );
}

#[test]
fn trial_search_markdown_keeps_partial_detail_note() {
    let note = "The count may be too high: we could not check 1 of the kept trials (NCT00000001), because the detail fetch failed, the eligibility text was missing, or the trial had no NCT ID. Eligibility and facility filters may not have applied to those trials.";
    let markdown = trial_search_markdown_with_footer_and_hints(
        "condition=melanoma",
        &[crate::entities::trial::TrialSearchHit::test(
            "NCT00000001",
            "RECRUITING",
        )],
        Some(1),
        "",
        false,
        None,
        &[],
        Some(note),
    )
    .expect("markdown");
    assert!(markdown.contains(&format!("Note: {note}")));
    assert!(markdown.contains("|NCT00000001|"));
}

#[test]
fn trial_search_keeps_the_matched_intervention_column_only_for_labeled_hits() {
    let mut hit = crate::entities::trial::TrialSearchHit::test("NCT00000001", "RECRUITING");
    let without_label = trial_search_markdown("intervention=fixture", &[hit.clone()], Some(1))
        .expect("unlabeled search markdown");
    assert!(!without_label.contains("Matched Intervention"));

    hit.matched_intervention_label = Some("RMC-6236".into());
    let with_label = trial_search_markdown("intervention=fixture", &[hit], Some(1))
        .expect("labeled search markdown");
    assert!(with_label.contains("Matched Intervention"));
    assert!(with_label.contains("RMC-6236"));
}

#[test]
fn trial_search_keeps_scoped_zero_result_guidance() {
    let nickname = trial_search_markdown_with_footer(
        "condition=CodeBreaK 300",
        &[],
        Some(0),
        "",
        true,
        Some("CodeBreaK 300"),
    )
    .expect("nickname guidance");
    assert!(nickname.contains("ClinicalTrials.gov does not index trial nicknames."));
    assert!(nickname.contains("biomcp search trial -i \"<drug>\" -c \"<condition>\""));
    assert!(nickname.contains("biomcp search article \"CodeBreaK 300\" to find the NCT ID"));

    let ordinary =
        trial_search_markdown_with_footer("condition=melanoma", &[], Some(0), "", false, None)
            .expect("ordinary empty search");
    assert!(!ordinary.contains("ClinicalTrials.gov does not index trial nicknames."));

    let hints = vec![
        "loosen or drop `--mutation`; it is an exact free-text boolean search".into(),
        "widen `--distance` or remove the geo filter".into(),
        "relax `--status` to include non-recruiting or not-yet-recruiting trials".into(),
        "try `--biomarker <gene>`".into(),
    ];
    let filtered = trial_search_markdown_with_footer_and_hints(
        "condition=melanoma, mutation=BRAF V600E",
        &[],
        Some(0),
        "",
        false,
        None,
        &hints,
        None,
    )
    .expect("filtered empty search");
    for guidance in [
        "Try broadening the filtered search:",
        "loosen or drop `--mutation`",
        "exact free-text boolean search",
        "widen `--distance`",
        "relax `--status`",
        "try `--biomarker <gene>`",
    ] {
        assert!(filtered.contains(guidance), "missing {guidance}");
    }
}

fn sectioned_trial_fixture(
    provenance: Option<crate::entities::trial::TrialEligibilityProvenance>,
) -> TrialResponse {
    let input = serde_json::json!({
        "protocolSection": {
            "identificationModule": {"nctId": "NCT41300002", "briefTitle": "Section renderer fixture"},
            "statusModule": {"overallStatus": "RECRUITING"},
            "sponsorCollaboratorsModule": {"leadSponsor": {"name": "Fixture Sponsor"}},
            "conditionsModule": {"conditions": ["Fixture Condition"]},
            "designModule": {"studyType": "INTERVENTIONAL"},
            "descriptionModule": {"briefSummary": FULL_SECTION_SUMMARY},
            "armsInterventionsModule": {
                "armGroups": [
                    {"label": "Fixture Arm", "type": "EXPERIMENTAL", "interventionNames": ["DRUG: Fixture Drug"]},
                    {"label": "Comparison Arm", "type": "ACTIVE_COMPARATOR", "interventionNames": ["DRUG: Comparison Drug"]}
                ],
                "interventions": [
                    {"name": "Fixture Drug", "type": "DRUG", "armGroupLabels": ["Fixture Arm"]},
                    {"name": "Comparison Drug", "type": "DRUG", "armGroupLabels": ["Comparison Arm"]}
                ]
            },
            "eligibilityModule": {"eligibilityCriteria": "Fixture criterion", "sex": "ALL"},
            "outcomesModule": {"primaryOutcomes": [{"measure": "Fixture Outcome"}]},
            "contactsLocationsModule": {
                "centralContacts": [{"name": "Central Coordinator", "role": "CONTACT", "email": "central@example.test"}],
                "locations": [{"facility": "Fixture Site", "country": "United States", "contacts": [
                    {"name": "Site Coordinator", "role": "CONTACT", "email": "site@example.test"}
                ]}]
            },
            "referencesModule": {"references": [
                {"pmid": "12345", "citation": "Fixture citation", "type": "BACKGROUND"},
                {"pmid": "67890"},
                {},
                {"pmid": "  ", "citation": "\t", "type": "  "}
            ]}
        }
    }).to_string();
    let plan = biodata::ClinicalTrialsGovApiV2DetailPlan::new("NCT41300002", true)
        .expect("section plan")
        .with_arms()
        .with_eligibility()
        .with_outcomes()
        .with_contacts()
        .with_locations();
    let response = biodata::ClinicalTrialsGovApiV2Response::parse(
        &plan,
        input.as_bytes(),
        &Default::default(),
    )
    .expect("section response");
    TrialResponse::new(
        response.into_projection().expect("section projection"),
        "ClinicalTrials.gov",
        provenance,
        crate::entities::trial::TrialSectionStates {
            arms: crate::entities::trial::TrialSectionState::Present,
            eligibility: crate::entities::trial::TrialSectionState::Present,
            outcomes: crate::entities::trial::TrialSectionState::Present,
            references: crate::entities::trial::TrialSectionState::Present,
            contacts: crate::entities::trial::TrialSectionState::Present,
            locations: crate::entities::trial::TrialSectionState::Present,
        },
    )
}

#[test]
fn trial_markdown_keeps_source_labels_and_safe_reference_fallbacks() {
    let trial = sectioned_trial_fixture(None);
    let markdown = trial_markdown(&trial, &["all".into()]).expect("section markdown");
    assert!(markdown.contains("Source: ClinicalTrials.gov"));
    for section in [
        "Conditions",
        "Interventions",
        "Summary",
        "Contacts",
        "Eligibility",
        "Locations",
        "Outcomes",
        "Arms",
        "References",
    ] {
        assert!(
            markdown.contains(&format!("## {section} (ClinicalTrials.gov)")),
            "missing {section}"
        );
    }
    for source_type in ["DRUG", "EXPERIMENTAL", "BACKGROUND"] {
        assert!(markdown.contains(source_type), "missing {source_type}");
    }
    assert!(markdown.contains("| Fixture Arm | EXPERIMENTAL | Fixture Drug |"));
    assert!(markdown.contains("| Comparison Arm | ACTIVE_COMPARATOR | Comparison Drug |"));
    assert!(!markdown.contains("| Fixture Arm | EXPERIMENTAL | Comparison Drug |"));
    assert!(markdown.contains("[PMID: 12345] Fixture citation"));
    assert!(markdown.contains("[PMID: 67890]"));
    assert_eq!(
        markdown.matches("Reference details unavailable.").count(),
        2
    );
    assert!(!markdown.contains("Posted trial documents"));
    assert!(markdown.contains("Background is established. This study enrolls 40 pts. with relapsed disease and compares two regimens..."));
    assert!(!markdown.contains("The endpoint is survival."));
    assert!(markdown.contains("### Central Contact\n- Name: Central Coordinator"));
    assert!(markdown.contains("Site Coordinator (CONTACT) site@example.test"));
    let locations = markdown
        .split_once("## Locations (ClinicalTrials.gov)")
        .expect("locations section")
        .1;
    assert!(!locations.contains("central@example.test"));
    assert_eq!(
        serde_json::to_value(&trial).expect("trial JSON")["summary"],
        FULL_SECTION_SUMMARY
    );
    let json = serde_json::to_value(&trial).expect("trial JSON");
    assert_eq!(json["contacts"][0]["email"], "central@example.test");
    assert_eq!(
        json["locations"][0]["contacts"][0]["email"],
        "site@example.test"
    );
    assert_eq!(json["eligibility"]["sexes"][0]["code"], "ALL");
}

#[test]
fn trial_eligibility_documents_note_follows_biodata_capture_provenance() {
    let provenance = crate::entities::trial::TrialEligibilityProvenance {
        source_kind: "registry".into(),
        source: "ClinicalTrials.gov registry".into(),
        posted_documents_available: true,
        documents_handle: Some("biomcp --json get trial NCT41300002 documents".into()),
    };
    let trial = sectioned_trial_fixture(Some(provenance));
    let markdown = trial_markdown(&trial, &["eligibility".into()]).expect("eligibility markdown");
    assert!(markdown.contains("**Posted trial documents:** Posted trial documents are available"));
    assert!(markdown.contains("`biomcp --json get trial NCT41300002 documents`"));
    assert!(!markdown.contains("central@example.test"));
    assert!(!markdown.contains("site@example.test"));
}

#[test]
fn trial_location_continuation_keeps_source_and_shell_safe_arguments() {
    let trial = preselected_trial_with_25_locations();
    let ordinary =
        trial_response_markdown(&trial, &["locations".into()]).expect("ordinary continuation");
    assert!(
        ordinary.contains("Next: `biomcp get trial NCT41300001 --offset 20 --limit 20 locations`")
    );

    let nci = trial_location_continuation_command(
        &trial,
        Some(crate::entities::trial::TrialSource::NciCts),
        20,
        20,
        true,
    )
    .expect("NCI continuation");
    assert_eq!(
        nci,
        "biomcp get trial NCT41300001 --source nci --offset 20 --limit 20 contacts locations"
    );
    let nci_card = trial_response_markdown(
        &preselected_trial_with_25_locations_from("NCI CTS"),
        &["locations".into()],
    )
    .expect("NCI continuation card");
    assert!(nci_card.contains(
        "Next: `biomcp get trial NCT41300001 --source nci --offset 20 --limit 20 locations`"
    ));
    let unknown_card = trial_response_markdown(
        &preselected_trial_with_25_locations_from("Unknown Provider"),
        &["locations".into()],
    )
    .expect("unknown-source card");
    assert!(!unknown_card.contains("\nNext:"));
    assert!(
        crate::next_command::NextCommand::biomcp()
            .args(["get", "trial"])
            .arg("NCT id` ;&")
            .render_shell()
            .contains("\"NCT id\\` ;&\"")
    );
}
