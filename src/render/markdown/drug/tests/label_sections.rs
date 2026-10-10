use super::*;

// Ticket 1300 review: the Markdown short-form pointer must quote multi-word
// drug names with the repo's shell quoting helper and may only print when
// the capped Markdown view actually omits label content.
fn label_drug(name: &str, boxed_warning: Option<String>) -> Drug {
    Drug {
        section_outcomes: crate::entities::drug::default_drug_section_outcomes(),
        name: name.to_string(),
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
        interaction_coverage_status: None,
        ddinter_synonyms: Vec::new(),
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: Some(crate::entities::drug::DrugLabel {
            indication_summary: vec![crate::entities::drug::DrugLabelIndication {
                name: "melanoma".to_string(),
                approval_date: None,
                pivotal_trial: None,
            }],
            indications: None,
            boxed_warning,
            warnings: None,
            dosage: None,
        }),
        label_set_id: Some("pointer-set-123".to_string()),
        shortage: None,
        approvals: None,
        fda_orphan_designations: None,
        us_safety_warnings: None,
        us_boxed_warning: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        who_note: None,
        civic: None,
        cell_lines: None,
    }
}

#[test]
fn label_pointer_stays_silent_when_nothing_was_cut() {
    let drug = label_drug("gefitinib", Some("WARNING: short boxed text.".to_string()));

    let markdown = drug_markdown_with_region(&drug, &["label".to_string()], DrugRegion::Us, false)
        .expect("markdown");
    assert!(markdown.contains("### Approved Indications"));
    assert!(
        !markdown.contains("Markdown caps label sections"),
        "{markdown}"
    );
}

#[test]
fn label_pointer_prints_when_the_view_cut_a_section() {
    let drug = label_drug(
        "gefitinib",
        Some(format!(
            "WARNING: {}\n\nSeek urgent care.",
            "w".repeat(2500)
        )),
    );

    let markdown = drug_markdown_with_region(&drug, &["label".to_string()], DrugRegion::Us, false)
        .expect("markdown");
    assert!(
        markdown.contains("Markdown caps label sections at a short form. Use `biomcp get drug gefitinib label --json` for whole sections, or `--raw` for the raw label text."),
        "{markdown}"
    );
    assert!(markdown.contains("(truncated, "));
}

#[test]
fn label_pointer_quotes_multi_word_drug_names() {
    let drug = label_drug(
        "trastuzumab deruxtecan",
        Some(format!(
            "WARNING: {}\n\nSeek urgent care.",
            "w".repeat(2500)
        )),
    );

    let markdown = drug_markdown_with_region(&drug, &["label".to_string()], DrugRegion::Us, false)
        .expect("markdown");
    assert!(
        markdown.contains("biomcp get drug \"trastuzumab deruxtecan\" label --json"),
        "{markdown}"
    );
    assert!(
        !markdown.contains("biomcp get drug trastuzumab deruxtecan label"),
        "{markdown}"
    );
}

#[test]
fn missing_label_reason_prints_from_the_label_section_outcome() {
    let mut drug = label_drug("adavosertib", None);
    drug.label = None;
    drug.label_set_id = None;
    drug.section_outcomes.complete(
        "label",
        crate::entities::section_outcome::SectionOutcome::empty_with_reason(
            "OpenFDA label",
            "No openFDA SPL label record matched this drug.",
        ),
    );

    let markdown = drug_markdown_with_region(&drug, &["label".to_string()], DrugRegion::Us, false)
        .expect("markdown");
    assert!(
        markdown.contains("## FDA Label\n\nNo openFDA SPL label record matched this drug."),
        "{markdown}"
    );
}

#[test]
fn a_card_without_label_content_prints_no_fda_label_heading() {
    // Ticket 2047: every drug card printed an empty "## FDA Label" heading
    // on the default card (CLI and MCP render the same template) because the
    // heading gated only on the section being shown. The heading now prints
    // only when label content, a label status, or a missing-label reason
    // exists; v0.9.1 printed no heading at all.
    let mut drug = label_drug("pertuzumab, trastuzumab, and hyaluronidase-zzxf", None);
    drug.label = None;
    drug.label_set_id = None;

    let markdown = drug_markdown_with_region(&drug, &[], DrugRegion::Us, false).expect("markdown");
    assert!(!markdown.contains("## FDA Label"), "{markdown}");

    // A requested label section that honestly found nothing keeps the
    // heading, because the reason line is the section's content.
    drug.section_outcomes.complete(
        "label",
        crate::entities::section_outcome::SectionOutcome::empty_with_reason(
            "OpenFDA label",
            "No openFDA SPL label record matched this drug.",
        ),
    );
    let markdown = drug_markdown_with_region(
        &drug,
        &["label".to_string()],
        DrugRegion::Us,
        false,
    )
    .expect("markdown");
    assert!(markdown.contains("## FDA Label"), "{markdown}");
}
