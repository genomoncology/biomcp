use super::*;

#[test]
fn drug_markdown_uses_label_interaction_text_before_public_unavailable_fallback() {
    let drug = Drug {
        name: "warfarin".to_string(),
        drugbank_id: Some("DB00682".to_string()),
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
        interaction_text: Some("DRUG INTERACTIONS\n\nWarfarin interacts with aspirin.".to_string()),
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
    };

    let markdown = drug_markdown(&drug, &["interactions".to_string()]).expect("markdown");
    assert!(markdown.contains("## Interactions (DDInter)"));
    assert!(markdown.contains("DRUG INTERACTIONS"));
    assert!(!markdown.contains("No known drug-drug interactions found."));
}

#[test]
fn drug_markdown_uses_truthful_public_unavailable_interactions_message() {
    let drug = Drug {
        name: "pembrolizumab".to_string(),
        drugbank_id: Some("DB09037".to_string()),
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
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
    };

    let markdown = drug_markdown(&drug, &["interactions".to_string()]).expect("markdown");
    assert!(
        markdown
            .contains("The current DDInter download bundle has no matching rows for this drug.")
    );
    assert!(!markdown.contains("No known drug-drug interactions found."));
}

#[test]
fn drug_markdown_shows_target_family_and_members_when_present() {
    let drug = Drug {
        name: "olaparib".to_string(),
        drugbank_id: Some("DB09074".to_string()),
        chembl_id: Some("CHEMBL1789941".to_string()),
        unii: None,
        drug_type: Some("small molecule".to_string()),
        mechanism: Some("PARP inhibitor".to_string()),
        mechanisms: vec!["PARP inhibitor".to_string()],
        approval_date: None,
        approval_date_raw: None,
        approval_date_display: None,
        approval_summary: None,
        brand_names: Vec::new(),
        route: None,
        targets: vec![
            "PARP1".to_string(),
            "PARP2".to_string(),
            "PARP3".to_string(),
        ],
        variant_targets: Vec::new(),
        target_family: Some("PARP".to_string()),
        target_family_name: Some("poly(ADP-ribose) polymerase".to_string()),
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
    };

    let markdown = drug_markdown(&drug, &["targets".to_string()]).expect("markdown");
    assert!(markdown.contains("Family: PARP (poly(ADP-ribose) polymerase)"));
    assert!(markdown.contains("Members: PARP1, PARP2, PARP3"));
}

#[test]
fn drug_markdown_renders_variant_targets_as_additive_line() {
    let drug = Drug {
        name: "rindopepimut".to_string(),
        drugbank_id: None,
        chembl_id: Some("CHEMBL2108508".to_string()),
        unii: None,
        drug_type: Some("vaccine".to_string()),
        mechanism: None,
        mechanisms: Vec::new(),
        approval_date: None,
        approval_date_raw: None,
        approval_date_display: None,
        approval_summary: None,
        brand_names: Vec::new(),
        route: None,
        targets: vec!["EGFR".to_string()],
        variant_targets: vec!["EGFRvIII".to_string()],
        target_family: None,
        target_family_name: None,
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
    };

    let markdown = drug_markdown(&drug, &["targets".to_string()]).expect("markdown");
    assert!(markdown.contains("## Targets (ChEMBL / Open Targets)"));
    assert!(markdown.contains("EGFR"));
    assert!(markdown.contains("Variant Targets (CIViC): EGFRvIII"));
}

#[test]
fn drug_markdown_omits_target_family_for_mixed_targets() {
    let drug = Drug {
        name: "imatinib".to_string(),
        drugbank_id: Some("DB00619".to_string()),
        chembl_id: Some("CHEMBL941".to_string()),
        unii: None,
        drug_type: Some("small-molecule".to_string()),
        mechanism: Some("Inhibitor of BCR-ABL".to_string()),
        mechanisms: vec!["Inhibitor of BCR-ABL".to_string()],
        approval_date: None,
        approval_date_raw: None,
        approval_date_display: None,
        approval_summary: None,
        brand_names: Vec::new(),
        route: None,
        targets: vec!["ABL1".to_string(), "KIT".to_string(), "PDGFRB".to_string()],
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
    };

    let markdown = drug_markdown(&drug, &["targets".to_string()]).expect("markdown");
    assert!(!markdown.contains("Family:"));
    assert!(!markdown.contains("Members:"));
    assert!(markdown.contains("ABL1, KIT, PDGFRB"));
}

