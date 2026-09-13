use super::*;

fn preselected_trial_with_25_locations() -> TrialResponse {
    let locations = (1..=25)
        .map(|number| {
            serde_json::json!({
                "facility": format!("Fixture Site {number:02}"),
                "city": format!("Fixture City {number:02}"),
                "state": "Michigan",
                "country": "United States",
                "status": "RECRUITING"
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
            "contactsLocationsModule": {"locations": locations}
        }
    })
    .to_string();
    let plan = biodata::ClinicalTrialsGovApiV2DetailPlan::new("NCT41300001", false)
        .expect("test plan")
        .with_locations();
    let response = biodata::ClinicalTrialsGovApiV2Response::parse(
        &plan,
        input.as_bytes(),
        &Default::default(),
    )
    .expect("test response");
    let mut trial = TrialResponse::new(
        response.into_projection().expect("test projection"),
        "ClinicalTrials.gov",
        None,
        crate::entities::trial::TrialSectionStates {
            arms: crate::entities::trial::TrialSectionState::NotRequested,
            eligibility: crate::entities::trial::TrialSectionState::NotRequested,
            outcomes: crate::entities::trial::TrialSectionState::NotRequested,
            references: crate::entities::trial::TrialSectionState::NotRequested,
            contacts: crate::entities::trial::TrialSectionState::NotRequested,
            locations: crate::entities::trial::TrialSectionState::Present,
        },
    );
    trial.set_site_page(0, 25);
    trial
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
