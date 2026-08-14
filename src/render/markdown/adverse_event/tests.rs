use super::*;

fn sample_vaers_payload(
    status: crate::entities::adverse_event::VaersSearchStatus,
) -> crate::entities::adverse_event::VaersSearchPayload {
    crate::entities::adverse_event::VaersSearchPayload {
        status,
        message: Some("VAERS message".to_string()),
        matched_vaccine: Some(crate::entities::adverse_event::VaersMatchedVaccine {
            display_name: "MMR".to_string(),
            wonder_code: "MMR".to_string(),
            cvx_codes: vec!["03".to_string(), "94".to_string()],
        }),
        summary: Some(crate::entities::adverse_event::VaersSearchSummary {
            total_reports: 83_359,
            serious_reports: 5_795,
            non_serious_reports: 77_564,
            age_distribution: vec![crate::entities::adverse_event::VaersAgeBucket {
                age_bucket: "< 6 months".to_string(),
                reports: 536,
                percentage: 0.64,
            }],
            top_reactions: vec![crate::entities::adverse_event::VaersReactionCount {
                reaction: "ABASIA".to_string(),
                count: 179,
                percentage: 0.21,
            }],
        }),
    }
}

#[test]
fn adverse_event_markdown_includes_openfda_sections() {
    let event = AdverseEvent {
        report_id: "10329882".to_string(),
        drug: "ivacaftor".to_string(),
        reactions: vec!["Cough".to_string()],
        outcomes: vec!["Hospitalization".to_string()],
        patient: None,
        concomitant_medications: vec!["azithromycin".to_string()],
        reporter_type: None,
        reporter_country: None,
        indication: None,
        serious: true,
        date: Some("2024-01-01".to_string()),
    };

    let markdown = adverse_event_markdown(&event, &["all".to_string()]).expect("faers");
    assert!(markdown.contains("Source: OpenFDA"));
    assert!(markdown.contains("## Reactions (OpenFDA)"));
    assert!(markdown.contains("## Outcomes (OpenFDA)"));
    assert!(markdown.contains("## Concomitant Drugs (OpenFDA)"));
}

#[test]
fn adverse_event_subset_markdown_keeps_identity_and_only_selected_bodies() {
    let event = AdverseEvent {
        report_id: "10329882".to_string(),
        drug: "drug with space".to_string(),
        reactions: vec!["Cough".to_string()],
        outcomes: vec!["Hospitalization".to_string()],
        patient: Some("Adult".to_string()),
        concomitant_medications: vec!["azithromycin".to_string()],
        reporter_type: Some("Physician".to_string()),
        reporter_country: Some("US".to_string()),
        indication: Some("lung cancer".to_string()),
        serious: true,
        date: Some("2024-01-01".to_string()),
    };

    let markdown = adverse_event_markdown(
        &event,
        &[
            "guidance".to_string(),
            "reactions".to_string(),
            "guidance".to_string(),
        ],
    )
    .expect("subset markdown");

    assert!(markdown.contains("Report ID: 10329882"));
    assert!(markdown.contains("Drug: drug with space"));
    assert!(markdown.contains("## Reactions (OpenFDA)"));
    assert!(!markdown.contains("## Outcomes (OpenFDA)"));
    assert!(!markdown.contains("## Concomitant Drugs (OpenFDA)"));
    assert!(!markdown.contains("Patient: Adult"));
    assert!(!markdown.contains("Reporter:"));
    assert!(!markdown.contains("More:"));
    assert!(!markdown.contains("Related:"));
    for command in adverse_event_guidance_commands(&event) {
        assert!(
            markdown.contains(&format!("`{command}`")),
            "missing {command}: {markdown}"
        );
        assert_eq!(
            markdown.matches(&command).count(),
            1,
            "duplicate {command}: {markdown}"
        );
    }
}