#[test]
fn drug_markdown_with_region_all_keeps_us_and_eu_blocks_separate() {
    let drug = Drug {
        name: "pembrolizumab".to_string(),
        drugbank_id: Some("DB09037".to_string()),
        chembl_id: None,
        unii: None,
        drug_type: None,
        mechanism: None,
        mechanisms: Vec::new(),
        approval_date: None,
        approval_date_raw: None,
        approval_date_display: None,
        approval_summary: None,
        brand_names: vec!["Keytruda".to_string()],
        route: None,
        targets: Vec::new(),
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        pharm_classes: Vec::new(),
        top_adverse_events: vec!["Rash".to_string()],
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: Some(vec![crate::entities::drug::DrugShortageEntry {
            status: Some("Current".to_string()),
            availability: Some("Limited".to_string()),
            company_name: Some("Example Pharma".to_string()),
            generic_name: Some("pembrolizumab".to_string()),
            related_info: Some("https://example.org/us-shortage".to_string()),
            update_date: Some("2026-01-13".to_string()),
            initial_posting_date: None,
        }]),
        approvals: Some(vec![DrugApproval {
            application_number: "BLA125514".to_string(),
            sponsor_name: Some("Merck Sharp & Dohme".to_string()),
            openfda_brand_names: vec!["Keytruda".to_string()],
            openfda_generic_names: vec!["pembrolizumab".to_string()],
            products: Vec::new(),
            submissions: Vec::new(),
        }]),
        us_safety_warnings: Some("Immune-mediated adverse reactions.".to_string()),
        ema_regulatory: Some(vec![EmaRegulatoryRow {
            medicine_name: "Keytruda".to_string(),
            active_substance: "pembrolizumab".to_string(),
            ema_product_number: "EMEA/H/C/003820".to_string(),
            status: "Authorised".to_string(),
            holder: Some("Merck Sharp & Dohme B.V.".to_string()),
            marketing_authorisation_date: Some("17/07/2015".to_string()),
            therapeutic_indication: Some(
                "Keytruda as monotherapy is indicated for the treatment of adults and adolescents aged 12 years and older with advanced (unresectable or metastatic) melanoma."
                    .to_string(),
            ),
            recent_activity: vec![crate::entities::drug::EmaRegulatoryActivity {
                first_published_date: "27/02/2026".to_string(),
                last_updated_date: None,
            }],
        }]),
        ema_safety: Some(EmaSafetyInfo {
            dhpcs: vec![crate::entities::drug::EmaDhpcEntry {
                medicine_name: "Keytruda".to_string(),
                dhpc_type: Some("DHPC".to_string()),
                regulatory_outcome: Some("Updated safety communication".to_string()),
                first_published_date: Some("15/01/2026".to_string()),
                last_updated_date: None,
            }],
            referrals: Vec::new(),
            psusas: Vec::new(),
        }),
        ema_shortage: Some(vec![EmaShortageEntry {
            medicine_affected: "Keytruda".to_string(),
            status: Some("Resolved".to_string()),
            availability_of_alternatives: Some("Yes".to_string()),
            first_published_date: Some("10/01/2026".to_string()),
            last_updated_date: Some("13/01/2026".to_string()),
        }]),
        who_prequalification: None,
        civic: None,
    };

    let markdown = drug_markdown_with_region(&drug, &["all".to_string()], DrugRegion::All, false)
        .expect("markdown");
    assert!(markdown.contains("## Regulatory (US - Drugs@FDA)"));
    assert!(markdown.contains("## Regulatory (EU - EMA)"));
    assert!(markdown.contains("## Safety (US - OpenFDA)"));
    assert!(markdown.contains("## Safety (EU - EMA)"));
    assert!(markdown.contains("## Shortage (US - OpenFDA Drug Shortages)"));
    assert!(markdown.contains("## Shortage (EU - EMA)"));
    assert!(markdown.contains("BLA125514"));
    assert!(markdown.contains("EMEA/H/C/003820"));
    assert!(
        markdown
            .contains("| Medicine | Active Substance | EMA Number | Status | Auth Date | Holder |")
    );
    assert!(markdown.contains("17/07/2015"));
    assert!(markdown.contains("### Authorized indications"));
    assert!(markdown.contains("advanced (unresectable or metastatic) melanoma"));
    assert!(markdown.contains("Immune-mediated adverse reactions."));
    assert!(markdown.contains("Resolved"));
}

