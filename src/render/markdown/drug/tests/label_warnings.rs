use super::*;

fn warning_drug() -> Drug {
    Drug {
        section_outcomes: crate::entities::drug::default_drug_section_outcomes(),
        name: "pembrolizumab".to_string(),
        drugbank_id: None,
        chembl_id: None,
        unii: None,
        drug_type: None,
        mechanism: None,
        mechanisms: Vec::new(),
        approval_date: None,
        approval_date_raw: None,
        approval_date_display: None,
        approval_summary: None,
        brand_names: Vec::new(),
        route: None,
        targets: Vec::new(),
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        interaction_pagination: None,
        interaction_bundle_freshness: None,
        pharm_classes: Vec::new(),
        top_adverse_events: vec!["Rash".to_string()],
        faers_query: None,
        label: Some(crate::entities::drug::DrugLabel {
            indication_summary: vec![crate::entities::drug::DrugLabelIndication {
                name: "melanoma".to_string(),
                approval_date: None,
                pivotal_trial: None,
            }],
            indications: None,
            boxed_warning: Some("WARNING: SERIOUS SKIN REACTIONS".to_string()),
            warnings: Some("Immune-mediated adverse reactions.".to_string()),
            dosage: None,
        }),
        label_set_id: Some("warning-set-123".to_string()),
        shortage: None,
        approvals: None,
        fda_orphan_designations: None,
        us_safety_warnings: Some("Immune-mediated adverse reactions.".to_string()),
        us_boxed_warning: Some("WARNING: SERIOUS SKIN REACTIONS".to_string()),
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
        cell_lines: None,
    }
}

#[test]
fn drug_markdown_renders_label_boxed_warning_ahead_of_other_label_sections() {
    let drug = warning_drug();

    let raw = drug_markdown_with_region(&drug, &["label".to_string()], DrugRegion::Us, true)
        .expect("markdown");
    let boxed_at = raw
        .find("### Boxed Warning\nWARNING: SERIOUS SKIN REACTIONS")
        .expect("boxed block in raw mode");
    let warnings_at = raw
        .find("### Warnings")
        .expect("warnings heading in raw mode");
    assert!(boxed_at < warnings_at);
    assert!(raw.contains("Immune-mediated adverse reactions."));

    let summary = drug_markdown_with_region(&drug, &["label".to_string()], DrugRegion::Us, false)
        .expect("markdown");
    let boxed_at = summary
        .find("### Boxed Warning\nWARNING: SERIOUS SKIN REACTIONS")
        .expect("boxed block in summary mode");
    let indications_at = summary
        .find("### Approved Indications")
        .expect("indications heading in summary mode");
    assert!(boxed_at < indications_at);
}

#[test]
fn drug_markdown_us_safety_block_renders_boxed_warning_first() {
    let drug = warning_drug();
    let markdown = drug_markdown_with_region(&drug, &["safety".to_string()], DrugRegion::Us, false)
        .expect("markdown");
    let boxed_at = markdown
        .find("### Boxed Warning\nWARNING: SERIOUS SKIN REACTIONS")
        .expect("boxed subsection in safety block");
    let warnings_at = markdown
        .find("### Warnings")
        .expect("warnings subsection in safety block");
    assert!(boxed_at < warnings_at);
    assert!(markdown.contains("Immune-mediated adverse reactions."));
}

#[test]
fn combined_label_and_safety_render_one_boxed_warning_heading() {
    let drug = warning_drug();
    let markdown = drug_markdown_with_region(
        &drug,
        &["label".to_string(), "safety".to_string()],
        DrugRegion::Us,
        false,
    )
    .expect("markdown");

    assert_eq!(markdown.matches("### Boxed Warning").count(), 1);
    assert_eq!(
        markdown.matches("WARNING: SERIOUS SKIN REACTIONS").count(),
        1
    );
}

#[test]
fn truncated_safety_warning_links_exact_dailymed_set_when_label_is_absent() {
    let mut drug = warning_drug();
    drug.label = None;
    drug.us_boxed_warning = None;
    drug.us_safety_warnings = Some(
        "Warning text.\n\n(truncated, 2007 chars total; full label: https://dailymed.nlm.nih.gov/dailymed/drugInfo.cfm?setid=warning-set-123)"
            .to_string(),
    );

    let markdown = drug_markdown_with_region(&drug, &["safety".to_string()], DrugRegion::Us, false)
        .expect("markdown");
    assert!(markdown.contains(
        "(truncated, 2007 chars total; full label: https://dailymed.nlm.nih.gov/dailymed/drugInfo.cfm?setid=warning-set-123)"
    ));
    assert!(markdown.contains(
        "[DailyMed](https://dailymed.nlm.nih.gov/dailymed/drugInfo.cfm?setid=warning-set-123)"
    ));
}