#[test]
fn adverse_event_search_markdown_renders_summary_and_filters() {
    let summary = AdverseEventSearchSummary {
        total_reports: 12,
        returned_report_count: 1,
        top_reactions: vec![
            crate::entities::adverse_event::AdverseEventReactionSummary {
                reaction: "Cough".to_string(),
                count: 4,
                percentage: 33.3,
            },
        ],
    };
    let results = vec![AdverseEventSearchResult {
        report_id: "1001".to_string(),
        drug: "ivacaftor".to_string(),
        reactions: vec!["Cough".to_string()],
        serious: true,
    }];

    let markdown = adverse_event_search_markdown("ivacaftor", &results, &summary).expect("search");
    assert!(markdown.contains("# Adverse Events: ivacaftor"));
    assert!(markdown.contains("## Summary (OpenFDA FAERS sample)"));
    assert!(markdown.contains(
        "Top reactions: returned-report sample only; Percent = count / returned reports, not population frequency."
    ));
    assert!(markdown.contains("| Cough | 4 | 33.3% |"));
    assert!(markdown.contains("Use `get adverse-event <report_id>` for details."));
}

#[test]
fn adverse_event_search_markdown_can_label_aggregate_summary() {
    let summary = AdverseEventSearchSummary {
        total_reports: 20,
        returned_report_count: 1,
        top_reactions: vec![
            crate::entities::adverse_event::AdverseEventReactionSummary {
                reaction: "Rash".to_string(),
                count: 5,
                percentage: 25.0,
            },
        ],
    };
    let results = vec![AdverseEventSearchResult {
        report_id: "1001".to_string(),
        drug: "ivacaftor".to_string(),
        reactions: vec!["Cough".to_string()],
        serious: true,
    }];

    let markdown = adverse_event_search_markdown_with_source_label(
        "drug=ivacaftor",
        &results,
        &summary,
        "",
        None,
        &[],
        None,
        "OpenFDA FAERS aggregate",
    )
    .expect("aggregate summary");

    assert!(markdown.contains("## Summary (OpenFDA FAERS aggregate)"));
    assert!(
        markdown.contains("Top reactions: aggregate count query; Percent = count / total reports.")
    );
    assert!(markdown.contains("| Rash | 5 | 25% |") || markdown.contains("| Rash | 5 | 25.0% |"));
    assert!(
        markdown.contains("| Rash | 5 | 25.0% |\n\n|Report ID|")
            || markdown.contains("| Rash | 5 | 25% |\n\n|Report ID|")
            || markdown.contains("| Rash | 5 | 25.0% |\n\n\n|Report ID|")
            || markdown.contains("| Rash | 5 | 25% |\n\n\n|Report ID|"),
        "summary table should be visually separated from the per-report list: {markdown}"
    );
}

#[test]
fn adverse_event_search_markdown_renders_contextual_empty_state() {
    let markdown = adverse_event_search_markdown_with_context(
        "drug=daraxonrasib",
        &[],
        &AdverseEventSearchSummary {
            total_reports: 0,
            returned_report_count: 0,
            top_reactions: Vec::new(),
        },
        "",
        Some("Drug not found in FAERS. FAERS is a post-marketing database."),
        &[],
        None,
    )
    .expect("empty state");

    assert!(markdown.contains("Drug not found in FAERS. FAERS is a post-marketing database."));
    assert!(!markdown.contains("## Summary"));
}

#[test]
fn adverse_event_search_markdown_renders_trial_fallback_section() {
    let markdown = adverse_event_search_markdown_with_context(
        "drug=daraxonrasib",
        &[],
        &AdverseEventSearchSummary {
            total_reports: 0,
            returned_report_count: 0,
            top_reactions: Vec::new(),
        },
        "",
        Some("Drug not found in FAERS. FAERS is a post-marketing database."),
        &[
            crate::entities::adverse_event::TrialAdverseEventTerm {
                term: "Rash".to_string(),
                trial_count: 2,
            },
            crate::entities::adverse_event::TrialAdverseEventTerm {
                term: "Fatigue".to_string(),
                trial_count: 1,
            },
        ],
        Some("daraxonrasib"),
    )
    .expect("trial fallback");

    assert!(markdown.contains("## Trial-Reported Adverse Events (ClinicalTrials.gov)"));
    assert!(markdown.contains("| Rash | 2 |"));
    assert!(markdown.contains("Source: ClinicalTrials.gov trial results"));
}