#[test]
fn drug_markdown_with_region_who_renders_regulatory_block() {
    let drug = Drug {
        name: "trastuzumab".to_string(),
        drugbank_id: Some("DB00072".to_string()),
        chembl_id: None,
        unii: None,
        drug_type: None,
        mechanism: None,
        mechanisms: Vec::new(),
        approval_date: None,
        approval_date_raw: None,
        approval_date_display: None,
        approval_summary: None,
        brand_names: vec!["Herceptin".to_string()],
        route: None,
        targets: Vec::new(),
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: Some(vec![WhoPrequalificationEntry {
            kind: crate::entities::drug::WhoPrequalificationKind::FinishedPharma,
            who_reference_number: Some("BT-ON001".to_string()),
            inn: "Trastuzumab".to_string(),
            presentation: Some(
                "Trastuzumab Powder for concentrate for solution for infusion 150 mg".to_string(),
            ),
            dosage_form: Some("Powder for concentrate for solution for infusion".to_string()),
            product_type: "Biotherapeutic Product".to_string(),
            therapeutic_area: "Oncology".to_string(),
            applicant: "Samsung Bioepis NL B.V.".to_string(),
            listing_basis: Some("Prequalification - Abridged".to_string()),
            alternative_listing_basis: None,
            prequalification_date: Some("2019-12-18".to_string()),
            who_product_id: None,
            grade: None,
            confirmation_document_date: None,
            vaccine_type: None,
            commercial_name: None,
            dose_count: None,
            manufacturer: None,
            responsible_nra: None,
        }]),
        civic: None,
    };

    let markdown =
        drug_markdown_with_region(&drug, &["regulatory".to_string()], DrugRegion::Who, false)
            .expect("markdown");

    assert!(markdown.contains("## Regulatory (WHO Prequalification)"));
    assert!(markdown.contains("| WHO ID | Type | Presentation / INN |"));
    assert!(markdown.contains("BT-ON001"));
    assert!(markdown.contains("Samsung Bioepis NL B.V."));
    assert!(markdown.contains("2019-12-18"));
}

#[test]
fn drug_search_all_region_markdown_includes_who_block() {
    let markdown = drug_search_markdown_with_region(
        "trastuzumab",
        DrugRegion::All,
        &[crate::entities::drug::DrugSearchResult {
            name: "trastuzumab".to_string(),
            drugbank_id: None,
            mechanism: None,
            target: Some("ERBB2".to_string()),
            drug_type: None,
        }],
        Some(1),
        &[crate::entities::drug::EmaDrugSearchResult {
            name: "Herzuma".to_string(),
            active_substance: "trastuzumab".to_string(),
            ema_product_number: "EMEA/H/C/004123".to_string(),
            status: "Authorised".to_string(),
        }],
        Some(1),
        &[crate::entities::drug::WhoPrequalificationSearchResult {
            kind: crate::entities::drug::WhoPrequalificationKind::FinishedPharma,
            inn: "Trastuzumab".to_string(),
            product_type: "Biotherapeutic Product".to_string(),
            therapeutic_area: "Oncology".to_string(),
            presentation: Some(
                "Trastuzumab Powder for concentrate for solution for infusion 150 mg".to_string(),
            ),
            dosage_form: Some("Powder for concentrate for solution for infusion".to_string()),
            applicant: "Samsung Bioepis NL B.V.".to_string(),
            who_reference_number: Some("BT-ON001".to_string()),
            who_product_id: None,
            listing_basis: Some("Prequalification - Abridged".to_string()),
            prequalification_date: Some("2019-12-18".to_string()),
            vaccine_type: None,
            commercial_name: None,
            dose_count: None,
            manufacturer: None,
            responsible_nra: None,
        }],
        Some(1),
        "",
    )
    .expect("markdown");

    assert!(markdown.contains("## US (MyChem.info / OpenFDA)"));
    assert!(markdown.contains("## EU (EMA)"));
    assert!(markdown.contains("## WHO (WHO Prequalification)"));
    assert!(markdown.contains("BT-ON001"));
    assert!(markdown.contains("EMEA/H/C/004123"));
}

