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
        locations: Some(vec![crate::entities::trial::TrialLocation {
            facility: "Example Hospital".to_string(),
            city: "Boston".to_string(),
            state: Some("MA".to_string()),
            country: "United States".to_string(),
            status: Some("Recruiting".to_string()),
            contact_name: None,
            contact_phone: None,
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
}