#[test]
fn adverse_event_count_markdown_renders_bucket_rows() {
    let markdown = adverse_event_count_markdown(
        "drug=ivacaftor",
        "reaction",
        &[AdverseEventCountBucket {
            value: "Cough".to_string(),
            count: 7,
        }],
    )
    .expect("count markdown");

    assert!(markdown.contains("# Adverse Event Counts"));
    assert!(markdown.contains("Count field: reaction"));
    assert!(markdown.contains("Counted rows shown: 7"));
    assert!(markdown.contains("| Cough | 7 | 100.0% |"));
}

#[test]
fn device_event_renderers_include_openfda_content() {
    let event = DeviceEvent {
        report_id: "MDR-123".to_string(),
        report_number: None,
        device: "Infusion Pump".to_string(),
        manufacturer: Some("Example".to_string()),
        event_type: Some("Malfunction".to_string()),
        date: Some("2024-02-01".to_string()),
        description: Some("Description text.".to_string()),
    };

    let markdown = device_event_markdown(&event).expect("device");
    assert!(markdown.contains("Source: OpenFDA"));
    assert!(markdown.contains("## Description (OpenFDA)"));

    let search_markdown = device_event_search_markdown(
        "pump",
        &[DeviceEventSearchResult {
            report_id: event.report_id.clone(),
            device: event.device.clone(),
            event_type: event.event_type.clone(),
            date: event.date.clone(),
            description: event.description.clone(),
        }],
    )
    .expect("device search");
    assert!(search_markdown.contains("# Device Events: pump"));
    assert!(search_markdown.contains("|Report Key|Device|Event Type|Date|Description|"));
}

#[test]
fn recall_search_markdown_renders_result_table() {
    let markdown = recall_search_markdown(
        "insulin pump",
        &[RecallSearchResult {
            recall_number: "Z-1234-2025".to_string(),
            classification: "Class II".to_string(),
            product_description: "Infusion pump cartridge".to_string(),
            reason_for_recall: "Leak risk".to_string(),
            status: "Ongoing".to_string(),
            distribution_pattern: None,
            recall_initiation_date: Some("2025-01-01".to_string()),
        }],
    )
    .expect("recall search");

    assert!(markdown.contains("# Recalls"));
    assert!(markdown.contains("|Recall #|Classification|Product|Status|"));
    assert!(markdown.contains("|Z-1234-2025|Class II|Infusion pump cartridge|Ongoing|"));
}

#[test]
fn combined_adverse_event_search_markdown_appends_vaers_summary_for_unavailable_status() {
    let markdown = combined_adverse_event_search_markdown(
        "drug=MMR vaccine",
        &[],
        &AdverseEventSearchSummary {
            total_reports: 0,
            returned_report_count: 0,
            top_reactions: Vec::new(),
        },
        "",
        Some("Drug not found in FAERS. FAERS is a post-marketing database."),
        Some(&sample_vaers_payload(
            crate::entities::adverse_event::VaersSearchStatus::Unavailable,
        )),
    )
    .expect("combined markdown");

    assert!(markdown.contains("## CDC VAERS Summary"));
    assert!(markdown.contains("Status: unavailable"));
    assert!(markdown.contains("Source: CDC VAERS"));
}

#[test]
fn combined_adverse_event_search_markdown_skips_query_not_vaccine_status() {
    let markdown = combined_adverse_event_search_markdown(
        "drug=ibuprofen",
        &[],
        &AdverseEventSearchSummary {
            total_reports: 0,
            returned_report_count: 0,
            top_reactions: Vec::new(),
        },
        "",
        Some("Drug not found in FAERS. FAERS is a post-marketing database."),
        Some(&sample_vaers_payload(
            crate::entities::adverse_event::VaersSearchStatus::QueryNotVaccine,
        )),
    )
    .expect("combined markdown");

    assert!(!markdown.contains("## CDC VAERS Summary"));
}

#[test]
fn vaers_only_markdown_renders_snake_case_status_labels() {
    let markdown = vaers_only_markdown(
        "ibuprofen",
        &sample_vaers_payload(crate::entities::adverse_event::VaersSearchStatus::QueryNotVaccine),
    );

    assert!(markdown.contains("# Adverse Events: ibuprofen"));
    assert!(markdown.contains("Status: query_not_vaccine"));
    assert!(markdown.contains("VAERS message"));
    assert!(markdown.contains("Source: CDC VAERS"));
}