#[test]
fn drug_search_who_vaccine_markdown_uses_vaccine_table_and_no_get_footer() {
    let markdown = drug_search_markdown_with_region(
        "BCG",
        DrugRegion::Who,
        &[],
        None,
        &[],
        None,
        &[crate::entities::drug::WhoPrequalificationSearchResult {
            kind: crate::entities::drug::WhoPrequalificationKind::Vaccine,
            inn: "BCG".to_string(),
            product_type: "Vaccine".to_string(),
            therapeutic_area: "Vaccine".to_string(),
            presentation: Some("Ampoule".to_string()),
            dosage_form: None,
            applicant: "Japan BCG Laboratory".to_string(),
            who_reference_number: None,
            who_product_id: None,
            listing_basis: None,
            prequalification_date: Some("1987-01-01".to_string()),
            vaccine_type: Some("BCG".to_string()),
            commercial_name: Some("BCG Freeze Dried Glutamate vaccine".to_string()),
            dose_count: None,
            manufacturer: Some("Japan BCG Laboratory".to_string()),
            responsible_nra: Some("Pharmaceutical and Medical Devices Agency".to_string()),
        }],
        Some(1),
        "",
    )
    .expect("markdown");

    assert!(markdown.contains(
        "|Vaccine Type|Commercial Name|Presentation|Doses|Manufacturer|Responsible NRA|Date|"
    ));
    assert!(markdown.contains("BCG Freeze Dried Glutamate vaccine"));
    assert!(markdown.contains("|-|"));
    assert!(!markdown.contains("Use `get drug <name>` for full details."));
}

#[test]
fn drug_markdown_with_region_eu_all_suppresses_us_header_facts() {
    // Criterion 9: `get drug <name> all --region eu` must not show US-specific
    // header lines (FDA Approved, Safety FAERS) even though the full card is rendered.
    let drug = Drug {
        name: "pembrolizumab".to_string(),
        drugbank_id: Some("DB09037".to_string()),
        chembl_id: None,
        unii: None,
        drug_type: None,
        mechanism: None,
        mechanisms: Vec::new(),
        approval_date: Some("2014-09-04".to_string()),
        approval_date_raw: Some("20140904".to_string()),
        approval_date_display: Some("September 4, 2014".to_string()),
        approval_summary: Some("FDA approved September 4, 2014".to_string()),
        brand_names: vec!["Keytruda".to_string()],
        route: None,
        targets: Vec::new(),
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        pharm_classes: Vec::new(),
        top_adverse_events: vec!["Fatigue".to_string(), "Rash".to_string()],
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        us_safety_warnings: None,
        ema_regulatory: Some(vec![EmaRegulatoryRow {
            medicine_name: "Keytruda".to_string(),
            active_substance: "pembrolizumab".to_string(),
            ema_product_number: "EMEA/H/C/003820".to_string(),
            status: "Authorised".to_string(),
            holder: None,
            marketing_authorisation_date: None,
            therapeutic_indication: None,
            recent_activity: Vec::new(),
        }]),
        ema_safety: Some(EmaSafetyInfo {
            dhpcs: Vec::new(),
            referrals: Vec::new(),
            psusas: Vec::new(),
        }),
        ema_shortage: Some(Vec::new()),
        who_prequalification: None,
        civic: None,
    };

    let markdown = drug_markdown_with_region(&drug, &["all".to_string()], DrugRegion::Eu, false)
        .expect("markdown");

    // EU EMA section must be present
    assert!(markdown.contains("## Regulatory (EU - EMA)"));
    assert!(markdown.contains("EMEA/H/C/003820"));

    // US-specific header facts must be absent
    assert!(
        !markdown.contains("FDA Approved"),
        "US approval date must not appear in EU-only output"
    );
    assert!(
        !markdown.contains("Safety (OpenFDA FAERS)"),
        "US FAERS safety line must not appear in EU-only output"
    );
}

