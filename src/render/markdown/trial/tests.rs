use super::*;

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
        phase: Some("Phase 2".to_string()),
        study_type: Some("Interventional".to_string()),
        age_range: Some("18 Years and older".to_string()),
        conditions: vec!["cystic fibrosis".to_string()],
        interventions: vec!["ivacaftor".to_string()],
        intervention_details: Vec::new(),
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
            facility: "Example Hospital".to_string(),
            city: "Boston".to_string(),
            state: Some("MA".to_string()),
            country: "United States".to_string(),
            status: Some("Recruiting".to_string()),
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
    assert!(!markdown.contains("Posted trial documents"));
}

#[test]
fn trial_markdown_renders_contacts_eligibility_and_json_fields() {
    let mut trial = crate::entities::trial::Trial {
        nct_id: "NCT41300001".to_string(),
        source: Some("ClinicalTrials.gov".to_string()),
        title: "Contact trial".to_string(),
        status: "Recruiting".to_string(),
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
            facility: "Rare Disease Center".to_string(),
            city: "Ann Arbor".to_string(),
            state: Some("Michigan".to_string()),
            country: "United States".to_string(),
            status: Some("Recruiting".to_string()),
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
    assert!(markdown.contains("## Contacts (ClinicalTrials.gov)"));
    assert!(markdown.contains("Central Contact"));
    assert!(markdown.contains("central@example.test"));
    assert!(markdown.contains("Sex: Female"));
    assert!(markdown.contains("Eligible Ages: 2 Years to 18 Years"));
    assert!(markdown.contains("site@example.test"));
    assert!(markdown.contains("Posted trial documents"));
    assert!(markdown.contains("may contain additional eligibility detail"));
    assert!(markdown.contains("biomcp --json get trial NCT41300001 documents"));

    let json = serde_json::to_value(&trial).expect("trial json");
    assert_eq!(json["contacts"][0]["email"], "central@example.test");
    assert_eq!(json["eligibility"]["sex"], "Female");
    assert_eq!(json["locations"][0]["contact_email"], "site@example.test");

    trial.eligibility_provenance = Some(crate::entities::trial::TrialEligibilityProvenance {
        source_kind: "registry".to_string(),
        source: "ClinicalTrials.gov registry".to_string(),
        posted_documents_available: false,
        documents_handle: None,
    });
    let markdown = trial_markdown(&trial, &["eligibility".to_string()]).expect("trial markdown");
    assert!(!markdown.contains("Posted trial documents"));
}