#[test]
fn drug_markdown_with_region_eu_safety_shows_truthful_empty_subsections() {
    let drug = Drug {
        name: "semaglutide".to_string(),
        drugbank_id: Some("DB13928".to_string()),
        chembl_id: None,
        unii: None,
        drug_type: None,
        mechanism: None,
        mechanisms: Vec::new(),
        approval_date: None,
        approval_date_raw: None,
        approval_date_display: None,
        approval_summary: None,
        brand_names: vec!["Ozempic".to_string()],
        route: None,
        targets: Vec::new(),
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: Some(EmaSafetyInfo {
            dhpcs: vec![crate::entities::drug::EmaDhpcEntry {
                medicine_name: "Ozempic".to_string(),
                dhpc_type: Some("DHPC".to_string()),
                regulatory_outcome: Some("Medicine shortage".to_string()),
                first_published_date: Some("10/01/2026".to_string()),
                last_updated_date: Some("13/01/2026".to_string()),
            }],
            referrals: Vec::new(),
            psusas: Vec::new(),
        }),
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
    };

    let markdown = drug_markdown_with_region(&drug, &["safety".to_string()], DrugRegion::Eu, false)
        .expect("markdown");
    assert!(markdown.contains("## Safety (EU - EMA)"));
    assert!(markdown.contains("### DHPCs"));
    assert!(markdown.contains("Medicine shortage"));
    assert!(markdown.contains("### Referrals"));
    assert!(markdown.contains("### PSUSAs"));
    assert!(markdown.contains("No data found (EMA)"));
}

#[test]
fn drug_search_empty_state_frames_zero_indication_miss_as_regulatory_signal() {
    let markdown = drug_search_markdown_with_region(
        "indication=Marfan syndrome",
        DrugRegion::Us,
        &[],
        Some(0),
        &[],
        None,
        &[],
        None,
        "",
    )
    .expect("markdown");

    assert!(markdown.contains("U.S. regulatory data"));
    assert!(markdown.contains("This absence is informative"));
    assert!(markdown.contains(
        "biomcp search article -k \"Marfan syndrome treatment\" --type review --limit 5"
    ));
    assert!(markdown.contains("Try: biomcp discover \"Marfan syndrome\""));
    assert!(!markdown.contains("No drugs found\n"));
}

#[test]
fn drug_search_standard_empty_state_includes_discover_hint() {
    let markdown = drug_search_markdown_with_footer("MK-3475", &[], Some(0), "").expect("markdown");

    assert!(markdown.contains("Try: biomcp discover MK-3475"));
}

#[test]
fn drug_search_eu_empty_state_includes_discover_hint() {
    let markdown = drug_search_markdown_with_region(
        "MK-3475",
        DrugRegion::Eu,
        &[],
        None,
        &[],
        Some(0),
        &[],
        None,
        "",
    )
    .expect("markdown");

    assert!(markdown.contains("Try: biomcp discover MK-3475"));
}

#[test]
fn drug_search_all_region_empty_state_calls_out_regulatory_absence() {
    let markdown = drug_search_markdown_with_region(
        "indication=Marfan syndrome",
        DrugRegion::All,
        &[],
        Some(0),
        &[],
        Some(0),
        &[],
        Some(0),
        "",
    )
    .expect("markdown");

    assert!(
        markdown.contains("specific to the structured regulatory portion of the combined search")
    );
    assert!(markdown.contains("## US (MyChem.info / OpenFDA)"));
    assert!(markdown.contains("## EU (EMA)"));
}

#[test]
fn drug_search_all_region_empty_state_includes_discover_only_when_both_regions_are_empty() {
    let empty_markdown = drug_search_markdown_with_region(
        "MK-3475",
        DrugRegion::All,
        &[],
        Some(0),
        &[],
        Some(0),
        &[],
        Some(0),
        "",
    )
    .expect("markdown");
    assert!(empty_markdown.contains("Try: biomcp discover MK-3475"));

    let us_only_markdown = drug_search_markdown_with_region(
        "MK-3475",
        DrugRegion::All,
        &[crate::entities::drug::DrugSearchResult {
            name: "pembrolizumab".to_string(),
            drugbank_id: None,
            mechanism: None,
            target: None,
            drug_type: None,
        }],
        Some(1),
        &[],
        Some(0),
        &[],
        Some(0),
        "",
    )
    .expect("markdown");
    assert!(!us_only_markdown.contains("Try: biomcp discover MK-3475"));

    let eu_only_markdown = drug_search_markdown_with_region(
        "MK-3475",
        DrugRegion::All,
        &[],
        Some(0),
        &[crate::entities::drug::EmaDrugSearchResult {
            name: "Keytruda".to_string(),
            active_substance: "pembrolizumab".to_string(),
            ema_product_number: "EMEA/H/C/003820".to_string(),
            status: "Authorized".to_string(),
        }],
        Some(1),
        &[],
        Some(0),
        "",
    )
    .expect("markdown");
    assert!(!eu_only_markdown.contains("Try: biomcp discover MK-3475"));
}
